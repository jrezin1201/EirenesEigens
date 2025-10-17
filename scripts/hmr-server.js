#!/usr/bin/env node

/**
 * RavensOne Hot Module Reloading (HMR) Dev Server
 *
 * Watches files for changes and notifies browser clients via WebSocket
 * Enables live updates without losing application state
 */

const fs = require('fs');
const path = require('path');
const http = require('http');
const WebSocket = require('ws');

class HMRServer {
    constructor(options = {}) {
        this.port = options.port || 3002;
        this.watchDirs = options.watchDirs || ['dist', 'examples', 'src'];
        this.clients = new Set();
        this.watchers = [];
        this.debounceTimers = new Map();
        this.debounceDelay = 300; // ms
    }

    /**
     * Start the HMR server
     */
    start() {
        console.log('🔥 Starting RavensOne HMR Server...\n');

        // Create HTTP server for WebSocket upgrade
        this.httpServer = http.createServer((req, res) => {
            res.writeHead(200, { 'Content-Type': 'text/plain' });
            res.end('RavensOne HMR Server Running\n');
        });

        // Create WebSocket server
        this.wss = new WebSocket.Server({
            server: this.httpServer,
            path: '/hmr'
        });

        this.wss.on('connection', (ws) => {
            this.handleConnection(ws);
        });

        // Start HTTP server
        this.httpServer.listen(this.port, () => {
            console.log(`✅ HMR Server listening on port ${this.port}`);
            console.log(`   WebSocket endpoint: ws://localhost:${this.port}/hmr\n`);
        });

        // Watch directories
        this.watchDirectories();

        // Ping clients periodically
        this.startPingInterval();

        console.log('👀 Watching for file changes...\n');
    }

    /**
     * Handle new WebSocket connection
     */
    handleConnection(ws) {
        console.log('🔌 Client connected');
        this.clients.add(ws);

        // Send welcome message
        this.send(ws, {
            type: 'connected',
            data: {
                version: '1.0.0',
                message: 'HMR connected'
            }
        });

        ws.on('message', (message) => {
            try {
                const data = JSON.parse(message);
                this.handleMessage(ws, data);
            } catch (error) {
                console.error('Invalid message:', error);
            }
        });

        ws.on('close', () => {
            console.log('🔌 Client disconnected');
            this.clients.delete(ws);
        });

        ws.on('error', (error) => {
            console.error('WebSocket error:', error);
            this.clients.delete(ws);
        });
    }

    /**
     * Handle client messages
     */
    handleMessage(ws, message) {
        const { type, data } = message;

        switch (type) {
            case 'pong':
                // Client responded to ping
                ws.isAlive = true;
                break;

            default:
                console.log('Unknown message type:', type);
        }
    }

    /**
     * Watch directories for file changes
     */
    watchDirectories() {
        const projectRoot = path.join(__dirname, '..');

        for (const dir of this.watchDirs) {
            const fullPath = path.join(projectRoot, dir);

            if (!fs.existsSync(fullPath)) {
                console.log(`⚠️  Directory not found: ${dir}`);
                continue;
            }

            console.log(`👁️  Watching: ${dir}/`);

            try {
                const watcher = fs.watch(fullPath, { recursive: true }, (eventType, filename) => {
                    if (filename) {
                        this.handleFileChange(dir, filename, eventType);
                    }
                });

                this.watchers.push(watcher);
            } catch (error) {
                console.error(`Failed to watch ${dir}:`, error.message);
            }
        }
    }

    /**
     * Handle file change event
     */
    handleFileChange(baseDir, filename, eventType) {
        const fullPath = path.join(baseDir, filename);

        // Ignore certain files
        if (this.shouldIgnoreFile(filename)) {
            return;
        }

        // Debounce rapid changes
        const debounceKey = fullPath;
        if (this.debounceTimers.has(debounceKey)) {
            clearTimeout(this.debounceTimers.get(debounceKey));
        }

        this.debounceTimers.set(debounceKey, setTimeout(() => {
            this.debounceTimers.delete(debounceKey);
            this.processFileChange(fullPath, eventType);
        }, this.debounceDelay));
    }

