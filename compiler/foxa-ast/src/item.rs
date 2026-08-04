//! Item and module AST nodes.

use crate::Block;
use foxa_span::Span;

/// Top-level module (one source file or explicit `mod`).
#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    /// Module items.
    pub items: Vec<Item>,
    /// Source span of the whole module.
    pub span: Span,
}

/// Visibility modifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Visibility {
    /// Private (default).
    #[default]
    Private,
    /// `pub`
    Public,
}

/// Top-level or nested item.
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    /// Item kind.
    pub kind: ItemKind,
    /// Visibility.
    pub vis: Visibility,
    /// Source span.
    pub span: Span,
}

/// Item variants.
#[derive(Debug, Clone, PartialEq)]
pub enum ItemKind {
    /// Function definition.
    Fn(FnItem),
    /// Struct definition.
    Struct(StructItem),
    /// Enum definition.
    Enum(EnumItem),
    /// Placeholder for incomplete parse recovery.
    Error,
}

/// Function item.
#[derive(Debug, Clone, PartialEq)]
pub struct FnItem {
    /// Function name.
    pub name: String,
    /// Parameters.
    pub params: Vec<Param>,
    /// Optional return type as source text (refined in later phases).
    pub return_ty: Option<String>,
    /// Function body.
    pub body: Block,
}

/// Function parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    /// Parameter name.
    pub name: String,
    /// Type as source text.
    pub ty: String,
    /// Source span.
    pub span: Span,
}

/// Struct definition: `struct Name { field: Type, ... }`
#[derive(Debug, Clone, PartialEq)]
pub struct StructItem {
    /// Struct name.
    pub name: String,
    /// Fields.
    pub fields: Vec<FieldDef>,
    /// Source span.
    pub span: Span,
}

/// A named field in a struct.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldDef {
    /// Field name.
    pub name: String,
    /// Type as source text.
    pub ty: String,
    /// Source span.
    pub span: Span,
}

/// Enum definition: `enum Name { Variant, Variant(Type), Variant { f: T } }`
#[derive(Debug, Clone, PartialEq)]
pub struct EnumItem {
    /// Enum name.
    pub name: String,
    /// Variants.
    pub variants: Vec<VariantDef>,
    /// Source span.
    pub span: Span,
}

/// An enum variant.
#[derive(Debug, Clone, PartialEq)]
pub struct VariantDef {
    /// Variant name.
    pub name: String,
    /// Variant data.
    pub kind: VariantKind,
    /// Source span.
    pub span: Span,
}

/// Shape of an enum variant.
#[derive(Debug, Clone, PartialEq)]
pub enum VariantKind {
    /// Unit variant: `None`
    Unit,
    /// Tuple variant: `Some(T)`
    Tuple(Vec<String>),
    /// Struct variant: `Ok { value: T }`
    Struct(Vec<FieldDef>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use foxa_span::FileId;

    #[test]
    fn empty_module() {
        let m = Module {
            items: vec![],
            span: Span::new(FileId::from_raw(0), 0, 0),
        };
        assert!(m.items.is_empty());
    }
}
