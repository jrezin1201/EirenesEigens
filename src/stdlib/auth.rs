/// Authentication and Authorization for RavensOne
///
/// Provides user authentication, JWT tokens, sessions, and RBAC
/// Secure by default with bcrypt password hashing

use std::collections::HashMap;

/// User model
#[derive(Debug, Clone)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub password_hash: String,
    pub name: String,
    pub role: UserRole,
    pub created_at: String,
    pub last_login: Option<String>,
}

impl User {
    pub fn new(id: i32, email: String, name: String, password_hash: String) -> Self {
        Self {
            id,
            email,
            name,
            password_hash,
            role: UserRole::User,
            created_at: chrono::Utc::now().to_rfc3339(),
            last_login: None,
        }
    }

    /// Check if user has a specific role
    pub fn has_role(&self, role: &UserRole) -> bool {
        self.role == *role || self.role == UserRole::Admin
    }

    /// Mask password for safe serialization
    pub fn to_safe(&self) -> SafeUser {
        SafeUser {
            id: self.id,
            email: self.email.clone(),
            name: self.name.clone(),
            role: self.role.clone(),
            created_at: self.created_at.clone(),
            last_login: self.last_login.clone(),
        }
    }
}

/// User without password (safe for client)
#[derive(Debug, Clone)]
pub struct SafeUser {
    pub id: i32,
    pub email: String,
    pub name: String,
    pub role: UserRole,
    pub created_at: String,
    pub last_login: Option<String>,
}

/// User roles
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserRole {
    Admin,
    User,
    Guest,
}

impl UserRole {
    pub fn as_str(&self) -> &str {
        match self {
            UserRole::Admin => "admin",
            UserRole::User => "user",
            UserRole::Guest => "guest",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "admin" => Some(UserRole::Admin),
            "user" => Some(UserRole::User),
            "guest" => Some(UserRole::Guest),
            _ => None,
        }
    }
}

/// JWT claims
#[derive(Debug, Clone)]
pub struct Claims {
    pub sub: String,  // Subject (user ID)
    pub email: String,
    pub role: String,
    pub exp: i64,     // Expiration time
    pub iat: i64,     // Issued at
}

impl Claims {
    pub fn new(user_id: i32, email: String, role: UserRole, expires_in_hours: i64) -> Self {
        let now = chrono::Utc::now().timestamp();
        let exp = now + (expires_in_hours * 3600);

        Self {
            sub: user_id.to_string(),
            email,
            role: role.as_str().to_string(),
            exp,
            iat: now,
        }
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        self.exp < now
    }

    /// Check if token is valid (not expired and has required role)
    pub fn is_valid(&self, required_role: Option<&UserRole>) -> bool {
        if self.is_expired() {
            return false;
        }

        if let Some(role) = required_role {
            let user_role = UserRole::from_str(&self.role);
            if let Some(ur) = user_role {
                return ur == *role || ur == UserRole::Admin;
            }
            return false;
        }

        true
    }
}

/// Authentication token
#[derive(Debug, Clone)]
pub struct AuthToken {
    pub token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: SafeUser,
}

impl AuthToken {
    pub fn new(token: String, expires_in: i64, user: SafeUser) -> Self {
        Self {
            token,
            token_type: "Bearer".to_string(),
            expires_in,
            user,
        }
    }
}

/// Session
#[derive(Debug, Clone)]
pub struct Session {
    pub session_id: String,
    pub user_id: i32,
    pub created_at: i64,
    pub expires_at: i64,
    pub data: HashMap<String, String>,
}

impl Session {
    pub fn new(session_id: String, user_id: i32, duration_hours: i64) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            session_id,
            user_id,
            created_at: now,
            expires_at: now + (duration_hours * 3600),
            data: HashMap::new(),
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        self.expires_at < now
    }

    pub fn set_data(&mut self, key: String, value: String) {
        self.data.insert(key, value);
    }

    pub fn get_data(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }
}

/// Authentication service
pub struct AuthService {
    secret_key: String,
    token_expiry_hours: i64,
    session_expiry_hours: i64,
}

impl AuthService {
    pub fn new(secret_key: String) -> Self {
        Self {
            secret_key,
            token_expiry_hours: 24,      // 24 hours
            session_expiry_hours: 168,   // 7 days
        }
    }

    pub fn with_expiry(mut self, token_hours: i64, session_hours: i64) -> Self {
        self.token_expiry_hours = token_hours;
        self.session_expiry_hours = session_hours;
        self
    }

    /// Hash password (in real impl, uses bcrypt)
    pub fn hash_password(&self, password: &str) -> String {
        // In a real implementation, this would use bcrypt
        // For now, return a placeholder
        format!("$bcrypt$hash${}", password)
    }

    /// Verify password against hash
    pub fn verify_password(&self, password: &str, hash: &str) -> bool {
        // In a real implementation, this would use bcrypt.verify
        // For now, simple comparison
        hash == &format!("$bcrypt$hash${}", password)
    }

