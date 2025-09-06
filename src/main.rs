#![feature(box_into_inner)]
mod interpreter;
mod lox;
mod parser;
mod scanner;
mod token;

use std::io::{self, BufRead, Write};

use interpreter::{AstInterpreter, Expr};
use lox::Lox;
use parser::Parser;
use scanner::Scanner;
use token::{Literal, Token};

fn main() {
    let expression = Expr::Binary {
        left: Box::new(Expr::Unary {
            operator: Token::new(token::TokenType::Minus, "-", None, 1),
            right: Box::new(Expr::Literal(Literal::Number(123.0))),
        }),
        operator: Token::new(token::TokenType::Star, "*", None, 1),
        right: Box::new(Expr::Grouping(Box::new(Expr::Literal(Literal::Number(
            45.67,
        ))))),
    };

    println!("{:?}", AstInterpreter.print(&expression));

    // let args: Vec<_> = std::env::args().skip(1).collect();
    // let mut lox = Lox::new();
    // match args.as_slice() {
    //     [] => run_prompt(&mut lox),
    //     [path] => run_file(&mut lox, path).unwrap(),
    //     _ => {
    //         eprintln!("Usage: jlox [script]");
    //         std::process::exit(64)
    //     }
    // }
}

fn run_file(lox: &mut Lox, path: &str) -> std::io::Result<()> {
    let source = std::fs::read_to_string(path).unwrap();
    run(lox, &source);
    Ok(())
}

fn run_prompt(lox: &mut Lox) {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        match lines.next() {
            None => break,
            Some(Ok(line)) => run(lox, &line),
            Some(Err(e)) => {
                eprintln!("{e}");
                break;
            }
        }
    }
}

fn run(lox: &mut Lox, source: &str) {
    let mut scanner = Scanner::new(source, lox);
    let tokens = scanner.scan_tokens();
    let mut parser = Parser::new(tokens.clone(), lox);
    let expression = parser.parse().unwrap();

    if lox.had_error {
        return;
    };

    println!("{:?}", AstInterpreter.print(&expression));

    // for token in tokens.iter() {
    //     println!("{token}")
    // }

    if lox.had_error {
        lox.clear_error();
        std::process::exit(65);
    }
}
