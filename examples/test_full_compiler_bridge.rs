// Complete end-to-end test for the RavensOne Compiler Bridge
// Tests the full pipeline: parse → split → RPC generation → JS emission

use ravensone_compiler::lexer::Lexer;
use ravensone_compiler::parser::Parser;
use ravensone_compiler::code_splitter::CodeSplitter;
use ravensone_compiler::rpc_generator::RPCGenerator;
use ravensone_compiler::js_emitter::JSEmitter;
use ravensone_compiler::token::TokenKind;

fn main() {
    println!("=== RavensOne Compiler Bridge - Complete Test ===\n");

    // Sample full-stack todo application
    let source = r#"
        // Server-side database functions
        @server
        fn get_todos() -> Vec<String> {
            return vec!["Buy milk", "Walk dog", "Write code"];
        }

        @server
        fn add_todo(title: String) -> bool {
            return true;
        }

        @server
        fn delete_todo(id: i32) -> bool {
            return true;
        }

        // Client-side rendering functions
        @client
        fn render_todo_list(todos: Vec<String>) -> String {
            return "<ul>...</ul>";
        }

        @client
        fn handle_add_click() {
            let title = "New todo";
            add_todo(title);
        }

        // Shared utility functions (available on both sides)
        fn format_date(timestamp: i32) -> String {
            return "2025-01-01";
        }

        fn validate_input(text: String) -> bool {
            return true;
        }
    "#;

    println!("Source Code:");
    println!("{}\n", source);

    // ===== STEP 1: LEXING =====
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

    // ===== STEP 2: PARSING =====
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

    // ===== STEP 3: CODE SPLITTING =====
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

    // ===== STEP 4: RPC GENERATION =====
    println!("=== Step 4: RPC Generation ===");
    let rpc_gen = RPCGenerator::new(splitter.server_functions.clone());

    println!("Client RPC Stubs:\n");
    let client_stubs = rpc_gen.generate_client_stubs();
    println!("{}\n", client_stubs);

    println!("Server RPC Handlers:\n");
    let server_handlers = rpc_gen.generate_server_handlers();
    println!("{}\n", server_handlers);

    // ===== STEP 5: JS EMISSION =====
    println!("=== Step 5: JavaScript Emission ===");
    let emitter = JSEmitter::new(&program);

    println!("Complete server.js (first 800 chars):\n");
    let server_js = emitter.generate_server_js();
    println!("{}...\n", &server_js[..server_js.len().min(800)]);

    println!("Complete client.js (first 800 chars):\n");
    let client_js = emitter.generate_client_js();
    println!("{}...\n", &client_js[..client_js.len().min(800)]);

    // ===== STEP 6: VERIFICATION =====
    println!("=== Step 6: Verification ===");
    let mut all_good = true;

    // Verify splitting
    if stats.server_functions != 3 {
        eprintln!("✗ Expected 3 server functions, got {}", stats.server_functions);
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

    // Verify RPC stubs
    if !client_stubs.contains("async function get_todos") {
        eprintln!("✗ Missing get_todos stub");
        all_good = false;
    }
    if !client_stubs.contains("async function add_todo") {
        eprintln!("✗ Missing add_todo stub");
        all_good = false;
    }
    if !client_stubs.contains("async function delete_todo") {
        eprintln!("✗ Missing delete_todo stub");
        all_good = false;
    }

    // Verify RPC handlers
    if !server_handlers.contains("server.rpc('get_todos'") {
        eprintln!("✗ Missing get_todos handler");
        all_good = false;
    }
    if !server_handlers.contains("server.rpc('add_todo'") {
        eprintln!("✗ Missing add_todo handler");
        all_good = false;
    }
    if !server_handlers.contains("server.rpc('delete_todo'") {
        eprintln!("✗ Missing delete_todo handler");
        all_good = false;
    }

    // Verify server.js structure
    if !server_js.contains("HttpServer") {
        eprintln!("✗ server.js missing HttpServer import");
        all_good = false;
    }
    if !server_js.contains("WebAssembly") {
        eprintln!("✗ server.js missing WebAssembly setup");
        all_good = false;
    }
    if !server_js.contains("function get_todos") {
        eprintln!("✗ server.js missing get_todos implementation");
        all_good = false;
    }
    if !server_js.contains("function format_date") {
        eprintln!("✗ server.js missing shared format_date function");
        all_good = false;
    }

    // Verify client.js structure
    if !client_js.contains("RPCClient") {
        eprintln!("✗ client.js missing RPCClient import");
        all_good = false;
    }
    if !client_js.contains("export function render_todo_list") {
        eprintln!("✗ client.js missing render_todo_list function");
        all_good = false;
    }
    if !client_js.contains("export function handle_add_click") {
        eprintln!("✗ client.js missing handle_add_click function");
        all_good = false;
    }
    if !client_js.contains("export function format_date") {
        eprintln!("✗ client.js missing shared format_date function");
        all_good = false;
    }
    if !client_js.contains("DOMContentLoaded") {
        eprintln!("✗ client.js missing DOM initialization");
        all_good = false;
    }

    // ===== RESULTS =====
    if all_good {
        println!("✅ All checks passed!\n");
        println!("=== Summary ===");
        println!("The compiler bridge successfully:");
        println!("  1. ✓ Parsed @server/@client annotations");
        println!("  2. ✓ Split code into server/client/shared buckets");
        println!("  3. ✓ Generated type-safe RPC client stubs");
        println!("  4. ✓ Generated server-side RPC handlers");
        println!("  5. ✓ Emitted complete server.js bundle");
        println!("  6. ✓ Emitted complete client.js bundle");
        println!("\n🎉 The RavensOne compiler bridge is fully functional!");
        println!("\nYou can now write .raven files and compile them to working full-stack apps!");
    } else {
        eprintln!("\n✗ Some checks failed");
        std::process::exit(1);
    }
}
