# How to Run the Analytics Dashboard

## Quick Start (Browser Demo)

The fastest way to see the analytics dashboard in action:

```bash
# From the ravensone directory
open examples/run_dashboard.html
```

This will open a **client-side simulation** of the RavensOne application that demonstrates:
- ✅ Reactive state management (Signal, ReactiveVec)
- ✅ Computed values (Total Revenue calculation)
- ✅ Effects (auto-rendering on state changes)
- ✅ Component architecture
- ✅ Interactive date range selection
- ✅ Real-time chart rendering with Canvas API
- ✅ Loading states
- ✅ Simulated server functions

## What You'll See

1. **Metrics Dashboard** with 4 key metrics:
   - Total Revenue
   - Active Users
   - Conversion Rate
   - Avg Session Duration

2. **Interactive Features**:
   - Change date range to see metrics update
   - Watch loading states
   - See trend indicators (↑/↓)
   - View historical charts for each metric

3. **Reactive Updates**:
   - Total revenue recalculates automatically
   - Charts redraw when data changes
   - Smooth loading transitions

## Production Build (When RavensOne Compiler is Complete)

Once the full compiler is implemented, you would run:

### Step 1: Compile to WebAssembly

```bash
# Compile for client (browser)
raven compile examples/analytics_dashboard.raven --target=client -o dist/dashboard.wasm

# Compile for server (SSR)
raven compile examples/analytics_dashboard.raven --target=server -o dist/dashboard_ssr.wasm
```

### Step 2: Start Development Server

```bash
# Start dev server with SSR and hot reload
raven dev examples/analytics_dashboard.raven
```

This will:
- Server-render the initial HTML for fast load times
- Inject hydration script for client interactivity
- Enable hot module replacement
- Watch for file changes

### Step 3: Build for Production

```bash
# Optimized production build
raven build examples/analytics_dashboard.raven --optimize
```

This creates:
- Minified WASM bundle (~50KB)
- SSR-generated HTML
- Optimized CSS
- Hydration runtime

### Step 4: Deploy

```bash
# Deploy to static hosting (Vercel, Netlify, etc.)
raven deploy examples/analytics_dashboard.raven --platform=vercel

# Or deploy to Node.js server
raven deploy examples/analytics_dashboard.raven --platform=node
```

## Architecture

The production version would work like this:

```
Client Request
    ↓
Server (Node.js/Deno)
    ↓
Run dashboard_ssr.wasm
    ↓
Generate HTML with initial data
    ↓
Send HTML + dashboard.wasm to browser
    ↓
Browser loads HTML (instant visual)
    ↓
Load WASM bundle
    ↓
Hydrate: Attach event listeners
    ↓
Reactive system takes over
    ↓
User interactions → State updates → UI updates
```

## Performance Metrics (Estimated)

**Demo Mode (Current)**:
- First Paint: ~100ms
- Interactive: ~150ms
- Bundle Size: 0KB (inline JS)

**Production Mode (With Compiler)**:
- First Paint: ~50ms (SSR)
- Time to Interactive: ~200ms
- Bundle Size: ~50KB (WASM + hydration)
- Memory Usage: ~5MB
- 60 FPS smooth interactions

## Customization

### Change Mock Data

Edit `run_dashboard.html`, modify the `mockMetrics` array:

```javascript
const mockMetrics = [
    {
        id: 'm1',
        name: 'Your Metric',
        value: 12345,
        change_percent: 10.5
    },
    // Add more metrics...
];
```

### Adjust Styling

Edit `analytics_dashboard.css`:

```css
:root {
    --primary-color: #3b82f6;  /* Change primary color */
    --success-color: #10b981;   /* Change positive trend color */
}
```

### Add More Charts

In `run_dashboard.html`, add to the charts section:

```javascript
function renderCustomChart(data) {
    return `
        <div class="chart-widget">
            <h4>Custom Chart</h4>
            <!-- Your chart HTML -->
        </div>
    `;
}
```

## Troubleshooting

### Browser Not Opening?

```bash
# macOS
open examples/run_dashboard.html

# Linux
xdg-open examples/run_dashboard.html

# Windows
start examples/run_dashboard.html

# Or just navigate to the file in your browser
```

### Charts Not Rendering?

- Ensure JavaScript is enabled
- Check browser console for errors (F12)
- Try refreshing the page

### Metrics Not Updating?

- The demo simulates a 500ms network delay
- Wait for the loading spinner to disappear
- Check that the date range dropdown is working

## Extending the Demo

### Add Real API Integration

Replace the `fetchMetrics` function with a real API call:

```javascript
async function fetchMetrics(dateRange) {
    loading.set(true);

    try {
        const response = await fetch('/api/metrics', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ dateRange })
        });

        const data = await response.json();

        metrics.clear();
        data.forEach(m => metrics.push(m));
    } catch (error) {
        console.error('Failed to fetch metrics:', error);
    } finally {
        loading.set(false);
        render();
    }
}
```

### Add WebSocket for Real-Time Updates

```javascript
const ws = new WebSocket('ws://localhost:8080/metrics');

ws.onmessage = (event) => {
    const newMetric = JSON.parse(event.data);

    // Find and update metric
    const currentMetrics = metrics.get();
    const index = currentMetrics.findIndex(m => m.id === newMetric.id);

    if (index >= 0) {
        currentMetrics[index] = newMetric;
        metrics.signal.set([...currentMetrics]);
    }
};
```

### Add User Authentication

```javascript
let currentUser = new Signal(null);

async function login(email, password) {
    const response = await fetch('/api/auth/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email, password })
    });

    const user = await response.json();
    currentUser.set(user);
}
```

## Next Steps

1. **Explore the Code**: Open `analytics_dashboard.raven` to see the RavensOne source
2. **Read the Architecture**: Check `ANALYTICS_README.md` for design details
3. **Customize**: Modify colors, metrics, charts to fit your needs
4. **Integrate**: Add real backend APIs when ready

## Support

For issues or questions:
- Check the main README: `../README.md`
- View implementation details: `../IMPLEMENTATION_SUMMARY.md`
- Report bugs: https://github.com/ravensone/ravensone/issues

---

**Enjoy exploring RavensOne's capabilities!** 🚀
