/**
 * RavensOne HTTP Runtime
 *
 * Provides HTTP client functionality for WASM modules
 * Uses Fetch API in browser, native HTTP in Node.js
 */

class HttpRuntime {
    constructor() {
        this.pendingRequests = new Map();
        this.requestIdCounter = 0;
    }

    /**
     * Make an HTTP request
     * @param {string} method - HTTP method (GET, POST, etc.)
     * @param {string} url - Request URL
     * @param {Object} headers - Request headers
     * @param {string|null} body - Request body
     * @returns {Promise<Object>} Response object
     */
    async fetch(method, url, headers = {}, body = null) {
        const requestId = this.requestIdCounter++;

        console.log(`[HTTP] ${method} ${url}`);

        try {
            const options = {
                method: method,
                headers: headers,
            };

            if (body !== null && method !== 'GET' && method !== 'HEAD') {
                options.body = body;
            }

            const response = await fetch(url, options);

            // Read response body
            const responseText = await response.text();

            // Parse response headers
            const responseHeaders = {};
            response.headers.forEach((value, key) => {
                responseHeaders[key] = value;
            });

            const result = {
                status: response.status,
                statusText: response.statusText,
                headers: responseHeaders,
                body: responseText,
                ok: response.ok,
            };

            console.log(`[HTTP] ${method} ${url} → ${response.status}`);

            return result;

        } catch (error) {
            console.error(`[HTTP] ${method} ${url} → Error:`, error);

            return {
                status: 0,
                statusText: error.message,
                headers: {},
                body: '',
                ok: false,
                error: error.message,
            };
        }
    }

    /**
     * Make a GET request
     */
    async get(url, headers = {}) {
        return this.fetch('GET', url, headers, null);
    }

    /**
     * Make a POST request
     */
    async post(url, body, headers = {}) {
        return this.fetch('POST', url, headers, body);
    }

    /**
     * Make a PUT request
     */
    async put(url, body, headers = {}) {
        return this.fetch('PUT', url, headers, body);
    }

    /**
     * Make a DELETE request
     */
    async delete(url, headers = {}) {
        return this.fetch('DELETE', url, headers, null);
    }

    /**
     * Make a PATCH request
     */
    async patch(url, body, headers = {}) {
        return this.fetch('PATCH', url, headers, body);
    }

    /**
     * Fetch JSON data
     */
    async getJSON(url, headers = {}) {
        const response = await this.get(url, headers);
        if (response.ok) {
            try {
                response.json = JSON.parse(response.body);
            } catch (e) {
                console.error('[HTTP] Failed to parse JSON:', e);
            }
        }
        return response;
    }

    /**
     * Post JSON data
     */
    async postJSON(url, data, headers = {}) {
        const combinedHeaders = {
            'Content-Type': 'application/json',
            ...headers
        };
        const body = JSON.stringify(data);
        return this.post(url, body, combinedHeaders);
    }
}

// Global HTTP runtime instance
if (typeof window !== 'undefined') {
    window.RavensHTTP = new HttpRuntime();
    console.log('[HTTP] Runtime initialized');
}

// Export for Node.js
if (typeof module !== 'undefined' && module.exports) {
    module.exports = HttpRuntime;
}
