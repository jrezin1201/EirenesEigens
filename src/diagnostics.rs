// Enhanced Diagnostics and Error Reporting for RavensOne
// Beautiful, helpful error messages with colors, context, and suggestions

use crate::errors::CompileError;
use crate::token::Token;
use std::fmt;

/// ANSI color codes for terminal output
pub mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";

    pub const RED: &str = "\x1b[31m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BLUE: &str = "\x1b[34m";
    pub const MAGENTA: &str = "\x1b[35m";
    pub const CYAN: &str = "\x1b[36m";
    pub const WHITE: &str = "\x1b[37m";

    pub const BG_RED: &str = "\x1b[41m";
    pub const BG_YELLOW: &str = "\x1b[43m";
}

/// Severity level of a diagnostic
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Help,
}

impl Severity {
    pub fn color(&self) -> &'static str {
        match self {
            Severity::Error => colors::RED,
            Severity::Warning => colors::YELLOW,
            Severity::Info => colors::CYAN,
            Severity::Help => colors::GREEN,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
            Severity::Help => "help",
        }
    }
}

/// Source location for error reporting
#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub length: usize,
}

impl SourceLocation {
    pub fn from_token(token: &Token, file: &str) -> Self {
        SourceLocation {
            file: file.to_string(),
            line: token.line,
            column: token.column,
            length: token.lexeme.len(),
        }
    }

    pub fn unknown() -> Self {
        SourceLocation {
            file: "<unknown>".to_string(),
            line: 0,
            column: 0,
            length: 0,
        }
    }
}

/// A diagnostic message (error, warning, info, or help)
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub location: Option<SourceLocation>,
    pub suggestions: Vec<String>,
    pub notes: Vec<String>,
    pub code: Option<String>, // Error code like E001
}

