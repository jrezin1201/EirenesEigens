# RavensOne Full-Stack Development Guide

## Overview

RavensOne's compiler bridge enables you to write full-stack applications in a single `.raven` file. Using `@server` and `@client` annotations, you can mark where code runs, and the compiler automatically:

- Splits code into server and client bundles
- Generates type-safe RPC stubs for client-server communication
- Outputs working `server.js`, `client.js`, and `app.wasm` files
- Handles shared utility functions on both sides

## Quick Start

### 1. Create a New File

Create `my-app.raven`:

```raven
// Server-side functions (run on Node.js)
@server
fn get_todos() -> Vec<String> {
    return vec!["Buy milk", "Walk dog", "Write code"];
}

@server
fn add_todo(title: String) -> bool {
    // Database logic here
    return true;
}

// Client-side functions (run in browser)
@client
fn render_todo_list(todos: Vec<String>) -> String {
    return "<ul><li>" + todos.join("</li><li>") + "</li></ul>";
}

@client
fn handle_add_click() {
    let title = prompt("Enter todo:");
    add_todo(title);  // Automatically becomes an RPC call!
}

// Shared functions (available on both sides)
fn format_date(timestamp: i32) -> String {
    return "2025-01-01";
}

fn validate_input(text: String) -> bool {
    return text.length() > 0;
}
```

### 2. Compile

```bash
raven compile my-app.raven

# With minification for production
raven compile my-app.raven --minify

# Custom output directory
raven compile my-app.raven --output build/
```

Output:
```
🔥 Compiling full-stack application: my-app.raven
   📦 Output: server.js + client.js + app.wasm

   Parsing...
   ✓ Parsed 6 statements
   Generating JavaScript bundles...
   ✓ Split: 2 server, 2 client, 2 shared functions
   Compiling to WebAssembly...
   ✓ Generated WASM module (1024 bytes)

   Writing output files...
   ✓ dist/server.js
   ✓ dist/client.js
   ✓ dist/app.wasm
   ✓ dist/index.html

✨ Compilation complete!
   Run: cd dist && node server.js
```

### 3. Run

```bash
cd dist
node server.js
```

Your full-stack app is now running! Open `http://localhost:3000` in your browser.

## Annotations

### `@server`

Marks functions that run only on the server (Node.js).

**Use cases:**
- Database queries
- File system access
- API calls to external services
- Authentication/authorization
- Business logic that shouldn't be exposed to clients

**Example:**

```raven
@server
fn get_user_from_db(id: i32) -> User {
    let db = Database::connect("postgres://...");
    return db.query("SELECT * FROM users WHERE id = ?", id);
}

@server
fn send_email(to: String, subject: String, body: String) -> bool {
    // Email sending logic
    return true;
}
```

### `@client`

Marks functions that run only in the browser.

**Use cases:**
- DOM manipulation
- Event handlers
- UI rendering
- Browser-specific APIs (localStorage, etc.)
- Client-side validation

**Example:**

```raven
@client
fn render_user_card(user: User) -> String {
    return `
        <div class="user-card">
            <h2>${user.name}</h2>
            <p>${user.email}</p>
        </div>
    `;
}

@client
fn handle_login_click() {
    let username = document.getElementById("username").value;
    let password = document.getElementById("password").value;

    // This automatically becomes an RPC call
    let result = login(username, password);
    if (result.success) {
        window.location = "/dashboard";
    }
}
```

### No Annotation (Shared)

Functions without annotations are available on **both** server and client.

**Use cases:**
- Utility functions
- Data validation
- Formatting functions
- Pure business logic with no side effects

**Example:**

```raven
fn format_currency(amount: f64) -> String {
    return "$" + amount.to_string();
}

fn validate_email(email: String) -> bool {
    return email.contains("@") && email.contains(".");
}

fn calculate_tax(price: f64, rate: f64) -> f64 {
    return price * rate;
}
```

## Automatic RPC Generation

When you call a `@server` function from `@client` code, RavensOne automatically:

1. Generates an async client stub
2. Serializes arguments to JSON
3. Makes an HTTP POST request to `/_rpc/function_name`
4. Deserializes the response
5. Returns the result

### Example

**Source code:**

```raven
@server
fn get_user(id: i32) -> User {
    // Server logic
}

@client
fn show_profile(user_id: i32) {
    let user = get_user(user_id);  // Looks like a normal call!
    display_user(user);
}
```

**Generated client.js:**

```javascript
// Auto-generated RPC client stub
export async function get_user(id) {
    return await client.call('get_user', [id]);
}

// Your client code (automatically awaits the RPC)
export function show_profile(user_id) {
    let user = await get_user(user_id);
    display_user(user);
}
```

**Generated server.js:**

```javascript
// Auto-generated RPC handler
server.rpc('get_user', async (params) => {
    const [id] = params;
    return await get_user(id);
});
```

## Type Safety

RavensOne preserves type information through the compilation process:

