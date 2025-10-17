/// Standard collections for RavensOne
///
/// RArray<T> - Dynamic array (like Rust Vec)
/// RMap<K, V> - Hash map (like Rust HashMap)

use std::collections::HashMap;
use std::hash::Hash;

/// Dynamic array type
///
/// # Example
/// ```raven
/// let nums = RArray::new();
/// nums.push(1);
/// nums.push(2);
/// let sum = nums.reduce(|acc, x| acc + x, 0);
/// ```
#[derive(Clone, Debug)]
pub struct RArray<T> {
    items: Vec<T>,
}

impl<T> RArray<T> {
    pub fn new() -> Self {
        RArray { items: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        RArray {
            items: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.items.get(index)
    }

    pub fn iter(&self) -> std::slice::Iter<T> {
        self.items.iter()
    }
}

impl<T: Clone> RArray<T> {
    pub fn map<U, F>(&self, f: F) -> RArray<U>
    where
        F: Fn(&T) -> U,
    {
        RArray {
            items: self.items.iter().map(f).collect(),
        }
    }

    pub fn filter<F>(&self, predicate: F) -> RArray<T>
    where
        F: Fn(&T) -> bool,
    {
        RArray {
            items: self.items.iter().filter(|x| predicate(x)).cloned().collect(),
        }
    }
}

impl<T> RArray<T>
where
    T: Clone,
{
    pub fn reduce<U, F>(&self, f: F, initial: U) -> U
    where
        F: Fn(U, &T) -> U,
        U: Clone,
    {
        self.items.iter().fold(initial, f)
    }
}

/// Hash map type
///
/// # Example
/// ```raven
/// let scores = RMap::new();
/// scores.set("Alice", 100);
/// scores.set("Bob", 95);
/// let alice = scores.get("Alice");  // Some(100)
/// ```
#[derive(Clone, Debug)]
pub struct RMap<K, V> {
    map: HashMap<K, V>,
}

impl<K: Eq + Hash, V> RMap<K, V> {
    pub fn new() -> Self {
        RMap {
            map: HashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        RMap {
            map: HashMap::with_capacity(capacity),
        }
    }

    pub fn set(&mut self, key: K, value: V) -> Option<V> {
        self.map.insert(key, value)
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.map.remove(key)
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.map.keys()
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.map.values()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn array_basic() {
        let mut arr = RArray::new();
        arr.push(1);
        arr.push(2);
        arr.push(3);

        assert_eq!(arr.len(), 3);
        assert_eq!(arr.get(0), Some(&1));
    }

    #[test]
    fn array_map() {
        let mut arr = RArray::new();
        arr.push(1);
        arr.push(2);
        arr.push(3);

        let doubled = arr.map(|x| x * 2);
        assert_eq!(doubled.get(0), Some(&2));
        assert_eq!(doubled.get(1), Some(&4));
    }

    #[test]
    fn map_basic() {
        let mut map = RMap::new();
        map.set("Alice", 100);
        map.set("Bob", 95);

        assert_eq!(map.get(&"Alice"), Some(&100));
        assert_eq!(map.get(&"Charlie"), None);
    }
}
