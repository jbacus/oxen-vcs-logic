# Multi-stage Dockerfile for Auxin Server

# Stage 1: Build the Rust backend
FROM rust:1.91-slim as backend-builder

WORKDIR /build

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build for release
RUN cargo build --release --features mock-oxen

# Stage 2: Build the frontend
FROM node:20-alpine as frontend-builder

WORKDIR /build/frontend

# Copy frontend package files
COPY frontend/package*.json ./

# Install dependencies
RUN npm ci

# Copy frontend source
COPY frontend/ ./

# Build frontend
RUN npm run build

# Stage 3: Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 auxin

# Create directories
RUN mkdir -p /var/oxen/data && \
    chown -R auxin:auxin /var/oxen

WORKDIR /app

# Copy binary from backend builder
COPY --from=backend-builder /build/target/release/auxin-server /app/auxin-server

# Copy frontend from frontend builder
COPY --from=frontend-builder /build/frontend/dist /app/frontend/dist

# Copy default configuration
COPY config.docker.toml /app/config.toml

# Set ownership
RUN chown -R auxin:auxin /app

# Switch to app user
USER auxin

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Set environment variables (can be overridden at runtime)
# Using AUXIN_* prefix for consistency with unified configuration system
ENV RUST_LOG=info

# Run the server
# Configuration loaded from /app/config.toml by default
# Override with environment variables using AUXIN_* prefix
# Example: AUXIN_SERVER_PORT=3000
CMD ["/app/auxin-server"]
