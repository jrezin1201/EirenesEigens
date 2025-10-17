/**
 * RavensOne Client Runtime
 *
 * Provides client-side APIs for browser
 * - DOM manipulation
 * - RPC calls to server
 * - Browser storage
 * - Client-side routing
 */

class ClientRuntime {
    constructor() {
        this.rpcEndpoint = '/_rpc';
        this.rpcId = 1;
        this.serverUrl = '';
    }

    /**
     * Initialize client runtime
     */
    async init(config = {}) {
        this.serverUrl = config.serverUrl || '';
        this.rpcEndpoint = config.rpcEndpoint || '/_rpc';
        console.log('[Client] Runtime initialized');
        console.log(`[Client] Server: ${this.serverUrl || 'same-origin'}`);
        return this;
    }

    // ==================== RPC ====================

    /**
     * Call server RPC method
     */
    async call(method, params = []) {
        console.log(`[Client] RPC call: ${method}`, params);

        const id = this.rpcId++;
        const url = `${this.serverUrl}${this.rpcEndpoint}`;

        try {
            const response = await fetch(url, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    jsonrpc: '2.0',
                    method,
                    params,
                    id,
                }),
            });

            if (!response.ok) {
                throw new Error(`RPC request failed: ${response.status} ${response.statusText}`);
            }

            const data = await response.json();

            if (data.error) {
                throw new Error(data.error.message || 'RPC error');
            }

            console.log(`[Client] RPC result: ${method}`, data.result);
            return data.result;
        } catch (error) {
            console.error(`[Client] RPC error: ${method}`, error);
            throw error;
        }
    }

    /**
     * Batch RPC calls
     */
    async callBatch(calls) {
        console.log(`[Client] RPC batch: ${calls.length} calls`);

        const url = `${this.serverUrl}${this.rpcEndpoint}`;
        const requests = calls.map(({ method, params }) => ({
            jsonrpc: '2.0',
            method,
            params: params || [],
            id: this.rpcId++,
        }));

        try {
            const response = await fetch(url, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(requests),
            });

            if (!response.ok) {
                throw new Error(`RPC batch failed: ${response.status}`);
            }

            const results = await response.json();
            return results.map(r => r.error ? { error: r.error } : r.result);
        } catch (error) {
            console.error('[Client] RPC batch error:', error);
            throw error;
        }
    }

    // ==================== DOM ====================

    /**
     * Select element
     */
    select(selector) {
        return document.querySelector(selector);
    }

    /**
     * Select all elements
     */
    selectAll(selector) {
        return Array.from(document.querySelectorAll(selector));
    }

    /**
     * Create element
     */
    createElement(tag, props = {}, children = []) {
        const element = document.createElement(tag);

        // Set properties
        for (const [key, value] of Object.entries(props)) {
            if (key === 'className') {
                element.className = value;
            } else if (key === 'style' && typeof value === 'object') {
                Object.assign(element.style, value);
            } else if (key.startsWith('on') && typeof value === 'function') {
                const event = key.substring(2).toLowerCase();
                element.addEventListener(event, value);
            } else {
                element.setAttribute(key, value);
            }
        }

        // Add children
        for (const child of children) {
            if (typeof child === 'string') {
                element.appendChild(document.createTextNode(child));
            } else if (child instanceof Node) {
                element.appendChild(child);
            }
        }

        return element;
    }

    /**
     * Mount component
     */
    mount(selector, element) {
        const container = typeof selector === 'string'
            ? this.select(selector)
            : selector;

        if (!container) {
            throw new Error(`Mount target not found: ${selector}`);
        }

        container.innerHTML = '';
        container.appendChild(element);
    }

    /**
     * Add event listener
     */
    on(selector, event, handler) {
        const element = typeof selector === 'string'
            ? this.select(selector)
            : selector;

        if (element) {
            element.addEventListener(event, handler);
        }
    }

    /**
     * Remove event listener
     */
    off(selector, event, handler) {
        const element = typeof selector === 'string'
            ? this.select(selector)
            : selector;

        if (element) {
            element.removeEventListener(event, handler);
        }
    }

    // ==================== Storage ====================

    /**
     * Get from localStorage
     */
    getLocal(key, defaultValue = null) {
        try {
            const value = localStorage.getItem(key);
            return value !== null ? JSON.parse(value) : defaultValue;
        } catch (error) {
            console.error('[Client] localStorage get error:', error);
            return defaultValue;
        }
    }

    /**
     * Set to localStorage
     */
    setLocal(key, value) {
        try {
            localStorage.setItem(key, JSON.stringify(value));
            return true;
        } catch (error) {
            console.error('[Client] localStorage set error:', error);
            return false;
        }
    }

    /**
     * Remove from localStorage
     */
    removeLocal(key) {
        try {
            localStorage.removeItem(key);
            return true;
        } catch (error) {
            console.error('[Client] localStorage remove error:', error);
            return false;
        }
    }

    /**
     * Clear localStorage
     */
    clearLocal() {
        try {
            localStorage.clear();
            return true;
        } catch (error) {
            console.error('[Client] localStorage clear error:', error);
            return false;
        }
    }

    /**
     * Get from sessionStorage
     */
    getSession(key, defaultValue = null) {
        try {
            const value = sessionStorage.getItem(key);
            return value !== null ? JSON.parse(value) : defaultValue;
        } catch (error) {
            console.error('[Client] sessionStorage get error:', error);
            return defaultValue;
        }
    }

    /**
     * Set to sessionStorage
     */
    setSession(key, value) {
        try {
            sessionStorage.setItem(key, JSON.stringify(value));
            return true;
        } catch (error) {
            console.error('[Client] sessionStorage set error:', error);
            return false;
        }
    }

    // ==================== Routing ====================

    /**
     * Navigate to URL
     */
    navigate(url) {
        window.history.pushState({}, '', url);
        window.dispatchEvent(new PopStateEvent('popstate'));
    }

    /**
     * Get current route
     */
    getRoute() {
        return window.location.pathname;
    }

    /**
     * Get query params
     */
    getQuery() {
        return Object.fromEntries(new URLSearchParams(window.location.search));
    }

    /**
     * Listen to route changes
     */
    onRoute(handler) {
        window.addEventListener('popstate', () => {
            handler(this.getRoute(), this.getQuery());
        });

        // Call immediately
        handler(this.getRoute(), this.getQuery());
    }

    // ==================== Utilities ====================

    /**
     * Wait for DOM ready
     */
    async ready() {
        return new Promise((resolve) => {
            if (document.readyState === 'loading') {
                document.addEventListener('DOMContentLoaded', resolve);
            } else {
                resolve();
            }
        });
    }

    /**
     * Debounce function
     */
    debounce(func, wait) {
        let timeout;
        return function executedFunction(...args) {
            const later = () => {
                clearTimeout(timeout);
                func(...args);
            };
            clearTimeout(timeout);
            timeout = setTimeout(later, wait);
        };
    }

    /**
     * Throttle function
     */
    throttle(func, limit) {
        let inThrottle;
        return function(...args) {
            if (!inThrottle) {
                func.apply(this, args);
                inThrottle = true;
                setTimeout(() => inThrottle = false, limit);
            }
        };
    }

    /**
     * Show notification
     */
    notify(message, type = 'info', duration = 3000) {
        const notification = this.createElement('div', {
            className: `ravens-notification ravens-notification-${type}`,
            style: {
                position: 'fixed',
                top: '20px',
                right: '20px',
                padding: '15px 20px',
                background: type === 'error' ? '#f44336' : type === 'success' ? '#4caf50' : '#2196f3',
                color: 'white',
                borderRadius: '8px',
                boxShadow: '0 4px 12px rgba(0,0,0,0.2)',
                zIndex: '10000',
                animation: 'slideIn 0.3s ease',
            }
        }, [message]);

        document.body.appendChild(notification);

        setTimeout(() => {
            notification.style.animation = 'slideOut 0.3s ease';
            setTimeout(() => notification.remove(), 300);
        }, duration);
    }

    /**
     * Show loading indicator
     */
    showLoading(message = 'Loading...') {
        const loader = this.createElement('div', {
            id: 'ravens-loader',
            style: {
                position: 'fixed',
                top: '0',
                left: '0',
                width: '100%',
                height: '100%',
                background: 'rgba(0,0,0,0.5)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                zIndex: '9999',
            }
        }, [
            this.createElement('div', {
                style: {
                    background: 'white',
                    padding: '30px',
                    borderRadius: '15px',
                    textAlign: 'center',
                }
            }, [message])
        ]);

        document.body.appendChild(loader);
    }

    /**
     * Hide loading indicator
     */
    hideLoading() {
        const loader = document.getElementById('ravens-loader');
        if (loader) {
            loader.remove();
        }
    }

    /**
     * Format date
     */
    formatDate(date, format = 'short') {
        const d = typeof date === 'string' ? new Date(date) : date;

        if (format === 'short') {
            return d.toLocaleDateString();
        } else if (format === 'long') {
            return d.toLocaleString();
        } else if (format === 'time') {
            return d.toLocaleTimeString();
        }

        return d.toString();
    }

    /**
     * Copy to clipboard
     */
    async copyToClipboard(text) {
        try {
            await navigator.clipboard.writeText(text);
            this.notify('Copied to clipboard!', 'success', 1500);
            return true;
        } catch (error) {
            console.error('[Client] Clipboard error:', error);
            this.notify('Failed to copy', 'error', 1500);
            return false;
        }
    }
}

// Global client runtime instance
if (typeof window !== 'undefined') {
    window.RavensClient = new ClientRuntime();
    console.log('[Client] Runtime initialized');
}

// Export for module systems
if (typeof module !== 'undefined' && module.exports) {
    module.exports = ClientRuntime;
}
