#!/bin/bash

# Lucky Framework + Leptos Rust Local Development Starter
# Crystal Backend + Leptos Rust/WASM Frontend
# Date: 2026-01-27

set -e

echo "=========================================="
echo "Lucky Framework + Leptos Rust Starter"
echo "=========================================="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

# Check if in correct directory
if [ ! -f "backend/shard.yml" ]; then
    print_error "Please run from baraba-2 directory"
    exit 1
fi

# Check PostgreSQL
echo "Checking PostgreSQL..."
if pg_isready > /dev/null 2>&1; then
    print_success "PostgreSQL is running"
else
    print_error "PostgreSQL is not running"
    echo "Start it with: sudo systemctl start postgresql"
    exit 1
fi
echo ""

# Check Crystal
echo "Checking Crystal..."
if command -v crystal >/dev/null 2>&1; then
    CRYSTAL_VERSION=$(crystal --version | head -1)
    print_success "Crystal installed: $CRYSTAL_VERSION"
else
    print_error "Crystal is not installed"
    echo "Install: https://crystal-lang.org/install/"
    exit 1
fi
echo ""

# Check Rust
echo "Checking Rust..."
if command -v rustc >/dev/null 2>&1; then
    RUST_VERSION=$(rustc --version)
    print_success "Rust installed: $RUST_VERSION"
else
    print_warning "Rust is not installed"
    echo "Install: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
fi
echo ""

# Check Trunk
echo "Checking Trunk..."
if command -v trunk >/dev/null 2>&1; then
    print_success "Trunk installed"
else
    print_warning "Trunk not found"
    echo "Install: cargo install trunk"
fi
echo ""

# Check Lucky CLI
echo "Checking Lucky CLI..."
if command -v lucky >/dev/null 2>&1; then
    print_success "Lucky CLI installed"
else
    print_warning "Lucky CLI not found (optional)"
fi
echo ""

# Create database if not exists
echo "Checking database..."
DB_NAME="lucky"
DB_EXISTS=$(psql -U postgres -tAc "SELECT 1 FROM pg_database WHERE datname='$DB_NAME'" 2>/dev/null || echo "")

if [ -z "$DB_EXISTS" ]; then
    echo "Creating database '$DB_NAME'..."
    psql -U postgres -c "CREATE DATABASE $DB_NAME;" > /dev/null 2>&1 && \
        print_success "Database '$DB_NAME' created" || \
        print_error "Failed to create database"
else
    print_success "Database '$DB_NAME' exists"
fi
echo ""

# Menu
echo "Please select an option:"
echo "1) Start Backend only (Crystal/Lucky)"
echo "2) Start Frontend only (Leptos Rust)"
echo "3) Start Both (recommended)"
echo "4) Install Dependencies"
echo "5) Run Database Migrations"
echo "6) Stop All"
echo "7) Check Status"
echo "8) Clean Database"
echo "9) Exit"
echo ""

read -p "Enter option [1-9]: " option

