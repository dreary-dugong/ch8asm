use std::fs;

pub struct Config;

impl Config{
    pub fn make() -> Config{
        Config{}
    }
}

#[derive(Debug)]
enum ParseError<'a>{
    UnknownOp(&'a str),
    MissingArgs(&'a str),
    ExtraArgs(&'a str),
    InvalidParam(&'a str),
}
impl <'a> ParseError<'a>{
    fn to_string(&'a self) -> String{
        match self {
            Self::UnknownOp(inst) => format!("Attempted use of unknown operation at {}", inst),
            Self::MissingArgs(inst) => format!("Attempted use of operation with too few arguments: {}", inst),
            Self::ExtraArgs(inst) => format!("Attempted use of operation with too many arguments: {}", inst),
            Self::InvalidParam(inst) => format!("Attempted use of operation with one or more invalid arguments: {}", inst),
        }
    }
}

pub fn run(config: Config) -> Result<(), &'static str>{
    let data = fs::read_to_string("example.asm").unwrap();
    let lines = data.lines()
                    .map(|l| l.trim())
                    .filter(|l| !l.is_empty())
                    .filter(|l| !l.starts_with(";"))
                    .map(|inst| assemble_instruction(inst).unwrap())
                    .collect::<Vec<u16>>();
    println!("{:?}", lines);

    Ok(())
}

fn assemble_instruction<'a>(inst: &'a str) -> Result<u16, ParseError<'a>>{
    println!("{}", ParseError::UnknownOp(inst).to_string());
    Ok(0)
}