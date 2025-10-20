# RavensOne Package Manifest Specification

**Version**: 1.0.0
**Format**: TOML
**Filename**: `raven.toml`

---

## Overview

The `raven.toml` file is the package manifest for RavensOne projects. It defines project metadata, dependencies, build configuration, and deployment settings.

## File Structure

```toml
[package]
# Package metadata

[dependencies]
# Runtime dependencies

[dev-dependencies]
# Development-only dependencies

[build]
# Build configuration

[features]
# Optional features

[profile.dev]
# Development profile

[profile.release]
# Release profile
```

---

## [package] Section

Defines package metadata.

### Required Fields

```toml
[package]
name = "my-app"              # Package name (lowercase, hyphens allowed)
version = "0.1.0"            # Semantic version (MAJOR.MINOR.PATCH)
authors = ["Name <email@example.com>"]  # List of authors
```

### Optional Fields

```toml
[package]
description = "A full-stack app built with RavensOne"
homepage = "https://example.com"
repository = "https://github.com/user/repo"
license = "MIT"
keywords = ["web", "fullstack", "reactive"]
categories = ["web-programming", "gui"]
readme = "README.md"
edition = "2025"             # RavensOne edition (default: 2025)
```

### Example

```toml
[package]
name = "todo-app"
version = "1.0.0"
authors = ["Alice <alice@example.com>", "Bob <bob@example.com>"]
description = "A reactive todo list application"
homepage = "https://todo-app.example.com"
repository = "https://github.com/alice/todo-app"
license = "MIT"
keywords = ["todo", "productivity", "reactive"]
readme = "README.md"
edition = "2025"
```

---

## [dependencies] Section

Runtime dependencies required by the package.

### Format

```toml
[dependencies]
package-name = "version"              # Simple version
package-name = { version = "1.0" }    # Extended format
package-name = { git = "url" }        # Git dependency
package-name = { path = "../local" }  # Local path dependency
```

### Version Constraints

| Operator | Meaning | Example |
|----------|---------|---------|
| `1.2.3` | Exact version | `"1.2.3"` |
| `^1.2.3` | Compatible (default) | `"^1.2.3"` = `>=1.2.3 <2.0.0` |
| `~1.2.3` | Minor updates only | `"~1.2.3"` = `>=1.2.3 <1.3.0` |
| `>=1.2.3` | Greater than or equal | `">=1.2.3"` |
| `<2.0.0` | Less than | `"<2.0.0"` |
| `*` | Any version | `"*"` |

### Example

```toml
[dependencies]
raven-ui = "^1.0.0"              # UI component library
raven-router = "~2.1.0"          # Routing library
raven-http = { version = "0.5", features = ["json"] }
my-shared-lib = { path = "../shared" }
experimental-lib = { git = "https://github.com/user/lib" }
```

---

## [dev-dependencies] Section

Dependencies used only during development (testing, benchmarks).

```toml
[dev-dependencies]
raven-test = "^0.5.0"            # Testing framework
raven-bench = "^0.3.0"           # Benchmarking tools
```

---

## [build] Section

Build configuration and compiler options.

```toml
[build]
target = "wasm32-unknown-unknown"    # Compilation target
entry = "src/main.raven"             # Entry point file
output-dir = "dist"                  # Output directory

# Optimization settings
optimize = true                      # Enable optimizations
minify = true                        # Minify output
source-maps = true                   # Generate source maps

# Server-side rendering
ssr = true                           # Enable SSR
hydrate = true                       # Enable hydration

# Code splitting
code-split = true                    # Enable code splitting
chunk-size-limit = 500               # Max chunk size in KB
```

### Example

```toml
[build]
target = "wasm32-unknown-unknown"
entry = "src/app.raven"
output-dir = "build"
optimize = true
minify = true
source-maps = true
ssr = true
hydrate = true
code-split = true
chunk-size-limit = 250
```

---

## [features] Section

Optional features that can be enabled/disabled.

```toml
[features]
default = ["ssr", "router"]       # Default features
ssr = []                           # Server-side rendering
csr = []                           # Client-side rendering only
router = ["raven-router"]          # Routing (enables dependency)
animations = ["raven-animation"]   # Animation system
forms = ["raven-forms"]            # Form validation
```

### Using Features

```toml
[dependencies]
my-lib = { version = "1.0", features = ["ssr", "router"] }
```

### Enable Feature via CLI

```bash
raven build --features ssr,router
raven build --no-default-features --features csr
```

---

## [profile.dev] Section

Development profile (default for `raven dev`).

```toml
[profile.dev]
optimize = false                  # No optimization (fast builds)
debug = true                      # Include debug info
source-maps = true                # Generate source maps
hot-reload = true                 # Enable HMR
watch = true                      # Watch for file changes
```

---

## [profile.release] Section

Release profile (default for `raven build --release`).

```toml
[profile.release]
optimize = true                   # Full optimization
minify = true                     # Minify output
debug = false                     # No debug info
source-maps = false               # No source maps
tree-shake = true                 # Remove dead code
inline-threshold = 25             # Inline functions < 25 lines
```

---

## Complete Example

