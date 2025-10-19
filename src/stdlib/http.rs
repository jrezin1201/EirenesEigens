/// HTTP client for RavensOne
///
/// Provides type-safe HTTP requests with async/await support
/// Works on both client (fetch API) and server (native HTTP)

use std::collections::HashMap;

/// HTTP error types
#[derive(Debug, Clone)]
pub enum HttpError {
    NetworkError(String),
    InvalidUrl(String),
    TimeoutError,
    JsonParseError(String),
    StatusError { status: u16, body: String },
    RequestBuildError(String),
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            HttpError::InvalidUrl(msg) => write!(f, "Invalid URL: {}", msg),
            HttpError::TimeoutError => write!(f, "Request timed out"),
            HttpError::JsonParseError(msg) => write!(f, "JSON parse error: {}", msg),
            HttpError::StatusError { status, body } => {
                write!(f, "HTTP error {}: {}", status, body)
            }
            HttpError::RequestBuildError(msg) => write!(f, "Request build error: {}", msg),
        }
    }
}

impl std::error::Error for HttpError {}

/// HTTP method types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

impl HttpMethod {
    pub fn as_str(&self) -> &str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Head => "HEAD",
            HttpMethod::Options => "OPTIONS",
        }
    }
}

/// HTTP request builder
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub url: String,
    pub method: HttpMethod,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl HttpRequest {
    /// Create a new GET request
    pub fn get(url: &str) -> Self {
        Self {
            url: url.to_string(),
            method: HttpMethod::Get,
            headers: HashMap::new(),
            body: None,
        }
    }

    /// Create a new POST request
    pub fn post(url: &str) -> Self {
        Self {
            url: url.to_string(),
            method: HttpMethod::Post,
            headers: HashMap::new(),
            body: None,
        }
    }

    /// Create a new PUT request
    pub fn put(url: &str) -> Self {
        Self {
            url: url.to_string(),
            method: HttpMethod::Put,
            headers: HashMap::new(),
            body: None,
        }
    }

    /// Create a new DELETE request
    pub fn delete(url: &str) -> Self {
        Self {
            url: url.to_string(),
            method: HttpMethod::Delete,
            headers: HashMap::new(),
            body: None,
        }
    }

    /// Add a header
    pub fn header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }

    /// Set JSON body
    pub fn json(mut self, body: String) -> Self {
        self.headers.insert("Content-Type".to_string(), "application/json".to_string());
        self.body = Some(body);
        self
    }

    /// Set text body
    pub fn body(mut self, body: String) -> Self {
        self.body = Some(body);
        self
    }

    /// Send the HTTP request (async version)
    pub async fn send(self) -> Result<HttpResponse, HttpError> {
        let client = reqwest::Client::new();

        // Convert HttpMethod to reqwest::Method
        let method = match self.method {
            HttpMethod::Get => reqwest::Method::GET,
            HttpMethod::Post => reqwest::Method::POST,
            HttpMethod::Put => reqwest::Method::PUT,
            HttpMethod::Delete => reqwest::Method::DELETE,
            HttpMethod::Patch => reqwest::Method::PATCH,
            HttpMethod::Head => reqwest::Method::HEAD,
            HttpMethod::Options => reqwest::Method::OPTIONS,
        };

        // Build the request
        let mut request_builder = client.request(method, &self.url);

        // Add headers
        for (key, value) in self.headers {
            request_builder = request_builder.header(&key, &value);
        }

        // Add body if present
        if let Some(body) = self.body {
            request_builder = request_builder.body(body);
        }

        // Send the request
        let response = request_builder
            .send()
            .await
            .map_err(|e| HttpError::NetworkError(e.to_string()))?;

        // Extract response data
        let status = response.status().as_u16();
        let status_text = response.status().to_string();

        // Extract headers
        let mut headers = HashMap::new();
        for (key, value) in response.headers() {
            if let Ok(value_str) = value.to_str() {
                headers.insert(key.to_string(), value_str.to_string());
            }
        }

        // Get body
        let body = response
            .text()
            .await
            .map_err(|e| HttpError::NetworkError(e.to_string()))?;

        Ok(HttpResponse {
            status,
            status_text,
            headers,
            body,
        })
    }

    /// Send the HTTP request (blocking version for non-async contexts)
    pub fn send_blocking(self) -> Result<HttpResponse, HttpError> {
        let client = reqwest::blocking::Client::new();

        // Convert HttpMethod to reqwest::Method
        let method = match self.method {
            HttpMethod::Get => reqwest::Method::GET,
            HttpMethod::Post => reqwest::Method::POST,
            HttpMethod::Put => reqwest::Method::PUT,
            HttpMethod::Delete => reqwest::Method::DELETE,
            HttpMethod::Patch => reqwest::Method::PATCH,
            HttpMethod::Head => reqwest::Method::HEAD,
            HttpMethod::Options => reqwest::Method::OPTIONS,
        };

        // Build the request
        let mut request_builder = client.request(method, &self.url);

        // Add headers
        for (key, value) in self.headers {
            request_builder = request_builder.header(&key, &value);
        }

        // Add body if present
        if let Some(body) = self.body {
            request_builder = request_builder.body(body);
        }

        // Send the request
        let response = request_builder
            .send()
            .map_err(|e| HttpError::NetworkError(e.to_string()))?;

        // Extract response data
        let status = response.status().as_u16();
        let status_text = response.status().to_string();

        // Extract headers
        let mut headers = HashMap::new();
        for (key, value) in response.headers() {
            if let Ok(value_str) = value.to_str() {
                headers.insert(key.to_string(), value_str.to_string());
            }
        }

        // Get body
        let body = response
            .text()
            .map_err(|e| HttpError::NetworkError(e.to_string()))?;

        Ok(HttpResponse {
            status,
            status_text,
            headers,
            body,
        })
    }
}

