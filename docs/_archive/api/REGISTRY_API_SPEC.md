# RavensOne Package Registry API Specification

**Version**: 1.0.0
**Date**: October 17, 2025
**Status**: Draft

---

## Overview

The RavensOne Package Registry is a centralized service for publishing, discovering, and downloading RavensOne packages. It provides a REST API for the `raven pkg` CLI to interact with.

### Base URL

```
Production: https://registry.ravensone.dev/api/v1
Development: http://localhost:8080/api/v1
```

### Authentication

All write operations require token-based authentication via HTTP headers:

```
Authorization: Bearer <token>
```

Tokens are obtained via the `/auth/login` endpoint and expire after 30 days.

---

## API Endpoints

### 1. Authentication

#### `POST /auth/register`

Register a new user account.

**Request Body**:
```json
{
  "username": "alice",
  "email": "alice@example.com",
  "password": "secure_password_123"
}
```

**Response** (201 Created):
```json
{
  "user_id": "usr_abc123",
  "username": "alice",
  "email": "alice@example.com",
  "created_at": "2025-10-17T12:00:00Z",
  "token": "tok_xyz789..."
}
```

**Errors**:
- `400` - Invalid request (missing fields, weak password)
- `409` - Username or email already exists

---

#### `POST /auth/login`

Authenticate and receive an API token.

**Request Body**:
```json
{
  "email": "alice@example.com",
  "password": "secure_password_123"
}
```

**Response** (200 OK):
```json
{
  "token": "tok_xyz789...",
  "expires_at": "2025-11-16T12:00:00Z",
  "user": {
    "user_id": "usr_abc123",
    "username": "alice",
    "email": "alice@example.com"
  }
}
```

**Errors**:
- `401` - Invalid credentials
- `429` - Rate limit exceeded (max 10 attempts per hour)

---

#### `POST /auth/refresh`

Refresh an expiring token.

**Headers**: `Authorization: Bearer <token>`

**Response** (200 OK):
```json
{
  "token": "tok_new123...",
  "expires_at": "2025-12-16T12:00:00Z"
}
```

**Errors**:
- `401` - Invalid or expired token

---

### 2. Package Publishing

#### `POST /packages/publish`

Publish a new package version.

**Headers**:
- `Authorization: Bearer <token>`
- `Content-Type: multipart/form-data`

**Request Body** (multipart):
- `manifest` (JSON): Package manifest (raven.toml as JSON)
- `tarball` (file): Gzipped tarball of package source

**Manifest Schema**:
```json
{
  "name": "raven-ui",
  "version": "1.2.3",
  "authors": ["Alice <alice@example.com>"],
  "description": "UI component library for RavensOne",
  "license": "MIT",
  "repository": "https://github.com/alice/raven-ui",
  "homepage": "https://raven-ui.dev",
  "keywords": ["ui", "components", "reactive"],
  "dependencies": {
    "raven-signals": "^0.1.0"
  },
  "dev_dependencies": {
    "raven-test": "^0.2.0"
  }
}
```

**Response** (201 Created):
```json
{
  "package_id": "pkg_ui123",
  "name": "raven-ui",
  "version": "1.2.3",
  "published_at": "2025-10-17T12:00:00Z",
  "download_url": "https://registry.ravensone.dev/packages/raven-ui/1.2.3/download",
  "checksum": "sha256:abc123..."
}
```

**Errors**:
- `400` - Invalid manifest or tarball
- `401` - Authentication required
- `403` - Not authorized to publish (not package owner)
- `409` - Version already exists
- `413` - Package too large (max 50MB)

---

#### `GET /packages/:name`

Get package metadata.

