/**
 * RavensOne WebAssembly Runtime
 *
 * Loads and executes WASM modules compiled from RavensOne code
 * Provides JavaScript bridges for WASM ↔ Browser interaction
 */

class WasmRuntime {
    constructor() {
        this.modules = new Map();
        this.memory = null;
        this.imports = {
            env: {
                // Console output
                log: (ptr, len) => {
                    const message = this.readString(ptr, len);
                    console.log('[WASM]', message);
                },

                // Memory allocation (simple bump allocator)
                alloc: (size) => {
                    if (!this.allocator) {
                        this.allocator = { offset: 1024 }; // Start after initial data
                    }
                    const ptr = this.allocator.offset;
                    this.allocator.offset += size;
                    return ptr;
                },

                // DOM manipulation
                createElement: (tagPtr, tagLen) => {
                    const tag = this.readString(tagPtr, tagLen);
                    const el = document.createElement(tag);
                    return this.storeObject(el);
                },

                setInnerHTML: (elId, htmlPtr, htmlLen) => {
                    const el = this.getObject(elId);
                    const html = this.readString(htmlPtr, htmlLen);
                    if (el) {
                        el.innerHTML = html;
                    }
                },

                appendChild: (parentId, childId) => {
                    const parent = this.getObject(parentId);
                    const child = this.getObject(childId);
                    if (parent && child) {
                        parent.appendChild(child);
                    }
                },

                mountElement: (elId, selectorPtr, selectorLen) => {
                    const el = this.getObject(elId);
                    const selector = this.readString(selectorPtr, selectorLen);
                    const container = document.querySelector(selector);
                    if (container && el) {
                        container.appendChild(el);
                    }
                },

                // Math operations (examples)
                Math_abs: Math.abs,
                Math_sqrt: Math.sqrt,
                Math_pow: Math.pow,
                Math_floor: Math.floor,
                Math_ceil: Math.ceil,
                Math_random: Math.random,
            }
        };

        // Object store for DOM elements
        this.objects = new Map();
        this.nextObjectId = 1;
    }

    /**
     * Load WASM module from URL or ArrayBuffer
     */
    async load(source, moduleName = 'main') {
        console.log(`[WASM] Loading module: ${moduleName}`);

        let wasmBytes;

        if (typeof source === 'string') {
            // Load from URL
            const response = await fetch(source);
            wasmBytes = await response.arrayBuffer();
        } else {
            // Use provided ArrayBuffer
            wasmBytes = source;
        }

        // Compile and instantiate
        const wasmModule = await WebAssembly.instantiate(wasmBytes, this.imports);

        this.modules.set(moduleName, wasmModule.instance);
        this.memory = wasmModule.instance.exports.memory;

        console.log(`[WASM] Module loaded: ${moduleName}`);
        console.log(`[WASM] Exports:`, Object.keys(wasmModule.instance.exports));

        return wasmModule.instance;
    }

    /**
     * Call exported WASM function
     */
    call(moduleName, functionName, ...args) {
        const module = this.modules.get(moduleName);

        if (!module) {
            throw new Error(`Module not found: ${moduleName}`);
        }

        const func = module.exports[functionName];

        if (!func) {
            throw new Error(`Function not found: ${functionName} in module ${moduleName}`);
        }

        console.log(`[WASM] Calling ${moduleName}.${functionName}(${args.join(', ')})`);

        const result = func(...args);

        console.log(`[WASM] Result:`, result);

        return result;
    }

    /**
     * Read string from WASM memory
     */
    readString(ptr, len) {
        if (!this.memory) {
            return '';
        }

        const bytes = new Uint8Array(this.memory.buffer, ptr, len);
        return new TextDecoder().decode(bytes);
    }

    /**
     * Write string to WASM memory
     */
    writeString(str) {
        const bytes = new TextEncoder().encode(str);
        const ptr = this.imports.env.alloc(bytes.length);

        const memory = new Uint8Array(this.memory.buffer);
        memory.set(bytes, ptr);

        return { ptr, len: bytes.length };
    }

    /**
     * Store JavaScript object for WASM access
     */
    storeObject(obj) {
        const id = this.nextObjectId++;
        this.objects.set(id, obj);
        return id;
    }

    /**
     * Get stored JavaScript object
     */
    getObject(id) {
        return this.objects.get(id);
    }

    /**
     * Clear all modules
     */
    clear() {
        this.modules.clear();
        this.objects.clear();
        this.memory = null;
        this.allocator = null;
        this.nextObjectId = 1;
    }

    /**
     * Get module exports
     */
    getExports(moduleName = 'main') {
        const module = this.modules.get(moduleName);
        return module ? module.exports : null;
    }

    /**
     * Check if module is loaded
     */
    hasModule(moduleName) {
        return this.modules.has(moduleName);
    }

    /**
     * List loaded modules
     */
    listModules() {
        return Array.from(this.modules.keys());
    }
}

// Global runtime instance
if (typeof window !== 'undefined') {
    window.RavensWasm = new WasmRuntime();
    console.log('[WASM] Runtime initialized');
}

// Export for modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = WasmRuntime;
}
