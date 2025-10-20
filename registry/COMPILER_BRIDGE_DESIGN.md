# Compiler Bridge Architecture Design

**Goal**: Enable writing `.raven` files that automatically split into server.js + client.js + app.wasm

---

## 🏗️ Architecture Overview

```
app.raven
    ↓
┌─────────────────────┐
│  1. Lexer/Parser    │ → Parse @server/@client annotations
│     (Modified)      │    Tag AST nodes with context
└─────────────────────┘
    ↓
┌─────────────────────┐
│  2. Code Splitter   │ → Separate into 3 buckets:
│     (New Module)    │    - Server-only code
└─────────────────────┘    - Client-only code
    ↓                      - Shared code
┌─────────────────────┐
│  3. RPC Generator   │ → Auto-generate RPC stubs
│     (New Module)    │    - Client→Server calls
└─────────────────────┘    - Type-safe interfaces
    ↓
┌─────────────────────┐
│  4. JS Emitter      │ → Generate JavaScript:
│     (New Module)    │    - server.js (Node.js)
└─────────────────────┘    - client.js (Browser)
    ↓                      - Import runtimes
┌─────────────────────┐
│  5. WASM Linker     │ → Link WASM to JS runtimes
│     (Modified)      │    - Import/export setup
└─────────────────────┘
    ↓
    ↓
server.js + client.js + app.wasm
```

---

## 📋 Step-by-Step Implementation Plan

### Step 1: Annotation Parser (3 days)

**Goal**: Parse `@server` and `@client` annotations

**Changes Needed**:

1. **Lexer** (`src/lexer.rs`):
   ```rust
   // Add new tokens
   pub enum Token {
       // ... existing tokens
       At,           // @
       Server,       // server (keyword after @)
       Client,       // client (keyword after @)
   }
   ```

2. **Parser** (`src/parser.rs`):
   ```rust
   // Add annotation parsing
   fn parse_annotation(&mut self) -> Option<Annotation> {
       if self.current_token == Token::At {
           self.advance();
           match self.current_token {
               Token::Identifier(ref name) if name == "server" => {
                   Some(Annotation::Server)
               }
               Token::Identifier(ref name) if name == "client" => {
                   Some(Annotation::Client)
               }
               _ => None
           }
       } else {
           None
       }
   }
   ```

3. **AST** (`src/ast.rs`):
   ```rust
   #[derive(Debug, Clone, PartialEq)]
   pub enum Annotation {
       Server,
       Client,
       Shared,  // Default if no annotation
   }

   #[derive(Debug, Clone)]
   pub struct Function {
       pub name: String,
       pub params: Vec<FunctionParam>,
       pub return_type: Option<Type>,
       pub body: Vec<Statement>,
       pub annotation: Annotation,  // NEW FIELD
   }

   #[derive(Debug, Clone)]
   pub struct ComponentDecl {
       pub name: String,
       pub props: Option<Type>,
       pub body: Vec<Statement>,
       pub annotation: Annotation,  // NEW FIELD
   }
   ```

**Example Input**:
```raven
@server
fn get_user(id: i32) -> User {
    db.users.find(id)
}

@client
component UserProfile(user: User) {
    <div>{user.name}</div>
}

// No annotation = shared
fn format_date(date: DateTime) -> String {
    date.format("%Y-%m-%d")
}
```

**Test**:
```rust
#[test]
fn test_parse_server_annotation() {
    let input = "@server\nfn hello() { }";
    let ast = parse(input);
    assert_eq!(ast.functions[0].annotation, Annotation::Server);
}
```

---

### Step 2: Code Splitter (5 days)

**Goal**: Separate code into server/client/shared buckets

**New Module**: `src/code_splitter.rs`

