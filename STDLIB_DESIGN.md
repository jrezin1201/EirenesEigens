# RavensOne Standard Library Architecture

## Philosophy

The standard library should be:
1. **Compiler-integrated** - No external dependencies
2. **Zero-config** - Works out of the box
3. **Type-safe** - Full compiler checking
4. **AI-friendly** - Consistent, predictable APIs

---

## Module Structure

```
std::
├── collections/     # Data structures
│   ├── Array<T>
│   ├── Map<K, V>
│   ├── Set<T>
│   └── Queue<T>
│
├── http/           # HTTP client/server
│   ├── Request
│   ├── Response
│   ├── route()
│   └── fetch()
│
├── db/             # Database ORM
│   ├── schema
│   ├── query
│   ├── migrate
│   └── transaction
│
├── auth/           # Authentication
│   ├── User
│   ├── Session
│   ├── jwt
│   └── oauth
│
├── reactive/       # Reactivity primitives
│   ├── Signal<T>
│   ├── Computed<T>
│   ├── Effect
│   └── Memo<T>
│
├── dom/            # DOM manipulation (client-only)
│   ├── Element
│   ├── Event
│   ├── query()
│   └── mount()
│
├── io/             # Input/Output
│   ├── File
│   ├── read()
│   ├── write()
│   └── Stream
│
├── time/           # Date/Time
│   ├── DateTime
│   ├── Duration
│   ├── now()
│   └── sleep()
│
├── crypto/         # Cryptography
│   ├── hash()
│   ├── encrypt()
│   ├── decrypt()
│   └── random()
│
├── json/           # JSON serialization
│   ├── parse()
│   ├── stringify()
│   ├── validate()
│   └── schema
│
└── test/           # Testing framework
    ├── assert
    ├── expect
    ├── describe
    └── it
```

---

## Core Modules (Priority Order)

### 1. **std::reactive** - CRITICAL FOR MVP
The foundation of RavensOne's reactivity.

```raven
use std::reactive::{Signal, Computed, Effect};

// Auto-wrapped by compiler
let count = Signal::new(0);

// Computed values
let doubled = Computed::new(() => count.get() * 2);

// Side effects
Effect::new(() => {
    console.log("Count changed:", count.get());
});

// In components, just use directly:
let count = 0;  // Compiler wraps in Signal automatically
count++;        // Triggers reactivity
```

**Implementation:**
- `Signal<T>` - Reactive state container
- `Computed<T>` - Derived reactive values
- `Effect` - Side effects that run when dependencies change
- `Memo<T>` - Cached computed values

**Compiler integration:**
- Semantic analyzer detects reactive variables
- Auto-wraps in Signal<T>
- Tracks dependencies in JSX
- Generates re-render calls

---

### 2. **std::http** - Essential for APIs

```raven
use std::http::{route, Request, Response};

// Define routes
route("/api/users", handleUsers);
route("/api/users/:id", handleUser);

server fn handleUsers(req: Request) -> Response {
    match req.method {
        "GET" => Response.json(db.users.findAll()),
        "POST" => {
            let user = req.json::<User>();
            Response.json(db.users.create(user))
        }
        _ => Response.error(405, "Method not allowed")
    }
}

// Client-side fetch
let users = fetch("/api/users").json::<Array<User>>();
```

**Key Types:**
```raven
struct Request {
    method: string,
    path: string,
    headers: Map<string, string>,
    body: string,
    params: Map<string, string>,  // Route params
    query: Map<string, string>,   // Query params
}

struct Response {
    status: i32,
    headers: Map<string, string>,
    body: string,
}
```

---

### 3. **std::db** - THE GAME CHANGER

```raven
use std::db::{schema, query};

// Schema definition (parsed by compiler)
schema User {
    id: i32 @primary @auto,
    email: string @unique @index,
    name: string,
    created_at: DateTime @default(now),
    updated_at: DateTime @auto_update,
}

schema Post {
    id: i32 @primary @auto,
    user_id: i32 @foreign(User),
    title: string,
    content: string,
    published: bool @default(false),
}

// Type-safe queries (compiler-checked!)
server fn getUser(id: i32) -> User? {
    return db.users.findById(id);
}

server fn createUser(email: string, name: string) -> User {
    return db.users.create({
        email: email,
        name: name,
    });
}

server fn getUserPosts(userId: i32) -> Array<Post> {
    return db.posts
        .where(user_id == userId)
        .where(published == true)
        .orderBy(created_at, "desc")
        .limit(10);
}

// Relationships
server fn getUserWithPosts(id: i32) -> User {
    return db.users
        .findById(id)
        .include(posts);  // Automatic join!
}
```

