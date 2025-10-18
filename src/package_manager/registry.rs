// Registry Client for RavensOne Package Manager
// Handles authentication, publishing, and downloading packages from the registry

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Registry client configuration
pub struct RegistryClient {
    base_url: String,
    token: Option<String>,
    credentials_path: PathBuf,
}

impl RegistryClient {
    /// Create a new registry client
    pub fn new() -> Self {
        let home = dirs::home_dir().expect("Could not find home directory");
        let raven_dir = home.join(".raven");

        RegistryClient {
            base_url: "https://registry.ravensone.dev/api/v1".to_string(),
            token: None,
            credentials_path: raven_dir.join("credentials.json"),
        }
    }

    /// Set a custom registry URL (for development)
    pub fn with_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }

    /// Load saved credentials
    pub fn load_credentials(&mut self) -> Result<(), RegistryError> {
        if !self.credentials_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&self.credentials_path)
            .map_err(|e| RegistryError::IoError(e.to_string()))?;

        let creds: SavedCredentials = serde_json::from_str(&content)
            .map_err(|e| RegistryError::ParseError(e.to_string()))?;

        self.token = Some(creds.token);
        Ok(())
    }

    /// Save credentials to disk
    fn save_credentials(&self, token: &str, username: &str) -> Result<(), RegistryError> {
        // Create .raven directory if it doesn't exist
        if let Some(parent) = self.credentials_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| RegistryError::IoError(e.to_string()))?;
        }

        let creds = SavedCredentials {
            token: token.to_string(),
            username: username.to_string(),
        };

        let json = serde_json::to_string_pretty(&creds)
            .map_err(|e| RegistryError::SerializationError(e.to_string()))?;

        fs::write(&self.credentials_path, json)
            .map_err(|e| RegistryError::IoError(e.to_string()))?;

        // Set file permissions to user-only on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&self.credentials_path)
                .map_err(|e| RegistryError::IoError(e.to_string()))?
                .permissions();
            perms.set_mode(0o600); // rw-------
            fs::set_permissions(&self.credentials_path, perms)
                .map_err(|e| RegistryError::IoError(e.to_string()))?;
        }

        Ok(())
    }

    /// Login to the registry
    pub fn login(&mut self) -> Result<LoginResponse, RegistryError> {
        println!("🔐 RavensOne Package Registry Login");
        println!();

        // Prompt for email
        print!("Email: ");
        std::io::stdout().flush().unwrap();
        let mut email = String::new();
        std::io::stdin()
            .read_line(&mut email)
            .map_err(|e| RegistryError::IoError(e.to_string()))?;
        let email = email.trim();

        // Prompt for password (silently)
        print!("Password: ");
        std::io::stdout().flush().unwrap();
        let password = rpassword::read_password()
            .map_err(|e| RegistryError::IoError(e.to_string()))?;

        // Make API request
        let client = reqwest::blocking::Client::new();
        let url = format!("{}/auth/login", self.base_url);

        let request_body = LoginRequest {
            email: email.to_string(),
            password,
        };

        let response = client
            .post(&url)
            .json(&request_body)
            .send()
            .map_err(|e| RegistryError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().unwrap_or_else(|_| "Unknown error".to_string());
            return Err(RegistryError::AuthenticationFailed(error_text));
        }

        let login_response: LoginResponse = response
            .json()
            .map_err(|e| RegistryError::ParseError(e.to_string()))?;

        // Save credentials
        self.save_credentials(&login_response.token, &login_response.user.username)?;
        self.token = Some(login_response.token.clone());

        println!();
        println!("✅ Successfully logged in as {}", login_response.user.username);
        println!("📝 Credentials saved to ~/.raven/credentials.json");

        Ok(login_response)
    }

    /// Register a new user account
    pub fn register(&mut self) -> Result<RegisterResponse, RegistryError> {
        println!("📝 Create a RavensOne Account");
        println!();

        // Prompt for username
        print!("Username: ");
        std::io::stdout().flush().unwrap();
        let mut username = String::new();
        std::io::stdin()
            .read_line(&mut username)
            .map_err(|e| RegistryError::IoError(e.to_string()))?;
        let username = username.trim();

        // Prompt for email
        print!("Email: ");
        std::io::stdout().flush().unwrap();
        let mut email = String::new();
        std::io::stdin()
            .read_line(&mut email)
            .map_err(|e| RegistryError::IoError(e.to_string()))?;
        let email = email.trim();

        // Prompt for password
        print!("Password: ");
        std::io::stdout().flush().unwrap();
        let password = rpassword::read_password()
            .map_err(|e| RegistryError::IoError(e.to_string()))?;

        // Make API request
        let client = reqwest::blocking::Client::new();
        let url = format!("{}/auth/register", self.base_url);

        let request_body = RegisterRequest {
            username: username.to_string(),
            email: email.to_string(),
            password,
        };

        let response = client
            .post(&url)
            .json(&request_body)
            .send()
            .map_err(|e| RegistryError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().unwrap_or_else(|_| "Unknown error".to_string());
            return Err(RegistryError::RegistrationFailed(error_text));
        }

        let register_response: RegisterResponse = response
            .json()
            .map_err(|e| RegistryError::ParseError(e.to_string()))?;

        // Save credentials
        self.save_credentials(&register_response.token, &register_response.username)?;
        self.token = Some(register_response.token.clone());

        println!();
        println!("✅ Account created successfully!");
        println!("👤 Username: {}", register_response.username);
        println!("📝 Credentials saved to ~/.raven/credentials.json");

        Ok(register_response)
    }

    /// Publish a package to the registry
    pub fn publish(&self, package_dir: &Path) -> Result<PublishResponse, RegistryError> {
        // Ensure user is logged in
        let token = self.token.as_ref().ok_or(RegistryError::NotAuthenticated)?;

        // Load package manifest
        let manifest_path = package_dir.join("raven.toml");
        if !manifest_path.exists() {
            return Err(RegistryError::ManifestNotFound);
        }

        let manifest_content = fs::read_to_string(&manifest_path)
            .map_err(|e| RegistryError::IoError(e.to_string()))?;

        let manifest: super::PackageManifest = toml::from_str(&manifest_content)
            .map_err(|e| RegistryError::ParseError(e.to_string()))?;

        println!("📦 Publishing {} v{}", manifest.package.name, manifest.package.version);

        // Create tarball
        println!("  📁 Creating package tarball...");
        let tarball_path = self.create_tarball(package_dir, &manifest)?;

        // Convert manifest to JSON for API
        let manifest_json = serde_json::to_string(&manifest)
            .map_err(|e| RegistryError::SerializationError(e.to_string()))?;

        // Upload to registry
        println!("  ⬆️  Uploading to registry...");
        let client = reqwest::blocking::Client::new();
        let url = format!("{}/packages/publish", self.base_url);

        let tarball_bytes = fs::read(&tarball_path)
            .map_err(|e| RegistryError::IoError(e.to_string()))?;

        let form = reqwest::blocking::multipart::Form::new()
            .text("manifest", manifest_json)
            .part(
                "tarball",
                reqwest::blocking::multipart::Part::bytes(tarball_bytes)
                    .file_name(format!("{}-{}.tar.gz", manifest.package.name, manifest.package.version)),
            );

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .multipart(form)
            .send()
            .map_err(|e| RegistryError::NetworkError(e.to_string()))?;

        // Clean up tarball
        let _ = fs::remove_file(tarball_path);

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().unwrap_or_else(|_| "Unknown error".to_string());
            return Err(RegistryError::PublishFailed(format!(
                "HTTP {}: {}",
                status, error_text
            )));
        }

        let publish_response: PublishResponse = response
            .json()
            .map_err(|e| RegistryError::ParseError(e.to_string()))?;

        println!();
        println!("✅ Package published successfully!");
        println!("📦 {} v{}", publish_response.name, publish_response.version);
        println!("🔗 {}", publish_response.download_url);

        Ok(publish_response)
    }

    /// Download a package from the registry
    pub fn download(
        &self,
        name: &str,
        version: &str,
        dest_dir: &Path,
    ) -> Result<(), RegistryError> {
        let url = format!("{}/packages/{}/{}/download", self.base_url, name, version);

        println!("  📥 Downloading {} v{}", name, version);

        let client = reqwest::blocking::Client::new();
        let mut response = client
            .get(&url)
            .send()
            .map_err(|e| RegistryError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(RegistryError::DownloadFailed(format!(
                "Package {} v{} not found",
                name, version
            )));
        }

        // Create destination directory
        fs::create_dir_all(dest_dir)
            .map_err(|e| RegistryError::IoError(e.to_string()))?;

        // Save tarball temporarily
        let tarball_path = dest_dir.join(format!("{}-{}.tar.gz", name, version));
        let mut tarball_file = fs::File::create(&tarball_path)
            .map_err(|e| RegistryError::IoError(e.to_string()))?;

        response
            .copy_to(&mut tarball_file)
            .map_err(|e| RegistryError::NetworkError(e.to_string()))?;

        // Extract tarball
        self.extract_tarball(&tarball_path, dest_dir)?;

        // Clean up tarball
        let _ = fs::remove_file(tarball_path);

        Ok(())
    }

    /// Get package metadata
    pub fn get_package_info(&self, name: &str) -> Result<PackageInfo, RegistryError> {
        let url = format!("{}/packages/{}", self.base_url, name);

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&url)
            .send()
            .map_err(|e| RegistryError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(RegistryError::PackageNotFound(name.to_string()));
        }

        let package_info: PackageInfo = response
            .json()
            .map_err(|e| RegistryError::ParseError(e.to_string()))?;

        Ok(package_info)
    }

    /// Search for packages
    pub fn search(&self, query: &str, limit: u32) -> Result<SearchResponse, RegistryError> {
        let url = format!(
            "{}/search?q={}&limit={}",
            self.base_url,
            urlencoding::encode(query),
            limit
        );

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&url)
            .send()
            .map_err(|e| RegistryError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(RegistryError::SearchFailed("Search request failed".to_string()));
        }

        let search_response: SearchResponse = response
            .json()
            .map_err(|e| RegistryError::ParseError(e.to_string()))?;

        Ok(search_response)
    }

    /// Create a tarball from a package directory
    fn create_tarball(
        &self,
        package_dir: &Path,
        manifest: &super::PackageManifest,
    ) -> Result<PathBuf, RegistryError> {
        let tarball_name = format!("{}-{}.tar.gz", manifest.package.name, manifest.package.version);
        let tarball_path = std::env::temp_dir().join(&tarball_name);

        let tarball_file = fs::File::create(&tarball_path)
            .map_err(|e| RegistryError::IoError(e.to_string()))?;

        let encoder = flate2::write::GzEncoder::new(tarball_file, flate2::Compression::default());
        let mut archive = tar::Builder::new(encoder);

        // Add all files from src/ directory
        let src_dir = package_dir.join("src");
        if src_dir.exists() {
            archive
                .append_dir_all("src", &src_dir)
                .map_err(|e| RegistryError::IoError(e.to_string()))?;
        }

        // Add manifest
        archive
            .append_path_with_name(&package_dir.join("raven.toml"), "raven.toml")
            .map_err(|e| RegistryError::IoError(e.to_string()))?;

        // Add README if exists
        let readme_path = package_dir.join("README.md");
        if readme_path.exists() {
            archive
                .append_path_with_name(&readme_path, "README.md")
                .map_err(|e| RegistryError::IoError(e.to_string()))?;
        }

        archive
            .finish()
            .map_err(|e| RegistryError::IoError(e.to_string()))?;

        Ok(tarball_path)
    }

    /// Extract a tarball to a destination directory
    fn extract_tarball(&self, tarball_path: &Path, dest_dir: &Path) -> Result<(), RegistryError> {
        let tarball_file = fs::File::open(tarball_path)
            .map_err(|e| RegistryError::IoError(e.to_string()))?;

        let decoder = flate2::read::GzDecoder::new(tarball_file);
        let mut archive = tar::Archive::new(decoder);

        archive
            .unpack(dest_dir)
            .map_err(|e| RegistryError::IoError(e.to_string()))?;

        Ok(())
    }
}

