use axum::{
    extract::{Path, State},
    http::{header, Method, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber;
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    db: PgPool,
}

// --- Models ---

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct User {
    id: Uuid,
    username: String,
    display_name: String,
    avatar_gradient: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Post {
    id: Uuid,
    user: User,
    caption: String,
    image_url: String,
    like_count: i64,
    comment_count: i64,
    liked_by_current_user: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct PostRow {
    id: Uuid,
    caption: String,
    image_url: String,
    like_count: i64,
    comment_count: i64,
    created_at: chrono::DateTime<chrono::Utc>,
    user_id: Uuid,
    username: String,
    display_name: String,
    avatar_gradient: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Comment {
    id: Uuid,
    user: User,
    content: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
struct CreateCommentRequest {
    content: String,
    user_id: Uuid, // In production, this would come from auth
}

#[derive(Debug, Deserialize)]
struct ToggleLikeRequest {
    user_id: Uuid, // In production, this would come from auth
}

// --- Handlers ---

async fn get_posts(State(state): State<AppState>) -> Result<Json<Vec<Post>>, StatusCode> {
    let rows = sqlx::query_as::<_, PostRow>(
        r#"
        SELECT
            p.id,
            p.caption,
            p.image_url,
            p.created_at,
            COUNT(DISTINCT l.id) as like_count,
            COUNT(DISTINCT c.id) as comment_count,
            u.id as user_id,
            u.username,
            u.display_name,
            u.avatar_gradient
        FROM posts p
        JOIN users u ON p.user_id = u.id
        LEFT JOIN likes l ON p.id = l.post_id
        LEFT JOIN comments c ON p.id = c.post_id
        GROUP BY p.id, u.id, u.username, u.display_name, u.avatar_gradient
        ORDER BY p.created_at DESC
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let posts: Vec<Post> = rows
        .into_iter()
        .map(|row| Post {
            id: row.id,
            user: User {
                id: row.user_id,
                username: row.username,
                display_name: row.display_name,
                avatar_gradient: row.avatar_gradient,
            },
            caption: row.caption,
            image_url: row.image_url,
            like_count: row.like_count,
            comment_count: row.comment_count,
            liked_by_current_user: false, // TODO: Check against current user
            created_at: row.created_at,
        })
        .collect();

    Ok(Json(posts))
}

async fn toggle_like(
    State(state): State<AppState>,
    Path(post_id): Path<Uuid>,
    Json(req): Json<ToggleLikeRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Check if like exists
    let existing_like = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM likes WHERE post_id = $1 AND user_id = $2"
    )
    .bind(post_id)
    .bind(req.user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(_) = existing_like {
        // Unlike
        sqlx::query("DELETE FROM likes WHERE post_id = $1 AND user_id = $2")
            .bind(post_id)
            .bind(req.user_id)
            .execute(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(Json(serde_json::json!({ "liked": false })))
    } else {
        // Like
        sqlx::query("INSERT INTO likes (post_id, user_id) VALUES ($1, $2)")
            .bind(post_id)
            .bind(req.user_id)
            .execute(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(Json(serde_json::json!({ "liked": true })))
    }
}

async fn get_comments(
    State(state): State<AppState>,
    Path(post_id): Path<Uuid>,
) -> Result<Json<Vec<Comment>>, StatusCode> {
    let comments = sqlx::query_as::<_, (Uuid, String, chrono::DateTime<chrono::Utc>, Uuid, String, String, String)>(
        r#"
        SELECT
            c.id,
            c.content,
            c.created_at,
            u.id,
            u.username,
            u.display_name,
            u.avatar_gradient
        FROM comments c
        JOIN users u ON c.user_id = u.id
        WHERE c.post_id = $1
        ORDER BY c.created_at DESC
        "#,
    )
    .bind(post_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .into_iter()
    .map(|(id, content, created_at, user_id, username, display_name, avatar_gradient)| {
        Comment {
            id,
            content,
            created_at,
            user: User {
                id: user_id,
                username,
                display_name,
                avatar_gradient,
            },
        }
    })
    .collect();

    Ok(Json(comments))
}

async fn create_comment(
    State(state): State<AppState>,
    Path(post_id): Path<Uuid>,
    Json(req): Json<CreateCommentRequest>,
) -> Result<Json<Comment>, StatusCode> {
    let comment_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO comments (id, post_id, user_id, content) VALUES ($1, $2, $3, $4)"
    )
    .bind(comment_id)
    .bind(post_id)
    .bind(req.user_id)
    .bind(&req.content)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Fetch the created comment with user info
    let (content, created_at, user_id, username, display_name, avatar_gradient) = sqlx::query_as::<_, (String, chrono::DateTime<chrono::Utc>, Uuid, String, String, String)>(
        r#"
        SELECT c.content, c.created_at, u.id, u.username, u.display_name, u.avatar_gradient
        FROM comments c
        JOIN users u ON c.user_id = u.id
        WHERE c.id = $1
        "#,
    )
    .bind(comment_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(Comment {
        id: comment_id,
        content,
        created_at,
        user: User {
            id: user_id,
            username,
            display_name,
            avatar_gradient,
        },
    }))
}

async fn health_check() -> &'static str {
    "OK"
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/bluebird".to_string());

    println!("🔗 Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    println!("✅ Database connected");

    let state = AppState { db: pool };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE]);

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/posts", get(get_posts))
        .route("/api/posts/:id/like", post(toggle_like))
        .route("/api/posts/:id/comments", get(get_comments))
        .route("/api/posts/:id/comments", post(create_comment))
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 9000));
    println!("🚀 Bluebird API server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
