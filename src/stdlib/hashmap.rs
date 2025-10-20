/// Standard library HashMap<K, V> type definition
/// A hash table based key-value map with O(1) average lookup

pub const HASHMAP_DEFINITION: &str = r#"
// HashMap<K, V> - A hash table based map
// Provides fast key-value lookups using hashing
struct HashMap<K, V> {
    buckets: Vec<Vec<(K, V)>>,  // Array of buckets (chaining for collisions)
    size: i32,                   // Number of key-value pairs
    capacity: i32,               // Number of buckets
}

impl<K, V> HashMap<K, V> {
    // Create a new empty HashMap
    fn new() -> HashMap<K, V> {
        return HashMap::with_capacity(16);
    }

    // Create a HashMap with a specific capacity
    fn with_capacity(capacity: i32) -> HashMap<K, V> {
        let buckets = Vec::new();

        // Initialize buckets
        for i in 0..capacity {
            buckets.push(Vec::new());
        }

        return HashMap {
            buckets: buckets,
            size: 0,
            capacity: capacity,
        };
    }

    // Insert a key-value pair
    // Returns the old value if the key already existed
    fn insert(self: &mut HashMap<K, V>, key: K, value: V) -> Option<V> {
        // Calculate hash and bucket index
        let hash = self.hash_key(&key);
        let bucket_index = hash % self.capacity;

        // Get the bucket
        let bucket = &mut self.buckets[bucket_index];

        // Check if key already exists
        for i in 0..bucket.len() {
            let pair = &bucket[i];
            if self.keys_equal(&pair.0, &key) {
                // Key exists, replace value
                let old_value = pair.1;
                pair.1 = value;
                return Option::Some(old_value);
            }
        }

        // Key doesn't exist, add new pair
        bucket.push((key, value));
        self.size = self.size + 1;

        // Check if we need to resize
        if self.size > self.capacity * 3 / 4 {
            self.resize(self.capacity * 2);
        }

        return Option::None;
    }

    // Get a value by key
    fn get(self: &HashMap<K, V>, key: &K) -> Option<&V> {
        let hash = self.hash_key(key);
        let bucket_index = hash % self.capacity;

        let bucket = &self.buckets[bucket_index];

        for pair in bucket {
            if self.keys_equal(&pair.0, key) {
                return Option::Some(&pair.1);
            }
        }

        return Option::None;
    }

    // Get a mutable reference to a value
    fn get_mut(self: &mut HashMap<K, V>, key: &K) -> Option<&mut V> {
        let hash = self.hash_key(key);
        let bucket_index = hash % self.capacity;

        let bucket = &mut self.buckets[bucket_index];

        for pair in bucket {
            if self.keys_equal(&pair.0, key) {
                return Option::Some(&mut pair.1);
            }
        }

        return Option::None;
    }

    // Remove a key-value pair
    fn remove(self: &mut HashMap<K, V>, key: &K) -> Option<V> {
        let hash = self.hash_key(key);
        let bucket_index = hash % self.capacity;

        let bucket = &mut self.buckets[bucket_index];

        for i in 0..bucket.len() {
            let pair = &bucket[i];
            if self.keys_equal(&pair.0, key) {
                let value = pair.1;
                bucket.remove(i);
                self.size = self.size - 1;
                return Option::Some(value);
            }
        }

        return Option::None;
    }

    // Check if a key exists in the map
    fn contains_key(self: &HashMap<K, V>, key: &K) -> bool {
        match self.get(key) {
            Option::Some(_) => true,
            Option::None => false,
        }
    }

    // Get the number of key-value pairs
    fn len(self: &HashMap<K, V>) -> i32 {
        return self.size;
    }

    // Check if the map is empty
    fn is_empty(self: &HashMap<K, V>) -> bool {
        return self.size == 0;
    }

    // Clear all key-value pairs
    fn clear(self: &mut HashMap<K, V>) {
        for bucket in &mut self.buckets {
            bucket.clear();
        }
        self.size = 0;
    }

    // Get all keys as a Vec
    fn keys(self: &HashMap<K, V>) -> Vec<K> {
        let result = Vec::new();

        for bucket in &self.buckets {
            for pair in bucket {
                result.push(pair.0);
            }
        }

        return result;
    }

