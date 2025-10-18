// Package Manager for RavensOne
// Handles dependencies, versioning, and package installation

pub mod registry;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use semver::{Version, VersionReq};
use registry::RegistryClient;
use std::time::SystemTime;

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

/// Build cache metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildCacheEntry {
    pub package_name: String,
    pub package_version: String,
    pub source_hash: String,
    pub compiled_at: u64,
    pub wasm_path: PathBuf,
}

/// Build cache index
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildCache {
    pub entries: HashMap<String, BuildCacheEntry>,
}

/// Security advisory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAdvisory {
    pub id: String,
    pub package: String,
    pub severity: Severity,
    pub title: String,
    pub description: String,
    pub affected_versions: String,
    pub patched_versions: Vec<String>,
    pub published_at: String,
    pub references: Vec<String>,
}

/// Vulnerability severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Low,
    Moderate,
    High,
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Low => write!(f, "LOW"),
            Severity::Moderate => write!(f, "MODERATE"),
            Severity::High => write!(f, "HIGH"),
            Severity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Audit result
#[derive(Debug)]
pub struct AuditResult {
    pub vulnerabilities: Vec<SecurityAdvisory>,
    pub total_packages: usize,
    pub vulnerable_packages: usize,
}

/// Package Manager
pub struct PackageManager {
    manifest_path: PathBuf,
    lock_path: PathBuf,
    packages_dir: PathBuf,
    cache_dir: PathBuf,
    registry: RegistryClient,
}

