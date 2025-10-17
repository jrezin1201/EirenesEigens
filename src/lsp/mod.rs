// Language Server Protocol implementation for RavensOne
// Provides IDE features: autocomplete, hover, diagnostics, etc.

use crate::diagnostics::{Diagnostic, DiagnosticCollector, SourceLocation};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::semantic_analyzer::SemanticAnalyzer;
use crate::type_checker::TypeChecker;
use crate::{Compiler, BuildTarget, LexerExt};
use std::collections::HashMap;

/// LSP Server for RavensOne
pub struct LanguageServer {
    documents: HashMap<String, Document>,
    stdlib_docs: StdlibDocs,
}

/// Represents an open document
pub struct Document {
    pub uri: String,
    pub content: String,
    pub version: i32,
    pub diagnostics: Vec<Diagnostic>,
}

/// Position in a document (line, character)
#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub line: usize,
    pub character: usize,
}

/// Range in a document
#[derive(Debug, Clone, Copy)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

/// Completion item for autocomplete
#[derive(Debug, Clone)]
pub struct CompletionItem {
    pub label: String,
    pub kind: CompletionItemKind,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub insert_text: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionItemKind {
    Function,
    Variable,
    Keyword,
    Class,
    Module,
    Property,
    Snippet,
}

/// Hover information
#[derive(Debug, Clone)]
pub struct Hover {
    pub contents: String,
    pub range: Option<Range>,
}

impl LanguageServer {
    pub fn new() -> Self {
        LanguageServer {
            documents: HashMap::new(),
            stdlib_docs: StdlibDocs::new(),
        }
    }

    /// Open a document
    pub fn open_document(&mut self, uri: String, content: String, version: i32) {
        let diagnostics = self.analyze_document(&content);
        let document = Document {
            uri: uri.clone(),
            content,
            version,
            diagnostics,
        };
        self.documents.insert(uri, document);
    }

    /// Change document content
    pub fn change_document(&mut self, uri: &str, content: String, version: i32) {
        // Analyze first to avoid borrow checker issues
        let diagnostics = self.analyze_document(&content);

        if let Some(doc) = self.documents.get_mut(uri) {
            doc.content = content;
            doc.version = version;
            doc.diagnostics = diagnostics;
        }
    }

    /// Close a document
    pub fn close_document(&mut self, uri: &str) {
        self.documents.remove(uri);
    }

    /// Get diagnostics for a document
    pub fn get_diagnostics(&self, uri: &str) -> Vec<Diagnostic> {
        self.documents
            .get(uri)
            .map(|doc| doc.diagnostics.clone())
            .unwrap_or_default()
    }

    /// Analyze document and return diagnostics
    fn analyze_document(&self, content: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Lexical analysis
        let mut lexer = Lexer::new(content.to_string());
        let tokens = match lexer.collect_tokens() {
            Ok(t) => t,
            Err(e) => {
                // Convert compile error to diagnostic
                diagnostics.push(Diagnostic::error(format!("{:?}", e)));
                return diagnostics;
            }
        };

        // Parse
        let mut parser = Parser::new(tokens);
        let ast = match parser.parse_program() {
            Ok(ast) => ast,
            Err(e) => {
                diagnostics.push(Diagnostic::error(format!("{:?}", e)));
                return diagnostics;
            }
        };

        // Semantic analysis
        let mut analyzer = SemanticAnalyzer::new();
        if let Err(e) = analyzer.analyze_program(&ast) {
            diagnostics.push(Diagnostic::error(format!("{:?}", e)));
            return diagnostics;
        }

        // Type checking
        let mut type_checker = TypeChecker::new();
        if let Err(e) = type_checker.check_program(&ast.statements) {
            diagnostics.push(Diagnostic::error(format!("{:?}", e)));
        }

        diagnostics
    }

    /// Get completions at a position
    pub fn get_completions(&self, uri: &str, position: Position) -> Vec<CompletionItem> {
        let mut completions = Vec::new();

        // Add keywords
        completions.extend(self.get_keyword_completions());

        // Add stdlib functions
        completions.extend(self.stdlib_docs.get_completions());

        // Add reactive primitives
        completions.extend(self.get_reactive_completions());

        // TODO: Add local variables and functions from current scope

        completions
    }

