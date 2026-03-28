# Local Development Setup
# Date: 2026-01-26
# Architecture: Crystal/Lucky Backend + Karax Nim Frontend

---

## 🚀 Quick Start

### Prerequisites
- ✅ PostgreSQL running (localhost:5432)
- ✅ Crystal >= 1.16.3 installed
- ✅ Nim >= 2.0.0 installed
- ✅ Lucky CLI (optional but recommended)

---

## 📦 Database Setup

### 1. Create Database (if not exists)

```bash
# Connect to PostgreSQL
psql -U postgres -d postgres

# Create database
CREATE DATABASE lucky;

# Set password
ALTER USER postgres WITH PASSWORD 'your_password_here';

# Exit
\q
```

### 2. Database Connection

**Configuration** (already set in `backend/.env`):
- Host: localhost
- Port: 5432
- Database: lucky
- User: postgres
- Password: your_password_here

---

## 🔧 Backend Setup (Crystal/Lucky)

### 1. Install Dependencies

```bash
cd backend
shards install
```

### 2. Create & Migrate Database

```bash
# Create database
lucky db.create

# Run migrations
lucky db.migrate

# Or using Crystal directly
crystal run tasks/db/create.cr
crystal run tasks/db/migrate.cr
```

### 3. Start Development Server

```bash
# Using Lucky CLI (recommended)
lucky dev

# Or using Crystal directly
crystal run src/backend.cr
```

### 4. Access Backend

- **API**: http://localhost:5000
- **Health Check**: http://localhost:5000/api/health

---

## 🎨 Frontend Setup (Karax Nim)

### 1. Install Nim (if not already installed)

```bash
# Using choosenim (recommended)
curl -fsSL https://nim-lang.org/choosenim/init.sh -sSf | sh
source ~/.bashrc

# Or on Ubuntu/Debian
sudo apt install nim
```

### 2. Install Dependencies

```bash
cd frontend

# Install Karax and other dependencies
nimble install karax
nimble install --accept
```

### 3. Compile the Application

```bash
# Compile Nim to JavaScript
nim js src/app.nim

# This will generate app.js in the frontend directory
```

### 4. Start HTTP Server

```bash
# Option 1: Using Python (recommended for development)
python3 -m http.server 3000

# Option 2: Using Node.js and serve
npx serve -p 3000

# Option 3: Using Node.js and http-server
npx http-server -p 3000
```

### 5. Access Frontend

- **Frontend**: http://localhost:3000
- **Login**: Use email and password from backend

---

## 🔄 Full Workflow

### Step 1: Start PostgreSQL

```bash
# PostgreSQL should be running on port 5432
sudo systemctl status postgresql
# or
pg_isready
```

### Step 2: Start Backend

```bash
cd backend

# Install dependencies if needed
shards install

# Start server
lucky dev

# Wait for: "Server listening on http://localhost:5000"
```

### Step 3: Start Frontend

```bash
cd frontend

# Compile (do this whenever you make changes)
nim js src/app.nim

# Start HTTP server (in another terminal)
python3 -m http.server 3000

# Or use one command for quick testing
nim js src/app.nim && python3 -m http.server 3000
```

### Step 4: Test Application

1. Open http://localhost:3000
2. Register new user (via API or create in backend)
3. Login
4. Verify functionality

---

## 🧪 Testing

### Test Backend

```bash
# Health check
curl http://localhost:5000/api/health

# Sign up
curl -X POST http://localhost:5000/api/sign_ups \
  -H "Content-Type: application/json" \
  -d '{
    "user": {
      "email": "test@example.com",
      "password": "password123",
      "password_confirmation": "password123"
    }
  }'

# Sign in
curl -X POST http://localhost:5000/api/sign_ins \
  -H "Content-Type: application/json" \
  -d '{
    "user": {
      "email": "test@example.com",
      "password": "password123"
    }
  }'

# Get current user (needs token)
curl http://localhost:5000/api/me \
  -H "Authorization: Bearer <TOKEN>"
```

### Test Frontend

```bash
# Open browser
http://localhost:3000

# Check browser console for errors
# Check that API requests are being made
```

---

## 📝 Environment Variables

### Backend (.env in backend/)

```env
# Database
DB_HOST=localhost
DB_PORT=5432
DB_NAME=lucky
DB_USERNAME=postgres
DB_PASSWORD=your_secure_password_here

# Server
LUCKY_ENV=development

# JWT
SECRET_KEY_BASE=your-secret-key-here

# Database URL
DATABASE_URL=postgres://postgres:your_secure_password_here@localhost:5432/lucky
```

### Frontend (Constants in app.nim)

```nim
# In frontend/src/app.nim
const API_BASE = "http://localhost:5000"
```

---

## 🐛 Troubleshooting

### PostgreSQL Connection Failed

```bash
# Check if PostgreSQL is running
sudo systemctl status postgresql

# Start if stopped
sudo systemctl start postgresql

# Check port
sudo netstat -tlnp | grep 5432

# Check connection
psql -U postgres -d lucky -c "SELECT version();"
```

