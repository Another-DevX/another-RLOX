use crate::{
    lox::Lox,
    token::{Literal, Token, TokenType},
};

pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping(Box<Expr>),
    Literal(Literal),
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
}

#[derive(Clone)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    Str(String),
}

pub struct RuntimeError {
    pub token: Token,
    pub message: String,
}

impl RuntimeError {
    pub fn new(token: Token, message: &str) -> Self {
        RuntimeError {
            token,
            message: message.to_string(),
        }
    }
}

pub struct Interpreter;

impl Interpreter {
    pub fn new() -> Self {
        Interpreter
    }

    pub fn interpret(&mut self, lox: &mut Lox, expr: Expr) {
        match self.evaluate(&expr) {
            Ok(value) => {
                println!("{}", self.stringify(value));
            }
            Err(error) => lox.runtime_error(error),
        };
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal(value) => Ok(value.clone().into()),
            Expr::Grouping(inner) => self.evaluate(inner),
            Expr::Unary { operator, right } => {
                let right = self.evaluate(right)?;

                match operator.kind {
                    TokenType::Minus => {
                        if let Value::Number(n) = right {
                            Ok(Value::Number(-n))
                        } else {
                            Err(self.number_operand_error(operator))
                        }
                    }
                    TokenType::Bang => Ok(Value::Bool(!self.is_truthy(right))),
                    _ => Ok(Value::Nil),
                }
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(left)?;
                let right = self.evaluate(right)?;

                match operator.kind {
                    TokenType::Minus => match (left, right) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Number(left - right))
                        }
                        _ => Err(self.number_operands_error(operator)),
                    },

                    TokenType::Plus => match (left, right) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Number(left + right))
                        }
                        (Value::Str(left), Value::Str(right)) => Ok(Value::Str(left + &right)),
                        _ => {
                            let error = RuntimeError::new(
                                operator.clone(),
                                "Operands must be two numbers or two strings.",
                            );
                            Err(error)
                        }
                    },

                    TokenType::Slash => match (left, right) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Number(left / right))
                        }
                        _ => Err(self.number_operands_error(operator)),
                    },

                    TokenType::Star => match (left, right) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Number(left * right))
                        }
                        _ => Err(self.number_operands_error(operator)),
                    },

                    TokenType::Greater => match (left, right) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Bool(left > right))
                        }
                        _ => Err(self.number_operands_error(operator)),
                    },

                    TokenType::GreaterEqual => match (left, right) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Bool(left >= right))
                        }
                        _ => Err(self.number_operands_error(operator)),
                    },

                    TokenType::Less => match (left, right) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Bool(left < right))
                        }
                        _ => Err(self.number_operands_error(operator)),
                    },

                    TokenType::LessEqual => match (left, right) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Bool(left <= right))
                        }
                        _ => Err(self.number_operands_error(operator)),
                    },

                    TokenType::BangEqual => Ok(Value::Bool(!self.is_equal(left, right))),

                    TokenType::EqualEqual => Ok(Value::Bool(self.is_equal(left, right))),

                    _ => Ok(Value::Nil),
                }
            }
        }
    }

    fn number_operand_error(&mut self, operator: &Token) -> RuntimeError {
        RuntimeError::new(operator.clone(), "Operand must be a number.")
    }

    fn number_operands_error(&mut self, operator: &Token) -> RuntimeError {
        RuntimeError::new(operator.clone(), "Operands must be a number.")
    }

    fn is_truthy(&mut self, val: Value) -> bool {
        match val {
            Value::Nil => false,
            Value::Bool(b) => b,
            _ => true,
        }
    }

    fn is_equal(&mut self, left: Value, right: Value) -> bool {
        match (left, right) {
            (Value::Bool(l), Value::Bool(r)) => l == r,
            (Value::Nil, Value::Nil) => true,
            _ => false,
        }
    }

    fn stringify(&mut self, value: Value) -> String {
        match value {
            Value::Nil => "nil".into(),
            Value::Str(str) => str,
            Value::Bool(b) => b.to_string(),
            Value::Number(number) => {
                let text = number.to_string();
                if text.ends_with(".0") {
                    text.trim_end_matches(".0").to_string()
                } else {
                    text
                }
            }
        }
    }
}