/// HTTP response
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl HttpResponse {
    /// Create a new response
    pub fn new(status: u16, status_text: String) -> Self {
        Self {
            status,
            status_text,
            headers: HashMap::new(),
            body: String::new(),
        }
    }

    /// Check if response is successful (2xx)
    pub fn is_ok(&self) -> bool {
        self.status >= 200 && self.status < 300
    }

    /// Check if response is a client error (4xx)
    pub fn is_client_error(&self) -> bool {
        self.status >= 400 && self.status < 500
    }

    /// Check if response is a server error (5xx)
    pub fn is_server_error(&self) -> bool {
        self.status >= 500 && self.status < 600
    }

    /// Parse response as JSON (returns serde_json::Value)
    pub fn json(&self) -> Result<serde_json::Value, HttpError> {
        serde_json::from_str(&self.body)
            .map_err(|e| HttpError::JsonParseError(e.to_string()))
    }

    /// Parse response as JSON into a specific type
    pub fn json_as<T: serde::de::DeserializeOwned>(&self) -> Result<T, HttpError> {
        serde_json::from_str(&self.body)
            .map_err(|e| HttpError::JsonParseError(e.to_string()))
    }

    /// Get response as text
    pub fn text(&self) -> String {
        self.body.clone()
    }
}

/// HTTP client
pub struct HttpClient {
    base_url: Option<String>,
    default_headers: HashMap<String, String>,
}

impl HttpClient {
    /// Create a new HTTP client
    pub fn new() -> Self {
        Self {
            base_url: None,
            default_headers: HashMap::new(),
        }
    }

