use std::num::ParseIntError;
use thiserror::Error;

/// An enum representing a possible argument passed to an operation in the assembly code
/// It's up to assemble.rs to make sure that the arguments make sense for any given operation
/// It's also up to assemble.rs to figure out a numeric arg represents and if it's valid
pub enum AsmArgument {
    Numeric(u16), //the largest possible numeric arg is a 12 bit address and all numeric args are unsigned
    Register(u8),
    AnyKey,
    IPointer,
    IRange,
    DelayTimer,
    SoundTimer,
    Sprite,
    Bcd,
}

/// An unit-like struct representing an error during any part of argument parsing
/// More precise error returns may be added at a later date
#[derive(Debug, Error)]
#[error("Error occurred while parsing argument for asm instruction")]
pub struct AsmArgParseError;

impl From<ParseIntError> for AsmArgParseError {
    fn from(_err: ParseIntError) -> Self {
        AsmArgParseError {}
    }
}

/// Given a collection of string slices, return parsed AsmArgument enums or error if one or more is invalid
pub fn parse_asm_args(args: &[&str]) -> Result<Vec<AsmArgument>, AsmArgParseError> {
    let mut out = Vec::with_capacity(args.len());
    for arg in args {
        match parse_asm_arg(arg) {
            Ok(asm_arg) => out.push(asm_arg),
            Err(err) => return Err(err),
        };
    }
    Ok(out)
}

/// Given a string slice, parse it into an AsmArgument if possible, otherwise error
fn parse_asm_arg(arg: &str) -> Result<AsmArgument, AsmArgParseError> {
    match arg {
        "K" | "k" => Ok(AsmArgument::AnyKey),
        "I" | "i" => Ok(AsmArgument::IPointer),
        "[I]" | "[i]" => Ok(AsmArgument::IRange),
        "DT" | "Dt" | "dT" | "dt" => Ok(AsmArgument::DelayTimer),
        "ST" | "St" | "sT" | "st" => Ok(AsmArgument::SoundTimer),
        "F" | "f" => Ok(AsmArgument::Sprite),
        "B" | "b" => Ok(AsmArgument::Bcd),
        _ => parse_numeric_asm_arg(arg),
    }
}

/// Given a string slice that can't be any other valid asm_arg, parse it into a valid numeric or register variant, otherwise error
fn parse_numeric_asm_arg(arg: &str) -> Result<AsmArgument, AsmArgParseError> {
    // register
    if arg.starts_with('V') || arg.starts_with('v') {
        if arg.len() != 2 {
            Err(AsmArgParseError {})
        } else {
            Ok(AsmArgument::Register(u8::from_str_radix(&arg[1..2], 16)?))
        }

    // other numeric arg in hex
    } else if let Some(hex_num) = arg.strip_prefix("0x") {
        Ok(AsmArgument::Numeric(u16::from_str_radix(hex_num, 16)?))

    // other numeric arg in binary
    } else if let Some(hex_num) = arg.strip_prefix("0b") {
        Ok(AsmArgument::Numeric(u16::from_str_radix(hex_num, 2)?))

    // other numeric arg in decimal
    } else {
        Ok(AsmArgument::Numeric(arg.parse::<u16>()?))
    }
}

/// Given an AsmArgument numeric variant, ensure that it represents a valid address and pass back the value
pub fn parse_valid_addr(arg: &AsmArgument) -> Result<u16, AsmArgParseError> {
    if let AsmArgument::Numeric(addr) = *arg {
        if addr <= 0xFFF {
            Ok(addr)
        } else {
            Err(AsmArgParseError {})
        }
    } else {
        panic!("parse_valid_addr called with invalid AsmArgument variant. If this happens a lot, consider using the type state pattern.");
    }
}

/// Given an AsmArgument numeric variant, ensure that it represents a valid byte and pass back the value
pub fn parse_valid_byte(arg: &AsmArgument) -> Result<u8, AsmArgParseError> {
    if let AsmArgument::Numeric(byte) = *arg {
        if byte <= 0xFF {
            Ok(byte as u8)
        } else {
            Err(AsmArgParseError {})
        }
    } else {
        panic!("parse_valid_byte called with invalid AsmArgument variant. If this happens a lot, consider using the type state pattern.");
    }
}

/// Given an AsmArgument numeric variant, ensure that it represents a valid nibble and pass back the value
pub fn parse_valid_nibble(arg: &AsmArgument) -> Result<u8, AsmArgParseError> {
    if let AsmArgument::Numeric(nibble) = *arg {
        if nibble <= 0xF {
            Ok(nibble as u8)
        } else {
            Err(AsmArgParseError {})
        }
    } else {
        panic!("parse_valid_nibble called with invalid AsmArgument variant. If this happens a lot, consider using the type state pattern.");
    }
}

/// Given a slice of string tokens, either convert from hex u16 or error
pub fn parse_raw(tokens: &[&str]) -> Result<u16, AsmArgParseError> {
    if tokens.len() != 1 || !tokens[0].starts_with("0x") {
        Err(AsmArgParseError)
    } else {
        let num = tokens[0].strip_prefix("0x").unwrap();
        Ok(u16::from_str_radix(num, 16)?)
    }
}
