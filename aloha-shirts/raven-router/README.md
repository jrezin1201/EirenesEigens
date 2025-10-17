# raven-router

**Client-Side Routing Library for RavensOne**

A powerful, flexible routing solution for building single-page applications with RavensOne.

## 🚀 Quick Start

### Installation

```bash
raven pkg add raven-router
```

### Basic Usage

```raven
import { Router, Link } from "raven-router"

component App() {
    let routes = [
        {
            path: "/",
            component: HomePage,
            name: "home"
        },
        {
            path: "/about",
            component: AboutPage,
            name: "about"
        },
        {
            path: "/user/:id",
            component: UserPage,
            name: "user"
        }
    ];

    <div>
        <nav>
            <Link to="/">Home</Link>
            <Link to="/about">About</Link>
        </nav>

        <Router
            routes={routes}
            mode="history"
            fallback={NotFoundPage}
        />
    </div>
}
```

## 📦 Features

- ✅ **Multiple Modes** - History API, Hash, or Memory-based routing
- ✅ **Dynamic Routes** - Path parameters like `/user/:id`
- ✅ **Navigation Guards** - Protect routes with authentication checks
- ✅ **Nested Routes** - Build complex route hierarchies
- ✅ **Programmatic Navigation** - Navigate via hooks or router instance
- ✅ **Active Link Detection** - Automatic active class on current route
- ✅ **Query Parameters** - Full query string parsing and access
- ✅ **Lazy Loading** - Code-split routes for better performance
- ✅ **TypeScript-like Types** - Full type safety with RavensOne's type system

## 🎯 Core Concepts

### Router Component

The main router that manages navigation and renders matched routes.

```raven
<Router
    routes={routes}
    mode="history"      // "history" | "hash" | "memory"
    base="/app"         // Base path for all routes
    fallback={NotFound} // 404 component
/>
```

### Route Configuration

```raven
let routes = [
    {
        path: "/",
        component: HomePage,
        name: "home",
        meta: { requiresAuth: false }
    },
    {
        path: "/dashboard",
        component: DashboardPage,
        name: "dashboard",
        beforeEnter: requireAuth, // Route-level guard
        meta: { requiresAuth: true }
    },
    {
        path: "/user/:id",
        component: UserProfile,
        name: "user-profile"
    },
    {
        path: "/products/*category",
        component: ProductList,
        name: "products"
    }
];
```

### Link Component

Declarative navigation with automatic active state detection.

```raven
// Basic link
<Link to="/about">About</Link>

// Replace instead of push
<Link to="/login" replace={true}>Login</Link>

// Custom active class
<Link to="/dashboard" activeClass="active">Dashboard</Link>

// Exact match for active state
<Link to="/" exact={true}>Home</Link>
```

### Router Hooks

Access router state and navigation programmatically.

```raven
component UserProfile() {
    // Get router instance
    let router = useRouter();

    // Get current route info
    let route = useRoute();
    console.log(route.path); // "/user/123"

    // Get route parameters
    let params = useParams();
    let userId = params.get("id"); // "123"

    // Get query parameters
    let query = useQuery();
    let tab = query.get("tab"); // From ?tab=settings

    // Navigate programmatically
    let navigate = useNavigate();
    let goToDashboard = () => navigate("/dashboard", false);

    // History navigation
    let back = useBack();
    let forward = useForward();

    <div>
        <h1>User {userId}</h1>
        <button onclick={back}>Back</button>
        <button onclick={goToDashboard}>Go to Dashboard</button>
    </div>
}
```

## 🛡️ Navigation Guards

Protect routes with authentication or permission checks.

### Global Guards

```raven
import { BeforeEach, AfterEach } from "raven-router"

// Run before every navigation
BeforeEach((context) => {
    console.log("Navigating from", context.from, "to", context.to);

    // Check authentication
    if context.to.starts_with("/dashboard") {
        let isAuth = localStorage.getItem("token") != null;
        return isAuth; // Return false to cancel navigation
    }

    return true;
});

// Run after every navigation
AfterEach((context) => {
    // Track page views
    analytics.track("pageview", { path: context.to });
});
```

### Built-in Guards

