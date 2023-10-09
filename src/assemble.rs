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

        "SYS" | "sYs" | "Sys" | "syS" | "SYs" | "sYS" | "SyS" | "sys" => assemble_sys(&tokens),
        "CALL" | "call" => assemble_call(&tokens),
        "SE" | "sE" | "Se" | "se" => assemble_se(&tokens),
        "SNE" | "snE" | "sNe" | "Sne" | "SNe" | "SnE" | "sNE" | "sne" => assemble_sne(&tokens),
        "ADD" | "adD" | "aDd" | "Add" | "ADd" | "AdD" | "aDD" | "add" => assemble_add(&tokens),

        "OR" | "or" | "oR" | "Or" => assemble_or(&tokens),
        "AND" | "anD" | "aNd" | "And" | "ANd" | "AnD" | "aND" | "and" => assemble_and(&tokens),
        "XOR" | "xoR" | "xOr" | "Xor" | "XOr" | "XoR" | "xOR" | "xor" => assemble_xor(&tokens),

        "SUB" | "suB" | "sUb" | "Sub" | "SUb" | "SuB" | "sUB" | "sub" => assemble_sub(&tokens),
        "SUBN" | "subn" => assemble_subn(&tokens),

        "SHR" | "shR" | "sHr" | "Shr" | "SHr" | "ShR" | "sHR" | "shr" => assemble_shr(&tokens),
        "SHL" | "shL" | "sHl" | "Shl" | "SHl" | "ShL" | "sHL" | "shl" => assemble_shl(&tokens),

        "RND" | "rnD" | "rNd" | "Rnd" | "RNd" | "RnD" | "rND" | "rnd" => assemble_rnd(&tokens),
        "DRW" | "drW" | "dRw" | "Drw" | "DRw" | "DrW" | "dRW" | "drw" => assemble_drw(&tokens),

        "SKP" | "skP" | "sKp" | "Skp" | "SKp" | "SkP" | "sKP" | "skp" => assemble_skp(&tokens),
        "SKNP" | "sknp" => assemble_sknp(&tokens),


        other => {
            if other.starts_with("0x") && tokens.len() == 1{
                Ok(parse::parse_raw(&tokens)?)
            } else {
                Err(AssembleError::UnknownOp)
            }
        }
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

/// Given the tokens of a SYS instruction, return its machine code or an error
// SYS addr - 0nnn
fn assemble_sys(tokens: &[&str]) -> Result<u16, AssembleError>{
    if tokens.len() < 2 {
        Err(AssembleError::MissingArgs)
    } else if tokens.len() > 2 {
        Err(AssembleError::ExtraArgs)
    } else {
        let args = parse::parse_asm_args(&tokens[1..])?;
        if let AsmArgument::Numeric(_) = args[0] {
            let addr = parse::parse_valid_addr(&args[0])?;
            Ok(0x0000 + addr)
        } else {
            Err(AssembleError::InvalidArg)
        }
    }
}

/// Given the tokens of a CALL instruction, return its machine code or an error
// CALL addr - 2nnn
fn assemble_call(tokens: &[&str]) -> Result<u16, AssembleError>{
    if tokens.len() < 2 {
        Err(AssembleError::MissingArgs)
    } else if tokens.len() > 2 {
        Err(AssembleError::ExtraArgs)
    } else {
        let args = parse::parse_asm_args(&tokens[1..])?;
        if let AsmArgument::Numeric(_) = args[0] {
            let addr = parse::parse_valid_addr(&args[0])?;
            Ok(0x2000 + addr)
        } else {
            Err(AssembleError::InvalidArg)
        }
    }
}

