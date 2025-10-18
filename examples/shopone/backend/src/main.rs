use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
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
    address: Option<String>,
    phone: Option<String>,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct Product {
    id: Uuid,
    name: String,
    description: Option<String>,
    price: f64,
    category: String,
    image_url: Option<String>,
    stock: i32,
    featured: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProductWithRating {
    #[serde(flatten)]
    product: Product,
    average_rating: Option<f64>,
    review_count: i64,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct Cart {
    id: Uuid,
    user_id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct CartItem {
    id: Uuid,
    cart_id: Uuid,
    product_id: Uuid,
    quantity: i32,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CartItemWithProduct {
    #[serde(flatten)]
    cart_item: CartItem,
    product: Product,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct Order {
    id: Uuid,
    user_id: Uuid,
    total: f64,
    status: String,
    shipping_address: String,
    payment_method: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct OrderItem {
    id: Uuid,
    order_id: Uuid,
    product_id: Uuid,
    quantity: i32,
    price_at_purchase: f64,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct Review {
    id: Uuid,
    product_id: Uuid,
    user_id: Uuid,
    rating: i32,
    comment: Option<String>,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ReviewWithUser {
    #[serde(flatten)]
    review: Review,
    user_name: String,
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
struct ProductFilters {
    category: Option<String>,
    min_price: Option<f64>,
    max_price: Option<f64>,
    search: Option<String>,
    featured: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct AddToCartRequest {
    product_id: Uuid,
    quantity: i32,
}

#[derive(Debug, Deserialize)]
struct UpdateCartItemRequest {
    quantity: i32,
}

#[derive(Debug, Deserialize)]
struct CreateOrderRequest {
    shipping_address: String,
    payment_method: String,
}

#[derive(Debug, Deserialize)]
struct CreateReviewRequest {
    product_id: Uuid,
    rating: i32,
    comment: Option<String>,
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
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash password".to_string()))?
        .to_string();

    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (email, password_hash, name) VALUES ($1, $2, $3) RETURNING *",
    )
    .bind(&payload.email)
    .bind(&password_hash)
    .bind(&payload.name)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::BAD_REQUEST, format!("User already exists: {}", e)))?;

    let token = generate_jwt(&user.id, &state.jwt_secret)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate token".to_string()))?;

    Ok(Json(AuthResponse { token, user }))
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(&payload.email)
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
// Product Handlers
// ============================================================================

async fn get_products(
    State(state): State<Arc<AppState>>,
    Query(filters): Query<ProductFilters>,
) -> Result<Json<Vec<ProductWithRating>>, StatusCode> {
    let mut query = String::from(
        "SELECT p.*,
         COALESCE(AVG(r.rating), 0) as average_rating,
         COUNT(r.id) as review_count
         FROM products p
         LEFT JOIN reviews r ON p.id = r.product_id
         WHERE 1=1"
    );

    if let Some(category) = &filters.category {
        query.push_str(&format!(" AND p.category = '{}'", category));
    }

    if let Some(min_price) = filters.min_price {
        query.push_str(&format!(" AND p.price >= {}", min_price));
    }

    if let Some(max_price) = filters.max_price {
        query.push_str(&format!(" AND p.price <= {}", max_price));
    }

    if let Some(search) = &filters.search {
        query.push_str(&format!(" AND (p.name ILIKE '%{}%' OR p.description ILIKE '%{}%')", search, search));
    }

    if let Some(featured) = filters.featured {
        query.push_str(&format!(" AND p.featured = {}", featured));
    }

    query.push_str(" GROUP BY p.id ORDER BY p.created_at DESC");

    let rows = sqlx::query(&query)
        .fetch_all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let products: Vec<ProductWithRating> = rows
        .into_iter()
        .map(|row| ProductWithRating {
            product: Product {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
                price: row.get("price"),
                category: row.get("category"),
                image_url: row.get("image_url"),
                stock: row.get("stock"),
                featured: row.get("featured"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            },
            average_rating: row.try_get("average_rating").ok(),
            review_count: row.get("review_count"),
        })
        .collect();

    Ok(Json(products))
}

async fn get_product(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ProductWithRating>, StatusCode> {
    let row = sqlx::query(
        "SELECT p.*,
         COALESCE(AVG(r.rating), 0) as average_rating,
         COUNT(r.id) as review_count
         FROM products p
         LEFT JOIN reviews r ON p.id = r.product_id
         WHERE p.id = $1
         GROUP BY p.id"
    )
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(ProductWithRating {
        product: Product {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            price: row.get("price"),
            category: row.get("category"),
            image_url: row.get("image_url"),
            stock: row.get("stock"),
            featured: row.get("featured"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        },
        average_rating: row.try_get("average_rating").ok(),
        review_count: row.get("review_count"),
    }))
}

// ============================================================================
// Cart Handlers
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

async fn get_cart(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Vec<CartItemWithProduct>>, StatusCode> {
    let user_id = extract_user_from_header(&headers, &state.jwt_secret)?;

    // Get or create cart
    let cart = sqlx::query_as::<_, Cart>(
        "INSERT INTO carts (user_id) VALUES ($1) ON CONFLICT (user_id) DO UPDATE SET updated_at = NOW() RETURNING *"
    )
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get cart items with products
    let rows = sqlx::query(
        "SELECT ci.*, p.* FROM cart_items ci
         JOIN products p ON ci.product_id = p.id
         WHERE ci.cart_id = $1"
    )
    .bind(cart.id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let items: Vec<CartItemWithProduct> = rows
        .into_iter()
        .map(|row| CartItemWithProduct {
            cart_item: CartItem {
                id: row.get("id"),
                cart_id: row.get("cart_id"),
                product_id: row.get("product_id"),
                quantity: row.get("quantity"),
                created_at: row.get("created_at"),
            },
            product: Product {
                id: row.get("product_id"),
                name: row.get("name"),
                description: row.get("description"),
                price: row.get("price"),
                category: row.get("category"),
                image_url: row.get("image_url"),
                stock: row.get("stock"),
                featured: row.get("featured"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            },
        })
        .collect();

    Ok(Json(items))
}

async fn add_to_cart(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<AddToCartRequest>,
) -> Result<Json<CartItem>, StatusCode> {
    let user_id = extract_user_from_header(&headers, &state.jwt_secret)?;

    // Get or create cart
    let cart = sqlx::query_as::<_, Cart>(
        "INSERT INTO carts (user_id) VALUES ($1) ON CONFLICT (user_id) DO UPDATE SET updated_at = NOW() RETURNING *"
    )
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Add or update cart item
    let cart_item = sqlx::query_as::<_, CartItem>(
        "INSERT INTO cart_items (cart_id, product_id, quantity)
         VALUES ($1, $2, $3)
         ON CONFLICT (cart_id, product_id) DO UPDATE SET quantity = cart_items.quantity + $3
         RETURNING *"
    )
    .bind(cart.id)
    .bind(payload.product_id)
    .bind(payload.quantity)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(cart_item))
}

async fn update_cart_item(
    State(state): State<Arc<AppState>>,
    Path(item_id): Path<Uuid>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<UpdateCartItemRequest>,
) -> Result<Json<CartItem>, StatusCode> {
    let user_id = extract_user_from_header(&headers, &state.jwt_secret)?;

    // Verify ownership
    let _ = sqlx::query(
        "SELECT ci.id FROM cart_items ci
         JOIN carts c ON ci.cart_id = c.id
         WHERE ci.id = $1 AND c.user_id = $2"
    )
    .bind(item_id)
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    // Update quantity
    let cart_item = sqlx::query_as::<_, CartItem>(
        "UPDATE cart_items SET quantity = $1 WHERE id = $2 RETURNING *"
    )
    .bind(payload.quantity)
    .bind(item_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(cart_item))
}

async fn remove_from_cart(
    State(state): State<Arc<AppState>>,
    Path(item_id): Path<Uuid>,
    headers: axum::http::HeaderMap,
) -> Result<StatusCode, StatusCode> {
    let user_id = extract_user_from_header(&headers, &state.jwt_secret)?;

    // Verify ownership and delete
    sqlx::query(
        "DELETE FROM cart_items WHERE id = $1 AND cart_id IN (SELECT id FROM carts WHERE user_id = $2)"
    )
    .bind(item_id)
    .bind(user_id)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Order Handlers
// ============================================================================

async fn create_order(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<CreateOrderRequest>,
) -> Result<Json<Order>, StatusCode> {
    let user_id = extract_user_from_header(&headers, &state.jwt_secret)?;

    // Get cart items
    let cart = sqlx::query_as::<_, Cart>("SELECT * FROM carts WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let cart_items = sqlx::query(
        "SELECT ci.product_id, ci.quantity, p.price
         FROM cart_items ci
         JOIN products p ON ci.product_id = p.id
         WHERE ci.cart_id = $1"
    )
    .bind(cart.id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if cart_items.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Calculate total
    let total: f64 = cart_items
        .iter()
        .map(|row| {
            let price: f64 = row.get("price");
            let quantity: i32 = row.get("quantity");
            price * quantity as f64
        })
        .sum();

    // Create order
    let order = sqlx::query_as::<_, Order>(
        "INSERT INTO orders (user_id, total, shipping_address, payment_method)
         VALUES ($1, $2, $3, $4) RETURNING *"
    )
    .bind(user_id)
    .bind(total)
    .bind(&payload.shipping_address)
    .bind(&payload.payment_method)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create order items
    for row in cart_items {
        let product_id: Uuid = row.get("product_id");
        let quantity: i32 = row.get("quantity");
        let price: f64 = row.get("price");

        sqlx::query(
            "INSERT INTO order_items (order_id, product_id, quantity, price_at_purchase)
             VALUES ($1, $2, $3, $4)"
        )
        .bind(order.id)
        .bind(product_id)
        .bind(quantity)
        .bind(price)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // Clear cart
    sqlx::query("DELETE FROM cart_items WHERE cart_id = $1")
        .bind(cart.id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(order))
}

async fn get_orders(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Vec<Order>>, StatusCode> {
    let user_id = extract_user_from_header(&headers, &state.jwt_secret)?;

    let orders = sqlx::query_as::<_, Order>(
        "SELECT * FROM orders WHERE user_id = $1 ORDER BY created_at DESC"
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(orders))
}

// ============================================================================
// Review Handlers
// ============================================================================

async fn create_review(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<CreateReviewRequest>,
) -> Result<Json<Review>, StatusCode> {
    let user_id = extract_user_from_header(&headers, &state.jwt_secret)?;

    let review = sqlx::query_as::<_, Review>(
        "INSERT INTO reviews (product_id, user_id, rating, comment)
         VALUES ($1, $2, $3, $4) RETURNING *"
    )
    .bind(payload.product_id)
    .bind(user_id)
    .bind(payload.rating)
    .bind(payload.comment)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok(Json(review))
}

async fn get_product_reviews(
    State(state): State<Arc<AppState>>,
    Path(product_id): Path<Uuid>,
) -> Result<Json<Vec<ReviewWithUser>>, StatusCode> {
    let rows = sqlx::query(
        "SELECT r.*, u.name as user_name FROM reviews r
         JOIN users u ON r.user_id = u.id
         WHERE r.product_id = $1
         ORDER BY r.created_at DESC"
    )
    .bind(product_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let reviews: Vec<ReviewWithUser> = rows
        .into_iter()
        .map(|row| ReviewWithUser {
            review: Review {
                id: row.get("id"),
                product_id: row.get("product_id"),
                user_id: row.get("user_id"),
                rating: row.get("rating"),
                comment: row.get("comment"),
                created_at: row.get("created_at"),
            },
            user_name: row.get("user_name"),
        })
        .collect();

    Ok(Json(reviews))
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
        // Product routes
        .route("/api/products", get(get_products))
        .route("/api/products/:id", get(get_product))
        .route("/api/products/:id/reviews", get(get_product_reviews))
        // Cart routes
        .route("/api/cart", get(get_cart))
        .route("/api/cart", post(add_to_cart))
        .route("/api/cart/:id", put(update_cart_item))
        .route("/api/cart/:id", delete(remove_from_cart))
        // Order routes
        .route("/api/orders", get(get_orders))
        .route("/api/orders", post(create_order))
        // Review routes
        .route("/api/reviews", post(create_review))
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
                .unwrap_or_else(|_| "shopone_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/shopone".to_string());
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "super-secret-jwt-key-change-in-production-shopone".to_string());

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

    let state = Arc::new(AppState {
        db: pool,
        jwt_secret,
    });

    let app = create_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    tracing::info!("🚀 ShopOne backend listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}
