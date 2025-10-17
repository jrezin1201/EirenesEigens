// Reactive State Management System
// Fine-grained reactivity with signals, effects, and automatic dependency tracking

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/// Unique ID for reactive nodes
type NodeId = usize;

/// Global reactive context
thread_local! {
    static REACTIVE_CONTEXT: RefCell<ReactiveContext> = RefCell::new(ReactiveContext::new());
}

/// Reactive context - tracks dependencies and effects
pub struct ReactiveContext {
    /// Currently running effect (for dependency tracking)
    current_effect: Option<NodeId>,
    /// Map of signal ID to dependent effects
    dependencies: HashMap<NodeId, HashSet<NodeId>>,
    /// Map of effect ID to its function
    effects: HashMap<NodeId, Rc<RefCell<dyn FnMut()>>>,
    /// Next available node ID
    next_id: NodeId,
}

impl ReactiveContext {
    fn new() -> Self {
        ReactiveContext {
            current_effect: None,
            dependencies: HashMap::new(),
            effects: HashMap::new(),
            next_id: 0,
        }
    }

    fn next_id(&mut self) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn track(&mut self, signal_id: NodeId) {
        if let Some(effect_id) = self.current_effect {
            self.dependencies
                .entry(signal_id)
                .or_insert_with(HashSet::new)
                .insert(effect_id);
        }
    }

    fn trigger(&mut self, signal_id: NodeId) {
        if let Some(effect_ids) = self.dependencies.get(&signal_id).cloned() {
            for effect_id in effect_ids {
                if let Some(effect) = self.effects.get(&effect_id) {
                    let effect_clone = Rc::clone(effect);
                    let mut effect_fn = effect_clone.borrow_mut();
                    effect_fn();
                }
            }
        }
    }
}

/// Signal - reactive primitive that holds a value
pub struct Signal<T: Clone> {
    id: NodeId,
    value: Rc<RefCell<T>>,
}

impl<T: Clone> Signal<T> {
    /// Create a new signal with an initial value
    pub fn new(initial: T) -> Self {
        let id = REACTIVE_CONTEXT.with(|ctx| ctx.borrow_mut().next_id());
        Signal {
            id,
            value: Rc::new(RefCell::new(initial)),
        }
    }

    /// Get the current value (tracks dependency)
    pub fn get(&self) -> T {
        REACTIVE_CONTEXT.with(|ctx| ctx.borrow_mut().track(self.id));
        self.value.borrow().clone()
    }

    /// Set a new value (triggers effects)
    pub fn set(&self, new_value: T) {
        *self.value.borrow_mut() = new_value;
        REACTIVE_CONTEXT.with(|ctx| ctx.borrow_mut().trigger(self.id));
    }

    /// Update the value using a function
    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        {
            let mut value = self.value.borrow_mut();
            f(&mut *value);
        }
        REACTIVE_CONTEXT.with(|ctx| ctx.borrow_mut().trigger(self.id));
    }
}

impl<T: Clone> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Signal {
            id: self.id,
            value: Rc::clone(&self.value),
        }
    }
}

/// Computed - derived reactive value
pub struct Computed<T: Clone> {
    signal: Signal<T>,
}

impl<T: Clone + 'static> Computed<T> {
    /// Create a computed value from a function
    pub fn new<F>(compute: F) -> Self
    where
        F: Fn() -> T + 'static,
    {
        let initial = compute();
        let signal = Signal::new(initial);

        // Create effect to recompute when dependencies change
        let signal_clone = signal.clone();
        create_effect(move || {
            let new_value = compute();
            signal_clone.set(new_value);
        });

        Computed { signal }
    }

    /// Get the current computed value
    pub fn get(&self) -> T {
        self.signal.get()
    }
}

/// Effect - side effect that runs when dependencies change
pub struct Effect {
    id: NodeId,
}

impl Effect {
    /// Dispose of the effect (stop it from running)
    pub fn dispose(&self) {
        REACTIVE_CONTEXT.with(|ctx| {
            let mut ctx = ctx.borrow_mut();
            ctx.effects.remove(&self.id);
            // Remove from all dependency lists
            for deps in ctx.dependencies.values_mut() {
                deps.remove(&self.id);
            }
        });
    }
}

