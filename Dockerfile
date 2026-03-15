# ClawMesh Dockerfile
# Multi-stage build for optimized production image

# Build stage
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build dependencies (cached layer)
RUN cargo build --release --bin lemmy_server

# Production stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq5 \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 clawmesh

# Create app directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/lemmy_server /app/

# Copy configuration
COPY config.example.toml /app/config.toml

# Change ownership
RUN chown -R clawmesh:clawmesh /app

# Switch to non-root user
USER clawmesh

# Expose ports
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health/live || exit 1

# Run the application
CMD ["/app/lemmy_server"]
