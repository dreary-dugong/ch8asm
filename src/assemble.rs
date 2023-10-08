mod parse;
use parse::{AsmArgument, AsmArgParseError};

/// An error that occured while parsing the assembly string
#[derive(Debug)]
pub enum AssembleError{
    UnknownOp,
    MissingArgs,
    ExtraArgs,
    InvalidArg,
}

impl From<AsmArgParseError> for AssembleError{
    fn from(_e: AsmArgParseError) -> Self {
        Self::InvalidArg
    }
}

/// For a line of assembly, emit its machine code
pub fn assemble_instruction(inst: &str) -> Result<u16, AssembleError>{
    let tokens = inst.split_whitespace().collect::<Vec<&str>>();

    match *tokens.first().expect("Attempt to parse empty string as instruction") {
        "CLS" => Ok(0x00E0),
        "RET" => Ok(0x00EE),

        "JP" => assemble_jp(&tokens),


        _ => Err(AssembleError::UnknownOp),
    }
}

/// Given the tokens of a jp instrutction, return its machine code or an error
fn assemble_jp(tokens: &[&str]) -> Result<u16, AssembleError>{

    let args = parse::parse_asm_args(&tokens[1..])?;
    match args.len() {
        1 => {
            let addr = parse::parse_valid_addr(&args[0])?;
            Ok(0x1000 + addr)
        }

        2 => {
            match args[0] {
                AsmArgument::Register(0u8) =>  {
                    let addr = parse::parse_valid_addr(&args[1])?;
                    Ok(0xB000 + addr)
                }
                _ => Err(AssembleError::InvalidArg)
            }
        }

        0 => Err(AssembleError::MissingArgs),
        _ => Err(AssembleError::ExtraArgs)
    }
}