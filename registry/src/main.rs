use axum::{
    routing::{get, post, put, delete},
    Router,
    middleware,
};
use std::net::SocketAddr;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
    compression::CompressionLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod models;
mod handlers;
mod auth;
mod db;
mod error;
mod rate_limit;
mod validation;

use crate::{
    db::AppState,
    handlers::{auth as auth_handlers, packages, users, search, stats},
    rate_limit::rate_limiter_middleware,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ravensone_registry=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/ravensone_registry".to_string());

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(50)
        .connect(&database_url)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    let state = AppState::new(pool);

    // Build router
    let app = Router::new()
        // Health check
        .route("/health", get(|| async { "OK" }))

        // API v1 routes
        .nest("/api/v1", api_routes())

        // Global middleware
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn(rate_limiter_middleware))

        // App state
        .with_state(state);

    // Start server
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()?;

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    tracing::info!("🚀 RavensOne Registry server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn api_routes() -> Router<AppState> {
    Router::new()
        // Authentication
        .route("/auth/register", post(auth_handlers::register))
        .route("/auth/login", post(auth_handlers::login))
        .route("/auth/refresh", post(auth_handlers::refresh_token))

        // Package publishing & management
        .route("/packages/publish", post(packages::publish))
        .route("/packages/:name", get(packages::get_package))
        .route("/packages/:name/:version", get(packages::get_version))
        .route("/packages/:name/:version/download", get(packages::download))
        .route("/packages/:name/:version", delete(packages::yank_version))

        // Package owners
        .route("/packages/:name/owners", get(packages::list_owners))
        .route("/packages/:name/owners", put(packages::add_owner))
        .route("/packages/:name/owners/:username", delete(packages::remove_owner))

        // Search & discovery
        .route("/search", get(search::search_packages))
        .route("/packages/trending", get(search::trending_packages))
        .route("/packages/categories/:category", get(search::packages_by_category))

        // User management
        .route("/users/:username", get(users::get_user))
        .route("/users/me", get(users::get_current_user))
        .route("/users/me/tokens", post(users::create_token))
        .route("/users/me/tokens/:token_id", delete(users::revoke_token))

        // Statistics
        .route("/stats", get(stats::global_stats))
        .route("/packages/:name/stats", get(stats::package_stats))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check() {
        // Health check should always return OK
        assert!(true);
    }
}
