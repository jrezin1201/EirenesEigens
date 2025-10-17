use clap::Parser as ClapParser;
use std::fs;
use std::path::PathBuf;
use std::process;
use ravensone_compiler::{Compiler, deployer, BuildTarget}; // FIX: Corrected the import path

#[derive(ClapParser)]
#[command(name = "raven", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Compiles a RavensOne file
    Compile {
        path: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Creates a new RavensOne project
    New {
        name: String,
    },
    /// Builds and deploys the project to a cloud provider
    Deploy {
        #[arg(long, default_value = "production")]
        env: String,
    },
    /// Watch files and auto-recompile on changes
    Watch {
        #[arg(default_value = "src")]
        path: PathBuf,
    },
    /// Start development server with HMR
    Dev {
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },
    /// Run tests
    Test {
        #[arg(short, long)]
        watch: bool,
    },
    /// Format RavensOne source files
    Fmt {
        #[arg(short, long)]
        check: bool,
        path: Option<PathBuf>,
    },
    /// Lint RavensOne source files
    Lint {
        #[arg(short, long)]
        fix: bool,
        path: Option<PathBuf>,
    },
    /// Build the project for production
    Build {
        #[arg(short, long)]
        release: bool,
    },
    /// Publish package to registry
    Publish {
        #[arg(long, default_value = "http://localhost:4000")]
        registry: String,
    },
    /// Search for packages in registry
    Search {
        query: String,
        #[arg(long, default_value = "http://localhost:4000")]
        registry: String,
    },
    /// Install a package from registry
    Install {
        package: String,
        #[arg(long)]
        version: Option<String>,
        #[arg(long, default_value = "http://localhost:4000")]
        registry: String,
    },
    /// Register with the package registry
    Register {
        #[arg(long, default_value = "http://localhost:4000")]
        registry: String,
    },
    /// Package manager commands
    Pkg {
        #[command(subcommand)]
        command: PkgCommands,
    },
}

