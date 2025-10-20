# 5-Minute Tutorial

This tutorial shows the current end-to-end RavensOne workflow. Expect rough edges: the language is still stabilising, and several commands emit warnings when features are not yet implemented.

## 0: Prep (30 seconds)

```bash
git clone https://github.com/jrezin1201/RavensOne
cd RavensOne
cargo install --path .
```

Ensure you have Node.js 18+ installed for previewing output.

## 1: Initialise (45 seconds)

```bash
mkdir raven-demo
cd raven-demo
raven init
```

The CLI creates:

```
raven.toml
src/main.raven
```

`src/main.raven` contains a single `App` component using only supported syntax (annotations, tags, simple expressions).

## 2: Edit the component (90 seconds)

Replace the file contents with:

```raven
// demo: simple counter component
component App() {
    let title = "RavensOne Counter";
    let start = 0;

    <main class="stack">
        <h1>{title}</h1>
        <p>Static counter: {start}</p>
    </main>
}
```

Save the file. The current parser handles literals, identifiers, component tags, and interpolated expressions. Avoid advanced control flow until parser milestones land (see [roadmap](../../audit-top-20.json)).

## 3: Run the dev server (75 seconds)

```bash
raven dev --port 5173
```

What happens:

1. The watcher recompiles `src/*.raven` into `.wasm`/JS artifacts.
2. Python-based static server (`serve.py`) serves `dist/` on the chosen port.
3. HMR stub prints startup logs (hot reload is not yet wired).

Open the displayed URL to preview the rendered HTML. You may need to refresh manually after edits.

## 4: Build for distribution (45 seconds)

```bash
raven build --release
```

The release flag enables minification stubs and writes bundles to `dist/`. Inspect `dist/index.html`, `server.js`, and `client.js`. These files are prototypes; expect manual tweaks when preparing demos.

## 5: Share feedback (15 seconds)

- File issues with repro `.raven` files.
- Join the Discord (link forthcoming) to follow parser/codegen progress.
- Contribute docs fixes via pull requests following [Contributing](../contributing.md).

> ℹ️ Known gaps: package registry commands, auth/ORM modules, and SSR are placeholders. The CLI surfaces TODO notices where functionality is missing.
