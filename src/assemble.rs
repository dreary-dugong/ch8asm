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
    let tokens = inst.split_whitespace()
                     .map(|t| t.trim_end_matches(',')) // commas are optional
                     .collect::<Vec<&str>>();

    match *tokens.first().expect("Attempt to parse empty string as instruction") {
        // TODO: check for too many args on cls and ret
        "CLS" => Ok(0x00E0),
        "RET" => Ok(0x00EE),

        "JP" | "jp" | "jP" | "Jp" => assemble_jp(&tokens),
        "LD" | "ld" | "lD" | "Ld" => assemble_ld(&tokens),


        _ => Err(AssembleError::UnknownOp),
    }
}

/// Given the tokens of a jp instrutction, return its machine code or an error
fn assemble_jp(tokens: &[&str]) -> Result<u16, AssembleError>{

    let args = parse::parse_asm_args(&tokens[1..])?;
    match args.len() {
        // JP addr - 1nnn
        1 => {
            let addr = parse::parse_valid_addr(&args[0])?;
            Ok(0x1000 + addr)
        }

        // JP V0, addr - Bnnn
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

/// Given the tokens of a LD instruction, return its machine code or an error
fn assemble_ld(tokens: &[&str]) -> Result<u16, AssembleError>{
    if tokens.len() < 3 {
        Err(AssembleError::MissingArgs)
    } else if tokens.len() > 3 {
        Err(AssembleError::ExtraArgs)
    } else {
        let args = parse::parse_asm_args(&tokens[1..])?;

        match (&args[0], &args[1]) {
            // LD Vx, Vy - 8xy0
            (AsmArgument::Register(vx), AsmArgument::Register(vy)) => {
                let mut out = 0x8000;
                let vx = *vx as u16;
                let vy = *vy as u16;
                out += vx << 8;
                out += vy << 4;
                Ok(out)
            },

            // LD Vx, byte - 6xkk
            (AsmArgument::Register(vx), AsmArgument::Numeric(_)) => {
                let mut out = 0x6000;
                let vx = *vx as u16;
                let byte = parse::parse_valid_byte(&args[1])? as u16;
                out += vx << 8;
                out += byte;
                Ok(out)
            },

            // LD I, addr - Annn
            (AsmArgument::IPointer, AsmArgument::Numeric(_)) => {
                let mut out = 0xA000;
                let addr = parse::parse_valid_addr(&args[1])?;
                out += addr;
                Ok(out)
            },

            // LD Vx, DT - Fx07
            (AsmArgument::Register(vx), AsmArgument::DelayTimer) => {
                let mut out = 0xF007;
                let vx = *vx as u16;
                out += vx << 8;
                Ok(out)
            },

            // LD Vx, K - Fx0A
            (AsmArgument::Register(vx), AsmArgument::AnyKey) => {
                let mut out = 0xF00A;
                let vx = *vx as u16;
                out += vx << 8;
                Ok(out)
            },

            // LD DT, Vx - Fx15
            (AsmArgument::DelayTimer, AsmArgument::Register(vx)) => {
                let mut out = 0xF015;
                let vx = *vx as u16;
                out += vx << 8;
                Ok(out)
            },

            // LD ST, Vx - Fx18
            (AsmArgument::SoundTimer, AsmArgument::Register(vx)) => {
                let mut out = 0xF018;
                let vx = *vx as u16;
                out += vx << 8;
                Ok(out)
            },

            // LD F, Vx - Fx29
            (AsmArgument::Sprite, AsmArgument::Register(vx)) => {
                let mut out = 0xF029;
                let vx = *vx as u16;
                out += vx << 8;
                Ok(out)
            },

            // LD B, Vx - Fx33
            (AsmArgument::BCD, AsmArgument::Register(vx)) => {
                let mut out = 0xF033;
                let vx = *vx as u16;
                out += vx << 8;
                Ok(out)
            },

            // LD [I], Vx - Fx55
            (AsmArgument::IRange, AsmArgument::Register(vx)) => {
                let mut out = 0xF055;
                let vx = *vx as u16;
                out += vx << 8;
                Ok(out)
            },

            // LD Vx, [I] - Fx65
            (AsmArgument::Register(vx), AsmArgument::IRange) => {
                let mut out = 0xF065;
                let vx = *vx as u16;
                out += vx << 8;
                Ok(out)
            }
            
            (_, _) => {
                Err(AssembleError::InvalidArg)
            }
        }

    }


}