| RavensOne Type | TypeScript/JS Type |
|----------------|-------------------|
| `i32`, `i64`, `u32`, `u64`, `f32`, `f64` | `number` |
| `String` | `string` |
| `bool` | `boolean` |
| `Vec<T>` | `Array<T>` |
| `Option<T>` | `T \| null` |
| `(A, B, C)` | `[A, B, C]` |

**Example:**

```raven
@server
fn get_products(category: String, limit: i32) -> Vec<Product> {
    // Returns array of products
}
```

**Generated TypeScript types:**

```typescript
export function get_products(category: string, limit: number): Promise<Array<Product>>;
```

## Best Practices

### 1. Keep Shared Functions Pure

Shared functions should be pure (no side effects) since they run on both server and client:

```raven
// ✅ Good: Pure function
fn calculate_total(items: Vec<Item>) -> f64 {
    return items.map(|item| item.price).sum();
}

// ❌ Bad: Side effects (use @server or @client instead)
fn log_error(msg: String) {
    console.log(msg);  // Won't work the same on both sides!
}
```

### 2. Minimize RPC Calls

Each `@server` function call from client code is a network request. Batch operations when possible:

```raven
// ✅ Good: Single RPC call
@server
fn get_dashboard_data() -> DashboardData {
    return DashboardData {
        users: get_users(),
        posts: get_posts(),
        stats: get_stats(),
    };
}

// ❌ Bad: Three separate RPC calls
@client
fn load_dashboard() {
    let users = get_users();   // RPC call 1
    let posts = get_posts();   // RPC call 2
    let stats = get_stats();   // RPC call 3
}
```

### 3. Validate on Both Sides

Use shared validation functions, but always validate on the server too:

```raven
// Shared validation
fn validate_email(email: String) -> bool {
    return email.contains("@");
}

@client
fn handle_signup() {
    let email = get_email_input();
    if (!validate_email(email)) {
        show_error("Invalid email");
        return;
    }
    register_user(email);
}

@server
fn register_user(email: String) -> Result<User, String> {
    // ALWAYS validate on server too!
    if (!validate_email(email)) {
        return Err("Invalid email");
    }
    // Create user...
}
```

### 4. Handle Errors Gracefully

RPC calls can fail due to network issues. Always handle errors:

```raven
@client
async fn load_user_profile(id: i32) {
    try {
        let user = await get_user(id);
        display_user(user);
    } catch (error) {
        show_error("Failed to load user. Please try again.");
        console.error(error);
    }
}
```

### 5. Use Minification for Production

Always compile with `--minify` for production deployments:

```bash
raven compile app.raven --minify --output dist/
```

This reduces JavaScript bundle sizes by 30-50% on average.

## Project Structure

### Recommended Structure

```
my-project/
├── src/
│   ├── main.raven          # Main application file
│   ├── models/
│   │   ├── user.raven      # Data models
│   │   └── post.raven
│   ├── server/
│   │   ├── database.raven  # Server-only code
│   │   └── auth.raven
│   └── client/
│       ├── components.raven # Client-only code
│       └── ui.raven
├── dist/                    # Compiled output
│   ├── server.js
│   ├── client.js
│   ├── app.wasm
│   └── index.html
├── raven.toml              # Package manifest
└── README.md
```

### Compiling Multiple Files

Compile each file separately or combine them:

```bash
# Compile all files
for file in src/**/*.raven; do
    raven compile $file --output dist/
done

# Or use the build command
raven build --release
```

## CLI Commands

### `raven compile <file>`

Compiles a `.raven` file to full-stack JavaScript.

**Options:**
- `-o, --output <dir>` - Output directory (default: `dist/`)
- `-m, --minify` - Enable JavaScript minification

**Examples:**

```bash
# Basic compilation
raven compile app.raven

# With minification
raven compile app.raven --minify

# Custom output
raven compile app.raven --output build/
```

### `raven dev`

Start development server with hot module replacement:

```bash
raven dev --port 3000
```

### `raven build`

Build project for production:

```bash
raven build --release
```

### `raven serve`

Serve compiled application:

```bash
raven serve --port 8000 --open
```

## Troubleshooting

### "Compilation failed: Parse error"

Check your syntax. Common issues:
- Missing semicolons
- Unmatched braces
- Invalid annotation placement

### "RPC call failed: Network error"

Ensure your server is running:
```bash
cd dist && node server.js
```

### "Function not found"

Make sure `@server` functions are compiled and the server is restarted after changes.

### "Type mismatch"

Check that client and server agree on types. Recompile after changing function signatures.

## Next Steps

- Explore the [examples/](/examples) directory for sample applications
- Read the [API Documentation](/docs/api.md)
- Join the [RavensOne community](https://github.com/ravensone)
- Report issues on [GitHub](https://github.com/ravensone/ravensone/issues)

## Summary

RavensOne's full-stack compilation makes it easy to build modern web applications:

✅ Write server and client code in one file
✅ Automatic RPC generation
✅ Type-safe communication
✅ Code splitting and optimization
✅ Production-ready builds

Happy coding! 🚀
