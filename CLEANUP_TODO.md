# RavensOne Cleanup TODO

## ✅ COMPLETED: Remove Duplicate Registry Client Code (Oct 17, 2025)

### Problem (RESOLVED)

We currently have **TWO registry clients** that do the same thing:

1. **Old**: `src/registry_client.rs` (284 lines)
   - Used by top-level CLI commands
   - Token stored in `.raven/token`
   - Basic implementation

2. **New**: `src/package_manager/registry.rs` (575 lines)
   - Used by `raven pkg` subcommands
   - Token stored in `~/.raven/credentials.json`
   - More comprehensive with better error handling
   - Better security (file permissions)

### Duplicate CLI Commands

Users can currently run the same operation two ways:

```bash
# OLD WAY (using registry_client.rs)
raven login --registry http://localhost:4000
raven register --registry http://localhost:4000
raven publish --registry http://localhost:4000
raven search "query" --registry http://localhost:4000
raven install <package> --registry http://localhost:4000

# NEW WAY (using package_manager/registry.rs)
raven pkg login
raven pkg register
raven pkg publish
raven pkg search "query"
# Install handled differently through pkg add/install
```

### Recommended Solution

**Keep the NEW implementation**, remove the OLD:

1. ✅ **Keep**: `src/package_manager/registry.rs` (better implementation)
2. ❌ **Delete**: `src/registry_client.rs` (old implementation)
3. ❌ **Remove from Commands enum** (lines 77-105 in main.rs):
   - `Login { registry: String }`
   - `Register { registry: String }`
   - `Publish { registry: String }`
   - `Search { query: String, registry: String }`
   - `Install { package: String, version: Option<String>, registry: String }`

4. ❌ **Remove handler functions** (lines 773-1067 in main.rs):
   - `fn login_to_registry(...)`
   - `fn register_with_registry(...)`
   - `fn publish_package(...)`
   - `fn search_packages(...)`
   - `fn install_package(...)`

5. ❌ **Remove from main.rs match statement** (lines 273-307):
   - `Commands::Login { registry } => ...`
   - `Commands::Register { registry } => ...`
   - `Commands::Publish { registry } => ...`
   - `Commands::Search { query, registry } => ...`
   - `Commands::Install { package, version, registry } => ...`

6. ✅ **Update lib.rs** - Remove line 29:
   ```rust
   pub mod registry_client; // DELETE THIS LINE
   ```

### Migration Steps

```bash
# 1. Delete old registry client
rm src/registry_client.rs

# 2. Edit src/lib.rs - remove line 29
# Remove: pub mod registry_client;

# 3. Edit src/main.rs
# Remove lines 77-105 (duplicate Commands)
# Remove lines 273-307 (duplicate match arms)
# Remove lines 773-1067 (duplicate handler functions)

# 4. Test compilation
cargo check

# 5. Update documentation
# Update any docs that reference "raven login" to "raven pkg login"
```

### Files to Modify

1. `src/registry_client.rs` - **DELETE**
2. `src/lib.rs` - Remove module declaration (line 29)
3. `src/main.rs` - Remove duplicate commands and handlers (~350 lines)

### Testing Checklist

After cleanup, test these commands:

- [x] ✅ **COMPLETED** - Deleted `src/registry_client.rs`
- [x] ✅ **COMPLETED** - Removed module declaration from `src/lib.rs`
- [x] ✅ **COMPLETED** - Removed duplicate Commands enum entries (lines 77-105)
- [x] ✅ **COMPLETED** - Removed duplicate match arms (lines 273-307)
- [x] ✅ **COMPLETED** - Removed duplicate handler functions (~300 lines)
- [x] ✅ **COMPLETED** - `cargo check` compiles successfully with no errors
- [ ] `raven pkg login` - Should work (needs registry server running)
- [ ] `raven pkg register` - Should work (needs registry server running)
- [ ] `raven pkg publish` - Should work (needs registry server running)
- [ ] `raven pkg search "test"` - Should work (needs registry server running)

### Benefits of Cleanup

1. **Less confusion** - Only one way to do registry operations
2. **Better UX** - Consistent `raven pkg` namespace for package operations
3. **Cleaner codebase** - ~650 lines of duplicate code removed
4. **Better security** - Uses newer implementation with proper permissions
5. **Maintainability** - Only one registry client to maintain

### Estimated Time

- **15-20 minutes** for experienced developer
- Low risk - mostly deletions

### Related Files

- `docs-site/public/getting-started.html` - May reference old commands
- `README.md` - May reference old commands
- `REGISTRY_API_SPEC.md` - Already documents correct usage

---

## 🟡 Medium Priority: Other Cleanup Tasks

### 1. Remove Unused Imports

Several files have unused import warnings:
- `src/router.rs` - Unused imports (create_effect, Rc, RefCell)
- `src/forms.rs` - Unused `super::*`
- `src/diagnostics.rs` - Unused imports
- `src/lsp/mod.rs` - Unused imports

**Fix**: Run `cargo fix --allow-dirty --allow-staged`

### 2. Remove Unused Doc Comments

- `src/stdlib/reactive.rs:14` - Unused doc comment
- `src/reactive.rs:11` - Unused doc comment

**Fix**: Remove or attach to actual items

### 3. Registry Server Warnings

The registry server has some unused functions:
- `validate_package_name` in `registry/src/validation.rs:11`
- `validate_version` in `registry/src/validation.rs:37`

**Fix**: Either use them in validation or remove them

---

## 🟢 Low Priority: Nice-to-Have Improvements

### 1. Consolidate Error Types

Consider merging `PackageError` and `RegistryError` into a unified error type.

### 2. Add Integration Tests

Add end-to-end tests for:
- Publishing a package
- Downloading a package
- Searching packages

### 3. Add CLI Help Examples

Add examples to CLI help text:
```rust
/// Login to the package registry
///
/// Example: raven pkg login
Login,
```

### 4. Environment Variable for Registry URL

Allow users to set default registry:
```bash
export RAVEN_REGISTRY=http://localhost:8080
raven pkg login  # Uses RAVEN_REGISTRY
```

---

## 📊 Current Status

**Project Completion**: ~85%

✅ **Complete**:
- Compiler & type system
- Package manager (init, add, remove, update, login, publish)
- Registry server (100% - all handlers implemented!)
- Documentation site (4 pages)
- VSCode extension
- 8 seed packages
- Debugging tools

⚠️ **Needs Cleanup**:
- Duplicate registry client code

⏳ **Not Started**:
- SSR implementation
- Testing framework
- Performance benchmarks

---

## 🎯 Next Steps After Cleanup

1. **Test Registry End-to-End**
   - Start registry server
   - Publish seed packages
   - Test installation

2. **Deploy**
   - Deploy registry to production
   - Deploy docs site to Vercel
   - Publish announcement

3. **New Features**
   - Server-Side Rendering
   - Testing framework
   - Advanced routing

---

*Created: October 17, 2025*
*Priority: High - Should be done before v1.0 release*
