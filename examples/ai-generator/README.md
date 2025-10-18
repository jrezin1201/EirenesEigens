# RavensOne AI Generator

Generate complete RavensOne applications from natural language descriptions using Claude AI.

## 🎯 Overview

The RavensOne AI Generator allows you to:
1. Describe your project in plain English
2. Let Claude generate all `.raven` files
3. Automatically compile to WebAssembly
4. Deploy instantly

## 🚀 Quick Start

### Prerequisites

- RavensOne compiler installed
- Anthropic API key ([Get one here](https://console.anthropic.com/))

### Setup

```bash
# Set your API key
export ANTHROPIC_API_KEY='your-api-key-here'

# Make the script executable
chmod +x generate.sh
```

### Generate a Project

```bash
./generate.sh
```

You'll be prompted for:
- **Project name**: e.g., "todo-app"
- **Description**: e.g., "A simple todo list with add, delete, and mark complete"
- **Features**: e.g., "local storage persistence, filtering, dark mode"

The generator will:
1. ✅ Call Claude API with optimized prompt
2. ✅ Generate all necessary `.raven` files
3. ✅ Create `raven.toml` manifest
4. ✅ Compile to WebAssembly
5. ✅ Ready to run!

## 📚 Examples

### Example 1: Todo App

**Input:**
```
Project name: todo-app
Description: A todo list application with mark complete functionality
Features: local storage, filtering by status, clear completed
```

**Generated Files:**
```
todo-app/
├── src/
│   ├── main.raven           # Main App component
│   └── components/
│       ├── TodoItem.raven   # Individual todo component
│       └── Filter.raven     # Filter component
├── raven.toml               # Project manifest
└── dist/
    └── app.wasm             # Compiled output
```

### Example 2: Weather Dashboard

**Input:**
```
Project name: weather-dash
Description: Weather dashboard showing current conditions and forecast
Features: geolocation, 5-day forecast, temperature units toggle, weather icons
```

**Generated Files:**
```
weather-dash/
├── src/
│   ├── main.raven
│   └── components/
│       ├── CurrentWeather.raven
│       ├── Forecast.raven
│       └── LocationSearch.raven
└── raven.toml
```

### Example 3: Calculator

**Input:**
```
Project name: calculator
Description: Scientific calculator with history
Features: basic operations, scientific functions, calculation history, keyboard support
```

## 🎨 What Gets Generated

The AI generator creates production-ready code with:

### 1. Main Component (`src/main.raven`)
```raven
component App() {
    let todos = Signal::new(vec![]);
    let filter = Signal::new("all");

    fn add_todo(text: String) {
        let mut current = todos.get();
        current.push(Todo {
            id: uuid::v4(),
            text,
            completed: false
        });
        todos.set(current);
    }

    <div class="app">
        <h1>Todo App</h1>
        <TodoInput on_add={add_todo} />
        <TodoList todos={filtered_todos.get()} />
    </div>
}
```

### 2. Sub-Components
```raven
component TodoItem(props: TodoItemProps) {
    <div class="todo-item">
        <input
            type="checkbox"
            checked={props.todo.completed}
            onchange={|| props.on_toggle(props.todo.id)}
        />
        <span class={if props.todo.completed { "completed" } else { "" }}>
            {props.todo.text}
        </span>
    </div>
}
```

### 3. Project Manifest (`raven.toml`)
```toml
[package]
name = "todo-app"
version = "0.1.0"
edition = "2024"

[dependencies]
raven-ui = "0.1"
```

### 4. Styling
The generator includes proper CSS classes and responsive design.

## 🔧 Advanced Usage

### Custom System Prompt

You can customize the system prompt to generate specific patterns:

```bash
# Edit generate.sh and modify the SYSTEM_PROMPT variable
SYSTEM_PROMPT='Your custom instructions here...'
```

### API from Code

Use the Rust API directly:

```rust
use ravensone::ai_generator::{AIGenerator, GenerationRequest};

#[tokio::main]
async fn main() {
    let generator = AIGenerator::new(
        std::env::var("ANTHROPIC_API_KEY").unwrap()
    );

    let request = GenerationRequest {
        project_name: "my-app".to_string(),
        project_description: "A counter app".to_string(),
        features: vec!["increment".to_string(), "decrement".to_string()],
    };

    generator.generate_and_compile(
        &request,
        Path::new("output/my-app")
    ).await.unwrap();
}
```

## 🎯 Best Practices

### Good Prompts

✅ **DO:**
- Be specific about functionality
- Mention state management needs
- Specify UI components
- Include user interactions

```
Description: A kanban board with drag-and-drop
Features: multiple columns, card creation, drag to reorder, persist to local storage
```

❌ **DON'T:**
- Be too vague
- Request backend APIs (RavensOne is frontend-only)
- Expect complex animations

### Iterating on Generated Code

1. Generate initial version
2. Review the code
3. Manually refine as needed
4. Re-generate with improved prompt

## 📊 What's Generated

| Component | Description |
|-----------|-------------|
| **Main App** | Root component with overall state |
| **Sub-components** | Reusable UI pieces |
| **State Management** | Signals, Computed values |
| **Event Handlers** | onClick, onChange, etc. |
| **Styling** | CSS classes and layout |
| **Manifest** | raven.toml with dependencies |

## 🚀 Deployment

Once generated and compiled:

```bash
cd generated/my-app

# Development server
raven dev

# Build for production
raven build --release

# Deploy to Vercel
vercel dist/

# Deploy to Netlify
netlify deploy --dir=dist --prod
```

## 🔍 Troubleshooting

### "API key not set"
```bash
export ANTHROPIC_API_KEY='sk-ant-...'
```

### "Compilation failed"
Check the generated `src/main.raven` for syntax errors. You may need to:
- Fix imports
- Adjust component structure
- Verify Signal usage

### "Invalid JSON response"
Claude occasionally returns markdown. The script attempts to handle this, but you can:
1. Check `generated.json` for the raw response
2. Manually extract the JSON
3. Retry with a clearer prompt

## 📝 Examples Library

See `examples/generated/` for sample projects:
- `todo-app/` - Classic todo list
- `calculator/` - Scientific calculator
- `weather-dash/` - Weather dashboard
- `timer/` - Pomodoro timer
- `notes/` - Note-taking app

## 🤝 Contributing

Want to improve the generator?

1. Enhance the system prompt in `ai_generator.rs`
2. Add validation for generated code
3. Create templates for common patterns
4. Submit examples to the library

## 📄 License

MIT License - see [LICENSE](../../LICENSE)

---

## 🎓 How It Works

```
┌─────────────────────┐
│  User Description   │
│  "A todo app with..." │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  System Prompt      │
│  (RavensOne syntax, │
│   patterns, rules)  │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│   Claude API        │
│   (Code generation) │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Generated Files    │
│  *.raven, *.toml    │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  RavensOne Compiler │
│  (WASM output)      │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Deployable App     │
│  Ready to ship! 🚀  │
└─────────────────────┘
```

## 🌟 Why This is Powerful

1. **Rapid Prototyping**: Go from idea to working app in minutes
2. **Learn by Example**: See how experienced developers structure RavensOne apps
3. **Consistency**: Generated code follows best practices
4. **Customizable**: Tweak the output to match your needs
5. **Repeatable**: Generate similar apps with variations

---

Built with ❤️ using [RavensOne](https://github.com/ravensone/ravensone) and [Claude](https://anthropic.com)
