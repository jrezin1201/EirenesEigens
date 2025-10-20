# RavensOne (pre-alpha)

RavensOne is an experimental language and toolchain exploring how a single `.raven` file could compile into both client and server artefacts. The current codebase focuses on the compiler core: lexer, Pratt parser, AST, and a prototype emitter that targets JavaScript/HTML.

> ⚠️ Everything in this repository is work in progress. Many modules are scaffolds and several CLI subcommands emit placeholder output. Use at your own risk.

## Why experiment with RavensOne?

- **Single-file ergonomics** – describe UI, server helpers, and shared utilities together.
- **Transparent pipeline** – written in Rust with clear stages (lex → parse → AST → analysis → emit).
- **Demo-first** – aim for compelling showcase projects that highlight compiler progress quickly.

## Quickstart

1. Install prerequisites:
   - Rust 1.74+
   - Node.js 18+
2. Install the CLI locally:
   ```bash
   git clone https://github.com/jrezin1201/RavensOne
   cd RavensOne
   cargo install --path .
   ```
3. Scaffold a project and build:
   ```bash
   mkdir demo-app && cd demo-app
   raven init
   raven build
   ```
4. Serve the generated `dist/` folder using your preferred static server (for example `npx serve dist`).

See [docs/guide/quickstart.md](docs/guide/quickstart.md) for more detail and the [5-minute tutorial](docs/guide/tutorial-5-minutes.md) for an end-to-end walkthrough.

## Project Status

| Area | Status |
| --- | --- |
| Lexer | Handles annotations, component tags, literals, and basic operators. Needs polish on strings/comments. |
| Parser | Pratt-style expression parser with statement coverage; lacks robust error recovery. |
| AST & Analysis | Core nodes exist; semantic/type/borrow checkers still contain stubs. |
| Codegen | Prototype emitter produces JavaScript/HTML with limited features. WASM pipeline is experimental. |
| CLI | `build`, `compile`, `init`, `dev`, and `watch` work in prototype form. Other subcommands are placeholders. |
| Tooling | VS Code grammar and dev server are early drafts. |

Refer to [audit-top-20.json](audit-top-20.json) for the highest-priority work items.

## Documentation

- [Guide index](docs/guide/index.md)
- [CLI reference](docs/reference/cli.md)
- [Grammar reference](docs/reference/grammar.md)
- [Configuration](docs/reference/config.md)
- [Contributing](docs/contributing.md)
- [Changelog](docs/changelog.md)
- [Showcase roadmap](showcase-roadmap.md)

Legacy docs now live in [docs/_archive](docs/_archive/INDEX.md).

## Contributing

We use Conventional Commits and expect contributors to run:

```bash
cargo fmt
cargo test
npm run docs:check
```

Read the [Contributing guide](docs/contributing.md) before opening a pull request. Every PR should include a Before/After summary and acceptance criteria.

## License

MIT © 2024 RavensOne contributors.
