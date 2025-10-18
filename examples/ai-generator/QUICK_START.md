# RavensOne AI Generator - Quick Start

Generate RavensOne apps from descriptions in under 2 minutes!

## ⚡ 60-Second Setup

```bash
# 1. Get your Anthropic API key
# Visit: https://console.anthropic.com/settings/keys

# 2. Set the API key
export ANTHROPIC_API_KEY='sk-ant-your-key-here'

# 3. Navigate to the generator
cd examples/ai-generator

# 4. Run it!
./generate.sh
```

## 🎯 Example Session

```
$ ./generate.sh

🤖 RavensOne AI Project Generator
==================================

Project name: counter-app
Project description: A simple counter with increment and decrement buttons
Features: persist to local storage, keyboard shortcuts, reset button

🤖 Generating project with Claude...

✅ Created: src/main.raven
✅ Created: raven.toml

🔨 Compiling project...
✅ Compilation successful!

🎉 Project generated successfully!
📁 Location: generated/counter-app

Next steps:
  cd generated/counter-app
  raven dev    # Start development server
```

## 📁 What You Get

```
generated/counter-app/
├── src/
│   └── main.raven          # Your complete app!
├── raven.toml              # Project manifest
├── dist/
│   ├── app.wasm            # Compiled WebAssembly
│   └── index.html          # HTML wrapper
└── generated.json          # Raw Claude response (for debugging)
```

## 🚀 Run Your App

```bash
cd generated/counter-app
raven dev
```

Visit `http://localhost:8000` - your app is live!

## 🎨 Sample Apps to Try

### 1. Todo List
```
Name: todo-app
Description: Todo list with filtering
Features: mark complete, delete, filter by status, persist to local storage
```

### 2. Calculator
```
Name: calculator
Description: Basic calculator
Features: add, subtract, multiply, divide, clear, decimal support
```

### 3. Weather App
```
Name: weather
Description: Weather dashboard
Features: current conditions, 5-day forecast, location search, unit toggle
```

### 4. Timer
```
Name: timer
Description: Countdown timer
Features: set minutes/seconds, start/pause/reset, notifications
```

### 5. Notes App
```
Name: notes
Description: Simple note-taking app
Features: create, edit, delete notes, search, categories, markdown support
```

## 🔧 The Workflow

```
User Prompt
    ↓
Claude API (generates .raven files)
    ↓
File System (writes files)
    ↓
RavensOne Compiler (compiles to WASM)
    ↓
Ready to Deploy! 🚀
```

## 💡 Pro Tips

### Get Better Results

✅ **BE SPECIFIC**
```
Bad:  "A website"
Good: "A habit tracker with daily check-ins and streak counting"
```

✅ **LIST FEATURES**
```
Bad:  "Make it look nice"
Good: "dark mode, animations, mobile responsive"
```

✅ **MENTION DATA**
```
Bad:  "Store stuff"
Good: "persist to local storage, export to JSON"
```

### Iterate Quickly

```bash
# Generate v1
./generate.sh
# ... describe basic app

# Test it
cd generated/my-app
raven dev

# Generate v2 with improvements
cd ../..
./generate.sh
# ... describe enhanced version
```

## 🐛 Troubleshooting

### "API key not set"
```bash
export ANTHROPIC_API_KEY='your-key-here'
```

### "Command not found: jq"
```bash
# macOS
brew install jq

# Ubuntu/Debian
sudo apt-get install jq
```

### "Compilation failed"
- Check `generated/[app-name]/generated.json` for errors
- Verify RavensOne compiler is installed: `raven --version`
- Try regenerating with a simpler description

## 📚 Next Steps

1. **Customize** - Edit the generated `.raven` files
2. **Style** - Add CSS in the `<style>` section
3. **Deploy** - See [deployment guide](README.md#deployment)
4. **Learn** - Study the generated code to learn RavensOne patterns

## 🌟 Advanced Usage

### Use from Code

```rust
use ravensone::ai_generator::*;

let generator = AIGenerator::new(api_key);
let request = GenerationRequest {
    project_name: "my-app".into(),
    project_description: "A counter app".into(),
    features: vec!["increment".into(), "decrement".into()],
};

generator.generate_and_compile(&request, Path::new("output"))
    .await
    .unwrap();
```

### Custom Prompts

Edit `generate.sh` and modify the `SYSTEM_PROMPT` variable to customize:
- Component structure
- Styling approach
- State management patterns
- Code organization

## 📊 Stats

What the generator creates:

| Metric | Typical App |
|--------|-------------|
| **Generation Time** | 10-30 seconds |
| **Files Created** | 2-5 files |
| **Lines of Code** | 50-200 lines |
| **Compilation Time** | 1-3 seconds |
| **Total Time** | < 2 minutes |

## 🎓 Learn More

- [Full Documentation](README.md)
- [Live Demo Walkthrough](DEMO.md)
- [RavensOne Language Guide](../../docs/GETTING_STARTED.md)
- [Example Gallery](../README.md)

---

**Ready to build?**

```bash
cd examples/ai-generator
./generate.sh
```

Your idea → Working app in under 2 minutes! ⚡
