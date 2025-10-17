/**
 * RavensOne Server Runtime
 *
 * Provides server-side APIs for Node.js/Deno
 * - File system access
 * - Environment variables
 * - Server HTTP
 * - Database connections
 * - RPC endpoint handling
 */

const fs = require('fs');
const path = require('path');
const http = require('http');
const https = require('https');

class ServerRuntime {
    constructor() {
        this.rpcHandlers = new Map();
        this.middleware = [];
        this.routes = new Map();
        this.config = {
            port: 3000,
            host: 'localhost',
        };
    }

    /**
     * Initialize server runtime
     */
    async init(config = {}) {
        this.config = { ...this.config, ...config };
        console.log('[Server] Runtime initialized');
        console.log(`[Server] Port: ${this.config.port}`);
        return this;
    }

    // ==================== File System ====================

    /**
     * Read file from filesystem
     */
    async readFile(filePath, encoding = 'utf8') {
        console.log(`[Server] Reading file: ${filePath}`);
        return new Promise((resolve, reject) => {
            fs.readFile(filePath, encoding, (err, data) => {
                if (err) reject(err);
                else resolve(data);
            });
        });
    }

    /**
     * Write file to filesystem
     */
    async writeFile(filePath, data, encoding = 'utf8') {
        console.log(`[Server] Writing file: ${filePath}`);
        return new Promise((resolve, reject) => {
            fs.writeFile(filePath, data, encoding, (err) => {
                if (err) reject(err);
                else resolve(true);
            });
        });
    }

    /**
     * Check if file exists
     */
    async fileExists(filePath) {
        return new Promise((resolve) => {
            fs.access(filePath, fs.constants.F_OK, (err) => {
                resolve(!err);
            });
        });
    }

    /**
     * Read directory contents
     */
    async readDir(dirPath) {
        console.log(`[Server] Reading directory: ${dirPath}`);
        return new Promise((resolve, reject) => {
            fs.readdir(dirPath, (err, files) => {
                if (err) reject(err);
                else resolve(files);
            });
        });
    }

    /**
     * Create directory
     */
    async createDir(dirPath) {
        console.log(`[Server] Creating directory: ${dirPath}`);
        return new Promise((resolve, reject) => {
            fs.mkdir(dirPath, { recursive: true }, (err) => {
                if (err) reject(err);
                else resolve(true);
            });
        });
    }

    /**
     * Delete file
     */
    async deleteFile(filePath) {
        console.log(`[Server] Deleting file: ${filePath}`);
        return new Promise((resolve, reject) => {
            fs.unlink(filePath, (err) => {
                if (err) reject(err);
                else resolve(true);
            });
        });
    }

    // ==================== Environment ====================

    /**
     * Get environment variable
     */
    getEnv(key, defaultValue = null) {
        return process.env[key] || defaultValue;
    }

    /**
     * Set environment variable
     */
    setEnv(key, value) {
        process.env[key] = value;
    }

    /**
     * Load .env file
     */
    async loadEnv(filePath = '.env') {
        try {
            const exists = await this.fileExists(filePath);
            if (!exists) {
                console.log('[Server] No .env file found');
                return;
            }

            const content = await this.readFile(filePath);
            const lines = content.split('\n');

            for (const line of lines) {
                const trimmed = line.trim();
                if (trimmed && !trimmed.startsWith('#')) {
                    const [key, ...valueParts] = trimmed.split('=');
                    const value = valueParts.join('=').trim();
                    this.setEnv(key.trim(), value);
                }
            }

            console.log('[Server] Environment variables loaded');
        } catch (error) {
            console.error('[Server] Failed to load .env:', error);
        }
    }

    // ==================== HTTP Server ====================

    /**
     * Create HTTP server
     */
    createServer() {
        const server = http.createServer(async (req, res) => {
            await this.handleRequest(req, res);
        });

        return server;
    }

    /**
     * Start HTTP server
     */
    async listen(port = null, host = null) {
        const serverPort = port || this.config.port;
        const serverHost = host || this.config.host;

        const server = this.createServer();

        return new Promise((resolve) => {
            server.listen(serverPort, serverHost, () => {
                console.log(`[Server] 🚀 Listening on http://${serverHost}:${serverPort}`);
                resolve(server);
            });
        });
    }

