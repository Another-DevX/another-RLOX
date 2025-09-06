use crate::{
    interpreter::Expr,
    lox::Lox,
    token::{Literal, Token, TokenType},
};

type ParseFn<'lox> = for<'s> fn(&'s mut Parser<'lox>) -> Result<Expr, ParseError>;

#[derive(Debug, Clone, Copy)]
pub struct ParseError;

pub struct Parser<'a> {
    tokens: Vec<Token>,
    current: usize,
    lox: &'a mut Lox,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token>, lox: &'a mut Lox) -> Self {
        Self {
            tokens,
            current: 0,
            lox,
        }
    }

    pub fn parse(&mut self) -> Option<Expr> {
        match self.expression() {
            Ok(expr) => Some(expr),
            Err(err) => None,
        }
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        return self.equality();
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        self.left_binop(
            Self::comparison,
            &[TokenType::BangEqual, TokenType::EqualEqual],
        )
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        self.left_binop(
            Self::term,
            &[
                TokenType::Greater,
                TokenType::GreaterEqual,
                TokenType::Less,
                TokenType::LessEqual,
            ],
        )
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        self.left_binop(Self::factor, &[TokenType::Minus, TokenType::Plus])
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        self.left_binop(Self::unary, &[TokenType::Slash, TokenType::Star])
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self._match(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous().unwrap().clone();
            let right = self.unary()?;
            return Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            });
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self._match(&[TokenType::False]) {
            return Ok(Expr::Literal(Literal::Bool(false)));
        };
        if self._match(&[TokenType::True]) {
            return Ok(Expr::Literal(Literal::Bool(true)));
        };
        if self._match(&[TokenType::Nil]) {
            return Ok(Expr::Literal(Literal::Nil));
        };

        if self._match(&[TokenType::Number, TokenType::String]) {
            return Ok(Expr::Literal(
                self.previous().unwrap().literal.clone().unwrap(),
            ));
        };

        if self._match(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            let _ = self.consume(TokenType::RightParen, "Expect ')' after expression.");
            return Ok(Expr::Grouping(Box::new(expr)));
        }

        let error = {
            let token = self.peek().unwrap().clone();
            self.error(&token, "Expect expression.")
        };

        Err(error)
    }

    fn consume(&mut self, kind: TokenType, message: &str) -> Result<&Token, ParseError> {
        if self.check(kind) {
            return Ok(self.advance().unwrap());
        }
        let token = self.peek().unwrap().clone();
        let error = self.error(&token, message);
        Err(error)
    }

    fn error(&mut self, token: &Token, message: &str) -> ParseError {
        self.lox.error_at(token, message);
        ParseError
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if matches!(self.previous().map(|t| t.kind), Some(TokenType::Semicolon)) {
                return;
            }

            match self.peek().map(|t| t.kind) {
                Some(
                    TokenType::Class
                    | TokenType::Fun
                    | TokenType::Var
                    | TokenType::For
                    | TokenType::If
                    | TokenType::While
                    | TokenType::Print
                    | TokenType::Return,
                ) => return,
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn left_binop(&mut self, next: ParseFn<'a>, kinds: &[TokenType]) -> Result<Expr, ParseError> {
        let mut expr = next(self)?;
        while self._match(kinds) {
            let operator = self.previous().unwrap().clone();
            let right = next(self)?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn _match(&mut self, kinds: &[TokenType]) -> bool {
        for kind in kinds.iter() {
            if self.check(*kind) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn advance(&mut self) -> Option<&Token> {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn check(&mut self, kind: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        };
        self.peek().map(|t| t.kind == kind).unwrap_or(false)
    }

    fn peek(&mut self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn previous(&mut self) -> Option<&Token> {
        if self.current > 0 {
            self.tokens.get(self.current - 1)
        } else {
            None
        }
    }

    fn is_at_end(&mut self) -> bool {
        self.peek()
            .map(|t| t.kind == TokenType::Eof)
            .unwrap_or(false)
    }
}
