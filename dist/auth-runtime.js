/**
 * RavensOne Authentication Runtime
 *
 * Provides authentication and authorization for WASM modules
 * Handles login, signup, JWT tokens, and sessions
 */

class AuthRuntime {
    constructor() {
        this.currentUser = null;
        this.token = null;
        this.sessions = new Map();
        this.users = new Map(); // In-memory user store for demo
        this.userIdCounter = 1;

        // Load from localStorage if available
        this.loadFromStorage();
    }

    /**
     * Initialize with database connection
     */
    async init(db) {
        this.db = db;
        console.log('[Auth] Runtime initialized');
        return this;
    }

    /**
     * Register new user
     */
    async signup(email, password, name) {
        console.log(`[Auth] Signup: ${email}`);

        // Check if user already exists
        if (this.findUserByEmail(email)) {
            throw new Error('User already exists');
        }

        // Hash password (simplified for demo)
        const passwordHash = await this.hashPassword(password);

        // Create user
        const user = {
            id: this.userIdCounter++,
            email,
            name,
            passwordHash,
            role: 'user',
            createdAt: new Date().toISOString(),
            lastLogin: null,
        };

        this.users.set(user.id, user);
        this.users.set(email, user); // Also store by email for lookup

        console.log(`[Auth] User created:`, user.id);

        // Auto-login
        return this.login(email, password);
    }

    /**
     * Login user
     */
    async login(email, password) {
        console.log(`[Auth] Login attempt: ${email}`);

        const user = this.findUserByEmail(email);
        if (!user) {
            throw new Error('Invalid credentials');
        }

        // Verify password
        const valid = await this.verifyPassword(password, user.passwordHash);
        if (!valid) {
            throw new Error('Invalid credentials');
        }

        // Update last login
        user.lastLogin = new Date().toISOString();

        // Generate token
        const token = this.generateToken(user);

        // Create session
        const sessionId = this.generateSessionId();
        const session = {
            sessionId,
            userId: user.id,
            createdAt: Date.now(),
            expiresAt: Date.now() + (7 * 24 * 60 * 60 * 1000), // 7 days
            data: {},
        };

        this.sessions.set(sessionId, session);

        // Set current user
        this.currentUser = this.toSafeUser(user);
        this.token = token;

        // Save to storage
        this.saveToStorage();

        console.log(`[Auth] Login successful:`, user.id);

        return {
            token,
            tokenType: 'Bearer',
            expiresIn: 24 * 60 * 60, // 24 hours
            user: this.currentUser,
        };
    }

    /**
     * Logout current user
     */
    async logout() {
        console.log('[Auth] Logout');

        this.currentUser = null;
        this.token = null;

        // Clear storage
        this.clearStorage();

        return true;
    }

    /**
     * Get current user
     */
    getCurrentUser() {
        return this.currentUser;
    }

    /**
     * Check if user is authenticated
     */
    isAuthenticated() {
        return this.currentUser !== null && this.token !== null;
    }

    /**
     * Check if user has role
     */
    hasRole(role) {
        if (!this.currentUser) {
            return false;
        }

        if (this.currentUser.role === 'admin') {
            return true; // Admin has all roles
        }

        return this.currentUser.role === role;
    }

    /**
     * Require authentication
     */
    requireAuth() {
        if (!this.isAuthenticated()) {
            throw new Error('Authentication required');
        }
    }

    /**
     * Require specific role
     */
    requireRole(role) {
        this.requireAuth();

        if (!this.hasRole(role)) {
            throw new Error(`Role '${role}' required`);
        }
    }

    /**
     * Verify token
     */
    async verifyToken(token) {
        // Simplified token verification for demo
        if (!token || !token.startsWith('jwt.')) {
            throw new Error('Invalid token');
        }

        return true;
    }

    /**
     * Find user by email
     */
    findUserByEmail(email) {
        return this.users.get(email);
    }

    /**
     * Find user by ID
     */
    findUserById(id) {
        return this.users.get(id);
    }

    /**
     * Hash password
     */
    async hashPassword(password) {
        // In production, use bcrypt or argon2
        // For demo, simple hash
        return `$bcrypt$hash$${password}`;
    }

    /**
     * Verify password
     */
    async verifyPassword(password, hash) {
        // In production, use bcrypt.compare
        // For demo, simple comparison
        return hash === `$bcrypt$hash$${password}`;
    }

    /**
     * Generate JWT token
     */
    generateToken(user) {
        // In production, use proper JWT library
        // For demo, simple token
        const payload = {
            sub: user.id,
            email: user.email,
            role: user.role,
            iat: Math.floor(Date.now() / 1000),
            exp: Math.floor(Date.now() / 1000) + (24 * 60 * 60), // 24 hours
        };

        return `jwt.${btoa(JSON.stringify(payload))}.signature`;
    }

    /**
     * Generate session ID
     */
    generateSessionId() {
        return `session_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    }

    /**
     * Convert user to safe user (no password)
     */
    toSafeUser(user) {
        return {
            id: user.id,
            email: user.email,
            name: user.name,
            role: user.role,
            createdAt: user.createdAt,
            lastLogin: user.lastLogin,
        };
    }

    /**
     * Save auth state to localStorage
     */
    saveToStorage() {
        if (typeof localStorage === 'undefined') return;

        try {
            localStorage.setItem('ravens_auth_token', this.token);
            localStorage.setItem('ravens_auth_user', JSON.stringify(this.currentUser));
            console.log('[Auth] State saved to storage');
        } catch (e) {
            console.warn('[Auth] Failed to save to storage:', e);
        }
    }

    /**
     * Load auth state from localStorage
     */
    loadFromStorage() {
        if (typeof localStorage === 'undefined') return;

        try {
            const token = localStorage.getItem('ravens_auth_token');
            const userJson = localStorage.getItem('ravens_auth_user');

            if (token && userJson) {
                this.token = token;
                this.currentUser = JSON.parse(userJson);
                console.log('[Auth] State loaded from storage');
            }
        } catch (e) {
            console.warn('[Auth] Failed to load from storage:', e);
        }
    }

    /**
     * Clear storage
     */
    clearStorage() {
        if (typeof localStorage === 'undefined') return;

        try {
            localStorage.removeItem('ravens_auth_token');
            localStorage.removeItem('ravens_auth_user');
            console.log('[Auth] Storage cleared');
        } catch (e) {
            console.warn('[Auth] Failed to clear storage:', e);
        }
    }

    /**
     * Seed demo users
     */
    seedDemoUsers() {
        const demoUsers = [
            {
                email: 'admin@ravens.one',
                password: 'admin123',
                name: 'Admin User',
                role: 'admin',
            },
            {
                email: 'user@ravens.one',
                password: 'user123',
                name: 'Regular User',
                role: 'user',
            },
        ];

        for (const demo of demoUsers) {
            const user = {
                id: this.userIdCounter++,
                email: demo.email,
                name: demo.name,
                passwordHash: `$bcrypt$hash$${demo.password}`,
                role: demo.role,
                createdAt: new Date().toISOString(),
                lastLogin: null,
            };

            this.users.set(user.id, user);
            this.users.set(user.email, user);
        }

        console.log('[Auth] Demo users seeded');
        console.log('  - admin@ravens.one / admin123');
        console.log('  - user@ravens.one / user123');
    }
}

// Global auth runtime instance
if (typeof window !== 'undefined') {
    window.RavensAuth = new AuthRuntime();
    console.log('[Auth] Runtime initialized');
}

// Export for Node.js
if (typeof module !== 'undefined' && module.exports) {
    module.exports = AuthRuntime;
}