    /// Set base URL for all requests
    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = Some(url);
        self
    }

    /// Add a default header for all requests
    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.default_headers.insert(key, value);
        self
    }

    /// Make a GET request
    pub fn get(&self, url: &str) -> HttpRequest {
        self.build_request(HttpRequest::get(url))
    }

    /// Make a POST request
    pub fn post(&self, url: &str) -> HttpRequest {
        self.build_request(HttpRequest::post(url))
    }

    /// Make a PUT request
    pub fn put(&self, url: &str) -> HttpRequest {
        self.build_request(HttpRequest::put(url))
    }

    /// Make a DELETE request
    pub fn delete(&self, url: &str) -> HttpRequest {
        self.build_request(HttpRequest::delete(url))
    }

    /// Build request with default headers and base URL
    fn build_request(&self, mut req: HttpRequest) -> HttpRequest {
        // Add base URL if set
        if let Some(ref base) = self.base_url {
            req.url = format!("{}{}", base, req.url);
        }

        // Add default headers
        for (key, value) in &self.default_headers {
            if !req.headers.contains_key(key) {
                req.headers.insert(key.clone(), value.clone());
            }
        }

        req
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

// Convenience functions for quick HTTP requests

/// Make a simple GET request (async)
pub async fn get(url: &str) -> Result<HttpResponse, HttpError> {
    HttpRequest::get(url).send().await
}

/// Make a simple GET request (blocking)
pub fn get_blocking(url: &str) -> Result<HttpResponse, HttpError> {
    HttpRequest::get(url).send_blocking()
}

/// Make a simple POST request with JSON body (async)
pub async fn post_json(url: &str, json: serde_json::Value) -> Result<HttpResponse, HttpError> {
    let body = serde_json::to_string(&json)
        .map_err(|e| HttpError::JsonParseError(e.to_string()))?;
    HttpRequest::post(url).json(body).send().await
}

/// Make a simple POST request with JSON body (blocking)
pub fn post_json_blocking(url: &str, json: serde_json::Value) -> Result<HttpResponse, HttpError> {
    let body = serde_json::to_string(&json)
        .map_err(|e| HttpError::JsonParseError(e.to_string()))?;
    HttpRequest::post(url).json(body).send_blocking()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_builder() {
        let req = HttpRequest::get("https://api.example.com/users")
            .header("Authorization".to_string(), "Bearer token".to_string());

        assert_eq!(req.method, HttpMethod::Get);
        assert_eq!(req.url, "https://api.example.com/users");
        assert_eq!(req.headers.get("Authorization").unwrap(), "Bearer token");
    }

    #[test]
    fn test_response_status() {
        let resp = HttpResponse::new(200, "OK".to_string());
        assert!(resp.is_ok());

        let resp = HttpResponse::new(404, "Not Found".to_string());
        assert!(resp.is_client_error());

        let resp = HttpResponse::new(500, "Internal Server Error".to_string());
        assert!(resp.is_server_error());
    }

    #[test]
    fn test_http_client() {
        let client = HttpClient::new()
            .with_base_url("https://api.example.com".to_string())
            .with_header("User-Agent".to_string(), "RavensOne/1.0".to_string());

        let req = client.get("/users");
        assert_eq!(req.url, "https://api.example.com/users");
        assert_eq!(req.headers.get("User-Agent").unwrap(), "RavensOne/1.0");
    }

    // Integration tests (require network access)
    #[tokio::test]
    async fn test_get_request() {
        let resp = HttpRequest::get("https://httpbin.org/get")
            .send()
            .await
            .expect("Failed to make GET request");

        assert_eq!(resp.status, 200);
        assert!(resp.is_ok());
        assert!(!resp.body.is_empty());
    }

    #[tokio::test]
    async fn test_post_json() {
        let json_body = serde_json::json!({
            "name": "RavensOne",
            "version": "1.0"
        });

        let body_str = serde_json::to_string(&json_body).unwrap();

        let resp = HttpRequest::post("https://httpbin.org/post")
            .json(body_str)
            .send()
            .await
            .expect("Failed to make POST request");

        assert!(resp.is_ok());

        // Parse response JSON
        let response_json = resp.json().expect("Failed to parse JSON");
        assert!(response_json["json"]["name"].as_str().unwrap() == "RavensOne");
    }

    #[tokio::test]
    async fn test_custom_headers() {
        let resp = HttpRequest::get("https://httpbin.org/headers")
            .header("X-Custom-Header".to_string(), "RavensOne".to_string())
            .send()
            .await
            .expect("Failed to make request with headers");

        assert!(resp.is_ok());

        let response_json = resp.json().expect("Failed to parse JSON");
        assert!(response_json["headers"]["X-Custom-Header"]
            .as_str()
            .unwrap()
            .contains("RavensOne"));
    }

    #[tokio::test]
    async fn test_convenience_get() {
        let resp = get("https://httpbin.org/get")
            .await
            .expect("Failed to make GET request");

        assert!(resp.is_ok());
    }

    #[test]
    fn test_blocking_get() {
        let resp = get_blocking("https://httpbin.org/get")
            .expect("Failed to make blocking GET request");

        assert!(resp.is_ok());
        assert_eq!(resp.status, 200);
    }

    #[test]
    fn test_blocking_post_json() {
        let json_data = serde_json::json!({
            "test": "data"
        });

        let resp = post_json_blocking("https://httpbin.org/post", json_data)
            .expect("Failed to make blocking POST request");

        assert!(resp.is_ok());
    }

    #[tokio::test]
    async fn test_http_client_with_base_url() {
        let client = HttpClient::new()
            .with_base_url("https://httpbin.org".to_string());

        let resp = client.get("/get")
            .send()
            .await
            .expect("Failed to make request");

        assert!(resp.is_ok());
    }

    #[tokio::test]
    async fn test_404_error() {
        let resp = HttpRequest::get("https://httpbin.org/status/404")
            .send()
            .await
            .expect("Failed to make request");

        assert_eq!(resp.status, 404);
        assert!(resp.is_client_error());
        assert!(!resp.is_ok());
    }

    #[tokio::test]
    async fn test_json_parsing() {
        let resp = HttpRequest::get("https://httpbin.org/json")
            .send()
            .await
            .expect("Failed to make request");

        assert!(resp.is_ok());

        let json = resp.json().expect("Failed to parse JSON");
        assert!(json.is_object());
    }
}
