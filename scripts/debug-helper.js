/**
 * RavensOne Debug Helper
 *
 * This script helps with debugging RavensOne applications in Chrome DevTools.
 * It provides source map support and enhanced error messages.
 */

class RavensOneDebugger {
    constructor() {
        this.sourceMaps = new Map();
        this.originalErrors = [];
        this.setupErrorHandling();
    }

    /**
     * Register a source map for a WASM module
     */
    registerSourceMap(wasmUrl, sourceMap) {
        this.sourceMaps.set(wasmUrl, sourceMap);
        console.log(`📍 Registered source map for ${wasmUrl}`);
    }

    /**
     * Load source map from inline comment or external file
     */
    async loadSourceMap(wasmUrl, wasmContent) {
        // Check for inline source map
        const inlineMatch = wasmContent.match(/\/\/# sourceMappingURL=data:application\/json;charset=utf-8;base64,(.+)/);

        if (inlineMatch) {
            const decoded = atob(inlineMatch[1]);
            const sourceMap = JSON.parse(decoded);
            this.registerSourceMap(wasmUrl, sourceMap);
            return sourceMap;
        }

        // Check for external source map
        const mapMatch = wasmContent.match(/\/\/# sourceMappingURL=(.+\.map)/);

        if (mapMatch) {
            const mapUrl = new URL(mapMatch[1], wasmUrl).href;
            try {
                const response = await fetch(mapUrl);
                const sourceMap = await response.json();
                this.registerSourceMap(wasmUrl, sourceMap);
                return sourceMap;
            } catch (err) {
                console.warn(`Failed to load source map from ${mapUrl}:`, err);
            }
        }

        return null;
    }

    /**
     * Map a WASM stack frame to original source location
     */
    mapStackFrame(wasmUrl, line, column) {
        const sourceMap = this.sourceMaps.get(wasmUrl);

        if (!sourceMap) {
            return null;
        }

        // Parse the mappings to find the original location
        // This is a simplified implementation
        const originalLocation = this.findOriginalLocation(sourceMap, line, column);

        if (originalLocation) {
            return {
                source: originalLocation.source,
                line: originalLocation.line,
                column: originalLocation.column,
                name: originalLocation.name
            };
        }

        return null;
    }

    /**
     * Find original location from source map
     */
    findOriginalLocation(sourceMap, line, column) {
        // This is a simplified implementation
        // In a real implementation, you would decode the VLQ mappings

        if (sourceMap.sources && sourceMap.sources.length > 0) {
            return {
                source: sourceMap.sources[0],
                line: line, // Approximate
                column: column,
                name: null
            };
        }

        return null;
    }

    /**
     * Setup enhanced error handling
     */
    setupErrorHandling() {
        const self = this;

        window.addEventListener('error', (event) => {
            self.handleError(event.error, event);
        });

        window.addEventListener('unhandledrejection', (event) => {
            self.handleError(event.reason, event);
        });
    }

    /**
     * Handle an error and try to map it to source
     */
    handleError(error, event) {
        if (!error || !error.stack) {
            return;
        }

        this.originalErrors.push({
            error,
            timestamp: new Date(),
            mapped: this.mapErrorStack(error.stack)
        });

        // Print enhanced error to console
        console.group('🐛 RavensOne Error');
        console.error('Original error:', error.message);

        const mappedStack = this.mapErrorStack(error.stack);
        if (mappedStack.length > 0) {
            console.log('\n📍 Source locations:');
            mappedStack.forEach((frame, i) => {
                console.log(`  ${i + 1}. ${frame.source}:${frame.line}:${frame.column}`);
                if (frame.name) {
                    console.log(`     in ${frame.name}()`);
                }
            });
        }

        console.groupEnd();
    }

    /**
     * Map an error stack trace to original sources
     */
    mapErrorStack(stackTrace) {
        const frames = [];
        const lines = stackTrace.split('\n');

        for (const line of lines) {
            // Parse stack frame line
            const match = line.match(/at\s+(.+?)\s+\((.+?):(\d+):(\d+)\)/);

            if (match) {
                const [, name, file, line, column] = match;
                const mapped = this.mapStackFrame(file, parseInt(line), parseInt(column));

                if (mapped) {
                    frames.push({
                        ...mapped,
                        originalName: name
                    });
                }
            }
        }

        return frames;
    }

    /**
     * Get all recorded errors
     */
    getErrors() {
        return this.originalErrors;
    }

    /**
     * Clear all recorded errors
     */
    clearErrors() {
        this.originalErrors = [];
    }

    /**
     * Export errors as JSON for debugging
     */
    exportErrors() {
        return JSON.stringify(this.originalErrors, null, 2);
    }
}

// Create global debugger instance
window.RavensOneDebugger = window.RavensOneDebugger || new RavensOneDebugger();

// Expose helpful debugging utilities
window.raven = {
    debug: window.RavensOneDebugger,

    /**
     * Enable verbose logging
     */
    enableVerboseLogging() {
        localStorage.setItem('raven:debug', 'true');
        console.log('✅ Verbose logging enabled');
    },

    /**
     * Disable verbose logging
     */
    disableVerboseLogging() {
        localStorage.removeItem('raven:debug');
        console.log('✅ Verbose logging disabled');
    },

    /**
     * Check if verbose logging is enabled
     */
    isVerbose() {
        return localStorage.getItem('raven:debug') === 'true';
    },

    /**
     * Print performance metrics
     */
    printPerformance() {
        if (!performance || !performance.getEntriesByType) {
            console.warn('Performance API not available');
            return;
        }

        const entries = performance.getEntriesByType('measure');

        console.group('⚡ Performance Metrics');
        console.table(entries.map(e => ({
            name: e.name,
            duration: `${e.duration.toFixed(2)}ms`,
            startTime: `${e.startTime.toFixed(2)}ms`
        })));
        console.groupEnd();
    },

    /**
     * Measure execution time of a function
     */
    measure(name, fn) {
        const start = performance.now();
        const result = fn();
        const end = performance.now();

        console.log(`⏱️  ${name}: ${(end - start).toFixed(2)}ms`);

        return result;
    },

    /**
     * Measure async execution time
     */
    async measureAsync(name, fn) {
        const start = performance.now();
        const result = await fn();
        const end = performance.now();

        console.log(`⏱️  ${name}: ${(end - start).toFixed(2)}ms`);

        return result;
    }
};

console.log('🔧 RavensOne Debug Helper loaded');
console.log('   Use window.raven.debug for debugging utilities');
console.log('   Use window.raven.enableVerboseLogging() for detailed logs');
