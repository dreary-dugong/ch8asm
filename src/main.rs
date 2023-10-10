use ch8asm::Config;
use std::process;

fn main() {
    if let Err(err) = ch8asm::run(Config::make()) {
        eprintln!("{}", err.msg);
        process::exit(1);
    }
    process::exit(0);
}