**Response** (200 OK):
```json
{
  "name": "raven-ui",
  "description": "UI component library for RavensOne",
  "latest_version": "1.2.3",
  "versions": ["1.0.0", "1.1.0", "1.2.0", "1.2.3"],
  "owner": {
    "username": "alice",
    "user_id": "usr_abc123"
  },
  "license": "MIT",
  "repository": "https://github.com/alice/raven-ui",
  "homepage": "https://raven-ui.dev",
  "keywords": ["ui", "components", "reactive"],
  "downloads_total": 15234,
  "downloads_last_month": 842,
  "created_at": "2025-01-15T10:00:00Z",
  "updated_at": "2025-10-17T12:00:00Z"
}
```

**Errors**:
- `404` - Package not found

---

#### `GET /packages/:name/:version`

Get specific version metadata.

**Response** (200 OK):
```json
{
  "name": "raven-ui",
  "version": "1.2.3",
  "description": "UI component library for RavensOne",
  "authors": ["Alice <alice@example.com>"],
  "license": "MIT",
  "repository": "https://github.com/alice/raven-ui",
  "dependencies": {
    "raven-signals": "^0.1.0"
  },
  "dev_dependencies": {
    "raven-test": "^0.2.0"
  },
  "published_at": "2025-10-17T12:00:00Z",
  "download_url": "https://registry.ravensone.dev/packages/raven-ui/1.2.3/download",
  "checksum": "sha256:abc123...",
  "size_bytes": 125440
}
```

**Errors**:
- `404` - Package or version not found

---

#### `GET /packages/:name/:version/download`

Download package tarball.

**Response** (200 OK):
- Content-Type: `application/gzip`
- Content-Disposition: `attachment; filename="raven-ui-1.2.3.tar.gz"`
- Body: Gzipped tarball

**Errors**:
- `404` - Package or version not found

---

#### `DELETE /packages/:name/:version`

Yank (unpublish) a package version.

**Headers**: `Authorization: Bearer <token>`

**Response** (200 OK):
```json
{
  "name": "raven-ui",
  "version": "1.2.3",
  "yanked": true,
  "yanked_at": "2025-10-17T12:30:00Z"
}
```

**Note**: Yanked versions are still downloadable but won't appear in searches or dependency resolution.

**Errors**:
- `401` - Authentication required
- `403` - Not authorized (not package owner)
- `404` - Package or version not found

---

### 3. Package Search & Discovery

#### `GET /search?q=<query>&limit=<n>&offset=<n>`

Search packages by name, keywords, or description.

**Query Parameters**:
- `q` (required): Search query
- `limit` (optional): Results per page (default: 20, max: 100)
- `offset` (optional): Pagination offset (default: 0)

**Example**: `/search?q=ui+components&limit=10`

**Response** (200 OK):
```json
{
  "results": [
    {
      "name": "raven-ui",
      "version": "1.2.3",
      "description": "UI component library for RavensOne",
      "keywords": ["ui", "components", "reactive"],
      "downloads": 15234,
      "score": 0.95
    },
    {
      "name": "raven-forms",
      "version": "0.3.1",
      "description": "Form handling for RavensOne",
      "keywords": ["forms", "validation", "ui"],
      "downloads": 8421,
      "score": 0.78
    }
  ],
  "total": 42,
  "limit": 10,
  "offset": 0
}
```

**Errors**:
- `400` - Invalid query parameters

---

#### `GET /packages/trending?period=<week|month|all>&limit=<n>`

Get trending packages.

**Query Parameters**:
- `period` (optional): Time period (default: week)
- `limit` (optional): Number of results (default: 20, max: 100)

**Response** (200 OK):
```json
{
  "packages": [
    {
      "name": "raven-ui",
      "version": "1.2.3",
      "description": "UI component library",
      "downloads_period": 842,
      "trend_score": 12.4
    }
  ]
}
```

---

#### `GET /packages/categories/:category`

Get packages in a category.

**Categories**: `ui`, `http`, `database`, `testing`, `utilities`, `frameworks`, `tools`

**Response** (200 OK):
```json
{
  "category": "ui",
  "packages": [
    {
      "name": "raven-ui",
      "version": "1.2.3",
      "description": "UI component library",
      "downloads": 15234
    }
  ],
  "total": 12
}
```