case $option in
    1)
        echo ""
        echo "Starting Lucky Backend..."
        echo "=========================================="
        cd backend

        # Check if shards are installed
        if [ ! -d "lib" ]; then
            echo "Installing shards..."
            shards install
        fi

        # Start Lucky dev server
        if command -v lucky >/dev/null 2>&1; then
            lucky dev
        else
            echo "Compiling and running..."
            crystal run src/backend.cr
        fi
        ;;
    2)
        echo ""
        echo "Starting Leptos Rust Frontend..."
        echo "=========================================="
        cd leptos

        if ! command -v trunk >/dev/null 2>&1; then
            print_error "Trunk is not installed!"
            echo "Install with: cargo install trunk"
            exit 1
        fi

        # Start trunk dev server
        echo "Starting trunk dev server on port 8080..."
        trunk serve --port 8080
        ;;
    3)
        echo ""
        echo "Starting Backend & Frontend..."
        echo "=========================================="

        # Start backend in background
        echo "Starting Lucky backend..."
        cd backend

        if [ ! -d "lib" ]; then
            echo "Installing shards..."
            shards install
        fi

        if command -v lucky >/dev/null 2>&1; then
            lucky dev > /tmp/lucky-backend.log 2>&1 &
        else
            crystal run src/backend.cr > /tmp/lucky-backend.log 2>&1 &
        fi
        BACKEND_PID=$!
        echo "Backend started (PID: $BACKEND_PID)"
        echo "Logs: tail -f /tmp/lucky-backend.log"
        echo ""

        # Wait for backend
        echo "Waiting for backend to compile..."
        sleep 10

        # Start frontend
        cd ../leptos
        echo "Starting Leptos Rust frontend..."

        # Start trunk dev server
        trunk serve --port 8080 &
        FRONTEND_PID=$!
        echo "Frontend started (PID: $FRONTEND_PID)"
        echo ""

        echo "Both services are running!"
        echo "Backend: http://localhost:5000"
        echo "Frontend: http://localhost:8080"
        echo ""
        echo "Press Ctrl+C to stop all services"

        # Wait for Ctrl+C
        trap "kill $BACKEND_PID 2>/dev/null; kill $FRONTEND_PID 2>/dev/null; exit 0" INT TERM

        # Wait indefinitely
        wait
        ;;
    4)
        echo ""
        echo "Installing Dependencies..."
        echo "=========================================="

        echo "Installing Crystal shards..."
        cd backend
        shards install
        print_success "Backend dependencies installed"

        cd ../leptos
        if command -v cargo >/dev/null 2>&1; then
            echo "Installing Rust/WASM dependencies..."
            rustup target add wasm32-unknown-unknown 2>/dev/null || true
            cargo install trunk 2>/dev/null || true
            print_success "Frontend dependencies installed"
        else
            print_warning "Rust not found, skipping frontend dependencies"
        fi
        ;;
    5)
        echo ""
        echo "Running Database Migrations..."
        echo "=========================================="
        cd backend

        if [ ! -d "lib" ]; then
            echo "Installing shards first..."
            shards install
        fi

        if command -v lucky >/dev/null 2>&1; then
            lucky db.migrate
        else
            crystal run tasks/db/migrate.cr
        fi
        print_success "Migrations complete"
        ;;
    6)
        echo ""
        echo "Stopping all processes..."
        echo "=========================================="

        # Kill Crystal/Lucky processes
        pkill -f "crystal run" && print_success "Backend stopped" || print_warning "Backend not running"
        pkill -f "lucky" && print_success "Lucky stopped" || true

        # Kill trunk server
        pkill -f "trunk serve" && print_success "Trunk stopped" || print_warning "Trunk not running"

        print_success "All processes stopped"
        ;;
    7)
        echo ""
        echo "Status Check"
        echo "=========================================="

        # PostgreSQL
        echo -n "PostgreSQL: "
        pg_isready > /dev/null 2>&1 && echo -e "${GREEN}Running${NC}" || echo -e "${RED}Stopped${NC}"

        # Backend (Lucky default port 5000)
        echo -n "Backend (5000): "
        curl -s http://localhost:5000/api/health > /dev/null 2>&1 && echo -e "${GREEN}Running${NC}" || echo -e "${RED}Stopped${NC}"

        # Frontend (default port 8080)
        echo -n "Frontend (8080): "
        curl -s http://localhost:8080 > /dev/null 2>&1 && echo -e "${GREEN}Running${NC}" || echo -e "${RED}Stopped${NC}"

        # Database
        echo ""
        echo "Database tables:"
        psql -U postgres -d lucky -c "\dt" 2>/dev/null || echo "No tables or cannot connect"
        ;;
    8)
        echo ""
        echo "Cleaning Database..."
        echo "=========================================="
        print_warning "This will DELETE all data!"
        read -p "Are you sure? (yes/no): " confirm

        if [ "$confirm" == "yes" ]; then
            echo "Dropping database..."
            psql -U postgres -c "DROP DATABASE IF EXISTS lucky;" > /dev/null 2>&1
            sleep 1

            echo "Creating database..."
            psql -U postgres -c "CREATE DATABASE lucky;" > /dev/null 2>&1

            print_success "Database cleaned!"
            echo "Run option 5 to apply migrations"
        else
            echo "Cancelled."
        fi
        ;;
    9)
        echo "Goodbye!"
        exit 0
        ;;
    *)
        print_error "Invalid option"
        exit 1
        ;;
esac

echo ""
echo "=========================================="
print_success "Done!"
echo "=========================================="
