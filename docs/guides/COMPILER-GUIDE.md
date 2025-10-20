# RavensOne Compiler Guide

## Complete Compilation Pipeline Documentation

### Overview
The RavensOne compiler transforms `.raven` source code into WebAssembly bytecode.

---

## Compilation Stages

### Stage 1: Lexical Analysis (Lexer)
**File:** `src/lexer.rs`

Converts source code into tokens.

**Example Input:**
```raven
component App() {
    return <button />;
}
```

**Tokens Generated:**
- COMPONENT, IDENTIFIER("App"), LPAREN, RPAREN, LBRACE
- RETURN, LANGLE, IDENTIFIER("button"), SLASH, RANGLE
- SEMICOLON, RBRACE, EOF

---

### Stage 2: Syntax Analysis (Parser)
**File:** `src/parser.rs`

Builds Abstract Syntax Tree (AST) from tokens.

**Supported Syntax:**
- ✅ Component definitions
- ✅ JSX self-closing tags: `<button />`
- ✅ JSX with closing tags: `<h1>{"text"}</h1>`
- ✅ Expression children: `{expression}`
- ✅ Nested elements

---

### Stage 3: Code Generation
**File:** `src/codegen.rs`

Generates WebAssembly bytecode.

---

## Usage Examples

### Example 1: Simple Component

**Source:**
```raven
component Button() {
    return <button>{"Click Me"}</button>;
}
```

**Compile:**
```bash
./target/release/raven compile button.raven -o button.wasm
```

---

## Compilation Statistics

| Component Type | WASM Size |
|---------------|-----------|
| Empty element | 43 bytes |
| With text | 50-80 bytes |
| Nested elements | 80-150 bytes |

---

## Common Errors

### 1. "No prefix parse function for Slash"
**Fix:** Wrap text in curly braces
```raven
❌ <h1>"Hello"</h1>
✅ <h1>{"Hello"}</h1>
```

---

## Supported Features

### ✅ Working
- Component definitions
- JSX syntax (with expression children)
- Self-closing tags
- Nested elements
- Literals (string, int, float, bool)

### 🚧 In Progress
- Bare text children
- State management
- Event handlers

---

## Testing

```bash
cargo build --release
./target/release/raven compile test.raven
```

---

**Status:** ✅ Production Ready (Basic Features)