### Backend Compilation Issues

```bash
# Clean build
cd backend
rm -rf lib
shards install
lucky db.setup

# Check Crystal version
crystal --version

# Update shards
shards update
```

### Frontend Compilation Issues

```bash
# Check Nim version
nim --version

# Update Nim
choosenim update stable

# Clean and recompile
cd frontend
rm -f app.js
nim js src/app.nim

# Check for missing dependencies
nimble list
```

### Port Already in Use

```bash
# Check what's using port 5000
lsof -i :5000

# Kill process if needed
kill -9 <PID>

# Check port 3000
lsof -i :3000

# Kill process if needed
kill -9 <PID>
```

### Database Migrations Fail

```bash
# Check migration status
cd backend
lucky db.version

# Rollback and retry
lucky db.rollback
lucky db.migrate

# Or reset database (WARNING: deletes data!)
psql -U postgres -c "DROP DATABASE lucky;"
psql -U postgres -c "CREATE DATABASE lucky;"
lucky db.migrate
```

### Frontend Can't Connect to Backend

```bash
# Check if backend is running
curl http://localhost:5000/api/health

# Check API_BASE constant in app.nim
grep "API_BASE" frontend/src/app.nim

# Check CORS in backend (if needed)
# backend/src/actions/api/base_action.cr
```

---

## 📊 Database Schema

Current tables (after migrations):

- `users` - User accounts
- `user_tokens` - JWT tokens

Future tables (to be added):

- `companies` - Companies
- `accounts` - Chart of accounts
- `counterparts` - Counterparts
- `journal_entries` - Accounting entries
- `journal_entry_lines` - Entry lines
- `invoices` - Invoices
- `invoice_lines` - Invoice lines

Check after migrations:
```bash
psql -U postgres -d lucky -c "\dt"
```

---

## 🎯 Development Workflow

### Backend Workflow

1. **Make changes** to Crystal code
2. **Restart backend**: `lucky dev` (auto-reloads on save)
3. **Check logs** for errors
4. **Test API** with curl or Postman
5. **Run migrations** if needed: `lucky db.migrate`

### Frontend Workflow

1. **Make changes** to Nim code
2. **Recompile**: `nim js src/app.nim`
3. **Refresh browser** (cached app.js may need hard refresh)
4. **Check console** for errors
5. **Test UI** in browser

### Tips for Faster Development

```bash
# Watch mode for frontend (requires additional tool)
# Install nim-watch: nimble install nim-watch
nim-watch src/app.nim --cmd "nim js --opt:none src/app.nim"

# Or use a simple script:
while true; do
  inotifywait -e modify -r src/ 2>/dev/null
  nim js src/app.nim
  echo "Recompiled at $(date)"
done
```

---

## 📚 Useful Commands

### Backend

```bash
# Crystal/Lucky
cd backend

# Clean build
rm -rf lib
shards install

# Run specific migration
lucky db.migrate 20240101000001

# Rollback last migration
lucky db.rollback

# Create migration
lucky db.migrate.gen create_companies

# Run Crystal tests
crystal spec

# Build for production
crystal build src/backend.cr --release
```

### Frontend

```bash
# Nim/Karax
cd frontend

# Install dependency
nimble install <package-name>

# Compile with optimizations
nim js -d:release src/app.nim

# Compile without optimizations (faster for dev)
nim js -d:danger src/app.nim

# Run tests
nimble test

# Build for production
nim js -d:release --opt:speed src/app.nim

# Check for undefined references
nim js --listFullPaths src/app.nim
```

### Database

```bash
# Connect to database
psql -U postgres -d lucky

# List tables
\dt

# Describe table
\d users

# Run SQL file
\i /path/to/file.sql

# Export database
pg_dump -U postgres -d lucky > backup.sql

# Import database
psql -U postgres -d lucky < backup.sql

# Drop database
psql -U postgres -c "DROP DATABASE lucky;"

# Recreate database
psql -U postgres -c "DROP DATABASE IF EXISTS lucky;"
psql -U postgres -c "CREATE DATABASE lucky;"
cd backend
lucky db.migrate
```

---

## 🚀 Quick Start Script

Use the provided scripts for easy management:

```bash
# Start both backend and frontend
./start_local.sh

# Stop all services
./stop_local.sh

# Check status
./start_local.sh
# Then select option 7 (Check Status)
```

---

## 📖 Additional Resources

- **Crystal Documentation**: https://crystal-lang.org/docs/
- **Lucky Framework**: https://luckyframework.org/
- **Nim Documentation**: https://nim-lang.org/docs.html
- **Karax Documentation**: https://github.com/karaxnim/karax
- **PostgreSQL Documentation**: https://www.postgresql.org/docs/

---

## 🚀 Ready to Code!

Everything is set up for local development. Start coding! 🎉

**Backend**: http://localhost:5000
**Frontend**: http://localhost:3000
