use reqwest::blocking::{Client, multipart};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::io::{self, Write};

const DEFAULT_REGISTRY: &str = "http://localhost:4000";
const TOKEN_FILE: &str = ".raven/token";
const CONFIG_DIR: &str = ".raven";

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub user_id: String,
    pub username: String,
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user_id: String,
    pub username: String,
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublishRequest {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub authors: Vec<String>,
    pub license: String,
    pub repository: Option<String>,
    pub homepage: Option<String>,
    pub keywords: Vec<String>,
    pub dependencies: std::collections::HashMap<String, String>,
    pub dev_dependencies: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublishResponse {
    pub package_id: String,
    pub name: String,
    pub version: String,
    pub published_at: String,
    pub download_url: String,
    pub checksum: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub keywords: Vec<String>,
    pub downloads: i64,
    pub score: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

pub struct RegistryClient {
    client: Client,
    registry_url: String,
}

impl RegistryClient {
    pub fn new(registry_url: Option<&str>) -> Self {
        Self {
            client: Client::new(),
            registry_url: registry_url.unwrap_or(DEFAULT_REGISTRY).to_string(),
        }
    }

    /// Register a new user with the registry
    pub fn register(&self, username: &str, email: &str, password: &str) -> Result<RegisterResponse, Box<dyn std::error::Error>> {
        let url = format!("{}/api/v1/auth/register", self.registry_url);

        let request = RegisterRequest {
            username: username.to_string(),
            email: email.to_string(),
            password: password.to_string(),
        };

        let response = self.client
            .post(&url)
            .json(&request)
            .send()?;

        if response.status().is_success() {
            let result: RegisterResponse = response.json()?;
            Ok(result)
        } else {
            let error_text = response.text()?;
            Err(format!("Registration failed: {}", error_text).into())
        }
    }

    /// Login to the registry
    pub fn login(&self, username: &str, password: &str) -> Result<LoginResponse, Box<dyn std::error::Error>> {
        let url = format!("{}/api/v1/auth/login", self.registry_url);

        let request = LoginRequest {
            username: username.to_string(),
            password: password.to_string(),
        };

        let response = self.client
            .post(&url)
            .json(&request)
            .send()?;

        if response.status().is_success() {
            let result: LoginResponse = response.json()?;
            Ok(result)
        } else {
            let error_text = response.text()?;
            Err(format!("Login failed: {}", error_text).into())
        }
    }

    /// Publish a package to the registry
    pub fn publish(&self, metadata: PublishRequest, tarball_path: &Path) -> Result<PublishResponse, Box<dyn std::error::Error>> {
        let token = self.load_token()?;
        let url = format!("{}/api/v1/packages/publish", self.registry_url);

        // Read tarball
        let tarball_bytes = fs::read(tarball_path)?;

        // Create multipart form
        let metadata_json = serde_json::to_string(&metadata)?;

        let form = multipart::Form::new()
            .text("metadata", metadata_json)
            .part("tarball", multipart::Part::bytes(tarball_bytes)
                .file_name(tarball_path.file_name().unwrap().to_string_lossy().to_string()));

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .multipart(form)
            .send()?;

        if response.status().is_success() {
            let result: PublishResponse = response.json()?;
            Ok(result)
        } else {
            let error_text = response.text()?;
            Err(format!("Publish failed: {}", error_text).into())
        }
    }

    /// Search for packages in the registry
    pub fn search(&self, query: &str, limit: i64, offset: i64) -> Result<SearchResponse, Box<dyn std::error::Error>> {
        let url = format!("{}/api/v1/search?q={}&limit={}&offset={}",
            self.registry_url, query, limit, offset);

        let response = self.client
            .get(&url)
            .send()?;

        if response.status().is_success() {
            let result: SearchResponse = response.json()?;
            Ok(result)
        } else {
            let error_text = response.text()?;
            Err(format!("Search failed: {}", error_text).into())
        }
    }

    /// Download a package from the registry
    pub fn download(&self, name: &str, version: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let url = format!("{}/api/v1/packages/{}/{}/download",
            self.registry_url, name, version);

        let response = self.client
            .get(&url)
            .send()?;

        if response.status().is_success() {
            let bytes = response.bytes()?;
            Ok(bytes.to_vec())
        } else {
            let error_text = response.text()?;
            Err(format!("Download failed: {}", error_text).into())
        }
    }

    /// Save authentication token to disk
    pub fn save_token(&self, token: &str) -> Result<(), Box<dyn std::error::Error>> {
        let config_dir = PathBuf::from(CONFIG_DIR);
        fs::create_dir_all(&config_dir)?;

        let token_path = PathBuf::from(TOKEN_FILE);
        fs::write(&token_path, token)?;

        println!("🔐 Token saved to {}", TOKEN_FILE);
        println!("⚠️  Keep this file secure and add it to .gitignore!");

        Ok(())
    }

    /// Load authentication token from disk
    pub fn load_token(&self) -> Result<String, Box<dyn std::error::Error>> {
        let token_path = PathBuf::from(TOKEN_FILE);

        if !token_path.exists() {
            return Err("Authentication token not found. Run 'raven login' first.".into());
        }

        let token = fs::read_to_string(&token_path)?
            .trim()
            .to_string();

        Ok(token)
    }

    /// Check if user is authenticated
    pub fn is_authenticated(&self) -> bool {
        PathBuf::from(TOKEN_FILE).exists()
    }
}

/// Interactive login prompt
pub fn prompt_login() -> Result<(String, String), Box<dyn std::error::Error>> {
    print!("Username: ");
    io::stdout().flush()?;
    let mut username = String::new();
    io::stdin().read_line(&mut username)?;
    let username = username.trim().to_string();

    print!("Password: ");
    io::stdout().flush()?;
    let password = rpassword::read_password()?;

    Ok((username, password))
}

/// Interactive registration prompt
pub fn prompt_register() -> Result<(String, String, String), Box<dyn std::error::Error>> {
    print!("Username: ");
    io::stdout().flush()?;
    let mut username = String::new();
    io::stdin().read_line(&mut username)?;
    let username = username.trim().to_string();

    print!("Email: ");
    io::stdout().flush()?;
    let mut email = String::new();
    io::stdin().read_line(&mut email)?;
    let email = email.trim().to_string();

    print!("Password: ");
    io::stdout().flush()?;
    let password = rpassword::read_password()?;

    print!("Confirm password: ");
    io::stdout().flush()?;
    let confirm_password = rpassword::read_password()?;

    if password != confirm_password {
        return Err("Passwords do not match".into());
    }

    Ok((username, email, password))
}
