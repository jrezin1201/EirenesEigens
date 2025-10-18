use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::{delete, get, post},
    Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use std::{
    collections::HashMap,
    sync::Arc,
};
use tokio::sync::{broadcast, RwLock};
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

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
struct User {
    id: Uuid,
    username: String,
    #[serde(skip_serializing)]
    password_hash: String,
    display_name: String,
    avatar_url: Option<String>,
    status: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
struct Room {
    id: Uuid,
    name: String,
    description: Option<String>,
    is_private: bool,
    created_by: Option<Uuid>,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
struct ChatMessage {
    id: Uuid,
    room_id: Uuid,
    user_id: Uuid,
    content: String,
    message_type: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct MessageWithUser {
    #[serde(flatten)]
    message: ChatMessage,
    username: String,
    display_name: String,
    avatar_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct DirectMessage {
    id: Uuid,
    from_user_id: Uuid,
    to_user_id: Uuid,
    content: String,
    read: bool,
    created_at: DateTime<Utc>,
}

// ============================================================================
// Request/Response DTOs
// ============================================================================

#[derive(Debug, Deserialize)]
struct RegisterRequest {
    username: String,
    password: String,
    display_name: String,
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct AuthResponse {
    token: String,
    user: User,
}

#[derive(Debug, Deserialize)]
struct CreateRoomRequest {
    name: String,
    description: Option<String>,
    is_private: bool,
}

#[derive(Debug, Deserialize)]
struct SendMessageRequest {
    content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct WsMessage {
    #[serde(rename = "type")]
    msg_type: String,
    room_id: Option<Uuid>,
    message: Option<MessageWithUser>,
    user: Option<User>,
    users: Option<Vec<User>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Claims {
    sub: String, // user_id
    exp: usize,
}

// ============================================================================
// Application State
// ============================================================================

type Tx = broadcast::Sender<WsMessage>;

#[derive(Clone)]
struct AppState {
    db: PgPool,
    jwt_secret: String,
    tx: Tx,
    online_users: Arc<RwLock<HashMap<Uuid, String>>>, // user_id -> username
}

// ============================================================================
// Auth Handlers
// ============================================================================

async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash password".to_string()))?
        .to_string();

    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (username, password_hash, display_name) VALUES ($1, $2, $3) RETURNING *",
    )
    .bind(&payload.username)
    .bind(&password_hash)
    .bind(&payload.display_name)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::BAD_REQUEST, format!("Username already exists: {}", e)))?;

    let token = generate_jwt(&user.id, &state.jwt_secret)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate token".to_string()))?;

    Ok(Json(AuthResponse { token, user }))
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
        .bind(&payload.username)
        .fetch_one(&state.db)
        .await
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()))?;

    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Invalid password hash".to_string()))?;

    Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()))?;

    let token = generate_jwt(&user.id, &state.jwt_secret)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate token".to_string()))?;

    Ok(Json(AuthResponse { token, user }))
}

// ============================================================================
// Room Handlers
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