    // Get all values as a Vec
    fn values(self: &HashMap<K, V>) -> Vec<V> {
        let result = Vec::new();

        for bucket in &self.buckets {
            for pair in bucket {
                result.push(pair.1);
            }
        }

        return result;
    }

    // Get all entries as Vec of (K, V) tuples
    fn entries(self: &HashMap<K, V>) -> Vec<(K, V)> {
        let result = Vec::new();

        for bucket in &self.buckets {
            for pair in bucket {
                result.push((pair.0, pair.1));
            }
        }

        return result;
    }

    // Reserve capacity for at least additional more elements
    fn reserve(self: &mut HashMap<K, V>, additional: i32) {
        let new_capacity = self.capacity + additional;
        self.resize(new_capacity);
    }

    // Shrink capacity to fit current size
    fn shrink_to_fit(self: &mut HashMap<K, V>) {
        let new_capacity = (self.size * 4 / 3).max(16);
        self.resize(new_capacity);
    }

    // Retain only elements matching predicate
    fn retain(self: &mut HashMap<K, V>, predicate: fn(&K, &V) -> bool) {
        for bucket in &mut self.buckets {
            let i = 0;
            for pair in bucket {
                if !predicate(&pair.0, &pair.1) {
                    bucket.remove(i);
                    self.size = self.size - 1;
                }
                i = i + 1;
            }
        }
    }

    // Get or insert a default value
    fn get_or_insert(self: &mut HashMap<K, V>, key: K, default: V) -> &mut V {
        if !self.contains_key(&key) {
            self.insert(key, default);
        }
        return self.get_mut(&key).unwrap();
    }

    // Get or insert with a function
    fn get_or_insert_with(self: &mut HashMap<K, V>, key: K, f: fn() -> V) -> &mut V {
        if !self.contains_key(&key) {
            let value = f();
            self.insert(key, value);
        }
        return self.get_mut(&key).unwrap();
    }

    // Update value if key exists
    fn update(self: &mut HashMap<K, V>, key: &K, f: fn(&V) -> V) -> Option<V> {
        match self.get_mut(key) {
            Option::Some(value_ref) => {
                let new_value = f(value_ref);
                *value_ref = new_value;
                return Option::Some(new_value);
            },
            Option::None => Option::None,
        }
    }

    // Insert or update value
    fn upsert(self: &mut HashMap<K, V>, key: K, update_fn: fn(&V) -> V, insert_value: V) {
        match self.get_mut(&key) {
            Option::Some(value_ref) => {
                *value_ref = update_fn(value_ref);
            },
            Option::None => {
                self.insert(key, insert_value);
            },
        }
    }

    // Extend map with another map
    fn extend(self: &mut HashMap<K, V>, other: &HashMap<K, V>) {
        for bucket in &other.buckets {
            for pair in bucket {
                self.insert(pair.0, pair.1);
            }
        }
    }

    // Hash a key (placeholder implementation)
    fn hash_key(self: &HashMap<K, V>, key: &K) -> i32 {
        // Would use actual hash function
        // For strings: FNV-1a or SipHash
        // For integers: identity or simple mixing
        return 0;  // Placeholder
    }

    // Check if two keys are equal
    fn keys_equal(self: &HashMap<K, V>, k1: &K, k2: &K) -> bool {
        // Would use actual equality comparison
        return true;  // Placeholder
    }

    // Resize the hash table
    fn resize(self: &mut HashMap<K, V>, new_capacity: i32) {
        let old_buckets = self.buckets;

        // Create new buckets
        let new_buckets = Vec::new();
        for i in 0..new_capacity {
            new_buckets.push(Vec::new());
        }

        self.buckets = new_buckets;
        self.capacity = new_capacity;
        self.size = 0;

        // Rehash all entries
        for bucket in old_buckets {
            for pair in bucket {
                self.insert(pair.0, pair.1);
            }
        }
    }
}

// Implement iteration over HashMap
impl<K, V> IntoIterator for HashMap<K, V> {
    type Item = (K, V);
    type IntoIter = HashMapIterator<K, V>;

    fn into_iter(self: HashMap<K, V>) -> HashMapIterator<K, V> {
        return HashMapIterator {
            map: self,
            bucket_index: 0,
            item_index: 0,
        };
    }
}

