// Hot Module Replacement for RavensOne
// Real-time code updates without full page reload

use crate::{Compiler, BuildTarget, errors::CompileError};
use notify::{Watcher, RecursiveMode, Result as NotifyResult, Event, EventKind};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::sync::mpsc;
use futures_util::{StreamExt, SinkExt};
use tokio_tungstenite::tungstenite::Message;

/// HMR Server configuration
pub struct HmrConfig {
    pub watch_paths: Vec<PathBuf>,
    pub websocket_port: u16,
    pub debounce_ms: u64,
    pub preserve_state: bool,
}

impl Default for HmrConfig {
    fn default() -> Self {
        HmrConfig {
            watch_paths: vec![PathBuf::from("src")],
            websocket_port: 3001,
            debounce_ms: 100,
            preserve_state: true,
        }
    }
}

/// HMR Server
pub struct HmrServer {
    config: HmrConfig,
    compiler: Arc<Compiler>,
    clients: Arc<Mutex<Vec<mpsc::UnboundedSender<Message>>>>,
    last_compile: Arc<Mutex<Option<Vec<u8>>>>,
}

/// HMR Update message
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HmrUpdate {
    pub update_type: UpdateType,
    pub file_path: String,
    pub timestamp: u64,
    pub wasm_url: Option<String>,
    pub css_content: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum UpdateType {
    WasmUpdate,
    CssUpdate,
    FullReload,
    Connected,
}

impl HmrServer {
    pub fn new(config: HmrConfig) -> Self {
        HmrServer {
            config,
            compiler: Arc::new(Compiler::new()),
            clients: Arc::new(Mutex::new(Vec::new())),
            last_compile: Arc::new(Mutex::new(None)),
        }
    }

    /// Start the HMR server
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔥 Starting HMR server on ws://localhost:{}", self.config.websocket_port);

        // Start file watcher
        let (file_tx, mut file_rx) = mpsc::unbounded_channel();
        self.start_file_watcher(file_tx)?;

        // Start WebSocket server
        let addr = format!("127.0.0.1:{}", self.config.websocket_port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        println!("✅ WebSocket server listening on {}", addr);

        let clients = self.clients.clone();
        let clients_for_file_handler = self.clients.clone(); // Clone for second spawn
        let compiler = self.compiler.clone();
        let last_compile = self.last_compile.clone();
        let preserve_state = self.config.preserve_state;

        // Spawn WebSocket connection handler
        tokio::spawn(async move {
            while let Ok((stream, addr)) = listener.accept().await {
                println!("🔌 New client connected: {}", addr);
                let clients = clients.clone();

                tokio::spawn(async move {
                    if let Ok(ws_stream) = tokio_tungstenite::accept_async(stream).await {
                        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
                        let (tx, mut rx) = mpsc::unbounded_channel();

                        // Add client to broadcast list
                        clients.lock().unwrap().push(tx.clone());

                        // Send connected message
                        let connected = HmrUpdate {
                            update_type: UpdateType::Connected,
                            file_path: String::new(),
                            timestamp: current_timestamp(),
                            wasm_url: None,
                            css_content: None,
                        };
                        if let Ok(json) = serde_json::to_string(&connected) {
                            let _ = ws_sender.send(Message::Text(json)).await;
                        }

                        // Handle incoming messages and outgoing updates
                        loop {
                            tokio::select! {
                                msg = ws_receiver.next() => {
                                    if msg.is_none() {
                                        break;
                                    }
                                }
                                Some(msg) = rx.recv() => {
                                    if ws_sender.send(msg).await.is_err() {
                                        break;
                                    }
                                }
                            }
                        }

                        println!("🔌 Client disconnected: {}", addr);
                    }
                });
            }
        });

        // Handle file changes
        tokio::spawn(async move {
            while let Some(event) = file_rx.recv().await {
                println!("📝 File changed: {:?}", event);

                // Compile the changed file
                match Self::handle_file_change(
                    &compiler,
                    &last_compile,
                    &clients_for_file_handler,
                    event,
                    preserve_state,
                )
                .await
                {
                    Ok(_) => println!("✅ HMR update sent"),
                    Err(e) => eprintln!("❌ HMR error: {:?}", e),
                }
            }
        });

        // Keep server running
        tokio::signal::ctrl_c().await?;
        println!("\n👋 HMR server shutting down...");

        Ok(())
    }

    /// Start file watcher
    fn start_file_watcher(
        &self,
        tx: mpsc::UnboundedSender<PathBuf>,
    ) -> NotifyResult<()> {
        let mut watcher = notify::recommended_watcher(move |res: NotifyResult<Event>| {
            if let Ok(event) = res {
                match event.kind {
                    EventKind::Modify(_) | EventKind::Create(_) => {
                        for path in event.paths {
                            if path.extension().and_then(|s| s.to_str()) == Some("raven") {
                                let _ = tx.send(path);
                            }
                        }
                    }
                    _ => {}
                }
            }
        })?;

        for path in &self.config.watch_paths {
            watcher.watch(path, RecursiveMode::Recursive)?;
        }

        // Keep watcher alive
        std::mem::forget(watcher);

        Ok(())
    }

    /// Handle file change event
    async fn handle_file_change(
        compiler: &Arc<Compiler>,
        last_compile: &Arc<Mutex<Option<Vec<u8>>>>,
        clients: &Arc<Mutex<Vec<mpsc::UnboundedSender<Message>>>>,
        file_path: PathBuf,
        preserve_state: bool,
    ) -> Result<(), CompileError> {
        // Read file content
        let content = std::fs::read_to_string(&file_path)
            .map_err(|e| CompileError::LexerError(format!("Failed to read file: {}", e)))?;

        // Compile
        let wasm_bytes = compiler.compile_source(&content, BuildTarget::Client)?;

        // Store last compile
        *last_compile.lock().unwrap() = Some(wasm_bytes.clone());

        // Check if it's a CSS file
        let is_css = file_path.extension().and_then(|s| s.to_str()) == Some("css");

        // Create update message
        let update = HmrUpdate {
            update_type: if is_css {
                UpdateType::CssUpdate
            } else if preserve_state {
                UpdateType::WasmUpdate
            } else {
                UpdateType::FullReload
            },
            file_path: file_path.to_string_lossy().to_string(),
            timestamp: current_timestamp(),
            wasm_url: if !is_css {
                Some(format!("/hmr/wasm?t={}", current_timestamp()))
            } else {
                None
            },
            css_content: if is_css {
                Some(content)
            } else {
                None
            },
        };

        // Broadcast to all clients
        let json = serde_json::to_string(&update)
            .map_err(|e| CompileError::LexerError(format!("JSON error: {}", e)))?;

        let clients_lock = clients.lock().unwrap();
        let mut disconnected = Vec::new();

        for (i, client) in clients_lock.iter().enumerate() {
            if client.send(Message::Text(json.clone())).is_err() {
                disconnected.push(i);
            }
        }

        drop(clients_lock);

        // Remove disconnected clients
        if !disconnected.is_empty() {
            let mut clients_lock = clients.lock().unwrap();
            for i in disconnected.iter().rev() {
                clients_lock.remove(*i);
            }
        }

        Ok(())
    }

    /// Get the last compiled WASM
    pub fn get_last_wasm(&self) -> Option<Vec<u8>> {
        self.last_compile.lock().unwrap().clone()
    }
}

/// Get current timestamp in milliseconds
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

/// HMR Client runtime (injected into browser)
pub const HMR_CLIENT_SCRIPT: &str = r#"
(function() {
    console.log('🔥 HMR Client initializing...');

    let ws = null;
    let reconnectAttempts = 0;
    const maxReconnectAttempts = 10;

    function connect() {
        ws = new WebSocket('ws://localhost:3001');

        ws.onopen = () => {
            console.log('✅ HMR Connected');
            reconnectAttempts = 0;
        };

        ws.onmessage = async (event) => {
            try {
                const update = JSON.parse(event.data);
                console.log('📦 HMR Update:', update);

                switch (update.update_type) {
                    case 'WasmUpdate':
                        await handleWasmUpdate(update);
                        break;
                    case 'CssUpdate':
                        handleCssUpdate(update);
                        break;
                    case 'FullReload':
                        location.reload();
                        break;
                    case 'Connected':
                        console.log('🔌 HMR Ready');
                        break;
                }
            } catch (e) {
                console.error('❌ HMR Error:', e);
            }
        };

        ws.onerror = (error) => {
            console.error('❌ WebSocket error:', error);
        };

        ws.onclose = () => {
            console.log('🔌 HMR Disconnected');
            if (reconnectAttempts < maxReconnectAttempts) {
                reconnectAttempts++;
                console.log(`🔄 Reconnecting (${reconnectAttempts}/${maxReconnectAttempts})...`);
                setTimeout(connect, 1000 * reconnectAttempts);
            }
        };
    }

    async function handleWasmUpdate(update) {
        try {
            // Preserve reactive state
            const state = window.__RAVEN_STATE || {};

            // Fetch new WASM module
            const response = await fetch(update.wasm_url);
            const wasm = await response.arrayBuffer();

            // Reload WASM module
            const module = await WebAssembly.instantiate(wasm, window.__RAVEN_IMPORTS);

            // Update exports
            window.__RAVEN_EXPORTS = module.instance.exports;

            // Restore state
            window.__RAVEN_STATE = state;

            // Re-render
            if (window.__RAVEN_EXPORTS.render) {
                window.__RAVEN_EXPORTS.render();
            }

            console.log('✅ WASM module updated (state preserved)');
        } catch (e) {
            console.error('❌ Failed to update WASM:', e);
            location.reload();
        }
    }

    function handleCssUpdate(update) {
        try {
            // Find or create style element
            let styleEl = document.getElementById('hmr-styles');
            if (!styleEl) {
                styleEl = document.createElement('style');
                styleEl.id = 'hmr-styles';
                document.head.appendChild(styleEl);
            }

            // Update CSS content
            styleEl.textContent = update.css_content;

            console.log('✅ CSS updated');
        } catch (e) {
            console.error('❌ Failed to update CSS:', e);
        }
    }

    // Connect on load
    connect();
})();
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hmr_config_default() {
        let config = HmrConfig::default();
        assert_eq!(config.websocket_port, 3001);
        assert_eq!(config.debounce_ms, 100);
        assert_eq!(config.preserve_state, true);
    }

    #[test]
    fn test_update_type_serialization() {
        let update = HmrUpdate {
            update_type: UpdateType::WasmUpdate,
            file_path: "test.raven".to_string(),
            timestamp: 12345,
            wasm_url: Some("/hmr/wasm".to_string()),
            css_content: None,
        };

        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("WasmUpdate"));
        assert!(json.contains("test.raven"));
    }

    #[test]
    fn test_hmr_client_script_exists() {
        assert!(!HMR_CLIENT_SCRIPT.is_empty());
        assert!(HMR_CLIENT_SCRIPT.contains("WebSocket"));
        assert!(HMR_CLIENT_SCRIPT.contains("handleWasmUpdate"));
    }
}