```toml
[package]
name = "ecommerce-app"
version = "2.1.0"
authors = ["Dev Team <dev@company.com>"]
description = "Full-stack e-commerce platform built with RavensOne"
homepage = "https://shop.example.com"
repository = "https://github.com/company/ecommerce"
license = "MIT"
keywords = ["ecommerce", "shop", "cart"]
edition = "2025"

[dependencies]
raven-ui = "^1.2.0"
raven-router = "~2.0.0"
raven-forms = { version = "0.8", features = ["validation"] }
raven-http = "^0.5.0"
stripe = { git = "https://github.com/stripe/stripe-raven" }
shared-components = { path = "../components" }

[dev-dependencies]
raven-test = "^0.5.0"
raven-bench = "^0.3.0"

[build]
target = "wasm32-unknown-unknown"
entry = "src/main.raven"
output-dir = "dist"
optimize = true
minify = true
source-maps = true
ssr = true
hydrate = true
code-split = true
chunk-size-limit = 300

[features]
default = ["ssr", "router", "forms"]
ssr = []
router = ["raven-router"]
forms = ["raven-forms"]
payments = ["stripe"]
analytics = []

[profile.dev]
optimize = false
debug = true
source-maps = true
hot-reload = true
watch = true

[profile.release]
optimize = true
minify = true
debug = false
source-maps = false
tree-shake = true
inline-threshold = 30
```

---

## Lock File (`raven.lock`)

The lock file ensures reproducible builds by locking exact dependency versions.

### Format

```toml
# This file is automatically generated by raven pkg
# Do not edit manually

[[package]]
name = "raven-ui"
version = "1.2.5"
source = "registry+https://registry.ravensone.dev/"
checksum = "a3f9b8c7e2d1..."
dependencies = []

[[package]]
name = "my-app"
version = "2.1.0"
dependencies = [
    "raven-ui 1.2.5 (registry+https://registry.ravensone.dev/)",
]
```

### Updating Dependencies

```bash
# Update all dependencies
raven pkg update

# Update specific package
raven pkg update raven-ui

# Update to latest compatible version
raven pkg update --compatible

# Update to latest version (may break compatibility)
raven pkg update --latest
```

---

## Workspace Support

For multi-package projects (monorepos).

### Workspace `raven.toml`

```toml
[workspace]
members = [
    "packages/ui",
    "packages/router",
    "packages/forms",
    "apps/web",
    "apps/admin",
]

exclude = [
    "experiments/*"
]

[workspace.dependencies]
# Shared dependency versions
raven-utils = "0.3.0"
```

### Member Package `raven.toml`

```toml
[package]
name = "my-ui-lib"
version = "1.0.0"

[dependencies]
raven-utils = { workspace = true }  # Use workspace version
```

---

## CLI Commands

### Initialize New Project

```bash
raven pkg init
raven pkg init --name my-app
raven pkg init --template fullstack
```

### Install Dependencies

```bash
raven pkg install              # Install all dependencies
raven pkg install raven-ui     # Add and install package
raven pkg install --dev raven-test  # Add dev dependency
```

### Update Dependencies

```bash
raven pkg update               # Update all
raven pkg update raven-ui      # Update specific package
```

### Remove Dependencies

```bash
raven pkg remove raven-ui
```

### Publish Package

```bash
raven pkg publish
raven pkg publish --dry-run    # Test without publishing
```

### Search Registry

```bash
raven pkg search ui
raven pkg search --keyword reactive
```

---

## Package Registry

### Publishing Requirements

1. Valid `raven.toml` with required fields
2. Semantic version number
3. README.md file
4. License specified
5. No Git working directory changes (clean state)

### Registry URL

```
https://registry.ravensone.dev/
```

### Authentication

```bash
raven pkg login
# Opens browser for GitHub OAuth
```

---

## Best Practices

### Versioning

- Follow semantic versioning (SemVer)
- Breaking changes = MAJOR bump
- New features = MINOR bump
- Bug fixes = PATCH bump

### Dependencies

- Use `^` for libraries (e.g., `^1.2.0`)
- Use `~` for stability (e.g., `~1.2.0`)
- Pin exact versions for critical security deps

### Features

- Keep `default` features minimal
- Document feature flags in README
- Test with different feature combinations

### Profiles

- Use `dev` for fast iteration
- Use `release` for production
- Create custom profiles for CI/CD

---

## Migration Guide

### From npm/package.json

```json
// package.json
{
  "name": "my-app",
  "version": "1.0.0",
  "dependencies": {
    "react": "^18.0.0"
  }
}
```

```toml
# raven.toml
[package]
name = "my-app"
version = "1.0.0"

[dependencies]
raven-ui = "^1.0.0"  # Similar to React
```

### From Cargo.toml

```toml
# Cargo.toml
[package]
name = "my-app"
version = "1.0.0"

[dependencies]
serde = "1.0"
```

```toml
# raven.toml (very similar!)
[package]
name = "my-app"
version = "1.0.0"

[dependencies]
raven-json = "^1.0.0"
```

---

## See Also

- [Package Manager CLI Reference](./docs/cli-reference.md)
- [Publishing Guide](./docs/publishing.md)
- [Workspace Guide](./docs/workspaces.md)
- [Registry API](./docs/registry-api.md)

---

**Version**: 1.0.0
**Last Updated**: October 17, 2025
**Status**: Draft Specification
