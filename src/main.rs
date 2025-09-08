#![feature(box_into_inner)]
mod interpreter;
mod lox;
mod parser;
mod scanner;
mod token;

use lox::Lox;

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let mut lox = Lox::new();
    match args.as_slice() {
        [] => lox.run_prompt(),
        [path] => lox.run_file(path).unwrap(),
        _ => {
            eprintln!("Usage: jlox [script]");
            std::process::exit(64)
        }
    }
}
