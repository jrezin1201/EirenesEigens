//#[macro_use]
extern crate lazy_static;

pub mod ast;
pub mod borrow_checker;
pub mod codegen;
pub mod deployer; // Make sure deployer is a module
pub mod errors;
pub mod lexer;
pub mod macros;
pub mod parser;
pub mod semantic_analyzer;
pub mod token;
pub mod vdom;
pub mod stdlib; // Standard library
pub mod types; // Type system
pub mod type_checker; // Type checking and inference
pub mod ssr; // Server-side rendering
pub mod hydration; // Client-side hydration
pub mod reactive; // Reactive state management

use borrow_checker::BorrowChecker;
use codegen::CodeGenerator;
use errors::CompileError;
use lexer::Lexer;
use parser::Parser;
use semantic_analyzer::SemanticAnalyzer;
use type_checker::TypeChecker;
use token::{Token, TokenKind};

// This enum is now public so the deployer can use it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BuildTarget {
    Client,
    Server,
}

pub struct Compiler;

impl Compiler {
    pub fn new() -> Self {
        Compiler
    }

    // FIX: The function now takes the target as a required argument.
    pub fn compile_source(&self, source: &str, target: BuildTarget) -> Result<Vec<u8>, CompileError> {
        println!("   - Starting compilation for target: {:?}", target);

        // --- Lexing, Parsing, Macro Expansion ---
        let mut lexer = Lexer::new(source.to_string());
        let tokens = lexer.collect_tokens()?;
        let mut initial_parser = Parser::new(tokens);
        let initial_ast = initial_parser.parse_program()?;

        // This is a simplified macro expansion for now.
        let mut needs_reparse = false;
        for statement in &initial_ast.statements {
            if let ast::Statement::MacroInvocation(_) = statement {
                needs_reparse = true;
                break;
            }
        }
        
        // FIX: Rename the AST variable to avoid shadowing the `ast` module.
        let program_ast = if needs_reparse {
            // Placeholder for real expansion
            initial_ast
        } else {
            initial_ast
        };

        // --- Analysis Passes ---
        let mut analyzer = SemanticAnalyzer::new();
        analyzer.analyze_program(&program_ast)?;

        // Type checking with inference
        let mut type_checker = TypeChecker::new();
        type_checker.check_program(&program_ast.statements)?;

        let mut borrow_checker = BorrowChecker::new();
        borrow_checker.check_program(&program_ast)?;

        // --- Code Generation ---
        // FIX: Pass the target down to the CodeGenerator.
        let mut code_generator = CodeGenerator::new(target);
        let wasm_bytes = code_generator.generate_program(&program_ast)?;
        
        Ok(wasm_bytes)
    }
}

pub trait LexerExt {
    fn collect_tokens(&mut self) -> Result<Vec<Token>, CompileError>;
}

impl LexerExt for Lexer {
    fn collect_tokens(&mut self) -> Result<Vec<Token>, CompileError> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            let kind = token.kind.clone();
            if let TokenKind::Illegal(ch) = kind {
                return Err(CompileError::LexerError(format!("Illegal character: '{}'", ch)));
            }
            tokens.push(token);
            if kind == TokenKind::Eof {
                break;
            }
        }
        Ok(tokens)
    }
}