-- Initial database schema for RavensOne Package Registry

-- Users table
CREATE TABLE IF NOT EXISTS users (
    user_id UUID PRIMARY KEY,
    username VARCHAR(32) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL
);

CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email);

-- API tokens table
CREATE TABLE IF NOT EXISTS api_tokens (
    token_id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    name VARCHAR(100) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    last_used TIMESTAMP WITH TIME ZONE
);

CREATE INDEX idx_tokens_user ON api_tokens(user_id);
CREATE INDEX idx_tokens_hash ON api_tokens(token_hash);

-- Packages table
CREATE TABLE IF NOT EXISTS packages (
    package_id UUID PRIMARY KEY,
    name VARCHAR(64) UNIQUE NOT NULL,
    owner_id UUID NOT NULL REFERENCES users(user_id),
    description TEXT,
    license VARCHAR(100) NOT NULL,
    repository VARCHAR(255),
    homepage VARCHAR(255),
    keywords TEXT[] DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL
);

CREATE INDEX idx_packages_name ON packages(name);
CREATE INDEX idx_packages_owner ON packages(owner_id);
CREATE INDEX idx_packages_keywords ON packages USING GIN(keywords);

-- Package versions table
CREATE TABLE IF NOT EXISTS versions (
    version_id UUID PRIMARY KEY,
    package_id UUID NOT NULL REFERENCES packages(package_id) ON DELETE CASCADE,
    version VARCHAR(50) NOT NULL,
    authors TEXT[] NOT NULL,
    description TEXT,
    license VARCHAR(100) NOT NULL,
    repository VARCHAR(255),
    dependencies JSONB DEFAULT '{}',
    dev_dependencies JSONB DEFAULT '{}',
    tarball_url VARCHAR(500) NOT NULL,
    checksum VARCHAR(64) NOT NULL,
    size_bytes BIGINT NOT NULL,
    published_at TIMESTAMP WITH TIME ZONE NOT NULL,
    yanked BOOLEAN DEFAULT FALSE,
    yanked_at TIMESTAMP WITH TIME ZONE,
    UNIQUE(package_id, version)
);

CREATE INDEX idx_versions_package ON versions(package_id);
CREATE INDEX idx_versions_published ON versions(published_at DESC);

-- Downloads table (for analytics)
CREATE TABLE IF NOT EXISTS downloads (
    download_id UUID PRIMARY KEY,
    package_id UUID NOT NULL REFERENCES packages(package_id) ON DELETE CASCADE,
    version_id UUID NOT NULL REFERENCES versions(version_id) ON DELETE CASCADE,
    downloaded_at TIMESTAMP WITH TIME ZONE NOT NULL,
    ip_hash VARCHAR(64) NOT NULL
);

CREATE INDEX idx_downloads_package ON downloads(package_id);
CREATE INDEX idx_downloads_version ON downloads(version_id);
CREATE INDEX idx_downloads_date ON downloads(downloaded_at DESC);

-- Package owners table (multiple owners per package)
CREATE TABLE IF NOT EXISTS package_owners (
    package_id UUID NOT NULL REFERENCES packages(package_id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
    role VARCHAR(20) NOT NULL, -- 'owner' or 'maintainer'
    added_at TIMESTAMP WITH TIME ZONE NOT NULL,
    PRIMARY KEY (package_id, user_id)
);

CREATE INDEX idx_owners_package ON package_owners(package_id);
CREATE INDEX idx_owners_user ON package_owners(user_id);