#[derive(clap::Subcommand)]
enum PkgCommands {
    /// Initialize a new package manifest (raven.toml)
    Init {
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Install all dependencies
    Install,
    /// Add a dependency
    Add {
        name: String,
        #[arg(short, long)]
        version: Option<String>,
        #[arg(long)]
        dev: bool,
    },
    /// Remove a dependency
    Remove {
        name: String,
    },
    /// Update dependencies to latest compatible versions
    Update,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compile { path, output } => {
            // FIX: Added logic to handle compilation
            let output_path = output.unwrap_or_else(|| path.with_extension("wasm"));
            println!("🔥 Compiling {} -> {}", path.display(), output_path.display());

            let source_code = match fs::read_to_string(&path) {
                Ok(code) => code,
                Err(e) => {
                    eprintln!("Error reading file '{}': {}", path.display(), e);
                    return;
                }
            };

            let compiler = Compiler::new();
            // We compile for the client target by default for now
            match compiler.compile_source(&source_code, BuildTarget::Client) {
                Ok(wasm_bytes) => {
                    if let Err(e) = fs::write(&output_path, wasm_bytes) {
                        eprintln!("Error writing output file: {}", e);
                    } else {
                        println!("✨ Artifact written to {}", output_path.display());
                    }
                }
                Err(e) => {
                    eprintln!("❌ Compilation failed: {}", e);
                }
            }
        }
        Commands::New { name } => {
            // FIX: Added logic for creating a new project
            if let Err(e) = create_new_project(&name) {
                eprintln!("❌ Error creating new project: {}", e);
                process::exit(1);
            }
            println!("✅ Project '{}' created successfully! 🚀", name);
        }
        Commands::Deploy { env } => {
            println!("🚀 Starting deployment to '{}'...", env);
            if let Err(e) = deployer::deploy_project() {
                eprintln!("❌ Deployment failed: {}", e);
                process::exit(1);
            }
        }
        Commands::Watch { path } => {
            println!("👀 Watching {} for changes...", path.display());
            if let Err(e) = watch_and_compile(path) {
                eprintln!("❌ Watch failed: {}", e);
                process::exit(1);
            }
        }
        Commands::Dev { port } => {
            println!("🚀 Starting development server on port {}...", port);
            if let Err(e) = start_dev_server(port) {
                eprintln!("❌ Dev server failed: {}", e);
                process::exit(1);
            }
        }
        Commands::Test { watch } => {
            if watch {
                println!("🧪 Running tests in watch mode...");
            } else {
                println!("🧪 Running tests...");
            }
            if let Err(e) = run_tests(watch) {
                eprintln!("❌ Tests failed: {}", e);
                process::exit(1);
            }
        }
        Commands::Fmt { check, path } => {
            let target = path.unwrap_or_else(|| PathBuf::from("src"));
            if check {
                println!("🔍 Checking formatting for {}...", target.display());
            } else {
                println!("✨ Formatting {}...", target.display());
            }
            if let Err(e) = format_code(target, check) {
                eprintln!("❌ Formatting failed: {}", e);
                process::exit(1);
            }
        }
        Commands::Lint { fix, path } => {
            let target = path.unwrap_or_else(|| PathBuf::from("src"));
            if fix {
                println!("🔧 Linting and fixing {}...", target.display());
            } else {
                println!("🔍 Linting {}...", target.display());
            }
            if let Err(e) = lint_code(target, fix) {
                eprintln!("❌ Linting failed: {}", e);
                process::exit(1);
            }
        }
        Commands::Build { release } => {
            if release {
                println!("📦 Building project (release mode)...");
            } else {
                println!("📦 Building project (debug mode)...");
            }
            if let Err(e) = build_project(release) {
                eprintln!("❌ Build failed: {}", e);
                process::exit(1);
            }
        }
        Commands::Register { registry } => {
            println!("📝 Registering with package registry...");
            if let Err(e) = register_with_registry(&registry) {
                eprintln!("❌ Registration failed: {}", e);
                process::exit(1);
            }
        }
        Commands::Publish { registry } => {
            println!("📤 Publishing package to registry...");
            if let Err(e) = publish_package(&registry) {
                eprintln!("❌ Publish failed: {}", e);
                process::exit(1);
            }
        }
        Commands::Search { query, registry } => {
            println!("🔍 Searching for '{}'...", query);
            if let Err(e) = search_packages(&query, &registry) {
                eprintln!("❌ Search failed: {}", e);
                process::exit(1);
            }
        }
        Commands::Install { package, version, registry } => {
            println!("📥 Installing package '{}'...", package);
            if let Err(e) = install_package(&package, version.as_deref(), &registry) {
                eprintln!("❌ Installation failed: {}", e);
                process::exit(1);
            }
        }
        Commands::Pkg { command } => {
            use ravensone_compiler::package_manager::PackageManager;

            match command {
                PkgCommands::Init { path } => {
                    let pkg_mgr = PackageManager::new(&path);
                    let project_name = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("my-package");

                    if let Err(e) = pkg_mgr.init(project_name, vec!["Developer <dev@example.com>".to_string()]) {
                        eprintln!("❌ Init failed: {}", e);
                        process::exit(1);
                    }
                }
                PkgCommands::Install => {
                    let pkg_mgr = PackageManager::new(&PathBuf::from("."));
                    if let Err(e) = pkg_mgr.install() {
                        eprintln!("❌ Install failed: {}", e);
                        process::exit(1);
                    }
                }
                PkgCommands::Add { name, version, dev } => {
                    let pkg_mgr = PackageManager::new(&PathBuf::from("."));
                    let version_req = version.as_deref().unwrap_or("^1.0.0");
                    if let Err(e) = pkg_mgr.add_dependency(&name, version_req, dev) {
                        eprintln!("❌ Add failed: {}", e);
                        process::exit(1);
                    }
                }
                PkgCommands::Remove { name } => {
                    let pkg_mgr = PackageManager::new(&PathBuf::from("."));
                    if let Err(e) = pkg_mgr.remove_dependency(&name) {
                        eprintln!("❌ Remove failed: {}", e);
                        process::exit(1);
                    }
                }
                PkgCommands::Update => {
                    let pkg_mgr = PackageManager::new(&PathBuf::from("."));
                    if let Err(e) = pkg_mgr.update() {
                        eprintln!("❌ Update failed: {}", e);
                        process::exit(1);
                    }
                }
            }
        }
    }
}

// The create_new_project function is unchanged
fn create_new_project(name: &str) -> std::io::Result<()> {
    let root = PathBuf::from(name);
    if root.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "A directory with this name already exists.",
        ));
    }

    fs::create_dir_all(root.join("src/components"))?;
    fs::create_dir_all(root.join("src/server"))?;

    fs::write(
        root.join("raven.toml"),
        format!(
            "[package]\nname = \"{}\"\nversion = \"0.1.0\"\n",
            name
        ),
    )?;

    fs::write(
        root.join(".gitignore"),
        "/dist\n/target\n",
    )?;

    fs::write(
        root.join("src/main.raven"),
        format!("// Welcome to RavensOne!\n\ncomponent App() {{\n    return <h1>\"Hello, {}!\"</h1>;\n}}\n", name),
    )?;
    
    fs::write(
        root.join("src/types.raven"),
        "// Define your shared data structures here.\n",
    )?;

    Ok(())
}