/// Given the tokens of a SE instruction, return its machine code or an error
fn assemble_se(tokens: &[&str]) -> Result<u16, AssembleError>{
    if tokens.len() < 3 {
        Err(AssembleError::MissingArgs)
    } else if tokens.len() > 3 {
        Err(AssembleError::ExtraArgs)
    } else {
        let args = parse::parse_asm_args(&tokens[1..])?;

        match (&args[0], &args[1]) {
            // SE Vx, byte - 3xkk
            (AsmArgument::Register(vx), AsmArgument::Numeric(_)) => {
                let mut out = 0x3000;
                let vx = *vx as u16;
                out += vx << 8;
                out += parse::parse_valid_byte(&args[1])? as u16;
                Ok(out)
            }
            // SE Vx, Vy - 5xy0
            (AsmArgument::Register(vx), AsmArgument::Register(vy)) => {
                let mut out = 0x5000;
                let vx = *vx as u16;
                let vy = *vy as u16;
                out += vx << 8;
                out += vy << 4;
                Ok(out)
            }

             (_, _) => {
                Err(AssembleError::InvalidArg)
             }
        }
    }
}

/// Given the tokens of a SNE instruction, return its machine code or an error
fn assemble_sne(tokens: &[&str]) -> Result<u16, AssembleError>{
    if tokens.len() < 3 {
        Err(AssembleError::MissingArgs)
    } else if tokens.len() > 3 {
        Err(AssembleError::ExtraArgs)
    } else {
        let args = parse::parse_asm_args(&tokens[1..])?;

        match (&args[0], &args[1]) {
            // SNE Vx, byte - 4xkk
            (AsmArgument::Register(vx), AsmArgument::Numeric(_)) => {
                let mut out = 0x4000;
                let vx = *vx as u16;
                out += vx << 8;
                out += parse::parse_valid_byte(&args[1])? as u16;
                Ok(out)
            }
            // SNE Vx, Vy - 9xy0
            (AsmArgument::Register(vx), AsmArgument::Register(vy)) => {
                let mut out = 0x9000;
                let vx = *vx as u16;
                let vy = *vy as u16;
                out += vx << 8;
                out += vy << 4;
                Ok(out)
            }

             (_, _) => {
                Err(AssembleError::InvalidArg)
             }
        }
    }
}

