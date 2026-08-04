//! Symbol definitions.

use foxa_span::Span;

/// Opaque symbol identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(u32);

impl SymbolId {
    /// Creates a symbol ID from a raw index.
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns the raw index.
    #[must_use]
    pub const fn as_raw(self) -> u32 {
        self.0
    }
}

/// What kind of entity a symbol refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    /// Top-level or nested function.
    Function,
    /// Function parameter.
    Param,
    /// Local `let` binding.
    Local,
    /// Built-in / prelude symbol.
    Builtin,
    /// Struct type.
    Struct,
    /// Enum type.
    Enum,
    /// Enum variant.
    Variant,
}

/// A named definition in the program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    /// Stable ID.
    pub id: SymbolId,
    /// Name as written in source.
    pub name: String,
    /// Kind of definition.
    pub kind: SymbolKind,
    /// Defining span.
    pub span: Span,
    /// Whether the binding is mutable (`let mut` / mutable param later).
    pub mutable: bool,
}

impl Symbol {
    /// Creates a symbol.
    #[must_use]
    pub fn new(
        id: SymbolId,
        name: impl Into<String>,
        kind: SymbolKind,
        span: Span,
        mutable: bool,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            kind,
            span,
            mutable,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foxa_span::FileId;

    #[test]
    fn symbol_fields() {
        let s = Symbol::new(
            SymbolId::from_raw(0),
            "main",
            SymbolKind::Function,
            Span::new(FileId::from_raw(0), 0, 4),
            false,
        );
        assert_eq!(s.name, "main");
        assert_eq!(s.kind, SymbolKind::Function);
    }
}
