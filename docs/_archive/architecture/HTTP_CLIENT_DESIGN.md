# HTTP Client Library - Design & Implementation Plan

## Current State

The HTTP client has excellent type definitions but is **missing the actual HTTP execution**.

**What exists** (src/stdlib/http.rs):
- ✅ `HttpMethod` enum (GET, POST, PUT, DELETE, etc.)
- ✅ `HttpRequest` builder pattern
- ✅ `HttpResponse` with status checking
- ✅ `HttpClient` with base URL support
- ✅ Unit tests for types

**What's missing**:
- ❌ No actual HTTP request execution
- ❌ No async/await implementation
- ❌ No integration with reqwest or similar
- ❌ No JSON serialization/deserialization
- ❌ No error handling for network failures

## Design Goals

1. **Simple API** - Easy to use from .raven code
2. **Async by default** - Modern async/await pattern
3. **Type-safe** - Leverage Rust's type system
4. **Cross-platform** - Work in both CLI and WASM contexts
5. **Ergonomic** - Builder pattern with chaining

## Proposed API (How it should work in .raven)

```raven
// Simple GET request
let response = http::get("https://api.example.com/users").send().await?;
let users = response.json();

// POST with JSON
let body = json!({
    "name": "Jordan",
    "email": "jordan@example.com"
});

let response = http::post("https://api.example.com/users")
    .header("Authorization", "Bearer token")
    .json(body)
    .send()
    .await?;

// Using a configured client
let client = http::Client::new()
    .with_base_url("https://api.example.com")
    .with_header("User-Agent", "RavensOne/1.0");

let users = client.get("/users").send().await?;
let user = client.get("/users/123").send().await?;
```

## Implementation Plan

### Phase 1: Add HTTP Execution (NOW)

Add actual HTTP request execution using `reqwest`:

```rust
impl HttpRequest {
    pub async fn send(self) -> Result<HttpResponse, HttpError> {
        // Use reqwest to actually make the HTTP request
    }
}
```

**Dependencies needed**:
- `reqwest` - HTTP client library
- `tokio` - Async runtime (already have it)
- `serde_json` - JSON handling (already have it)

### Phase 2: Error Handling

Add proper error types:

```rust
pub enum HttpError {
    NetworkError(String),
    InvalidUrl(String),
    TimeoutError,
    JsonParseError(String),
    StatusError { status: u16, body: String },
}
```

### Phase 3: JSON Support

Add real JSON serialization:

```rust
impl HttpResponse {
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T, HttpError> {
        serde_json::from_str(&self.body)
            .map_err(|e| HttpError::JsonParseError(e.to_string()))
    }
}

impl HttpRequest {
    pub fn json<T: serde::Serialize>(mut self, data: &T) -> Self {
        let json_str = serde_json::to_string(data).unwrap();
        self.body(json_str)
    }
}
```

### Phase 4: Convenience Methods

Add shorthand functions:

```rust
// Simple GET
pub async fn get(url: &str) -> Result<HttpResponse, HttpError> {
    HttpRequest::get(url).send().await
}

// Simple POST with JSON
pub async fn post_json<T: serde::Serialize>(
    url: &str,
    data: &T
) -> Result<HttpResponse, HttpError> {
    HttpRequest::post(url).json(data).send().await
}
```

### Phase 5: Integration with Compiler

Make HTTP module available in .raven code:

1. Add to stdlib exports
2. Register in module system
3. Add type definitions for .raven
4. Generate proper WASM bindings

## Technical Challenges

### Challenge 1: WASM Compatibility

**Problem**: `reqwest` needs different backends for native vs WASM

**Solution**: Use feature flags

```toml
[dependencies]
reqwest = { version = "0.11", features = ["json"] }

[target.'cfg(target_arch = "wasm32")'.dependencies]
reqwest = { version = "0.11", features = ["json"], default-features = false }
```

### Challenge 2: Async Runtime in .raven

**Problem**: .raven doesn't have async/await yet

**Solution**: For now, provide blocking API, add async later:

```rust
// Blocking version (for now)
pub fn send_blocking(self) -> Result<HttpResponse, HttpError> {
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(self.send())
}
```

### Challenge 3: Error Propagation

**Problem**: .raven doesn't have `?` operator yet

**Solution**: Provide `.unwrap()` and `.expect()` helpers:

```raven
let response = http::get("url").send().unwrap();
```

## Testing Strategy

### Unit Tests

```rust
#[tokio::test]
async fn test_get_request() {
    let resp = HttpRequest::get("https://httpbin.org/get")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status, 200);
    assert!(resp.is_ok());
}

#[tokio::test]
async fn test_post_json() {
    let data = json!({ "name": "test" });
    let resp = HttpRequest::post("https://httpbin.org/post")
        .json(&data)
        .send()
        .await
        .unwrap();

    assert!(resp.is_ok());
}
```

### Integration Tests

Test against real APIs:
- httpbin.org - Echo service
- jsonplaceholder.typicode.com - Fake REST API
- Our own Bluebird backend - http://localhost:9000

### Example Programs

Create example .raven programs:

1. **fetch_users.raven** - Fetch list from API
2. **post_data.raven** - POST JSON to API
3. **api_client.raven** - Full CRUD operations
4. **weather_cli.raven** - Fetch weather data
5. **github_stats.raven** - Fetch GitHub repo stats

## Success Criteria

