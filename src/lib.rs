use std::fs;
use std::io::{self, Write, Read};
use std::path::PathBuf;

use clap::Parser;

mod assemble;
use assemble::AssembleError;

#[derive(Parser)]
#[command(name = "ch8asmcodechange")]
#[command(author = "Daniel Gysi <danielgysi@protonmail.com")]
#[command(about = "Basic assembler for the chip8 architecture")]
#[command(version, long_about=None)]
struct Args {
    /// The file from which to read the assembly instrucions to be assembled. If none is provided, stdin is used instead.
    #[arg(short, long)]
    input: Option<PathBuf>,
    /// The file into which the assembled bytes will be written. If none is provided, stdout is used instead.
    #[arg(short, long)]
    output: Option<PathBuf>,
}

/// An enum to represent the user's choice regarding output of assembled bytes
enum OutputConfig {
    Stdout,
    File(PathBuf),
}

/// An enum to represent the user's choice regarding input of assembly instructions
enum InputConfig {
    Stdin,
    File(PathBuf),
}

/// Represent the collection of choices made for how the assembler should be run
pub struct Config {
    input_config: InputConfig,
    output_config: OutputConfig,
}

impl Config {
    pub fn make() -> Config {
        let args = Args::parse();
        let input_config = match args.input {
            Some(f) => InputConfig::File(f),
            None => InputConfig::Stdin,
        };
        let output_config = match args.output {
            Some(f) => OutputConfig::File(f),
            None => OutputConfig::Stdout,
        };
        Config {
            input_config,
            output_config,
        }
    }
}

/// The error that gets returned to the caller from our run function
/// This should only be used to convey a message to the user
pub struct RunError {
    pub msg: String,
}

/// Convert an io error and give a helpful error message
impl From<io::Error> for RunError {
    fn from(_e: io::Error) -> Self {
        Self{msg: String::from("Encountered an issue while attempting to read or write to file. Does the file exist or is it open in another process?")}
    }
}

/// Convert an AssembleError and give a helpful error message
/// Note that we can't implement the from trait here because we need to know
/// which instruction caused the error in order to form the message
impl RunError {
    fn from(e: AssembleError, badinst: &str) -> Self {
        let mut out = String::from("Encountered an error while parsing:\n\t");
        match e {
            AssembleError::UnknownOp => out.push_str("Use of unknown operation at: "),
            AssembleError::MissingArgs => out.push_str("Missing one or more arguments at: "),
            AssembleError::ExtraArgs => out.push_str("Use of too many arguments at: "),
            AssembleError::InvalidArg => out.push_str("Use of one or more invalid arguments at: "),
        }
        out.push_str(badinst);
        Self { msg: out }
    }
}

/// Run the assembler
pub fn run(config: Config) -> Result<(), RunError> {

    // read our input
    let input_data = match config.input_config {
        InputConfig::Stdin => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            buf
        }
        InputConfig::File(f) => fs::read_to_string(f)?,
    };

    // process input into vec of instruction strings
    let instructions = input_data
        .lines()
        .map(|l| l.trim()) // remove leading and trailing whitespace
        .filter(|l| !l.is_empty()) // remove empty lines
        .filter(|l| !l.starts_with(';')) // remove comments
        .collect::<Vec<&str>>();

    // assemble instructions into individual opcodes
    // we need a for loop here in order to return a specific error
    let mut opcodes: Vec<u16> = Vec::with_capacity(instructions.len());

    for instruction in &instructions {
        let instruction = *instruction;
        match assemble::assemble_instruction(instruction) {
            Ok(opcode) => opcodes.push(opcode),
            Err(e) => return Err(RunError::from(e, instruction)),
        }
    }

    // convert opcodes into byte array in order to write rom
    let out_bytes = opcodes
        .into_iter()
        .map(|op| op.to_be_bytes())
        .flatten()
        .collect::<Vec<u8>>();

    // write to output
    match config.output_config {
        OutputConfig::File(f) => fs::write(f, out_bytes)?,
        OutputConfig::Stdout => io::stdout().write_all(&out_bytes)?
    };

    Ok(())
}
