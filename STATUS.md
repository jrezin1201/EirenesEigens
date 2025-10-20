# RavensOne Status Snapshot

_Last updated: October 2025 (synchronised with docs consolidation)_

RavensOne is in pre-alpha. Core compiler pieces exist but many subsystems remain prototypes or stubs. Use this page to understand what works today and where help is needed.

## Current Focus

1. **Stabilise compiler pipeline** — tighten lexer/parser coverage and align code generation with the planned showcase roadmap.
2. **Developer experience** — replace placeholder dev server/watch behaviour with reliable tooling.
3. **Documentation refresh** — maintain the new lean IA and keep the audit list up to date.

## Feature Readiness

| Area | Status | Notes |
| --- | --- | --- |
| Lexer | ⚠️ Prototype | Handles annotations/components; string escapes & comments pending. |
| Parser | ⚠️ Prototype | Pratt parser covers core expressions; error recovery limited. |
| AST & Analysis | 🚧 In progress | Semantic/type/borrow checkers contain significant stubs. |
| Code Generation | ⚠️ Prototype | JS/HTML emitter works for basic components; WASM optional. |
| CLI | ⚠️ Prototype | `build`, `compile`, `dev`, `init`, `watch` usable; other commands log TODOs. |
| Package Manager | ❌ Not implemented | Subcommands exist but talk to placeholder registry APIs. |
| VS Code Extension | 🚧 In progress | Grammar stub planned (see audit item A07). |

Legend: ✅ complete, ⚠️ functional but rough, 🚧 actively developing, ❌ not started.

## Recent Highlights

- Adopted new documentation structure with guide, reference, contributing, and changelog sections.
- Published [audit-top-20.json](audit-top-20.json) capturing priority tasks.
- Authored [showcase-roadmap.md](showcase-roadmap.md) defining the first 20 demo projects.

## Upcoming Milestones

1. **Compiler hardening** — implement lexer/parsing improvements (audit A02/A03).
2. **Codegen MVP** — deliver static HTML output for the "Hello Component" showcase (audit A12).
3. **Dev server v1** — replace Python/Node shim with Rust-based server + watcher (audit A06).

## How to Help

- Grab an item from [audit-top-20.json](audit-top-20.json) and open an issue/PR.
- Improve documentation accuracy using the [contributing guide](docs/contributing.md).
- Prototype showcase examples under `examples/` following the [showcase roadmap](showcase-roadmap.md).

Thanks to everyone exploring the edges of this experiment!
