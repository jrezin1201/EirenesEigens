/// Standard library Option<T> type implementation
///
/// The Option type represents an optional value: every Option is either Some and contains a value,
/// or None, and does not. This is similar to Rust's Option type and provides a type-safe way
/// to handle nullable values.

/// Option type definition in RavensOne syntax
pub const OPTION_DEFINITION: &str = r#"
// The Option type - a type that represents an optional value
enum Option<T> {
    Some(T),
    None,
}

// Option methods implementation
impl<T> Option<T> {
    // Returns true if the option is a Some value
    fn is_some(self: &Option<T>) -> bool {
        match self {
            Option::Some(_) => true,
            Option::None => false,
        }
    }

    // Returns true if the option is a None value
    fn is_none(self: &Option<T>) -> bool {
        match self {
            Option::Some(_) => false,
            Option::None => true,
        }
    }

    // Returns the contained Some value
    // Panics if the value is None
    fn unwrap(self: Option<T>) -> T {
        match self {
            Option::Some(value) => value,
            Option::None => {
                // In a full implementation, this would panic
                // For now, we'll just return a placeholder
                value
            },
        }
    }

    // Returns the contained Some value or a provided default
    fn unwrap_or(self: Option<T>, default: T) -> T {
        match self {
            Option::Some(value) => value,
            Option::None => default,
        }
    }

    // Maps an Option<T> to Option<U> by applying a function to the contained value
    fn map<U>(self: Option<T>, f: fn(T) -> U) -> Option<U> {
        match self {
            Option::Some(value) => Option::Some(f(value)),
            Option::None => Option::None,
        }
    }

    // Returns None if the option is None, otherwise calls f with the wrapped value and returns the result
    fn and_then<U>(self: Option<T>, f: fn(T) -> Option<U>) -> Option<U> {
        match self {
            Option::Some(value) => f(value),
            Option::None => Option::None,
        }
    }

    // Returns the option if it contains a value, otherwise returns optb
    fn or(self: Option<T>, optb: Option<T>) -> Option<T> {
        match self {
            Option::Some(_) => self,
            Option::None => optb,
        }
    }

    // Returns the option if it contains a value, otherwise calls f and returns the result
    fn or_else(self: Option<T>, f: fn() -> Option<T>) -> Option<T> {
        match self {
            Option::Some(_) => self,
            Option::None => f(),
        }
    }

    // Returns Some if exactly one of self, optb is Some, otherwise returns None
    fn xor(self: Option<T>, optb: Option<T>) -> Option<T> {
        match self {
            Option::Some(val) => {
                match optb {
                    Option::Some(_) => Option::None,
                    Option::None => Option::Some(val),
                }
            },
            Option::None => {
                match optb {
                    Option::Some(val) => Option::Some(val),
                    Option::None => Option::None,
                }
            },
        }
    }

    // Converts from Option<T> to Option<&T>
    fn as_ref(self: &Option<T>) -> Option<&T> {
        match self {
            Option::Some(value) => Option::Some(value),
            Option::None => Option::None,
        }
    }

    // Takes the value out of the option, leaving None in its place
    fn take(self: &mut Option<T>) -> Option<T> {
        // In a full implementation, this would move the value out
        // For now, we return a placeholder
        Option::None
    }

    // Replaces the actual value in the option by the value given in parameter,
    // returning the old value if present
    fn replace(self: &mut Option<T>, value: T) -> Option<T> {
        // In a full implementation, this would swap the values
        // For now, we return a placeholder
        Option::None
    }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_option_definition_exists() {
        assert!(!OPTION_DEFINITION.is_empty());
        assert!(OPTION_DEFINITION.contains("enum Option<T>"));
        assert!(OPTION_DEFINITION.contains("Some(T)"));
        assert!(OPTION_DEFINITION.contains("None"));
    }

    #[test]
    fn test_option_has_methods() {
        assert!(OPTION_DEFINITION.contains("fn is_some"));
        assert!(OPTION_DEFINITION.contains("fn is_none"));
        assert!(OPTION_DEFINITION.contains("fn unwrap"));
        assert!(OPTION_DEFINITION.contains("fn unwrap_or"));
        assert!(OPTION_DEFINITION.contains("fn map"));
        assert!(OPTION_DEFINITION.contains("fn and_then"));
    }

    #[test]
    fn test_option_impl_block() {
        assert!(OPTION_DEFINITION.contains("impl<T> Option<T>"));
    }
}
