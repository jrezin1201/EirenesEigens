# RavensOne Package Manager - Complete Guide

The RavensOne package manager (`raven pkg`) provides a complete solution for managing dependencies, packages, and builds in your RavensOne projects.

---

## Table of Contents

1. [Getting Started](#getting-started)
2. [Package Management](#package-management)
3. [Developer Experience](#developer-experience)
4. [Build Cache](#build-cache)
5. [Registry Integration](#registry-integration)
6. [Command Reference](#command-reference)

---

## Getting Started

### Initializing a New Package

Create a new `raven.toml` manifest in your project:

```bash
raven pkg init
```

This creates a `raven.toml` file with package metadata:

```toml
[package]
name = "my-package"
version = "0.1.0"
authors = ["Developer <dev@example.com>"]
description = ""
license = "MIT"
repository = ""
homepage = ""
keywords = []

[dependencies]

[dev-dependencies]
```

---

## Package Management

### Installing Dependencies

Install all dependencies from `raven.toml`:

```bash
raven pkg install
```

**Output:**
```
📦 Resolving dependencies...
📥 Installing 5 packages...
  📥 Installing raven-store @ 1.0.0
  📥 Downloading raven-store v1.0.0
  ...
✅ All dependencies installed!
```

**Features:**
- ✅ Transitive dependency resolution
- ✅ Circular dependency detection
- ✅ Dependency deduplication
- ✅ Lock file generation (`raven.lock`)

### Adding Dependencies

Add a new dependency to your project:

```bash
raven pkg add raven-store
raven pkg add raven-http --version "^0.1.0"
raven pkg add test-pkg --dev  # Add to dev-dependencies
```

### Removing Dependencies

Remove a dependency:

```bash
raven pkg remove raven-store
```

### Updating Dependencies

Update all dependencies to latest compatible versions:

```bash
raven pkg update
```

This removes the lock file and re-resolves all dependencies to find the newest compatible versions.

---

## Developer Experience

### Viewing the Dependency Tree

Visualize your complete dependency graph:

```bash
raven pkg tree
```

**Output:**
```
full-stack-demo v0.1.0
├── raven-forms v1.0.0
│   └── raven-store v1.0.0
├── raven-http v0.1.0
├── raven-i18n v1.0.0
│   └── raven-store v1.0.0
├── raven-router v0.1.0
└── raven-store v1.0.0
```

**Features:**
- Beautiful Unicode tree visualization
- Shows transitive dependencies with nesting
- Alphabetically sorted for easy reading

### Checking for Outdated Packages

Check if any dependencies have newer versions:

```bash
raven pkg outdated
```

**Output:**
```
Checking for outdated dependencies...

📦 some-package
   Current: 1.0.0 | Latest: 2.0.0 | Wanted: ^1.0.0

💡 Run 'raven pkg update' to update to latest compatible versions
```

### Listing Installed Packages

View all installed packages:

```bash
raven pkg list
```

**Output:**
```
Installed packages:

📦 raven-store @ 1.0.0
📦 raven-i18n @ 1.0.0
   Dependencies: raven-store
📦 raven-forms @ 1.0.0
   Dependencies: raven-store
📦 raven-http @ 0.1.0
📦 raven-router @ 0.1.0

✅ Total: 5 packages
```

### Package Information

Get detailed information about a package:

```bash
raven pkg info raven-store
```

**Output:**
```
Fetching package information...

📦 raven-store
   Advanced state management library for RavensOne applications

Latest version: 1.0.0

Available versions:
   • 1.0.0

Statistics:
   Total downloads: 156
   Downloads (last month): 42
   Repository: https://github.com/aloha-shirts/raven-store

Keywords: state, reactive, store, signals

✅ Installed: v1.0.0
```

---

## Build Cache

The package manager includes an intelligent build cache system to speed up compilation.

### Cache Location

Build artifacts are cached in:
```
~/.raven/cache/
```

### Viewing Cache Statistics

```bash
raven pkg cache
```

**Output:**
```
Build Cache Statistics:

Location: /Users/you/.raven/cache
Cached packages: 3

Cached builds:
  📦 raven-store@1.0.0 (compiled 2h ago)
  📦 raven-http@0.1.0 (compiled 5m ago)
  📦 raven-forms@1.0.0 (compiled 1d ago)
```

### Clearing the Cache

```bash
raven pkg clean
```

**Output:**
```
✅ Build cache cleared
```

### How Caching Works

1. **Source Hashing**: Package source files are hashed to detect changes
2. **Cache Validation**: Before using cache, hashes are verified
3. **Automatic Invalidation**: Cache is invalidated when source changes
4. **Timestamp Tracking**: Shows when each package was last compiled

---

## Registry Integration

### Searching for Packages

Search the registry for packages:

```bash
raven pkg search http
```

**Output:**
```
Found 2 packages:

📦 raven-http @ 0.1.0
   HTTP client and server for RavensOne
   Keywords: http, client, server, api
   Downloads: 203 | Score: 8.50

📦 raven-api @ 1.2.0
   REST API utilities
   Keywords: api, rest, http
   Downloads: 89 | Score: 7.20
```

### Publishing Packages

First, register an account:

```bash
raven pkg register
```

Then login:

```bash
raven pkg login
```

Finally, publish your package:

```bash
raven pkg publish
```

**Requirements:**
- Valid `raven.toml` with all required fields
- Unique package name
- Valid semantic version
- Source files in `src/` directory

---

## Command Reference

### Core Commands

| Command | Description |
|---------|-------------|
| `raven pkg init` | Initialize new package manifest |
| `raven pkg install` | Install all dependencies |
| `raven pkg add <name>` | Add a dependency |
| `raven pkg remove <name>` | Remove a dependency |
| `raven pkg update` | Update dependencies to latest versions |

### Developer Tools

| Command | Description |
|---------|-------------|
| `raven pkg tree` | Display dependency tree |
| `raven pkg outdated` | Check for outdated dependencies |
| `raven pkg list` | List installed packages |
| `raven pkg info <name>` | Show package details |

### Registry

| Command | Description |
|---------|-------------|
| `raven pkg search <query>` | Search for packages |
| `raven pkg register` | Register new account |
| `raven pkg login` | Login to registry |
| `raven pkg publish` | Publish package |

### Build Cache

| Command | Description |
|---------|-------------|
| `raven pkg cache` | Show cache statistics |
| `raven pkg clean` | Clear build cache |

---

## File Structure

A typical RavensOne project with dependencies:

```
my-project/
├── raven.toml          # Package manifest
├── raven.lock          # Dependency lock file
├── src/
│   └── main.raven      # Source code
├── raven_packages/     # Downloaded dependencies
│   ├── raven-store/
│   ├── raven-http/
│   └── raven-forms/
└── dist/               # Compiled output
```

Global cache location:

```
~/.raven/
├── credentials.json    # Registry authentication
└── cache/              # Build cache
    ├── index.json      # Cache metadata
    └── builds/         # Compiled artifacts
```

---

## Advanced Features

### Transitive Dependencies

The package manager automatically resolves and installs transitive dependencies:

```toml
# Your raven.toml
[dependencies]
raven-i18n = "^1.0.0"
```

If `raven-i18n` depends on `raven-store`, both will be installed automatically.

### Circular Dependency Detection

The package manager detects circular dependencies and provides clear error messages:

```
❌ Circular dependency detected: package-a -> package-b -> package-c -> package-a
```

### Semantic Versioning

Version specifiers follow semver:

- `1.0.0` - Exact version
- `^1.0.0` - Compatible (>=1.0.0, <2.0.0)
- `~1.0.0` - Patch level (>=1.0.0, <1.1.0)
- `>=1.0.0` - Greater than or equal

### Lock File (`raven.lock`)

The lock file ensures reproducible builds:

```toml
version = "1"

[[packages]]
name = "raven-store"
version = "1.0.0"
dependencies = []

[packages.source]
type = "Registry"
url = "https://packages.ravensone.dev/raven-store/1.0.0"

[[packages]]
name = "raven-i18n"
version = "1.0.0"
dependencies = ["raven-store"]

[packages.source]
type = "Registry"
url = "https://packages.ravensone.dev/raven-i18n/1.0.0"
```

---

## Best Practices

### 1. Version Pinning

Use `^` for libraries (allows minor updates):
```toml
[dependencies]
raven-store = "^1.0.0"  # Allows 1.x.x
```

Use exact versions for critical dependencies:
```toml
[dependencies]
core-lib = "2.5.0"  # Exact version
```

### 2. Keep Dependencies Updated

Regularly check for outdated dependencies:

```bash
raven pkg outdated
raven pkg update
```

### 3. Use Lock Files

Always commit `raven.lock` to version control for reproducible builds.

### 4. Clean Cache Periodically

Clear old build artifacts:

```bash
raven pkg clean
```

### 5. Minimal Dependencies

Only add dependencies you actually need. Check with:

```bash
raven pkg tree  # See what you're really pulling in
```

---

## Troubleshooting

### Lock File Not Found

```
❌ raven.lock not found. Run 'raven pkg install' first.
```

**Solution**: Run `raven pkg install` to create the lock file.

### Circular Dependency

```
❌ Circular dependency detected: A -> B -> C -> A
```

**Solution**: Review your dependencies and break the circular reference.

### Package Not Found

```
❌ Package 'unknown-package' not found in registry
```

**Solution**:
- Check the package name spelling
- Search the registry: `raven pkg search unknown`
- Verify the package exists in the registry

### No Compatible Version

```
❌ No compatible version found for package @ ^2.0.0
```

**Solution**:
- Check available versions: `raven pkg info package`
- Adjust version requirement in `raven.toml`
- Update dependencies: `raven pkg update`

---

## Performance

### Cache Hit Rates

With caching enabled:
- **First install**: Full download and compilation
- **Subsequent installs**: Instant (from cache)
- **After source change**: Selective recompilation

### Parallel Operations

The package manager performs several operations in parallel:
- Dependency resolution
- Package downloads
- Cache validation

---

## Comparison with Other Package Managers

| Feature | RavensOne | npm | Cargo | pip |
|---------|-----------|-----|-------|-----|
| Transitive deps | ✅ | ✅ | ✅ | ✅ |
| Lock file | ✅ | ✅ | ✅ | ❌ |
| Dependency tree | ✅ | ✅ | ✅ | ❌ |
| Build cache | ✅ | ✅ | ✅ | ❌ |
| Outdated check | ✅ | ✅ | ✅ | ✅ |
| Package info | ✅ | ✅ | ✅ | ❌ |

---

## Future Roadmap

- [ ] Workspace support (monorepos)
- [ ] Private registry support
- [ ] Package aliasing
- [ ] Patch dependencies
- [ ] Dependency overrides
- [ ] Audit for security vulnerabilities
- [ ] Performance benchmarks

---

## Getting Help

- **Documentation**: https://docs.ravensone.dev/package-manager
- **Registry**: https://packages.ravensone.dev
- **Issues**: https://github.com/ravensone/ravensone/issues
- **Discord**: https://discord.gg/ravensone

---

**Happy coding with RavensOne! 🚀**
