/// An enum reprsenting a possible argument passed to an operation in the assembly code
/// It's up to assemble.rs to make sure that the arguments make sense for any given operation
/// It's also up to assemble.rs to figure out a numeric arg represents and if it's valid
pub enum AsmArgument{
    Numeric(u16), //the largest possible numeric arg is a 12 bit address and all numeric args are unsigned
    Register(u8),
    AnyKey,
    IPointer,
    DelayTimer,
    SoundTimer,
}

/// An unit-like struct representing an error during any part of argument parsing
/// More precise error returns may be added at a later date
pub struct AsmArgParseError;

impl From<ParseIntError> for AsmArgParseError {
    fn from(err: ParseIntError) -> Self {
        AsmArgParseError{}
    }
}

/// Given a collection of string slices, return parsed AsmArgument enums or error if one or more is invalid
pub fn parse_asm_args(args: &[&str]) -> Result<Vec<AsmArgument>, AsmArgParseError> {
    let out = Vec::with_capacity(args.len());
    for arg in args {
        match parse_asm_arg(arg) {
            Ok(asmArg) => out.push(asmArg),
            Err(err) => return Err(err),
        };
    }
    Ok(out);
}

/// Given a string slice, parse it into an AsmArgument if possible, otherwise error
fn parse_asm_arg(arg: &str) -> Result<AsmArgument, AsmArgParseError>{
    match arg {
        "K" | "k" => Ok(Argument::AnyKey),
        "I" | "i" => Ok(Argument::IPointer),
        "DT" | "Dt" | "dT" | "dt" => Ok(Argument::DelayTimer),
        "ST" | "St" | "sT" | "st" => Ok(Argument::SoundTimer),
        _ => parse_numeric_asm_arg(arg),
    }
}

/// Given a string slice that can't be any other valid asm_arg, parse it into a valid numeric or register variant, otherwise error
fn parse_numeric_asm_arg(arg: &str) -> Result<AsmArgument, AsmArgParseError> {
    // register
    if arg.starts_with("V") || arg.starts_with("v") {
        if arg.len != 2 {
            Err(AsmArgPaseError{})
        }
        Ok(AsmArgument::Register(u8::from_str_radix(arg[1], 16)?))
    
    // other numeric arg in hex
    } else if arg.starts_with("0x") {
        if arg.len() < 3 {
            Err(AsmArgParseError{})
        }
        Ok(AsmArgument::Numeric(u16::from_str_radix(arg[2..], 16)?))

    // other numeric arg in decimal
    } else {
        Ok(AsmArgument::Numeric(u16::from_str_radix(arg, 10)?))
    }
}

/// Given an AsmArgument numeric variant, ensure that it represents a valid address and pass back the value
fn parse_valid_addr(arg: AsmArgument) -> Result<u16, AsmArgParseError> {
    if let Numeric(addr) = arg {
        if addr <= 0xFFF {
            Ok(addr)
        } else {
            Err(AsmArgParseError{})
        }

    } else {
        panic!("parse_valid_addr called with invalid AsmArgument variant. If this happens a lot, consider using the type state pattern.");
    }
}

/// Given an AsmArgument numeric variant, ensure that it represents a valid byte and pass back the value
fn parse_valid_byte(arg: AsmArgument) -> Result<u8, AsmArgParseError> {
    if let Numeric(byte) = arg {
        if byte <= 0xFF {
            Ok(byte as u8)
        } else {
            Err(AsmArgParseError{})
        }

    } else {
        panic!("parse_valid_byte called with invalid AsmArgument variant. If this happens a lot, consider using the type state pattern.");
    }
}

/// Given an AsmArgument numeric variant, ensure that it represents a valid nibble and pass back the value
fn parse_valid_nibble(arg: AsmArgument) -> Result<u8, AsmArgParseError> {
    if let Numeric(nibble) = arg {
        if nibble <= 0xF {
            Ok(nibble as u8)
        } else {
            Err(AsmArgParseError{})
        }

    } else {
        panic!("parse_valid_nibble called with invalid AsmArgument variant. If this happens a lot, consider using the type state pattern.");
    }
}






