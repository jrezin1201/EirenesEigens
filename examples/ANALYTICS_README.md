# Analytics Dashboard - RavensOne Production Application

A comprehensive multi-tenant analytics dashboard showcasing **all** of RavensOne's features in a real-world production scenario.

## Overview

This application demonstrates a complete full-stack analytics platform with:
- **Real-time reactive updates** as data changes
- **Server-side rendering** for fast initial page loads and SEO
- **Progressive hydration** for optimal performance
- **Type-safe** client-server communication
- **Async resource management** for data fetching
- **Reactive computations** for derived metrics

## Features Demonstrated

### ✅ Type System & Type Inference
- Strongly typed data structures (`User`, `Metric`, `ChartData`)
- Type-safe function parameters and return values
- Compile-time type checking prevents runtime errors
- Generic types with collections (`Vec<Metric>`, `HashMap`)

### ✅ Reactive State Management
- **Signals** for mutable reactive state (`date_range`, `loading`)
- **Computed** values for derived data (`total_revenue`)
- **Effects** for side effects (data fetching, chart rendering)
- **Resources** for async data loading with loading/error states
- **ReactiveVec** for reactive collections of metrics

### ✅ Server-Side Rendering (SSR)
- Full HTML generation on the server for initial page load
- Hydration markers for client takeover
- SEO-friendly with proper meta tags
- Fast Time to First Byte (TTFB)

### ✅ Hydration System
- Immediate hydration strategy for interactive dashboard
- Priority-based hydration for critical components
- Seamless transition from static to interactive

### ✅ Component Architecture
- **Composable components**: `MetricCard`, `ChartWidget`, `LineChart`, `DateRangePicker`
- **Props-based communication** with type safety
- **Nested component hierarchies** with proper data flow
- **Conditional rendering** based on state

### ✅ Full-Stack Type Safety
- Server functions with `extern server` modifier
- Type-checked RPC between client and server
- Shared type definitions across boundaries

### ✅ JSX with Bare Text Support
- Natural JSX syntax: `<h1>Analytics Dashboard</h1>`
- No need for `{text}` wrapping
- Expression interpolation with `{}`
- Nested elements and components

## Application Architecture

```
┌──────────────────────────────────────┐
│         Client (Browser)             │
│  ┌────────────────────────────────┐  │
│  │  Reactive State Layer          │  │
│  │  - Signals, Computed, Effects  │  │
│  └────────────────────────────────┘  │
│  ┌────────────────────────────────┐  │
│  │  Component Tree                │  │
│  │  - Dashboard                   │  │
│  │    ├─ MetricCard (×4)          │  │
│  │    └─ ChartWidget (×4)         │  │
│  │       └─ LineChart             │  │
│  └────────────────────────────────┘  │
└──────────────────────────────────────┘
              ↕ Type-Safe RPC
┌──────────────────────────────────────┐
│         Server (Node.js/Deno)        │
│  ┌────────────────────────────────┐  │
│  │  Server Functions              │  │
│  │  - fetch_metrics()             │  │
│  │  - fetch_chart_data()          │  │
│  └────────────────────────────────┘  │
│  ┌────────────────────────────────┐  │
│  │  SSR Engine                    │  │
│  │  - Render to HTML              │  │
│  │  - Inject hydration data       │  │
│  └────────────────────────────────┘  │
└──────────────────────────────────────┘
```

## Component Breakdown

### 1. **Dashboard** (Main Container)
- Manages application state
- Handles date range selection
- Fetches and displays metrics
- Shows loading states

**Reactive Features**:
```raven
let date_range = Signal::new("Last 30 Days");
let metrics = ReactiveVec::new();
let total_revenue = Computed::new(|| { /* derives from metrics */ });
```

### 2. **MetricCard** (Presentational)
- Displays individual metrics
- Shows trend indicators (↑/↓)
- Color-coded changes (green/red)

**Props**: Single `Metric` object

### 3. **ChartWidget** (Smart Component)
- Fetches time-series data for a metric
- Manages loading/error states with `Resource`
- Renders line chart when data is ready