```rust
use crate::ast::*;

pub struct CodeSplitter {
    server_functions: Vec<Function>,
    client_functions: Vec<Function>,
    shared_functions: Vec<Function>,
    server_components: Vec<ComponentDecl>,
    client_components: Vec<ComponentDecl>,
}

impl CodeSplitter {
    pub fn new() -> Self {
        CodeSplitter {
            server_functions: vec![],
            client_functions: vec![],
            shared_functions: vec![],
            server_components: vec![],
            client_components: vec![],
        }
    }

    pub fn split(&mut self, program: &Program) {
        for func in &program.functions {
            match func.annotation {
                Annotation::Server => self.server_functions.push(func.clone()),
                Annotation::Client => self.client_functions.push(func.clone()),
                Annotation::Shared => self.shared_functions.push(func.clone()),
            }
        }

        for comp in &program.components {
            match comp.annotation {
                Annotation::Client => self.client_components.push(comp.clone()),
                _ => {} // Components are client-only for now
            }
        }
    }

    pub fn get_server_code(&self) -> Vec<Function> {
        let mut all = self.server_functions.clone();
        all.extend(self.shared_functions.clone());
        all
    }

    pub fn get_client_code(&self) -> Vec<Function> {
        let mut all = self.client_functions.clone();
        all.extend(self.shared_functions.clone());
        all
    }
}
```

**Test**:
```rust
#[test]
fn test_code_splitting() {
    let program = parse_program("@server fn a() {} @client fn b() {} fn c() {}");
    let mut splitter = CodeSplitter::new();
    splitter.split(&program);
    
    assert_eq!(splitter.server_functions.len(), 1);
    assert_eq!(splitter.client_functions.len(), 1);
    assert_eq!(splitter.shared_functions.len(), 1);
}
```

---

### Step 3: RPC Generator (4 days)

**Goal**: Auto-generate RPC client stubs for server functions

**New Module**: `src/rpc_generator.rs`

```rust
use crate::ast::*;

pub struct RPCGenerator {
    server_functions: Vec<Function>,
}

impl RPCGenerator {
    pub fn new(server_functions: Vec<Function>) -> Self {
        RPCGenerator { server_functions }
    }

    pub fn generate_client_stubs(&self) -> String {
        let mut output = String::new();
        
        // Import RPC client
        output.push_str("const { RPCClient } = require('../dist/client-runtime.js');\n");
        output.push_str("const client = new RPCClient('http://localhost:3000/_rpc');\n\n");
        
        // Generate stub for each server function
        for func in &self.server_functions {
            output.push_str(&self.generate_stub(func));
        }
        
        output
    }

    fn generate_stub(&self, func: &Function) -> String {
        let name = &func.name;
        let params: Vec<String> = func.params.iter()
            .map(|p| p.name.clone())
            .collect();
        let params_str = params.join(", ");
        
        format!(
            "async function {}({}) {{\n",
            name, params_str
        ) + &format!(
            "    return await client.call('{}', [{}]);\n",
            name, params_str
        ) + "}\n\n"
    }

    pub fn generate_server_handlers(&self) -> String {
        let mut output = String::new();
        
        output.push_str("const { HttpServer } = require('../dist/server-runtime.js');\n");
        output.push_str("const server = new HttpServer(3000);\n\n");
        
        for func in &self.server_functions {
            let name = &func.name;
            output.push_str(&format!(
                "server.rpc('{}', async (params) => {{\n",
                name
            ));
            output.push_str(&format!(
                "    return await {}(...params);\n",
                name
            ));
            output.push_str("});\n\n");
        }
        
        output.push_str("server.start();\n");
        output
    }
}
```

**Example Output** (client stub):
```javascript
// Generated client stub
async function get_user(id) {
    return await client.call('get_user', [id]);
}
```

**Example Output** (server handler):
```javascript
// Generated server handler
server.rpc('get_user', async (params) => {
    return await get_user(...params);
});
```

---

### Step 4: JavaScript Emitter (3 days)

**Goal**: Generate complete server.js and client.js files

**New Module**: `src/js_emitter.rs`

