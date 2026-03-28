#!/bin/bash

# Stop All Services Script
# Stops all processes on ports: 3000, 3001, 5000, 5173, 8080

echo "=========================================="
echo "Stopping All Services"
echo "=========================================="
echo ""

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Ports to check
PORTS=(3000 3001 5000 5173 8080)

for PORT in "${PORTS[@]}"; do
    PID=$(lsof -ti :$PORT 2>/dev/null)
    if [ -n "$PID" ]; then
        echo -e "Port $PORT: ${YELLOW}Killing PID $PID${NC}"
        kill -9 $PID 2>/dev/null
        echo -e "Port $PORT: ${GREEN}Stopped${NC}"
    else
        echo -e "Port $PORT: ${GREEN}Free${NC}"
    fi
done

echo ""

# Kill by process name
echo "Stopping by process name..."

# Crystal/Lucky
pkill -f "crystal run" 2>/dev/null && echo -e "crystal run: ${GREEN}Stopped${NC}" || true
pkill -f "lucky" 2>/dev/null && echo -e "lucky: ${GREEN}Stopped${NC}" || true

# Trunk (Leptos dev server)
pkill -f "trunk serve" 2>/dev/null && echo -e "trunk serve: ${GREEN}Stopped${NC}" || true
pkill -f "trunk" 2>/dev/null && echo -e "trunk: ${GREEN}Stopped${NC}" || true

# HTTP servers
pkill -f "python3 -m http.server" 2>/dev/null && echo -e "python3 http.server: ${GREEN}Stopped${NC}" || true
pkill -f "npx serve" 2>/dev/null && echo -e "npx serve: ${GREEN}Stopped${NC}" || true

# Java/Spring
pkill -f "spring-boot:run" 2>/dev/null && echo -e "spring-boot: ${GREEN}Stopped${NC}" || true
pkill -f "java.*kankrum" 2>/dev/null && echo -e "java: ${GREEN}Stopped${NC}" || true

# Node/Vite
pkill -f "vite" 2>/dev/null && echo -e "vite: ${GREEN}Stopped${NC}" || true
pkill -f "node.*dev" 2>/dev/null && echo -e "node dev: ${GREEN}Stopped${NC}" || true

echo ""
echo "=========================================="
echo -e "${GREEN}All services stopped!${NC}"
echo "=========================================="
