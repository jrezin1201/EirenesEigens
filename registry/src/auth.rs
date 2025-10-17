use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const JWT_SECRET: &[u8] = b"your-secret-key-change-in-production"; // TODO: Load from env
const TOKEN_EXPIRY_DAYS: i64 = 30;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,      // User ID
    pub username: String, // Username
    pub exp: i64,         // Expiration time
    pub iat: i64,         // Issued at
}

/// Hash a password using Argon2id
pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();
    Ok(password_hash)
}

/// Verify a password against a hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hash)?;
    let argon2 = Argon2::default();
    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Generate a JWT token for a user
pub fn generate_token(user_id: Uuid, username: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let expiry = now + Duration::days(TOKEN_EXPIRY_DAYS);

    let claims = Claims {
        sub: user_id.to_string(),
        username: username.to_string(),
        exp: expiry.timestamp(),
        iat: now.timestamp(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET),
    )
}

/// Verify and decode a JWT token
pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let validation = Validation::default();
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET),
        &validation,
    )?;
    Ok(token_data.claims)
}

/// Extract token from Authorization header
pub fn extract_token(auth_header: &str) -> Option<&str> {
    auth_header.strip_prefix("Bearer ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "secure_password_123";
        let hash = hash_password(password).unwrap();

        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_token_generation() {
        let user_id = Uuid::new_v4();
        let username = "alice";

        let token = generate_token(user_id, username).unwrap();
        assert!(!token.is_empty());

        let claims = verify_token(&token).unwrap();
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.username, username);
    }

    #[test]
    fn test_extract_token() {
        let auth_header = "Bearer tok_abc123";
        assert_eq!(extract_token(auth_header), Some("tok_abc123"));

        let invalid_header = "Basic abc123";
        assert_eq!(extract_token(invalid_header), None);
    }
}
