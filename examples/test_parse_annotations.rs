use ravensone_compiler::lexer::Lexer;
use ravensone_compiler::parser::Parser;
use ravensone_compiler::token::TokenKind;
use ravensone_compiler::ast::Statement;
use std::fs;

fn main() {
    let source = fs::read_to_string("examples/test_annotations.raven")
        .expect("Failed to read test file");

    println!("=== Testing Annotation Parsing ===\n");
    println!("Source code:\n{}\n", source);

    // Lexing
    let mut lexer = Lexer::new(source);
    let mut tokens = Vec::new();
    loop {
        let token = lexer.next_token();
        let is_eof = token.kind == TokenKind::Eof;
        tokens.push(token);
        if is_eof { break; }
    }

    println!("=== Tokens ===");
    for token in &tokens[0..30.min(tokens.len())] {
        println!("{:?}", token);
    }

    // Parsing
    let mut parser = Parser::new(tokens);
    match parser.parse_program() {
        Ok(program) => {
            println!("\n=== AST ===");
            for (i, stmt) in program.statements.iter().enumerate() {
                match stmt {
                    Statement::Function(func) => {
                        println!("\nFunction #{}: {}", i, func.name.value);
                        println!("  is_server: {}", func.is_server);
                        println!("  is_client: {}", func.is_client);
                        println!("  is_async: {}", func.is_async);
                    }
                    Statement::Component(comp) => {
                        println!("\nComponent #{}: {}", i, comp.name.value);
                        println!("  is_client: {}", comp.is_client);
                    }
                    _ => {
                        println!("\nStatement #{}: {:?}", i, stmt);
                    }
                }
            }
            println!("\n✅ Parsing succeeded!");
        }
        Err(e) => {
            println!("\n❌ Parsing failed: {:?}", e);
        }
    }
}
