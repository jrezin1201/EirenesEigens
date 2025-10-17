# raven-http

**HTTP Client Library for RavensOne**

A powerful, flexible HTTP client for making API requests in RavensOne applications. Built on the Fetch API with interceptors, retry logic, caching, and more.

## 🚀 Quick Start

### Installation

```bash
raven pkg add raven-http
```

### Basic Usage

```raven
import { get, post } from "raven-http"

// Simple GET request
get("https://api.example.com/users")
    .then((response) => {
        console.log(response.data);
    })
    .catch((error) => {
        console.error(error);
    });

// POST request with data
post("https://api.example.com/users", {
    name: "John Doe",
    email: "john@example.com"
})
    .then((response) => {
        console.log("User created:", response.data);
    });
```

## 📦 Features

- ✅ **Fetch API Wrapper** - Modern, promise-based HTTP client
- ✅ **Interceptors** - Transform requests/responses globally
- ✅ **Automatic Retries** - Configurable retry logic for failed requests
- ✅ **Response Caching** - Cache GET requests to reduce network calls
- ✅ **Request/Response Transformation** - Automatic JSON parsing
- ✅ **Timeout Support** - Cancel long-running requests
- ✅ **Progress Tracking** - Monitor upload/download progress
- ✅ **TypeScript-like Types** - Full type safety
- ✅ **GraphQL & JSON-RPC** - Built-in helpers for common protocols

## 🎯 Creating a Client

### Basic Client

```raven
import { createHttpClient } from "raven-http"

let client = createHttpClient({
    baseURL: "https://api.example.com",
    timeout: 30000,
    headers: {
        "Content-Type": "application/json",
        "X-Custom-Header": "value"
    },
    withCredentials: false,
    responseType: "json"
});
```

### Client with Interceptors

```raven
import { createHttpClient, authInterceptor, loggingRequestInterceptor } from "raven-http"

let client = createHttpClient({
    baseURL: "https://api.example.com"
});

// Add authentication
let token = localStorage.getItem("auth_token");
client.use_request_interceptor(authInterceptor(token));

// Add logging
client.use_request_interceptor(loggingRequestInterceptor());
client.use_response_interceptor(loggingResponseInterceptor());
```

## 🔧 Making Requests

### GET Request

```raven
// Simple GET
client.get("/users")
    .then((response) => console.log(response.data));

// GET with query parameters
client.get("/users", {
    params: {
        page: "1",
        limit: "10",
        sort: "name"
    }
})
    .then((response) => console.log(response.data));
```

### POST Request

```raven
client.post("/users", {
    name: "Jane Doe",
    email: "jane@example.com",
    role: "admin"
})
    .then((response) => console.log("Created:", response.data));
```

### PUT Request

```raven
client.put("/users/123", {
    name: "Jane Smith",
    email: "jane.smith@example.com"
})
    .then((response) => console.log("Updated:", response.data));
```

### PATCH Request

```raven
client.patch("/users/123", {
    email: "newemail@example.com"
})
    .then((response) => console.log("Patched:", response.data));
```

### DELETE Request

```raven
client.delete("/users/123")
    .then((response) => console.log("Deleted"));
```

## 🎛️ Interceptors

Interceptors allow you to transform requests before they're sent and responses before they're returned.

### Request Interceptors

```raven
// Add authentication token
client.use_request_interceptor((config) => {
    let token = localStorage.getItem("token");
    if token {
        config.headers.insert("Authorization", `Bearer ${token}`);
    }
    return config;
});

// Add timestamp (cache busting)
client.use_request_interceptor((config) => {
    if !config.params {
        config.params = Map::new();
    }
    config.params.insert("_t", Date.now().toString());
    return config;
});

// Log all requests
client.use_request_interceptor((config) => {
    console.log(`[Request] ${config.method.to_string()} ${config.url}`);
    return config;
});
```

### Response Interceptors

```raven
// Transform errors
client.use_response_interceptor((response) => {
    if !response.is_ok() {
        console.error(`Error ${response.status}: ${response.statusText}`);
    }
    return response;
});

// Extract nested data
client.use_response_interceptor((response) => {
    if response.data && response.data.result {
        response.data = response.data.result;
    }
    return response;
});

// Log response time
let startTime = Date.now();
client.use_request_interceptor((config) => {
    config._startTime = Date.now();
    return config;
});

client.use_response_interceptor((response) => {
    let duration = Date.now() - response.config._startTime;
    console.log(`Request took ${duration}ms`);
    return response;
});
```

### Built-in Interceptors

```raven
import {
    authInterceptor,
    loggingRequestInterceptor,
    loggingResponseInterceptor,
    apiKeyInterceptor,
    timestampInterceptor,
    errorTransformInterceptor
} from "raven-http"

// Authentication
client.use_request_interceptor(authInterceptor("your-token"));

// API Key
client.use_request_interceptor(apiKeyInterceptor("your-api-key", "key"));

// Logging
client.use_request_interceptor(loggingRequestInterceptor());
client.use_response_interceptor(loggingResponseInterceptor());

// Error transformation
client.use_response_interceptor(errorTransformInterceptor());
```

