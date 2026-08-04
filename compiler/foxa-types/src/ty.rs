//! Type representations.

use std::fmt;

/// A Foxa type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Ty {
    /// `()` / void-like unit.
    Unit,
    /// `Bool`
    Bool,
    /// `Int`
    Int,
    /// `Float`
    Float,
    /// `String`
    String,
    /// `Char`
    Char,
    /// Function type.
    Fn {
        /// Parameter types.
        params: Vec<Ty>,
        /// Return type.
        ret: Box<Ty>,
    },
    /// Nominal struct/enum/type name (may include generics text).
    Named(String),
    /// `Vec[T]` / array sugar.
    Vec(Box<Ty>),
    /// `Option[T]`
    Option(Box<Ty>),
    /// `Result[T, E]`
    Result {
        /// Ok payload.
        ok: Box<Ty>,
        /// Err payload.
        err: Box<Ty>,
    },
    /// Type error placeholder.
    Error,
}

impl Ty {
    /// Parses a type name from source text, including `Vec[T]`, `Option[T]`, `Result[T, E]`.
    #[must_use]
    pub fn from_name(name: &str) -> Self {
        let name = name.trim();
        match name {
            "Int" => Self::Int,
            "Float" => Self::Float,
            "Bool" => Self::Bool,
            "String" => Self::String,
            "Char" => Self::Char,
            "Unit" | "()" => Self::Unit,
            other => {
                if let Some(inner) = strip_brackets(other, "Vec") {
                    return Self::Vec(Box::new(Self::from_name(inner)));
                }
                if let Some(inner) = strip_brackets(other, "Option") {
                    return Self::Option(Box::new(Self::from_name(inner)));
                }
                if let Some(inner) = strip_brackets(other, "Result") {
                    let parts: Vec<&str> = split_top_level(inner);
                    if parts.len() == 2 {
                        return Self::Result {
                            ok: Box::new(Self::from_name(parts[0])),
                            err: Box::new(Self::from_name(parts[1])),
                        };
                    }
                }
                if let Some(inner) = other.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                    return Self::Vec(Box::new(Self::from_name(inner)));
                }
                Self::Named(other.to_string())
            }
        }
    }

    /// Returns `true` if this is an error type.
    #[must_use]
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error)
    }
}

fn strip_brackets<'a>(s: &'a str, prefix: &str) -> Option<&'a str> {
    let rest = s.strip_prefix(prefix)?;
    let rest = rest.strip_prefix('[')?.strip_suffix(']')?;
    Some(rest)
}

fn split_top_level(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut depth = 0;
    for (i, c) in s.char_indices() {
        match c {
            '[' => depth += 1,
            ']' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(s[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(s[start..].trim());
    parts
}

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unit => write!(f, "Unit"),
            Self::Bool => write!(f, "Bool"),
            Self::Int => write!(f, "Int"),
            Self::Float => write!(f, "Float"),
            Self::String => write!(f, "String"),
            Self::Char => write!(f, "Char"),
            Self::Fn { params, ret } => {
                write!(f, "fn(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{p}")?;
                }
                write!(f, ") -> {ret}")
            }
            Self::Named(name) => write!(f, "{name}"),
            Self::Vec(inner) => write!(f, "Vec[{inner}]"),
            Self::Option(inner) => write!(f, "Option[{inner}]"),
            Self::Result { ok, err } => write!(f, "Result[{ok}, {err}]"),
            Self::Error => write!(f, "{{error}}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_name_primitives() {
        assert_eq!(Ty::from_name("Int"), Ty::Int);
        assert_eq!(Ty::from_name("Foo"), Ty::Named("Foo".into()));
        assert_eq!(Ty::from_name("Vec[Int]"), Ty::Vec(Box::new(Ty::Int)));
        assert_eq!(
            Ty::from_name("Option[String]"),
            Ty::Option(Box::new(Ty::String))
        );
        assert_eq!(
            Ty::from_name("Result[Int, String]"),
            Ty::Result {
                ok: Box::new(Ty::Int),
                err: Box::new(Ty::String)
            }
        );
    }

    #[test]
    fn display_fn() {
        let ty = Ty::Fn {
            params: vec![Ty::Int, Ty::Int],
            ret: Box::new(Ty::Int),
        };
        assert_eq!(ty.to_string(), "fn(Int, Int) -> Int");
    }
}
