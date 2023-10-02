use ch8asm::{self, Config};
use std::process;
fn main() {
    if let Err(err) = ch8asm::run(Config::make()){
        eprintln!("{}", err);
        process::exit(1);
    }
    process::exit(0);
}
