# Contributing

Thanks for helping shape RavensOne! This document outlines expectations for contributors and reviewers.

## Development Environment

1. Install Rust (1.74+) and Node.js 18+.
2. Clone the repository and install the CLI locally:
   ```bash
   git clone https://github.com/jrezin1201/RavensOne
   cd RavensOne
   cargo install --path .
   ```
3. Install optional tooling:
   ```bash
   cargo install cargo-nextest
   npm install
   ```

## Workflow

1. Create a feature branch off `main`.
2. Make focused commits following [Conventional Commits](https://www.conventionalcommits.org/).
3. Run the validation commands (see below).
4. Open a pull request that includes:
   - Summary and rationale.
   - Before/After screenshots or CLI logs when relevant.
   - Acceptance criteria checklist.

## Required Checks

```bash
cargo fmt
cargo clippy --no-deps
cargo test
npm run docs:check
```

Document deviations (e.g., missing dependencies) in the PR description.

## Commit Message Examples

- `feat(lexer): support multiline string literals`
- `fix(parser): handle nested JSX fragments`
- `docs: add quickstart tutorial`

## Code Style

- Prefer TypeScript and Rust.
- Avoid introducing global mutable state.
- Keep dependencies minimal; prefer stdlib and small crates.

## Filing Issues

Include a reproduction `.raven` file, CLI command, and observed vs expected behaviour. Tag with the appropriate area label (compiler, cli, docs, devexp, infra).

## Release Process

Releases are grouped by milestone. Update [docs/changelog.md](./changelog.md) and tag commits with `vX.Y.Z` when publishing crates or npm packages.

Happy hacking!
