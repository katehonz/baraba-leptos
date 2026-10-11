#!/bin/bash
set -e

# Baraba Production Deployment Script
# Домейн: baraba.org
# Block storage: /mnt/HC_Volume_1043336

BLOCK_STORAGE="/mnt/HC_Volume_104333652"
APP_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "=== Baraba Production Deployment ==="
echo "App directory: $APP_DIR"
echo "Block storage: $BLOCK_STORAGE"

# 1. Създай директории за данни на block storage
echo ""
echo "[1/6] Създаване на директории за PostgreSQL..."
sudo mkdir -p "$BLOCK_STORAGE/baraba/postgres"
sudo chown -R 999:999 "$BLOCK_STORAGE/baraba/postgres"

# 2. Провери .env файл
echo ""
echo "[2/6] Проверка на .env файл..."
if [ ! -f "$APP_DIR/.env" ]; then
    if [ -f "$APP_DIR/.env.production" ]; then
        cp "$APP_DIR/.env.production" "$APP_DIR/.env"
        echo "ВАЖНО: Редактирай .env файла и смени паролите!"
        echo "  nano $APP_DIR/.env"
        echo ""
        echo "Генерирай SECRET_KEY_BASE с: openssl rand -hex 32"
        exit 1
    else
        echo "ГРЕШКА: Липсва .env файл!"
        exit 1
    fi
fi

# 3. Провери дали caddy-network съществува
echo ""
echo "[3/6] Проверка на Docker мрежи..."
if ! docker network ls | grep -q "caddy-network"; then
    echo "Създаване на caddy-network..."
    docker network create caddy-network
else
    echo "caddy-network вече съществува"
fi

# 4. Build и стартиране
echo ""
echo "[4/6] Build на контейнери (това може да отнеме време)..."
cd "$APP_DIR"
docker compose -f docker-compose.prod.yml build

echo ""
echo "[5/6] Стартиране на контейнери..."
docker compose -f docker-compose.prod.yml up -d

# 5. Изчакай базата данни
echo ""
echo "[6/6] Изчакване базата данни да е готова..."
sleep 5

# Провери статуса
echo ""
echo "=== Статус на контейнерите ==="
docker compose -f docker-compose.prod.yml ps

echo ""
echo "=== Deployment завърши ==="
echo ""
echo "Следващи стъпки:"
echo "1. Добави Caddy конфигурацията:"
echo "   sudo cat $APP_DIR/docker/baraba.caddyfile >> /etc/caddy/Caddyfile"
echo "   sudo systemctl reload caddy"
echo ""
echo "2. Провери логовете:"
echo "   docker compose -f docker-compose.prod.yml logs -f"
echo ""
echo "3. Чакащите миграции вече са пуснати при старт на backend."
