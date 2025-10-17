/**
 * RavensOne Hot Module Reloading (HMR) Client
 *
 * Connects to dev server via WebSocket
 * Automatically reloads when files change
 * Preserves component state when possible
 */

class HMRClient {
    constructor() {
        this.ws = null;
        this.retryCount = 0;
        this.maxRetries = 10;
        this.retryDelay = 1000;
        this.connected = false;
        this.statePreservers = new Map();
        this.lastUpdate = Date.now();
        this.updateQueue = [];
    }

    /**
     * Connect to HMR server
     */
    connect(port = 3002) {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            return;
        }

        const url = `ws://localhost:${port}/hmr`;
        console.log('[HMR] Connecting to', url);

        try {
            this.ws = new WebSocket(url);

            this.ws.onopen = () => {
                this.connected = true;
                this.retryCount = 0;
                console.log('[HMR] 🔥 Connected - Hot reloading enabled');
                this.showNotification('Hot Reloading Active', 'success');
            };

            this.ws.onmessage = (event) => {
                this.handleMessage(JSON.parse(event.data));
            };

            this.ws.onclose = () => {
                this.connected = false;
                console.log('[HMR] Disconnected');
                this.reconnect(port);
            };

            this.ws.onerror = (error) => {
                console.error('[HMR] Error:', error);
            };
        } catch (error) {
            console.error('[HMR] Connection error:', error);
            this.reconnect(port);
        }
    }

    /**
     * Reconnect to server
     */
    reconnect(port) {
        if (this.retryCount >= this.maxRetries) {
            console.log('[HMR] Max retries reached. Please restart dev server.');
            return;
        }

        this.retryCount++;
        const delay = this.retryDelay * this.retryCount;

        console.log(`[HMR] Reconnecting in ${delay}ms... (${this.retryCount}/${this.maxRetries})`);

        setTimeout(() => {
            this.connect(port);
        }, delay);
    }

    /**
     * Handle server messages
     */
    handleMessage(message) {
        const { type, data } = message;

        console.log('[HMR] Message:', type, data);

        switch (type) {
            case 'connected':
                console.log('[HMR] Server version:', data.version);
                break;

            case 'update':
                this.handleUpdate(data);
                break;

            case 'full-reload':
                this.fullReload(data.reason);
                break;

            case 'error':
                this.handleError(data);
                break;

            case 'ping':
                this.send({ type: 'pong' });
                break;

            default:
                console.warn('[HMR] Unknown message type:', type);
        }
    }

    /**
     * Handle file update
     */
    async handleUpdate(data) {
        const { file, type: updateType, content } = data;

        console.log(`[HMR] 🔄 Updating ${file} (${updateType})`);

        this.showNotification(`Updating ${file}...`, 'info');

        try {
            switch (updateType) {
                case 'css':
                    await this.updateCSS(file, content);
                    break;

                case 'js':
                case 'wasm':
                    await this.updateModule(file, content);
                    break;

                case 'html':
                    this.fullReload('HTML changed');
                    break;

                default:
                    console.warn('[HMR] Unknown update type:', updateType);
            }

            this.showNotification(`Updated ${file}`, 'success');
            this.lastUpdate = Date.now();

        } catch (error) {
            console.error('[HMR] Update failed:', error);
            this.showNotification(`Update failed: ${error.message}`, 'error');
        }
    }

    /**
     * Update CSS without page reload
     */
    async updateCSS(file, content) {
        const links = document.querySelectorAll('link[rel="stylesheet"]');
        let updated = false;

        for (const link of links) {
            if (link.href.includes(file)) {
                // Replace stylesheet
                const newLink = link.cloneNode();
                newLink.href = link.href.split('?')[0] + '?t=' + Date.now();
                link.parentNode.insertBefore(newLink, link.nextSibling);
                link.remove();
                updated = true;
            }
        }

        if (!updated && content) {
            // Inject new style
            const style = document.createElement('style');
            style.textContent = content;
            style.setAttribute('data-hmr-file', file);
            document.head.appendChild(style);
        }

        console.log('[HMR] ✅ CSS updated');
    }

    /**
     * Update JavaScript/WASM module
     */
    async updateModule(file, content) {
        // Preserve component state if possible
        this.preserveState();

        // For now, do a full reload for JS/WASM changes
        // In the future, we could hot-swap modules
        this.fullReload('Module updated');
    }

    /**
     * Preserve component state before reload
     */
    preserveState() {
        // Try to preserve state from components
        if (window.RavensComponents) {
            // Store component state
            const state = {};

            // Find all mounted components
            document.querySelectorAll('[data-component-id]').forEach(el => {
                const id = el.getAttribute('data-component-id');
                const component = el.__component_instance__;

                if (component && component.state) {
                    state[id] = component.state;
                }
            });

            if (Object.keys(state).length > 0) {
                sessionStorage.setItem('hmr_state', JSON.stringify(state));
                console.log('[HMR] 💾 Preserved state for', Object.keys(state).length, 'components');
            }
        }

        // Store scroll position
        sessionStorage.setItem('hmr_scroll', JSON.stringify({
            x: window.scrollX,
            y: window.scrollY
        }));

        // Store form values
        const formData = {};
        document.querySelectorAll('input, textarea, select').forEach(input => {
            if (input.name || input.id) {
                formData[input.name || input.id] = input.value;
            }
        });

        if (Object.keys(formData).length > 0) {
            sessionStorage.setItem('hmr_forms', JSON.stringify(formData));
        }
    }

    /**
     * Restore state after reload
     */
    restoreState() {
        // Restore component state
        const savedState = sessionStorage.getItem('hmr_state');
        if (savedState) {
            try {
                const state = JSON.parse(savedState);
                // Components will pick this up on mount
                window.__HMR_STATE__ = state;
                sessionStorage.removeItem('hmr_state');
                console.log('[HMR] 🔄 Restored state');
            } catch (error) {
                console.error('[HMR] Failed to restore state:', error);
            }
        }

        // Restore scroll position
        const savedScroll = sessionStorage.getItem('hmr_scroll');
        if (savedScroll) {
            try {
                const { x, y } = JSON.parse(savedScroll);
                window.scrollTo(x, y);
                sessionStorage.removeItem('hmr_scroll');
            } catch (error) {
                console.error('[HMR] Failed to restore scroll:', error);
            }
        }

        // Restore form values
        const savedForms = sessionStorage.getItem('hmr_forms');
        if (savedForms) {
            try {
                const formData = JSON.parse(savedForms);
                for (const [key, value] of Object.entries(formData)) {
                    const input = document.querySelector(`[name="${key}"], #${key}`);
                    if (input) {
                        input.value = value;
                    }
                }
                sessionStorage.removeItem('hmr_forms');
            } catch (error) {
                console.error('[HMR] Failed to restore forms:', error);
            }
        }
    }

    /**
     * Full page reload
     */
    fullReload(reason) {
        console.log('[HMR] 🔄 Full reload:', reason);
        this.preserveState();
        window.location.reload();
    }

    /**
     * Handle compilation error
     */
    handleError(data) {
        console.error('[HMR] Compilation error:', data.error);
        this.showNotification(`Error: ${data.error}`, 'error', 5000);

        // Show error overlay
        this.showErrorOverlay(data.error, data.file, data.line);
    }

    /**
     * Send message to server
     */
    send(message) {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify(message));
        }
    }

    /**
     * Show notification
     */
    showNotification(message, type = 'info', duration = 2000) {
        // Remove existing HMR notifications
        document.querySelectorAll('.hmr-notification').forEach(el => el.remove());

        const colors = {
            info: '#2196F3',
            success: '#4CAF50',
            error: '#f44336',
            warning: '#ff9800',
        };

        const notification = document.createElement('div');
        notification.className = 'hmr-notification';
        notification.style.cssText = `
            position: fixed;
            top: 20px;
            right: 20px;
            background: ${colors[type]};
            color: white;
            padding: 12px 20px;
            border-radius: 8px;
            box-shadow: 0 4px 12px rgba(0,0,0,0.2);
            font-family: -apple-system, sans-serif;
            font-size: 14px;
            z-index: 999999;
            animation: slideIn 0.3s ease;
        `;
        notification.textContent = message;

        document.body.appendChild(notification);

        if (duration > 0) {
            setTimeout(() => {
                notification.style.animation = 'slideOut 0.3s ease';
                setTimeout(() => notification.remove(), 300);
            }, duration);
        }
    }

    /**
     * Show error overlay
     */
    showErrorOverlay(error, file, line) {
        // Remove existing overlay
        const existing = document.getElementById('hmr-error-overlay');
        if (existing) {
            existing.remove();
        }

        const overlay = document.createElement('div');
        overlay.id = 'hmr-error-overlay';
        overlay.style.cssText = `
            position: fixed;
            top: 0;
            left: 0;
            width: 100%;
            height: 100%;
            background: rgba(0,0,0,0.9);
            color: white;
            padding: 40px;
            font-family: 'Courier New', monospace;
            font-size: 14px;
            z-index: 999998;
            overflow: auto;
        `;

        overlay.innerHTML = `
            <div style="max-width: 800px; margin: 0 auto;">
                <h2 style="color: #f44336; margin-bottom: 20px;">🚨 Compilation Error</h2>
                ${file ? `<p style="color: #999;">File: ${file}${line ? `:${line}` : ''}</p>` : ''}
                <pre style="background: #1e1e1e; padding: 20px; border-radius: 8px; overflow-x: auto;">${error}</pre>
                <button onclick="document.getElementById('hmr-error-overlay').remove()"
                        style="margin-top: 20px; padding: 10px 20px; background: #f44336; color: white; border: none; border-radius: 5px; cursor: pointer;">
                    Dismiss
                </button>
            </div>
        `;

        document.body.appendChild(overlay);
    }

    /**
     * Disconnect
     */
    disconnect() {
        if (this.ws) {
            this.ws.close();
            this.ws = null;
        }
    }
}

// Add CSS animations
const style = document.createElement('style');
style.textContent = `
    @keyframes slideIn {
        from { transform: translateX(400px); opacity: 0; }
        to { transform: translateX(0); opacity: 1; }
    }
    @keyframes slideOut {
        from { transform: translateX(0); opacity: 1; }
        to { transform: translateX(400px); opacity: 0; }
    }
`;
document.head.appendChild(style);

// Global HMR instance
if (typeof window !== 'undefined') {
    window.HMR = new HMRClient();

    // Auto-connect if in development
    if (window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1') {
        window.HMR.connect();

        // Restore state on page load
        window.addEventListener('DOMContentLoaded', () => {
            window.HMR.restoreState();
        });
    }

    console.log('[HMR] Client loaded');
}

// Export for modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = HMRClient;
}