**Async Pattern**:
```raven
let chart_data = Resource::new();
create_effect(|| {
    chart_data.set_loading();
    let data = fetch_chart_data(metric.id, 30);
    chart_data.set_ready(data);
});
```

### 4. **LineChart** (Canvas Renderer)
- Renders chart using Canvas API
- Auto-updates when data changes
- Uses effects for imperative rendering

### 5. **DateRangePicker** (Form Control)
- Controlled component with Signal
- Triggers metric refresh on change

## Data Flow

1. **Initial Render (Server)**:
   - Server executes `App` component
   - Calls `fetch_metrics()` server function
   - Generates HTML with initial data
   - Injects hydration script

2. **Client Hydration**:
   - Browser loads HTML + WASM bundle
   - Hydration attaches event listeners
   - Reactive system initializes

3. **User Interaction**:
   - User changes date range
   - Signal updates trigger effect
   - Effect calls server function
   - Server returns new data
   - Reactive system updates UI

4. **Computed Values**:
   - `total_revenue` automatically recalculates
   - UI updates without manual intervention

## Performance Optimizations

1. **Server-Side Rendering**
   - Instant visual feedback on page load
   - Search engine indexable content

2. **Progressive Hydration**
   - Critical components hydrate immediately
   - Charts hydrate when visible (viewport)

3. **Reactive Granularity**
   - Only components using changed signals re-render
   - No virtual DOM diffing overhead

4. **WebAssembly Execution**
   - Near-native performance for computations
   - Efficient memory management

## Building & Running

```bash
# Compile the application
raven compile examples/analytics_dashboard.raven --target=client -o dist/dashboard.wasm
raven compile examples/analytics_dashboard.raven --target=server -o dist/dashboard_ssr.wasm

# Start development server with SSR
raven dev examples/analytics_dashboard.raven

# Build for production
raven build examples/analytics_dashboard.raven --optimize

# Run tests
raven test examples/analytics_dashboard.raven
```

## Testing Features

The application includes comprehensive testing for:
- Component rendering
- Reactive state updates
- Server function calls
- Type safety validation
- SSR output correctness
- Hydration integrity

## Real-World Enhancements

To make this production-ready, you would add:

1. **Authentication & Authorization**
   - User login/logout
   - Role-based access control
   - Tenant isolation

2. **Database Integration**
   - PostgreSQL/MongoDB for metrics storage
   - Time-series database for analytics
   - Caching layer (Redis)

3. **Real-time Updates**
   - WebSocket connection for live data
   - Reactive subscription to metric changes

4. **Advanced Features**
   - Custom date range picker with calendar
   - Export to PDF/Excel
   - Customizable dashboard layouts
   - Alert thresholds and notifications

5. **Monitoring & Observability**
   - Error tracking (Sentry)
   - Performance monitoring
   - User analytics

## Why This App Showcases RavensOne's Power

1. **Type Safety Everywhere**: From database queries to UI rendering, types prevent bugs
2. **Reactive by Default**: No manual DOM manipulation or state management boilerplate
3. **SSR + SPA**: Best of both worlds - fast initial load + smooth interactions
4. **Full-Stack in One Language**: No context switching between TypeScript/Python/etc.
5. **WebAssembly Performance**: Complex calculations run at near-native speed
6. **Developer Experience**: Clean syntax, great error messages, fast compilation

## Comparison with Other Frameworks

| Feature | RavensOne | React + Next.js | Svelte + SvelteKit |
|---------|-----------|-----------------|-------------------|
| Type System | Built-in, Inferred | TypeScript (separate) | TypeScript (separate) |
| Reactivity | Fine-grained signals | Virtual DOM | Compiled reactive |
| SSR | Built-in | Next.js framework | SvelteKit framework |
| Language | Rust-inspired | JavaScript/TS | JavaScript/TS |
| Bundle Size | ~50KB (WASM) | ~200KB+ (JS) | ~30KB (compiled JS) |
| Runtime Overhead | Minimal | React runtime | Minimal |
| Server Functions | Type-safe, built-in | tRPC/API routes | +server.ts files |

---

**Built with ❤️ using RavensOne**

For more information: https://github.com/ravensone/ravensone
