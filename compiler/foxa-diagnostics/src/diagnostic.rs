//! Diagnostic data model.

use foxa_span::Span;
use std::fmt;

/// How serious a diagnostic is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    /// Hard failure; compilation cannot succeed.
    Error,
    /// Soft failure; compilation may continue with `--deny-warnings` failing.
    Warning,
    /// Informational annotation attached to another diagnostic.
    Note,
    /// Suggested fix or next step.
    Help,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Error => write!(f, "error"),
            Self::Warning => write!(f, "warning"),
            Self::Note => write!(f, "note"),
            Self::Help => write!(f, "help"),
        }
    }
}

/// A labeled region of source code pointing at a diagnostic cause.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
    /// Source span.
    pub span: Span,
    /// Short message shown next to the span.
    pub message: String,
    /// Whether this is the primary highlight.
    pub primary: bool,
}

impl Label {
    /// Creates a primary label.
    #[must_use]
    pub fn primary(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
            primary: true,
        }
    }

    /// Creates a secondary label.
    #[must_use]
    pub fn secondary(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
            primary: false,
        }
    }
}

/// A single compiler diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// Severity level.
    pub severity: Severity,
    /// Primary human-readable message.
    pub message: String,
    /// Optional machine-readable error code (e.g. `E0001`).
    pub code: Option<String>,
    /// Labeled spans.
    pub labels: Vec<Label>,
    /// Optional help / suggestion text.
    pub help: Option<String>,
    /// Additional notes.
    pub notes: Vec<String>,
}

impl Diagnostic {
    /// Creates a diagnostic with the given severity and message.
    #[must_use]
    pub fn new(severity: Severity, message: impl Into<String>) -> Self {
        Self {
            severity,
            message: message.into(),
            code: None,
            labels: Vec::new(),
            help: None,
            notes: Vec::new(),
        }
    }

    /// Convenience constructor for errors.
    #[must_use]
    pub fn error(message: impl Into<String>) -> Self {
        Self::new(Severity::Error, message)
    }

    /// Convenience constructor for warnings.
    #[must_use]
    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(Severity::Warning, message)
    }

    /// Sets an error code.
    #[must_use]
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Adds a primary label.
    #[must_use]
    pub fn with_label(mut self, span: Span, message: impl Into<String>) -> Self {
        self.labels.push(Label::primary(span, message));
        self
    }

    /// Adds a secondary label.
    #[must_use]
    pub fn with_secondary(mut self, span: Span, message: impl Into<String>) -> Self {
        self.labels.push(Label::secondary(span, message));
        self
    }

    /// Adds help text.
    #[must_use]
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Adds a note.
    #[must_use]
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Returns `true` if this diagnostic is an error.
    #[must_use]
    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }
}

/// Accumulates diagnostics for a compilation unit.
#[derive(Debug, Default, Clone)]
pub struct DiagnosticBag {
    items: Vec<Diagnostic>,
}

impl DiagnosticBag {
    /// Creates an empty bag.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Pushes a diagnostic.
    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.items.push(diagnostic);
    }

    /// Returns all diagnostics.
    #[must_use]
    pub fn items(&self) -> &[Diagnostic] {
        &self.items
    }

    /// Returns `true` if any error was reported.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.items.iter().any(Diagnostic::is_error)
    }

    /// Number of errors.
    #[must_use]
    pub fn error_count(&self) -> usize {
        self.items.iter().filter(|d| d.is_error()).count()
    }

    /// Number of warnings.
    #[must_use]
    pub fn warning_count(&self) -> usize {
        self.items
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .count()
    }

    /// Clears all diagnostics.
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Appends diagnostics from another bag.
    pub fn extend(&mut self, other: DiagnosticBag) {
        self.items.extend(other.items);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foxa_span::FileId;

    #[test]
    fn bag_tracks_errors() {
        let mut bag = DiagnosticBag::new();
        assert!(!bag.has_errors());
        bag.push(Diagnostic::warning("unused"));
        assert!(!bag.has_errors());
        bag.push(
            Diagnostic::error("oops")
                .with_code("E0001")
                .with_label(Span::new(FileId::from_raw(0), 0, 1), "here")
                .with_help("try again"),
        );
        assert!(bag.has_errors());
        assert_eq!(bag.error_count(), 1);
        assert_eq!(bag.warning_count(), 1);
    }
}
