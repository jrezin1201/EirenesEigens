# Getting Started with RavensOne

Welcome to RavensOne! This guide will help you build your first RavensOne application in under 10 minutes.

---

## What is RavensOne?

RavensOne is a modern full-stack web framework that compiles to WebAssembly. It's designed for AI-assisted development and offers:

- ✅ **Type-safe** with automatic type inference
- ⚡ **Blazing fast** (65,000+ compilations/second)
- 🎨 **Reactive** state management with Signals
- 🚀 **Server-side rendering** (SSR) out of the box
- 📦 **Single-file components** with JSX-like syntax
- 🔒 **Memory safe** with borrow checking

---

## Installation

### Prerequisites

- macOS, Linux, or Windows
- Terminal/Command Prompt access

### Install RavensOne

**macOS/Linux:**
```bash
curl -sSf https://install.ravensone.dev | sh
```

**Windows:**
```powershell
iwr https://install.ravensone.dev/windows | iex
```

**From Source:**
```bash
git clone https://github.com/jrezin1201/RavensOne
cd RavensOne
cargo build --release
cargo install --path .
```

### Verify Installation

```bash
raven --version
# Should output: raven 0.1.0
```

---

## Your First Application

### 1. Create a New Project

```bash
raven new my-first-app
cd my-first-app
```

This creates a project with the following structure:

```
my-first-app/
├── raven.toml          # Package manifest
├── src/
│   └── main.raven      # Entry point
└── README.md
```

### 2. Edit `src/main.raven`

Open `src/main.raven` and replace its contents with:

```raven
component App() {
    let count = Signal::new(0);

    let increment = || {
        count.set(count.get() + 1);
    };

    <div>
        <h1>Hello, RavensOne!</h1>
        <p>Count: {count.get()}</p>
        <button onclick={increment}>Click me!</button>
    </div>
}
```

### 3. Run the Development Server

```bash
raven dev
```

Open your browser to `http://localhost:3000` and you'll see your app running!

Try clicking the button - the count updates instantly thanks to reactive state!

---

## Understanding the Code

Let's break down what's happening:

### Component Declaration

```raven
component App() {
    // Component body
}
```

Components are the building blocks of RavensOne applications. They're declared with the `component` keyword.

### Reactive State with Signals

```raven
let count = Signal::new(0);
```

`Signal` creates a reactive value. When it changes, any component using it automatically re-renders.

### Reading and Writing Signals

```raven
count.get()      // Read the current value
count.set(5)     // Set a new value
count.update(|v| v + 1)  // Update based on current value
```

### Event Handlers

```raven
let increment = || {
    count.set(count.get() + 1);
};

<button onclick={increment}>Click me!</button>
```

Event handlers are just functions! The `||` syntax creates a lambda (anonymous function).

### JSX-like Syntax

```raven
<div>
    <h1>Hello!</h1>
    <p>{count.get()}</p>
</div>
```

Curly braces `{}` embed RavensOne expressions in JSX.

---

## Adding More Features

### Computed Values

Derived values that automatically update:

```raven
let count = Signal::new(0);
let doubled = Computed::new(|| count.get() * 2);

<p>Count: {count.get()}</p>
<p>Doubled: {doubled.get()}</p>
```

### Effects (Side Effects)

Run code when dependencies change:

```raven
Effect::new(|| {
    console.log("Count changed:", count.get());
});
```

### Server Functions

Call server-side code from the client:

```raven
server fn get_user_data(user_id: Int) -> Result<User, Error> {
    // This runs on the server only
    let user = db::fetch_user(user_id).await?;
    Ok(user)
}

// Call from client component
let load_user = async || {
    let user = get_user_data(123).await?;
    user_signal.set(user);
};
```

---

## Building for Production

### Compile Your App

```bash
raven build --release
```

This creates optimized WASM bundles in the `dist/` directory:

```
dist/
├── index.html
├── app.wasm        # Your compiled code
├── app.js          # Runtime wrapper
└── styles.css
```

### Deploy to Vercel

```bash
cd dist
vercel --prod
```

Your app is now live! 🎉

---

## Next Steps

### Learn Core Concepts

- [Components](./concepts/components.md) - Building blocks of your app
- [Reactive State](./concepts/reactivity.md) - Signals, Computed, Effects
- [Routing](./concepts/routing.md) - Client-side navigation
- [Server Functions](./concepts/server-functions.md) - Full-stack RPC
- [Forms](./concepts/forms.md) - Form handling and validation

