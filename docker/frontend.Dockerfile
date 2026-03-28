# Frontend Dockerfile - Rust / Trunk
FROM rust:latest AS builder

WORKDIR /app

# Install nightly Rust, Node.js, wasm target, trunk and sass
RUN apt-get update && apt-get install -y --no-install-recommends \
    nodejs npm && \
    npm install -g sass && \
    rustup default nightly && \
    rustup target add wasm32-unknown-unknown && \
    cargo install trunk --locked && \
    rm -rf /var/lib/apt/lists/*

# Copy Cargo files first for better caching
COPY leptos/Cargo.toml leptos/Cargo.lock ./

# Create dummy src to cache dependencies
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    echo "pub fn hydrate() {}" > src/lib.rs && \
    cargo build --release --target wasm32-unknown-unknown --features csr || true && \
    rm -rf src target

# Copy source code
COPY leptos/src ./src
COPY leptos/index.html ./
COPY leptos/Trunk.toml ./
COPY leptos/style ./style
COPY leptos/assets ./assets

# Build with trunk
RUN trunk build --release

# Production image - nginx to serve static files
FROM nginx:alpine

# Copy built assets
COPY --from=builder /app/dist /usr/share/nginx/html

# Copy static files (robots.txt, etc.)
COPY leptos/static/ /usr/share/nginx/html/

# Custom nginx config for SPA
RUN echo 'server { \
    listen 80; \
    root /usr/share/nginx/html; \
    index index.html; \
    location / { \
        try_files $uri $uri/ /index.html; \
        add_header Cache-Control "no-cache"; \
    } \
    location ~* \.(js|css|wasm|png|jpg|jpeg|gif|ico|svg)$ { \
        expires 7d; \
        add_header Cache-Control "public, must-revalidate"; \
    } \
}' > /etc/nginx/conf.d/default.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
