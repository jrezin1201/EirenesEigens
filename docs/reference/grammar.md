# Grammar Reference

This document captures the currently supported RavensOne syntax. It mirrors the token and parser implementations in `src/token.rs` and `src/parser.rs`.

## Keywords

```
fn
component
let
return
if
else
while
for
struct
enum
trait
impl
server
client
async
use
```

Annotations use `@server` and `@client`. Additional annotations parse as identifiers and are ignored unless handled by macros.

## Literals

- Numeric: decimal integers (`42`, `3_200`). Floating point tokens exist but evaluation is incomplete.
- Strings: double-quoted, no escape sequences beyond `\"` yet.
- Booleans: `true`, `false`.
- Nullish: `nil` (parsed as identifier today; reserved for future use).

## Operators

| Token | Meaning |
| --- | --- |
| `+`, `-`, `*`, `/` | Arithmetic |
| `==`, `!=`, `<`, `<=`, `>`, `>=` | Comparisons |
| `=` | Assignment |
| `&&`, `||` | Logical (planned) |
| `!` | Prefix not |

## Components

Components are declared with the `component` keyword or via `@client component` annotations:

```raven
component Header(title: String) {
    <header>
        <h1>{title}</h1>
    </header>
}
```

Supported JSX-like features:

- Start/end tags with lowercase names map to HTML.
- Uppercase tag names map to other components.
- Attributes accept string literals or `{expr}` placeholders.
- Self-closing tags (`<input />`) are supported by the lexer but emitter support is limited.

## Blocks and Statements

- `let name = expr;`
- `return expr;`
- `if (expr) { ... } else { ... }`
- `while (expr) { ... }`
- `for (init; cond; update) { ... }` (parsing stub; prefer `for item in list` once implemented)

## Imports

`use` statements allow `use path::to::item;` and `use path::{ItemA, ItemB};`.

## Limitations

- Pattern matching, async/await, and macros exist as AST nodes but lack execution semantics.
- Comments are tokenised but not preserved in AST.
- Error recovery is minimal; parser stops at first unexpected token.

Refer to [Concepts](../guide/concepts.md) for the pipeline roadmap and to [audit item A03](../../audit-top-20.json) for planned lexer improvements.
