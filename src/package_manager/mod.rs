// Package Manager for RavensOne
// Handles dependencies, versioning, and package installation

pub mod registry;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use semver::{Version, VersionReq};
use registry::RegistryClient;

/// Package manifest (raven.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManifest {
    pub package: PackageInfo,
    #[serde(default)]
    pub dependencies: HashMap<String, DependencySpec>,
    #[serde(default, rename = "dev-dependencies")]
    pub dev_dependencies: HashMap<String, DependencySpec>,
    #[serde(default)]
    pub build: BuildConfig,
    #[serde(default)]
    pub features: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub authors: Vec<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub repository: String,
    #[serde(default)]
    pub homepage: String,
    #[serde(default)]
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DependencySpec {
    /// Simple version string: "1.0.0" or "^1.0.0"
    Simple(String),
    /// Detailed dependency with features
    Detailed {
        version: String,
        #[serde(default)]
        features: Vec<String>,
        #[serde(default)]
        optional: bool,
        #[serde(default)]
        git: Option<String>,
        #[serde(default)]
        branch: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildConfig {
    #[serde(default = "default_target")]
    pub target: String,
    #[serde(default)]
    pub optimize: bool,
    #[serde(default)]
    pub ssr: bool,
    #[serde(default)]
    pub hydrate: bool,
}

fn default_target() -> String {
    "wasm32-unknown-unknown".to_string()
}

/// Lock file (raven.lock)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockFile {
    pub version: String,
    pub packages: Vec<LockedPackage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedPackage {
    pub name: String,
    pub version: String,
    pub source: PackageSource,
    #[serde(default)]
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PackageSource {
    Registry { url: String },
    Git { url: String, rev: String },
    Path { path: String },
}

/// Package Manager
pub struct PackageManager {
    manifest_path: PathBuf,
    lock_path: PathBuf,
    packages_dir: PathBuf,
    registry: RegistryClient,
}

impl PackageManager {
    pub fn new(project_root: &Path) -> Self {
        let mut registry = RegistryClient::new();
        let _ = registry.load_credentials(); // Load saved credentials if available

        PackageManager {
            manifest_path: project_root.join("raven.toml"),
            lock_path: project_root.join("raven.lock"),
            packages_dir: project_root.join("raven_packages"),
            registry,
        }
    }

    /// Initialize a new package manifest
    pub fn init(&self, name: &str, authors: Vec<String>) -> Result<(), PackageError> {
        if self.manifest_path.exists() {
            return Err(PackageError::ManifestExists);
        }

        let manifest = PackageManifest {
            package: PackageInfo {
                name: name.to_string(),
                version: "0.1.0".to_string(),
                authors,
                description: String::new(),
                license: "MIT".to_string(),
                repository: String::new(),
                homepage: String::new(),
                keywords: vec![],
            },
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            build: BuildConfig::default(),
            features: HashMap::new(),
        };

        let toml = toml::to_string_pretty(&manifest)
            .map_err(|e| PackageError::SerializationError(e.to_string()))?;

        fs::write(&self.manifest_path, toml)
            .map_err(|e| PackageError::IoError(e.to_string()))?;

        println!("✅ Created raven.toml for package '{}'", name);
        Ok(())
    }

    /// Load the package manifest
    pub fn load_manifest(&self) -> Result<PackageManifest, PackageError> {
        if !self.manifest_path.exists() {
            return Err(PackageError::ManifestNotFound);
        }

        let content = fs::read_to_string(&self.manifest_path)
            .map_err(|e| PackageError::IoError(e.to_string()))?;

        let manifest: PackageManifest = toml::from_str(&content)
            .map_err(|e| PackageError::ParseError(e.to_string()))?;

        Ok(manifest)
    }

    /// Install all dependencies
    pub fn install(&self) -> Result<(), PackageError> {
        let manifest = self.load_manifest()?;

        println!("📦 Resolving dependencies...");

        let resolved = self.resolve_dependencies(&manifest)?;

        println!("📥 Installing {} packages...", resolved.len());

        // Create packages directory
        fs::create_dir_all(&self.packages_dir)
            .map_err(|e| PackageError::IoError(e.to_string()))?;

        // Install each package
        for package in &resolved {
            self.install_package(package)?;
        }

        // Write lock file
        self.write_lock_file(&resolved)?;

        println!("✅ All dependencies installed!");
        Ok(())
    }

    /// Add a new dependency
    pub fn add_dependency(
        &self,
        name: &str,
        version_req: &str,
        dev: bool,
    ) -> Result<(), PackageError> {
        let mut manifest = self.load_manifest()?;

        let dep_spec = DependencySpec::Simple(version_req.to_string());

        if dev {
            manifest.dev_dependencies.insert(name.to_string(), dep_spec);
        } else {
            manifest.dependencies.insert(name.to_string(), dep_spec);
        }

        // Write back
        let toml = toml::to_string_pretty(&manifest)
            .map_err(|e| PackageError::SerializationError(e.to_string()))?;

        fs::write(&self.manifest_path, toml)
            .map_err(|e| PackageError::IoError(e.to_string()))?;

        println!("✅ Added {} @ {} to {}",
                 name, version_req,
                 if dev { "dev-dependencies" } else { "dependencies" });

        // Install the new dependency
        self.install()?;

        Ok(())
    }

    /// Remove a dependency
    pub fn remove_dependency(&self, name: &str) -> Result<(), PackageError> {
        let mut manifest = self.load_manifest()?;

        let removed = manifest.dependencies.remove(name).is_some()
            || manifest.dev_dependencies.remove(name).is_some();

        if !removed {
            return Err(PackageError::DependencyNotFound(name.to_string()));
        }

        // Write back
        let toml = toml::to_string_pretty(&manifest)
            .map_err(|e| PackageError::SerializationError(e.to_string()))?;

        fs::write(&self.manifest_path, toml)
            .map_err(|e| PackageError::IoError(e.to_string()))?;

        println!("✅ Removed {}", name);
        Ok(())
    }

    /// Resolve dependencies
    fn resolve_dependencies(
        &self,
        manifest: &PackageManifest,
    ) -> Result<Vec<LockedPackage>, PackageError> {
        let mut resolved = Vec::new();
        let mut visited = HashMap::new();

        // Combine dependencies and dev-dependencies
        let mut all_deps = manifest.dependencies.clone();
        all_deps.extend(manifest.dev_dependencies.clone());

        for (name, spec) in all_deps {
            self.resolve_recursive(&name, &spec, &mut resolved, &mut visited)?;
        }

        Ok(resolved)
    }

    /// Recursively resolve dependencies
    fn resolve_recursive(
        &self,
        name: &str,
        spec: &DependencySpec,
        resolved: &mut Vec<LockedPackage>,
        visited: &mut HashMap<String, String>,
    ) -> Result<(), PackageError> {
        // Check if already visited
        if visited.contains_key(name) {
            return Ok(());
        }

        let version_req = match spec {
            DependencySpec::Simple(v) => v.clone(),
            DependencySpec::Detailed { version, .. } => version.clone(),
        };

        // Find compatible version
        let version = self.find_compatible_version(name, &version_req)?;

        visited.insert(name.to_string(), version.clone());

        // Add to resolved
        resolved.push(LockedPackage {
            name: name.to_string(),
            version: version.clone(),
            source: PackageSource::Registry {
                url: format!("https://packages.ravensone.dev/{}/{}", name, version),
            },
            dependencies: vec![],
        });

        Ok(())
    }

    /// Find a compatible version for a package
    fn find_compatible_version(
        &self,
        name: &str,
        version_req: &str,
    ) -> Result<String, PackageError> {
        // Parse version requirement
        let req = VersionReq::parse(version_req)
            .map_err(|e| PackageError::InvalidVersion(e.to_string()))?;

        // For now, return a mock version that satisfies the requirement
        // In production, this would query the package registry
        let mock_versions = vec!["1.0.0", "1.1.0", "1.2.0", "2.0.0"];

        for v_str in mock_versions.iter().rev() {
            if let Ok(version) = Version::parse(v_str) {
                if req.matches(&version) {
                    return Ok(v_str.to_string());
                }
            }
        }

        Err(PackageError::NoCompatibleVersion(
            name.to_string(),
            version_req.to_string(),
        ))
    }

    /// Install a single package
    fn install_package(&self, package: &LockedPackage) -> Result<(), PackageError> {
        let package_dir = self.packages_dir.join(&package.name);

        println!("  📥 Installing {} @ {}", package.name, package.version);

        // Create package directory
        fs::create_dir_all(&package_dir)
            .map_err(|e| PackageError::IoError(e.to_string()))?;

        // In production, this would:
        // 1. Download package from registry
        // 2. Verify checksums
        // 3. Extract to package_dir

        // For now, create a placeholder
        let placeholder = format!(
            "# {} v{}\nInstalled from registry",
            package.name, package.version
        );

        fs::write(package_dir.join("package.info"), placeholder)
            .map_err(|e| PackageError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Write lock file
    fn write_lock_file(&self, packages: &[LockedPackage]) -> Result<(), PackageError> {
        let lock_file = LockFile {
            version: "1".to_string(),
            packages: packages.to_vec(),
        };

        let toml = toml::to_string_pretty(&lock_file)
            .map_err(|e| PackageError::SerializationError(e.to_string()))?;

        fs::write(&self.lock_path, toml)
            .map_err(|e| PackageError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Update dependencies to latest compatible versions
    pub fn update(&self) -> Result<(), PackageError> {
        println!("🔄 Updating dependencies...");

        // Remove lock file to force resolution
        if self.lock_path.exists() {
            fs::remove_file(&self.lock_path)
                .map_err(|e| PackageError::IoError(e.to_string()))?;
        }

        self.install()?;

        println!("✅ Dependencies updated!");
        Ok(())
    }

    /// Login to the package registry
    pub fn login(&mut self) -> Result<(), PackageError> {
        self.registry
            .login()
            .map_err(|e| PackageError::RegistryError(e.to_string()))?;
        Ok(())
    }

    /// Register a new account on the package registry
    pub fn register(&mut self) -> Result<(), PackageError> {
        self.registry
            .register()
            .map_err(|e| PackageError::RegistryError(e.to_string()))?;
        Ok(())
    }

    /// Publish the current package to the registry
    pub fn publish(&self) -> Result<(), PackageError> {
        // Get package directory (parent of manifest)
        let package_dir = self
            .manifest_path
            .parent()
            .ok_or_else(|| PackageError::IoError("Invalid manifest path".to_string()))?;

        self.registry
            .publish(package_dir)
            .map_err(|e| PackageError::RegistryError(e.to_string()))?;

        Ok(())
    }

    /// Search for packages in the registry
    pub fn search(&self, query: &str) -> Result<(), PackageError> {
        let results = self
            .registry
            .search(query, 20)
            .map_err(|e| PackageError::RegistryError(e.to_string()))?;

        if results.results.is_empty() {
            println!("No packages found matching '{}'", query);
            return Ok(());
        }

        println!("Found {} packages:\n", results.total);

        for result in results.results {
            println!("📦 {} @ {}", result.name, result.version);
            println!("   {}", result.description);
            if !result.keywords.is_empty() {
                println!("   Keywords: {}", result.keywords.join(", "));
            }
            println!("   Downloads: {} | Score: {:.2}", result.downloads, result.score);
            println!();
        }

        Ok(())
    }

    /// Install a package from the registry
    fn install_package_from_registry(
        &self,
        name: &str,
        version: &str,
    ) -> Result<(), PackageError> {
        let package_dir = self.packages_dir.join(name);

        self.registry
            .download(name, version, &package_dir)
            .map_err(|e| PackageError::RegistryError(e.to_string()))?;

        Ok(())
    }
}

/// Package Manager Errors
#[derive(Debug)]
pub enum PackageError {
    ManifestNotFound,
    ManifestExists,
    IoError(String),
    ParseError(String),
    SerializationError(String),
    DependencyNotFound(String),
    InvalidVersion(String),
    NoCompatibleVersion(String, String),
    CircularDependency(String),
    RegistryError(String),
}

impl std::fmt::Display for PackageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageError::ManifestNotFound => write!(f, "raven.toml not found"),
            PackageError::ManifestExists => write!(f, "raven.toml already exists"),
            PackageError::IoError(e) => write!(f, "IO error: {}", e),
            PackageError::ParseError(e) => write!(f, "Parse error: {}", e),
            PackageError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            PackageError::DependencyNotFound(name) => write!(f, "Dependency '{}' not found", name),
            PackageError::InvalidVersion(e) => write!(f, "Invalid version: {}", e),
            PackageError::NoCompatibleVersion(name, req) => {
                write!(f, "No compatible version found for {} @ {}", name, req)
            }
            PackageError::CircularDependency(name) => {
                write!(f, "Circular dependency detected: {}", name)
            }
            PackageError::RegistryError(e) => write!(f, "Registry error: {}", e),
        }
    }
}

impl std::error::Error for PackageError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_manifest_serialization() {
        let manifest = PackageManifest {
            package: PackageInfo {
                name: "test-package".to_string(),
                version: "0.1.0".to_string(),
                authors: vec!["Test Author".to_string()],
                description: "A test package".to_string(),
                license: "MIT".to_string(),
                repository: String::new(),
                homepage: String::new(),
                keywords: vec![],
            },
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            build: BuildConfig::default(),
            features: HashMap::new(),
        };

        let toml = toml::to_string(&manifest).unwrap();
        assert!(toml.contains("test-package"));
        assert!(toml.contains("0.1.0"));
    }

    #[test]
    fn test_version_parsing() {
        let req = VersionReq::parse("^1.0.0").unwrap();
        let v1 = Version::parse("1.0.0").unwrap();
        let v2 = Version::parse("1.1.0").unwrap();
        let v3 = Version::parse("2.0.0").unwrap();

        assert!(req.matches(&v1));
        assert!(req.matches(&v2));
        assert!(!req.matches(&v3));
    }

    #[test]
    fn test_dependency_spec_simple() {
        let spec = DependencySpec::Simple("^1.0.0".to_string());
        let json = serde_json::to_string(&spec).unwrap();
        assert_eq!(json, "\"^1.0.0\"");
    }

    #[test]
    fn test_lock_file_structure() {
        let lock = LockFile {
            version: "1".to_string(),
            packages: vec![LockedPackage {
                name: "test-pkg".to_string(),
                version: "1.0.0".to_string(),
                source: PackageSource::Registry {
                    url: "https://example.com".to_string(),
                },
                dependencies: vec![],
            }],
        };

        let toml = toml::to_string(&lock).unwrap();
        assert!(toml.contains("test-pkg"));
        assert!(toml.contains("1.0.0"));
    }
}
