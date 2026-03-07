# CGAS Executor Service - Alpha Environment
# Multi-stage build for optimized image size

# ==================== Build Stage ====================
FROM rust:bookworm as builder

WORKDIR /app

# Install build dependencies for static linking
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    cmake \
    musl-tools \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY rust-workflow-engine/Cargo.toml rust-workflow-engine/Cargo.lock ./
COPY rust-workflow-engine/src ./src

# Build in release mode (dynamic linking)
RUN cargo build --release --bin rust-workflow-engine

# ==================== Runtime Stage ====================
FROM debian:bookworm-slim

# Install compatible libc version
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    postgresql-client \
    redis-tools \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -m -u 1000 cgas

# Copy binary from builder
COPY --from=builder /app/target/release/rust-workflow-engine /app/executor
COPY --from=builder /app/target/release/rust-workflow-engine /app/verifier

# Copy startup scripts
COPY docker/scripts/start-executor.sh /app/start-executor.sh
COPY docker/scripts/start-verifier.sh /app/start-verifier.sh

# Set ownership
RUN chown -R cgas:cgas /app \
    && chmod +x /app/start-executor.sh /app/start-verifier.sh

# Switch to non-root user
USER cgas

# Environment variables
ENV RUST_LOG=info
ENV ENVIRONMENT=alpha
ENV SERVICE_NAME=executor

# Expose ports
EXPOSE 8080 8081

# No healthcheck - mock mode uses idle loop
# HEALTHCHECK removed for mock executor

# Run executor service
CMD ["/app/start-executor.sh"]
