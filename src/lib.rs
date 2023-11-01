use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

use clap::Parser;
use thiserror::Error;

mod preprocess;
use preprocess::PreprocessingError;
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
#[derive(Error, Debug)]
pub enum RunError {
    #[error("encounterd an issue while attempting to read or write file; does thie file exist? do you have permission? is it open in another process?")]
    IoError(#[from] io::Error),
    #[error("{0}")]
    Preprocessing(#[from] PreprocessingError),
    #[error("{0}")]
    Assemble(#[from] AssembleError),
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
    let instructions = preprocess::preprocess(&input_data)?;

    // assemble instructions into individual opcodes
    // we need a for loop here in order to return a specific error
    let mut opcodes: Vec<u16> = Vec::with_capacity(instructions.len());

    for instruction in &instructions {
        opcodes.push(assemble::assemble_instruction(instruction)?);
    }

    // convert opcodes into byte array in order to write rom
    let out_bytes = opcodes
        .into_iter()
        .flat_map(|op| op.to_be_bytes())
        .collect::<Vec<u8>>();

    // write to output
    match config.output_config {
        OutputConfig::File(f) => fs::write(f, out_bytes)?,
        OutputConfig::Stdout => io::stdout().write_all(&out_bytes)?,
    };

    Ok(())
}