/// Given the tokens of a ADD instruction, return its machine code or an error
fn assemble_add(tokens: &[&str]) -> Result<u16, AssembleError>{
    if tokens.len() < 3 {
        Err(AssembleError::MissingArgs)
    } else if tokens.len() > 3 {
        Err(AssembleError::ExtraArgs)
    } else {
        let args = parse::parse_asm_args(&tokens[1..])?;

        match (&args[0], &args[1]) {
            // ADD Vx, byte - 7xkk
            (AsmArgument::Register(vx), AsmArgument::Numeric(_)) => {
                let mut out = 0x7000;
                let vx = *vx as u16;
                out += vx << 8;
                out += parse::parse_valid_byte(&args[1])? as u16;
                Ok(out)
            }
            // ADD Vx, Vy - 8xy4
            (AsmArgument::Register(vx), AsmArgument::Register(vy)) => {
                let mut out = 0x8004;
                let vx = *vx as u16;
                let vy = *vy as u16;
                out += vx << 8;
                out += vy << 4;
                Ok(out)
            }
            // ADD I, Vx - Fx1E 
            (AsmArgument::IPointer, AsmArgument::Register(vx)) => {
                let mut out = 0xF01E;
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

/// Given the tokens of a OR instruction, return its machine code or an error
// OR Vx, Vy - 8xy1
fn assemble_or(tokens: &[&str]) -> Result<u16, AssembleError>{
    if tokens.len() < 3 {
        Err(AssembleError::MissingArgs)
    } else if tokens.len() > 3 {
        Err(AssembleError::ExtraArgs)
    } else {
        let args = parse::parse_asm_args(&tokens[1..])?;
        if let (AsmArgument::Register(vx), AsmArgument::Register(vy)) = (&args[0], &args[1]) {
            let vx = *vx as u16;
            let vy = *vy as u16;
            Ok(0x8001 + (vx << 8) + (vy << 4))
        } else {
            Err(AssembleError::InvalidArg)
        }
    }
}

/// Given the tokens of a AND instruction, return its machine code or an error
// OR Vx, Vy - 8xy2
fn assemble_and(tokens: &[&str]) -> Result<u16, AssembleError>{
    if tokens.len() < 3 {
        Err(AssembleError::MissingArgs)
    } else if tokens.len() > 3 {
        Err(AssembleError::ExtraArgs)
    } else {
        let args = parse::parse_asm_args(&tokens[1..])?;
        if let (AsmArgument::Register(vx), AsmArgument::Register(vy)) = (&args[0], &args[1]) {
            let vx = *vx as u16;
            let vy = *vy as u16;
            Ok(0x8002 + (vx << 8) + (vy << 4))
        } else {
            Err(AssembleError::InvalidArg)
        }
    }
}

/// Given the tokens of a XOR instruction, return its machine code or an error
// OR Vx, Vy - 8xy3
fn assemble_xor(tokens: &[&str]) -> Result<u16, AssembleError>{
    if tokens.len() < 3 {
        Err(AssembleError::MissingArgs)
    } else if tokens.len() > 3 {
        Err(AssembleError::ExtraArgs)
    } else {
        let args = parse::parse_asm_args(&tokens[1..])?;
        if let (AsmArgument::Register(vx), AsmArgument::Register(vy)) = (&args[0], &args[1]) {
            let vx = *vx as u16;
            let vy = *vy as u16;
            Ok(0x8003 + (vx << 8) + (vy << 4))
        } else {
            Err(AssembleError::InvalidArg)
        }
    }
}

/// Given the tokens of a SUB instruction, return its machine code or an error
// SUB Vx, Vy - 8xy5
fn assemble_sub(tokens: &[&str]) -> Result<u16, AssembleError>{
    if tokens.len() < 3 {
        Err(AssembleError::MissingArgs)
    } else if tokens.len() > 3 {
        Err(AssembleError::ExtraArgs)
    } else {
        let args = parse::parse_asm_args(&tokens[1..])?;
        if let (AsmArgument::Register(vx), AsmArgument::Register(vy)) = (&args[0], &args[1]) {
            let vx = *vx as u16;
            let vy = *vy as u16;
            Ok(0x8005 + (vx << 8) + (vy << 4))
        } else {
            Err(AssembleError::InvalidArg)
        }
    }
}

/// Given the tokens of a XOR instruction, return its machine code or an error
// SUBN Vx, Vy - 8xy7
fn assemble_subn(tokens: &[&str]) -> Result<u16, AssembleError>{
    if tokens.len() < 3 {
        Err(AssembleError::MissingArgs)
    } else if tokens.len() > 3 {
        Err(AssembleError::ExtraArgs)
    } else {
        let args = parse::parse_asm_args(&tokens[1..])?;
        if let (AsmArgument::Register(vx), AsmArgument::Register(vy)) = (&args[0], &args[1]) {
            let vx = *vx as u16;
            let vy = *vy as u16;
            Ok(0x8007 + (vx << 8) + (vy << 4))
        } else {
            Err(AssembleError::InvalidArg)
        }
    }
}

/// Given the tokens of a SHR instruction, return its machine code or an error
// SHR Vx {, Vy} - 8xy6
fn assemble_shr(tokens: &[&str]) -> Result<u16, AssembleError>{
    let args = parse::parse_asm_args(&tokens[1..])?;

    match args.len() {
        // the second arg is optional
        1 => {
            if let AsmArgument::Register(vx) = &args[0] {
                let vx = *vx as u16;
                Ok(0x8006 + (vx << 8))
            } else {
                Err(AssembleError::InvalidArg)
            }
        },
        
        2 => {
            if let (AsmArgument::Register(vx), AsmArgument::Register(vy)) = (&args[0], &args[1]){
                let vx = *vx as u16;
                let vy = *vy as u16;
                Ok(0x8006 + (vx << 8) + (vy << 4))
            } else {
                Err(AssembleError::InvalidArg)
            }
        }

        0 => Err(AssembleError::MissingArgs),
        _ => Err(AssembleError::ExtraArgs)
    }
}

/// Given the tokens of a SHL instruction, return its machine code or an error
// SHL Vx {, Vy} - 8xyE
fn assemble_shl(tokens: &[&str]) -> Result<u16, AssembleError>{
    let args = parse::parse_asm_args(&tokens[1..])?;

    match args.len() {
        // the second arg is optional
        1 => {
            if let AsmArgument::Register(vx) = &args[0] {
                let vx = *vx as u16;
                Ok(0x800E + (vx << 8))
            } else {
                Err(AssembleError::InvalidArg)
            }
        },
        
        2 => {
            if let (AsmArgument::Register(vx), AsmArgument::Register(vy)) = (&args[0], &args[1]){
                let vx = *vx as u16;
                let vy = *vy as u16;
                Ok(0x800E + (vx << 8) + (vy << 4))
            } else {
                Err(AssembleError::InvalidArg)
            }
        }

        0 => Err(AssembleError::MissingArgs),
        _ => Err(AssembleError::ExtraArgs)
    }
}

/// Given the tokens of a RND instruction, return its machine code or an error
// RND Vx, byte - Cxkk
fn assemble_rnd(tokens: &[&str]) -> Result<u16, AssembleError>{
    if tokens.len() < 3 {
        Err(AssembleError::MissingArgs)
    } else if tokens.len() > 3 {
        Err(AssembleError::ExtraArgs)
    } else {
        let args = parse::parse_asm_args(&tokens[1..])?;
        if let (AsmArgument::Register(vx), AsmArgument::Numeric(_)) = (&args[0], &args[1]) {
            let vx = *vx as u16;
            let byte = parse::parse_valid_byte(&args[1])? as u16;
            Ok(0xC000 + (vx << 8) + byte)
        } else {
            Err(AssembleError::InvalidArg)
        }
    }
}

/// Given the tokens of a DRW instruction, return its machine code or an error
// DRW Vx, Vy, nibble - Dxyn
fn assemble_drw(tokens: &[&str]) -> Result<u16, AssembleError>{
    if tokens.len() < 4 {
        Err(AssembleError::MissingArgs)
    } else if tokens.len() > 4 {
        Err(AssembleError::ExtraArgs)
    } else {
        let args = parse::parse_asm_args(&tokens[1..])?;
        if let (AsmArgument::Register(vx), AsmArgument::Register(vy), AsmArgument::Numeric(_)) = (&args[0], &args[1], &args[2]) {
            let vx = *vx as u16;
            let vy = *vy as u16;
            let nibble = parse::parse_valid_nibble(&args[2])? as u16;
            Ok(0xC000 + (vx << 8) + (vy << 4) + nibble)
        } else {
            Err(AssembleError::InvalidArg)
        }
    }
}

/// Given the tokens of a SKP instruction, return its machine code or an error
// SKP Vx - Ex9E
fn assemble_skp(tokens: &[&str]) -> Result<u16, AssembleError>{
    if tokens.len() < 2 {
        Err(AssembleError::MissingArgs)
    } else if tokens.len() > 2 {
        Err(AssembleError::ExtraArgs)
    } else {
        let args = parse::parse_asm_args(&tokens[1..])?;
        if let AsmArgument::Register(vx) = &args[0]{
            let vx = *vx as u16;
            Ok(0xE09E + (vx << 8))
        } else {
            Err(AssembleError::InvalidArg)
        }
    }
}

/// Given the tokens of a SKNP instruction, return its machine code or an error
// SKNP Vx - ExA1
fn assemble_sknp(tokens: &[&str]) -> Result<u16, AssembleError>{
    if tokens.len() < 2 {
        Err(AssembleError::MissingArgs)
    } else if tokens.len() > 2 {
        Err(AssembleError::ExtraArgs)
    } else {
        let args = parse::parse_asm_args(&tokens[1..])?;
        if let AsmArgument::Register(vx) = &args[0]{
            let vx = *vx as u16;
            Ok(0xE0A1 + (vx << 8))
        } else {
            Err(AssembleError::InvalidArg)
        }
    }
}