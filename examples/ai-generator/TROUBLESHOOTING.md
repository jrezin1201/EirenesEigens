# AI Generator Troubleshooting Guide

## Error: "Cannot iterate over null (null)"

**Cause**: The Claude API returned null instead of generated code.

**Solutions**:

### 1. Check API Key is Set
```bash
echo $ANTHROPIC_API_KEY
```

If blank, set it:
```bash
export ANTHROPIC_API_KEY='sk-ant-your-actual-key-here'
```

### 2. Get API Key
Visit: https://console.anthropic.com/settings/keys

### 3. Verify Key is Valid
```bash
# Should print your key (starting with sk-ant-)
echo $ANTHROPIC_API_KEY
```

### 4. Try Again with Simple Project
```bash
cd examples/ai-generator
./generate.sh

# Use simple inputs:
Project name: test-counter
Description: A simple counter
Features: increment, decrement, reset
```

---

## Error: "unrecognized subcommand 'dev'"

**Cause**: The `raven dev` command doesn't exist yet.

**Solution**: Use manual compilation workflow:

```bash
cd generated/your-project

# 1. Create dist directory
mkdir -p dist

# 2. Compile to WASM
raven compile src/main.raven --output dist/app.wasm

# 3. Create index.html (see example below)

# 4. Serve locally
python3 -m http.server 8000 --directory dist
# or
npx serve dist
```

---

## Issue: Too Many Features

**Problem**: Requesting 30+ features overwhelms the generator.

**Solution**: Keep it simple!

❌ **Too Complex**:
```
Features: product grid, filtering, cart, wishlist, reviews, 
         checkout, payments, shipping, tracking, accounts, etc.
```

✅ **Just Right**:
```
Features: product grid with images, size filter, add to cart, price display
```

**Pro Tip**: Generate a simple version first, then iterate!

---

## Recommended First Projects

### 1. Counter (Simplest)
```
Name: counter
Description: A counter with buttons
Features: increment, decrement, reset, keyboard shortcuts
```

### 2. Todo List (Intermediate)
```
Name: todos
Description: Todo list with filtering
Features: add todo, mark complete, delete, filter by status
```

### 3. Product Grid (Your Goal)
```
Name: shirts
Description: Hawaiian shirt product showcase
Features: product cards, images, price, size selector, add to cart
```

---

## Complete Working Example

### See the Counter Demo
```bash
cd examples/ai-generator/generated/counter-demo
cat src/main.raven
```

This shows exactly what a successful generation produces!

---

## Quick Fixes Checklist

- [ ] API key is set: `echo $ANTHROPIC_API_KEY`
- [ ] Key starts with `sk-ant-`
- [ ] Using 5-10 features (not 30+)
- [ ] Project name is simple (no spaces)
- [ ] Using `raven compile` (not `raven dev`)
- [ ] Created `dist` directory before compiling

---

## Still Having Issues?

1. Check `generated/your-project/generated.json` for API errors
2. Try the exact example from QUICK_START.md
3. Start with counter-demo to verify setup works
4. Gradually add complexity

---

## Example HTML Loader

Save as `dist/index.html`:

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>RavensOne App</title>
</head>
<body>
    <div id="app"></div>
    <script type="module">
        import init from './app.wasm';
        init();
    </script>
</body>
</html>
```
