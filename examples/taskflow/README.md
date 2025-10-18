# TaskFlow

A full-stack todo application with authentication built with RavensOne.

![TaskFlow](https://img.shields.io/badge/RavensOne-Example-6366f1)
![License](https://img.shields.io/badge/license-MIT-blue)

## 🎯 Overview

TaskFlow demonstrates a complete production-ready application built with RavensOne, showcasing:

- **Authentication** - User registration and login with JWT tokens
- **CRUD Operations** - Create, read, update, and delete todos
- **Real-time State** - Reactive UI with fine-grained updates
- **Form Validation** - Client-side validation with error messages
- **Protected Routes** - Auth-guarded pages
- **Responsive Design** - Works on desktop and mobile
- **Professional UI** - Using raven-ui components

## 🏗️ Architecture

### Frontend (RavensOne)
- **Framework**: RavensOne with fine-grained reactivity
- **UI Library**: raven-ui for components
- **Routing**: raven-router for navigation
- **HTTP Client**: raven-http for API calls
- **Forms**: raven-forms for validation

### Backend (Rust)
- **Framework**: Axum web framework
- **Database**: PostgreSQL with SQLx
- **Auth**: JWT tokens + Argon2 password hashing
- **CORS**: Enabled for local development

## 📦 Features

### Authentication
- User registration with email/password
- Secure password hashing with Argon2
- JWT token-based authentication
- Persistent login with localStorage
- Protected dashboard routes

### Todo Management
- Create todos with title and description
- Mark todos as complete/incomplete
- Delete todos
- Filter by status (all, active, completed)
- Real-time statistics (total, active, completed)

### UI/UX
- Modern dark theme
- Responsive design
- Loading states
- Error handling
- Empty states
- Modal dialogs
- Form validation

## 🚀 Getting Started

### Prerequisites

- Rust 1.70+
- PostgreSQL 14+
- RavensOne CLI

### Backend Setup

1. **Create Database**:
```bash
createdb taskflow
```

2. **Configure Environment**:
```bash
cd backend
cp .env.example .env
# Edit .env with your DATABASE_URL
```

3. **Run Backend**:
```bash
cargo run
```

Backend will start on `http://localhost:3000`

### Frontend Setup

1. **Install Dependencies**:
```bash
cd frontend
raven pkg install
```

2. **Run Development Server**:
```bash
raven dev
```

Frontend will start on `http://localhost:8000`

## 📚 API Endpoints

### Authentication
- `POST /api/auth/register` - Register new user
- `POST /api/auth/login` - Login user

### Todos (Protected)
- `GET /api/todos` - Get all user's todos
- `POST /api/todos` - Create new todo
- `PUT /api/todos/:id` - Update todo
- `DELETE /api/todos/:id` - Delete todo

All protected endpoints require `Authorization: Bearer <token>` header.

## 🎨 Code Examples

### Creating a Todo

```rust
let new_todo = create_todo(
    token,
    "Buy groceries".to_string(),
    Some("Milk, eggs, bread".to_string())
).await?;
```

### Reactive State

```rust
// Todos automatically update UI when changed
todos.update(|list| {
    list.push(new_todo);
});

// Computed values automatically recalculate
let active_count = Computed::new(move || {
    todos.get().iter().filter(|t| !t.completed).count()
});
```

### Protected Routes

```rust
<Route path="/dashboard" component={ProtectedRoute(Dashboard)} />

component ProtectedRoute(component: Component) {
    let auth = auth_state.get();
    {if auth.token.is_some() {
        <component />
    } else {
        <Redirect to="/login" />
    }}
}
```

## 🔒 Security

- Passwords hashed with Argon2id (memory-hard, GPU-resistant)
- JWT tokens with 30-day expiry
- SQL injection protection with SQLx
- CORS configuration
- Input validation on both client and server

## 📊 Database Schema

```sql
users (
    id UUID PRIMARY KEY,
    email VARCHAR UNIQUE,
    password_hash TEXT,
    name VARCHAR,
    created_at TIMESTAMPTZ
)

todos (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    title TEXT,
    description TEXT,
    completed BOOLEAN,
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ
)
```

## 🎓 Learning Resources

This example demonstrates:

1. **State Management** - Global auth state with Signals
2. **Component Composition** - Reusable components (TodoItem, StatCard, etc.)
3. **Async Operations** - HTTP requests with loading/error states
4. **Form Handling** - Validation and submission
5. **Routing** - Multi-page navigation with protection
6. **Side Effects** - localStorage persistence
7. **Computed Values** - Derived state (statistics, filtering)

## 🚢 Deployment

### Backend (Fly.io)

```bash
cd backend
flyctl launch
flyctl postgres create
flyctl postgres attach
flyctl deploy
```

### Frontend (Vercel/Netlify)

```bash
cd frontend
raven build --release
# Deploy dist/ folder
```

## 📝 License

MIT License - see LICENSE file for details

## 🤝 Contributing

This is an example application. For RavensOne framework contributions, see the main repository.

## 🔗 Links

- [RavensOne Documentation](https://ravensone-docs.fly.dev)
- [Package Registry](https://ravensone-registry.fly.dev)
- [GitHub Repository](https://github.com/ravensone/ravensone)

---

Built with ❤️ using [RavensOne](https://ravensone.dev)
