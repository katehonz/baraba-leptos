#!/bin/bash
# Deployment script for Baraba on VPS

set -e

echo "=== Baraba Deployment ==="

# Check for .env file
if [ ! -f .env ]; then
    echo "ERROR: .env file not found!"
    echo "Copy .env.example to .env and fill in the values"
    exit 1
fi

# Load environment
source .env

# Check required variables
if [ -z "$POSTGRES_PASSWORD" ] || [ "$POSTGRES_PASSWORD" = "your-strong-password-here" ]; then
    echo "ERROR: POSTGRES_PASSWORD not set in .env"
    exit 1
fi

# Generate SECRET_KEY_BASE if not set
if [ -z "$SECRET_KEY_BASE" ] || [ "$SECRET_KEY_BASE" = "your-secret-key-here-minimum-32-chars" ]; then
    echo "Generating SECRET_KEY_BASE..."
    NEW_SECRET=$(openssl rand -hex 32)
    sed -i "s/SECRET_KEY_BASE=.*/SECRET_KEY_BASE=$NEW_SECRET/" .env
    echo "Generated and saved to .env"
fi

# Build and start containers
echo "Building containers (this may take a while)..."
docker compose build

echo "Starting database..."
docker compose up -d db
echo "Waiting for PostgreSQL to be ready..."
sleep 5

echo "Starting all services..."
docker compose up -d

# Wait for backend to be ready
echo "Waiting for backend to start..."
sleep 10

# Run migrations
echo "Running database migrations..."
docker compose exec -T backend ./baraba-server -- db.migrate

echo ""
echo "=== Deployment Complete ==="
echo ""
docker compose ps
echo ""
echo "Add the Caddyfile.example config to your Caddy configuration"
echo "to expose the application on your domain."
