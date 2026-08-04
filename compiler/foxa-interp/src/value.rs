//! Runtime values.

use std::collections::HashMap;
use std::fmt;

/// A runtime value in the interpreter.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Unit / void.
    Unit,
    /// Boolean.
    Bool(bool),
    /// 64-bit signed integer.
    Int(i64),
    /// 64-bit float.
    Float(f64),
    /// UTF-8 string.
    String(String),
    /// Unicode scalar.
    Char(char),
    /// Struct instance.
    Struct {
        /// Type name.
        name: String,
        /// Field values.
        fields: HashMap<String, Value>,
    },
    /// Enum variant instance.
    Enum {
        /// Enum type name (may be empty if unknown).
        type_name: String,
        /// Variant name.
        variant: String,
        /// Tuple payload.
        fields: Vec<Value>,
        /// Named payload.
        named: HashMap<String, Value>,
    },
    /// Dynamic vector.
    Vec(Vec<Value>),
}

impl Value {
    /// Returns `true` for truthy values used by `assert` / conditions.
    #[must_use]
    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Bool(b) => *b,
            Self::Unit => false,
            Self::Int(n) => *n != 0,
            Self::Float(n) => *n != 0.0,
            Self::String(s) => !s.is_empty(),
            Self::Char(_) => true,
            Self::Struct { .. } => true,
            Self::Enum { variant, .. } => variant != "None",
            Self::Vec(v) => !v.is_empty(),
        }
    }

    /// Field access helper.
    pub fn get_field(&self, name: &str) -> Option<&Value> {
        match self {
            Self::Struct { fields, .. } => fields.get(name),
            Self::Enum { named, .. } => named.get(name),
            _ => None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unit => write!(f, "()"),
            Self::Bool(b) => write!(f, "{b}"),
            Self::Int(n) => write!(f, "{n}"),
            Self::Float(n) => write!(f, "{n}"),
            Self::String(s) => write!(f, "{s}"),
            Self::Char(c) => write!(f, "{c}"),
            Self::Struct { name, fields } => {
                write!(f, "{name} {{")?;
                let mut first = true;
                for (k, v) in fields {
                    if !first {
                        write!(f, ", ")?;
                    }
                    first = false;
                    write!(f, "{k}: {v}")?;
                }
                write!(f, "}}")
            }
            Self::Enum {
                variant,
                fields,
                named,
                ..
            } => {
                write!(f, "{variant}")?;
                if !fields.is_empty() {
                    write!(f, "(")?;
                    for (i, v) in fields.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{v}")?;
                    }
                    write!(f, ")")?;
                } else if !named.is_empty() {
                    write!(f, " {{")?;
                    let mut first = true;
                    for (k, v) in named {
                        if !first {
                            write!(f, ", ")?;
                        }
                        first = false;
                        write!(f, "{k}: {v}")?;
                    }
                    write!(f, "}}")?;
                }
                Ok(())
            }
            Self::Vec(items) => {
                write!(f, "[")?;
                for (i, v) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{v}")?;
                }
                write!(f, "]")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_values() {
        assert_eq!(Value::Int(42).to_string(), "42");
        assert_eq!(Value::Bool(true).to_string(), "true");
        assert_eq!(Value::String("hi".into()).to_string(), "hi");
        assert_eq!(
            Value::Vec(vec![Value::Int(1), Value::Int(2)]).to_string(),
            "[1, 2]"
        );
    }
}
