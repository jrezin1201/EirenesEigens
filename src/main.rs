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
        #[arg(short, long)]
        minify: bool,
    },
    /// Creates a new RavensOne project
    New {
        name: String,
    },
    /// Initialize a new RavensOne project in the current directory
    Init {
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Start a local development server
    Serve {
        #[arg(short, long, default_value = "8000")]
        port: u16,
        #[arg(long)]
        open: bool,
    },
    /// Diagnose common issues with your RavensOne setup
    Doctor,
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
    /// Login to the package registry
    Login,
    /// Register a new account
    Register,
    /// Publish the current package to the registry
    Publish,
    /// Search for packages in the registry
    Search {
        query: String,
    },
    /// Display dependency tree
    Tree,
    /// Check for outdated dependencies
    Outdated,
    /// List all installed packages
    List,
    /// Show detailed information about a package
    Info {
        name: String,
    },
    /// Show build cache statistics
    Cache,
    /// Clear build cache
    Clean,
    /// Audit dependencies for security vulnerabilities
    Audit,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compile { path, output, minify } => {
            use ravensone_compiler::lexer::Lexer;
            use ravensone_compiler::parser::Parser;
            use ravensone_compiler::js_emitter::JSEmitter;
            use ravensone_compiler::js_minifier::JSMinifier;
            use ravensone_compiler::LexerExt;

            println!("🔥 Compiling full-stack application: {}", path.display());
            if minify {
                println!("   🗜️  Minification: enabled");
            }
            println!("   📦 Output: server.js + client.js + app.wasm\n");

            // Read source code
            let source_code = match fs::read_to_string(&path) {
                Ok(code) => code,
                Err(e) => {
                    eprintln!("❌ Error reading file '{}': {}", path.display(), e);
                    return;
                }
            };

            // Parse the source
            println!("   Parsing...");
            let mut lexer = Lexer::new(source_code.clone());
            let tokens = match lexer.collect_tokens() {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("❌ Lexing failed: {}", e);
                    return;
                }
            };

            let mut parser = Parser::new(tokens);
            let program = match parser.parse_program() {
                Ok(p) => {
                    println!("   ✓ Parsed {} statements", p.statements.len());
                    p
                }
                Err(e) => {
                    eprintln!("❌ Parsing failed: {:?}", e);
                    return;
                }
            };

            // Generate JavaScript bundles
            println!("   Generating JavaScript bundles...");
            let emitter = JSEmitter::new(&program);
            let mut server_js = emitter.generate_server_js();
            let mut client_js = emitter.generate_client_js();

            let stats = emitter.stats();
            println!("   ✓ Split: {} server, {} client, {} shared functions",
                stats.server_functions, stats.client_functions, stats.shared_functions);

            // Minify if requested
            if minify {
                println!("   Minifying JavaScript...");
                let minifier = JSMinifier::new();

                let server_minified = minifier.minify(&server_js);
                let client_minified = minifier.minify(&client_js);

                let server_stats = minifier.stats(&server_js, &server_minified);
                let client_stats = minifier.stats(&client_js, &client_minified);

                println!("   ✓ server.js: {} → {} bytes (-{:.1}%)",
                    server_stats.original_size, server_stats.minified_size, server_stats.reduction_percent);
                println!("   ✓ client.js: {} → {} bytes (-{:.1}%)",
                    client_stats.original_size, client_stats.minified_size, client_stats.reduction_percent);

                server_js = server_minified;
                client_js = client_minified;
            }

            // Compile to WASM
            println!("   Compiling to WebAssembly...");
            let compiler = Compiler::new();
            let wasm_bytes = match compiler.compile_source(&source_code, BuildTarget::Client) {
                Ok(bytes) => {
                    println!("   ✓ Generated WASM module ({} bytes)", bytes.len());
                    bytes
                }
                Err(e) => {
                    eprintln!("❌ WASM compilation failed: {}", e);
                    return;
                }
            };

            // Determine output directory
            let output_dir = output.unwrap_or_else(|| PathBuf::from("dist"));
            if let Err(e) = fs::create_dir_all(&output_dir) {
                eprintln!("❌ Failed to create output directory: {}", e);
                return;
            }

            // Write output files
            println!("\n   Writing output files...");

            let server_path = output_dir.join("server.js");
            if let Err(e) = fs::write(&server_path, server_js) {
                eprintln!("❌ Failed to write server.js: {}", e);
                return;
            }
            println!("   ✓ {}", server_path.display());

            let client_path = output_dir.join("client.js");
            if let Err(e) = fs::write(&client_path, client_js) {
                eprintln!("❌ Failed to write client.js: {}", e);
                return;
            }
            println!("   ✓ {}", client_path.display());

            let wasm_path = output_dir.join("app.wasm");
            if let Err(e) = fs::write(&wasm_path, wasm_bytes) {
                eprintln!("❌ Failed to write app.wasm: {}", e);
                return;
            }
            println!("   ✓ {}", wasm_path.display());

            // Create index.html
            let html_content = generate_index_html();
            let html_path = output_dir.join("index.html");
            if let Err(e) = fs::write(&html_path, html_content) {
                eprintln!("⚠️  Warning: Failed to write index.html: {}", e);
            } else {
                println!("   ✓ {}", html_path.display());
            }

            println!("\n✨ Compilation complete!");
            println!("   Run: cd {} && node server.js", output_dir.display());
        }
        Commands::New { name } => {
            // FIX: Added logic for creating a new project
            if let Err(e) = create_new_project(&name) {
                eprintln!("❌ Error creating new project: {}", e);
                process::exit(1);
            }
            println!("✅ Project '{}' created successfully! 🚀", name);
        }
        Commands::Init { path } => {
            println!("🚀 Initializing RavensOne project...");
            if let Err(e) = init_project(&path) {
                eprintln!("❌ Initialization failed: {}", e);
                process::exit(1);
            }
        }
        Commands::Serve { port, open } => {
            println!("🌐 Starting local development server on port {}...", port);
            if let Err(e) = serve_project(port, open) {
                eprintln!("❌ Server failed: {}", e);
                process::exit(1);
            }
        }
        Commands::Doctor => {
            println!("🏥 Running RavensOne diagnostics...\n");
            run_doctor();
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
                PkgCommands::Login => {
                    let mut pkg_mgr = PackageManager::new(&PathBuf::from("."));
                    if let Err(e) = pkg_mgr.login() {
                        eprintln!("❌ Login failed: {}", e);
                        process::exit(1);
                    }
                }
                PkgCommands::Register => {
                    let mut pkg_mgr = PackageManager::new(&PathBuf::from("."));
                    if let Err(e) = pkg_mgr.register() {
                        eprintln!("❌ Registration failed: {}", e);
                        process::exit(1);
                    }
                }
                PkgCommands::Publish => {
                    let pkg_mgr = PackageManager::new(&PathBuf::from("."));
                    if let Err(e) = pkg_mgr.publish() {
                        eprintln!("❌ Publish failed: {}", e);
                        process::exit(1);
                    }
                }
                PkgCommands::Search { query } => {
                    let pkg_mgr = PackageManager::new(&PathBuf::from("."));
                    if let Err(e) = pkg_mgr.search(&query) {
                        eprintln!("❌ Search failed: {}", e);
                        process::exit(1);
                    }
                }
                PkgCommands::Tree => {
                    let pkg_mgr = PackageManager::new(&PathBuf::from("."));
                    if let Err(e) = pkg_mgr.tree() {
                        eprintln!("❌ Tree failed: {}", e);
                        process::exit(1);
                    }
                }
                PkgCommands::Outdated => {
                    let pkg_mgr = PackageManager::new(&PathBuf::from("."));
                    if let Err(e) = pkg_mgr.outdated() {
                        eprintln!("❌ Outdated check failed: {}", e);
                        process::exit(1);
                    }
                }
                PkgCommands::List => {
                    let pkg_mgr = PackageManager::new(&PathBuf::from("."));
                    if let Err(e) = pkg_mgr.list() {
                        eprintln!("❌ List failed: {}", e);
                        process::exit(1);
                    }
                }
                PkgCommands::Info { name } => {
                    let pkg_mgr = PackageManager::new(&PathBuf::from("."));
                    if let Err(e) = pkg_mgr.info(&name) {
                        eprintln!("❌ Info failed: {}", e);
                        process::exit(1);
                    }
                }
                PkgCommands::Cache => {
                    let pkg_mgr = PackageManager::new(&PathBuf::from("."));
                    if let Err(e) = pkg_mgr.cache_stats() {
                        eprintln!("❌ Cache stats failed: {}", e);
                        process::exit(1);
                    }
                }
                PkgCommands::Clean => {
                    let pkg_mgr = PackageManager::new(&PathBuf::from("."));
                    if let Err(e) = pkg_mgr.clean_cache() {
                        eprintln!("❌ Cache clean failed: {}", e);
                        process::exit(1);
                    }
                }
                PkgCommands::Audit => {
                    let pkg_mgr = PackageManager::new(&PathBuf::from("."));
                    if let Err(e) = pkg_mgr.audit() {
                        eprintln!("❌ Audit failed: {}", e);
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

// New CLI commands

fn init_project(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    use ravensone_compiler::package_manager::PackageManager;

    let pkg_mgr = PackageManager::new(path);
    let project_name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("my-package");

    pkg_mgr.init(project_name, vec!["Developer <dev@example.com>".to_string()])?;

    println!("✅ Initialized RavensOne project in {}", path.display());
    println!("   Created raven.toml");
    println!("\n💡 Next steps:");
    println!("   1. Edit raven.toml to add package metadata");
    println!("   2. Run 'raven build' to compile your project");
    println!("   3. Run 'raven serve' to start a local development server");

    Ok(())
}

fn serve_project(port: u16, open: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("✅ Starting local development server...");
    println!("   📂 Serving from: ./dist");
    println!("   🌐 URL: http://localhost:{}", port);

    // Check if dist directory exists
    let dist_dir = PathBuf::from("dist");
    if !dist_dir.exists() {
        println!("\n⚠️  dist/ directory not found. Building project first...\n");
        build_project(true)?;
    }

    if open {
        // Open browser
        let url = format!("http://localhost:{}", port);
        #[cfg(target_os = "macos")]
        let _ = process::Command::new("open").arg(&url).spawn();
        #[cfg(target_os = "linux")]
        let _ = process::Command::new("xdg-open").arg(&url).spawn();
        #[cfg(target_os = "windows")]
        let _ = process::Command::new("cmd").arg("/C").arg("start").arg(&url).spawn();
    }

    // Start simple HTTP server
    println!("\n✨ Server running! Press Ctrl+C to stop.\n");

    let result = process::Command::new("python3")
        .arg("-m")
        .arg("http.server")
        .arg(port.to_string())
        .arg("--directory")
        .arg("dist")
        .spawn();

    if let Ok(mut child) = result {
        let _ = child.wait();
    } else {
        return Err("Failed to start HTTP server. Make sure python3 is installed.".into());
    }

    Ok(())
}

fn generate_index_html() -> String {
    r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>RavensOne App</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            margin: 0;
            padding: 20px;
            background: #f5f5f5;
        }
        #app {
            max-width: 800px;
            margin: 0 auto;
            background: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
    </style>
</head>
<body>
    <div id="app">
        <h1>Loading RavensOne App...</h1>
    </div>
    <script type="module" src="client.js"></script>
</body>
</html>"#.to_string()
}

fn run_doctor() {
    println!("🏥 RavensOne Doctor - Checking your setup...\n");

    let mut issues = 0;
    let mut warnings = 0;

    // Check Rust installation
    print!("  Checking Rust... ");
    if let Ok(output) = process::Command::new("rustc").arg("--version").output() {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("✅ {}", version.trim());
        } else {
            println!("❌ FAILED");
            issues += 1;
        }
    } else {
        println!("❌ NOT FOUND");
        issues += 1;
    }

    // Check Cargo
    print!("  Checking Cargo... ");
    if let Ok(output) = process::Command::new("cargo").arg("--version").output() {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("✅ {}", version.trim());
        } else {
            println!("❌ FAILED");
            issues += 1;
        }
    } else {
        println!("❌ NOT FOUND");
        issues += 1;
    }

    // Check Node.js (optional for HMR)
    print!("  Checking Node.js... ");
    if let Ok(output) = process::Command::new("node").arg("--version").output() {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("✅ {}", version.trim());
        } else {
            println!("⚠️  FAILED (optional)");
            warnings += 1;
        }
    } else {
        println!("⚠️  NOT FOUND (optional - needed for HMR)");
        warnings += 1;
    }

    // Check Python (optional for dev server)
    print!("  Checking Python... ");
    if let Ok(output) = process::Command::new("python3").arg("--version").output() {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("✅ {}", version.trim());
        } else {
            println!("⚠️  FAILED (optional)");
            warnings += 1;
        }
    } else {
        println!("⚠️  NOT FOUND (optional - needed for 'raven serve')");
        warnings += 1;
    }

    // Check project structure
    print!("  Checking project structure... ");
    if PathBuf::from("raven.toml").exists() {
        println!("✅ raven.toml found");
    } else {
        println!("⚠️  No raven.toml (run 'raven init' to create one)");
        warnings += 1;
    }

    print!("  Checking src/ directory... ");
    if PathBuf::from("src").exists() {
        println!("✅ src/ directory exists");
    } else {
        println!("⚠️  No src/ directory");
        warnings += 1;
    }

    // Summary
    println!("\n📊 Summary:");
    if issues == 0 && warnings == 0 {
        println!("   ✅ All checks passed! Your RavensOne setup looks good.");
    } else {
        if issues > 0 {
            println!("   ❌ {} critical issue(s) found", issues);
        }
        if warnings > 0 {
            println!("   ⚠️  {} warning(s)", warnings);
        }
    }

    if issues > 0 {
        println!("\n💡 Recommendations:");
        println!("   - Install Rust from: https://rustup.rs/");
        println!("   - Rust and Cargo are required for RavensOne to work");
    }

    if warnings > 0 {
        println!("\n💡 Optional improvements:");
        println!("   - Install Node.js for HMR support: https://nodejs.org/");
        println!("   - Install Python for 'raven serve' command");
        println!("   - Run 'raven init' to create a new project");
    }
}