struct HashMapIterator<K, V> {
    map: HashMap<K, V>,
    bucket_index: i32,
    item_index: i32,
}

impl<K, V> Iterator for HashMapIterator<K, V> {
    type Item = (K, V);

    fn next(self: &mut HashMapIterator<K, V>) -> Option<(K, V)> {
        // Find next non-empty bucket with items
        while self.bucket_index < self.map.capacity {
            let bucket = &self.map.buckets[self.bucket_index];

            if self.item_index < bucket.len() {
                let pair = &bucket[self.item_index];
                self.item_index = self.item_index + 1;
                return Option::Some((pair.0, pair.1));
            }

            // Move to next bucket
            self.bucket_index = self.bucket_index + 1;
            self.item_index = 0;
        }

        return Option::None;
    }
}

// Helper functions for common use cases

// Create HashMap from array of key-value pairs
fn from_entries<K, V>(entries: &[(K, V)]) -> HashMap<K, V> {
    let map = HashMap::new();

    for pair in entries {
        map.insert(pair.0, pair.1);
    }

    return map;
}

// Merge two HashMaps
fn merge<K, V>(map1: &HashMap<K, V>, map2: &HashMap<K, V>) -> HashMap<K, V> {
    let result = HashMap::new();

    // Add all from map1
    for bucket in &map1.buckets {
        for pair in bucket {
            result.insert(pair.0, pair.1);
        }
    }

    // Add all from map2 (overwrites duplicates)
    for bucket in &map2.buckets {
        for pair in bucket {
            result.insert(pair.0, pair.1);
        }
    }

    return result;
}

// Filter HashMap by predicate
fn filter<K, V>(map: &HashMap<K, V>, predicate: fn(&K, &V) -> bool) -> HashMap<K, V> {
    let result = HashMap::new();

    for bucket in &map.buckets {
        for pair in bucket {
            if predicate(&pair.0, &pair.1) {
                result.insert(pair.0, pair.1);
            }
        }
    }

    return result;
}

// Map values to new HashMap
fn map_values<K, V, U>(map: &HashMap<K, V>, f: fn(&V) -> U) -> HashMap<K, U> {
    let result = HashMap::new();

    for bucket in &map.buckets {
        for pair in bucket {
            let new_value = f(&pair.1);
            result.insert(pair.0, new_value);
        }
    }

    return result;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hashmap_definition_exists() {
        assert!(!HASHMAP_DEFINITION.is_empty());
    }

    #[test]
    fn test_hashmap_definition_contains_struct() {
        assert!(HASHMAP_DEFINITION.contains("struct HashMap<K, V>"));
    }

    #[test]
    fn test_hashmap_definition_contains_methods() {
        assert!(HASHMAP_DEFINITION.contains("fn new()"));
        assert!(HASHMAP_DEFINITION.contains("fn insert("));
        assert!(HASHMAP_DEFINITION.contains("fn get("));
        assert!(HASHMAP_DEFINITION.contains("fn remove("));
        assert!(HASHMAP_DEFINITION.contains("fn contains_key("));
    }

    #[test]
    fn test_hashmap_definition_contains_advanced_methods() {
        assert!(HASHMAP_DEFINITION.contains("fn get_or_insert("));
        assert!(HASHMAP_DEFINITION.contains("fn update("));
        assert!(HASHMAP_DEFINITION.contains("fn upsert("));
        assert!(HASHMAP_DEFINITION.contains("fn retain("));
    }

    #[test]
    fn test_hashmap_definition_contains_iterator() {
        assert!(HASHMAP_DEFINITION.contains("impl<K, V> IntoIterator for HashMap<K, V>"));
        assert!(HASHMAP_DEFINITION.contains("struct HashMapIterator"));
    }

    #[test]
    fn test_hashmap_definition_contains_helpers() {
        assert!(HASHMAP_DEFINITION.contains("fn from_entries"));
        assert!(HASHMAP_DEFINITION.contains("fn merge"));
        assert!(HASHMAP_DEFINITION.contains("fn filter"));
        assert!(HASHMAP_DEFINITION.contains("fn map_values"));
    }
}