✅ **Phase 1 Complete** when:
- Can make GET requests to real URLs
- Can make POST requests with JSON
- Tests pass against httpbin.org
- Can fetch from Bluebird backend

✅ **Phase 2 Complete** when:
- Proper error handling for network failures
- Timeout support
- Connection error messages

✅ **Phase 3 Complete** when:
- JSON serialization/deserialization works
- Can parse complex JSON responses
- Type-safe JSON handling

✅ **Phase 4 Complete** when:
- Convenience functions work
- Ergonomic API for common cases
- Documentation with examples

✅ **Phase 5 Complete** when:
- Accessible from .raven code
- Example programs run successfully
- Integrated into stdlib

## Timeline

- **Phase 1**: 2-3 hours (Add reqwest integration)
- **Phase 2**: 1 hour (Error handling)
- **Phase 3**: 1-2 hours (JSON support)
- **Phase 4**: 1 hour (Convenience methods)
- **Phase 5**: 2-4 hours (Compiler integration)

**Total**: 7-11 hours of work

## Next Immediate Steps

1. Add `reqwest` to Cargo.toml
2. Implement `HttpRequest::send()` method
3. Write basic test against httpbin.org
4. Verify it works with Bluebird backend
5. Create first example .raven program

---

## Implementation Status

**✅ Phase 1 COMPLETE**: HTTP Execution
- ✅ Added `HttpError` enum with proper error types
- ✅ Implemented `HttpRequest::send()` async method
- ✅ Implemented `HttpRequest::send_blocking()` method
- ✅ Full reqwest integration with proper error handling
- ✅ Header extraction from responses

**✅ Phase 2 COMPLETE**: Error Handling
- ✅ `HttpError` enum with Display and Error traits
- ✅ Network error handling
- ✅ JSON parse error handling
- ✅ Status code checking

**✅ Phase 3 COMPLETE**: JSON Support
- ✅ `HttpResponse::json()` returns `serde_json::Value`
- ✅ `HttpResponse::json_as<T>()` for type-safe deserialization
- ✅ JSON request body with automatic Content-Type header

**✅ Phase 4 COMPLETE**: Convenience Methods
- ✅ `http::get(url)` - Simple async GET
- ✅ `http::get_blocking(url)` - Simple blocking GET
- ✅ `http::post_json(url, data)` - Simple POST with JSON
- ✅ `http::post_json_blocking(url, data)` - Blocking POST with JSON

**✅ Phase 5 COMPLETE**: Testing & Examples
- ✅ 12 comprehensive tests (all passing)
- ✅ Integration tests against httpbin.org
- ✅ Blocking and async versions tested
- ✅ Example program against Bluebird backend

**🚧 Phase 6 IN PROGRESS**: Compiler Integration
- ⏳ Make HTTP module accessible from .raven code
- ⏳ Add type definitions for .raven
- ⏳ Generate WASM bindings

---

**Status**: Implementation Complete! Ready for .raven integration
**Date**: 2025-10-18
**Lines of Code**: 557 lines (src/stdlib/http.rs)
**Tests**: 12 passing
**Goal**: ✅ Enable Rust code to make HTTP requests

## Usage Examples

### Rust Code (Working Now!)

```rust
use ravensone_compiler::stdlib::http;

// Simple GET request
let response = http::get("https://api.example.com/users")
    .await?;
let users = response.json()?;

// POST with JSON
let data = serde_json::json!({
    "name": "Jordan",
    "email": "jordan@example.com"
});

let response = http::post_json("https://api.example.com/users", data)
    .await?;

// Using a configured client
let client = http::HttpClient::new()
    .with_base_url("https://api.example.com".to_string())
    .with_header("Authorization".to_string(), "Bearer token".to_string());

let users = client.get("/users").send().await?;
```

### Blocking Version (For Non-Async Contexts)

```rust
// Blocking GET
let response = http::get_blocking("https://api.example.com/users")?;
let users = response.json()?;

// Blocking POST
let data = serde_json::json!({"test": "data"});
let response = http::post_json_blocking("https://api.example.com/users", data)?;
```

### Running the Demo

```bash
# Start the Bluebird backend
cd examples/bluebird-backend
./target/release/bluebird-backend

# Run the HTTP client demo
cargo run --example http_client_demo
```

## Test Results

```
running 12 tests
test stdlib::http::tests::test_response_status ... ok
test stdlib::http::tests::test_request_builder ... ok
test stdlib::http::tests::test_http_client ... ok
test stdlib::http::tests::test_custom_headers ... ok
test stdlib::http::tests::test_get_request ... ok
test stdlib::http::tests::test_convenience_get ... ok
test stdlib::http::tests::test_post_json ... ok
test stdlib::http::tests::test_blocking_post_json ... ok
test stdlib::http::tests::test_blocking_get ... ok
test stdlib::http::tests::test_http_client_with_base_url ... ok
test stdlib::http::tests::test_404_error ... ok
test stdlib::http::tests::test_json_parsing ... ok

test result: ok. 12 passed; 0 failed; 0 ignored
```

## Real-World Test Against Bluebird Backend

Successfully tested all operations:
- ✅ GET /api/posts - Fetched 6 posts with full JSON parsing
- ✅ POST /api/posts/:id/like - Toggled like on a post
- ✅ GET /api/posts/:id/comments - Fetched comments (empty array)
- ✅ POST /api/posts/:id/comments - Created a new comment with emoji support 🚀