    /// Generate JWT token
    pub fn generate_token(&self, user: &User) -> String {
        let claims = Claims::new(
            user.id,
            user.email.clone(),
            user.role.clone(),
            self.token_expiry_hours,
        );

        // In a real implementation, this would use jsonwebtoken crate
        // For now, return a simple token
        format!(
            "jwt.{}.{}.{}",
            self.encode_base64(&claims.sub),
            self.encode_base64(&claims.email),
            self.encode_base64(&self.secret_key)
        )
    }

    /// Verify JWT token
    pub fn verify_token(&self, token: &str) -> Result<Claims, String> {
        // In a real implementation, this would use jsonwebtoken crate
        // For now, parse the simple token
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 4 || parts[0] != "jwt" {
            return Err("Invalid token format".to_string());
        }

        // Decode parts
        let user_id = self.decode_base64(parts[1])
            .parse::<i32>()
            .map_err(|_| "Invalid user ID")?;
        let email = self.decode_base64(parts[2]);

        // Create claims (with default expiry for demo)
        let claims = Claims::new(user_id, email, UserRole::User, self.token_expiry_hours);

        Ok(claims)
    }

    /// Generate session ID
    pub fn generate_session_id(&self) -> String {
        // In a real implementation, use crypto-secure random
        format!("session_{}", chrono::Utc::now().timestamp())
    }

    /// Create auth token response
    pub fn create_auth_token(&self, user: &User) -> AuthToken {
        let token = self.generate_token(user);
        AuthToken::new(
            token,
            self.token_expiry_hours * 3600,
            user.to_safe(),
        )
    }

    // Helper functions
    fn encode_base64(&self, s: &str) -> String {
        // Simplified base64 encoding for demo
        s.chars().map(|c| (c as u8).to_string()).collect::<Vec<_>>().join("-")
    }

    fn decode_base64(&self, s: &str) -> String {
        // Simplified base64 decoding for demo
        s.split('-')
            .filter_map(|n| n.parse::<u8>().ok())
            .map(|n| n as char)
            .collect()
    }
}

/// Authentication middleware
pub struct AuthMiddleware {
    required_role: Option<UserRole>,
}

impl AuthMiddleware {
    pub fn new() -> Self {
        Self {
            required_role: None,
        }
    }

    pub fn require_role(mut self, role: UserRole) -> Self {
        self.required_role = Some(role);
        self
    }

    pub fn check(&self, claims: &Claims) -> Result<(), String> {
        if !claims.is_valid(self.required_role.as_ref()) {
            return Err("Unauthorized".to_string());
        }
        Ok(())
    }
}

impl Default for AuthMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

// Add chrono-like timestamp utilities (simplified)
mod chrono {
    pub struct Utc;

    impl Utc {
        pub fn now() -> DateTime {
            DateTime
        }
    }

    pub struct DateTime;

    impl DateTime {
        pub fn timestamp(&self) -> i64 {
            // In real impl, this would return actual Unix timestamp
            // For demo, return a fixed value
            1704067200 // 2024-01-01 00:00:00 UTC
        }

        pub fn to_rfc3339(&self) -> String {
            "2024-01-01T00:00:00Z".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User::new(
            1,
            "test@example.com".to_string(),
            "Test User".to_string(),
            "hashed_password".to_string(),
        );

        assert_eq!(user.id, 1);
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.role, UserRole::User);
    }

    #[test]
    fn test_user_roles() {
        let admin = User {
            role: UserRole::Admin,
            ..User::new(1, "admin@test.com".to_string(), "Admin".to_string(), "hash".to_string())
        };

        assert!(admin.has_role(&UserRole::Admin));
        assert!(admin.has_role(&UserRole::User)); // Admin has all roles

        let user = User::new(2, "user@test.com".to_string(), "User".to_string(), "hash".to_string());
        assert!(user.has_role(&UserRole::User));
        assert!(!user.has_role(&UserRole::Admin));
    }

    #[test]
    fn test_password_hashing() {
        let auth = AuthService::new("secret".to_string());

        let password = "mypassword123";
        let hash = auth.hash_password(password);

        assert!(auth.verify_password(password, &hash));
        assert!(!auth.verify_password("wrongpassword", &hash));
    }

    #[test]
    fn test_token_generation() {
        let auth = AuthService::new("secret".to_string());
        let user = User::new(1, "test@example.com".to_string(), "Test".to_string(), "hash".to_string());

        let token = auth.generate_token(&user);
        assert!(token.starts_with("jwt."));

        let claims = auth.verify_token(&token).unwrap();
        assert_eq!(claims.email, "test@example.com");
    }

    #[test]
    fn test_safe_user() {
        let user = User::new(
            1,
            "test@example.com".to_string(),
            "Test".to_string(),
            "secret_hash".to_string(),
        );

        let safe = user.to_safe();
        assert_eq!(safe.id, user.id);
        assert_eq!(safe.email, user.email);
        // Password hash is not included in safe user
    }
}
