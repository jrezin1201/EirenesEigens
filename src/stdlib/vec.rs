/// Standard library Vec<T> (growable array) type definition
/// A dynamic array that can grow and shrink at runtime

pub const VEC_DEFINITION: &str = r#"
// Vec<T> - A growable, heap-allocated array
// Provides dynamic sizing and automatic memory management
struct Vec<T> {
    data: [T],        // Internal array storage
    length: i32,      // Current number of elements
    capacity: i32,    // Total allocated capacity
}

impl<T> Vec<T> {
    // Create a new empty Vec
    fn new() -> Vec<T> {
        return Vec {
            data: [],
            length: 0,
            capacity: 0,
        };
    }

    // Create a Vec with a specific initial capacity
    fn with_capacity(capacity: i32) -> Vec<T> {
        return Vec {
            data: [],  // Would allocate array of size capacity
            length: 0,
            capacity: capacity,
        };
    }

    // Create a Vec from an array
    fn from_array(arr: [T]) -> Vec<T> {
        // Would copy array elements
        return Vec {
            data: arr,
            length: 0,  // Would be arr.len()
            capacity: 0,
        };
    }

    // Add an element to the end of the Vec
    fn push(self: &mut Vec<T>, value: T) {
        // If length == capacity, need to grow
        // In real implementation, would:
        // 1. Allocate new array with double capacity
        // 2. Copy existing elements
        // 3. Add new element
        // 4. Update length
        self.length = self.length + 1;
    }

    // Remove and return the last element
    fn pop(self: &mut Vec<T>) -> Option<T> {
        if self.length == 0 {
            return Option::None;
        } else {
            self.length = self.length - 1;
            // Would return the element at self.data[self.length]
            return Option::None;  // Placeholder
        }
    }

    // Get the number of elements in the Vec
    fn len(self: &Vec<T>) -> i32 {
        return self.length;
    }

    // Check if the Vec is empty
    fn is_empty(self: &Vec<T>) -> bool {
        return self.length == 0;
    }

    // Get the current capacity
    fn capacity(self: &Vec<T>) -> i32 {
        return self.capacity;
    }

    // Get an element at a specific index
    fn get(self: &Vec<T>, index: i32) -> Option<&T> {
        if index >= 0 && index < self.length {
            // Would return Some(&self.data[index])
            return Option::None;  // Placeholder
        } else {
            return Option::None;
        }
    }

    // Get a mutable reference to an element
    fn get_mut(self: &mut Vec<T>, index: i32) -> Option<&mut T> {
        if index >= 0 && index < self.length {
            // Would return Some(&mut self.data[index])
            return Option::None;  // Placeholder
        } else {
            return Option::None;
        }
    }

    // Insert an element at a specific index
    fn insert(self: &mut Vec<T>, index: i32, value: T) {
        // Would shift elements right and insert
        // For now, just increment length
        if index >= 0 && index <= self.length {
            self.length = self.length + 1;
        }
    }

    // Remove an element at a specific index
    fn remove(self: &mut Vec<T>, index: i32) -> Option<T> {
        if index >= 0 && index < self.length {
            // Would shift elements left and return removed element
            self.length = self.length - 1;
            return Option::None;  // Placeholder
        } else {
            return Option::None;
        }
    }

    // Clear all elements from the Vec
    fn clear(self: &mut Vec<T>) {
        self.length = 0;
        // Would also reset data array
    }

    // Reserve additional capacity
    fn reserve(self: &mut Vec<T>, additional: i32) {
        let new_capacity = self.length + additional;
        if new_capacity > self.capacity {
            // Would reallocate with new capacity
            self.capacity = new_capacity;
        }
    }

    // Shrink capacity to fit the current length
    fn shrink_to_fit(self: &mut Vec<T>) {
        self.capacity = self.length;
        // Would reallocate to exact size
    }

    // Truncate the Vec to a specific length
    fn truncate(self: &mut Vec<T>, new_len: i32) {
        if new_len < self.length {
            self.length = new_len;
            // Would drop elements beyond new_len
        }
    }

    // Swap two elements by index
    fn swap(self: &mut Vec<T>, i: i32, j: i32) {
        if i >= 0 && i < self.length && j >= 0 && j < self.length {
            // Would swap self.data[i] with self.data[j]
        }
    }

