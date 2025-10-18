use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

// ============================================================================
// Models
// ============================================================================

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct User {
    id: Uuid,
    email: String,
    #[serde(skip_serializing)]
    password_hash: String,
    name: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct Todo {
    id: Uuid,
    user_id: Uuid,
    title: String,
    description: Option<String>,
    completed: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

// ============================================================================
// Request/Response DTOs
// ============================================================================

#[derive(Debug, Deserialize)]
struct RegisterRequest {
    email: String,
    password: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct AuthResponse {
    token: String,
    user: User,
}

#[derive(Debug, Deserialize)]
struct CreateTodoRequest {
    title: String,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UpdateTodoRequest {
    title: Option<String>,
    description: Option<String>,
    completed: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Claims {
    sub: String, // user_id
    exp: usize,
}

// ============================================================================
// Application State
// ============================================================================

#[derive(Clone)]
struct AppState {
    db: PgPool,
    jwt_secret: String,
}

// ============================================================================
// Auth Handlers
// ============================================================================

async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    // Hash password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash password".to_string()))?
        .to_string();

    // Insert user
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (email, password_hash, name) VALUES ($1, $2, $3) RETURNING *",
    )
    .bind(&payload.email)
    .bind(&password_hash)
    .bind(&payload.name)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::BAD_REQUEST, format!("User already exists: {}", e)))?;

    // Generate JWT
    let token = generate_jwt(&user.id, &state.jwt_secret)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate token".to_string()))?;

    Ok(Json(AuthResponse { token, user }))
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    // Find user
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(&payload.email)
        .fetch_one(&state.db)
        .await
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()))?;

    // Verify password
    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Invalid password hash".to_string()))?;

    Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()))?;

    // Generate JWT
    let token = generate_jwt(&user.id, &state.jwt_secret)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate token".to_string()))?;

    Ok(Json(AuthResponse { token, user }))
}

// ============================================================================
// Auth Helper
// ============================================================================

fn extract_user_from_header(headers: &axum::http::HeaderMap, secret: &str) -> Result<Uuid, StatusCode> {
    let auth_header = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    verify_jwt(token, secret).map_err(|_| StatusCode::UNAUTHORIZED)
}

// ============================================================================
// Todo Handlers
// ============================================================================

async fn get_todos(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Vec<Todo>>, StatusCode> {
    let user_id = extract_user_from_header(&headers, &state.jwt_secret)?;

    let todos = sqlx::query_as::<_, Todo>(
        "SELECT * FROM todos WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(todos))
}

async fn create_todo(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<CreateTodoRequest>,
) -> Result<Json<Todo>, StatusCode> {
    let user_id = extract_user_from_header(&headers, &state.jwt_secret)?;

    let todo = sqlx::query_as::<_, Todo>(
        "INSERT INTO todos (user_id, title, description) VALUES ($1, $2, $3) RETURNING *",
    )
    .bind(user_id)
    .bind(&payload.title)
    .bind(&payload.description)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(todo))
}

async fn update_todo(
    State(state): State<Arc<AppState>>,
    Path(todo_id): Path<Uuid>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<UpdateTodoRequest>,
) -> Result<Json<Todo>, StatusCode> {
    let user_id = extract_user_from_header(&headers, &state.jwt_secret)?;

    // Verify ownership
    let existing = sqlx::query!("SELECT user_id FROM todos WHERE id = $1", todo_id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    if existing.user_id != user_id {
        return Err(StatusCode::FORBIDDEN);
    }

    // Update todo
    let todo = sqlx::query_as::<_, Todo>(
        r#"
        UPDATE todos
        SET title = COALESCE($1, title),
            description = COALESCE($2, description),
            completed = COALESCE($3, completed),
            updated_at = NOW()
        WHERE id = $4
        RETURNING *
        "#,
    )
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(&payload.completed)
    .bind(todo_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(todo))
}

async fn delete_todo(
    State(state): State<Arc<AppState>>,
    Path(todo_id): Path<Uuid>,
    headers: axum::http::HeaderMap,
) -> Result<StatusCode, StatusCode> {
    let user_id = extract_user_from_header(&headers, &state.jwt_secret)?;

    // Verify ownership
    let existing = sqlx::query!("SELECT user_id FROM todos WHERE id = $1", todo_id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    if existing.user_id != user_id {
        return Err(StatusCode::FORBIDDEN);
    }

    sqlx::query!("DELETE FROM todos WHERE id = $1", todo_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// JWT Utilities
// ============================================================================

fn generate_jwt(user_id: &Uuid, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(30))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

fn verify_jwt(token: &str, secret: &str) -> Result<Uuid, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;

    Uuid::parse_str(&token_data.claims.sub)
        .map_err(|_| jsonwebtoken::errors::ErrorKind::InvalidToken.into())
}

// ============================================================================
// Routes
// ============================================================================

fn create_router(state: Arc<AppState>) -> Router {
    let auth_routes = Router::new()
        .route("/register", post(register))
        .route("/login", post(login));

    let todo_routes = Router::new()
        .route("/todos", get(get_todos))
        .route("/todos", post(create_todo))
        .route("/todos/:id", put(update_todo))
        .route("/todos/:id", delete(delete_todo));

    Router::new()
        .nest("/api/auth", auth_routes)
        .nest("/api", todo_routes)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state)
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "taskflow_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/taskflow".to_string());
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "super-secret-jwt-key-change-in-production".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    tracing::info!("✅ Connected to database");

    // Run migrations
    sqlx::query(include_str!("../schema.sql"))
        .execute(&pool)
        .await
        .ok(); // Ignore errors if tables already exist

    let state = Arc::new(AppState {
        db: pool,
        jwt_secret,
    });

    let app = create_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("🚀 TaskFlow backend listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}