    /// Get hover information at a position
    pub fn get_hover(&self, uri: &str, position: Position) -> Option<Hover> {
        let doc = self.documents.get(uri)?;
        let word = self.get_word_at_position(&doc.content, position)?;

        // Check stdlib
        if let Some(docs) = self.stdlib_docs.get_documentation(&word) {
            return Some(Hover {
                contents: docs,
                range: None,
            });
        }

        // Check reactive primitives
        if let Some(docs) = self.get_reactive_docs(&word) {
            return Some(Hover {
                contents: docs,
                range: None,
            });
        }

        None
    }

    /// Get word at position
    fn get_word_at_position(&self, content: &str, position: Position) -> Option<String> {
        let lines: Vec<&str> = content.lines().collect();
        if position.line >= lines.len() {
            return None;
        }

        let line = lines[position.line];
        if position.character >= line.len() {
            return None;
        }

        // Find word boundaries
        let chars: Vec<char> = line.chars().collect();
        let mut start = position.character;
        let mut end = position.character;

        // Go backwards to find start
        while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
            start -= 1;
        }

        // Go forwards to find end
        while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
            end += 1;
        }

        if start < end {
            Some(chars[start..end].iter().collect())
        } else {
            None
        }
    }

    /// Get keyword completions
    fn get_keyword_completions(&self) -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "component".to_string(),
                kind: CompletionItemKind::Keyword,
                detail: Some("Define a component".to_string()),
                documentation: Some("Create a new component".to_string()),
                insert_text: Some("component ${1:Name}() {\n    $0\n}".to_string()),
            },
            CompletionItem {
                label: "fn".to_string(),
                kind: CompletionItemKind::Keyword,
                detail: Some("Define a function".to_string()),
                documentation: None,
                insert_text: Some("fn ${1:name}($2) -> $3 {\n    $0\n}".to_string()),
            },
            CompletionItem {
                label: "let".to_string(),
                kind: CompletionItemKind::Keyword,
                detail: Some("Declare a variable".to_string()),
                documentation: None,
                insert_text: Some("let ${1:name} = $0;".to_string()),
            },
            CompletionItem {
                label: "server".to_string(),
                kind: CompletionItemKind::Keyword,
                detail: Some("Server-side code".to_string()),
                documentation: Some("Mark function as server-side only".to_string()),
                insert_text: None,
            },
            CompletionItem {
                label: "if".to_string(),
                kind: CompletionItemKind::Keyword,
                detail: Some("Conditional".to_string()),
                documentation: None,
                insert_text: Some("if $1 {\n    $0\n}".to_string()),
            },
            CompletionItem {
                label: "for".to_string(),
                kind: CompletionItemKind::Keyword,
                detail: Some("Loop".to_string()),
                documentation: None,
                insert_text: Some("for ${1:item} in ${2:items} {\n    $0\n}".to_string()),
            },
        ]
    }

    /// Get reactive primitive completions
    fn get_reactive_completions(&self) -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "Signal::new".to_string(),
                kind: CompletionItemKind::Function,
                detail: Some("Create a new Signal".to_string()),
                documentation: Some("let count = Signal::new(0);".to_string()),
                insert_text: Some("Signal::new($0)".to_string()),
            },
            CompletionItem {
                label: "Computed::new".to_string(),
                kind: CompletionItemKind::Function,
                detail: Some("Create a computed value".to_string()),
                documentation: Some("let doubled = Computed::new(|| count.get() * 2);".to_string()),
                insert_text: Some("Computed::new(|| $0)".to_string()),
            },
            CompletionItem {
                label: "Effect::new".to_string(),
                kind: CompletionItemKind::Function,
                detail: Some("Create an effect".to_string()),
                documentation: Some("Effect::new(|| { console.log(count.get()); });".to_string()),
                insert_text: Some("Effect::new(|| $0)".to_string()),
            },
            CompletionItem {
                label: "Resource::new".to_string(),
                kind: CompletionItemKind::Function,
                detail: Some("Create an async resource".to_string()),
                documentation: Some("let data = Resource::new(async { fetch_data().await });".to_string()),
                insert_text: Some("Resource::new(async { $0 })".to_string()),
            },
        ]
    }

    /// Get reactive primitive documentation
    fn get_reactive_docs(&self, word: &str) -> Option<String> {
        match word {
            "Signal" => Some("**Signal** - A reactive value that can be read and written.\n\n```raven\nlet count = Signal::new(0);\ncount.set(5);\nlet value = count.get();\n```".to_string()),
            "Computed" => Some("**Computed** - A derived value that automatically updates.\n\n```raven\nlet doubled = Computed::new(|| count.get() * 2);\n```".to_string()),
            "Effect" => Some("**Effect** - Run side effects when dependencies change.\n\n```raven\nEffect::new(|| {\n    console.log(count.get());\n});\n```".to_string()),
            "Resource" => Some("**Resource** - Async data loading with automatic refetching.\n\n```raven\nlet data = Resource::new(async {\n    fetch_data().await\n});\n```".to_string()),
            _ => None,
        }
    }
}

