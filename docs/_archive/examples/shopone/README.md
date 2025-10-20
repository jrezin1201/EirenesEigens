# ShopOne - E-Commerce Platform

A full-featured e-commerce platform built with RavensOne.

![ShopOne](https://img.shields.io/badge/RavensOne-Example-6366f1)
![License](https://img.shields.io/badge/license-MIT-blue)

## 🎯 Overview

ShopOne demonstrates a production-ready e-commerce application built with RavensOne, showcasing:

- **Product Catalog** - Browse products with filtering, search, and categories
- **Shopping Cart** - Add, update, and remove items with persistent state
- **Checkout Flow** - Multi-step checkout with address and payment
- **Order Management** - Order history and tracking
- **User Accounts** - Registration, login, and profile management
- **Reviews & Ratings** - Product reviews with star ratings
- **Responsive Design** - Optimized for desktop, tablet, and mobile
- **Professional UI** - Modern design with raven-ui components

## 🏗️ Architecture

### Backend (Rust + Axum)
- **Products API** - CRUD, filtering, search by name/description
- **Cart Management** - Add/update/remove with user sessions
- **Order Processing** - Create orders from cart items
- **User Management** - Auth with JWT tokens
- **Review System** - Submit and retrieve product reviews
- **PostgreSQL** - Relational database with proper indexes

### Frontend (RavensOne)
- **Product Catalog** - Grid layout with lazy loading
- **Cart State** - Global cart with raven-store
- **Checkout Wizard** - Multi-step form with validation
- **User Dashboard** - Profile, orders, and saved items
- **Product Details** - Images, descriptions, reviews
- **Search & Filter** - Category, price range, featured items

## 📦 Features

### Product Catalog
- 12 sample products across multiple categories
- Product images from Unsplash
- Category filtering (Electronics, Apparel, Accessories, Fitness, Home)
- Price range filtering
- Text search across name and description
- Featured products section
- Stock tracking
- Average ratings and review counts

### Shopping Cart
- Add products with quantity selection
- Update quantities in cart
- Remove items from cart
- Persistent cart per user
- Real-time total calculation
- Stock validation

### Checkout & Orders
- Shipping address entry
- Payment method selection (demo only)
- Order creation from cart
- Order history view
- Order status tracking
- Items with prices locked at purchase time

### Reviews & Ratings
- 1-5 star ratings
- Text comments
- User attribution
- Average rating calculation
- Review count per product
- One review per user per product

## 🚀 Getting Started

### Prerequisites

- Rust 1.70+
- PostgreSQL 14+
- RavensOne CLI

### Backend Setup

1. **Create Database**:
```bash
createdb shopone
```

2. **Configure Environment**:
```bash
cd backend
cp .env.example .env
# Edit .env with your DATABASE_URL
```

3. **Run Backend**:
```bash
cargo run
```

Backend will start on `http://localhost:3001`

### Frontend Setup

1. **Install Dependencies**:
```bash
cd frontend
raven pkg install
```

2. **Run Development Server**:
```bash
raven dev
```

Frontend will start on `http://localhost:8000`

## 📚 API Endpoints

### Authentication
- `POST /api/auth/register` - Register new user
- `POST /api/auth/login` - Login user

### Products
- `GET /api/products` - Get all products (with filters)
- `GET /api/products/:id` - Get product details
- `GET /api/products/:id/reviews` - Get product reviews

Query parameters for `/api/products`:
- `category` - Filter by category
- `min_price` - Minimum price
- `max_price` - Maximum price
- `search` - Search in name/description
- `featured` - Show only featured products

### Cart (Protected)
- `GET /api/cart` - Get user's cart
- `POST /api/cart` - Add item to cart
- `PUT /api/cart/:id` - Update cart item quantity
- `DELETE /api/cart/:id` - Remove item from cart

### Orders (Protected)
- `GET /api/orders` - Get user's order history
- `POST /api/orders` - Create order from cart

### Reviews (Protected)
- `POST /api/reviews` - Submit product review

All protected endpoints require `Authorization: Bearer <token>` header.

## 🎨 Code Examples

### Adding to Cart

```rust
let response = add_to_cart(
    token,
    AddToCartRequest {
        product_id: product.id,
        quantity: 2,
    }
).await?;
```

### Creating an Order

```rust
let order = create_order(
    token,
    CreateOrderRequest {
        shipping_address: "123 Main St, City, State 12345".to_string(),
        payment_method: "credit_card".to_string(),
    }
).await?;
```

### Filtering Products

```rust
// Get Electronics under $200
let products = get_products(ProductFilters {
    category: Some("Electronics".to_string()),
    max_price: Some(200.0),
    ..Default::default()
}).await?;
```

## 🔒 Security

- Passwords hashed with Argon2id
- JWT tokens with 30-day expiry
- SQL injection protection with SQLx
- CORS configuration
- Input validation on client and server
- Cart ownership verification
- Order authorization checks

## 📊 Database Schema

See `schema.sql` for complete schema. Key tables:

- **users** - User accounts with address
- **products** - Product catalog with stock
- **carts** - User shopping carts
- **cart_items** - Items in carts
- **orders** - Placed orders with status
- **order_items** - Order contents with locked prices
- **reviews** - Product reviews and ratings

## 🎓 Learning Resources

This example demonstrates:

1. **Complex State Management** - Cart with multiple items, global state
2. **Multi-Step Forms** - Checkout flow with validation
3. **Data Relationships** - Products, carts, orders, reviews
4. **Filtering & Search** - Dynamic queries with multiple criteria
5. **Computed Values** - Totals, averages, counts
6. **Authorization** - User-specific cart and orders
7. **Decimal Handling** - Precise money calculations

## 🚢 Deployment

### Backend (Fly.io)

```bash
cd backend
flyctl launch
flyctl postgres create
flyctl postgres attach
flyctl deploy
```

### Frontend (Vercel/Netlify)

```bash
cd frontend
raven build --release
# Deploy dist/ folder
```

## 📝 License

MIT License - see LICENSE file for details

## 🔗 Links

- [RavensOne Documentation](https://ravensone-docs.fly.dev)
- [Package Registry](https://ravensone-registry.fly.dev)
- [GitHub Repository](https://github.com/ravensone/ravensone)

---

Built with ❤️ using [RavensOne](https://ravensone.dev)
