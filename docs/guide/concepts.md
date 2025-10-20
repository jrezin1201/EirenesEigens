# Concepts

This page summarises the current RavensOne architecture so you can reason about limitations and plan contributions.

## Pipeline Overview

1. **Lexer** (`src/lexer.rs`)
   - Implements a token stream with support for annotations (`@server`, `@client`), component tags, literals, and operators.
   - Known gaps: string interpolation escapes, attribute parsing, comment handling.
2. **Parser** (`src/parser.rs`)
   - Pratt-style expression parser that recognises statements, components, and function definitions.
   - Missing support for loops with blocks, complex pattern matching, and advanced JSX spread syntax.
3. **AST** (`src/ast.rs`)
   - Defines program structure consumed by later stages. Some nodes are placeholders pending implementation (e.g., async blocks, router definitions).
4. **Semantic analysis & typing** (`src/semantic_analyzer.rs`, `src/type_checker.rs`)
   - Performs basic symbol tracking. Borrow checker/type checker contain stubs for future work.
5. **Code generation** (`src/codegen.rs`, `src/js_emitter.rs`)
   - Emits prototype JavaScript/TSX for server and client bundles. WebAssembly emission is experimental and should be treated as optional.
6. **CLI integration** (`src/main.rs`)
   - Exposes `raven build`, `raven dev`, and related commands. Many advanced commands log TODO notices.

## Runtime Model

- Components currently compile to functions returning HTML strings. Reactive/stateful features are roadmap items.
- Server/client separation uses annotations but RPC scaffolding is not wired yet.
- Dev server shells out to Python/Node helpers while a native Rust server is under development.

## File Types

- `.raven` — source files processed by the compiler.
- `raven.toml` — project manifest (see [config reference](../reference/config.md)).
- Generated assets — `dist/` contains experimental JS/HTML/WASM files.

## Limitations to Highlight During Demos

- Hot module replacement is not connected.
- Package manager (`raven pkg ...`) executes placeholder logic.
- Syntax errors may show generic messages; improving diagnostics is tracked in [audit item A04](../../audit-top-20.json).

Contributions that improve the fidelity of any stage are welcome. See [Contributing](../contributing.md) for workflow guidance.