---

### 4. User Management

#### `GET /users/:username`

Get user profile.

**Response** (200 OK):
```json
{
  "username": "alice",
  "user_id": "usr_abc123",
  "joined_at": "2025-01-10T08:00:00Z",
  "packages": [
    {
      "name": "raven-ui",
      "version": "1.2.3",
      "downloads": 15234
    },
    {
      "name": "raven-forms",
      "version": "0.3.1",
      "downloads": 8421
    }
  ],
  "total_downloads": 23655
}
```

**Errors**:
- `404` - User not found

---

#### `GET /users/me`

Get authenticated user's profile (includes email).

**Headers**: `Authorization: Bearer <token>`

**Response** (200 OK):
```json
{
  "username": "alice",
  "user_id": "usr_abc123",
  "email": "alice@example.com",
  "joined_at": "2025-01-10T08:00:00Z",
  "packages": [...],
  "tokens": [
    {
      "token_id": "tok_123",
      "name": "My Laptop",
      "created_at": "2025-10-01T10:00:00Z",
      "last_used": "2025-10-17T11:00:00Z"
    }
  ]
}
```

**Errors**:
- `401` - Authentication required

---

#### `POST /users/me/tokens`

Create a new API token.

**Headers**: `Authorization: Bearer <token>`

**Request Body**:
```json
{
  "name": "CI/CD Pipeline",
  "expires_in_days": 90
}
```

**Response** (201 Created):
```json
{
  "token": "tok_new456...",
  "token_id": "tok_456",
  "name": "CI/CD Pipeline",
  "created_at": "2025-10-17T12:00:00Z",
  "expires_at": "2026-01-15T12:00:00Z"
}
```

**Errors**:
- `401` - Authentication required

---

#### `DELETE /users/me/tokens/:token_id`

Revoke an API token.

**Headers**: `Authorization: Bearer <token>`

**Response** (200 OK):
```json
{
  "token_id": "tok_456",
  "revoked": true
}
```

**Errors**:
- `401` - Authentication required
- `404` - Token not found

---

### 5. Package Owners

#### `GET /packages/:name/owners`

List package owners.

**Response** (200 OK):
```json
{
  "owners": [
    {
      "username": "alice",
      "user_id": "usr_abc123",
      "role": "owner"
    },
    {
      "username": "bob",
      "user_id": "usr_def456",
      "role": "maintainer"
    }
  ]
}
```

---

#### `PUT /packages/:name/owners`

Add a package owner.

**Headers**: `Authorization: Bearer <token>`

**Request Body**:
```json
{
  "username": "bob",
  "role": "maintainer"
}
```

**Roles**: `owner` (full control), `maintainer` (can publish)

**Response** (200 OK):
```json
{
  "username": "bob",
  "role": "maintainer",
  "added_at": "2025-10-17T12:00:00Z"
}
```

**Errors**:
- `401` - Authentication required
- `403` - Not authorized (only owners can add owners)
- `404` - Package or user not found

---

#### `DELETE /packages/:name/owners/:username`

Remove a package owner.

**Headers**: `Authorization: Bearer <token>`

**Response** (200 OK):
```json
{
  "username": "bob",
  "removed": true
}
```

**Errors**:
- `401` - Authentication required
- `403` - Not authorized or cannot remove last owner

---

### 6. Statistics & Analytics

#### `GET /stats`

Get global registry statistics.

**Response** (200 OK):
```json
{
  "total_packages": 1247,
  "total_versions": 8432,
  "total_downloads": 2841923,
  "total_users": 542,
  "updated_at": "2025-10-17T12:00:00Z"
}
```

---

#### `GET /packages/:name/stats`

Get package-specific statistics.

**Response** (200 OK):
```json
{
  "name": "raven-ui",
  "downloads": {
    "total": 15234,
    "last_week": 234,
    "last_month": 842,
    "last_year": 12450
  },
  "versions_count": 12,
  "dependents_count": 48,
  "updated_at": "2025-10-17T12:00:00Z"
}
```

