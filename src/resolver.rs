use std::{collections::HashMap, rc::Rc, thread::scope};

use crate::{
    interpreter::{Expr, Interpreter, RuntimeError, Stmt},
    parser::ParseError,
    resolver,
    token::{Literal, Token},
};

struct Resolver {
    interpreter: Rc<Interpreter>,
    scopes: Vec<HashMap<String, bool>>,
}

impl Resolver {
    fn resolve(&mut self, statements: &Vec<Stmt>) {
        for stmt in statements {
            self.resolve_stmt(stmt);
        }
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::Block(statements) => {
                self.begin_scope();
                self.resolve(statements);
                self.end_scope();
                Ok(())
            }
            Stmt::Var { name, initializer } => {
                self.declare(name);
                if let Some(initializer) = initializer {
                    self.resolve_expr(initializer);
                }
                self.define(name);
                Ok(())
            }
            Stmt::Expression(expr) | Stmt::Print(expr) => {
                self.resolve_expr(expr);
                Ok(())
            }
            Stmt::Function { name, params, body } => {
                self.declare(name);
                self.define(name);

                self.resolve_function(params, body);
                Ok(())
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expr(condition);
                self.resolve_stmt(then_branch);
                if let Some(else_branch) = else_branch {
                    self.resolve_stmt(else_branch);
                }
                Ok(())
            }
            Stmt::Return(stmt) => {
                if let Some(return_stmt) = stmt {
                    self.resolve_stmt(return_stmt);
                }
                Ok(())
            }
            Stmt::While { condition, body } => {
                self.resolve_expr(condition);
                self.resolve_stmt(body);
                Ok(())
            }
            Stmt::Break => Ok(()),
        }
    }

    fn resolve_expr(&mut self, expr: &Expr) -> Result<(), RuntimeError> {
        match expr {
            Expr::Variable { name } => {
                let state = self.scopes.last().and_then(|scope| scope.get);

                if self.scopes.is_empty()
                    && self
                        .scopes
                        .last()
                        .unwrap()
                        .get(&name.lexeme)
                        .unwrap()
                        .eq(&false)
                {
                    return Err(RuntimeError::new(
                        name.clone(),
                        "Can't read local variable in its own initializer.",
                    ));
                }
                self.resolve_local(expr, name);
                Ok(())
            }
            Expr::Assignment { name, value } => {
                self.resolve_expr(value);
                self.resolve_local(value, name);
                Ok(())
            }
            Expr::Binary {
                left,
                operator: _,
                right,
            }
            | Expr::Logical {
                left,
                operator: _,
                right,
            } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
                Ok(())
            }
            Expr::Call {
                callee,
                paren,
                arguments,
            } => {
                self.resolve_expr(callee);
                for argument in arguments.iter() {
                    self.resolve_expr(argument);
                }
                Ok(())
            }

            Expr::Grouping(grouping) => {
                self.resolve_expr(grouping);
                Ok(())
            }
            Expr::Literal(_) => Ok(()),
            Expr::Unary { operator, right } => {
                self.resolve_expr(right);
                Ok(())
            }

            _ => Ok(()),
        }
    }

    fn resolve_local(&mut self, expr: &Expr, name: &Token) {
        for (index, scope) in self.scopes.iter().enumerate() {
            if scope.contains_key(name.lexeme.as_str()) {
                self.interpreter
                    .resolve(expr, self.scopes.len() - 1 - index);
                break;
            }
        }
    }

    fn resolve_function(&mut self, params: &Vec<Token>, body: &Vec<Stmt>) {
        self.begin_scope();
        for param in params.iter() {
            self.declare(param);
            self.define(param);
        }

        self.resolve(body);
        self.end_scope();
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }

        self.scopes
            .last_mut()
            .unwrap()
            .insert(name.lexeme.clone(), false);
    }

    fn define(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }

        self.scopes
            .last_mut()
            .unwrap()
            .insert(name.lexeme.clone(), true);
    }
}
