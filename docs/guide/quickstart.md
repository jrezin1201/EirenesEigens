# Quickstart

Follow these steps to install the experimental RavensOne toolchain and compile a minimal component. The quickstart mirrors the demo flow used for pre-release showcases.

## 1. Install prerequisites

- Rust toolchain (1.74 or newer)
- Node.js 18+ (for running generated bundles)
- npm or pnpm (optional, for VS Code extension testing)

Clone and build the CLI:

```bash
git clone https://github.com/jrezin1201/RavensOne
cd RavensOne
cargo install --path .
```

Verify the binary:

```bash
raven --version
```

## 2. Scaffold a workspace

Create a new folder and initialize a project:

```bash
mkdir demo-app
cd demo-app
raven init
```

`raven init` generates:

```
.
├── raven.toml
├── src/
│  └── main.raven
└── README.md
```

## 3. Compile

Compile the default component to static assets:

```bash
raven build
```

The command emits files to `dist/`. At this stage the build pipeline produces prototype JavaScript/HTML artefacts intended for demos. Expect limitations noted in the CLI output.

## 4. Preview

Serve the generated files using your preferred static server (for example `npx serve dist`). The dev server workflow is evolving; see the [5-minute tutorial](./tutorial-5-minutes.md) for the most up-to-date guidance.

## 5. Next steps

- Customize `src/main.raven` and re-run `raven build`.
- Try `raven dev` to experiment with the file watcher (pre-alpha, expect rough edges).
- Review [Concepts](./concepts.md) to understand the pipeline.
