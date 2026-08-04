//! Nested environments for local bindings.

use crate::error::InterpError;
use crate::value::Value;
use std::collections::HashMap;

/// A stack of binding frames.
#[derive(Debug, Default, Clone)]
pub struct Environment {
    frames: Vec<HashMap<String, Value>>,
}

impl Environment {
    /// Creates an environment with one empty global frame.
    #[must_use]
    pub fn new() -> Self {
        Self {
            frames: vec![HashMap::new()],
        }
    }

    /// Pushes a new frame.
    pub fn push(&mut self) {
        self.frames.push(HashMap::new());
    }

    /// Pops the innermost frame.
    pub fn pop(&mut self) {
        if self.frames.len() > 1 {
            self.frames.pop();
        }
    }

    /// Defines a binding in the current frame.
    pub fn define(&mut self, name: impl Into<String>, value: Value) {
        if let Some(frame) = self.frames.last_mut() {
            frame.insert(name.into(), value);
        }
    }

    /// Assigns to an existing binding, searching outward.
    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), InterpError> {
        for frame in self.frames.iter_mut().rev() {
            if frame.contains_key(name) {
                frame.insert(name.to_string(), value);
                return Ok(());
            }
        }
        Err(InterpError::Undefined(name.to_string()))
    }

    /// Looks up a binding.
    pub fn get(&self, name: &str) -> Result<Value, InterpError> {
        for frame in self.frames.iter().rev() {
            if let Some(v) = frame.get(name) {
                return Ok(v.clone());
            }
        }
        Err(InterpError::Undefined(name.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_lookup() {
        let mut env = Environment::new();
        env.define("x", Value::Int(1));
        env.push();
        env.define("y", Value::Int(2));
        assert_eq!(env.get("x").unwrap(), Value::Int(1));
        assert_eq!(env.get("y").unwrap(), Value::Int(2));
        env.pop();
        assert!(env.get("y").is_err());
    }
}
