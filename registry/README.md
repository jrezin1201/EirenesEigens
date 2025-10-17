# RavensOne Package Registry

Official package registry server for RavensOne packages.

## 🚀 Quick Start

### Prerequisites

- Rust 1.75+ (`cargo --version`)
- PostgreSQL 14+ (`psql --version`)

### 1. Setup Database

```bash
# Create database
createdb ravensone_registry

# Or using psql
psql -c "CREATE DATABASE ravensone_registry;"
```

### 2. Configure Environment

```bash
cp .env.example .env
# Edit .env with your database credentials
```

### 3. Run Migrations

```bash
sqlx database setup
```

### 4. Start Server

```bash
# Development mode
cargo run

# Production mode
cargo build --release
./target/release/registry-server
```

Server will start on `http://localhost:8080`

## 📚 API Documentation

See [REGISTRY_API_SPEC.md](../REGISTRY_API_SPEC.md) for complete API documentation.

### Quick API Overview

```bash
# Health check
curl http://localhost:8080/health

# Register user
curl -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","email":"alice@example.com","password":"securepass123"}'

# Login
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"alice@example.com","password":"securepass123"}'

# Global statistics
curl http://localhost:8080/api/v1/stats
```

## 🏗️ Architecture

```
registry/
├── src/
│   ├── main.rs           # Server entry point
│   ├── models.rs         # Data models
│   ├── auth.rs           # Authentication (JWT, Argon2)
│   ├── db.rs             # Database layer
│   ├── error.rs          # Error handling
│   ├── validation.rs     # Input validation
│   ├── rate_limit.rs     # Rate limiting
│   └── handlers/         # API handlers
│       ├── auth.rs       # /auth/* endpoints
│       ├── packages.rs   # /packages/* endpoints
│       ├── users.rs      # /users/* endpoints
│       ├── search.rs     # /search endpoint
│       └── stats.rs      # /stats endpoint
├── migrations/           # Database migrations
└── Cargo.toml            # Dependencies
```

## 🔧 Technology Stack

- **Web Framework**: Axum (async, fast, type-safe)
- **Database**: PostgreSQL with SQLx (compile-time SQL verification)
- **Authentication**: JWT tokens + Argon2id password hashing
- **Validation**: Validator crate with custom rules
- **Logging**: Tracing with structured logging

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_auth_handlers
```

## 📊 Database Schema

### Core Tables

- **users** - User accounts
- **api_tokens** - API authentication tokens
- **packages** - Package metadata
- **versions** - Package versions with dependencies
- **downloads** - Download analytics
- **package_owners** - Package ownership mapping

See `migrations/20251017_init.sql` for complete schema.

## 🔐 Security Features

1. **Password Security**
   - Argon2id hashing (memory-hard, GPU-resistant)
   - Salted hashes stored in database

2. **Token Security**
   - JWT tokens with 30-day expiry
   - Tokens can be refreshed and revoked

3. **Input Validation**
   - Package names: lowercase, alphanumeric, hyphens only
   - Semver version validation
   - Email validation

4. **Rate Limiting**
   - 60 req/hour for unauthenticated users
   - 5000 req/hour for authenticated users

## 🌐 Deployment

### Production Checklist

- [ ] Update `JWT_SECRET` in src/auth.rs (use env variable)
- [ ] Configure PostgreSQL with proper credentials
- [ ] Set up HTTPS/TLS (use Nginx or Caddy as reverse proxy)
- [ ] Enable rate limiting with Redis backend
- [ ] Set up S3 or object storage for package tarballs
- [ ] Configure CDN for package downloads
- [ ] Set up monitoring and logging

### Docker Deployment

```bash
# Build Docker image (TODO: Create Dockerfile)
docker build -t ravensone-registry .

# Run with Docker Compose (TODO: Create docker-compose.yml)
docker-compose up -d
```

## 🔄 CLI Integration

The `raven pkg` CLI integrates with this registry:

```bash
# Login (stores token in ~/.raven/credentials)
raven pkg login

# Publish package
raven pkg publish

# Install package
raven pkg install raven-ui@1.2.3
```

## 📈 Status

**Current Implementation**: 70% complete

✅ **Completed**:
- API specification
- Database schema
- Authentication system (register, login)
- User management database layer
- Package database layer
- Statistics endpoints
- Error handling
- Input validation

⏳ **TODO**:
- Package publishing logic
- Package download handler
- Search implementation
- Owner management
- Token refresh endpoint
- Rate limiting (Redis integration)
- File storage (S3 integration)
- Webhook events

## 🤝 Contributing

This is part of the RavensOne project. See main repository README for contribution guidelines.

## 📄 License

MIT License - see LICENSE file in main repository

---

*Part of RavensOne v2.0 - The AI-first full-stack web framework*
