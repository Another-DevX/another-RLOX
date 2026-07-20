use std::{cell::RefCell, rc::Rc};

use crate::{
    environment::Environment,
    lox::Lox,
    token::{
        Literal, Token,
        TokenType::{self},
    },
};

pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping(Box<Expr>),
    Literal(Literal),
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Variable {
        name: Token,
    },
    Assignment {
        name: Token,
        value: Box<Expr>,
    },
}

pub enum Stmt {
    Expression(Expr),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    Print(Expr),
    Var {
        name: Token,
        initializer: Option<Expr>,
    },
    Block(Vec<Stmt>),
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

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            environment: Rc::new(RefCell::new(Environment::default())),
        }
    }

    pub fn interpret(&mut self, lox: &mut Lox, statements: Vec<Stmt>) {
        for statement in statements.iter() {
            match self.execute(statement) {
                Ok(()) => {}
                Err(error) => lox.runtime_error(error),
            };
        }

        // match self.evaluate(&expr) {
        //     Ok(value) => {
        //         println!("{}", self.stringify(value));
        //     }
        //     Err(error) => lox.runtime_error(error),
        // };
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        use Stmt::*;
        match stmt {
            Expression(expr) => match self.evaluate(expr) {
                Ok(_) => Ok(()),
                Err(error) => Err(error),
            },
            If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition = self.evaluate(condition)?;
                if self.is_truthy(&condition) {
                    self.execute(then_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self.execute(else_branch)?;
                }

                Ok(())
            }

            While { condition, body } => {

                loop {
                    let condition = self.evaluate(condition)?;
                    if !self.is_truthy(&condition) {
                        break;
                    }
                    self.execute(body)?;
                }

                Ok(())
            }

            Print(expr) => {
                let value = self.evaluate(expr)?;
                println!("{}", self.stringify(value));
                Ok(())
            }
            Var { name, initializer } => {
                let value = if let Some(expr) = initializer {
                    self.evaluate(expr)?
                } else {
                    Value::Nil
                };

                self.environment.borrow_mut().define(&name.lexeme, value);
                Ok(())
            }
            Block(statements) => self.execute_block(
                statements,
                Rc::new(RefCell::new(Environment::new(Some(
                    self.environment.clone(),
                )))),
            ),
        }
    }

    fn execute_block(
        &mut self,
        statements: &Vec<Stmt>,
        environment: Rc<RefCell<Environment>>,
    ) -> Result<(), RuntimeError> {
        let previous = Rc::clone(&self.environment);

        self.environment = environment;
        let result = statements.iter().try_for_each(|stmt| self.execute(stmt));

        self.environment = previous;

        result
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal(value) => Ok(value.clone().into()),
            Expr::Grouping(inner) => self.evaluate(inner),
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(left)?;
                let is_truthy = self.is_truthy(&left);

                match operator.kind {
                    TokenType::Or if is_truthy => return Ok(left),
                    TokenType::And if !is_truthy => return Ok(left),
                    TokenType::Or | TokenType::And => self.evaluate(right),
                    _ => unreachable!("Invalid logical operator"),
                }
            }
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
                    TokenType::Bang => Ok(Value::Bool(!self.is_truthy(&right))),
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

                    TokenType::BangEqual => Ok(Value::Bool(!self.is_equal(&left, &right))),

                    TokenType::EqualEqual => Ok(Value::Bool(self.is_equal(&left, &right))),

                    _ => Ok(Value::Nil),
                }
            }
            Expr::Assignment { name, value } => {
                let value = self.evaluate(value)?;
                self.environment.borrow_mut().assign(name, value.clone())?;
                Ok(value)
            }
            Expr::Variable { name } => {
                let variable = self.environment.borrow().get(name)?;
                Ok(variable)
            }
        }
    }

    fn number_operand_error(&mut self, operator: &Token) -> RuntimeError {
        RuntimeError::new(operator.clone(), "Operand must be a number.")
    }

    fn number_operands_error(&mut self, operator: &Token) -> RuntimeError {
        RuntimeError::new(operator.clone(), "Operands must be a number.")
    }

    fn is_truthy(&mut self, val: &Value) -> bool {
        match val {
            Value::Nil => false,
            Value::Bool(b) => *b,
            _ => true,
        }
    }

    fn is_equal(&mut self, left: &Value, right: &Value) -> bool {
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
