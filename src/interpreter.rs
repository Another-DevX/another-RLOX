use core::str;

use crate::token::{Literal, Token};

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

pub struct AstInterpreter;

impl AstInterpreter {
    pub fn print(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::Binary {
                left,
                operator,
                right,
            } => self.parenthesize(operator.lexeme.as_str(), &[left.as_ref(), right.as_ref()]),
            Expr::Grouping(expr) => self.parenthesize("group", &[expr]),
            Expr::Unary { operator, right } => {
                self.parenthesize(&operator.lexeme, &[right.as_ref()])
            }
            Expr::Literal(opt) => match opt {
                Literal::Str(s) => s.clone(),
                Literal::Number(n) => n.to_string(),
                Literal::Bool(b) => b.to_string(),
                Literal::Nil => "nil".to_string(),
            },
        }
    }

    fn parenthesize(&mut self, name: &str, exprs: &[&Expr]) -> String {
        let mut builder = String::new();
        builder.push('(');
        builder.push_str(name);

        for expr in exprs.iter() {
            builder.push(' ');
            builder.push_str(self.print(expr).as_str());
        }
        builder.push(')');
        builder
    }
}
