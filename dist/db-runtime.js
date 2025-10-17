/**
 * RavensOne Database Runtime
 *
 * Provides database operations for WASM modules
 * Uses SQLite (via sql.js) in browser, native SQLite/PostgreSQL on server
 */

class DatabaseRuntime {
    constructor() {
        this.db = null;
        this.tables = new Map();
        this.queryLog = [];
    }

    /**
     * Initialize database with schema
     */
    async init(schema) {
        console.log('[DB] Initializing database...');

        // In browser, use sql.js (for demo)
        // In production, this would connect to real database
        this.db = this.createInMemoryDB();

        // Create tables from schema
        if (schema && schema.tables) {
            for (const table of schema.tables) {
                await this.createTable(table);
            }
        }

        console.log('[DB] Database initialized');
        return this;
    }

    /**
     * Create in-memory database (for demo)
     */
    createInMemoryDB() {
        return {
            tables: new Map(),
            autoIncrementCounters: new Map(),
        };
    }

    /**
     * Create table from schema
     */
    async createTable(schema) {
        console.log(`[DB] Creating table: ${schema.name}`);

        this.db.tables.set(schema.name, {
            name: schema.name,
            columns: schema.columns,
            rows: [],
            indexes: new Map(),
        });

        this.db.autoIncrementCounters.set(schema.name, 1);
        this.tables.set(schema.name, schema);

        this.logQuery(`CREATE TABLE ${schema.name}`);
    }

    /**
     * Execute SELECT query
     */
    async select(tableName, conditions = {}, options = {}) {
        console.log(`[DB] SELECT from ${tableName}`, conditions, options);

        const table = this.db.tables.get(tableName);
        if (!table) {
            throw new Error(`Table ${tableName} not found`);
        }

        let rows = table.rows;

        // Apply WHERE conditions
        if (Object.keys(conditions).length > 0) {
            rows = rows.filter(row => {
                return Object.entries(conditions).every(([key, value]) => {
                    return row[key] === value;
                });
            });
        }

        // Apply ORDER BY
        if (options.orderBy) {
            const { column, direction } = options.orderBy;
            rows = [...rows].sort((a, b) => {
                if (direction === 'DESC') {
                    return b[column] > a[column] ? 1 : -1;
                }
                return a[column] > b[column] ? 1 : -1;
            });
        }

        // Apply LIMIT
        if (options.limit) {
            rows = rows.slice(0, options.limit);
        }

        // Apply OFFSET
        if (options.offset) {
            rows = rows.slice(options.offset);
        }

        this.logQuery(`SELECT * FROM ${tableName}`, { conditions, options });

        return rows;
    }

    /**
     * Execute INSERT query
     */
    async insert(tableName, data) {
        console.log(`[DB] INSERT into ${tableName}`, data);

        const table = this.db.tables.get(tableName);
        if (!table) {
            throw new Error(`Table ${tableName} not found`);
        }

        // Auto-increment ID if needed
        const schema = this.tables.get(tableName);
        const idColumn = schema.columns.find(c => c.primary_key);

        if (idColumn && idColumn.auto_increment && !data[idColumn.name]) {
            const counter = this.db.autoIncrementCounters.get(tableName);
            data[idColumn.name] = counter;
            this.db.autoIncrementCounters.set(tableName, counter + 1);
        }

        // Add timestamps if columns exist
        if (schema.columns.find(c => c.name === 'created_at')) {
            data.created_at = new Date().toISOString();
        }
        if (schema.columns.find(c => c.name === 'updated_at')) {
            data.updated_at = new Date().toISOString();
        }

        table.rows.push({ ...data });

        this.logQuery(`INSERT INTO ${tableName}`, data);

        return data;
    }

    /**
     * Execute UPDATE query
     */
    async update(tableName, conditions, data) {
        console.log(`[DB] UPDATE ${tableName}`, conditions, data);

        const table = this.db.tables.get(tableName);
        if (!table) {
            throw new Error(`Table ${tableName} not found`);
        }

        // Update timestamp
        const schema = this.tables.get(tableName);
        if (schema.columns.find(c => c.name === 'updated_at')) {
            data.updated_at = new Date().toISOString();
        }

        let updated = 0;
        table.rows = table.rows.map(row => {
            const matches = Object.entries(conditions).every(([key, value]) => {
                return row[key] === value;
            });

            if (matches) {
                updated++;
                return { ...row, ...data };
            }
            return row;
        });

        this.logQuery(`UPDATE ${tableName}`, { conditions, data });

        return updated;
    }

    /**
     * Execute DELETE query
     */
    async delete(tableName, conditions) {
        console.log(`[DB] DELETE from ${tableName}`, conditions);

        const table = this.db.tables.get(tableName);
        if (!table) {
            throw new Error(`Table ${tableName} not found`);
        }

        const before = table.rows.length;
        table.rows = table.rows.filter(row => {
            return !Object.entries(conditions).every(([key, value]) => {
                return row[key] === value;
            });
        });
        const deleted = before - table.rows.length;

        this.logQuery(`DELETE FROM ${tableName}`, conditions);

        return deleted;
    }

    /**
     * Count rows
     */
    async count(tableName, conditions = {}) {
        const rows = await this.select(tableName, conditions);
        return rows.length;
    }

    /**
     * Find one record
     */
    async findOne(tableName, conditions) {
        const rows = await this.select(tableName, conditions, { limit: 1 });
        return rows[0] || null;
    }

    /**
     * Find by ID
     */
    async findById(tableName, id) {
        const idColumn = this.tables.get(tableName).columns.find(c => c.primary_key);
        return this.findOne(tableName, { [idColumn.name]: id });
    }

    /**
     * Get all records
     */
    async all(tableName) {
        return this.select(tableName);
    }

    /**
     * Log query for debugging
     */
    logQuery(query, params = null) {
        const entry = {
            query,
            params,
            timestamp: new Date().toISOString(),
        };
        this.queryLog.push(entry);
        console.log(`[DB Query]`, query, params);
    }

    /**
     * Get query log
     */
    getQueryLog() {
        return this.queryLog;
    }

    /**
     * Clear query log
     */
    clearQueryLog() {
        this.queryLog = [];
    }

    /**
     * Get table stats
     */
    async getStats() {
        const stats = {};

        for (const [tableName, table] of this.db.tables) {
            stats[tableName] = {
                rows: table.rows.length,
                columns: table.columns.length,
            };
        }

        return stats;
    }

    /**
     * Export database as JSON
     */
    exportJSON() {
        const data = {};

        for (const [tableName, table] of this.db.tables) {
            data[tableName] = table.rows;
        }

        return JSON.stringify(data, null, 2);
    }

    /**
     * Import database from JSON
     */
    async importJSON(jsonData) {
        const data = JSON.parse(jsonData);

        for (const [tableName, rows] of Object.entries(data)) {
            const table = this.db.tables.get(tableName);
            if (table) {
                table.rows = rows;

                // Update auto-increment counter
                const schema = this.tables.get(tableName);
                const idColumn = schema.columns.find(c => c.primary_key && c.auto_increment);
                if (idColumn) {
                    const maxId = Math.max(...rows.map(r => r[idColumn.name] || 0));
                    this.db.autoIncrementCounters.set(tableName, maxId + 1);
                }
            }
        }

        console.log('[DB] Imported data');
    }
}

// Global database runtime instance
if (typeof window !== 'undefined') {
    window.RavensDB = new DatabaseRuntime();
    console.log('[DB] Runtime initialized');
}

// Export for Node.js
if (typeof module !== 'undefined' && module.exports) {
    module.exports = DatabaseRuntime;
}
