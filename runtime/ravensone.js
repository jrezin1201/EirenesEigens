/**
 * RavensOne Runtime - JavaScript bridge for WASM components
 *
 * This runtime provides:
 * 1. WASM module loading and instantiation
 * 2. Virtual DOM rendering to real DOM
 * 3. Reactive state management
 * 4. Event handling
 */

class RavensOneRuntime {
    constructor() {
        this.wasmInstance = null;
        this.memory = null;
        this.reactiveState = new Map(); // Legacy reactive state
        this.signals = new Map(); // Signal ID -> Signal instance
        this.subscribers = new Map();
        this.rootElement = null;
        this.componentEffects = new Map(); // componentId -> Effect instance
    }

    /**
     * Initialize the runtime with a WASM module
     */
    async init(wasmPath, rootElementId) {
        this.rootElement = document.getElementById(rootElementId);
        if (!this.rootElement) {
            throw new Error(`Root element #${rootElementId} not found`);
        }

        // Load WASM module
        const response = await fetch(wasmPath);
        const wasmBytes = await response.arrayBuffer();

        // Create imports for WASM
        const importObject = {
            env: {
                // DOM manipulation functions
                createElement: this.createElement.bind(this),
                createTextNode: this.createTextNode.bind(this),
                appendChild: this.appendChild.bind(this),
                setAttribute: this.setAttribute.bind(this),

                // Reactive state functions (legacy)
                getState: this.getState.bind(this),
                setState: this.setState.bind(this),

                // Signal-based reactive functions
                signal_new: this.signalNew.bind(this),
                signal_get: this.signalGet.bind(this),
                signal_set: this.signalSet.bind(this),
                signal_update: this.signalUpdate.bind(this),

                // Component registration
                register_component: this.registerComponent.bind(this),
                create_effect: this.createEffect.bind(this),

                // Event handling
                addEventListener: this.addEventListenerWasm.bind(this),
                removeEventListener: this.removeEventListenerWasm.bind(this),

                // Logging
                log: this.log.bind(this),
            }
        };

        const wasmModule = await WebAssembly.instantiate(wasmBytes, importObject);
        this.wasmInstance = wasmModule.instance;
        this.memory = this.wasmInstance.exports.memory;

        return this;
    }

    /**
     * Mount a component to the DOM
     */
    mount(componentName = 'main') {
        if (!this.wasmInstance) {
            throw new Error('Runtime not initialized. Call init() first.');
        }

        // Call the WASM component's main/entry function
        const mainFn = this.wasmInstance.exports[componentName];
        if (!mainFn) {
            throw new Error(`Component '${componentName}' not found in WASM exports`);
        }

        // For now, since we're not yet generating proper VDOM from WASM,
        // we'll create a placeholder DOM structure
        this.renderPlaceholder(componentName);
    }

    /**
     * Temporary placeholder renderer
     */
    renderPlaceholder(componentName) {
        const container = document.createElement('div');
        container.className = 'raven-component';
        container.innerHTML = `
            <div class="container">
                <p>Count: <span id="count">0</span></p>
                <button id="increment-btn">Increment</button>
            </div>
        `;

        // Add event listener for increment
        const btn = container.querySelector('#increment-btn');
        const countSpan = container.querySelector('#count');
        let count = 0;

        btn.addEventListener('click', () => {
            count++;
            countSpan.textContent = count;
        });

        this.rootElement.appendChild(container);
    }

    /**
     * DOM manipulation exports (called from WASM)
     */
    createElement(tagNamePtr, tagNameLen) {
        const tagName = this.readString(tagNamePtr, tagNameLen);
        const el = document.createElement(tagName);
        return this.storeElement(el);
    }

    createTextNode(textPtr, textLen) {
        const text = this.readString(textPtr, textLen);
        const node = document.createTextNode(text);
        return this.storeElement(node);
    }

    appendChild(parentId, childId) {
        const parent = this.getElement(parentId);
        const child = this.getElement(childId);
        parent.appendChild(child);
    }

    setAttribute(elementId, namePtr, nameLen, valuePtr, valueLen) {
        const element = this.getElement(elementId);
        const name = this.readString(namePtr, nameLen);
        const value = this.readString(valuePtr, valueLen);
        element.setAttribute(name, value);
    }

    /**
     * Reactive state management
     */
    getState(keyPtr, keyLen) {
        const key = this.readString(keyPtr, keyLen);
        return this.reactiveState.get(key) || 0;
    }

