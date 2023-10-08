use std::fs;
use std::io;

mod assemble;
use assemble::AssembleError;

pub struct Config;

impl Config{
    pub fn make() -> Config{
        Config{}
    }
}

/// The error that gets returned to the caller from our run function
/// This should only be used to convey a message to the user
pub struct RunError{
    msg: String,
}

/// Convert an io error and give a helpful error message
impl From<io::Error> for RunError{
    fn from(e: io::Error) -> Self{
        Self{msg: String::from("Encountered an issue while attempting to read the file. Does the file exist or is it open in another process?")}
    }
}

/// Convert an AssembleError and give a helpful error message
/// Note that we can't implement the from trait here because we need to know
/// which instruction caused the error in order to form the message
impl RunError{
    fn from(e: AssembleError, badinst: &str) -> Self {
        let mut out = String::from("Encountered an error while parsing:\n\t");
        match e {
            AssembleError::UnknownOp => out.push_str("Use of unknown operation at: "),
            AssembleError::MissingArgs => out.push_str("Missing one or more arguments at: "),
            AssembleError::ExtraArgs => out.push_str("Use of too many arguments at: "),
            AssembleError::InvalidArg => out.push_str("Use of one or more invalid arguments at: "),
        }
        out.push_str(badinst);
        Self{msg: out}
    }
}

/// Run the assembler
pub fn run<'a>(config: Config) -> Result<(), RunError>{
    let data = fs::read_to_string("example.asm")?;
    let instructions = data.lines()
                    .map(|l| l.trim())
                    .filter(|l| !l.is_empty())
                    .filter(|l| !l.starts_with(";"))
                    .collect::<Vec<&str>>();

    let mut opcodes = Vec::with_capacity(instructions.len());

    for instruction in &instructions{
        let instruction = *instruction;
        match  assemble::assemble_instruction(instruction) {
            Ok(opcode) => opcodes.push(opcode),
            Err(e) => return Err(RunError::from(e, instruction)),
        }
    }

    println!("{:?}", opcodes);
    Ok(())
}