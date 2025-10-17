/**
 * RavensOne Full-Stack Demo - Server
 *
 * This demonstrates server-side code that:
 * - Handles data persistence
 * - Exposes RPC methods
 * - Manages business logic
 */

const ServerRuntime = require('./dist/server-runtime.js');

// Create server instance
const server = new ServerRuntime();

// In-memory data store (in production, use real database)
let todos = [
    { id: 1, text: 'Build RavensOne compiler', completed: true, createdAt: new Date().toISOString() },
    { id: 2, text: 'Implement HTTP client', completed: true, createdAt: new Date().toISOString() },
    { id: 3, text: 'Build database ORM', completed: true, createdAt: new Date().toISOString() },
    { id: 4, text: 'Add authentication system', completed: true, createdAt: new Date().toISOString() },
    { id: 5, text: 'Create server/client split', completed: false, createdAt: new Date().toISOString() },
];

let nextId = 6;

// ==================== RPC Methods ====================
// These methods are called from the client via RPC

/**
 * Get all todos
 * @server - This only runs on the server
 */
async function getTodos() {
    console.log('[Server] RPC: getTodos');
    // Simulate database query
    await server.sleep(100);
    return todos;
}

/**
 * Get single todo by ID
 * @server
 */
async function getTodo(id) {
    console.log(`[Server] RPC: getTodo(${id})`);
    const todo = todos.find(t => t.id === id);
    if (!todo) {
        throw new Error('Todo not found');
    }
    return todo;
}

/**
 * Create new todo
 * @server
 */
async function createTodo(text) {
    console.log(`[Server] RPC: createTodo("${text}")`);

    // Validation (server-side only)
    if (!text || text.trim().length === 0) {
        throw new Error('Todo text cannot be empty');
    }

    if (text.length > 200) {
        throw new Error('Todo text too long (max 200 characters)');
    }

    const todo = {
        id: nextId++,
        text: text.trim(),
        completed: false,
        createdAt: new Date().toISOString(),
    };

    todos.push(todo);

    // Simulate database write
    await server.sleep(50);

    console.log(`[Server] Created todo: ${todo.id}`);
    return todo;
}

/**
 * Update todo
 * @server
 */
async function updateTodo(id, updates) {
    console.log(`[Server] RPC: updateTodo(${id})`, updates);

    const todo = todos.find(t => t.id === id);
    if (!todo) {
        throw new Error('Todo not found');
    }

    // Validation
    if (updates.text !== undefined) {
        if (updates.text.trim().length === 0) {
            throw new Error('Todo text cannot be empty');
        }
        if (updates.text.length > 200) {
            throw new Error('Todo text too long');
        }
        todo.text = updates.text.trim();
    }

    if (updates.completed !== undefined) {
        todo.completed = updates.completed;
    }

    todo.updatedAt = new Date().toISOString();

    // Simulate database write
    await server.sleep(50);

    console.log(`[Server] Updated todo: ${todo.id}`);
    return todo;
}

/**
 * Delete todo
 * @server
 */
async function deleteTodo(id) {
    console.log(`[Server] RPC: deleteTodo(${id})`);

    const index = todos.findIndex(t => t.id === id);
    if (index === -1) {
        throw new Error('Todo not found');
    }

    todos.splice(index, 1);

    // Simulate database write
    await server.sleep(50);

    console.log(`[Server] Deleted todo: ${id}`);
    return true;
}

/**
 * Get statistics
 * @server
 */
async function getStats() {
    console.log('[Server] RPC: getStats');

    return {
        total: todos.length,
        completed: todos.filter(t => t.completed).length,
        pending: todos.filter(t => !t.completed).length,
        timestamp: new Date().toISOString(),
    };
}

/**
 * Clear all completed todos
 * @server
 */
async function clearCompleted() {
    console.log('[Server] RPC: clearCompleted');

    const before = todos.length;
    todos = todos.filter(t => !t.completed);
    const deleted = before - todos.length;

    await server.sleep(50);

    console.log(`[Server] Cleared ${deleted} completed todos`);
    return { deleted };
}

// ==================== Server Setup ====================

async function main() {
    // Initialize server
    await server.init({
        port: 3000,
        host: 'localhost',
    });

    // Load environment variables
    await server.loadEnv();

    // Register RPC handlers
    server.registerRPCHandlers({
        getTodos,
        getTodo,
        createTodo,
        updateTodo,
        deleteTodo,
        getStats,
        clearCompleted,
    });

    // Add CORS middleware
    server.use(async (ctx) => {
        ctx.res.setHeader('Access-Control-Allow-Origin', '*');
        ctx.res.setHeader('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, OPTIONS');
        ctx.res.setHeader('Access-Control-Allow-Headers', 'Content-Type');

        if (ctx.method === 'OPTIONS') {
            ctx.res.writeHead(200);
            ctx.res.end();
        }
    });

    // Add logging middleware
    server.use(async (ctx) => {
        const start = Date.now();
        console.log(`[Server] → ${ctx.method} ${ctx.url}`);

        // Continue to next middleware/handler
        // (In full implementation, this would use proper middleware chain)

        const duration = Date.now() - start;
        console.log(`[Server] ← ${ctx.method} ${ctx.url} (${duration}ms)`);
    });

    // Add REST endpoints (optional, can use RPC or REST)
    server.get('/api/todos', async (ctx) => {
        return await getTodos();
    });

    server.post('/api/todos', async (ctx) => {
        return await createTodo(ctx.body.text);
    });

    server.get('/health', async (ctx) => {
        return {
            status: 'ok',
            timestamp: new Date().toISOString(),
            uptime: process.uptime(),
        };
    });

    // Serve static files
    server.get('/', async (ctx) => {
        try {
            const html = await server.readFile('demo-fullstack-client.html');
            server.sendHTML(ctx.res, html);
            return null; // Don't try to send JSON
        } catch (error) {
            server.sendError(ctx.res, 500, 'Failed to load page');
            return null;
        }
    });

    // Serve dist files
    server.get('/dist/client-runtime.js', async (ctx) => {
        try {
            const js = await server.readFile('dist/client-runtime.js');
            ctx.res.writeHead(200, {
                'Content-Type': 'application/javascript',
                'Access-Control-Allow-Origin': '*',
            });
            ctx.res.end(js);
            return null;
        } catch (error) {
            server.sendError(ctx.res, 404, 'File not found');
            return null;
        }
    });

    // Start server
    await server.listen();

    console.log('');
    console.log('🚀 RavensOne Full-Stack Demo Server');
    console.log('');
    console.log('📡 RPC Endpoint: http://localhost:3000/_rpc');
    console.log('🌐 Client Demo:  http://localhost:3000/');
    console.log('❤️  Health Check: http://localhost:3000/health');
    console.log('');
    console.log('Available RPC Methods:');
    console.log('  - getTodos()');
    console.log('  - getTodo(id)');
    console.log('  - createTodo(text)');
    console.log('  - updateTodo(id, updates)');
    console.log('  - deleteTodo(id)');
    console.log('  - getStats()');
    console.log('  - clearCompleted()');
    console.log('');
}

// Start server
main().catch(console.error);
