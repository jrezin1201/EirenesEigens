# CLI Reference

`raven` is the entrypoint to the RavensOne toolchain. This document lists each subcommand, its arguments, and current implementation status.

| Command | Description | Status | Notes |
| --- | --- | --- | --- |
| `raven build [--release]` | Compile `.raven` sources into `dist/` artefacts. | ✅ Prototype | Generates JS/HTML; WASM optional. Release flag toggles minifier stubs. |
| `raven compile <file> [--output <dir>] [--minify]` | One-off compile of a specific file. | ⚠️ Prototype | Primary path used by internal tests. |
| `raven init [path]` | Initialise project in directory. | ✅ Prototype | Writes manifest + `src/main.raven`. |
| `raven new <name>` | Create new folder then initialise. | ⚠️ Needs polish | Shares templates with `init`; improve README scaffolding. |
| `raven dev [--port <port>]` | Run watcher + static server. | ⚠️ Experimental | Uses Python HTTP server + Node HMR stub. |
| `raven serve [--port <port>] [--open]` | Serve existing `dist/`. | ⚠️ Prototype | Wraps same Python server; auto-open not implemented. |
| `raven watch [path]` | Watch directory and compile on change. | ⚠️ Prototype | Busy-loop watcher; replace with native file notifications. |
| `raven test [--watch]` | Run compiler test suite. | 🚧 Planned | Stub command; prints placeholder logs. |
| `raven fmt [--check] [path]` | Format `.raven` files. | 🚧 Planned | Hooked to placeholder function; integrate formatter when ready. |
| `raven lint [--fix] [path]` | Lint `.raven` files. | 🚧 Planned | Currently no lint rules. |
| `raven doctor` | Environment diagnostics. | 🚧 Planned | Prints static checklist. |
| `raven deploy [--env <name>]` | Build and deploy to cloud. | 🚧 Planned | Calls stub in `src/deployer.rs`. |
| `raven pkg <subcommand>` | Manage dependencies. | 🚧 Planned | Subcommands exist but all call placeholder code. |

## Environment Variables

- `RAVEN_LOG` — Set to `debug` to enable verbose compiler output.
- `RAVEN_DEV_PORT` — Overrides default dev server port when `--port` is absent.

## Exit Codes

- `0` — Success.
- `1` — User or runtime error (e.g., failed IO, compilation error).

## See also

- [Quickstart](../guide/quickstart.md)
- [Configuration](./config.md)