    /**
     * Process file change after debounce
     */
    processFileChange(filePath, eventType) {
        console.log(`📝 File changed: ${filePath}`);

        const ext = path.extname(filePath);
        const updateType = this.getUpdateType(ext);

        if (!updateType) {
            console.log(`   Ignoring ${ext} file`);
            return;
        }

        // Read file content for CSS updates
        let content = null;
        if (updateType === 'css') {
            try {
                const fullPath = path.join(__dirname, '..', filePath);
                if (fs.existsSync(fullPath)) {
                    content = fs.readFileSync(fullPath, 'utf-8');
                }
            } catch (error) {
                console.error('   Failed to read file:', error.message);
            }
        }

        // Broadcast update to all clients
        this.broadcast({
            type: 'update',
            data: {
                file: filePath,
                type: updateType,
                content: content,
                timestamp: Date.now()
            }
        });

        console.log(`   ✅ Notified ${this.clients.size} client(s)`);
    }

    /**
     * Get update type based on file extension
     */
    getUpdateType(ext) {
        const typeMap = {
            '.css': 'css',
            '.js': 'js',
            '.mjs': 'js',
            '.wasm': 'wasm',
            '.html': 'html',
            '.htm': 'html'
        };

        return typeMap[ext.toLowerCase()] || null;
    }

    /**
     * Check if file should be ignored
     */
    shouldIgnoreFile(filename) {
        const ignorePatterns = [
            /node_modules/,
            /\.git/,
            /\.DS_Store/,
            /\.swp$/,
            /~$/,
            /\.tmp$/,
            /\.log$/,
            /package-lock\.json$/,
            /\.md$/  // Ignore markdown files
        ];

        return ignorePatterns.some(pattern => pattern.test(filename));
    }

    /**
     * Send message to specific client
     */
    send(ws, message) {
        if (ws.readyState === WebSocket.OPEN) {
            ws.send(JSON.stringify(message));
        }
    }

    /**
     * Broadcast message to all clients
     */
    broadcast(message) {
        const json = JSON.stringify(message);

        for (const client of this.clients) {
            if (client.readyState === WebSocket.OPEN) {
                client.send(json);
            }
        }
    }

    /**
     * Start ping interval to keep connections alive
     */
    startPingInterval() {
        setInterval(() => {
            for (const client of this.clients) {
                if (client.readyState === WebSocket.OPEN) {
                    if (client.isAlive === false) {
                        console.log('🔌 Client timed out, terminating');
                        client.terminate();
                        this.clients.delete(client);
                        continue;
                    }

                    client.isAlive = false;
                    this.send(client, { type: 'ping' });
                }
            }
        }, 30000); // Ping every 30 seconds
    }

    /**
     * Notify clients of compilation error
     */
    notifyError(file, error, line = null) {
        console.error(`❌ Compilation error in ${file}:`, error);

        this.broadcast({
            type: 'error',
            data: {
                file,
                error: error.toString(),
                line,
                timestamp: Date.now()
            }
        });
    }

    /**
     * Stop the server
     */
    stop() {
        console.log('\n🛑 Stopping HMR server...');

        // Close all watchers
        for (const watcher of this.watchers) {
            watcher.close();
        }

        // Close all WebSocket connections
        for (const client of this.clients) {
            client.close();
        }

        // Close servers
        if (this.wss) {
            this.wss.close();
        }

        if (this.httpServer) {
            this.httpServer.close();
        }

        console.log('✅ HMR server stopped');
    }
}

// CLI Usage
if (require.main === module) {
    const server = new HMRServer({
        port: process.env.HMR_PORT || 3002,
        watchDirs: ['dist', 'examples', 'src']
    });

    server.start();

    // Graceful shutdown
    process.on('SIGINT', () => {
        server.stop();
        process.exit(0);
    });

    process.on('SIGTERM', () => {
        server.stop();
        process.exit(0);
    });
}

module.exports = HMRServer;