---

## Rate Limiting

All endpoints are rate-limited to prevent abuse:

- **Unauthenticated**: 60 requests per hour per IP
- **Authenticated**: 5000 requests per hour per user
- **Search**: 100 requests per hour per IP
- **Download**: Unlimited (CDN cached)

**Rate Limit Headers**:
```
X-RateLimit-Limit: 5000
X-RateLimit-Remaining: 4823
X-RateLimit-Reset: 1697554800
```

**Error Response** (429 Too Many Requests):
```json
{
  "error": "rate_limit_exceeded",
  "message": "Rate limit exceeded. Try again in 42 minutes.",
  "retry_after": 2520
}
```

---

## Error Response Format

All errors follow a consistent format:

```json
{
  "error": "error_code",
  "message": "Human-readable error message",
  "details": {
    "field": "Additional context"
  }
}
```

**Common Error Codes**:
- `invalid_request` - Malformed request
- `authentication_required` - Missing or invalid token
- `permission_denied` - User lacks required permissions
- `not_found` - Resource not found
- `conflict` - Resource already exists
- `rate_limit_exceeded` - Too many requests
- `internal_error` - Server error

---

## Security Considerations

### 1. Authentication
- Tokens use cryptographically secure random generation
- Passwords hashed with Argon2id
- Token rotation encouraged (30-day expiry)

### 2. Authorization
- Package owners verified before write operations
- Token scopes limit permissions

### 3. Package Integrity
- SHA256 checksums for all packages
- Tarball size limits (50MB)
- Malware scanning (planned)

### 4. Input Validation
- Package names: lowercase, alphanumeric, hyphens (max 64 chars)
- Versions: strict semver (e.g., 1.2.3)
- Manifest validation against JSON schema

### 5. HTTPS Only
- All production traffic over TLS 1.3
- HSTS headers enforced

---

## Webhook Events (Future)

Planned webhook support for:
- `package.published` - New version published
- `package.yanked` - Version yanked
- `package.deprecated` - Package deprecated

---

## CLI Integration

The `raven pkg` CLI integrates with this API:

```bash
# Login (stores token in ~/.raven/credentials)
raven pkg login

# Publish package (POST /packages/publish)
raven pkg publish

# Search (GET /search)
raven pkg search ui components

# Install (GET /packages/:name/:version/download)
raven pkg install raven-ui@1.2.3

# User info (GET /users/me)
raven pkg whoami
```

---

## Implementation Notes

### Technology Stack
- **Server**: Rust with Axum web framework
- **Database**: PostgreSQL for metadata, S3 for tarballs
- **Cache**: Redis for rate limiting and popular packages
- **CDN**: CloudFront for package downloads

### Database Schema (High-Level)

**users**:
- user_id (PK)
- username (unique)
- email (unique)
- password_hash
- created_at

**packages**:
- package_id (PK)
- name (unique)
- owner_id (FK → users)
- description
- license
- repository
- created_at

**versions**:
- version_id (PK)
- package_id (FK → packages)
- version (semver)
- tarball_url
- checksum
- published_at
- yanked (boolean)

**downloads**:
- download_id (PK)
- package_id (FK → packages)
- version_id (FK → versions)
- downloaded_at
- ip_hash

**tokens**:
- token_id (PK)
- user_id (FK → users)
- token_hash
- name
- created_at
- expires_at

---

## Versioning

This API follows semantic versioning. Breaking changes will increment the major version in the URL path (e.g., `/api/v2`).

---

**Status**: Ready for implementation
**Next Steps**:
1. Implement registry server (Rust/Axum)
2. Set up PostgreSQL schema
3. Integrate with `raven pkg` CLI
4. Deploy to production

---

*Last Updated: October 17, 2025*
*Maintainer: RavensOne Core Team*