```raven
import { requireAuth, requirePermission, redirectIfAuth } from "raven-router"

// Require authentication
BeforeEach(requireAuth);

// Require specific permission
BeforeEach(requirePermission("admin"));

// Redirect authenticated users (for login page)
BeforeEach(redirectIfAuth("/dashboard"));
```

### Route-Level Guards

```raven
let routes = [
    {
        path: "/admin",
        component: AdminPanel,
        beforeEnter: (context) => {
            let user = getCurrentUser();
            return user.role == "admin";
        }
    }
];
```

## 🔗 Dynamic Routes

Extract parameters from URLs.

```raven
// Route definition
{
    path: "/blog/:category/:slug",
    component: BlogPost
}

// In component
component BlogPost() {
    let params = useParams();
    let category = params.get("category"); // "tech"
    let slug = params.get("slug");         // "my-post"

    // Fetch blog post data...
}

// URL: /blog/tech/my-post
```

## 🔍 Query Parameters

Access query string parameters.

```raven
component SearchResults() {
    let query = useQuery();
    let searchTerm = query.get("q");      // "raven"
    let page = query.get("page") || "1";  // "2"
    let sort = query.get("sort");         // "date"

    // URL: /search?q=raven&page=2&sort=date
}
```

## 🧭 History Modes

### History Mode (Default)

Uses HTML5 History API for clean URLs.

```raven
<Router routes={routes} mode="history" />
```

URLs: `/about`, `/user/123`

**Requires server configuration** to handle all routes (e.g., Nginx rewrite rules).

### Hash Mode

Uses URL hash for routing (works with static hosting).

```raven
<Router routes={routes} mode="hash" />
```

URLs: `/#/about`, `/#/user/123`

### Memory Mode

In-memory routing for testing or non-browser environments.

```raven
<Router routes={routes} mode="memory" />
```

## 🎨 Active Link Styling

Links automatically get an active class when the current route matches.

```raven
<Link to="/" exact={true}>Home</Link>
<Link to="/about" activeClass="current-page">About</Link>

<style>
    .router-link-active {
        font-weight: bold;
        color: #667eea;
    }

    .current-page {
        border-bottom: 2px solid #764ba2;
    }
</style>
```

## 📊 API Reference

### Router Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `routes` | `Vec<RouteConfig>` | Required | Route configuration array |
| `mode` | `String` | `"history"` | Routing mode: `"history"`, `"hash"`, or `"memory"` |
| `base` | `String` | `""` | Base path for all routes |
| `fallback` | `Any` | 404 component | Component to render when no route matches |

### RouteConfig Type

```raven
type RouteConfig = {
    path: String,               // Route pattern (e.g., "/user/:id")
    component: Any,             // Component to render
    name: String,               // Route name for reference
    children: Vec<RouteConfig>, // Nested routes
    beforeEnter: (Any) -> Bool, // Route-level guard
    meta: Map<String, Any>      // Custom metadata
}
```

### Link Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `to` | `String` | Required | Target path |
| `replace` | `Bool` | `false` | Replace instead of push |
| `activeClass` | `String` | `"router-link-active"` | CSS class when active |
| `exact` | `Bool` | `false` | Exact match for active state |

## 🛠️ Advanced Usage

### Lazy Loading Routes

```raven
let routes = [
    {
        path: "/dashboard",
        component: () => import("./pages/Dashboard.raven")
    }
];
```

### Nested Routes

```raven
let routes = [
    {
        path: "/user/:id",
        component: UserLayout,
        children: [
            {
                path: "profile",
                component: UserProfile
            },
            {
                path: "settings",
                component: UserSettings
            }
        ]
    }
];

// URLs: /user/123/profile, /user/123/settings
```

### Programmatic Navigation with State

```raven
let navigate = useNavigate();

navigate("/user/123?tab=settings", false);
```

## 📄 License

MIT License

## 🤝 Contributing

Contributions welcome! Please see the main RavensOne repository for contribution guidelines.

## 🔗 Links

- **Repository**: https://github.com/jrezin1201/RavensOne
- **Documentation**: https://ravensone.dev/docs/packages/raven-router
- **Registry**: https://registry.ravensone.dev/packages/raven-router
- **Issues**: https://github.com/jrezin1201/RavensOne/issues

---

**Made with ❤️ for the RavensOne community**
