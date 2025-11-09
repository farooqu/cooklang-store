# Build stage
FROM rust:1.83-alpine as builder

WORKDIR /app

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    openssl-libs-static

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build for release with musl target for maximum compatibility
RUN cargo build --release --target x86_64-unknown-linux-musl

# Runtime stage
FROM alpine:3.20

WORKDIR /app

# Install minimal runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    libcrypto3 \
    libssl3 \
    git \
    curl

# Create non-root user
RUN addgroup -g 1000 cooklang && \
    adduser -D -u 1000 -G cooklang cooklang

# Create recipes directory with proper ownership
RUN mkdir -p /recipes && \
    chown -R cooklang:cooklang /app /recipes

# Copy the binary from builder
COPY --from=builder --chown=cooklang:cooklang /app/target/x86_64-unknown-linux-musl/release/cooklang-store /usr/local/bin/cooklang-store

# Set environment variables
ENV RUST_LOG=info
ENV RECIPES_PATH=/recipes
ENV COOKLANG_STORAGE_TYPE=disk

EXPOSE 3000

# Switch to non-root user
USER cooklang

# Health check for orchestration systems
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

CMD ["cooklang-store"]
