// RPC Generator - Auto-generates RPC stubs for server/client communication
//
// This module generates:
// 1. Client-side stubs: Functions that make HTTP calls to server functions
// 2. Server-side handlers: Express-style route handlers for RPC endpoints

use crate::ast::{FunctionDefinition, FunctionParameter, TypeExpression, Identifier};

#[derive(Debug, Clone)]
pub struct RPCGenerator {
    pub server_functions: Vec<FunctionDefinition>,
}

impl RPCGenerator {
    pub fn new(server_functions: Vec<FunctionDefinition>) -> Self {
        RPCGenerator { server_functions }
    }

    /// Generates client-side RPC stubs (async functions that call the server)
    pub fn generate_client_stubs(&self) -> String {
        let mut output = String::new();

        // Import RPC client runtime
        output.push_str("// Auto-generated RPC client stubs\n");
        output.push_str("import { RPCClient } from '../dist/client-runtime.js';\n\n");
        output.push_str("const client = new RPCClient(window.location.origin + '/_rpc');\n\n");

        // Generate stub for each server function
        for func in &self.server_functions {
            output.push_str(&self.generate_client_stub(func));
            output.push('\n');
        }

        output
    }

    /// Generates a single client stub function
    fn generate_client_stub(&self, func: &FunctionDefinition) -> String {
        let name = &func.name.value;
        let params = self.format_parameters(&func.parameters);
        let param_names = self.extract_parameter_names(&func.parameters);

        format!(
            "export async function {}({}) {{\n\
            \x20   return await client.call('{}', [{}]);\n\
            }}",
            name, params, name, param_names
        )
    }

    /// Generates server-side RPC handlers (Express-style routes)
    pub fn generate_server_handlers(&self) -> String {
        let mut output = String::new();

        // Import server runtime
        output.push_str("// Auto-generated RPC server handlers\n");
        output.push_str("const { HttpServer } = require('../dist/server-runtime.js');\n\n");
        output.push_str("const server = new HttpServer(process.env.PORT || 3000);\n\n");

        // Import WASM module
        output.push_str("// Load WebAssembly module\n");
        output.push_str("const fs = require('fs');\n");
        output.push_str("const wasmBytes = fs.readFileSync('./app.wasm');\n");
        output.push_str("const wasmModule = new WebAssembly.Module(wasmBytes);\n");
        output.push_str("const wasmInstance = new WebAssembly.Instance(wasmModule, {});\n\n");

        // Generate handler for each server function
        for func in &self.server_functions {
            output.push_str(&self.generate_server_handler(func));
            output.push('\n');
        }

        // Start the server
        output.push_str("// Start RPC server\n");
        output.push_str("server.start();\n");
        output.push_str("console.log(`RPC server listening on port ${server.port}`);\n");

        output
    }

    /// Generates a single server handler
    fn generate_server_handler(&self, func: &FunctionDefinition) -> String {
        let name = &func.name.value;
        let param_names = self.extract_parameter_names(&func.parameters);

        format!(
            "server.rpc('{}', async (params) => {{\n\
            \x20   // Call WASM function or JavaScript implementation\n\
            \x20   const [{}] = params;\n\
            \x20   return await {}({});\n\
            }});",
            name, param_names, name, param_names
        )
    }