    setState(keyPtr, keyLen, value) {
        const key = this.readString(keyPtr, keyLen);
        this.reactiveState.set(key, value);
        this.notifySubscribers(key);
    }

    notifySubscribers(key) {
        const subs = this.subscribers.get(key) || [];
        subs.forEach(callback => callback());
    }

    /**
     * Signal-based reactive state management
     */
    signalNew(initialValue) {
        const signal = new window.RavensReactive.Signal(initialValue);
        this.signals.set(signal.id, signal);
        console.log(`[Runtime] Created Signal #${signal.id} with value:`, initialValue);
        return signal.id;
    }

    signalGet(signalId) {
        const signal = this.signals.get(signalId);
        if (!signal) {
            console.error(`[Runtime] Signal #${signalId} not found`);
            return 0;
        }
        return signal.get();
    }

    signalSet(signalId, newValue) {
        const signal = this.signals.get(signalId);
        if (!signal) {
            console.error(`[Runtime] Signal #${signalId} not found`);
            return;
        }
        console.log(`[Runtime] Setting Signal #${signalId} to:`, newValue);
        signal.set(newValue);
    }

    signalUpdate(signalId, delta) {
        const signal = this.signals.get(signalId);
        if (!signal) {
            console.error(`[Runtime] Signal #${signalId} not found`);
            return;
        }
        const currentValue = signal.get();
        signal.set(currentValue + delta);
    }

    registerComponent(componentId) {
        console.log(`[Runtime] Registering component #${componentId}`);
        // Component registration logic will be expanded later
        return componentId;
    }

    createEffect(componentId, renderFnPtr) {
        console.log(`[Runtime] Creating effect for component #${componentId}`);
        // Effect creation logic will be expanded later
        // This would typically call back into WASM to run the render function
        return 0;
    }

    /**
     * Event handling
     */
    addEventListenerWasm(elementId, eventNamePtr, eventNameLen, handlerFnIndex) {
        const element = this.getElement(elementId);
        if (!element) {
            console.error(`[Runtime] Element #${elementId} not found for event listener`);
            return;
        }

        const eventName = this.readString(eventNamePtr, eventNameLen);
        console.log(`[Runtime] Adding '${eventName}' listener to element #${elementId}`);

        // Create a wrapper that calls back into WASM
        const handler = (event) => {
            console.log(`[Runtime] Event '${eventName}' triggered on element #${elementId}`);

            // Call the WASM handler function
            if (this.wasmInstance && this.wasmInstance.exports[`handler_${handlerFnIndex}`]) {
                this.wasmInstance.exports[`handler_${handlerFnIndex}`]();
            } else {
                console.warn(`[Runtime] Handler function 'handler_${handlerFnIndex}' not found`);
            }
        };

        element.addEventListener(eventName, handler);

        // Store handler reference for potential removal
        if (!this.eventHandlers) {
            this.eventHandlers = new Map();
        }
        const key = `${elementId}_${eventName}`;
        this.eventHandlers.set(key, handler);
    }

    removeEventListenerWasm(elementId, eventNamePtr, eventNameLen) {
        const element = this.getElement(elementId);
        if (!element) {
            console.error(`[Runtime] Element #${elementId} not found for event listener removal`);
            return;
        }

        const eventName = this.readString(eventNamePtr, eventNameLen);
        const key = `${elementId}_${eventName}`;

        const handler = this.eventHandlers?.get(key);
        if (handler) {
            element.removeEventListener(eventName, handler);
            this.eventHandlers.delete(key);
            console.log(`[Runtime] Removed '${eventName}' listener from element #${elementId}`);
        }
    }

    /**
     * Helper functions
     */
    readString(ptr, len) {
        const bytes = new Uint8Array(this.memory.buffer, ptr, len);
        return new TextDecoder().decode(bytes);
    }

    log(msgPtr, msgLen) {
        const msg = this.readString(msgPtr, msgLen);
        console.log('[RavensOne]', msg);
    }

    // Element storage (temporary solution until we have proper VDOM)
    elementStore = new Map();
    elementIdCounter = 1;

    storeElement(el) {
        const id = this.elementIdCounter++;
        this.elementStore.set(id, el);
        return id;
    }

    getElement(id) {
        return this.elementStore.get(id);
    }
}

// Export for use in browser
if (typeof window !== 'undefined') {
    window.RavensOneRuntime = RavensOneRuntime;
}

// Export for Node.js
if (typeof module !== 'undefined' && module.exports) {
    module.exports = RavensOneRuntime;
}