async fn get_rooms(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Room>>, StatusCode> {
    let rooms = sqlx::query_as::<_, Room>(
        "SELECT * FROM rooms WHERE is_private = false ORDER BY created_at ASC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(rooms))
}

async fn create_room(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<CreateRoomRequest>,
) -> Result<Json<Room>, StatusCode> {
    let user_id = extract_user_from_header(&headers, &state.jwt_secret)?;

    let room = sqlx::query_as::<_, Room>(
        "INSERT INTO rooms (name, description, is_private, created_by)
         VALUES ($1, $2, $3, $4) RETURNING *"
    )
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(payload.is_private)
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Auto-join creator to the room
    sqlx::query(
        "INSERT INTO room_members (room_id, user_id) VALUES ($1, $2)"
    )
    .bind(room.id)
    .bind(user_id)
    .execute(&state.db)
    .await
    .ok();

    Ok(Json(room))
}

async fn join_room(
    State(state): State<Arc<AppState>>,
    Path(room_id): Path<Uuid>,
    headers: axum::http::HeaderMap,
) -> Result<StatusCode, StatusCode> {
    let user_id = extract_user_from_header(&headers, &state.jwt_secret)?;

    sqlx::query(
        "INSERT INTO room_members (room_id, user_id) VALUES ($1, $2)
         ON CONFLICT (room_id, user_id) DO NOTHING"
    )
    .bind(room_id)
    .bind(user_id)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

async fn leave_room(
    State(state): State<Arc<AppState>>,
    Path(room_id): Path<Uuid>,
    headers: axum::http::HeaderMap,
) -> Result<StatusCode, StatusCode> {
    let user_id = extract_user_from_header(&headers, &state.jwt_secret)?;

    sqlx::query(
        "DELETE FROM room_members WHERE room_id = $1 AND user_id = $2"
    )
    .bind(room_id)
    .bind(user_id)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Message Handlers
// ============================================================================

async fn get_room_messages(
    State(state): State<Arc<AppState>>,
    Path(room_id): Path<Uuid>,
) -> Result<Json<Vec<MessageWithUser>>, StatusCode> {
    let rows = sqlx::query(
        "SELECT m.id, m.room_id, m.user_id, m.content, m.message_type, m.created_at,
                u.username, u.display_name, u.avatar_url
         FROM messages m
         JOIN users u ON m.user_id = u.id
         WHERE m.room_id = $1
         ORDER BY m.created_at ASC
         LIMIT 100"
    )
    .bind(room_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let messages: Vec<MessageWithUser> = rows
        .into_iter()
        .map(|row| MessageWithUser {
            message: ChatMessage {
                id: row.get("id"),
                room_id: row.get("room_id"),
                user_id: row.get("user_id"),
                content: row.get("content"),
                message_type: row.get("message_type"),
                created_at: row.get("created_at"),
            },
            username: row.get("username"),
            display_name: row.get("display_name"),
            avatar_url: row.get("avatar_url"),
        })
        .collect();

    Ok(Json(messages))
}

async fn send_message(
    State(state): State<Arc<AppState>>,
    Path(room_id): Path<Uuid>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<SendMessageRequest>,
) -> Result<Json<MessageWithUser>, StatusCode> {
    let user_id = extract_user_from_header(&headers, &state.jwt_secret)?;

    let message = sqlx::query_as::<_, ChatMessage>(
        "INSERT INTO messages (room_id, user_id, content, message_type)
         VALUES ($1, $2, $3, 'text') RETURNING *"
    )
    .bind(room_id)
    .bind(user_id)
    .bind(&payload.content)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let message_with_user = MessageWithUser {
        message: message.clone(),
        username: user.username.clone(),
        display_name: user.display_name.clone(),
        avatar_url: user.avatar_url.clone(),
    };

    // Broadcast to WebSocket clients
    let _ = state.tx.send(WsMessage {
        msg_type: "message".to_string(),
        room_id: Some(room_id),
        message: Some(message_with_user.clone()),
        user: None,
        users: None,
    });

    Ok(Json(message_with_user))
}

// ============================================================================
// User Handlers
// ============================================================================

async fn get_online_users(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<User>>, StatusCode> {
    let user_ids: Vec<Uuid> = state.online_users.read().await.keys().cloned().collect();

    if user_ids.is_empty() {
        return Ok(Json(vec![]));
    }

    let users = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = ANY($1)"
    )
    .bind(&user_ids)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(users))
}

// ============================================================================
// WebSocket Handler
// ============================================================================

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let token = params.get("token").cloned();
    ws.on_upgrade(|socket| handle_socket(socket, state, token))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>, token: Option<String>) {
    let user_id = match token {
        Some(t) => match verify_jwt(&t, &state.jwt_secret) {
            Ok(id) => id,
            Err(_) => return,
        },
        None => return,
    };

    // Get user info
    let user = match sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&state.db)
        .await
    {
        Ok(u) => u,
        Err(_) => return,
    };

    // Add to online users
    state.online_users.write().await.insert(user_id, user.username.clone());

    // Update user status
    let _ = sqlx::query("UPDATE users SET status = 'online' WHERE id = $1")
        .bind(user_id)
        .execute(&state.db)
        .await;

    // Broadcast user joined
    let _ = state.tx.send(WsMessage {
        msg_type: "user_joined".to_string(),
        room_id: None,
        message: None,
        user: Some(user.clone()),
        users: None,
    });

    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.tx.subscribe();

    // Send task - forward broadcasts to this WebSocket
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&msg) {
                if sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });

    // Receive task - handle incoming messages
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Close(_) = msg {
                break;
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }

    // Clean up: remove from online users
    state.online_users.write().await.remove(&user_id);

    // Update user status
    let _ = sqlx::query("UPDATE users SET status = 'offline' WHERE id = $1")
        .bind(user_id)
        .execute(&state.db)
        .await;

    // Broadcast user left
    let _ = state.tx.send(WsMessage {
        msg_type: "user_left".to_string(),
        room_id: None,
        message: None,
        user: Some(user),
        users: None,
    });
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
    Router::new()
        // Auth routes
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
        // Room routes
        .route("/api/rooms", get(get_rooms))
        .route("/api/rooms", post(create_room))
        .route("/api/rooms/:id/join", post(join_room))
        .route("/api/rooms/:id/leave", delete(leave_room))
        // Message routes
        .route("/api/rooms/:id/messages", get(get_room_messages))
        .route("/api/rooms/:id/messages", post(send_message))
        // User routes
        .route("/api/users/online", get(get_online_users))
        // WebSocket
        .route("/ws", get(ws_handler))
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
                .unwrap_or_else(|_| "chatwave_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/chatwave".to_string());
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "super-secret-jwt-key-change-in-production-chatwave".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    tracing::info!("✅ Connected to database");

    // Run migrations
    sqlx::query(include_str!("../schema.sql"))
        .execute(&pool)
        .await
        .ok();

    // Create broadcast channel for WebSocket messages
    let (tx, _rx) = broadcast::channel(100);

    let state = Arc::new(AppState {
        db: pool,
        jwt_secret,
        tx,
        online_users: Arc::new(RwLock::new(HashMap::new())),
    });

    let app = create_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3002").await?;
    tracing::info!("🚀 ChatWave backend listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}
