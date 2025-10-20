/// Standard library Iterator trait definition
/// This allows for iteration over collections using for-in loops

pub const ITERATOR_DEFINITION: &str = r#"
// The core Iterator trait
// Types that implement this trait can be iterated over
trait Iterator {
    type Item;  // Associated type for the element type

    // Returns the next element in the iteration, or None when finished
    fn next(self: &mut Self) -> Option<Self::Item>;
}

// Trait for types that can be converted into an iterator
trait IntoIterator {
    type Item;
    type IntoIter: Iterator;  // The iterator type this converts to

    // Converts self into an iterator
    fn into_iter(self: Self) -> Self::IntoIter;
}

// Iterator implementation for arrays
// Note: This is a conceptual implementation - actual implementation
// would need compiler support for generic array iteration
impl<T> IntoIterator for [T] {
    type Item = T;
    type IntoIter = ArrayIterator<T>;

    fn into_iter(self: [T]) -> ArrayIterator<T> {
        return ArrayIterator::new(self);
    }
}

// Array iterator helper struct
struct ArrayIterator<T> {
    array: [T],
    index: i32,
    length: i32,
}

impl<T> ArrayIterator<T> {
    fn new(array: [T]) -> ArrayIterator<T> {
        return ArrayIterator {
            array: array,
            index: 0,
            length: 0,  // Would be array.len() in real implementation
        };
    }
}

impl<T> Iterator for ArrayIterator<T> {
    type Item = T;

    fn next(self: &mut ArrayIterator<T>) -> Option<T> {
        if self.index < self.length {
            let value = self.array[self.index];
            self.index = self.index + 1;
            return Option::Some(value);
        } else {
            return Option::None;
        }
    }
}

// Range iterator implementation
impl Iterator for Range {
    type Item = i32;

    fn next(self: &mut Range) -> Option<i32> {
        if self.current < self.end {
            let value = self.current;
            self.current = self.current + 1;
            return Option::Some(value);
        } else {
            return Option::None;
        }
    }
}

impl IntoIterator for Range {
    type Item = i32;
    type IntoIter = Range;

    fn into_iter(self: Range) -> Range {
        return self;
    }
}

// Option iterator - iterates 0 or 1 times
impl<T> IntoIterator for Option<T> {
    type Item = T;
    type IntoIter = OptionIterator<T>;

    fn into_iter(self: Option<T>) -> OptionIterator<T> {
        return OptionIterator { option: self };
    }
}

struct OptionIterator<T> {
    option: Option<T>,
}

impl<T> Iterator for OptionIterator<T> {
    type Item = T;

    fn next(self: &mut OptionIterator<T>) -> Option<T> {
        match self.option {
            Option::Some(value) => {
                self.option = Option::None;
                return Option::Some(value);
            },
            Option::None => {
                return Option::None;
            },
        }
    }
}

// Iterator combinators - methods that transform iterators

// Extension methods for Iterator trait
impl<T> Iterator {
    // Map: transforms each element using a function
    fn map<U>(self: Self, f: fn(T) -> U) -> MapIterator<Self, U> {
        return MapIterator {
            iter: self,
            func: f,
        };
    }

    // Filter: only keeps elements that satisfy a predicate
    fn filter(self: Self, predicate: fn(&T) -> bool) -> FilterIterator<Self> {
        return FilterIterator {
            iter: self,
            predicate: predicate,
        };
    }

    // Take: takes only the first n elements
    fn take(self: Self, n: i32) -> TakeIterator<Self> {
        return TakeIterator {
            iter: self,
            remaining: n,
        };
    }

    // Skip: skips the first n elements
    fn skip(self: Self, n: i32) -> SkipIterator<Self> {
        return SkipIterator {
            iter: self,
            to_skip: n,
        };
    }

    // Collect: consumes the iterator and collects into a collection
    fn collect(self: Self) -> [T] {
        let result = [];
        // Would append each element to result
        return result;
    }

    // Count: counts the number of elements
    fn count(self: Self) -> i32 {
        let count = 0;
        // Would iterate and count
        return count;
    }

    // Find: finds the first element matching a predicate
    fn find(self: Self, predicate: fn(&T) -> bool) -> Option<T> {
        // Would iterate until predicate returns true
        return Option::None;
    }

    // Any: checks if any element satisfies the predicate
    fn any(self: Self, predicate: fn(&T) -> bool) -> bool {
        // Would iterate until predicate returns true
        return false;
    }

    // All: checks if all elements satisfy the predicate
    fn all(self: Self, predicate: fn(&T) -> bool) -> bool {
        // Would iterate and check all elements
        return true;
    }

    // Fold: reduces the iterator to a single value
    fn fold<B>(self: Self, init: B, f: fn(B, T) -> B) -> B {
        let acc = init;
        // Would iterate and apply f to accumulate
        return acc;
    }
}

// Helper iterator types for combinators

struct MapIterator<I, U> {
    iter: I,
    func: fn(T) -> U,
}

impl<I: Iterator, U> Iterator for MapIterator<I, U> {
    type Item = U;

    fn next(self: &mut MapIterator<I, U>) -> Option<U> {
        match self.iter.next() {
            Option::Some(value) => {
                return Option::Some((self.func)(value));
            },
            Option::None => {
                return Option::None;
            },
        }
    }
}

struct FilterIterator<I> {
    iter: I,
    predicate: fn(&T) -> bool,
}

impl<I: Iterator> Iterator for FilterIterator<I> {
    type Item = I::Item;

    fn next(self: &mut FilterIterator<I>) -> Option<I::Item> {
        // Would loop until finding an element that matches predicate
        return Option::None;
    }
}

struct TakeIterator<I> {
    iter: I,
    remaining: i32,
}

impl<I: Iterator> Iterator for TakeIterator<I> {
    type Item = I::Item;

    fn next(self: &mut TakeIterator<I>) -> Option<I::Item> {
        if self.remaining > 0 {
            self.remaining = self.remaining - 1;
            return self.iter.next();
        } else {
            return Option::None;
        }
    }
}

struct SkipIterator<I> {
    iter: I,
    to_skip: i32,
}

impl<I: Iterator> Iterator for SkipIterator<I> {
    type Item = I::Item;

    fn next(self: &mut SkipIterator<I>) -> Option<I::Item> {
        // Skip the remaining elements first
        for i in 0..self.to_skip {
            self.iter.next();
        }
        self.to_skip = 0;

        return self.iter.next();
    }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iterator_definition_exists() {
        assert!(!ITERATOR_DEFINITION.is_empty());
    }

    #[test]
    fn test_iterator_definition_contains_trait() {
        assert!(ITERATOR_DEFINITION.contains("trait Iterator"));
    }

    #[test]
    fn test_iterator_definition_contains_into_iterator() {
        assert!(ITERATOR_DEFINITION.contains("trait IntoIterator"));
    }

    #[test]
    fn test_iterator_definition_contains_combinators() {
        assert!(ITERATOR_DEFINITION.contains("fn map"));
        assert!(ITERATOR_DEFINITION.contains("fn filter"));
        assert!(ITERATOR_DEFINITION.contains("fn take"));
        assert!(ITERATOR_DEFINITION.contains("fn skip"));
    }
}