    // Reverse the elements in the Vec
    fn reverse(self: &mut Vec<T>) {
        let i = 0;
        let j = self.length - 1;
        // Would swap elements from both ends moving inward
    }

    // Check if the Vec contains a value
    fn contains(self: &Vec<T>, value: &T) -> bool {
        // Would iterate and compare each element
        return false;
    }

    // Find the first index of a value
    fn index_of(self: &Vec<T>, value: &T) -> Option<i32> {
        // Would iterate and return index if found
        return Option::None;
    }

    // Append another Vec to this one
    fn append(self: &mut Vec<T>, other: &mut Vec<T>) {
        // Would move all elements from other to self
        self.length = self.length + other.length;
        other.length = 0;
    }

    // Split the Vec at an index
    fn split_off(self: &mut Vec<T>, at: i32) -> Vec<T> {
        if at >= 0 && at <= self.length {
            let new_vec = Vec::new();
            // Would move elements from at..length to new_vec
            self.length = at;
            return new_vec;
        } else {
            return Vec::new();
        }
    }

    // Resize the Vec to a new length, filling with a value
    fn resize(self: &mut Vec<T>, new_len: i32, value: T) {
        if new_len > self.length {
            // Would fill with copies of value
        } else if new_len < self.length {
            // Would truncate
        }
        self.length = new_len;
    }

    // Deduplicate consecutive equal elements
    fn dedup(self: &mut Vec<T>) {
        // Would remove consecutive duplicates
        // Requires T to implement equality
    }

    // Retain only elements that match a predicate
    fn retain(self: &mut Vec<T>, predicate: fn(&T) -> bool) {
        // Would keep only elements where predicate returns true
        // Would update length accordingly
    }

    // Convert to an array (if possible)
    fn to_array(self: Vec<T>) -> [T] {
        return self.data;
    }

    // Get first element
    fn first(self: &Vec<T>) -> Option<&T> {
        if self.length > 0 {
            return self.get(0);
        } else {
            return Option::None;
        }
    }

    // Get last element
    fn last(self: &Vec<T>) -> Option<&T> {
        if self.length > 0 {
            return self.get(self.length - 1);
        } else {
            return Option::None;
        }
    }

    // Get first element (mutable)
    fn first_mut(self: &mut Vec<T>) -> Option<&mut T> {
        if self.length > 0 {
            return self.get_mut(0);
        } else {
            return Option::None;
        }
    }

    // Get last element (mutable)
    fn last_mut(self: &mut Vec<T>) -> Option<&mut T> {
        if self.length > 0 {
            return self.get_mut(self.length - 1);
        } else {
            return Option::None;
        }
    }
}

// Implement Iterator for Vec<T>
impl<T> IntoIterator for Vec<T> {
    type Item = T;
    type IntoIter = VecIterator<T>;

    fn into_iter(self: Vec<T>) -> VecIterator<T> {
        return VecIterator {
            vec: self,
            index: 0,
        };
    }
}

struct VecIterator<T> {
    vec: Vec<T>,
    index: i32,
}

impl<T> Iterator for VecIterator<T> {
    type Item = T;

    fn next(self: &mut VecIterator<T>) -> Option<T> {
        if self.index < self.vec.length {
            let value = self.vec.get(self.index);
            self.index = self.index + 1;
            // Would return the actual value
            return Option::None;  // Placeholder
        } else {
            return Option::None;
        }
    }
}

// Implement indexing for Vec<T>
// vec[i] would desugar to vec.get(i).unwrap()
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec_definition_exists() {
        assert!(!VEC_DEFINITION.is_empty());
    }

    #[test]
    fn test_vec_definition_contains_struct() {
        assert!(VEC_DEFINITION.contains("struct Vec<T>"));
    }

    #[test]
    fn test_vec_definition_contains_methods() {
        assert!(VEC_DEFINITION.contains("fn new()"));
        assert!(VEC_DEFINITION.contains("fn push("));
        assert!(VEC_DEFINITION.contains("fn pop("));
        assert!(VEC_DEFINITION.contains("fn len("));
        assert!(VEC_DEFINITION.contains("fn is_empty("));
    }

    #[test]
    fn test_vec_definition_contains_iterator() {
        assert!(VEC_DEFINITION.contains("impl<T> IntoIterator for Vec<T>"));
        assert!(VEC_DEFINITION.contains("impl<T> Iterator for VecIterator<T>"));
    }
}
