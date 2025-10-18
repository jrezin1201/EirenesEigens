# RavensOne AI Generator - Live Demo

## 🎬 Example: Generating a Todo App

Let's walk through generating a complete todo application.

### Step 1: Run the Generator

```bash
$ ./generate.sh

🤖 RavensOne AI Project Generator
==================================

Project name: todo-app
Project description: A todo list with add, delete, mark complete, and filtering
Features: local storage persistence, filter by status, clear completed tasks

🤖 Generating project with Claude...
```

### Step 2: Claude Generates the Code

The system sends this prompt to Claude:

```
Create a RavensOne project:
Name: todo-app
Description: A todo list with add, delete, mark complete, and filtering
Features: local storage persistence, filter by status, clear completed tasks

Generate all .raven files needed. Return ONLY the JSON.
```

### Step 3: Files Are Created

```
✅ Created: src/main.raven
✅ Created: src/components/TodoItem.raven
✅ Created: src/components/TodoInput.raven
✅ Created: src/components/Filter.raven
✅ Created: raven.toml
```

### Step 4: Compilation

```
🔨 Compiling project...
   Compiling todo-app v0.1.0
   Finished release [optimized] target(s) in 2.3s
✅ Compilation successful!

🎉 Project generated successfully!
📁 Location: generated/todo-app
```

## 📝 Generated Code

### `src/main.raven`

```raven
use raven::prelude::*;

component App() {
    // State
    let todos = Signal::new(vec![]);
    let filter = Signal::new("all");
    let input_value = Signal::new(String::new());

    // Load from localStorage on mount
    Effect::new(|| {
        if let Some(stored) = localStorage::get("todos") {
            if let Ok(parsed) = serde_json::from_str(&stored) {
                todos.set(parsed);
            }
        }
    });

    // Save to localStorage when todos change
    Effect::new(|| {
        let serialized = serde_json::to_string(&todos.get()).unwrap();
        localStorage::set("todos", &serialized);
    });

    // Filtered todos
    let filtered_todos = Computed::new(|| {
        let all_todos = todos.get();
        match filter.get().as_str() {
            "active" => all_todos.iter()
                .filter(|t| !t.completed)
                .cloned()
                .collect(),
            "completed" => all_todos.iter()
                .filter(|t| t.completed)
                .cloned()
                .collect(),
            _ => all_todos
        }
    });

    // Event handlers
    fn add_todo() {
        let text = input_value.get();
        if !text.is_empty() {
            let mut current = todos.get();
            current.push(Todo {
                id: uuid::v4(),
                text: text.clone(),
                completed: false
            });
            todos.set(current);
            input_value.set(String::new());
        }
    }

    fn toggle_todo(id: Uuid) {
        let mut current = todos.get();
        if let Some(todo) = current.iter_mut().find(|t| t.id == id) {
            todo.completed = !todo.completed;
        }
        todos.set(current);
    }

    fn delete_todo(id: Uuid) {
        let mut current = todos.get();
        current.retain(|t| t.id != id);
        todos.set(current);
    }

    fn clear_completed() {
        let mut current = todos.get();
        current.retain(|t| !t.completed);
        todos.set(current);
    }

    // Template
    <div class="app">
        <header class="header">
            <h1>Todo App</h1>
            <TodoInput
                value={input_value.get()}
                on_change={|v| input_value.set(v)}
                on_submit={add_todo}
            />
        </header>

        <main class="main">
            <Filter
                current={filter.get()}
                on_change={|f| filter.set(f)}
            />

            <ul class="todo-list">
                {filtered_todos.get().iter().map(|todo| {
                    <TodoItem
                        key={todo.id}
                        todo={todo.clone()}
                        on_toggle={toggle_todo}
                        on_delete={delete_todo}
                    />
                }).collect::<Vec<_>>()}
            </ul>

            {if filtered_todos.get().is_empty() {
                <p class="empty-state">No todos yet. Add one above!</p>
            }}
        </main>

        <footer class="footer">
            <span class="todo-count">
                {todos.get().iter().filter(|t| !t.completed).count()} items left
            </span>
            <button
                class="clear-completed"
                onclick={clear_completed}
            >
                Clear completed
            </button>
        </footer>
    </div>
}

#[derive(Clone, Serialize, Deserialize)]
struct Todo {
    id: Uuid,
    text: String,
    completed: bool,
}
```

### `src/components/TodoItem.raven`

```raven
use raven::prelude::*;

#[derive(Props)]
struct TodoItemProps {
    todo: Todo,
    on_toggle: Box<dyn Fn(Uuid)>,
    on_delete: Box<dyn Fn(Uuid)>,
}

component TodoItem(props: TodoItemProps) {
    <li class={format!("todo-item {}", if props.todo.completed { "completed" } else { "" })}>
        <input
            type="checkbox"
            class="toggle"
            checked={props.todo.completed}
            onchange={|| (props.on_toggle)(props.todo.id)}
        />
        <label class="todo-text">{props.todo.text}</label>
        <button
            class="delete"
            onclick={|| (props.on_delete)(props.todo.id)}
        >
            ×
        </button>
    </li>
}
```

