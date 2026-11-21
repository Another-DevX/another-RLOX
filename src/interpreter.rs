use std::{
    cell::RefCell,
    iter::zip,
    ops::ControlFlow,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    environment::Environment,
    lox::Lox,
    token::{
        Literal, Token,
        TokenType::{self},
    },
};

pub trait Callable {
    fn arity(&self) -> usize;
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
    ) -> Result<Value, RuntimeError>;
    fn to_string(&self) -> String;
}

struct Function {
    declaration: Stmt,
    closure: Rc<RefCell<Environment>>,
}

impl Callable for Function {
    fn arity(&self) -> usize {
        match &self.declaration {
            Stmt::Function { params, .. } => params.len(),
            _ => unreachable!("Function must contains a function."),
        }
    }

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        let mut environment = Environment::new(Some(Rc::clone(&self.closure)));
        match &self.declaration {
            Stmt::Function { params, body, .. } => {
                for (param, argument) in zip(params.iter(), arguments.iter()) {
                    environment.define(&param.lexeme, argument.clone());
                }
                interpreter.execute_block(&body, Rc::new(RefCell::new(environment)))?;
            }
            _ => unreachable!("Function must contains a function."),
        };

        Ok(Value::Nil)
    }

    fn to_string(&self) -> String {
        match &self.declaration {
            Stmt::Function { name, .. } => format!("<fn {}>", name.lexeme),
            _ => unreachable!("Function must contains a function."),
        }
    }
}

#[derive(Clone)]
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
    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    },
}
#[derive(Clone)]
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
    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    },
    Break,
    Return(Option<Box<Stmt>>),
}

#[derive(Clone)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    Str(String),
    Callable(Rc<dyn Callable>),
}

pub struct RuntimeError {
    pub token: Token,
    pub message: String,
}

pub enum Signal {
    Break,
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
    globals: Rc<RefCell<Environment>>,
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        struct Clock;

        impl Callable for Clock {
            fn arity(&self) -> usize {
                0
            }

            fn call(&self, _: &mut Interpreter, _: Vec<Value>) -> Result<Value, RuntimeError> {
                Ok(Value::Number(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("System time is before the Unix epoch")
                        .as_secs_f64() as f64,
                ))
            }

            fn to_string(&self) -> String {
                "<native fn>".to_owned()
            }
        }

        let globals = Rc::new(RefCell::new(Environment::default()));

        globals
            .borrow_mut()
            .define("clock", Value::Callable(Rc::new(Clock)));

        Interpreter {
            globals: Rc::clone(&globals),
            environment: Rc::clone(&globals),
        }
    }

    pub fn interpret(&mut self, lox: &mut Lox, statements: Vec<Stmt>) {
        for statement in statements.iter() {
            match self.execute(statement) {
                Ok(None) => {}
                Err(error) => lox.runtime_error(error),
                _ => {}
            };
        }

        // match self.evaluate(&expr) {
        //     Ok(value) => {
        //         println!("{}", self.stringify(value));
        //     }
        //     Err(error) => lox.runtime_error(error),
        // };
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<Option<Signal>, RuntimeError> {
        use Stmt::*;
        match stmt {
            Expression(expr) => match self.evaluate(expr) {
                Ok(_) => Ok(None),
                Err(error) => Err(error),
            },
            If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition = self.evaluate(condition)?;
                let mut signal = None;
                if self.is_truthy(&condition) {
                    signal = self.execute(then_branch)?
                } else if let Some(else_branch) = else_branch {
                    signal = self.execute(else_branch)?;
                }

                return Ok(signal);
            }

            While { condition, body } => {
                loop {
                    let condition = self.evaluate(condition)?;
                    if !self.is_truthy(&condition) {
                        break;
                    }
                    if self
                        .execute(body)?
                        .is_some_and(|signal| matches!(signal, Signal::Break))
                    {
                        break;
                    }
                }

                Ok(None)
            }
            Print(expr) => {
                let value = self.evaluate(expr)?;
                println!("{}", self.stringify(value));
                Ok(None)
            }
            Var { name, initializer } => {
                let value = if let Some(expr) = initializer {
                    self.evaluate(expr)?
                } else {
                    Value::Nil
                };

                self.environment.borrow_mut().define(&name.lexeme, value);
                Ok(None)
            }
            Block(statements) => self.execute_block(
                statements,
                Rc::new(RefCell::new(Environment::new(Some(
                    self.environment.clone(),
                )))),
            ),
            Function { name, .. } => {
                let function = self::Function {
                    declaration: stmt.clone(),
                    closure: Rc::clone(&self.environment),
                };
                self.environment
                    .borrow_mut()
                    .define(&name.lexeme, Value::Callable(Rc::new(function)));
                Ok(None)
            }
            Break => Ok(Some(Signal::Break)),
        }
    }

    fn execute_block(
        &mut self,
        statements: &Vec<Stmt>,
        environment: Rc<RefCell<Environment>>,
    ) -> Result<Option<Signal>, RuntimeError> {
        let previous = Rc::clone(&self.environment);

        self.environment = environment;
        let result = statements
            .iter()
            .try_for_each(|stmt| match self.execute(stmt) {
                Ok(None) => ControlFlow::Continue(()),
                result => ControlFlow::Break(result),
            });

        self.environment = previous;

        match result {
            ControlFlow::Continue(()) => Ok(None),
            ControlFlow::Break(result) => result,
        }
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
            Expr::Call {
                callee,
                paren,
                arguments,
            } => {
                let _callee = self.evaluate(&callee)?;
                let mut _arguments = Vec::new();

                for arg in arguments.iter() {
                    _arguments.push(self.evaluate(arg)?);
                }

                match _callee {
                    Value::Callable(function) => {
                        if _arguments.len() != function.arity() {
                            return Err(RuntimeError::new(
                                paren.clone(),
                                &format!(
                                    "Expected {} arguments but got {}.",
                                    function.arity(),
                                    _arguments.len()
                                ),
                            ));
                        }

                        function.call(self, _arguments)
                    }

                    _ => Err(RuntimeError::new(
                        paren.clone(),
                        "Can only call functions and classes.",
                    )),
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
            (Value::Number(a), Value::Number(b)) => a == b,
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
            Value::Callable(callable) => callable.to_string(),
        }
    }
}