**Compiler Integration:**
- Parse `schema` blocks in parser
- Generate database adapter code
- Type-check all queries at compile time
- Auto-generate migrations

**Database Adapters:**
- PostgreSQL (via pg_wire)
- SQLite (via rusqlite)
- MySQL (via mysql_async)
- Cloudflare D1 (via Workers KV)

---

### 4. **std::auth** - Production-ready auth

```raven
use std::auth::{User, Session, jwt};

// Built-in User model
schema User {
    id: i32 @primary @auto,
    email: string @unique,
    password_hash: string @private,  // Never sent to client
    role: string @default("user"),
}

server fn signup(email: string, password: string) -> AuthToken {
    let user = auth.createUser(email, password);  // Auto-hashes password
    return auth.createToken(user);
}

server fn login(email: string, password: string) -> AuthToken {
    return auth.login(email, password);
}

// Protected routes
@auth
server fn getProfile() -> User {
    return auth.currentUser();  // Compiler provides context
}

@auth(role: "admin")
server fn deleteUser(id: i32) -> bool {
    return db.users.delete(id);
}
```

**Key Functions:**
```raven
auth.createUser(email, password) -> User
auth.login(email, password) -> AuthToken
auth.logout() -> void
auth.verifyToken(token) -> User?
auth.currentUser() -> User
auth.hashPassword(password) -> string
auth.verifyPassword(password, hash) -> bool
```

---

### 5. **std::collections** - Data structures

```raven
use std::collections::{Array, Map, Set};

// Arrays (like Rust Vec)
let nums = Array::<i32>::new();
nums.push(1);
nums.push(2);
nums.push(3);

let doubled = nums.map(|x| x * 2);
let sum = nums.reduce(|acc, x| acc + x, 0);

// Maps (like Rust HashMap)
let scores = Map::<string, i32>::new();
scores.set("Alice", 100);
scores.set("Bob", 95);

let alice_score = scores.get("Alice")?;

// Sets (unique values)
let tags = Set::<string>::new();
tags.add("rust");
tags.add("wasm");
tags.has("rust");  // true
```

---

### 6. **std::json** - JSON handling

```raven
use std::json;

// Serialize
let user = User { name: "Alice", age: 30 };
let json_str = json.stringify(user);

// Deserialize (type-safe!)
let user = json.parse::<User>(json_str)?;

// Validate against schema
schema UserInput {
    name: string @min(3) @max(50),
    email: string @email,
    age: i32 @min(0) @max(120),
}

let validated = json.validate::<UserInput>(input)?;
```

---

### 7. **std::time** - Date/Time operations

```raven
use std::time::{DateTime, Duration};

let now = DateTime.now();
let tomorrow = now + Duration.days(1);

let formatted = now.format("YYYY-MM-DD HH:mm:ss");
let unix = now.timestamp();

// Sleep/delay
server fn delayed() -> string {
    time.sleep(Duration.seconds(5));
    return "Done!";
}
```

---

### 8. **std::crypto** - Cryptography

```raven
use std::crypto;

// Hashing
let hash = crypto.sha256("hello");
let bcrypt_hash = crypto.bcrypt("password");

// Random
let uuid = crypto.uuid();
let random_int = crypto.randomInt(1, 100);

// Encryption
let encrypted = crypto.encrypt("secret", key);
let decrypted = crypto.decrypt(encrypted, key);
```

---

### 9. **std::test** - Testing framework

```raven
use std::test::{describe, it, expect};

describe("User model", () => {
    it("should create user", () => {
        let user = User { name: "Alice", age: 30 };
        expect(user.name).toBe("Alice");
    });

    it("should validate email", () => {
        let result = validateEmail("invalid");
        expect(result).toBe(false);
    });
});

// Run tests:
$ raven test
```