/// Saved credentials
#[derive(Debug, Serialize, Deserialize)]
struct SavedCredentials {
    token: String,
    username: String,
}

/// Login request
#[derive(Debug, Serialize)]
struct LoginRequest {
    email: String,
    password: String,
}

/// Login response
#[derive(Debug, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub expires_at: String,
    pub user: UserInfo,
}

/// User info
#[derive(Debug, Deserialize)]
pub struct UserInfo {
    pub user_id: String,
    pub username: String,
    pub email: String,
}

/// Register request
#[derive(Debug, Serialize)]
struct RegisterRequest {
    username: String,
    email: String,
    password: String,
}

/// Register response
#[derive(Debug, Deserialize)]
pub struct RegisterResponse {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub created_at: String,
    pub token: String,
}

/// Publish response
#[derive(Debug, Deserialize)]
pub struct PublishResponse {
    pub package_id: String,
    pub name: String,
    pub version: String,
    pub published_at: String,
    pub download_url: String,
    pub checksum: String,
}

/// Package info from registry
#[derive(Debug, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub description: String,
    pub latest_version: String,
    pub versions: Vec<String>,
    pub owner: OwnerInfo,
    pub license: String,
    pub repository: String,
    pub homepage: String,
    pub keywords: Vec<String>,
    pub downloads_total: u64,
    pub downloads_last_month: u64,
    pub created_at: String,
    pub updated_at: String,
}