## 🔄 Retry Logic

Automatically retry failed requests.

```raven
import { retryInterceptor } from "raven-http"

client.use_response_interceptor(
    retryInterceptor(
        maxRetries: 3,
        retryDelay: 1000 // 1 second
    )
);
```

## 💾 Response Caching

Cache GET requests to reduce network calls.

```raven
import { cacheInterceptor } from "raven-http"

let cache = cacheInterceptor(300000); // 5 minutes

client.use_request_interceptor(cache.request);
client.use_response_interceptor(cache.response);
```

## 📊 Response Object

```raven
type Response = {
    data: Any,              // Response body
    status: Int,            // HTTP status code (200, 404, etc.)
    statusText: String,     // Status text ("OK", "Not Found", etc.)
    headers: Headers,       // Response headers
    config: RequestConfig   // Original request config
}

// Helper methods
response.is_ok()           // true if 2xx
response.is_client_error() // true if 4xx
response.is_server_error() // true if 5xx
response.get_header("Content-Type")
```

## 🛠️ Advanced Features

### File Upload

```raven
import { uploadFile } from "raven-http"

let fileInput = document.querySelector("#file-input");
let file = fileInput.files[0];

uploadFile("/upload", file, "file")
    .then((response) => {
        console.log("Upload complete:", response.data);
    });
```

### File Download

```raven
import { downloadFile } from "raven-http"

downloadFile("/files/report.pdf", "monthly-report.pdf")
    .then(() => {
        console.log("Download started");
    });
```

### Batch Requests

```raven
import { batch } from "raven-http"

let requests = [
    get("/users"),
    get("/posts"),
    get("/comments")
];

batch(requests)
    .then((responses) => {
        let [users, posts, comments] = responses;
        console.log(users.data, posts.data, comments.data);
    });
```

### Parallel Requests with Limit

```raven
import { parallel } from "raven-http"

let userIds = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

let requests = userIds.map(id => {
    return () => get(`/users/${id}`);
});

// Execute max 3 requests at a time
parallel(requests, 3)
    .then((responses) => {
        console.log("All users loaded:", responses);
    });
```

### GraphQL

```raven
import { graphql } from "raven-http"

let query = `
    query GetUser($id: ID!) {
        user(id: $id) {
            name
            email
            posts {
                title
            }
        }
    }
`;

graphql("https://api.example.com/graphql", query, {
    id: "123"
})
    .then((response) => {
        console.log(response.data.data.user);
    });
```

### JSON-RPC

```raven
import { jsonrpc } from "raven-http"

jsonrpc("https://api.example.com/rpc", "getUser", ["123"], 1)
    .then((response) => {
        console.log(response.data.result);
    });
```

## ⚙️ Configuration

### HttpConfig

```raven
type HttpConfig = {
    baseURL: String,              // Base URL for all requests
    timeout: Int,                 // Request timeout in milliseconds
    headers: Map<String, String>, // Default headers
    withCredentials: Bool,        // Include credentials (cookies)
    responseType: String          // "json" | "text" | "blob" | "arraybuffer"
}
```

### RequestConfig

```raven
type RequestConfig = {
    url: String,
    method: RequestMethod,
    headers: Map<String, String>,
    params: Map<String, String>,     // Query parameters
    data: Any,                       // Request body
    timeout: Int,
    responseType: String,
    withCredentials: Bool,
    onUploadProgress: (Event) -> Void,
    onDownloadProgress: (Event) -> Void
}
```

## 🎨 Usage with Components

```raven
import { get } from "raven-http"
import { Signal, Effect } from "raven:core"

component UserList() {
    let users = Signal::new([]);
    let loading = Signal::new(true);
    let error = Signal::new(null);

    Effect::new(|| {
        get("https://api.example.com/users")
            .then((response) => {
                users.set(response.data);
                loading.set(false);
            })
            .catch((err) => {
                error.set(err.message);
                loading.set(false);
            });
    });

    <div>
        {if loading.get() {
            <p>Loading...</p>
        } else if error.get() {
            <p>Error: {error.get()}</p>
        } else {
            <ul>
                {users.get().map(user => {
                    <li>{user.name}</li>
                })}
            </ul>
        }}
    </div>
}
```

## 📄 License

MIT License

## 🤝 Contributing

Contributions welcome! Please see the main RavensOne repository for contribution guidelines.

## 🔗 Links

- **Repository**: https://github.com/jrezin1201/RavensOne
- **Documentation**: https://ravensone.dev/docs/packages/raven-http
- **Registry**: https://registry.ravensone.dev/packages/raven-http
- **Issues**: https://github.com/jrezin1201/RavensOne/issues

---

**Made with ❤️ for the RavensOne community**