    /**
     * Handle HTTP request
     */
    async handleRequest(req, res) {
        const url = new URL(req.url, `http://${req.headers.host}`);
        const pathname = url.pathname;
        const method = req.method;

        console.log(`[Server] ${method} ${pathname}`);

        // Parse request body
        let body = null;
        if (method === 'POST' || method === 'PUT' || method === 'PATCH') {
            body = await this.parseBody(req);
        }

        // Create request context
        const ctx = {
            req,
            res,
            method,
            url: pathname,
            query: Object.fromEntries(url.searchParams),
            body,
            headers: req.headers,
        };

        // Run middleware
        for (const mw of this.middleware) {
            try {
                await mw(ctx);
            } catch (error) {
                console.error('[Server] Middleware error:', error);
                this.sendError(res, 500, 'Internal Server Error');
                return;
            }
        }

        // Check for route handler
        const routeKey = `${method}:${pathname}`;
        if (this.routes.has(routeKey)) {
            try {
                const handler = this.routes.get(routeKey);
                const result = await handler(ctx);

                // Only send JSON if handler returned a value and response not already sent
                if (result !== null && result !== undefined && !res.headersSent) {
                    this.sendJSON(res, result);
                }
                return;
            } catch (error) {
                console.error('[Server] Route error:', error);
                if (!res.headersSent) {
                    this.sendError(res, 500, error.message);
                }
                return;
            }
        }

        // Check for RPC endpoint
        if (pathname === '/_rpc' && method === 'POST') {
            await this.handleRPC(ctx);
            return;
        }

        // 404 Not Found
        this.sendError(res, 404, 'Not Found');
    }

    /**
     * Parse request body
     */
    async parseBody(req) {
        return new Promise((resolve, reject) => {
            let body = '';
            req.on('data', chunk => {
                body += chunk.toString();
            });
            req.on('end', () => {
                try {
                    const contentType = req.headers['content-type'] || '';
                    if (contentType.includes('application/json')) {
                        resolve(JSON.parse(body));
                    } else {
                        resolve(body);
                    }
                } catch (error) {
                    reject(error);
                }
            });
            req.on('error', reject);
        });
    }

    /**
     * Handle RPC call
     */
    async handleRPC(ctx) {
        try {
            const { method, params } = ctx.body;

            if (!method) {
                this.sendError(ctx.res, 400, 'Missing method');
                return;
            }

            if (!this.rpcHandlers.has(method)) {
                this.sendError(ctx.res, 404, `RPC method not found: ${method}`);
                return;
            }

            console.log(`[Server] RPC call: ${method}`, params);

            const handler = this.rpcHandlers.get(method);
            const result = await handler(...(params || []));

            this.sendJSON(ctx.res, {
                jsonrpc: '2.0',
                result,
                id: ctx.body.id || null,
            });
        } catch (error) {
            console.error('[Server] RPC error:', error);
            this.sendJSON(ctx.res, {
                jsonrpc: '2.0',
                error: {
                    code: -32603,
                    message: error.message,
                },
                id: ctx.body.id || null,
            }, 500);
        }
    }

    // ==================== Routing ====================

    /**
     * Register route handler
     */
    route(method, path, handler) {
        const key = `${method.toUpperCase()}:${path}`;
        this.routes.set(key, handler);
        console.log(`[Server] Route registered: ${key}`);
    }

    /**
     * GET route
     */
    get(path, handler) {
        this.route('GET', path, handler);
    }

    /**
     * POST route
     */
    post(path, handler) {
        this.route('POST', path, handler);
    }

    /**
     * PUT route
     */
    put(path, handler) {
        this.route('PUT', path, handler);
    }

    /**
     * DELETE route
     */
    delete(path, handler) {
        this.route('DELETE', path, handler);
    }

    /**
     * Add middleware
     */
    use(middleware) {
        this.middleware.push(middleware);
        console.log('[Server] Middleware added');
    }

    // ==================== RPC ====================

    /**
     * Register RPC handler
     */
    registerRPC(method, handler) {
        this.rpcHandlers.set(method, handler);
        console.log(`[Server] RPC registered: ${method}`);
    }

    /**
     * Register multiple RPC handlers
     */
    registerRPCHandlers(handlers) {
        for (const [method, handler] of Object.entries(handlers)) {
            this.registerRPC(method, handler);
        }
    }

    // ==================== Response Helpers ====================

    /**
     * Send JSON response
     */
    sendJSON(res, data, status = 200) {
        res.writeHead(status, {
            'Content-Type': 'application/json',
            'Access-Control-Allow-Origin': '*',
        });
        res.end(JSON.stringify(data));
    }

    /**
     * Send error response
     */
    sendError(res, status, message) {
        this.sendJSON(res, { error: message }, status);
    }

    /**
     * Send HTML response
     */
    sendHTML(res, html, status = 200) {
        res.writeHead(status, {
            'Content-Type': 'text/html',
            'Access-Control-Allow-Origin': '*',
        });
        res.end(html);
    }

    /**
     * Redirect
     */
    redirect(res, url, status = 302) {
        res.writeHead(status, {
            'Location': url,
        });
        res.end();
    }

    // ==================== Utilities ====================

    /**
     * Get current timestamp
     */
    now() {
        return Date.now();
    }

    /**
     * Sleep for ms
     */
    async sleep(ms) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }

    /**
     * Generate random ID
     */
    randomId() {
        return Math.random().toString(36).substr(2, 9);
    }
}

// Export for Node.js
if (typeof module !== 'undefined' && module.exports) {
    module.exports = ServerRuntime;
}

// Global instance
if (typeof global !== 'undefined') {
    global.RavensServer = new ServerRuntime();
}