/// Create an effect that runs when its dependencies change
pub fn create_effect<F>(f: F) -> Effect
where
    F: FnMut() + 'static,
{
    let effect_id = REACTIVE_CONTEXT.with(|ctx| ctx.borrow_mut().next_id());

    let effect_fn: Rc<RefCell<dyn FnMut()>> = Rc::new(RefCell::new(f));

    REACTIVE_CONTEXT.with(|ctx| {
        ctx.borrow_mut().effects.insert(effect_id, Rc::clone(&effect_fn));
    });

    // Run the effect once to establish dependencies
    REACTIVE_CONTEXT.with(|ctx| {
        ctx.borrow_mut().current_effect = Some(effect_id);
    });

    effect_fn.borrow_mut()();

    REACTIVE_CONTEXT.with(|ctx| {
        ctx.borrow_mut().current_effect = None;
    });

    Effect { id: effect_id }
}

/// Batch multiple updates together
pub fn batch<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    // In a full implementation, this would defer effect execution
    // For now, just run the function
    f()
}

/// Reactive store - object with reactive properties
pub struct Store<T: Clone> {
    data: Signal<T>,
}

impl<T: Clone> Store<T> {
    /// Create a new store
    pub fn new(initial: T) -> Self {
        Store {
            data: Signal::new(initial),
        }
    }

    /// Get a clone of the current state
    pub fn get(&self) -> T {
        self.data.get()
    }

    /// Update the store
    pub fn set(&self, value: T) {
        self.data.set(value);
    }

    /// Update using a function
    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        self.data.update(f);
    }
}

impl<T: Clone> Clone for Store<T> {
    fn clone(&self) -> Self {
        Store {
            data: self.data.clone(),
        }
    }
}

/// Reactive resource - async data fetching with loading states
#[derive(Clone, Debug, PartialEq)]
pub enum ResourceState<T> {
    Loading,
    Ready(T),
    Error(String),
}

pub struct Resource<T: Clone> {
    state: Signal<ResourceState<T>>,
}

impl<T: Clone + 'static> Resource<T> {
    /// Create a new resource
    pub fn new() -> Self {
        Resource {
            state: Signal::new(ResourceState::Loading),
        }
    }

    /// Get the current resource state
    pub fn state(&self) -> ResourceState<T> {
        self.state.get()
    }

    /// Set the resource to ready
    pub fn set_ready(&self, value: T) {
        self.state.set(ResourceState::Ready(value));
    }

    /// Set the resource to error
    pub fn set_error(&self, error: String) {
        self.state.set(ResourceState::Error(error));
    }

    /// Set the resource to loading
    pub fn set_loading(&self) {
        self.state.set(ResourceState::Loading);
    }
}

impl<T: Clone + 'static> Default for Resource<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Reactive array operations
pub struct ReactiveVec<T: Clone> {
    items: Signal<Vec<T>>,
}

impl<T: Clone> ReactiveVec<T> {
    /// Create a new reactive vector
    pub fn new() -> Self {
        ReactiveVec {
            items: Signal::new(Vec::new()),
        }
    }

    /// Create from initial items
    pub fn from_vec(items: Vec<T>) -> Self {
        ReactiveVec {
            items: Signal::new(items),
        }
    }

    /// Get all items
    pub fn get(&self) -> Vec<T> {
        self.items.get()
    }

    /// Push an item
    pub fn push(&self, item: T) {
        self.items.update(|items| items.push(item));
    }

    /// Remove an item at index
    pub fn remove(&self, index: usize) -> Option<T> {
        let mut result = None;
        self.items.update(|items| {
            if index < items.len() {
                result = Some(items.remove(index));
            }
        });
        result
    }

    /// Get length
    pub fn len(&self) -> usize {
        self.items.get().len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear all items
    pub fn clear(&self) {
        self.items.set(Vec::new());
    }

    /// Filter items
    pub fn filter<F>(&self, predicate: F) -> Vec<T>
    where
        F: FnMut(&T) -> bool,
    {
        self.items.get().into_iter().filter(predicate).collect()
    }

    /// Map over items
    pub fn map<U, F>(&self, f: F) -> Vec<U>
    where
        F: FnMut(T) -> U,
    {
        self.items.get().into_iter().map(f).collect()
    }
}

impl<T: Clone> Default for ReactiveVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Reactive map operations
pub struct ReactiveMap<K: Clone + Eq + std::hash::Hash, V: Clone> {
    items: Signal<HashMap<K, V>>,
}

impl<K: Clone + Eq + std::hash::Hash, V: Clone> ReactiveMap<K, V> {
    /// Create a new reactive map
    pub fn new() -> Self {
        ReactiveMap {
            items: Signal::new(HashMap::new()),
        }
    }

