# Build stage
FROM rust:latest as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY schema.sql ./

# Build for release
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install PostgreSQL client and SSL libraries
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/shopone-backend /app/shopone-backend
COPY --from=builder /app/schema.sql /app/schema.sql

# Expose port
EXPOSE 3001

# Run the binary
CMD ["/app/shopone-backend"]
