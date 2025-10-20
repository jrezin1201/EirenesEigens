// End-to-end test for the compiler bridge
// Tests the full pipeline: parse → split → generate RPC → output

use ravensone_compiler::lexer::Lexer;
use ravensone_compiler::parser::Parser;
use ravensone_compiler::code_splitter::CodeSplitter;
use ravensone_compiler::rpc_generator::RPCGenerator;
use ravensone_compiler::token::TokenKind;

fn main() {
    println!("=== RavensOne Compiler Bridge End-to-End Test ===\n");

    // Sample full-stack application
    let source = r#"
        // Server-side functions
        @server
        fn get_user(id: i32) -> String {
            return "John Doe";
        }

        @server
        fn save_post(title: String, content: String) -> bool {
            return true;
        }

        // Client-side functions
        @client
        fn render_user_card(name: String) -> String {
            return "<div>" + name + "</div>";
        }

        @client
        fn handle_click() {
            return "clicked";
        }

        // Shared utility functions (no annotation)
        fn format_date(timestamp: i32) -> String {
            return "2025-01-01";
        }

        fn validate_email(email: String) -> bool {
            return true;
        }
    "#;

    println!("Source Code:\n{}\n", source);

    // Step 1: Lexing
    println!("=== Step 1: Lexing ===");
    let mut lexer = Lexer::new(source.to_string());
    let mut tokens = Vec::new();
    loop {
        let token = lexer.next_token();
        let is_eof = token.kind == TokenKind::Eof;
        tokens.push(token);
        if is_eof { break; }
    }
    println!("✓ Generated {} tokens\n", tokens.len());

    // Step 2: Parsing
    println!("=== Step 2: Parsing ===");
    let mut parser = Parser::new(tokens);
    let program = match parser.parse_program() {
        Ok(p) => {
            println!("✓ Parsed {} statements\n", p.statements.len());
            p
        }
        Err(e) => {
            eprintln!("✗ Parse error: {:?}", e);
            return;
        }
    };

    // Step 3: Code Splitting
    println!("=== Step 3: Code Splitting ===");
    let mut splitter = CodeSplitter::new();
    splitter.split(&program);

    let stats = splitter.stats();
    println!("Server functions: {}", stats.server_functions);
    println!("Client functions: {}", stats.client_functions);
    println!("Shared functions: {}", stats.shared_functions);
    println!("Total server code: {}", stats.total_server_code);
    println!("Total client code: {}", stats.total_client_code);
    println!();

    // Step 4: RPC Generation
    println!("=== Step 4: RPC Generation ===");
    let rpc_gen = RPCGenerator::new(splitter.server_functions.clone());

    println!("Client Stubs:\n");
    let client_stubs = rpc_gen.generate_client_stubs();
    println!("{}\n", client_stubs);

    println!("Server Handlers:\n");
    let server_handlers = rpc_gen.generate_server_handlers();
    println!("{}\n", server_handlers);

    // Step 5: Verification
    println!("=== Step 5: Verification ===");
    let mut all_good = true;

    // Verify client stubs
    if !client_stubs.contains("async function get_user") {
        eprintln!("✗ Missing get_user stub");
        all_good = false;
    }
    if !client_stubs.contains("async function save_post") {
        eprintln!("✗ Missing save_post stub");
        all_good = false;
    }
    if !client_stubs.contains("client.call") {
        eprintln!("✗ Missing RPC client call");
        all_good = false;
    }

    // Verify server handlers
    if !server_handlers.contains("server.rpc('get_user'") {
        eprintln!("✗ Missing get_user handler");
        all_good = false;
    }
    if !server_handlers.contains("server.rpc('save_post'") {
        eprintln!("✗ Missing save_post handler");
        all_good = false;
    }
    if !server_handlers.contains("HttpServer") {
        eprintln!("✗ Missing HttpServer");
        all_good = false;
    }

    // Verify splitting worked correctly
    if stats.server_functions != 2 {
        eprintln!("✗ Expected 2 server functions, got {}", stats.server_functions);
        all_good = false;
    }
    if stats.client_functions != 2 {
        eprintln!("✗ Expected 2 client functions, got {}", stats.client_functions);
        all_good = false;
    }
    if stats.shared_functions != 2 {
        eprintln!("✗ Expected 2 shared functions, got {}", stats.shared_functions);
        all_good = false;
    }

    if all_good {
        println!("✅ All checks passed!");
        println!("\n=== Summary ===");
        println!("The compiler bridge successfully:");
        println!("  1. Parsed @server/@client annotations");
        println!("  2. Split code into server/client/shared buckets");
        println!("  3. Generated type-safe RPC client stubs");
        println!("  4. Generated server-side RPC handlers");
        println!("\nThe compiler bridge is working correctly! 🎉");
    } else {
        eprintln!("\n✗ Some checks failed");
        std::process::exit(1);
    }
}
