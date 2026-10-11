#!/bin/bash

# Baraba Accounting - Full Stack Starter
# Backend: Lucky Framework (Crystal) - Port 5000
# Frontend: Leptos (Rust/WASM) - Port 8080

set -e

export PATH="$HOME/.local/bin:$PATH"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}=== Baraba Accounting ===${NC}"
echo -e "${BLUE}Backend:${NC}  http://localhost:5000"
echo -e "${BLUE}Frontend:${NC} http://localhost:8080"
echo ""

# Cleanup on exit
cleanup() {
    echo -e "\n${YELLOW}Stopping services...${NC}"
    [ -n "$BACKEND_PID" ] && kill $BACKEND_PID 2>/dev/null
    [ -n "$FRONTEND_PID" ] && kill $FRONTEND_PID 2>/dev/null
    exit 0
}
trap cleanup INT TERM

# Start backend
echo -e "${BLUE}Starting backend...${NC}"
cd backend
lucky dev &
BACKEND_PID=$!
cd ..

# Wait for backend
echo "Waiting for backend to initialize..."
sleep 5

# Start frontend with trunk
echo -e "${BLUE}Starting Leptos frontend with trunk...${NC}"
cd leptos
trunk serve --port 8080 &
FRONTEND_PID=$!
cd ..

echo ""
echo -e "${GREEN}All services running!${NC}"
echo "Press Ctrl+C to stop"

wait
