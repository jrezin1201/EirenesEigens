/// Standard library Result<T, E> type implementation
///
/// The Result type represents either success (Ok) or failure (Err). It's used for
/// functions that can fail in a predictable way. This is similar to Rust's Result type
/// and provides a type-safe way to handle errors without exceptions.

/// Result type definition in RavensOne syntax
pub const RESULT_DEFINITION: &str = r#"
// The Result type - a type that represents either success (Ok) or an error (Err)
enum Result<T, E> {
    Ok(T),
    Err(E),
}

// Result methods implementation
impl<T, E> Result<T, E> {
    // Returns true if the result is Ok
    fn is_ok(self: &Result<T, E>) -> bool {
        match self {
            Result::Ok(_) => true,
            Result::Err(_) => false,
        }
    }

    // Returns true if the result is Err
    fn is_err(self: &Result<T, E>) -> bool {
        match self {
            Result::Ok(_) => false,
            Result::Err(_) => true,
        }
    }

    // Returns the contained Ok value
    // Panics if the value is an Err
    fn unwrap(self: Result<T, E>) -> T {
        match self {
            Result::Ok(value) => value,
            Result::Err(_) => {
                // In a full implementation, this would panic with the error
                // For now, we'll just return a placeholder
                value
            },
        }
    }

    // Returns the contained Err value
    // Panics if the value is Ok
    fn unwrap_err(self: Result<T, E>) -> E {
        match self {
            Result::Ok(_) => {
                // In a full implementation, this would panic
                // For now, we'll just return a placeholder
                err
            },
            Result::Err(err) => err,
        }
    }

    // Returns the contained Ok value or a provided default
    fn unwrap_or(self: Result<T, E>, default: T) -> T {
        match self {
            Result::Ok(value) => value,
            Result::Err(_) => default,
        }
    }

    // Returns the contained Ok value or computes it from the error
    fn unwrap_or_else(self: Result<T, E>, f: fn(E) -> T) -> T {
        match self {
            Result::Ok(value) => value,
            Result::Err(err) => f(err),
        }
    }

    // Maps a Result<T, E> to Result<U, E> by applying a function to the Ok value
    fn map<U>(self: Result<T, E>, f: fn(T) -> U) -> Result<U, E> {
        match self {
            Result::Ok(value) => Result::Ok(f(value)),
            Result::Err(err) => Result::Err(err),
        }
    }

    // Maps a Result<T, E> to Result<T, F> by applying a function to the Err value
    fn map_err<F>(self: Result<T, E>, f: fn(E) -> F) -> Result<T, F> {
        match self {
            Result::Ok(value) => Result::Ok(value),
            Result::Err(err) => Result::Err(f(err)),
        }
    }

    // Calls the function if the result is Ok, otherwise returns the Err value
    fn and_then<U>(self: Result<T, E>, f: fn(T) -> Result<U, E>) -> Result<U, E> {
        match self {
            Result::Ok(value) => f(value),
            Result::Err(err) => Result::Err(err),
        }
    }

    // Returns res if the result is Err, otherwise returns the Ok value of self
    fn or(self: Result<T, E>, res: Result<T, E>) -> Result<T, E> {
        match self {
            Result::Ok(_) => self,
            Result::Err(_) => res,
        }
    }

    // Calls op if the result is Err, otherwise returns the Ok value of self
    fn or_else<F>(self: Result<T, E>, op: fn(E) -> Result<T, F>) -> Result<T, F> {
        match self {
            Result::Ok(value) => Result::Ok(value),
            Result::Err(err) => op(err),
        }
    }

    // Converts from Result<T, E> to Option<T>
    fn ok(self: Result<T, E>) -> Option<T> {
        match self {
            Result::Ok(value) => Option::Some(value),
            Result::Err(_) => Option::None,
        }
    }

    // Converts from Result<T, E> to Option<E>
    fn err(self: Result<T, E>) -> Option<E> {
        match self {
            Result::Ok(_) => Option::None,
            Result::Err(err) => Option::Some(err),
        }
    }

    // Converts from &Result<T, E> to Result<&T, &E>
    fn as_ref(self: &Result<T, E>) -> Result<&T, &E> {
        match self {
            Result::Ok(value) => Result::Ok(value),
            Result::Err(err) => Result::Err(err),
        }
    }

    // Transposes a Result of an Option into an Option of a Result
    fn transpose(self: Result<Option<T>, E>) -> Option<Result<T, E>> {
        match self {
            Result::Ok(opt) => {
                match opt {
                    Option::Some(value) => Option::Some(Result::Ok(value)),
                    Option::None => Option::None,
                }
            },
            Result::Err(err) => Option::Some(Result::Err(err)),
        }
    }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_definition_exists() {
        assert!(!RESULT_DEFINITION.is_empty());
        assert!(RESULT_DEFINITION.contains("enum Result<T, E>"));
        assert!(RESULT_DEFINITION.contains("Ok(T)"));
        assert!(RESULT_DEFINITION.contains("Err(E)"));
    }

    #[test]
    fn test_result_has_methods() {
        assert!(RESULT_DEFINITION.contains("fn is_ok"));
        assert!(RESULT_DEFINITION.contains("fn is_err"));
        assert!(RESULT_DEFINITION.contains("fn unwrap"));
        assert!(RESULT_DEFINITION.contains("fn unwrap_err"));
        assert!(RESULT_DEFINITION.contains("fn unwrap_or"));
        assert!(RESULT_DEFINITION.contains("fn map"));
        assert!(RESULT_DEFINITION.contains("fn map_err"));
        assert!(RESULT_DEFINITION.contains("fn and_then"));
    }

    #[test]
    fn test_result_impl_block() {
        assert!(RESULT_DEFINITION.contains("impl<T, E> Result<T, E>"));
    }

    #[test]
    fn test_result_conversion_methods() {
        assert!(RESULT_DEFINITION.contains("fn ok"));
        assert!(RESULT_DEFINITION.contains("fn err"));
        assert!(RESULT_DEFINITION.contains("fn transpose"));
    }
}