/// Owner info
#[derive(Debug, Deserialize)]
pub struct OwnerInfo {
    pub username: String,
    pub user_id: String,
}

/// Search response
#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total: u32,
    pub limit: u32,
    pub offset: u32,
}

/// Search result
#[derive(Debug, Deserialize)]
pub struct SearchResult {
    pub name: String,
    pub version: String,
    pub description: String,
    pub keywords: Vec<String>,
    pub downloads: u64,
    pub score: f64,
}

/// Registry errors
#[derive(Debug)]
pub enum RegistryError {
    NotAuthenticated,
    AuthenticationFailed(String),
    RegistrationFailed(String),
    ManifestNotFound,
    IoError(String),
    NetworkError(String),
    ParseError(String),
    SerializationError(String),
    PublishFailed(String),
    DownloadFailed(String),
    PackageNotFound(String),
    SearchFailed(String),
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryError::NotAuthenticated => {
                write!(f, "Not authenticated. Run 'raven pkg login' first")
            }
            RegistryError::AuthenticationFailed(msg) => write!(f, "Login failed: {}", msg),
            RegistryError::RegistrationFailed(msg) => write!(f, "Registration failed: {}", msg),
            RegistryError::ManifestNotFound => write!(f, "raven.toml not found"),
            RegistryError::IoError(e) => write!(f, "IO error: {}", e),
            RegistryError::NetworkError(e) => write!(f, "Network error: {}", e),
            RegistryError::ParseError(e) => write!(f, "Parse error: {}", e),
            RegistryError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            RegistryError::PublishFailed(msg) => write!(f, "Publish failed: {}", msg),
            RegistryError::DownloadFailed(msg) => write!(f, "Download failed: {}", msg),
            RegistryError::PackageNotFound(name) => write!(f, "Package '{}' not found", name),
            RegistryError::SearchFailed(msg) => write!(f, "Search failed: {}", msg),
        }
    }
}

impl std::error::Error for RegistryError {}
