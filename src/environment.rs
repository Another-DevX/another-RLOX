use std::collections::HashMap;

use crate::{
    interpreter::{RuntimeError, Value},
    token::Token,
};

pub struct Environment {
    variables: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            variables: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: &str, value: Value) {
        self.variables.insert(name.to_string(), value.clone());
    }

    pub fn get(&mut self, name: &Token) -> Result<Value, RuntimeError> {
        if let Some(result) = self.variables.get(&name.lexeme) {
            return Ok(result.clone());
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

        let error = RuntimeError::new(
            name.clone(),
            &format!("Undefined variable '{}'.", name.lexeme),
        );
        Err(error)
    }
}