impl Diagnostic {
    pub fn error(message: impl Into<String>) -> Self {
        Diagnostic {
            severity: Severity::Error,
            message: message.into(),
            location: None,
            suggestions: Vec::new(),
            notes: Vec::new(),
            code: None,
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Diagnostic {
            severity: Severity::Warning,
            message: message.into(),
            location: None,
            suggestions: Vec::new(),
            notes: Vec::new(),
            code: None,
        }
    }

    pub fn at(mut self, location: SourceLocation) -> Self {
        self.location = Some(location);
        self
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestions.push(suggestion.into());
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Format and display the diagnostic with colors
    pub fn display(&self, source_code: Option<&str>) -> String {
        let mut output = String::new();

        // Header: severity and message
        output.push_str(&format!(
            "{bold}{color}{severity}:{reset} {bold}{message}{reset}\n",
            bold = colors::BOLD,
            color = self.severity.color(),
            severity = self.severity.label(),
            reset = colors::RESET,
            message = self.message,
        ));

        // Location info
        if let Some(loc) = &self.location {
            output.push_str(&format!(
                "  {dim}-->{reset} {file}:{line}:{column}\n",
                dim = colors::DIM,
                reset = colors::RESET,
                file = loc.file,
                line = loc.line,
                column = loc.column,
            ));

            // Show source code snippet if available
            if let Some(source) = source_code {
                output.push_str(&self.format_source_snippet(source, loc));
            }
        }

        // Error code
        if let Some(code) = &self.code {
            output.push_str(&format!(
                "  {dim}[{code}]{reset}\n",
                dim = colors::DIM,
                code = code,
                reset = colors::RESET,
            ));
        }

        // Suggestions
        for suggestion in &self.suggestions {
            output.push_str(&format!(
                "  {green}{bold}help:{reset} {suggestion}\n",
                green = colors::GREEN,
                bold = colors::BOLD,
                reset = colors::RESET,
                suggestion = suggestion,
            ));
        }

        // Notes
        for note in &self.notes {
            output.push_str(&format!(
                "  {cyan}note:{reset} {note}\n",
                cyan = colors::CYAN,
                reset = colors::RESET,
                note = note,
            ));
        }

        output
    }

    /// Format a source code snippet with the error highlighted
    fn format_source_snippet(&self, source: &str, loc: &SourceLocation) -> String {
        let mut output = String::new();
        let lines: Vec<&str> = source.lines().collect();

        if loc.line == 0 || loc.line > lines.len() {
            return output;
        }

        let line_idx = loc.line - 1;
        let line_content = lines[line_idx];
        let line_num_width = loc.line.to_string().len();

        // Show previous line for context (if available)
        if line_idx > 0 {
            output.push_str(&format!(
                "   {dim}{:>width$} |{reset} {}\n",
                line_idx,
                lines[line_idx - 1],
                dim = colors::DIM,
                reset = colors::RESET,
                width = line_num_width,
            ));
        }

        // Show the error line
        output.push_str(&format!(
            "   {cyan}{:>width$} |{reset} {}\n",
            loc.line,
            line_content,
            cyan = colors::CYAN,
            reset = colors::RESET,
            width = line_num_width,
        ));

        // Show the error indicator (^^^)
        let padding = " ".repeat(line_num_width + 3 + loc.column - 1);
        let underline = "^".repeat(loc.length.max(1));
        output.push_str(&format!(
            "   {dim}{:>width$} |{reset}{padding}{color}{underline}{reset}\n",
            "",
            dim = colors::DIM,
            reset = colors::RESET,
            width = line_num_width,
            padding = padding,
            color = self.severity.color(),
            underline = underline,
        ));

        // Show next line for context (if available)
        if line_idx + 1 < lines.len() {
            output.push_str(&format!(
                "   {dim}{:>width$} |{reset} {}\n",
                line_idx + 2,
                lines[line_idx + 1],
                dim = colors::DIM,
                reset = colors::RESET,
                width = line_num_width,
            ));
        }

        output
    }
}

/// Diagnostic builder for common error patterns
pub struct DiagnosticBuilder;

impl DiagnosticBuilder {
    /// Type mismatch error
    pub fn type_mismatch(expected: &str, found: &str, location: SourceLocation) -> Diagnostic {
        Diagnostic::error(format!("type mismatch: expected `{}`, found `{}`", expected, found))
            .at(location)
            .with_code("E001")
            .with_suggestion(format!("consider converting `{}` to `{}`", found, expected))
    }

    /// Undefined variable error
    pub fn undefined_variable(name: &str, location: SourceLocation) -> Diagnostic {
        Diagnostic::error(format!("cannot find variable `{}` in this scope", name))
            .at(location)
            .with_code("E002")
            .with_suggestion("check that the variable is declared before use")
            .with_note("variables must be declared with `let` before they can be used")
    }

    /// Undefined function error
    pub fn undefined_function(name: &str, location: SourceLocation, similar: Option<&str>) -> Diagnostic {
        let mut diag = Diagnostic::error(format!("cannot find function `{}` in this scope", name))
            .at(location)
            .with_code("E003");

        if let Some(similar_name) = similar {
            diag = diag.with_suggestion(format!("did you mean `{}`?", similar_name));
        }

        diag
    }

    /// Invalid syntax error
    pub fn syntax_error(expected: &str, found: &str, location: SourceLocation) -> Diagnostic {
        Diagnostic::error(format!("expected {}, found {}", expected, found))
            .at(location)
            .with_code("E004")
    }

    /// Borrow checker error
    pub fn borrow_error(message: impl Into<String>, location: SourceLocation) -> Diagnostic {
        Diagnostic::error(message)
            .at(location)
            .with_code("E005")
            .with_note("RavensOne enforces memory safety through borrow checking")
    }

    /// Invalid JSX error
    pub fn jsx_error(message: impl Into<String>, location: SourceLocation) -> Diagnostic {
        Diagnostic::error(message)
            .at(location)
            .with_code("E006")
            .with_note("JSX syntax must follow proper HTML-like structure")
    }

    /// Warning: unused variable
    pub fn unused_variable(name: &str, location: SourceLocation) -> Diagnostic {
        Diagnostic::warning(format!("unused variable: `{}`", name))
            .at(location)
            .with_code("W001")
            .with_suggestion(format!("prefix with `_` to silence: `_{}`", name))
    }

    /// Warning: unreachable code
    pub fn unreachable_code(location: SourceLocation) -> Diagnostic {
        Diagnostic::warning("unreachable code")
            .at(location)
            .with_code("W002")
            .with_note("any code after a `return` statement will never execute")
    }
}

/// Diagnostic collector for managing multiple diagnostics
pub struct DiagnosticCollector {
    diagnostics: Vec<Diagnostic>,
    error_count: usize,
    warning_count: usize,
}

impl DiagnosticCollector {
    pub fn new() -> Self {
        DiagnosticCollector {
            diagnostics: Vec::new(),
            error_count: 0,
            warning_count: 0,
        }
    }

    pub fn add(&mut self, diagnostic: Diagnostic) {
        match diagnostic.severity {
            Severity::Error => self.error_count += 1,
            Severity::Warning => self.warning_count += 1,
            _ => {}
        }
        self.diagnostics.push(diagnostic);
    }

    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    pub fn has_warnings(&self) -> bool {
        self.warning_count > 0
    }

    pub fn error_count(&self) -> usize {
        self.error_count
    }

    pub fn warning_count(&self) -> usize {
        self.warning_count
    }

    pub fn display_all(&self, source_code: Option<&str>) -> String {
        let mut output = String::new();

        for diagnostic in &self.diagnostics {
            output.push_str(&diagnostic.display(source_code));
            output.push('\n');
        }

        // Summary
        if self.has_errors() || self.has_warnings() {
            output.push_str(&format!(
                "{bold}compilation result:{reset} ",
                bold = colors::BOLD,
                reset = colors::RESET,
            ));

            if self.has_errors() {
                output.push_str(&format!(
                    "{red}{error_count} error{s}{reset}",
                    red = colors::RED,
                    error_count = self.error_count,
                    s = if self.error_count == 1 { "" } else { "s" },
                    reset = colors::RESET,
                ));
            }

            if self.has_warnings() {
                if self.has_errors() {
                    output.push_str(", ");
                }
                output.push_str(&format!(
                    "{yellow}{warning_count} warning{s}{reset}",
                    yellow = colors::YELLOW,
                    warning_count = self.warning_count,
                    s = if self.warning_count == 1 { "" } else { "s" },
                    reset = colors::RESET,
                ));
            }

            output.push('\n');
        }

        output
    }

    pub fn clear(&mut self) {
        self.diagnostics.clear();
        self.error_count = 0;
        self.warning_count = 0;
    }
}

impl Default for DiagnosticCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Levenshtein distance for fuzzy matching (did you mean...?)
pub fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if s1.chars().nth(i - 1) == s2.chars().nth(j - 1) {
                0
            } else {
                1
            };

            matrix[i][j] = std::cmp::min(
                std::cmp::min(matrix[i - 1][j] + 1, matrix[i][j - 1] + 1),
                matrix[i - 1][j - 1] + cost,
            );
        }
    }

    matrix[len1][len2]
}

