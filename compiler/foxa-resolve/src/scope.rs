//! Lexical scopes.

use crate::symbol::SymbolId;
use std::collections::HashMap;

/// Opaque scope identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeId(u32);

impl ScopeId {
    /// Creates a scope ID from a raw index.
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

/// A lexical scope with an optional parent.
#[derive(Debug, Clone)]
pub struct Scope {
    /// Scope ID.
    pub id: ScopeId,
    /// Parent scope, if any.
    pub parent: Option<ScopeId>,
    /// Name → symbol bindings introduced in this scope.
    bindings: HashMap<String, SymbolId>,
}

impl Scope {
    /// Creates an empty scope.
    #[must_use]
    pub fn new(id: ScopeId, parent: Option<ScopeId>) -> Self {
        Self {
            id,
            parent,
            bindings: HashMap::new(),
        }
    }

    /// Inserts a binding. Returns the previous symbol if the name existed.
    pub fn define(&mut self, name: impl Into<String>, symbol: SymbolId) -> Option<SymbolId> {
        self.bindings.insert(name.into(), symbol)
    }

    /// Looks up a name in this scope only (not parents).
    #[must_use]
    pub fn lookup_local(&self, name: &str) -> Option<SymbolId> {
        self.bindings.get(name).copied()
    }

    /// Returns all local bindings.
    #[must_use]
    pub fn bindings(&self) -> &HashMap<String, SymbolId> {
        &self.bindings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn define_and_lookup() {
        let mut scope = Scope::new(ScopeId::from_raw(0), None);
        let id = SymbolId::from_raw(1);
        assert!(scope.define("x", id).is_none());
        assert_eq!(scope.lookup_local("x"), Some(id));
        assert!(scope.lookup_local("y").is_none());
    }
}