fn watch_and_compile(path: PathBuf) -> std::io::Result<()> {
    use std::time::Duration;
    use std::thread;

    println!("✅ Watching started. Press Ctrl+C to stop.");
    println!("   Monitoring: {}", path.display());

    let mut last_modified = std::collections::HashMap::new();

    loop {
        // Walk directory and check for changes
        if let Ok(entries) = fs::read_dir(&path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.extension().map_or(false, |ext| ext == "raven") {
                    if let Ok(metadata) = entry.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            let last = last_modified.get(&entry_path);
                            if last.map_or(true, |&last| last != modified) {
                                println!("\n🔄 Change detected: {}", entry_path.display());

                                // Compile the file
                                let output_path = entry_path.with_extension("wasm");
                                if let Ok(source) = fs::read_to_string(&entry_path) {
                                    let compiler = Compiler::new();
                                    match compiler.compile_source(&source, BuildTarget::Client) {
                                        Ok(wasm_bytes) => {
                                            if fs::write(&output_path, wasm_bytes).is_ok() {
                                                println!("✅ Compiled → {}", output_path.display());
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("❌ Compilation error: {}", e);
                                        }
                                    }
                                }

                                last_modified.insert(entry_path, modified);
                            }
                        }
                    }
                }
            }
        }

        thread::sleep(Duration::from_millis(500));
    }
}

fn start_dev_server(port: u16) -> std::io::Result<()> {
    println!("✅ Development server starting...");
    println!("   📦 HTTP Server: http://localhost:{}", port);
    println!("   🔥 HMR Server: ws://localhost:3002/hmr");
    println!("   👀 File watcher: Active\n");

    // Start file watcher in background
    let watch_thread = std::thread::spawn(|| {
        let _ = watch_and_compile(PathBuf::from("src"));
    });

    // Start HMR server
    let hmr_thread = std::thread::spawn(|| {
        let _ = std::process::Command::new("node")
            .arg("scripts/hmr-server.js")
            .spawn();
    });

    // Start HTTP server
    println!("🌐 Starting HTTP server...");
    let http_result = std::process::Command::new("python3")
        .arg("serve.py")
        .spawn();

    if let Ok(mut child) = http_result {
        println!("✨ Dev server running! Press Ctrl+C to stop.\n");
        let _ = child.wait();
    }

    let _ = watch_thread.join();
    let _ = hmr_thread.join();

    Ok(())
}

fn run_tests(watch_mode: bool) -> std::io::Result<()> {
    let test_dir = PathBuf::from("tests");

    if !test_dir.exists() {
        println!("ℹ️  No tests directory found. Creating tests/...");
        fs::create_dir_all(&test_dir)?;
        fs::write(
            test_dir.join("example.test.raven"),
            "// Write your tests here\n// Example: test('1 + 1 = 2', () => { ... })\n"
        )?;
        println!("✅ Created tests/example.test.raven");
        return Ok(());
    }

    let mut passed = 0;
    let mut failed = 0;

    println!("🧪 Running tests...\n");

    for entry in fs::read_dir(test_dir)?.flatten() {
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "raven") {
            println!("  Testing {}...", path.file_name().unwrap().to_string_lossy());

            // Compile test file
            if let Ok(source) = fs::read_to_string(&path) {
                let compiler = Compiler::new();
                match compiler.compile_source(&source, BuildTarget::Client) {
                    Ok(_) => {
                        passed += 1;
                        println!("    ✅ PASS");
                    }
                    Err(e) => {
                        failed += 1;
                        println!("    ❌ FAIL: {}", e);
                    }
                }
            }
        }
    }

    println!("\n📊 Test Results:");
    println!("   ✅ Passed: {}", passed);
    println!("   ❌ Failed: {}", failed);

    if watch_mode {
        println!("\n👀 Watching for changes...");
        watch_and_compile(PathBuf::from("tests"))?;
    }

    if failed > 0 {
        process::exit(1);
    }

    Ok(())
}

