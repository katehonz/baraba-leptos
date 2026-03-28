# Backend Dockerfile - Crystal 1.17.0
FROM crystallang/crystal:1.17.0-alpine AS builder

WORKDIR /app

# Install build dependencies
RUN apk add --no-cache \
    postgresql-dev \
    yaml-dev \
    yaml-static \
    openssl-dev \
    openssl-libs-static \
    zlib-dev \
    zlib-static \
    libxml2-dev \
    xz-dev

# Copy shard files first for better caching
COPY backend/shard.yml backend/shard.lock ./

# Install dependencies
RUN shards install --production

# Copy source code
COPY backend/src ./src
COPY backend/config ./config
COPY backend/db ./db

# Build the application
RUN crystal build src/start_server.cr -o baraba-server --release --no-debug

# Production image
FROM alpine:3.19

WORKDIR /app

# Install runtime dependencies
RUN apk add --no-cache \
    libpq \
    ca-certificates \
    tzdata \
    libxml2 \
    pcre2 \
    gmp \
    gc \
    libevent \
    libgcc \
    musl \
    yaml \
    gnu-libiconv \
    postgresql-client \
    python3 \
    py3-pip \
    && pip3 install --no-cache-dir --break-system-packages boto3>=1.26.0

# Use gnu-libiconv instead of musl iconv (needed for Windows-1251)
ENV LD_PRELOAD=/usr/lib/preloadable_libiconv.so

# Copy the binary
COPY --from=builder /app/baraba-server ./baraba-server

# Copy migrations, config and data
COPY backend/db/migrations ./db/migrations
COPY backend/config ./config
COPY backend/data ./data
COPY backend/scripts ./scripts

ENV PORT=3000
ENV LUCKY_ENV=production

EXPOSE 3000

CMD ["./baraba-server"]
