use crate::token::{Token, TokenType};

pub struct Lox {
    pub had_error: bool,
}

impl Lox {
    pub fn new() -> Self {
        Self { had_error: false }
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

    pub fn clear_error(&mut self) {
        self.had_error = false;
    }
}