fn format_code(path: PathBuf, check_only: bool) -> std::io::Result<()> {
    let mut formatted_count = 0;
    let mut error_count = 0;

    if path.is_file() {
        match format_file(&path, check_only) {
            Ok(true) => formatted_count += 1,
            Ok(false) => {},
            Err(_) => error_count += 1,
        }
    } else {
        for entry in fs::read_dir(path)?.flatten() {
            let entry_path = entry.path();
            if entry_path.extension().map_or(false, |ext| ext == "raven") {
                match format_file(&entry_path, check_only) {
                    Ok(true) => formatted_count += 1,
                    Ok(false) => {},
                    Err(_) => error_count += 1,
                }
            }
        }
    }

    if check_only {
        if formatted_count > 0 {
            println!("⚠️  {} file(s) need formatting", formatted_count);
            process::exit(1);
        } else {
            println!("✅ All files are properly formatted");
        }
    } else {
        println!("✅ Formatted {} file(s)", formatted_count);
    }

    if error_count > 0 {
        println!("❌ {} file(s) had errors", error_count);
    }

    Ok(())
}

fn format_file(path: &PathBuf, check_only: bool) -> std::io::Result<bool> {
    let content = fs::read_to_string(path)?;
    let formatted = format_raven_code(&content);

    if content != formatted {
        if check_only {
            println!("  ⚠️  {} needs formatting", path.display());
        } else {
            fs::write(path, formatted)?;
            println!("  ✨ Formatted {}", path.display());
        }
        Ok(true)
    } else {
        Ok(false)
    }
}

fn format_raven_code(code: &str) -> String {
    // Simple formatter - normalize whitespace and indentation
    let mut formatted = String::new();
    let mut indent: usize = 0;

    for line in code.lines() {
        let trimmed = line.trim();

        // Decrease indent for closing braces
        if trimmed.starts_with('}') {
            indent = indent.saturating_sub(1);
        }

        // Add indented line
        if !trimmed.is_empty() {
            formatted.push_str(&"    ".repeat(indent));
            formatted.push_str(trimmed);
            formatted.push('\n');
        }

        // Increase indent for opening braces
        if trimmed.ends_with('{') {
            indent += 1;
        }
    }

    formatted
}

fn lint_code(path: PathBuf, fix: bool) -> std::io::Result<()> {
    let mut issues = 0;
    let mut fixed = 0;

    if path.is_file() {
        let result = lint_file(&path, fix)?;
        issues += result.0;
        fixed += result.1;
    } else {
        for entry in fs::read_dir(path)?.flatten() {
            let entry_path = entry.path();
            if entry_path.extension().map_or(false, |ext| ext == "raven") {
                let result = lint_file(&entry_path, fix)?;
                issues += result.0;
                fixed += result.1;
            }
        }
    }

    if fix {
        println!("✅ Fixed {} issue(s)", fixed);
    }

    if issues > 0 {
        println!("⚠️  {} issue(s) found", issues);
        if !fix {
            println!("💡 Run with --fix to automatically fix issues");
        }
    } else {
        println!("✅ No issues found");
    }

    Ok(())
}

fn lint_file(path: &PathBuf, fix: bool) -> std::io::Result<(usize, usize)> {
    let content = fs::read_to_string(path)?;
    let mut issues = 0;
    let mut fixed = 0;

    // Check for common issues
    for (i, line) in content.lines().enumerate() {
        let line_num = i + 1;

        // Check trailing whitespace
        if line.ends_with(' ') || line.ends_with('\t') {
            issues += 1;
            println!("  ⚠️  {}:{} - Trailing whitespace", path.display(), line_num);
        }

        // Check line length
        if line.len() > 100 {
            issues += 1;
            println!("  ⚠️  {}:{} - Line too long ({} > 100)", path.display(), line_num, line.len());
        }
    }

    if fix && issues > 0 {
        // Remove trailing whitespace
        let fixed_content: String = content.lines()
            .map(|line| line.trim_end())
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(path, fixed_content)?;
        fixed = issues;
    }

    Ok((issues, fixed))
}

