/// HTTP client for RavensOne
///
/// Provides type-safe HTTP requests with async/await support
/// Works on both client (fetch API) and server (native HTTP)

use std::collections::HashMap;

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

    /// Parse response as JSON
    /// In a full implementation, this would deserialize to the given type
    pub fn json(&self) -> String {
        self.body.clone()
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
}
