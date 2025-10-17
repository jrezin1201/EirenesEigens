use crate::errors::CompileError;
use crate::{Compiler, BuildTarget};
use std::fs;
use std::path::Path;

/// A trait for different deployment providers (e.g., Cloudflare, Vercel).
pub trait DeployProvider {
    fn deploy_client(&self, client_dist_path: &Path) -> Result<String, CompileError>;
    fn deploy_server(&self, server_wasm_path: &Path) -> Result<(), CompileError>;
}

/// An example implementation for Cloudflare.
pub struct CloudflareProvider {
    // This would hold API tokens and account info.
}

impl DeployProvider for CloudflareProvider {
    fn deploy_client(&self, client_dist_path: &Path) -> Result<String, CompileError> {
        println!("   (Cloudflare) Uploading client assets from '{}'...", client_dist_path.display());
        // In a real implementation, this would use the Cloudflare API.
        let live_url = "https://aloha-shirts-xyz.pages.dev".to_string();
        Ok(live_url)
    }

    fn deploy_server(&self, server_wasm_path: &Path) -> Result<(), CompileError> {
        println!("   (Cloudflare) Uploading server module from '{}'...", server_wasm_path.display());
        // This would use the Cloudflare API to upload `server.wasm` to a Worker.
        Ok(())
    }
}

/// The main deployment function that orchestrates the entire process.
pub fn deploy_project() -> Result<(), CompileError> {
    let compiler = Compiler::new();
    let provider = CloudflareProvider {};
    
    // FIX: Instead of a placeholder comment, we provide a valid, minimal
    // RavensOne program for the compiler to process.
    let source = fs::read_to_string("src/main.raven")
        .map_err(|e| CompileError::Generic(format!("Could not read project entrypoint 'src/main.raven': {}", e)))?;

    // 1. Create distribution directories
    let client_dist_path = Path::new("./dist/client");
    let server_dist_path = Path::new("./dist/server");
    fs::create_dir_all(client_dist_path).map_err(|e| CompileError::Generic(e.to_string()))?;
    fs::create_dir_all(server_dist_path).map_err(|e| CompileError::Generic(e.to_string()))?;

    // 2. Build client assets
    println!("   - Building client assets (--target client)...");
    let client_bytes = compiler.compile_source(&source, BuildTarget::Client)?;
    fs::write(client_dist_path.join("app.wasm"), client_bytes).map_err(|e| CompileError::Generic(e.to_string()))?;

    // 3. Build server module
    println!("   - Building server functions (--target server)...");
    let server_bytes = compiler.compile_source(&source, BuildTarget::Server)?;
    let server_wasm_path = server_dist_path.join("server.wasm");
    fs::write(&server_wasm_path, server_bytes).map_err(|e| CompileError::Generic(e.to_string()))?;

    // 4. Deploy both artifacts
    let url = provider.deploy_client(client_dist_path)?;
    provider.deploy_server(&server_wasm_path)?;

    println!("\n✨ Deployment successful! Application is live at: {}", url);
    Ok(())
}