### `src/components/TodoInput.raven`

```raven
use raven::prelude::*;

#[derive(Props)]
struct TodoInputProps {
    value: String,
    on_change: Box<dyn Fn(String)>,
    on_submit: Box<dyn Fn()>,
}

component TodoInput(props: TodoInputProps) {
    fn handle_keypress(e: KeyboardEvent) {
        if e.key == "Enter" {
            (props.on_submit)();
        }
    }

    <input
        type="text"
        class="new-todo"
        placeholder="What needs to be done?"
        value={props.value}
        oninput={|e| (props.on_change)(e.target.value)}
        onkeypress={handle_keypress}
    />
}
```

### `src/components/Filter.raven`

```raven
use raven::prelude::*;

#[derive(Props)]
struct FilterProps {
    current: String,
    on_change: Box<dyn Fn(String)>,
}

component Filter(props: FilterProps) {
    <div class="filters">
        <button
            class={if props.current == "all" { "selected" } else { "" }}
            onclick={|| (props.on_change)("all".to_string())}
        >
            All
        </button>
        <button
            class={if props.current == "active" { "selected" } else { "" }}
            onclick={|| (props.on_change)("active".to_string())}
        >
            Active
        </button>
        <button
            class={if props.current == "completed" { "selected" } else { "" }}
            onclick={|| (props.on_change)("completed".to_string())}
        >
            Completed
        </button>
    </div>
}
```

### `raven.toml`

```toml
[package]
name = "todo-app"
version = "0.1.0"
edition = "2024"

[dependencies]
raven-ui = "0.1"
uuid = { version = "1.0", features = ["v4", "serde"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## 🚀 Running the App

```bash
cd generated/todo-app

# Development mode
raven dev
# Opens http://localhost:8000

# Production build
raven build --release
# Creates optimized dist/ folder

# Deploy
vercel dist/
# or
netlify deploy --dir=dist --prod
```

## 🎨 What You Get

The generated app includes:

✅ **Reactive State Management** - Signals for all state
✅ **Local Storage Persistence** - Todos saved automatically
✅ **Filtering** - View all, active, or completed todos
✅ **CRUD Operations** - Add, toggle, delete todos
✅ **Component Architecture** - Clean, reusable components
✅ **Proper Styling** - CSS classes for all elements
✅ **Type Safety** - Full TypeScript-like type checking
✅ **Production Ready** - Compiles to optimized WASM

## 🔧 More Examples

### Weather Dashboard

```bash
Project name: weather-dash
Description: Weather dashboard with current conditions and 5-day forecast
Features: geolocation, unit toggle (C/F), weather icons, hourly forecast
```

Generated structure:
```
weather-dash/
├── src/
│   ├── main.raven
│   └── components/
│       ├── CurrentWeather.raven
│       ├── HourlyForecast.raven
│       ├── DailyForecast.raven
│       └── LocationSearch.raven
```

### Calculator

```bash
Project name: calculator
Description: Scientific calculator with history
Features: basic operations, scientific functions, memory, history log
```

Generated structure:
```
calculator/
├── src/
│   ├── main.raven
│   └── components/
│       ├── Display.raven
│       ├── ButtonPad.raven
│       ├── ScientificPad.raven
│       └── History.raven
```

### Timer

```bash
Project name: pomodoro-timer
Description: Pomodoro timer with customizable work/break intervals
Features: start/pause/reset, notifications, session tracking, statistics
```

## 🎯 Tips for Best Results

### ✅ Good Prompts

```
Description: A habit tracker with daily check-ins
Features: streak tracking, multiple habits, visual calendar, motivational quotes

Description: A markdown editor with live preview
Features: syntax highlighting, export to PDF, auto-save, dark mode

Description: A budget tracker for personal finance
Features: income/expense tracking, categories, charts, monthly summaries
```

### ❌ Avoid

```
Description: Make me a website  // Too vague
Description: An app with AI  // RavensOne is frontend-only
Description: Complex 3D game  // Beyond scope
```

## 📊 Comparison

| Manual Development | AI Generator |
|-------------------|-------------|
| 2-3 hours setup | 2 minutes |
| Learning curve | Instant examples |
| Potential errors | Best practices enforced |
| Blank slate | Working baseline |

## 🔮 Future Enhancements

Coming soon:
- [ ] Component library integration
- [ ] API route generation
- [ ] Test generation
- [ ] Style variants (Material, Tailwind, etc.)
- [ ] Multi-page app generation
- [ ] State management patterns (Redux, MobX)

---

**Try it now:**
```bash
cd examples/ai-generator
./generate.sh
```

Build something amazing! 🚀
