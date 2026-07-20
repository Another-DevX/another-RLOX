use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    interpreter::{RuntimeError, Value},
    token::Token,
};

pub struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>,
    variables: HashMap<String, Value>,
}

impl Environment {
    pub fn new(enclosing: Option<Rc<RefCell<Environment>>>) -> Self {
        Environment {
            enclosing,
            variables: HashMap::new(),
        }
    }

    pub fn default() -> Self {
        Environment {
            enclosing: None,
            variables: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: &str, value: Value) {
        self.variables.insert(name.to_string(), value.clone());
    }

    pub fn get(&self, name: &Token) -> Result<Value, RuntimeError> {
        if let Some(result) = self.variables.get(&name.lexeme) {
            return Ok(result.clone());
        }

        if let Some(enclosing) = self.enclosing.as_ref() {
            return enclosing.borrow().get(name);
        }

        let error = RuntimeError::new(
            name.clone(),
            &format!("Undefined variable '{}'.", name.lexeme),
        );
        Err(error)
    }

    pub fn assign(&mut self, name: &Token, value: Value) -> Result<(), RuntimeError> {
        if let Some(current_value) = self.variables.get_mut(&name.lexeme) {
            *current_value = value;
            return Ok(());
        }

        if let Some(enclosing) = self.enclosing.as_mut() {
            return enclosing.borrow_mut().assign(name, value)
        }

        let error = RuntimeError::new(
            name.clone(),
            &format!("Undefined variable '{}'.", name.lexeme),
        );
        Err(error)
    }
}
