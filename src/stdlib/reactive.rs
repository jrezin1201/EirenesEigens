/// Reactive primitives for building reactive UIs
///
/// Signal<T> - Reactive state container
/// Computed<T> - Derived reactive values
/// Effect - Side effects that run when dependencies change

use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashSet;

/// Unique identifier for reactive values
type SignalId = usize;

/// Global reactive context
thread_local! {
    static SIGNAL_COUNTER: RefCell<usize> = RefCell::new(0);
    static CURRENT_OBSERVER: RefCell<Option<SignalId>> = RefCell::new(None);
    static DEPENDENCIES: RefCell<Vec<(SignalId, HashSet<SignalId>)>> = RefCell::new(Vec::new());
}

fn next_signal_id() -> SignalId {
    SIGNAL_COUNTER.with(|counter| {
        let id = *counter.borrow();
        *counter.borrow_mut() = id + 1;
        id
    })
}

/// Signal<T> - Reactive state container
///
/// Automatically tracks dependencies and triggers re-renders when modified.
///
/// # Example
/// ```raven
/// let count = Signal::new(0);
/// count.set(count.get() + 1);  // Triggers re-render
/// ```
#[derive(Clone)]
pub struct Signal<T: Clone> {
    id: SignalId,
    value: Rc<RefCell<T>>,
    subscribers: Rc<RefCell<HashSet<SignalId>>>,
}

impl<T: Clone> Signal<T> {
    /// Create a new Signal with an initial value
    pub fn new(initial: T) -> Self {
        Signal {
            id: next_signal_id(),
            value: Rc::new(RefCell::new(initial)),
            subscribers: Rc::new(RefCell::new(HashSet::new())),
        }
    }

    /// Get the current value (tracks dependency)
    pub fn get(&self) -> T {
        // Track this signal as a dependency of the current observer
        CURRENT_OBSERVER.with(|observer| {
            if let Some(observer_id) = *observer.borrow() {
                self.subscribers.borrow_mut().insert(observer_id);
            }
        });
        self.value.borrow().clone()
    }

    /// Set a new value (triggers subscribers)
    pub fn set(&self, new_value: T) {
        *self.value.borrow_mut() = new_value;
        self.notify_subscribers();
    }

    /// Update the value with a function
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        f(&mut self.value.borrow_mut());
        self.notify_subscribers();
    }

    /// Get the signal ID
    pub fn id(&self) -> SignalId {
        self.id
    }

    fn notify_subscribers(&self) {
        let subscribers = self.subscribers.borrow().clone();
        for subscriber_id in subscribers {
            // Trigger re-computation of subscriber
            // TODO: Implement effect/computed re-execution
            println!("[Reactive] Signal {} changed, notifying subscriber {}", self.id, subscriber_id);
        }
    }
}

/// Computed<T> - Derived reactive value
///
/// Automatically recomputes when dependencies change.
///
/// # Example
/// ```raven
/// let count = Signal::new(0);
/// let doubled = Computed::new(|| count.get() * 2);
/// ```
pub struct Computed<T: Clone> {
    id: SignalId,
    compute: Rc<dyn Fn() -> T>,
    cached_value: Rc<RefCell<Option<T>>>,
}

impl<T: Clone> Computed<T> {
    /// Create a new computed value
    pub fn new<F>(compute: F) -> Self
    where
        F: Fn() -> T + 'static,
    {
        Computed {
            id: next_signal_id(),
            compute: Rc::new(compute),
            cached_value: Rc::new(RefCell::new(None)),
        }
    }

    /// Get the computed value (recomputes if dependencies changed)
    pub fn get(&self) -> T {
        // Set this as the current observer
        CURRENT_OBSERVER.with(|observer| {
            *observer.borrow_mut() = Some(self.id);
        });

        // Compute the value (this will track dependencies)
        let value = (self.compute)();

        // Clear current observer
        CURRENT_OBSERVER.with(|observer| {
            *observer.borrow_mut() = None;
        });

        // Cache the value
        *self.cached_value.borrow_mut() = Some(value.clone());

        value
    }
}

/// Effect - Side effect that runs when dependencies change
///
/// # Example
/// ```raven
/// let count = Signal::new(0);
/// Effect::new(|| {
///     console.log("Count:", count.get());
/// });
/// ```
pub struct Effect {
    id: SignalId,
    effect: Rc<dyn Fn()>,
}

impl Effect {
    /// Create a new effect that runs immediately and on dependency changes
    pub fn new<F>(effect: F) -> Self
    where
        F: Fn() + 'static,
    {
        let id = next_signal_id();

        // Set this as the current observer
        CURRENT_OBSERVER.with(|observer| {
            *observer.borrow_mut() = Some(id);
        });

        // Run the effect once to track dependencies
        effect();

        // Clear current observer
        CURRENT_OBSERVER.with(|observer| {
            *observer.borrow_mut() = None;
        });

        Effect {
            id,
            effect: Rc::new(effect),
        }
    }

    /// Manually run the effect
    pub fn run(&self) {
        (self.effect)();
    }
}

/// Reactive state marker for compiler
///
/// Variables marked as reactive will be auto-wrapped in Signal<T>
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReactiveMarker {
    /// Variable is reactive
    Reactive,
    /// Variable is non-reactive
    NonReactive,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signal_get_set() {
        let count = Signal::new(0);
        assert_eq!(count.get(), 0);

        count.set(5);
        assert_eq!(count.get(), 5);
    }

    #[test]
    fn signal_update() {
        let count = Signal::new(0);
        count.update(|v| *v += 1);
        assert_eq!(count.get(), 1);
    }

    #[test]
    fn computed_basic() {
        let count = Signal::new(5);
        let count_clone = count.clone();
        let doubled = Computed::new(move || count_clone.get() * 2);

        assert_eq!(doubled.get(), 10);

        count.set(10);
        assert_eq!(doubled.get(), 20);
    }
}