---

## Implementation Strategy

### Phase 1: Core Primitives (Week 1-2)
1. `std::reactive` - Signal, Computed, Effect
2. Basic `std::collections` - Array, Map
3. `std::json` - parse, stringify

### Phase 2: HTTP & Auth (Week 3-4)
4. `std::http` - Request, Response, route()
5. `std::auth` - User, Session, JWT

### Phase 3: Database (Week 5-6) ⭐ **KILLER FEATURE**
6. `std::db` - schema, queries, migrations

### Phase 4: Utilities (Week 7-8)
7. `std::time` - DateTime, Duration
8. `std::crypto` - hashing, encryption
9. `std::test` - testing framework

---

## Compiler Integration Points

### 1. **Auto-imports**
Common modules imported automatically:
```raven
// No need to import these:
Signal, Computed, Effect  // std::reactive
Array, Map, Set           // std::collections
Request, Response         // std::http
```

### 2. **Special syntax**
```raven
schema User { }      // Parsed as special syntax
route("/path", fn)   // Registered at compile time
@auth                // Decorator syntax
server fn            // Splits code for client/server
```

### 3. **Type checking**
```raven
db.users.findById(id)  // Compiler knows User schema
                       // Returns User? not any
```

### 4. **Code generation**
```raven
schema User { }
// Compiler generates:
// - Database migration
// - Type definitions
// - Query builders
// - Serialization code
```

---

## File Organization

```
src/
├── stdlib/
│   ├── mod.rs           # Module exports
│   ├── reactive.rs      # Signal, Computed, Effect
│   ├── http.rs          # HTTP types and functions
│   ├── db/
│   │   ├── mod.rs
│   │   ├── schema.rs    # Schema parsing
│   │   ├── query.rs     # Query builder
│   │   └── adapters/
│   │       ├── postgres.rs
│   │       ├── sqlite.rs
│   │       └── d1.rs    # Cloudflare D1
│   ├── auth.rs          # Authentication
│   ├── collections.rs   # Data structures
│   ├── json.rs          # JSON handling
│   ├── time.rs          # Date/Time
│   ├── crypto.rs        # Cryptography
│   └── test.rs          # Testing framework
│
└── codegen.rs           # Imports stdlib modules
```

---

## Example: Full-Stack App with Stdlib

```raven
// schemas.raven - THE ENTIRE APP IN ONE FILE!

use std::{http, db, auth, reactive};

// Database schema
schema User {
    id: i32 @primary @auto,
    email: string @unique,
    name: string,
}

schema Todo {
    id: i32 @primary @auto,
    user_id: i32 @foreign(User),
    title: string,
    completed: bool @default(false),
}

// API Routes
@auth
server fn getTodos() -> Array<Todo> {
    let user = auth.currentUser();
    return db.todos.where(user_id == user.id);
}

@auth
server fn createTodo(title: string) -> Todo {
    let user = auth.currentUser();
    return db.todos.create({
        user_id: user.id,
        title: title,
    });
}

// UI Component
component TodoApp() {
    let todos = getTodos();
    let newTodo = "";

    return <div>
        <h1>"My Todos"</h1>
        <input value={newTodo} onchange={(e) => newTodo = e.value} />
        <button onclick={() => {
            createTodo(newTodo);
            todos = getTodos();  // Refresh
            newTodo = "";
        }}>"Add"</button>

        <ul>
            {todos.map(todo => <li>{todo.title}</li>)}
        </ul>
    </div>;
}
```

Deploy:
```bash
$ raven deploy
🎉 https://my-todos.ravens.one
```

**One file. Full stack. Production ready.**

---

## Next Steps

1. ✅ Design standard library structure (this document)
2. Implement `std::reactive` (Signal, Computed, Effect)
3. Implement `std::collections` (Array, Map)
4. Implement `std::http` (Request, Response, route)
5. Implement `std::db` (schema, queries) ⭐ GAME CHANGER
6. Implement `std::auth` (User, Session, JWT)

---

**"Batteries included. Zero config. Maximum velocity."**
