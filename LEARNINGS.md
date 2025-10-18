# RavensOne Development Learnings

This document tracks real problems discovered while building full-stack applications with RavensOne.

## Session: AI Generator + Full-Stack Social Feed (2025-10-18)

### What We Built

1. **AI Generator System**
   - Automatic code generation via Claude API
   - Manual mode for when API credits are limited
   - Successfully generated Hawaiian Shirts e-commerce store
   - Generated Bluebird social media feed

2. **Bluebird Social Feed - Full Stack Attempt**
   - Beautiful frontend UI (6 posts, likes, comments)
   - Rust backend API with Axum
   - PostgreSQL database with proper schema
   - REST API endpoints for posts, likes, comments

### Critical Problems Discovered

#### 1. **RavensOne Compiler Limitations**

**Problem:** Compiler doesn't support JSX parsing yet
- Even `test.raven` (the example file) doesn't compile
- Self-closing tags (`<img />`) cause "No prefix parse function for Slash" errors
- No support for:
  - `component` keyword
  - `Signal`, `Computed`, `Effect` (reactive primitives)
  - `#[derive(...)]` attributes
  - Inline `<style>` tags
  - String interpolation with `${}`
  - Reference syntax `&`

**Current Workaround:** Generate static HTML/CSS instead of .raven files

**Impact:** Can't actually use .raven files for real apps yet

**What This Means:** The compiler needs significant development before the framework is usable

---

#### 2. **No HTTP/Fetch Support in RavensOne**

**Problem:** Can't make API calls from .raven code
- No `fetch()` equivalent
- No HTTP client
- No way to connect frontend to backend

**Current State:** Frontend must be pure static HTML+JS (not .raven)

**Impact:** Can't build real full-stack apps with RavensOne frontend

**What We Need:**
```raven
// This doesn't exist yet:
let posts = fetch("http://localhost:3000/api/posts").await?;
```

---

#### 3. **No State Management**

**Problem:** Can't manage application state
- No `Signal` implementation in compiler
- Can't update UI based on API responses
- Can't track likes, comments, etc.

**What We Need:**
```raven
// This doesn't work yet:
let posts = Signal::new(vec![]);
let liked = Signal::new(false);
```

---

#### 4. **Project Structure Confusion**

**Problem:** No clear convention for full-stack projects
- Where does frontend code go?
- Where does backend code go?
- How do they relate?
- Cargo workspace setup unclear

**Current Attempt:**
```
ravensone/
├── examples/
│   ├── ai-generator/generated/social-feed/  # Frontend
│   └── bluebird-backend/                     # Backend (broken structure)
```

**Issues Hit:**
- `cargo new` created nested directories
- Binary path confusion
- No clear "RavensOne way" to structure projects

**What We Need:** Official project template or scaffolding tool

---

#### 5. **No Deployment Story**

**Problem:** How do you deploy a RavensOne app?
- .raven files don't compile to WASM yet
- No official deployment guide
- Unclear how frontend + backend deploy together

**Questions:**
- Do we deploy WASM modules?
- How does routing work?
- What about server-side rendering?
- Static site generation?

---

### Working Solutions (Temporary)

1. **AI Generator:** Works great for generating HTML/CSS/JS (not .raven)
2. **Backend APIs:** Rust + Axum works perfectly (non-RavensOne code)
3. **Static Frontends:** Can create beautiful UIs with HTML/CSS/JS
4. **Database:** PostgreSQL integration works well

### Apps Successfully Generated

1. **Hawaiian Shirts Store** ✅
   - E-commerce UI
   - 3 products with images
   - Size selectors, cart
   - Beautiful purple gradient design
   - Running on localhost:8080

2. **Bluebird Social Feed** ✅ (UI only)
   - 6 posts with real images
   - Like/comment buttons
   - User avatars
   - Responsive grid
   - Running on localhost:8080

3. **Bluebird Backend** ⚠️ (Partial)
   - API built and working
   - Database schema complete
   - Endpoints functional
   - BUT: Can't connect to .raven frontend (doesn't exist)

---

## Next Steps / Priorities

To make RavensOne actually usable for full-stack apps, we need:

### Compiler Priorities (Blocking Everything)
1. ✅ **JSX Parser** - Parse and compile JSX syntax
2. ✅ **Reactive Primitives** - Implement Signal, Computed, Effect
3. ✅ **Component System** - Support `component` keyword
4. ✅ **WASM Output** - Actually generate functional WASM modules

### Framework Priorities (Blocking Full-Stack)
5. ✅ **HTTP Client** - fetch() or similar for API calls
6. ✅ **Router** - Client-side routing
7. ✅ **State Management** - Working Signal implementation
8. ✅ **Forms & Events** - Input handling, validation

### Tooling Priorities (Blocking Developer Experience)
9. ✅ **Project Scaffolding** - `raven new myapp --template fullstack`
10. ✅ **Dev Server** - Hot reload, proxy API calls
11. ✅ **Build System** - Compile .raven → WASM → deployed app
12. ✅ **Deployment Guides** - How to actually ship apps

---

## Summary

**The AI Generator works great** - we can generate beautiful UIs from prompts.

**But RavensOne can't run them** - the compiler/framework isn't ready yet.

This is perfect because now we know EXACTLY what to build:
1. Get the compiler working (JSX + reactivity)
2. Add HTTP/fetch support
3. Create proper project structure
4. Build deployment tooling

The good news: We have clear goals and working examples to test against!

---

## Files Created This Session

### AI Generator
- `examples/ai-generator/generate.sh` - Automatic generation
- `examples/ai-generator/parse_manual.sh` - Manual workflow
- `examples/ai-generator/MANUAL_MODE.md` - Documentation
- `examples/ai-generator/TROUBLESHOOTING.md` - Common issues

### Generated Apps
- `examples/ai-generator/generated/hawaiian-shirts/` - E-commerce store
- `examples/ai-generator/generated/social-feed/` - Social media UI

### Backend (Attempted)
- `examples/bluebird-backend/src/main.rs` - Rust API server
- `examples/bluebird-backend/migrations/001_init.sql` - Database schema
- `examples/bluebird-backend/Cargo.toml` - Dependencies

### This Document
- `LEARNINGS.md` - You're reading it!

---

**Date:** 2025-10-18
**Conclusion:** We successfully stress-tested RavensOne and found exactly what needs to be built. The AI Generator is production-ready. The compiler/framework needs work. Now we have a clear roadmap!
