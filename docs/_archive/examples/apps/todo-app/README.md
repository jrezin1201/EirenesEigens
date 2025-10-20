# RavensOne Todo App with Authentication

A complete full-stack todo application demonstrating the RavensOne compiler bridge with `@server` and `@client` annotations.

## 🎯 What This Demonstrates

This example showcases the core features of RavensOne's compiler bridge:

### ✅ @server/@client Separation
- **Server functions** (`@server`) - Run only on Node.js
- **Client functions** (`@client`) - Run only in the browser
- **Shared functions** (no annotation) - Run on both sides

### ✅ Automatic RPC Generation
Client calls to server functions become network requests automatically:

```raven
@client
async fn handle_login() {
    // This looks like a local function call...
    let result = login(email, password);
    // ...but it's actually an RPC call to the server!
}
```

### ✅ Shared Validation Logic
Write validation once, use everywhere:

```raven
// This function runs on BOTH server and client
fn validate_email(email: String) -> bool {
    return email.contains("@") && email.contains(".");
}
```

### ✅ Type-Safe Communication
Types are preserved across the client-server boundary:

```raven
@server
fn get_user_todos(user_id: i32) -> Vec<Todo> {
    // Returns Vec<Todo>
}

@client
async fn show_todos() {
    let todos: Vec<Todo> = get_user_todos(1);  // Type-safe!
}
```

---

## 📁 Project Structure

```
todo-app/
├── app.raven       # Single file with ALL code (server + client + shared)
├── styles.css      # Styling (separate for clarity)
└── README.md       # This file
```

**Just one `.raven` file!** That's the power of RavensOne.

---

## 🚀 How to Run

### 1. Compile the App

```bash
cd examples/apps/todo-app
raven compile app.raven --minify
```

**Output:**
```
dist/
├── server.js      # Server bundle with RPC handlers
├── client.js      # Client bundle with RPC stubs
├── app.wasm       # WebAssembly module
└── index.html     # Entry point
```

### 2. Run the Server

```bash
cd dist
node server.js
```

### 3. Open in Browser

```bash
open http://localhost:3000
```

---

## 🎨 Features

### Authentication
- ✅ User registration
- ✅ Login with validation
- ✅ JWT token storage
- ✅ Persistent sessions

### Todo Management
- ✅ Create new todos
- ✅ Mark as complete
- ✅ Delete todos
- ✅ Real-time stats

### User Experience
- ✅ Beautiful gradient UI
- ✅ Smooth animations
- ✅ Mobile responsive
- ✅ Form validation
- ✅ Error handling

---

## 📖 Code Walkthrough

### Shared Validation (Runs Everywhere)

```raven
fn validate_todo_title(title: String) -> bool {
    return title.length() > 0 && title.length() < 100;
}

fn validate_email(email: String) -> bool {
    return email.contains("@") && email.contains(".");
}
```

These functions are included in **both** server and client bundles.

### Server-Side (Node.js Only)

```raven
@server
fn create_todo(user_id: i32, title: String) -> Result<Todo, String> {
    // Validate on server (even though client validates too!)
    if !validate_todo_title(title) {
        return Err("Invalid todo title");
    }

    // Database operations (not accessible from client)
    let todo = db.todos.create({ user_id, title, completed: false });

    return Ok(todo);
}

@server
fn login(email: String, password: String) -> Result<AuthToken, String> {
    // Find user, verify password, generate JWT
    // This code NEVER reaches the client bundle
}
```

### Client-Side (Browser Only)

```raven
@client
async fn handle_add_todo() {
    let title = document.getElementById("new-todo-input").value;
    let user_id = get_current_user_id();

    // Use shared validation!
    if !validate_todo_title(title) {
        alert("Invalid title");
        return;
    }

    // Call server function (automatic RPC!)
    let result = create_todo(user_id, title);

    match result {
        Ok(todo) => refresh_todos(),
        Err(msg) => alert(msg)
    }
}

@client
fn show_login_form() {
    // DOM manipulation only happens in browser
    document.getElementById("app").innerHTML = "<form>...</form>";
}
```

---

## 🔍 How the Compiler Bridge Works

### 1. Code Splitting

The compiler reads your `.raven` file and separates code into buckets:

- **Server bucket**: All `@server` functions + shared functions
- **Client bucket**: All `@client` functions + shared functions
- **Shared bucket**: Functions with no annotation

### 2. RPC Generation

For each `@server` function, the compiler generates:

**Client-side stub (in client.js):**
```javascript
export async function create_todo(user_id, title) {
    return await client.call('create_todo', [user_id, title]);
}
```

**Server-side handler (in server.js):**
```javascript
server.rpc('create_todo', async (params) => {
    const [user_id, title] = params;
    return await create_todo(user_id, title);
});
```

### 3. Type Preservation

RavensOne types map to JavaScript/TypeScript:

| RavensOne | JavaScript |
|-----------|------------|
| `i32` | `number` |
| `String` | `string` |
| `bool` | `boolean` |
| `Vec<T>` | `Array<T>` |
| `Result<T, E>` | `{ok: T} \| {err: E}` |

---

## 🎯 Best Practices Demonstrated

### 1. Validate on Both Sides

```raven
// Shared validation
fn validate_email(email: String) -> bool {
    return email.contains("@");
}

@client
fn handle_login() {
    if !validate_email(email) {  // Client-side check
        show_error("Invalid email");
        return;
    }
    login(email, password);  // Server validates too!
}

@server
fn login(email: String, password: String) -> Result<AuthToken, String> {
    if !validate_email(email) {  // Server-side check
        return Err("Invalid email");
    }
    // Continue...
}
```

**Why?** Client-side validation provides instant feedback. Server-side validation ensures security.

### 2. Keep Shared Functions Pure

```raven
// ✅ Good: Pure function, works on both sides
fn format_date(timestamp: i64) -> String {
    return "2025-10-19";
}

// ❌ Bad: Side effects, use @client or @server instead
fn log_error(msg: String) {
    console.log(msg);  // console works differently on server/client!
}
```

### 3. Use Result Types for Errors

```raven
@server
fn create_todo(title: String) -> Result<Todo, String> {
    if title.length() == 0 {
        return Err("Title required");
    }
    return Ok(/* ... */);
}

@client
async fn handle_add() {
    match create_todo(title) {
        Ok(todo) => show_success(),
        Err(msg) => show_error(msg)
    }
}
```

### 4. Minimize RPC Calls

```raven
// ✅ Good: One RPC call
@server
fn get_dashboard_data() -> DashboardData {
    return DashboardData {
        todos: get_todos(),
        stats: get_stats(),
        user: get_user()
    };
}

// ❌ Bad: Three separate RPC calls
@client
fn load_dashboard() {
    let todos = get_todos();   // RPC 1
    let stats = get_stats();   // RPC 2
    let user = get_user();     // RPC 3
}
```

---

## 🚀 Production Deployment

For production, compile with minification:

```bash
raven compile app.raven --minify --output build/
```

This:
- ✅ Removes comments and whitespace (30-50% smaller)
- ✅ Optimizes bundle sizes
- ✅ Generates production-ready output

Then deploy `build/` to:
- **Server**: Heroku, Fly.io, AWS, DigitalOcean
- **Static files**: Vercel, Netlify, Cloudflare Pages

---

## 📊 Code Statistics

- **Total Lines**: ~550 lines (in one file!)
- **Server Functions**: 9
- **Client Functions**: 12
- **Shared Functions**: 4
- **Data Models**: 3 structs

**Compare to traditional stack:**
- Next.js: ~15 files, 1,200+ lines
- Express + React: ~20 files, 1,500+ lines
- **RavensOne: 1 file, 550 lines** 🎉

---

## 🎓 Learning Outcomes

After studying this example, you'll understand:

1. ✅ How to structure a full-stack app in one file
2. ✅ When to use `@server` vs `@client` vs shared
3. ✅ How automatic RPC generation works
4. ✅ Best practices for validation and error handling
5. ✅ How to minimize network requests
6. ✅ Type-safe communication across the stack

---

## 🔗 Related Documentation

- [FULLSTACK_GUIDE.md](../../../FULLSTACK_GUIDE.md) - Complete compiler bridge guide
- [README.md](../../../README.md) - Main RavensOne documentation
- [STATUS.md](../../../STATUS.md) - Project roadmap and progress

---

## 🤝 Contributing

Found a bug or want to improve this example? PRs welcome!

---

**Built with RavensOne** - _One language. One file. Full stack._
