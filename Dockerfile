# Build stage
FROM rust:1.83-slim as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build for release
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    git \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/target/release/cooklang-backend /usr/local/bin/cooklang-backend

# Create directories for data
RUN mkdir -p /data/recipes /data/db

# Set environment variables
ENV RUST_LOG=info
ENV DATABASE_URL=sqlite:///data/db/cooklang.db
ENV RECIPES_PATH=/data/recipes

EXPOSE 3000

CMD ["cooklang-backend"]