```rust
use crate::ast::*;
use crate::rpc_generator::RPCGenerator;

pub struct JSEmitter {
    server_functions: Vec<Function>,
    client_functions: Vec<Function>,
    shared_functions: Vec<Function>,
}

impl JSEmitter {
    pub fn emit_server_js(&self) -> String {
        let mut output = String::new();
        
        // Imports
        output.push_str("// Auto-generated server.js\n");
        output.push_str("const fs = require('fs');\n");
        output.push_str("const { Database } = require('../dist/db-runtime.js');\n");
        output.push_str("const { HttpClient } = require('../dist/http-runtime.js');\n");
        output.push_str("const { AuthService } = require('../dist/auth-runtime.js');\n\n");
        
        // Load WASM
        output.push_str("const wasmBytes = fs.readFileSync('./app.wasm');\n");
        output.push_str("const wasmModule = new WebAssembly.Module(wasmBytes);\n");
        output.push_str("const wasmInstance = new WebAssembly.Instance(wasmModule);\n\n");
        
        // Server functions (converted from WASM)
        for func in &self.server_functions {
            output.push_str(&self.emit_function(func));
        }
        
        // Shared functions
        for func in &self.shared_functions {
            output.push_str(&self.emit_function(func));
        }
        
        // RPC handlers
        let rpc_gen = RPCGenerator::new(self.server_functions.clone());
        output.push_str(&rpc_gen.generate_server_handlers());
        
        output
    }

    pub fn emit_client_js(&self) -> String {
        let mut output = String::new();
        
        // Imports
        output.push_str("// Auto-generated client.js\n");
        output.push_str("import { Component, h } from '../dist/component-runtime.js';\n");
        output.push_str("import { RPCClient } from '../dist/client-runtime.js';\n\n");
        
        // RPC stubs
        let rpc_gen = RPCGenerator::new(self.server_functions.clone());
        output.push_str(&rpc_gen.generate_client_stubs());
        
        // Client functions
        for func in &self.client_functions {
            output.push_str(&self.emit_function(func));
        }
        
        // Shared functions (duplicated on client)
        for func in &self.shared_functions {
            output.push_str(&self.emit_function(func));
        }
        
        output
    }

    fn emit_function(&self, func: &Function) -> String {
        // Convert RavensOne function to JavaScript
        // This is a simplified version - full implementation would be more complex
        format!("function {}() {{\n    // TODO: Emit function body\n}}\n\n", func.name)
    }
}
```

---

### Step 5: Integration (2 days)

**Goal**: Wire everything together in main.rs

**Modified**: `src/main.rs`

```rust
mod code_splitter;
mod rpc_generator;
mod js_emitter;

use code_splitter::CodeSplitter;
use js_emitter::JSEmitter;

fn compile_fullstack(input_file: &str) {
    // 1. Parse
    let source = fs::read_to_string(input_file)?;
    let program = parser::parse(&source)?;
    
    // 2. Split code
    let mut splitter = CodeSplitter::new();
    splitter.split(&program);
    
    // 3. Emit JavaScript
    let emitter = JSEmitter {
        server_functions: splitter.get_server_functions(),
        client_functions: splitter.get_client_functions(),
        shared_functions: splitter.get_shared_functions(),
    };
    
    // 4. Write output files
    fs::write("server.js", emitter.emit_server_js())?;
    fs::write("client.js", emitter.emit_client_js())?;
    
    // 5. Compile WASM (existing codegen)
    let wasm_bytes = codegen::generate_wasm(&program)?;
    fs::write("app.wasm", wasm_bytes)?;
    
    println!("✅ Compiled successfully!");
    println!("   - server.js");
    println!("   - client.js");
    println!("   - app.wasm");
}
```

---

## 🧪 End-to-End Test

**Test File**: `examples/first_fullstack_app.raven`

```raven
// Server function
@server
fn get_greeting(name: String) -> String {
    return "Hello, " + name + "!";
}

// Client component
@client
component Greeter() {
    let name = Signal::new("");
    let greeting = Signal::new("");
    
    let fetch_greeting = async || {
        let result = get_greeting(name.get()).await;
        greeting.set(result);
    };
    
    <div>
        <input
            value={name.get()}
            oninput={|e| name.set(e.target.value)}
        />
        <button onclick={fetch_greeting}>Greet</button>
        <p>{greeting.get()}</p>
    </div>
}
```

**Compile**:
```bash
cargo run -- examples/first_fullstack_app.raven
# Generates: server.js, client.js, app.wasm
```

**Run**:
```bash
node server.js  # Start server on :3000
# Open index.html in browser
# Type name, click "Greet", see server response!
```

---

## ✅ Success Criteria

1. Can write `.raven` files with `@server` and `@client` annotations
2. Compiler generates `server.js`, `client.js`, and `app.wasm`
3. Server functions are accessible via RPC from client
4. Type checking works across server/client boundary
5. Shared code is included in both bundles
6. Everything runs and actually works!

---

**Estimated Timeline**: 2-3 weeks for basic version
**First Milestone**: Working hello-world fullstack app
**Final Milestone**: Can build real apps like todo lists, blogs, etc.

Let's do this! 🚀
