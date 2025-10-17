use crate::error::{AppError, AppResult};
use validator::Validate;

/// Validate a request body
pub fn validate<T: Validate>(data: &T) -> AppResult<()> {
    data.validate()
        .map_err(|e| AppError::ValidationError(format!("Validation failed: {}", e)))
}

/// Validate package name (lowercase, alphanumeric, hyphens only)
pub fn validate_package_name(name: &str) -> AppResult<()> {
    if name.is_empty() || name.len() > 64 {
        return Err(AppError::ValidationError(
            "Package name must be 1-64 characters".to_string(),
        ));
    }

    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(AppError::ValidationError(
            "Package name must contain only lowercase letters, digits, and hyphens".to_string(),
        ));
    }

    if name.starts_with('-') || name.ends_with('-') {
        return Err(AppError::ValidationError(
            "Package name cannot start or end with a hyphen".to_string(),
        ));
    }

    Ok(())
}

/// Validate semantic version
pub fn validate_version(version: &str) -> AppResult<()> {
    semver::Version::parse(version)
        .map(|_| ())
        .map_err(|e| AppError::ValidationError(format!("Invalid version: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_name_validation() {
        assert!(validate_package_name("raven-ui").is_ok());
        assert!(validate_package_name("my-package-123").is_ok());

        assert!(validate_package_name("Invalid-Name").is_err()); // uppercase
        assert!(validate_package_name("-invalid").is_err()); // starts with hyphen
        assert!(validate_package_name("invalid-").is_err()); // ends with hyphen
        assert!(validate_package_name("").is_err()); // empty
    }

    #[test]
    fn test_version_validation() {
        assert!(validate_version("1.2.3").is_ok());
        assert!(validate_version("0.0.1").is_ok());
        assert!(validate_version("10.20.30").is_ok());

        assert!(validate_version("1.2").is_err()); // not semver
        assert!(validate_version("v1.2.3").is_err()); // has 'v' prefix
        assert!(validate_version("invalid").is_err());
    }
}