    /// Formats function parameters as a string
    fn format_parameters(&self, params: &[FunctionParameter]) -> String {
        params
            .iter()
            .map(|p| {
                let name = &p.name.value;
                let type_str = self.format_type(&p.type_annotation);
                format!("{}: {}", name, type_str)
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Extracts just the parameter names (for passing to functions)
    fn extract_parameter_names(&self, params: &[FunctionParameter]) -> String {
        params
            .iter()
            .map(|p| p.name.value.clone())
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Formats a type annotation as a TypeScript/JavaScript type
    fn format_type(&self, type_expr: &TypeExpression) -> String {
        match type_expr {
            TypeExpression::Named(ident) => {
                // Map RavensOne types to TypeScript types
                match ident.value.as_str() {
                    "i32" | "i64" | "u32" | "u64" | "f32" | "f64" => "number".to_string(),
                    "String" => "string".to_string(),
                    "bool" => "boolean".to_string(),
                    other => other.to_string(),
                }
            }
            TypeExpression::Generic(ident, args) => {
                let base = self.format_type_name(&ident.value);
                let arg_types = args
                    .iter()
                    .map(|arg| self.format_type(arg))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}<{}>", base, arg_types)
            }
            TypeExpression::Tuple(types) => {
                let type_strs = types
                    .iter()
                    .map(|t| self.format_type(t))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{}]", type_strs)
            }
            TypeExpression::Reference(inner) | TypeExpression::MutableReference(inner) => {
                self.format_type(inner)
            }
            TypeExpression::Slice(inner) => {
                format!("{}[]", self.format_type(inner))
            }
        }
    }

    /// Formats a type name for TypeScript
    fn format_type_name(&self, name: &str) -> String {
        match name {
            "Vec" => "Array".to_string(),
            "Option" => "Optional".to_string(),
            "Result" => "Result".to_string(),
            other => other.to_string(),
        }
    }

    /// Generates TypeScript type definitions for server functions
    pub fn generate_type_definitions(&self) -> String {
        let mut output = String::new();

        output.push_str("// Auto-generated TypeScript type definitions\n\n");

        for func in &self.server_functions {
            let name = &func.name.value;
            let params = self.format_parameters(&func.parameters);

            // Note: We'd need to track return types in the AST for this to be complete
            output.push_str(&format!(
                "export function {}({}): Promise<any>;\n",
                name, params
            ));
        }

        output
    }

    /// Returns statistics about the RPC generation
    pub fn stats(&self) -> RPCStats {
        RPCStats {
            server_functions: self.server_functions.len(),
            total_parameters: self.server_functions
                .iter()
                .map(|f| f.parameters.len())
                .sum(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RPCStats {
    pub server_functions: usize,
    pub total_parameters: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BlockStatement, Statement};
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::code_splitter::CodeSplitter;

    #[test]
    fn test_rpc_generation() {
        let source = r#"
            @server
            fn get_user(id: i32) -> String {
                return "user";
            }

            @server
            fn save_data(name: String, age: i32) -> bool {
                return true;
            }
        "#;

        // Parse and split
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

        // Generate RPC code
        let rpc_gen = RPCGenerator::new(splitter.server_functions.clone());

        // Test client stubs
        let client_stubs = rpc_gen.generate_client_stubs();
        assert!(client_stubs.contains("async function get_user(id: number)"));
        assert!(client_stubs.contains("async function save_data(name: string, age: number)"));
        assert!(client_stubs.contains("client.call('get_user'"));
        assert!(client_stubs.contains("client.call('save_data'"));

        // Test server handlers
        let server_handlers = rpc_gen.generate_server_handlers();
        assert!(server_handlers.contains("server.rpc('get_user'"));
        assert!(server_handlers.contains("server.rpc('save_data'"));
        assert!(server_handlers.contains("HttpServer"));

        // Test stats
        let stats = rpc_gen.stats();
        assert_eq!(stats.server_functions, 2);
        assert_eq!(stats.total_parameters, 3); // id + name + age
    }

    #[test]
    fn test_type_formatting() {
        let rpc_gen = RPCGenerator::new(vec![]);

        // Test basic types
        let i32_type = TypeExpression::Named(Identifier { value: "i32".to_string() });
        assert_eq!(rpc_gen.format_type(&i32_type), "number");

        let string_type = TypeExpression::Named(Identifier { value: "String".to_string() });
        assert_eq!(rpc_gen.format_type(&string_type), "string");

        let bool_type = TypeExpression::Named(Identifier { value: "bool".to_string() });
        assert_eq!(rpc_gen.format_type(&bool_type), "boolean");

        // Test generic types
        let vec_i32 = TypeExpression::Generic(
            Identifier { value: "Vec".to_string() },
            vec![TypeExpression::Named(Identifier { value: "i32".to_string() })]
        );
        assert_eq!(rpc_gen.format_type(&vec_i32), "Array<number>");
    }

    #[test]
    fn test_parameter_extraction() {
        let params = vec![
            FunctionParameter {
                name: Identifier { value: "id".to_string() },
                type_annotation: TypeExpression::Named(Identifier { value: "i32".to_string() }),
            },
            FunctionParameter {
                name: Identifier { value: "name".to_string() },
                type_annotation: TypeExpression::Named(Identifier { value: "String".to_string() }),
            },
        ];

        let rpc_gen = RPCGenerator::new(vec![]);
        let param_names = rpc_gen.extract_parameter_names(&params);
        assert_eq!(param_names, "id, name");

        let formatted = rpc_gen.format_parameters(&params);
        assert_eq!(formatted, "id: number, name: string");
    }
}
