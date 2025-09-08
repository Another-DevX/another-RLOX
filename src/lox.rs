use std::io::{self, BufRead, Write};

use crate::{
    interpreter::{Interpreter, RuntimeError},
    parser::Parser,
    scanner::Scanner,
    token::{Token, TokenType},
};

pub struct Lox {
    pub had_error: bool,
    pub had_runtime_error: bool,
}

impl Lox {
    pub fn new() -> Self {
        Self {
            had_error: false,
            had_runtime_error: false,
        }
    }

    pub fn run(&mut self, source: &str) {
        let mut scanner = Scanner::new(source, self);
        let mut interpreter = Interpreter::new();
        let tokens = scanner.scan_tokens();
        let mut parser = Parser::new(tokens.clone(), self);
        let statements = parser.parse().unwrap();

        if self.had_error {
            return;
        };

        println!("{:?}", interpreter.interpret(self, statements));

        if self.had_error {
            self.clear_error();
            std::process::exit(65);
        }

        if self.had_runtime_error {
            self.clear_error();
        }
    }

    pub fn run_file(&mut self, path: &str) -> std::io::Result<()> {
        let source = std::fs::read_to_string(path).unwrap();
        self.run(&source);

        if self.had_error {
            std::process::exit(65)
        }
        if self.had_runtime_error {
            std::process::exit(70)
        }
        Ok(())
    }

    pub fn run_prompt(&mut self) {
        let stdin = io::stdin();
        let mut lines = stdin.lock().lines();

        loop {
            print!("> ");
            io::stdout().flush().unwrap();
            match lines.next() {
                None => break,
                Some(Ok(line)) => self.run(&line),
                Some(Err(e)) => {
                    eprintln!("{e}");
                    break;
                }
            }
        }
    }

    pub fn error(&mut self, line: usize, message: &str) {
        self.report(line, "", message);
    }

    pub fn error_at(&mut self, token: &Token, message: &str) {
        if token.kind == TokenType::Eof {
            self.report(token.line, " at end", message);
        } else {
            let _where = format!(" at '{}'", &token.lexeme);
            self.report(token.line, _where.as_str(), message);
        }
    }

    fn report(&mut self, line: usize, where_: &str, message: &str) {
        eprintln!("[line {line}] Error{where_}: {message}");
        self.had_error = true;
    }

    pub fn runtime_error(&mut self, error: RuntimeError) {
        eprintln!("{} \n[line {} ]", error.message, error.token.line);
        self.had_runtime_error = true;
    }

    pub fn clear_error(&mut self) {
        self.had_error = false;
    }
}