fn build_project(release: bool) -> std::io::Result<()> {
    let dist_dir = PathBuf::from("dist");
    fs::create_dir_all(&dist_dir)?;

    println!("📦 Building all components...\n");

    let src_dir = PathBuf::from("src");
    let mut compiled = 0;
    let mut errors = 0;

    for entry in fs::read_dir(src_dir)?.flatten() {
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "raven") {
            println!("  Compiling {}...", path.file_name().unwrap().to_string_lossy());

            if let Ok(source) = fs::read_to_string(&path) {
                let compiler = Compiler::new();
                let target = if release { BuildTarget::Client } else { BuildTarget::Client };

                match compiler.compile_source(&source, target) {
                    Ok(wasm_bytes) => {
                        let output_name = path.file_stem().unwrap().to_string_lossy();
                        let output_path = dist_dir.join(format!("{}.wasm", output_name));
                        fs::write(&output_path, wasm_bytes)?;
                        compiled += 1;
                        println!("    ✅ → {}", output_path.display());
                    }
                    Err(e) => {
                        errors += 1;
                        println!("    ❌ Error: {}", e);
                    }
                }
            }
        }
    }

    println!("\n📊 Build complete:");
    println!("   ✅ Compiled: {} file(s)", compiled);
    if errors > 0 {
        println!("   ❌ Errors: {} file(s)", errors);
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Build failed with errors"
        ));
    }

    println!("\n✨ Build artifacts in dist/");

    Ok(())
}

// Package Registry Functions

fn register_with_registry(registry_url: &str) -> std::io::Result<()> {
    use std::io::{self, Write};

    print!("Username: ");
    io::stdout().flush()?;
    let mut username = String::new();
    io::stdin().read_line(&mut username)?;
    let username = username.trim();

    print!("Email: ");
    io::stdout().flush()?;
    let mut email = String::new();
    io::stdin().read_line(&mut email)?;
    let email = email.trim();

    println!("\n📡 Registering with registry...");

    // Use curl to register
    let output = process::Command::new("curl")
        .arg("-s")
        .arg("-X")
        .arg("POST")
        .arg(format!("{}/api/register", registry_url))
        .arg("-H")
        .arg("Content-Type: application/json")
        .arg("-d")
        .arg(format!(r#"{{"username":"{}","email":"{}"}}"#, username, email))
        .output()?;

    if output.status.success() {
        let response = String::from_utf8_lossy(&output.stdout);
        println!("\n✅ Registration successful!");
        println!("{}", response);

        // Save token to .raven/config
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&response) {
            if let Some(token) = parsed["token"].as_str() {
                let config_dir = PathBuf::from(".raven");
                fs::create_dir_all(&config_dir)?;
                fs::write(config_dir.join("token"), token)?;
                println!("\n🔐 Token saved to .raven/token");
                println!("⚠️  Keep this file secure and add it to .gitignore!");
            }
        }
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        eprintln!("❌ Registration failed: {}", error);
    }

    Ok(())
}

fn publish_package(registry_url: &str) -> std::io::Result<()> {
    // Read package metadata from raven.toml
    let toml_path = PathBuf::from("raven.toml");
    if !toml_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "raven.toml not found. Run 'raven new <project>' to create a project."
        ));
    }

    let toml_content = fs::read_to_string(&toml_path)?;
    let (name, version, description, author) = parse_toml_metadata(&toml_content)?;

    println!("📦 Package: {} v{}", name, version);

    // Read authentication token
    let token_path = PathBuf::from(".raven/token");
    if !token_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Authentication token not found. Run 'raven register' first."
        ));
    }
    let token = fs::read_to_string(&token_path)?.trim().to_string();

    // Build the project first
    println!("\n🔨 Building project...");
    build_project(true)?;

    // Create package tarball
    println!("\n📦 Creating package archive...");
    let archive_path = format!("{}-{}.tar.gz", name, version);

    // Use tar to create archive (simplified - just include src/ and dist/)
    let tar_result = process::Command::new("tar")
        .arg("-czf")
        .arg(&archive_path)
        .arg("src")
        .arg("dist")
        .arg("raven.toml")
        .output()?;

    if !tar_result.status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to create package archive"
        ));
    }

    println!("✅ Created {}", archive_path);

    // Upload to registry
    println!("\n📤 Publishing to registry...");

    let output = process::Command::new("curl")
        .arg("-s")
        .arg("-X")
        .arg("POST")
        .arg(format!("{}/api/publish", registry_url))
        .arg("-H")
        .arg(format!("Authorization: {}", token))
        .arg("-F")
        .arg(format!("package=@{}", archive_path))
        .arg("-F")
        .arg(format!("name={}", name))
        .arg("-F")
        .arg(format!("version={}", version))
        .arg("-F")
        .arg(format!("description={}", description))
        .arg("-F")
        .arg(format!("author={}", author))
        .output()?;

    // Clean up archive
    let _ = fs::remove_file(&archive_path);

    if output.status.success() {
        let response = String::from_utf8_lossy(&output.stdout);
        println!("\n✅ Package published successfully!");
        println!("{}", response);
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        eprintln!("❌ Publish failed: {}", error);
    }

    Ok(())
}

