# raven-store

Advanced state management library for RavensOne applications with reactive primitives, middleware support, and persistence.

## Features

- **Reactive Stores**: Centralized state management with automatic reactivity
- **Signals & Computed**: Fine-grained reactive primitives
- **Middleware System**: Extensible middleware for logging, persistence, time-travel
- **Local Storage**: Automatic state persistence to browser storage
- **Devtools Integration**: Time-travel debugging and state inspection
- **Async Actions**: Built-in support for async state updates
- **Selectors**: Memoized derived state with automatic dependency tracking
- **Type Safety**: Full type inference and compile-time checking

## Installation

```bash
raven pkg add raven-store
```

## Quick Start

### Basic Store

```raven
import { createStore } from "raven-store"

let store = createStore({
    count: 0,
    user: None
});

// Subscribe to changes
store.subscribe((state) => {
    console.log("State changed:", state);
});

// Update state
store.update((state) => {
    state.count += 1;
});

// Get current state
let current = store.get();
```

### Actions

```raven
import { createStore } from "raven-store"

struct AppState {
    count: i32,
    loading: bool
}

let store = createStore(AppState {
    count: 0,
    loading: false
});

// Define actions
let increment = (amount: i32) => {
    store.update((state) => {
        state.count += amount;
    });
};

let fetchData = async () => {
    store.update((state) => state.loading = true);

    let data = await fetch("/api/data");

    store.update((state) => {
        state.loading = false;
        state.count = data.count;
    });
};
```

### Computed Values

```raven
import { createStore, computed } from "raven-store"

let store = createStore({
    firstName: "John",
    lastName: "Doe"
});

let fullName = computed(() => {
    let state = store.get();
    return state.firstName + " " + state.lastName;
});

// fullName automatically updates when firstName or lastName changes
console.log(fullName.get()); // "John Doe"
```

### Middleware

```raven
import { createStore, logger, persist } from "raven-store"

let store = createStore(
    { count: 0 },
    [
        logger(), // Log all state changes
        persist("my-app-state") // Persist to localStorage
    ]
);
```

### Selectors

```raven
import { createStore, createSelector } from "raven-store"

struct Todo {
    id: i32,
    text: String,
    completed: bool
}

let store = createStore({
    todos: Vec<Todo>::new()
});

// Memoized selector
let completedTodos = createSelector(
    store,
    (state) => state.todos.filter((todo) => todo.completed)
);

// Only recomputes when todos change
let completed = completedTodos.get();
```

## Advanced Usage

### Custom Middleware

```raven
import { Middleware } from "raven-store"

fn createAnalytics() -> Middleware {
    Middleware::new(|action, state, next| {
        // Track action
        analytics.track(action.type, action.payload);

        // Continue to next middleware
        next(action, state);
    })
}

let store = createStore(
    initialState,
    [createAnalytics()]
);
```

### Time-Travel Debugging

```raven
import { createStore, devtools } from "raven-store"

let store = createStore(
    initialState,
    [devtools({ maxHistory: 50 })]
);

// Go back in time
store.undo();

// Go forward
store.redo();

// Jump to specific state
store.jumpTo(10);
```

### Derived Stores

```raven
import { createStore, derived } from "raven-store"

let count = createStore(0);
let doubled = derived(count, (n) => n * 2);
let quadrupled = derived(doubled, (n) => n * 2);

count.set(5);
console.log(quadrupled.get()); // 20
```

## API Reference

### `createStore(initialState, middleware?)`

Create a new reactive store.

**Parameters**:
- `initialState`: Initial state object
- `middleware`: Optional array of middleware functions

**Returns**: Store instance

### `Store.get()`

Get the current state.

### `Store.set(newState)`

Replace the entire state.

### `Store.update(updater)`

Update state with a function.

**Parameters**:
- `updater`: Function that receives current state and modifies it

### `Store.subscribe(listener)`

Subscribe to state changes.

**Parameters**:
- `listener`: Function called when state changes

**Returns**: Unsubscribe function

### `computed(getter)`

Create a computed value that automatically updates.

**Parameters**:
- `getter`: Function that computes the value

**Returns**: Computed instance with `.get()` method

### `createSelector(store, selector)`

Create a memoized selector.

**Parameters**:
- `store`: Store to select from
- `selector`: Function that derives value from state

**Returns**: Selector instance with `.get()` method

## Middleware

### `logger(options?)`

Logs all state changes to console.

### `persist(key, options?)`

Persists state to localStorage.

**Parameters**:
- `key`: localStorage key
- `options`: Optional configuration

### `devtools(options?)`

Enables time-travel debugging.

**Parameters**:
- `options.maxHistory`: Maximum number of history entries (default: 50)

## Best Practices

1. **Keep stores focused**: Create separate stores for different domains
2. **Use selectors**: Avoid computing derived state in components
3. **Async actions**: Handle side effects in actions, not reducers
4. **Middleware order**: Place devtools last in middleware chain
5. **Type your state**: Always define state types for type safety

## Examples

See the `/examples` directory for complete examples:
- `todo-app.raven` - Todo list with persistence
- `async-data.raven` - Async data fetching
- `time-travel.raven` - Time-travel debugging
- `multi-store.raven` - Multiple coordinated stores

## Performance

raven-store uses fine-grained reactivity for optimal performance:
- Only re-renders components that use changed state
- Memoized selectors prevent unnecessary computations
- Batched updates reduce render cycles

## License

MIT License - See LICENSE file for details
