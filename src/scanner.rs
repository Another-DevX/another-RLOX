use std::{collections::HashMap, sync::LazyLock};

use crate::{
    lox::Lox,
    token::{Literal, Token, TokenType},
};

pub struct Scanner<'a> {
    source: String,
    start: usize,
    current: usize,
    line: usize,
    tokens: Vec<Token>,
    lox: &'a mut Lox,
}

pub static KEYWORDS: LazyLock<HashMap<&str, TokenType>> = std::sync::LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("and", TokenType::And);
    m.insert("class", TokenType::Class);
    m.insert("else", TokenType::Else);
    m.insert("false", TokenType::False);
    m.insert("fun", TokenType::Fun);
    m.insert("for", TokenType::For);
    m.insert("if", TokenType::If);
    m.insert("nil", TokenType::Nil);
    m.insert("or", TokenType::Or);
    m.insert("print", TokenType::Print);
    m.insert("return", TokenType::Return);
    m.insert("super", TokenType::Super);
    m.insert("this", TokenType::This);
    m.insert("true", TokenType::True);
    m.insert("var", TokenType::Var);
    m.insert("while", TokenType::While);
    m
});

impl<'a> Scanner<'a> {
    pub fn new(source: impl Into<String>, lox: &'a mut Lox) -> Scanner<'a> {
        Scanner {
            source: source.into(),
            start: 0,
            current: 0,
            line: 1,
            tokens: vec![],
            lox,
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens
            .push(Token::new(TokenType::Eof, "", None, self.line));

        std::mem::take(&mut self.tokens)
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        use TokenType::*;
        match c {
            '(' => self.add_token(LeftParen, None),
            ')' => self.add_token(RightParen, None),
            '{' => self.add_token(LeftBrace, None),
            '}' => self.add_token(RightBrace, None),
            ',' => self.add_token(Comma, None),
            '.' => self.add_token(Dot, None),
            '-' => self.add_token(Minus, None),
            '+' => self.add_token(Plus, None),
            ';' => self.add_token(Semicolon, None),
            '*' => self.add_token(Star, None),
            '!' => {
                let kind = if self.match_char('=') {
                    BangEqual
                } else {
                    Bang
                };
                self.add_token(kind, None);
            }
            '=' => {
                let kind = if self.match_char('=') {
                    EqualEqual
                } else {
                    Equal
                };
                self.add_token(kind, None);
            }
            '<' => {
                let kind = if self.match_char('=') {
                    LessEqual
                } else {
                    Less
                };
                self.add_token(kind, None);
            }
            '>' => {
                let kind = if self.match_char('=') {
                    GreaterEqual
                } else {
                    Greater
                };
                self.add_token(kind, None);
            }
            '/' => {
                if self.match_char('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(Slash, None);
                }
            }
            ' ' => {}
            '\r' => {}
            '\t' => {}
            '\n' => {
                self.line += 1;
            }
            '"' => self.string(),
            _ => {
                if c.is_ascii_digit() {
                    self.number();
                } else if c.is_alphabetic() {
                    self.identifier();
                } else {
                    self.lox.error(self.line, "Unexpected character");
                }
            }
        }
    }

    fn add_token(&mut self, kind: TokenType, literal: Option<Literal>) {
        let text = &self.source[self.start..self.current];
        self.tokens.push(Token::new(kind, text, literal, self.line))
    }

    #[inline]
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    #[inline]
    fn peek(&self) -> char {
        self.source[self.current..].chars().next().unwrap_or('\0')
    }

    #[inline]
    fn peek_next(&self) -> char {
        let mut it = self.source[self.current..].chars();
        let _ = it.next(); // salta actual
        it.next().unwrap_or('\0')
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.peek() == expected {
            self.current += expected.len_utf8();
            return true;
        }
        false
    }

    /// Avanza un carácter y lo devuelve. Si está al final, retorna '\0'.
    #[inline]
    fn advance(&mut self) -> char {
        let ch = self.peek();
        if ch != '\0' {
            self.current += ch.len_utf8(); // mover cursor en bytes
        }
        ch
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
                self.advance();
            }
        }

        if self.is_at_end() {
            self.lox.error(self.line, "Unterminated string.");
            return;
        }
        self.advance();
        let value = &self.source[self.start + 1..self.current - 1];
        self.add_token(TokenType::String, Some(Literal::Str(value.to_string())));
    }

    fn number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let value = &self.source[self.start..self.current];
        self.add_token(
            TokenType::Number,
            Some(Literal::Number(value.parse().unwrap_or(0.0))),
        );
    }

    fn identifier(&mut self) {
        while self.peek().is_alphanumeric() {
            self.advance();
        }
        let text = &self.source[self.start..self.current];
        let kind = KEYWORDS.get(text).copied().unwrap_or(TokenType::Identifier);
        self.add_token(kind, None);
    }
}
