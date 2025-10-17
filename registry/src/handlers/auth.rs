use axum::{extract::State, http::StatusCode, Json};
use crate::{
    auth::{hash_password, verify_password, generate_token},
    db::{AppState, UserDb},
    error::{AppError, AppResult},
    models::{LoginRequest, LoginResponse, RegisterRequest, RegisterResponse, UserInfo},
    validation::validate,
};

/// POST /auth/register - Register a new user
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> AppResult<(StatusCode, Json<RegisterResponse>)> {
    // Validate request
    validate(&req)?;

    // Check if username exists
    if UserDb::find_by_username(&state.pool, &req.username)
        .await?
        .is_some()
    {
        return Err(AppError::Conflict("Username already exists".to_string()));
    }

    // Check if email exists
    if UserDb::find_by_email(&state.pool, &req.email)
        .await?
        .is_some()
    {
        return Err(AppError::Conflict("Email already exists".to_string()));
    }

    // Hash password
    let password_hash = hash_password(&req.password)?;

    // Create user
    let user = UserDb::create(&state.pool, &req.username, &req.email, &password_hash).await?;

    // Generate token
    let token = generate_token(user.user_id, &user.username)?;

    Ok((
        StatusCode::CREATED,
        Json(RegisterResponse {
            user_id: user.user_id,
            username: user.username,
            email: user.email,
            created_at: user.created_at,
            token,
        }),
    ))
}

/// POST /auth/login - Authenticate and receive token
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<LoginResponse>> {
    // Find user by email
    let user = UserDb::find_by_email(&state.pool, &req.email)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

    // Verify password
    if !verify_password(&req.password, &user.password_hash)? {
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    // Generate token
    let token = generate_token(user.user_id, &user.username)?;

    // Calculate expiry (30 days from now)
    let expires_at = chrono::Utc::now() + chrono::Duration::days(30);

    Ok(Json(LoginResponse {
        token,
        expires_at,
        user: UserInfo {
            user_id: user.user_id,
            username: user.username,
            email: user.email,
        },
    }))
}

/// POST /auth/refresh - Refresh an expiring token
pub async fn refresh_token(
    State(_state): State<AppState>,
    // TODO: Extract current token from Authorization header
) -> AppResult<Json<serde_json::Value>> {
    // TODO: Implement token refresh logic
    Ok(Json(serde_json::json!({
        "message": "Token refresh not yet implemented"
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_handlers() {
        // Auth handler tests placeholder
        assert!(true);
    }
}
