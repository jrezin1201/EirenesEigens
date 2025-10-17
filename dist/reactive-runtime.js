/**
 * RavensOne Reactive Runtime
 *
 * Provides Signal<T>, Computed<T>, and Effect primitives that integrate with WASM code.
 * This runtime tracks dependencies and triggers re-renders when state changes.
 */

// Global state
let signalCounter = 0;
let currentObserver = null;
const dependencies = new Map(); // signalId -> Set<observerId>
const observers = new Map(); // observerId -> callback

/**
 * Signal<T> - Reactive state container
 */
class Signal {
    constructor(initialValue) {
        this.id = signalCounter++;
        this.value = initialValue;
        this.subscribers = new Set();

        console.log(`[Reactive] Created Signal #${this.id} with value:`, initialValue);
    }

    get() {
        // Track this signal as a dependency of the current observer
        if (currentObserver !== null) {
            this.subscribers.add(currentObserver);
            console.log(`[Reactive] Signal #${this.id} tracked by observer #${currentObserver}`);
        }
        return this.value;
    }

    set(newValue) {
        if (this.value !== newValue) {
            console.log(`[Reactive] Signal #${this.id} changed:`, this.value, '->', newValue);
            this.value = newValue;
            this.notifySubscribers();
        }
    }

    update(fn) {
        this.set(fn(this.value));
    }

    notifySubscribers() {
        console.log(`[Reactive] Signal #${this.id} notifying ${this.subscribers.size} subscribers`);
        for (const observerId of this.subscribers) {
            const callback = observers.get(observerId);
            if (callback) {
                console.log(`[Reactive] Running observer #${observerId}`);
                callback();
            }
        }
    }
}

/**
 * Computed<T> - Derived reactive value
 */
class Computed {
    constructor(computeFn) {
        this.id = signalCounter++;
        this.computeFn = computeFn;
        this.cachedValue = undefined;
        this.dirty = true;

        console.log(`[Reactive] Created Computed #${this.id}`);
    }

    get() {
        if (this.dirty) {
            const prevObserver = currentObserver;
            currentObserver = this.id;

            this.cachedValue = this.computeFn();
            this.dirty = false;

            currentObserver = prevObserver;
            console.log(`[Reactive] Computed #${this.id} recomputed:`, this.cachedValue);
        }
        return this.cachedValue;
    }

    invalidate() {
        this.dirty = true;
    }
}

/**
 * Effect - Side effect that runs when dependencies change
 */
class Effect {
    constructor(effectFn) {
        this.id = signalCounter++;
        this.effectFn = effectFn;

        observers.set(this.id, () => this.run());

        console.log(`[Reactive] Created Effect #${this.id}`);
        this.run();
    }

    run() {
        const prevObserver = currentObserver;
        currentObserver = this.id;

        this.effectFn();

        currentObserver = prevObserver;
    }
}

/**
 * Component re-render tracking
 */
const componentRenderFunctions = new Map(); // componentId -> renderFn

function registerComponent(componentId, renderFn) {
    componentRenderFunctions.set(componentId, renderFn);
    console.log(`[Reactive] Registered component #${componentId}`);
}

function createReactiveEffect(componentId) {
    const renderFn = componentRenderFunctions.get(componentId);
    if (!renderFn) {
        console.error(`[Reactive] Component #${componentId} not found`);
        return;
    }

    return new Effect(() => {
        console.log(`[Reactive] Re-rendering component #${componentId}`);
        renderFn();
    });
}

// Export for WASM imports
window.RavensReactive = {
    Signal,
    Computed,
    Effect,
    registerComponent,
    createReactiveEffect,
};

console.log('[Reactive] Runtime initialized');
