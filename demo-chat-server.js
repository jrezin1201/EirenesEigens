/**
 * RavensOne Real-Time Chat Demo - Server
 *
 * Demonstrates WebSocket communication with:
 * - Real-time message broadcasting
 * - Multiple chat rooms
 * - User presence
 * - Typing indicators
 */

const ServerRuntime = require('./dist/server-runtime.js');
const WebSocket = require('ws');

// Create server instance
const server = new ServerRuntime();

// Chat state
const rooms = new Map(); // roomId -> Room
const users = new Map(); // userId -> User
const connections = new Map(); // ws -> userId

class Room {
    constructor(id, name) {
        this.id = id;
        this.name = name;
        this.messages = [];
        this.users = new Set();
        this.typing = new Set();
        this.createdAt = new Date().toISOString();
    }

    addUser(userId) {
        this.users.add(userId);
    }

    removeUser(userId) {
        this.users.delete(userId);
        this.typing.delete(userId);
    }

    addMessage(message) {
        this.messages.push(message);
        // Keep only last 100 messages
        if (this.messages.length > 100) {
            this.messages.shift();
        }
    }

    toJSON() {
        return {
            id: this.id,
            name: this.name,
            userCount: this.users.size,
            messageCount: this.messages.length,
            createdAt: this.createdAt,
        };
    }
}

class User {
    constructor(id, name) {
        this.id = id;
        this.name = name;
        this.currentRoom = 'general';
        this.joinedAt = new Date().toISOString();
        this.lastSeen = new Date().toISOString();
    }

    toJSON() {
        return {
            id: this.id,
            name: this.name,
            currentRoom: this.currentRoom,
            joinedAt: this.joinedAt,
            lastSeen: this.lastSeen,
        };
    }
}

// Initialize default rooms
rooms.set('general', new Room('general', 'General'));
rooms.set('random', new Room('random', 'Random'));
rooms.set('tech', new Room('tech', 'Tech Talk'));

// ==================== RPC Methods ====================

async function getRooms() {
    console.log('[Chat] RPC: getRooms');
    return Array.from(rooms.values()).map(r => r.toJSON());
}

async function getRoom(roomId) {
    console.log(`[Chat] RPC: getRoom(${roomId})`);
    const room = rooms.get(roomId);
    if (!room) {
        throw new Error('Room not found');
    }

    return {
        ...room.toJSON(),
        messages: room.messages,
        users: Array.from(room.users).map(id => users.get(id)?.toJSON()),
    };
}

async function createRoom(name) {
    console.log(`[Chat] RPC: createRoom("${name}")`);

    if (!name || name.trim().length === 0) {
        throw new Error('Room name required');
    }

    const id = name.toLowerCase().replace(/[^a-z0-9]/g, '-');

    if (rooms.has(id)) {
        throw new Error('Room already exists');
    }

    const room = new Room(id, name);
    rooms.set(id, room);

    console.log(`[Chat] Created room: ${id}`);
    return room.toJSON();
}

async function getUsers() {
    console.log('[Chat] RPC: getUsers');
    return Array.from(users.values()).map(u => u.toJSON());
}

async function getStats() {
    console.log('[Chat] RPC: getStats');

    let totalMessages = 0;
    for (const room of rooms.values()) {
        totalMessages += room.messages.length;
    }

    return {
        rooms: rooms.size,
        users: users.size,
        messages: totalMessages,
        timestamp: new Date().toISOString(),
    };
}

// ==================== WebSocket Handler ====================

function setupWebSocket(httpServer) {
    const wss = new WebSocket.Server({ server: httpServer, path: '/ws' });

    console.log('[Chat] WebSocket server initialized on /ws');

    wss.on('connection', (ws) => {
        console.log('[Chat] New WebSocket connection');

        let userId = null;

        ws.on('message', (data) => {
            try {
                const message = JSON.parse(data.toString());
                handleWebSocketMessage(ws, message);
            } catch (error) {
                console.error('[Chat] WebSocket message error:', error);
                sendToClient(ws, {
                    type: 'error',
                    error: error.message,
                });
            }
        });

        ws.on('close', () => {
            console.log(`[Chat] WebSocket closed: ${userId}`);
            if (userId) {
                handleUserDisconnect(userId);
                connections.delete(ws);
            }
        });

        ws.on('error', (error) => {
            console.error('[Chat] WebSocket error:', error);
        });
    });

    return wss;
}

function handleWebSocketMessage(ws, message) {
    const { type, payload } = message;

    switch (type) {
        case 'join':
            handleJoin(ws, payload);
            break;

        case 'message':
            handleMessage(ws, payload);
            break;

        case 'typing':
            handleTyping(ws, payload);
            break;

        case 'changeRoom':
            handleChangeRoom(ws, payload);
            break;

        default:
            console.warn('[Chat] Unknown message type:', type);
    }
}

function handleJoin(ws, payload) {
    const { userId, userName } = payload;

    console.log(`[Chat] User joining: ${userName} (${userId})`);

    // Create or update user
    let user = users.get(userId);
    if (!user) {
        user = new User(userId, userName);
        users.set(userId, user);
    } else {
        user.name = userName;
        user.lastSeen = new Date().toISOString();
    }

    // Store connection
    connections.set(ws, userId);

    // Add to general room
    const room = rooms.get('general');
    room.addUser(userId);
    user.currentRoom = 'general';

    // Send welcome message
    sendToClient(ws, {
        type: 'welcome',
        user: user.toJSON(),
        room: {
            ...room.toJSON(),
            messages: room.messages,
        },
    });

    // Broadcast user joined
    broadcastToRoom('general', {
        type: 'userJoined',
        user: user.toJSON(),
    }, userId);

    console.log(`[Chat] User joined: ${userName} -> ${room.name}`);
}

