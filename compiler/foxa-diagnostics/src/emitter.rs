//! Diagnostic rendering.

use crate::diagnostic::{Diagnostic, DiagnosticBag, Severity};
use foxa_span::SourceMap;
use std::io::{self, Write};

/// Output style for diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderStyle {
    /// Human-readable multi-line format (default).
    #[default]
    Human,
    /// One diagnostic per line, machine-friendly.
    Short,
}

/// Renders diagnostics against a [`SourceMap`].
#[derive(Debug)]
pub struct Emitter<'a> {
    source_map: &'a SourceMap,
    style: RenderStyle,
    color: bool,
}

impl<'a> Emitter<'a> {
    /// Creates an emitter for the given source map.
    #[must_use]
    pub fn new(source_map: &'a SourceMap) -> Self {
        Self {
            source_map,
            style: RenderStyle::Human,
            color: false,
        }
    }

    /// Enables or disables ANSI colors.
    #[must_use]
    pub fn with_color(mut self, color: bool) -> Self {
        self.color = color;
        self
    }

    /// Sets the render style.
    #[must_use]
    pub fn with_style(mut self, style: RenderStyle) -> Self {
        self.style = style;
        self
    }

    /// Emits all diagnostics in a bag to `out`.
    pub fn emit_all(&self, bag: &DiagnosticBag, out: &mut dyn Write) -> io::Result<()> {
        for diag in bag.items() {
            self.emit_one(diag, out)?;
        }
        Ok(())
    }

    /// Emits a single diagnostic.
    pub fn emit_one(&self, diag: &Diagnostic, out: &mut dyn Write) -> io::Result<()> {
        match self.style {
            RenderStyle::Short => self.emit_short(diag, out),
            RenderStyle::Human => self.emit_human(diag, out),
        }
    }

    fn severity_tag(&self, severity: Severity) -> String {
        if !self.color {
            return severity.to_string();
        }
        match severity {
            Severity::Error => format!("\x1b[31m{severity}\x1b[0m"),
            Severity::Warning => format!("\x1b[33m{severity}\x1b[0m"),
            Severity::Note => format!("\x1b[36m{severity}\x1b[0m"),
            Severity::Help => format!("\x1b[32m{severity}\x1b[0m"),
        }
    }

    fn emit_short(&self, diag: &Diagnostic, out: &mut dyn Write) -> io::Result<()> {
        let loc = diag
            .labels
            .first()
            .and_then(|l| self.source_map.format_span_start(l.span))
            .unwrap_or_else(|| "<unknown>".to_string());
        let code = diag
            .code
            .as_ref()
            .map(|c| format!("[{c}] "))
            .unwrap_or_default();
        writeln!(
            out,
            "{loc}: {}: {code}{}",
            self.severity_tag(diag.severity),
            diag.message
        )
    }

    fn emit_human(&self, diag: &Diagnostic, out: &mut dyn Write) -> io::Result<()> {
        let code = diag
            .code
            .as_ref()
            .map(|c| format!("[{c}] "))
            .unwrap_or_default();
        writeln!(
            out,
            "{}: {code}{}",
            self.severity_tag(diag.severity),
            diag.message
        )?;

        for label in &diag.labels {
            if let Some(loc) = self.source_map.format_span_start(label.span) {
                writeln!(out, "  --> {loc}")?;
            }
            if let Some(snippet) = self.source_map.snippet(label.span) {
                let marker = if label.primary { "^" } else { "-" };
                writeln!(out, "   |")?;
                writeln!(out, "   |  {snippet}")?;
                writeln!(
                    out,
                    "   |  {} {}",
                    marker.repeat(snippet.chars().count().max(1)),
                    label.message
                )?;
            } else if !label.message.is_empty() {
                writeln!(out, "   = {}", label.message)?;
            }
        }

        for note in &diag.notes {
            writeln!(out, "   = note: {note}")?;
        }
        if let Some(help) = &diag.help {
            writeln!(out, "   = help: {help}")?;
        }
        writeln!(out)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::Diagnostic;
    use foxa_span::{SourceMap, Span};

    #[test]
    fn renders_human_diagnostic() {
        let mut map = SourceMap::new();
        let id = map.add_file("t.foxa", "let x = ;\n");
        let mut bag = DiagnosticBag::new();
        bag.push(
            Diagnostic::error("expected expression")
                .with_code("E0100")
                .with_label(Span::new(id, 8, 9), "unexpected `;`")
                .with_help("provide a value after `=`"),
        );

        let emitter = Emitter::new(&map);
        let mut buf = Vec::new();
        emitter.emit_all(&bag, &mut buf).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains("error: [E0100] expected expression"));
        assert!(text.contains("t.foxa:1:9"));
        assert!(text.contains("help: provide a value after `=`"));
    }
}
