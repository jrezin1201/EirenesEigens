// Code Splitter - Separates annotated code into server/client/shared buckets
//
// This module analyzes the AST and splits code based on @server/@client annotations:
// - @server functions → server_functions
// - @client functions → client_functions
// - No annotation → shared_functions (available on both sides)
// - @client components → client_components

use crate::ast::{Program, Statement, FunctionDefinition, ComponentDefinition};

#[derive(Debug, Clone)]
pub struct CodeSplitter {
    pub server_functions: Vec<FunctionDefinition>,
    pub client_functions: Vec<FunctionDefinition>,
    pub shared_functions: Vec<FunctionDefinition>,
    pub client_components: Vec<ComponentDefinition>,
}

impl CodeSplitter {
    pub fn new() -> Self {
        CodeSplitter {
            server_functions: Vec::new(),
            client_functions: Vec::new(),
            shared_functions: Vec::new(),
            client_components: Vec::new(),
        }
    }

    /// Analyzes a program and splits code into buckets based on annotations
    pub fn split(&mut self, program: &Program) {
        for statement in &program.statements {
            match statement {
                Statement::Function(func) => {
                    self.split_function(func);
                }
                Statement::Component(comp) => {
                    self.split_component(comp);
                }
                // Other statements (structs, enums, etc.) are currently ignored
                // In the future, we may want to handle these differently
                _ => {}
            }
        }
    }

    fn split_function(&mut self, func: &FunctionDefinition) {
        if func.is_server {
            // @server function - only available on server
            self.server_functions.push(func.clone());
        } else if func.is_client {
            // @client function - only available on client
            self.client_functions.push(func.clone());
        } else {
            // No annotation - shared code (available on both sides)
            self.shared_functions.push(func.clone());
        }
    }

    fn split_component(&mut self, comp: &ComponentDefinition) {
        // Components are always client-side (they render UI)
        if comp.is_client {
            self.client_components.push(comp.clone());
        }
    }

    /// Returns all functions that should be available on the server
    /// (server functions + shared functions)
    pub fn get_server_code(&self) -> Vec<FunctionDefinition> {
        let mut all = self.server_functions.clone();
        all.extend(self.shared_functions.clone());
        all
    }

    /// Returns all functions that should be available on the client
    /// (client functions + shared functions)
    pub fn get_client_code(&self) -> Vec<FunctionDefinition> {
        let mut all = self.client_functions.clone();
        all.extend(self.shared_functions.clone());
        all
    }

    /// Returns statistics about the split
    pub fn stats(&self) -> SplitStats {
        SplitStats {
            server_functions: self.server_functions.len(),
            client_functions: self.client_functions.len(),
            shared_functions: self.shared_functions.len(),
            client_components: self.client_components.len(),
            total_server_code: self.get_server_code().len(),
            total_client_code: self.get_client_code().len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SplitStats {
    pub server_functions: usize,
    pub client_functions: usize,
    pub shared_functions: usize,
    pub client_components: usize,
    pub total_server_code: usize,
    pub total_client_code: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    #[test]
    fn test_code_splitting() {
        let source = r#"
            @server
            fn get_data() -> String {
                return "server";
            }

            @client
            fn render() -> String {
                return "client";
            }

            fn shared_util() -> String {
                return "shared";
            }
        "#;

        // Parse the code
        let mut lexer = Lexer::new(source.to_string());
        let mut tokens = Vec::new();
        loop {
            let token = lexer.next_token();
            let is_eof = token.kind == crate::token::TokenKind::Eof;
            tokens.push(token);
            if is_eof { break; }
        }

        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().expect("Parse failed");

        // Split the code
        let mut splitter = CodeSplitter::new();
        splitter.split(&program);

        // Verify the split
        assert_eq!(splitter.server_functions.len(), 1);
        assert_eq!(splitter.server_functions[0].name.value, "get_data");

        assert_eq!(splitter.client_functions.len(), 1);
        assert_eq!(splitter.client_functions[0].name.value, "render");

        assert_eq!(splitter.shared_functions.len(), 1);
        assert_eq!(splitter.shared_functions[0].name.value, "shared_util");

        // Verify server code includes shared
        let server_code = splitter.get_server_code();
        assert_eq!(server_code.len(), 2); // server + shared

        // Verify client code includes shared
        let client_code = splitter.get_client_code();
        assert_eq!(client_code.len(), 2); // client + shared
    }

    #[test]
    fn test_stats() {
        let source = r#"
            @server fn s1() {}
            @server fn s2() {}
            @client fn c1() {}
            fn shared1() {}
            fn shared2() {}
        "#;

        let mut lexer = Lexer::new(source.to_string());
        let mut tokens = Vec::new();
        loop {
            let token = lexer.next_token();
            let is_eof = token.kind == crate::token::TokenKind::Eof;
            tokens.push(token);
            if is_eof { break; }
        }

        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().expect("Parse failed");

        let mut splitter = CodeSplitter::new();
        splitter.split(&program);

        let stats = splitter.stats();
        assert_eq!(stats.server_functions, 2);
        assert_eq!(stats.client_functions, 1);
        assert_eq!(stats.shared_functions, 2);
        assert_eq!(stats.total_server_code, 4); // 2 server + 2 shared
        assert_eq!(stats.total_client_code, 3); // 1 client + 2 shared
    }
}
