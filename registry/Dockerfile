# Multi-stage Dockerfile for RavensOne Registry Server
# Optimized for small image size and fast builds

# Stage 1: Build
FROM rust:latest as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code and migrations
COPY src ./src
COPY migrations ./migrations

# Build for release
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/registry-server /app/registry-server

# Copy migrations for runtime
COPY --from=builder /app/migrations /app/migrations

# Create storage directory
RUN mkdir -p /app/storage

# Expose port
EXPOSE 8080

# Run the server
CMD ["/app/registry-server"]
