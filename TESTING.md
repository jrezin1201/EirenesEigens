# RavensOne Testing Guide

## Quick Start

### 1. Start the Development Server

```bash
python3 serve.py
```

This will start a server at `http://localhost:8000`

### 2. Open Test Pages

#### Test 1: Reactive System Test
**URL:** `http://localhost:8000/test-reactive.html`

**What it tests:**
- ✅ Reactive Signal creation
- ✅ Effect registration and auto-updates
- ✅ WASM module loading
- ✅ Component exports
- ✅ Manual counter with reactivity

**Expected behavior:**
- Page loads with green "All systems operational!" status
- Clicking Increment/Decrement updates the count
- Count updates are logged to console
- Component is called successfully

#### Test 2: Basic WASM Loading
**URL:** `http://localhost:8000/test-wasm.html`

**What it tests:**
- ✅ Basic WASM loading
- ✅ Module instantiation
- ✅ Export discovery

#### Test 3: Full Runtime (Future)
**URL:** `http://localhost:8000/runtime/index.html`

**What it tests:**
- ✅ Complete counter component
- ✅ JSX rendering
- ✅ Event handlers
- ✅ Full reactive loop

## Compilation

### Compile Test Files

```bash
# Compile test component
cargo run --bin raven compile test-component.raven --output test-component.wasm

# Compile if statement test
cargo run --bin raven compile test-if.raven --output test-if.wasm

# Compile examples
cargo run --bin raven compile examples/simple-counter.raven --output examples/simple-counter.wasm
```

## Debugging

### Browser Console

Open DevTools (F12) and check:

1. **Console tab** - Look for:
   - `✅ Reactive runtime loaded`
   - `✅ Signal created`
   - `✅ Effect registered`
   - `✅ WASM loaded`
   - Signal update logs

2. **Network tab** - Verify:
   - `reactive-runtime.js` loads (200 OK)
   - `test-component.wasm` loads (200 OK)
   - Content-Type: `application/wasm`

### Common Issues

#### Issue: "Failed to fetch WASM"
- **Solution**: Make sure the dev server is running
- **Check**: WASM file exists in the correct location

#### Issue: "RavensReactive is not defined"
- **Solution**: Ensure `reactive-runtime.js` loads before test scripts
- **Check**: Script order in HTML

#### Issue: "Import object missing"
- **Solution**: Check that all WASM imports are provided
- **Check**: `importObject` in test HTML

## Test Checklist

### Phase 1: Foundation (Current)
- [x] Compiler generates valid WASM
- [x] WASM loads in browser
- [x] Reactive runtime initializes
- [x] Signals can be created
- [x] Effects run and update DOM
- [x] Component exports are callable

### Phase 2: Integration (Next)
- [ ] JSX renders to actual DOM
- [ ] Event handlers connect to WASM
- [ ] Signal updates trigger re-renders
- [ ] Component state persists

### Phase 3: Full Stack (Future)
- [ ] Multiple components
- [ ] Component props
- [ ] Conditional rendering
- [ ] List rendering
- [ ] Form handling

## Performance Monitoring

### Signal Operations

Check console for:
```
[Reactive] Signal #0 created with value: 0
[Reactive] Signal #0 tracked by observer #1
[Reactive] Signal #0 changed: 0 -> 1
[Reactive] Running observer #1
```

### WASM Operations

Check console for:
```
WASM created signal: 0 with value: 0
WASM getting signal: 0
WASM setting signal: 0 to: 5
```

## Next Steps

1. ✅ Verify reactive system works in browser
2. Implement proper DOM rendering from VDOM
3. Add string memory allocation
4. Wire up event handlers
5. Build full reactive counter
6. Test complete reactivity loop

## File Structure

```
ravensone/
├── serve.py                    # Development server
├── test-reactive.html          # Comprehensive reactive test
├── test-wasm.html              # Basic WASM loading test
├── test-component.wasm         # Compiled test component
├── test-if.wasm                # Compiled if/else test
├── dist/
│   └── reactive-runtime.js     # Reactive Signal/Effect runtime
└── runtime/
    ├── index.html              # Full app runtime
    └── ravensone.js            # WASM bridge
```

## Success Criteria

**The system is working when:**

1. ✅ `test-reactive.html` shows all green checkmarks
2. ✅ Manual counter increments/decrements smoothly
3. ✅ Console shows reactive updates logging
4. ✅ WASM component can be called
5. ✅ No errors in browser console

---

**Current Status:** Phase 1 complete, moving to Phase 2
**Last Updated:** 2025-10-17