### Build Real Apps

- [Todo App Tutorial](./tutorials/todo-app.md) - Full CRUD application
- [Blog with SSR](./tutorials/blog.md) - Server-side rendering
- [E-commerce Store](./tutorials/ecommerce.md) - Complex state management
- [Real-time Chat](./tutorials/chat.md) - WebSockets and live updates

### API Reference

- [Standard Library](./api/stdlib.md) - Built-in functions and types
- [Reactive Primitives](./api/reactive.md) - Signal, Computed, Effect, Resource
- [DOM APIs](./api/dom.md) - Working with the DOM
- [HTTP Client](./api/http.md) - Making requests

---

## Common Patterns

### Conditional Rendering

```raven
let show_message = Signal::new(true);

<div>
    {if show_message.get() {
        <p>Hello!</p>
    }}
</div>
```

### List Rendering

```raven
let items = Signal::new([1, 2, 3, 4, 5]);

<ul>
    {items.get().map(|item| {
        <li>Item {item}</li>
    })}
</ul>
```

### Form Handling

```raven
let name = Signal::new("");

let handle_submit = |e| {
    e.preventDefault();
    console.log("Submitted:", name.get());
};

<form onsubmit={handle_submit}>
    <input
        type="text"
        value={name.get()}
        oninput={|e| name.set(e.target.value)}
    />
    <button type="submit">Submit</button>
</form>
```

---

## Troubleshooting

### Common Issues

**Issue**: `raven: command not found`
- **Solution**: Make sure RavensOne is in your PATH. Try `source ~/.bashrc` or restart your terminal.

**Issue**: Type errors during compilation
- **Solution**: Check the error message for line numbers. RavensOne's type inference is smart but may need hints for complex types.

**Issue**: Hot reload not working
- **Solution**: Ensure you're running `raven dev` and your browser has WebSocket support.

### Getting Help

- **GitHub Issues**: [github.com/jrezin1201/RavensOne/issues](https://github.com/jrezin1201/RavensOne/issues)
- **Discussions**: [github.com/jrezin1201/RavensOne/discussions](https://github.com/jrezin1201/RavensOne/discussions)
- **Discord**: Coming soon!

---

## Examples

Check out these complete example applications:

- [Counter App](../examples/counter.raven) - Simple reactive counter
- [Todo List](../examples/todo_app.raven) - Full CRUD with server functions
- [Analytics Dashboard](../examples/analytics_dashboard.raven) - Complex state, charts, responsive design

---

## Cheat Sheet

### Basic Syntax

```raven
// Variables
let x = 10;
let name = "Alice";

// Functions
fn add(a: Int, b: Int) -> Int {
    return a + b;
}

// Components
component Button(props: { text: String }) {
    <button>{props.text}</button>
}

// Signals
let count = Signal::new(0);
count.get()        // Read
count.set(5)       // Write
count.update(|v| v + 1)  // Update

// Computed
let doubled = Computed::new(|| count.get() * 2);

// Effects
Effect::new(|| {
    console.log(count.get());
});

// Server Functions
server fn get_data() -> Result<Data, Error> {
    // Server-only code
}
```

---

## What Makes RavensOne Special?

### 1. AI-First Development

RavensOne is designed to work seamlessly with AI assistants. The syntax is clear, concise, and easy for LLMs to generate.

### 2. No Virtual DOM

Fine-grained reactivity means updates are surgical - only the exact DOM nodes that need to change are updated.

### 3. Type Safety Without Annotations

Hindley-Milner type inference means you get type safety without writing type annotations everywhere.

### 4. True Full-Stack

Server functions let you call server-side code as if it were a local function. No REST APIs, no GraphQL schemas - just functions.

### 5. WebAssembly Native

Compiles to WASM for near-native performance. Your app runs at 60 FPS even with complex UIs.

---

## Ready to Build?

You now know the basics! Here are some next steps:

1. **Build the Todo app** - [Tutorial](./tutorials/todo-app.md)
2. **Explore the examples** - [Examples folder](../examples/)
3. **Read the API docs** - [API Reference](./api/)
4. **Join the community** - [GitHub Discussions](https://github.com/jrezin1201/RavensOne/discussions)

**Happy coding with RavensOne!** 🚀

---

*Last Updated: October 17, 2025*
*RavensOne Version: 0.1.0*