function handleMessage(ws, payload) {
    const userId = connections.get(ws);
    if (!userId) {
        return;
    }

    const user = users.get(userId);
    if (!user) {
        return;
    }

    const { text } = payload;
    const roomId = user.currentRoom;
    const room = rooms.get(roomId);

    if (!room) {
        return;
    }

    const message = {
        id: Date.now() + Math.random(),
        userId: user.id,
        userName: user.name,
        text: text.trim(),
        roomId,
        timestamp: new Date().toISOString(),
    };

    room.addMessage(message);

    console.log(`[Chat] Message in ${roomId}: ${user.name}: ${text.substring(0, 50)}`);

    // Broadcast message to room
    broadcastToRoom(roomId, {
        type: 'message',
        message,
    });

    // Clear typing indicator
    if (room.typing.has(userId)) {
        room.typing.delete(userId);
        broadcastToRoom(roomId, {
            type: 'typing',
            userId,
            isTyping: false,
        }, userId);
    }
}

function handleTyping(ws, payload) {
    const userId = connections.get(ws);
    if (!userId) {
        return;
    }

    const user = users.get(userId);
    if (!user) {
        return;
    }

    const { isTyping } = payload;
    const roomId = user.currentRoom;
    const room = rooms.get(roomId);

    if (!room) {
        return;
    }

    if (isTyping) {
        room.typing.add(userId);
    } else {
        room.typing.delete(userId);
    }

    // Broadcast typing indicator
    broadcastToRoom(roomId, {
        type: 'typing',
        userId,
        userName: user.name,
        isTyping,
    }, userId);
}

function handleChangeRoom(ws, payload) {
    const userId = connections.get(ws);
    if (!userId) {
        return;
    }

    const user = users.get(userId);
    if (!user) {
        return;
    }

    const { roomId } = payload;
    const newRoom = rooms.get(roomId);

    if (!newRoom) {
        sendToClient(ws, {
            type: 'error',
            error: 'Room not found',
        });
        return;
    }

    // Leave old room
    const oldRoomId = user.currentRoom;
    const oldRoom = rooms.get(oldRoomId);
    if (oldRoom) {
        oldRoom.removeUser(userId);
        broadcastToRoom(oldRoomId, {
            type: 'userLeft',
            user: user.toJSON(),
        }, userId);
    }

    // Join new room
    newRoom.addUser(userId);
    user.currentRoom = roomId;

    // Send room info
    sendToClient(ws, {
        type: 'roomChanged',
        room: {
            ...newRoom.toJSON(),
            messages: newRoom.messages,
        },
    });

    // Broadcast user joined
    broadcastToRoom(roomId, {
        type: 'userJoined',
        user: user.toJSON(),
    }, userId);

    console.log(`[Chat] User ${user.name} moved to ${newRoom.name}`);
}

function handleUserDisconnect(userId) {
    const user = users.get(userId);
    if (!user) {
        return;
    }

    const roomId = user.currentRoom;
    const room = rooms.get(roomId);

    if (room) {
        room.removeUser(userId);
        broadcastToRoom(roomId, {
            type: 'userLeft',
            user: user.toJSON(),
        });
    }

    users.delete(userId);
    console.log(`[Chat] User disconnected: ${user.name}`);
}

// ==================== Helper Functions ====================

function sendToClient(ws, message) {
    if (ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify(message));
    }
}

function broadcastToRoom(roomId, message, excludeUserId = null) {
    const room = rooms.get(roomId);
    if (!room) {
        return;
    }

    for (const [ws, userId] of connections.entries()) {
        if (excludeUserId && userId === excludeUserId) {
            continue;
        }

        const user = users.get(userId);
        if (user && user.currentRoom === roomId) {
            sendToClient(ws, message);
        }
    }
}

// ==================== Server Setup ====================

async function main() {
    // Initialize server
    await server.init({
        port: 3001,
        host: 'localhost',
    });

    // Register RPC handlers
    server.registerRPCHandlers({
        getRooms,
        getRoom,
        createRoom,
        getUsers,
        getStats,
    });

    // Add CORS middleware
    server.use(async (ctx) => {
        ctx.res.setHeader('Access-Control-Allow-Origin', '*');
        ctx.res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
        ctx.res.setHeader('Access-Control-Allow-Headers', 'Content-Type');

        if (ctx.method === 'OPTIONS') {
            ctx.res.writeHead(200);
            ctx.res.end();
        }
    });

    // Serve client
    server.get('/', async (ctx) => {
        try {
            const html = await server.readFile('demo-chat-client.html');
            server.sendHTML(ctx.res, html);
            return null;
        } catch (error) {
            server.sendError(ctx.res, 500, 'Failed to load page');
            return null;
        }
    });

    // Serve static files
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

    // Start HTTP server
    const httpServer = await server.listen();

    // Setup WebSocket server
    setupWebSocket(httpServer);

    console.log('');
    console.log('💬 RavensOne Real-Time Chat Server');
    console.log('');
    console.log('🌐 Client:  http://localhost:3001/');
    console.log('📡 RPC:     http://localhost:3001/_rpc');
    console.log('🔌 WebSocket: ws://localhost:3001/ws');
    console.log('');
    console.log('Default Rooms:');
    console.log('  - General');
    console.log('  - Random');
    console.log('  - Tech Talk');
    console.log('');
}

// Start server
main().catch(console.error);