    /// Insert a key-value pair
    pub fn insert(&self, key: K, value: V) {
        self.items.update(|items| {
            items.insert(key, value);
        });
    }

    /// Get a value by key
    pub fn get(&self, key: &K) -> Option<V> {
        self.items.get().get(key).cloned()
    }

    /// Remove a key
    pub fn remove(&self, key: &K) {
        self.items.update(|items| {
            items.remove(key);
        });
    }

    /// Check if key exists
    pub fn contains_key(&self, key: &K) -> bool {
        self.items.get().contains_key(key)
    }

    /// Get all keys
    pub fn keys(&self) -> Vec<K> {
        self.items.get().keys().cloned().collect()
    }

    /// Get all values
    pub fn values(&self) -> Vec<V> {
        self.items.get().values().cloned().collect()
    }

    /// Clear the map
    pub fn clear(&self) {
        self.items.set(HashMap::new());
    }
}

impl<K: Clone + Eq + std::hash::Hash, V: Clone> Default for ReactiveMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal() {
        let count = Signal::new(0);
        assert_eq!(count.get(), 0);

        count.set(5);
        assert_eq!(count.get(), 5);

        count.update(|n| *n += 10);
        assert_eq!(count.get(), 15);
    }

    #[test]
    fn test_effect() {
        let count = Signal::new(0);
        let double = Signal::new(0);

        let count_clone = count.clone();
        let double_clone = double.clone();

        create_effect(move || {
            let value = count_clone.get();
            double_clone.value.borrow_mut().clone_from(&(value * 2));
        });

        // Effect runs once initially, so double should be 0
        assert_eq!(double.get(), 0);
    }

    #[test]
    fn test_computed() {
        let a = Signal::new(2);
        let b = Signal::new(3);

        let a_clone = a.clone();
        let b_clone = b.clone();

        let sum = Computed::new(move || a_clone.get() + b_clone.get());

        // Computed value should be 5 initially
        assert_eq!(sum.get(), 5);
    }

    #[test]
    fn test_store() {
        #[derive(Clone, Debug, PartialEq)]
        struct User {
            name: String,
            age: u32,
        }

        let store = Store::new(User {
            name: "Alice".to_string(),
            age: 30,
        });

        assert_eq!(store.get().name, "Alice");

        store.update(|user| {
            user.age = 31;
        });

        assert_eq!(store.get().age, 31);
    }

    #[test]
    fn test_reactive_vec() {
        let items = ReactiveVec::new();
        assert_eq!(items.len(), 0);

        items.push(1);
        items.push(2);
        items.push(3);

        assert_eq!(items.len(), 3);
        assert_eq!(items.get(), vec![1, 2, 3]);

        items.remove(1);
        assert_eq!(items.get(), vec![1, 3]);

        items.clear();
        assert!(items.is_empty());
    }

    #[test]
    fn test_reactive_map() {
        let map = ReactiveMap::new();

        map.insert("a", 1);
        map.insert("b", 2);

        assert_eq!(map.get(&"a"), Some(1));
        assert_eq!(map.get(&"b"), Some(2));

        map.remove(&"a");
        assert_eq!(map.get(&"a"), None);

        assert!(map.contains_key(&"b"));
    }

    #[test]
    fn test_resource() {
        let resource: Resource<String> = Resource::new();

        match resource.state() {
            ResourceState::Loading => (),
            _ => panic!("Expected loading state"),
        }

        resource.set_ready("data".to_string());

        match resource.state() {
            ResourceState::Ready(data) => assert_eq!(data, "data"),
            _ => panic!("Expected ready state"),
        }

        resource.set_error("error".to_string());

        match resource.state() {
            ResourceState::Error(msg) => assert_eq!(msg, "error"),
            _ => panic!("Expected error state"),
        }
    }
}