impl Default for LanguageServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Standard library documentation
pub struct StdlibDocs {
    functions: HashMap<String, FunctionDoc>,
}

#[derive(Debug, Clone)]
pub struct FunctionDoc {
    pub name: String,
    pub signature: String,
    pub description: String,
    pub examples: Vec<String>,
}

impl StdlibDocs {
    pub fn new() -> Self {
        let mut functions = HashMap::new();

        // HTTP functions
        functions.insert(
            "fetch".to_string(),
            FunctionDoc {
                name: "fetch".to_string(),
                signature: "fn fetch(url: String) -> Result<Response, Error>".to_string(),
                description: "Make an HTTP request".to_string(),
                examples: vec![
                    "let response = fetch(\"https://api.example.com/data\").await?;".to_string(),
                ],
            },
        );

        // Console functions
        functions.insert(
            "console.log".to_string(),
            FunctionDoc {
                name: "console.log".to_string(),
                signature: "fn log(message: Any)".to_string(),
                description: "Log a message to the console".to_string(),
                examples: vec![
                    "console.log(\"Hello, world!\");".to_string(),
                ],
            },
        );

        StdlibDocs { functions }
    }

    pub fn get_completions(&self) -> Vec<CompletionItem> {
        self.functions
            .values()
            .map(|doc| CompletionItem {
                label: doc.name.clone(),
                kind: CompletionItemKind::Function,
                detail: Some(doc.signature.clone()),
                documentation: Some(doc.description.clone()),
                insert_text: None,
            })
            .collect()
    }

    pub fn get_documentation(&self, name: &str) -> Option<String> {
        self.functions.get(name).map(|doc| {
            format!(
                "**{}**\n\n{}\n\n```raven\n{}\n```\n\n{}",
                doc.name,
                doc.signature,
                doc.examples.join("\n"),
                doc.description
            )
        })
    }
}

impl Default for StdlibDocs {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_server_open_document() {
        let mut server = LanguageServer::new();
        server.open_document(
            "file:///test.raven".to_string(),
            "let x = 10;".to_string(),
            1,
        );

        assert!(server.documents.contains_key("file:///test.raven"));
    }

    #[test]
    fn test_get_completions() {
        let server = LanguageServer::new();
        let completions = server.get_completions("file:///test.raven", Position { line: 0, character: 0 });

        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.label == "component"));
        assert!(completions.iter().any(|c| c.label == "Signal::new"));
    }

    #[test]
    fn test_get_word_at_position() {
        let server = LanguageServer::new();
        let content = "let count = Signal::new(0);";
        let word = server.get_word_at_position(content, Position { line: 0, character: 5 });

        assert_eq!(word, Some("count".to_string()));
    }

    #[test]
    fn test_reactive_docs() {
        let server = LanguageServer::new();
        let docs = server.get_reactive_docs("Signal");

        assert!(docs.is_some());
        assert!(docs.unwrap().contains("reactive value"));
    }
}