/// Find the most similar string from a list (for "did you mean?" suggestions)
pub fn find_similar(target: &str, candidates: &[&str]) -> Option<String> {
    let mut best_match: Option<(String, usize)> = None;

    for candidate in candidates {
        let distance = levenshtein_distance(target, candidate);

        // Only consider matches within reasonable distance (< 3 edits)
        if distance < 3 {
            if let Some((_, best_distance)) = best_match {
                if distance < best_distance {
                    best_match = Some((candidate.to_string(), distance));
                }
            } else {
                best_match = Some((candidate.to_string(), distance));
            }
        }
    }

    best_match.map(|(name, _)| name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_display() {
        let diag = Diagnostic::error("type mismatch")
            .at(SourceLocation {
                file: "test.raven".to_string(),
                line: 10,
                column: 5,
                length: 3,
            })
            .with_suggestion("consider converting types")
            .with_code("E001");

        let output = diag.display(None);
        assert!(output.contains("error"));
        assert!(output.contains("type mismatch"));
        assert!(output.contains("test.raven:10:5"));
        assert!(output.contains("help"));
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("test", "test"), 0);
        assert_eq!(levenshtein_distance("", "test"), 4);
    }

    #[test]
    fn test_find_similar() {
        let candidates = vec!["Signal", "Computed", "Effect", "Resource"];

        assert_eq!(find_similar("Signa", &candidates), Some("Signal".to_string()));
        assert_eq!(find_similar("Compited", &candidates), Some("Computed".to_string()));
        assert_eq!(find_similar("xyz", &candidates), None); // Too different
    }

    #[test]
    fn test_diagnostic_collector() {
        let mut collector = DiagnosticCollector::new();

        collector.add(Diagnostic::error("error 1"));
        collector.add(Diagnostic::warning("warning 1"));
        collector.add(Diagnostic::error("error 2"));

        assert_eq!(collector.error_count(), 2);
        assert_eq!(collector.warning_count(), 1);
        assert!(collector.has_errors());
        assert!(collector.has_warnings());
    }

    #[test]
    fn test_source_snippet_formatting() {
        let source = "let x = 10;\nlet y = 20;\nlet z = x + y;";
        let loc = SourceLocation {
            file: "test.raven".to_string(),
            line: 2,
            column: 5,
            length: 1,
        };

        let diag = Diagnostic::error("undefined variable").at(loc);
        let output = diag.display(Some(source));

        assert!(output.contains("let y = 20"));
        assert!(output.contains("^"));
    }
}