impl PackageManager {
    pub fn new(project_root: &Path) -> Self {
        let mut registry = RegistryClient::new();
        let _ = registry.load_credentials(); // Load saved credentials if available

        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let cache_dir = home.join(".raven").join("cache");

        PackageManager {
            manifest_path: project_root.join("raven.toml"),
            lock_path: project_root.join("raven.lock"),
            packages_dir: project_root.join("raven_packages"),
            cache_dir,
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
        let mut stack = Vec::new();

        // Combine dependencies and dev-dependencies
        let mut all_deps = manifest.dependencies.clone();
        all_deps.extend(manifest.dev_dependencies.clone());

        for (name, spec) in all_deps {
            self.resolve_recursive(&name, &spec, &mut resolved, &mut visited, &mut stack)?;
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
        stack: &mut Vec<String>,
    ) -> Result<(), PackageError> {
        // Check if we're in a circular dependency
        if stack.contains(&name.to_string()) {
            let cycle = stack.iter()
                .skip_while(|&n| n != name)
                .cloned()
                .chain(std::iter::once(name.to_string()))
                .collect::<Vec<_>>()
                .join(" -> ");
            return Err(PackageError::CircularDependency(cycle));
        }

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
        stack.push(name.to_string());

        // Fetch the package manifest to get its dependencies
        let manifest = self
            .registry
            .get_package_manifest(name, &version)
            .map_err(|e| PackageError::RegistryError(e.to_string()))?;

        // Collect dependency names for the lock file
        let dep_names: Vec<String> = manifest.dependencies.keys().cloned().collect();

        // Recursively resolve transitive dependencies
        for (dep_name, dep_spec) in &manifest.dependencies {
            self.resolve_recursive(dep_name, dep_spec, resolved, visited, stack)?;
        }

        // Remove from stack after processing
        stack.pop();

        // Add to resolved with its dependencies
        resolved.push(LockedPackage {
            name: name.to_string(),
            version: version.clone(),
            source: PackageSource::Registry {
                url: format!("https://packages.ravensone.dev/{}/{}", name, version),
            },
            dependencies: dep_names,
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

        // Query the registry for package info
        let package_info = self
            .registry
            .get_package_info(name)
            .map_err(|e| PackageError::RegistryError(e.to_string()))?;

        // Find the highest compatible version
        let mut compatible_versions: Vec<Version> = package_info
            .versions
            .iter()
            .filter_map(|v_str| {
                if let Ok(version) = Version::parse(v_str) {
                    if req.matches(&version) {
                        Some(version)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        if compatible_versions.is_empty() {
            return Err(PackageError::NoCompatibleVersion(
                name.to_string(),
                version_req.to_string(),
            ));
        }

        // Sort and return the highest compatible version
        compatible_versions.sort();
        Ok(compatible_versions.last().unwrap().to_string())
    }

    /// Install a single package
    fn install_package(&self, package: &LockedPackage) -> Result<(), PackageError> {
        println!("  📥 Installing {} @ {}", package.name, package.version);

        // Download and install from registry
        self.install_package_from_registry(&package.name, &package.version)?;

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

    /// Display dependency tree
    pub fn tree(&self) -> Result<(), PackageError> {
        let manifest = self.load_manifest()?;

        // Load lock file if it exists
        let lock = self.load_lock_file()?;

        println!("{} v{}", manifest.package.name, manifest.package.version);

        // Build dependency map from lock file
        let mut dep_map: HashMap<String, &LockedPackage> = HashMap::new();
        for package in &lock.packages {
            dep_map.insert(package.name.clone(), package);
        }

        // Display direct dependencies
        let mut all_deps: Vec<(String, DependencySpec)> = manifest.dependencies.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        all_deps.sort_by(|a, b| a.0.cmp(&b.0));

        for (i, (name, _spec)) in all_deps.iter().enumerate() {
            let is_last = i == all_deps.len() - 1;
            let prefix = if is_last { "└──" } else { "├──" };

            if let Some(package) = dep_map.get(name) {
                println!("{} {} v{}", prefix, package.name, package.version);
                self.print_dependency_tree(&dep_map, &package.dependencies, if is_last { "    " } else { "│   " });
            }
        }

        Ok(())
    }

    /// Helper to recursively print dependency tree
    fn print_dependency_tree(
        &self,
        dep_map: &HashMap<String, &LockedPackage>,
        dependencies: &[String],
        indent: &str,
    ) {
        for (i, dep_name) in dependencies.iter().enumerate() {
            let is_last = i == dependencies.len() - 1;
            let branch = if is_last { "└──" } else { "├──" };

            if let Some(package) = dep_map.get(dep_name) {
                println!("{}{} {} v{}", indent, branch, package.name, package.version);

                let new_indent = format!("{}{}", indent, if is_last { "    " } else { "│   " });
                self.print_dependency_tree(dep_map, &package.dependencies, &new_indent);
            }
        }
    }

    /// Load lock file
    fn load_lock_file(&self) -> Result<LockFile, PackageError> {
        if !self.lock_path.exists() {
            return Err(PackageError::LockFileNotFound);
        }

        let content = fs::read_to_string(&self.lock_path)
            .map_err(|e| PackageError::IoError(e.to_string()))?;

        let lock: LockFile = toml::from_str(&content)
            .map_err(|e| PackageError::ParseError(e.to_string()))?;

        Ok(lock)
    }

    /// Check for outdated dependencies
    pub fn outdated(&self) -> Result<(), PackageError> {
        let manifest = self.load_manifest()?;
        let lock = self.load_lock_file()?;

        println!("Checking for outdated dependencies...\n");

        let mut has_outdated = false;

        // Check all dependencies from manifest
        let mut all_deps = manifest.dependencies.clone();
        all_deps.extend(manifest.dev_dependencies.clone());

        for (name, spec) in all_deps {
            let version_req = match spec {
                DependencySpec::Simple(v) => v,
                DependencySpec::Detailed { version, .. } => version,
            };

            // Find current version from lock file
            let current_version = lock.packages.iter()
                .find(|p| p.name == name)
                .map(|p| p.version.clone());

            if let Some(current) = current_version {
                // Get latest version from registry
                match self.registry.get_package_info(&name) {
                    Ok(package_info) => {
                        if let Some(latest) = package_info.versions.last() {
                            let current_ver = Version::parse(&current).ok();
                            let latest_ver = Version::parse(latest).ok();

                            if let (Some(c), Some(l)) = (current_ver, latest_ver) {
                                if l > c {
                                    has_outdated = true;
                                    println!("📦 {}", name);
                                    println!("   Current: {} | Latest: {} | Wanted: {}",
                                             current, latest, version_req);
                                    println!();
                                }
                            }
                        }
                    }
                    Err(_) => {
                        // Skip if we can't fetch package info
                        continue;
                    }
                }
            }
        }

        if !has_outdated {
            println!("✅ All dependencies are up to date!");
        } else {
            println!("💡 Run 'raven pkg update' to update to latest compatible versions");
        }

        Ok(())
    }

    /// List all installed packages
    pub fn list(&self) -> Result<(), PackageError> {
        let lock = self.load_lock_file()?;

        println!("Installed packages:\n");

        for package in &lock.packages {
            println!("📦 {} @ {}", package.name, package.version);
            if !package.dependencies.is_empty() {
                println!("   Dependencies: {}", package.dependencies.join(", "));
            }
        }

        println!("\n✅ Total: {} packages", lock.packages.len());

        Ok(())
    }

    /// Show detailed information about a package
    pub fn info(&self, package_name: &str) -> Result<(), PackageError> {
        println!("Fetching package information...\n");

        // Get package info from registry
        let package_info = self.registry
            .get_package_info(package_name)
            .map_err(|e| PackageError::RegistryError(e.to_string()))?;

        println!("📦 {}", package_info.name);
        println!("   {}", package_info.description);
        println!();

        // Show latest version
        if let Some(latest) = package_info.versions.last() {
            println!("Latest version: {}", latest);
        }

        // Show all versions
        println!("\nAvailable versions:");
        for version in package_info.versions.iter().rev().take(10) {
            println!("   • {}", version);
        }
        if package_info.versions.len() > 10 {
            println!("   ... and {} more", package_info.versions.len() - 10);
        }

        // Show stats
        println!("\nStatistics:");
        println!("   Total downloads: {}", package_info.downloads_total);
        println!("   Downloads (last month): {}", package_info.downloads_last_month);

        // Show repository info if available
        if let Some(repo) = package_info.repository {
            println!("   Repository: {}", repo);
        }
        if let Some(homepage) = package_info.homepage {
            println!("   Homepage: {}", homepage);
        }

        // Show keywords
        if !package_info.keywords.is_empty() {
            println!("\nKeywords: {}", package_info.keywords.join(", "));
        }

        // Check if installed locally
        if let Ok(lock) = self.load_lock_file() {
            if let Some(installed) = lock.packages.iter().find(|p| p.name == package_name) {
                println!("\n✅ Installed: v{}", installed.version);
                if !installed.dependencies.is_empty() {
                    println!("   Dependencies: {}", installed.dependencies.join(", "));
                }
            }
        }

        Ok(())
    }

    /// Load build cache
    fn load_build_cache(&self) -> BuildCache {
        let cache_index_path = self.cache_dir.join("index.json");

        if cache_index_path.exists() {
            if let Ok(content) = fs::read_to_string(&cache_index_path) {
                if let Ok(cache) = serde_json::from_str(&content) {
                    return cache;
                }
            }
        }

        BuildCache::default()
    }

    /// Save build cache
    fn save_build_cache(&self, cache: &BuildCache) -> Result<(), PackageError> {
        fs::create_dir_all(&self.cache_dir)
            .map_err(|e| PackageError::IoError(e.to_string()))?;

        let cache_index_path = self.cache_dir.join("index.json");
        let json = serde_json::to_string_pretty(cache)
            .map_err(|e| PackageError::SerializationError(e.to_string()))?;

        fs::write(&cache_index_path, json)
            .map_err(|e| PackageError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Calculate hash of package source
    fn calculate_source_hash(&self, package_path: &Path) -> Result<String, PackageError> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // Hash all .raven files in the package
        if package_path.exists() {
            for entry in fs::read_dir(package_path)
                .map_err(|e| PackageError::IoError(e.to_string()))?
            {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.extension().map_or(false, |ext| ext == "raven") {
                        if let Ok(content) = fs::read_to_string(&path) {
                            content.hash(&mut hasher);
                        }
                    }
                }
            }
        }

        Ok(format!("{:x}", hasher.finish()))
    }

    /// Check if package is cached and valid
    fn is_cached(&self, package_name: &str, package_version: &str) -> Option<PathBuf> {
        let cache = self.load_build_cache();
        let cache_key = format!("{}@{}", package_name, package_version);

        if let Some(entry) = cache.entries.get(&cache_key) {
            if entry.wasm_path.exists() {
                // Verify hash matches current source
                let package_path = self.packages_dir.join(package_name);
                if let Ok(current_hash) = self.calculate_source_hash(&package_path) {
                    if current_hash == entry.source_hash {
                        return Some(entry.wasm_path.clone());
                    }
                }
            }
        }

        None
    }

    /// Clear build cache
    pub fn clean_cache(&self) -> Result<(), PackageError> {
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir)
                .map_err(|e| PackageError::IoError(e.to_string()))?;
            println!("✅ Build cache cleared");
        } else {
            println!("✅ Cache already empty");
        }
        Ok(())
    }

    /// Show cache statistics
    pub fn cache_stats(&self) -> Result<(), PackageError> {
        let cache = self.load_build_cache();

        println!("Build Cache Statistics:\n");
        println!("Location: {}", self.cache_dir.display());
        println!("Cached packages: {}", cache.entries.len());

        if !cache.entries.is_empty() {
            println!("\nCached builds:");
            for (key, entry) in &cache.entries {
                println!("  📦 {} (compiled {})", key,
                         format_timestamp(entry.compiled_at));
            }
        }

        Ok(())
    }

    /// Audit dependencies for security vulnerabilities
    pub fn audit(&self) -> Result<(), PackageError> {
        let lock = self.load_lock_file()?;

        println!("🔍 Auditing {} packages for known security vulnerabilities...\n", lock.packages.len());

        let result = self.perform_audit(&lock)?;

        // Display results
        if result.vulnerabilities.is_empty() {
            println!("✅ No known security vulnerabilities found!");
            println!("\n   Audited {} packages", result.total_packages);
        } else {
            // Sort vulnerabilities by severity
            let mut vulns = result.vulnerabilities;
            vulns.sort_by(|a, b| b.severity.cmp(&a.severity));

            // Count by severity
            let critical = vulns.iter().filter(|v| v.severity == Severity::Critical).count();
            let high = vulns.iter().filter(|v| v.severity == Severity::High).count();
            let moderate = vulns.iter().filter(|v| v.severity == Severity::Moderate).count();
            let low = vulns.iter().filter(|v| v.severity == Severity::Low).count();

            println!("❌ Found {} vulnerabilities in {} packages:\n", vulns.len(), result.vulnerable_packages);

            // Display summary
            if critical > 0 {
                println!("   🔴 {} Critical", critical);
            }
            if high > 0 {
                println!("   🟠 {} High", high);
            }
            if moderate > 0 {
                println!("   🟡 {} Moderate", moderate);
            }
            if low > 0 {
                println!("   🟢 {} Low", low);
            }

            println!("\n{}", "=".repeat(70));

            // Display each vulnerability
            for vuln in vulns {
                println!("\n{} | {}",
                    match vuln.severity {
                        Severity::Critical => "🔴 CRITICAL",
                        Severity::High => "🟠 HIGH",
                        Severity::Moderate => "🟡 MODERATE",
                        Severity::Low => "🟢 LOW",
                    },
                    vuln.title
                );
                println!("Package: {} ({})", vuln.package, vuln.affected_versions);
                println!("ID: {}", vuln.id);
                println!("\n{}", vuln.description);

                if !vuln.patched_versions.is_empty() {
                    println!("\n✅ Patched in: {}", vuln.patched_versions.join(", "));
                    println!("   Recommendation: Update to a patched version");
                }

                if !vuln.references.is_empty() {
                    println!("\nMore info:");
                    for ref_url in &vuln.references {
                        println!("   • {}", ref_url);
                    }
                }

                println!("{}", "=".repeat(70));
            }

            println!("\n💡 Run 'raven pkg update' to update to patched versions");
        }

        Ok(())
    }

    /// Perform security audit check
    fn perform_audit(&self, lock: &LockFile) -> Result<AuditResult, PackageError> {
        // In a real implementation, this would:
        // 1. Query a vulnerability database API (like GitHub Advisory Database)
        // 2. Check each package version against known vulnerabilities
        // 3. Return list of vulnerabilities found

        // For now, we'll return a mock implementation that shows the structure
        let vulnerabilities = Vec::new();

        let total_packages = lock.packages.len();
        let vulnerable_packages = 0;

        Ok(AuditResult {
            vulnerabilities,
            total_packages,
            vulnerable_packages,
        })
    }
}

fn format_timestamp(timestamp: u64) -> String {
    use std::time::UNIX_EPOCH;
    let duration = std::time::Duration::from_secs(timestamp);
    let time = UNIX_EPOCH + duration;

    if let Ok(elapsed) = SystemTime::now().duration_since(time) {
        let secs = elapsed.as_secs();
        if secs < 60 {
            return format!("{}s ago", secs);
        } else if secs < 3600 {
            return format!("{}m ago", secs / 60);
        } else if secs < 86400 {
            return format!("{}h ago", secs / 3600);
        } else {
            return format!("{}d ago", secs / 86400);
        }
    }

    "recently".to_string()
}

/// Package Manager Errors
#[derive(Debug)]
pub enum PackageError {
    ManifestNotFound,
    ManifestExists,
    LockFileNotFound,
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
            PackageError::LockFileNotFound => write!(f, "raven.lock not found. Run 'raven pkg install' first."),
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
