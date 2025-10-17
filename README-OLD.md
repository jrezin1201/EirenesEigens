# RavensOne

**The Full-Stack Programming Language for Human-AI Collaboration**

RavensOne is a revolutionary language designed from the ground up to maximize productivity when humans and AI work together. It eliminates the "dependency hell" and complexity of modern web development by providing a complete, batteries-included stack in a single unified language.

## Why RavensOne?

Modern web development requires juggling dozens of technologies:
- Frontend: TypeScript, React, Tailwind, Webpack, Vite
- Backend: Node.js, Express, databases, ORMs
- DevOps: Docker, Kubernetes, CI/CD pipelines
- Testing: Jest, Playwright, Cypress

**This complexity kills AI productivity.** Claude and other LLMs generate excellent code, but coordinating across this fragmented ecosystem slows everything down.

### RavensOne's Solution

✅ **Single file type** - Only `.raven` files
✅ **Complete stack** - UI, API, database, auth built-in
✅ **Edge-first** - Compiles to WASM, deploys to Cloudflare Workers
✅ **Reactive by default** - `let count = 0; count++` auto-updates UI
✅ **Strong typing** - Rust-like safety without the ceremony
✅ **Instant deployment** - `raven deploy` → production in seconds

## Quick Start

### Installation

```bash
git clone https://github.com/yourusername/ravensone
cd ravensone
cargo build --release
```

### Your First RavensOne App

Create `counter.raven`:

```raven
// Reactive state
let count = 0;

// Component with JSX
component Counter() {
    return <div class="container">
        <p>"Count: "</p>
        <p>{count}</p>
        <button>"Increment"</button>
    </div>;
}
```

### Compile and Run

```bash
# Compile to WASM
./target/release/raven compile counter.raven

# Start dev server
cd runtime
./serve.sh

# Open http://localhost:8000 in your browser
```

## Current Status

🎉 **MVP Complete!** The core language and runtime are working.

### ✅ Implemented

- [x] Lexer & Parser with JSX support
- [x] Semantic analyzer with type checking
- [x] Borrow checker for memory safety
- [x] WASM code generation
- [x] Component syntax (`component Name() { }`)
- [x] JSX with interpolation (`{expr}`)
- [x] JavaScript runtime bridge
- [x] Reactive state foundations

### 🚧 In Progress

- [ ] Full VDOM rendering from WASM
- [ ] Event handlers (`onclick`, `onchange`)
- [ ] Database ORM & migrations
- [ ] HTTP routing & middleware
- [ ] Authentication & sessions
- [ ] Cloudflare Workers deployment
- [ ] Hot module replacement
- [ ] TypeScript-level tooling (LSP, formatter)

## Architecture

### Compilation Pipeline

```
.raven file
    ↓ Lexer
Tokens
    ↓ Parser
AST
    ↓ Semantic Analyzer
Typed AST
    ↓ Borrow Checker
Safe AST
    ↓ Code Generator
WASM Bytecode
```

### Runtime Architecture

```
WASM Module ←→ JavaScript Runtime ←→ Browser DOM
                      ↓
              Reactive State Manager
                      ↓
                Event Handlers
```

## Language Features

### Strong Static Typing

```raven
let count: i32 = 0;
let name: string = "Alice";
let active: bool = true;
let scores: Array<i32> = [1, 2, 3];
```

### Reactive State

```raven
let count = 0;  // Reactive by default

// Mutation triggers UI update
count = count + 1;
```

### Components

```raven
component Button(label: string, onClick: fn()) {
    return <button onclick={onClick}>{label}</button>;
}
```

### Server Functions (Coming Soon)

```raven
server fn getUser(id: i32) -> User {
    return db.users.findById(id);
}

// Automatically becomes RPC call on client
let user = getUser(123);
```

## Project Structure

```
ravensone/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── lib.rs           # Compiler library
│   ├── lexer.rs         # Tokenization
│   ├── parser.rs        # AST construction
│   ├── ast.rs           # AST definitions
│   ├── semantic_analyzer.rs  # Type checking
│   ├── borrow_checker.rs     # Memory safety
│   ├── codegen.rs       # WASM generation
│   ├── vdom.rs          # Virtual DOM
│   └── deployer.rs      # Cloud deployment
├── runtime/
│   ├── ravensone.js     # JavaScript runtime
│   ├── index.html       # Test page
│   └── serve.sh         # Dev server
├── examples/
│   ├── simple.raven
│   └── counter-v2.raven
└── README.md
```

## CLI Commands

```bash
# Compile a .raven file to WASM
raven compile <file.raven> [--output <file.wasm>]

# Create a new project
raven new <project-name>

# Deploy to Cloudflare Workers (coming soon)
raven deploy [--env production]
```

## Philosophy

RavensOne is built on three core principles:

1. **AI-First Design** - Every decision optimizes for human-AI collaboration
2. **Batteries Included** - Everything you need ships with the language
3. **Zero Ceremony** - Minimal boilerplate, maximum productivity

## Roadmap

### Phase 1: Core Language ✅ **DONE**
- Compiler pipeline (lexer → parser → codegen)
- Component syntax
- JSX support
- Basic WASM output

### Phase 2: Reactivity (In Progress)
- Full reactive state system
- VDOM diffing and patching
- Event handling
- Component lifecycle

### Phase 3: Full Stack
- HTTP routing
- Database ORM
- Authentication
- Real-time (WebSockets)

### Phase 4: Deployment
- Cloudflare Workers adapter
- Deno Deploy support
- Edge caching
- CDN integration

### Phase 5: Tooling
- Language Server Protocol
- VS Code extension
- Formatter & linter
- Debugger

## Contributing

RavensOne is in active development. Contributions welcome!

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request

## License

MIT License - See LICENSE file for details

## Credits

Built with ❤️ by developers who believe AI should accelerate, not complicate, software development.

---

**"One language. One file type. Full stack. Maximum velocity."**