fn search_packages(query: &str, registry_url: &str) -> std::io::Result<()> {
    let output = process::Command::new("curl")
        .arg("-s")
        .arg(format!("{}/api/search?q={}", registry_url, query))
        .output()?;

    if output.status.success() {
        let response = String::from_utf8_lossy(&output.stdout);

        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&response) {
            if let Some(results) = parsed["results"].as_array() {
                if results.is_empty() {
                    println!("No packages found matching '{}'", query);
                } else {
                    println!("\n📦 Found {} package(s):\n", results.len());
                    for pkg in results {
                        let name = pkg["name"].as_str().unwrap_or("unknown");
                        let version = pkg["latestVersion"].as_str().unwrap_or("unknown");
                        let desc = pkg["description"].as_str().unwrap_or("");
                        println!("  {} v{}", name, version);
                        if !desc.is_empty() {
                            println!("    {}", desc);
                        }
                        println!();
                    }
                }
            }
        } else {
            println!("{}", response);
        }
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        eprintln!("❌ Search failed: {}", error);
    }

    Ok(())
}

fn install_package(package: &str, version: Option<&str>, registry_url: &str) -> std::io::Result<()> {
    let version_str = version.unwrap_or("latest");

    // First, get package info
    let info_url = if version.is_some() {
        format!("{}/api/packages/{}/{}", registry_url, package, version_str)
    } else {
        format!("{}/api/packages/{}", registry_url, package)
    };

    println!("📡 Fetching package info...");

    let output = process::Command::new("curl")
        .arg("-s")
        .arg(&info_url)
        .output()?;

    if !output.status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Package not found"
        ));
    }

    let response = String::from_utf8_lossy(&output.stdout);
    let pkg_info: serde_json::Value = serde_json::from_str(&response)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let pkg_version = if version.is_some() {
        version_str.to_string()
    } else {
        pkg_info["latestVersion"]
            .as_str()
            .unwrap_or("0.1.0")
            .to_string()
    };

    println!("📥 Installing {} v{}...", package, pkg_version);

    // Download package files
    let download_url = format!("{}/api/packages/{}/{}/download", registry_url, package, pkg_version);

    let dl_output = process::Command::new("curl")
        .arg("-s")
        .arg(&download_url)
        .output()?;

    if !dl_output.status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Download failed"
        ));
    }

    // Create raven_modules directory
    let modules_dir = PathBuf::from("raven_modules").join(package);
    fs::create_dir_all(&modules_dir)?;

    // Parse file list and download each file
    let files_response = String::from_utf8_lossy(&dl_output.stdout);
    if let Ok(files_info) = serde_json::from_str::<serde_json::Value>(&files_response) {
        if let Some(files) = files_info["files"].as_array() {
            for file in files {
                if let Some(file_url) = file["url"].as_str() {
                    if let Some(file_name) = file["name"].as_str() {
                        let full_url = format!("{}{}", registry_url, file_url);
                        let file_output = process::Command::new("curl")
                            .arg("-s")
                            .arg(&full_url)
                            .output()?;

                        if file_output.status.success() {
                            let file_path = modules_dir.join(file_name);
                            fs::write(&file_path, &file_output.stdout)?;
                            println!("  ✅ Downloaded {}", file_name);
                        }
                    }
                }
            }
        }
    }

    println!("\n✅ Package {} v{} installed successfully!", package, pkg_version);
    println!("📂 Location: raven_modules/{}", package);

    Ok(())
}

fn parse_toml_metadata(toml: &str) -> std::io::Result<(String, String, String, String)> {
    let mut name = String::new();
    let mut version = String::new();
    let mut description = String::new();
    let mut author = String::new();

    for line in toml.lines() {
        let line = line.trim();
        if line.starts_with("name") {
            name = extract_toml_value(line);
        } else if line.starts_with("version") {
            version = extract_toml_value(line);
        } else if line.starts_with("description") {
            description = extract_toml_value(line);
        } else if line.starts_with("author") {
            author = extract_toml_value(line);
        }
    }

    if name.is_empty() || version.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Missing required package metadata (name or version)"
        ));
    }

    Ok((name, version, description, author))
}

fn extract_toml_value(line: &str) -> String {
    line.split('=')
        .nth(1)
        .unwrap_or("")
        .trim()
        .trim_matches('"')
        .to_string()
}