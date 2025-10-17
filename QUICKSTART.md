# 🚀 RavensOne Quick Start Guide

**Get up and running in 5 minutes!**

---

## Prerequisites

- **Rust** (for future compiler work)
- **Node.js** (for runtime)
- **Python 3** (for dev server)

---

## Installation

```bash
# Clone the repository
git clone <repo-url>
cd ravensone

# Install Node dependencies
npm install

# Start dev server
python3 serve.py
```

Server starts at `http://localhost:8000`

---

## Test All Demos

### 1. HTTP Client Demo
```bash
open http://localhost:8000/demo-http.html
```

**What you'll see:**
- Fetch from GitHub API
- Fetch from JSONPlaceholder
- Fetch from Random User API
- Beautiful cards displaying data

### 2. Database ORM Demo
```bash
open http://localhost:8000/demo-orm.html
```

**What you'll see:**
- Create/read/delete blog posts
- Auto-increment IDs
- Timestamps
- SQL query log
- Stats dashboard

### 3. Authentication Demo
```bash
open http://localhost:8000/demo-auth.html
```

**What you'll see:**
- Login/signup forms
- Protected content
- Role-based access (Admin vs User)
- Session persistence

**Demo accounts:**
- Admin: `admin@ravens.one` / `admin123`
- User: `user@ravens.one` / `user123`

### 4. Component System Demo
```bash
open http://localhost:8000/demo-components.html
```

**What you'll see:**
- Buttons (variants & sizes)
- Cards
- Modals
- Inputs
- Badges
- Lists
- Interactive counter with state

### 5. Full-Stack Todo App
```bash
# Terminal 1: Start server
node demo-fullstack-server.js

# Terminal 2: Open browser
open http://localhost:3000/
```

**What you'll see:**
- Add/delete todos
- Toggle completion
- Real-time stats
- RPC communication demo

### 6. Real-Time Chat App
```bash
# Terminal 1: Start server
node demo-chat-server.js

# Terminal 2: Open browser
open http://localhost:3001/

# Terminal 3: Open another browser window (test multi-user)
open http://localhost:3001/
```

**What you'll see:**
- Real-time messaging
- Multiple rooms (General, Random, Tech)
- Typing indicators
- User presence
- Join/leave notifications

---

## Project Structure (Quick Reference)

```
ravensone/
├── src/stdlib/          # Rust standard library
│   ├── http.rs         # HTTP client
│   ├── db.rs           # Database ORM
│   └── auth.rs         # Authentication
│
├── dist/                # JavaScript runtimes
│   ├── http-runtime.js
│   ├── db-runtime.js
│   ├── auth-runtime.js
│   ├── server-runtime.js
│   ├── client-runtime.js
│   └── component-runtime.js
│
└── demo-*.html          # Working demos
```

---

## What We've Built

✅ **Phase 1:** HTTP Client
✅ **Phase 2:** Database ORM
✅ **Phase 3:** Authentication
✅ **Phase 4:** Server/Client Splitting
✅ **Phase 5.1:** Real-Time Chat
✅ **Phase 5.2:** Component System

---

## What's Next

**Task 3:** WebAssembly Compiler Bridge
**Task 4:** Hot Module Reloading
**Task 5:** Package Manager

---

## Need Help?

Read the full **README.md** for detailed documentation!

---

**Let's build the future! 🚀**
