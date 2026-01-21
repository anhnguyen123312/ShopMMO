# MMO API - Setup & Run Guide

Complete guide for running MongoDB + Redis in Docker and the API application locally.

## Prerequisites

- **Rust** 1.75+ (check with `rustc --version`)
- **Docker** & **Docker Compose** (check with `docker --version`)
- **Git** (check with `git --version`)

---

## Step 1: Start Database Services (MongoDB + Redis)

The Docker Compose file is configured to run **only** MongoDB and Redis. The API will run outside Docker.

```bash
# Start MongoDB and Redis in background
docker-compose up -d

# Verify services are running
docker-compose ps

# Check logs if needed
docker-compose logs -f mongodb
docker-compose logs -f redis
```

**Expected Output:**
```
NAME                IMAGE               STATUS
mmo-mongodb         mongo:7.0           Up (healthy)
mmo-redis           redis:7-alpine      Up (healthy)
```

### Service Details

| Service | Port | Credentials | Connection String |
|---------|------|-------------|-------------------|
| MongoDB | 27017 | User: `mmo_admin`<br>Pass: `mmo_secret_password` | `mongodb://mmo_admin:mmo_secret_password@localhost:27017` |
| Redis | 6379 | Pass: `mmo_redis_password` | `redis://:mmo_redis_password@localhost:6379` |

---

## Step 2: Environment Configuration

The `.env` file is already configured with the correct connection strings.

**Verify your `.env` file contains:**

```bash
# Database - MongoDB
MONGODB_URI=mongodb://mmo_admin:mmo_secret_password@localhost:27017
MONGODB_DATABASE=mmo_db

# Database - Redis
REDIS_URI=redis://:mmo_redis_password@localhost:6379

# Server
HOST=127.0.0.1
PORT=8080
RUST_LOG=info,mmo_api=debug

# JWT
JWT_SECRET=2dfdabafbc4109f4bb02c4c7207e71f5ebb59af31e736f2203bf3878
JWT_ACCESS_TOKEN_EXPIRES_IN=15m
JWT_REFRESH_TOKEN_EXPIRES_IN=7d
```

---

## Step 3: Generate & View Swagger Documentation

### Option A: Generate OpenAPI Spec Only

```bash
# Generate swagger/openapi.json
cargo run --bin generate_openapi
```

**Output:** `swagger/openapi.json` (277 KB)

### Option B: Run Interactive Swagger UI (Recommended)

```bash
# Start Swagger UI server on http://localhost:8081
cargo run --bin swagger_server
```

Then open in your browser:
```
http://localhost:8081/swagger-ui/
```

This provides an interactive API documentation interface where you can:
- Browse all endpoints
- View request/response schemas
- Test API calls directly from the browser

---

## Step 4: Seed Initial Data (Optional but Recommended)

Before running the API, seed the database with permissions, roles, and a super admin user:

```bash
# 1. Seed permissions
cargo run --bin seed_permissions

# 2. Seed roles (creates user, admin, moderator roles)
cargo run --bin seed_roles

# 3. Create super admin user
cargo run --bin create_super_admin

# 4. (Optional) Seed test data for development
cargo run --bin seed_test_data
```

**Default Super Admin Credentials:**
- Username: `admin` or `superadmin`
- Password: Check the script output or configure in the seed script

---

## Step 5: Run the API Application

### Development Mode (with hot reload logging)

```bash
# Run with debug logging
cargo run

# Or with verbose logging
RUST_LOG=debug cargo run
```

### Production Mode (optimized build)

```bash
# Build release binary
cargo build --release

# Run the optimized binary
./target/release/mmo-api
```

**Expected Output:**
```
🚀 Starting MMO API Server...
📊 Environment: development
🔌 Host: 127.0.0.1:8080
📝 Log Level: info,mmo_api=debug
✅ MongoDB connected: mmo_db
✅ Redis connected
🎯 Server started at http://127.0.0.1:8080
```

---

## Step 6: Test the API

### Health Check

```bash
curl http://localhost:8080/health
```

**Expected Response:**
```json
{
  "status": "ok",
  "timestamp": "2025-01-21T16:00:00Z"
}
```

### Register a User

```bash
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "email": "test@example.com",
    "password": "SecurePass123",
    "name": "Test User"
  }'
```

### Login

```bash
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "identifier": "test@example.com",
    "password": "SecurePass123"
  }'
```

**Response:**
```json
{
  "success": true,
  "data": {
    "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "user": {
      "id": "...",
      "username": "testuser",
      "email": "test@example.com"
    }
  }
}
```

### Authenticated Request

```bash
# Replace <ACCESS_TOKEN> with the token from login response
curl -X GET http://localhost:8080/api/auth/me \
  -H "Authorization: Bearer <ACCESS_TOKEN>"
```

---

## Step 7: Access Swagger UI

While the API is running, open your browser to:

```
http://localhost:8081/swagger-ui/
```

You can interact with all API endpoints directly from the Swagger UI:
1. Click on an endpoint to expand it
2. Click "Try it out"
3. Fill in parameters
4. Click "Execute" to test the API

---

## Quick Start Commands (TL;DR)

```bash
# 1. Start databases
docker-compose up -d

# 2. Generate Swagger (optional - view docs)
cargo run --bin generate_openapi

# 3. Seed data (first time only)
cargo run --bin seed_permissions
cargo run --bin seed_roles
cargo run --bin create_super_admin

# 4. Run API in development mode
cargo run

# 5. (In another terminal) Run Swagger UI
cargo run --bin swagger_server
```

**Access Points:**
- API: http://localhost:8080
- Swagger UI: http://localhost:8081/swagger-ui/

---

## Stopping Services

```bash
# Stop API application
# Press Ctrl+C in the terminal running cargo run

# Stop Swagger server
# Press Ctrl+C in the terminal running swagger_server

# Stop databases (keeps data)
docker-compose stop

# Stop and remove containers (keeps data in volumes)
docker-compose down

# Stop and remove everything including data
docker-compose down -v
```

---

## Troubleshooting

### MongoDB Connection Failed

```bash
# Check if MongoDB is running
docker-compose ps mongodb

# View MongoDB logs
docker-compose logs -f mongodb

# Test connection
mongosh "mongodb://mmo_admin:mmo_secret_password@localhost:27017"
```

### Redis Connection Failed

```bash
# Check if Redis is running
docker-compose ps redis

# View Redis logs
docker-compose logs -f redis

# Test connection
redis-cli -a mmo_redis_password ping
# Expected: PONG
```

### Port Already in Use

If port 8080 or 27017 or 6379 is already in use:

```bash
# Find process using port 8080
lsof -i :8080

# Kill process (replace PID)
kill -9 <PID>

# Or change the port in .env
PORT=8081
```

### Swagger Not Loading

Make sure you're running both:
1. The API server: `cargo run`
2. The Swagger server: `cargo run --bin swagger_server`

### Build Errors

```bash
# Clean build cache and rebuild
cargo clean
cargo build

# Update dependencies
cargo update
```

---

## Development Workflow

### Run with Auto-Reload (using cargo-watch)

```bash
# Install cargo-watch
cargo install cargo-watch

# Run with auto-reload on file changes
cargo watch -x run
```

### Format & Lint Code

```bash
# Format code
cargo fmt

# Lint with clippy
cargo clippy

# Fix clippy warnings automatically
cargo clippy --fix
```

### Run Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

---

## Production Deployment

For production deployment, consider:

1. **Use environment variables** for secrets (not `.env` file)
2. **Change JWT_SECRET** to a strong random value
3. **Set RUST_LOG** to `info` or `warn`
4. **Enable TLS/HTTPS** (use nginx/traefik as reverse proxy)
5. **Use managed MongoDB** (MongoDB Atlas) instead of Docker
6. **Use managed Redis** (Redis Cloud/ElastiCache)
7. **Run behind a load balancer** for high availability
8. **Enable monitoring** (Prometheus/Grafana)

---

## Useful Commands

```bash
# View API logs in real-time
cargo run 2>&1 | tee api.log

# Check database size
mongosh "mongodb://mmo_admin:mmo_secret_password@localhost:27017" --eval "db.stats()"

# Monitor Redis
redis-cli -a mmo_redis_password --stat

# View Docker resource usage
docker stats mmo-mongodb mmo-redis

# Backup MongoDB
docker exec mmo-mongodb mongodump --username mmo_admin --password mmo_secret_password --out /backup

# Restore MongoDB
docker exec mmo-mongodb mongorestore --username mmo_admin --password mmo_secret_password /backup
```

---

## API Endpoints Overview

| Category | Endpoint | Method | Auth |
|----------|----------|--------|------|
| **Auth** | `/api/auth/register` | POST | No |
| | `/api/auth/login` | POST | No |
| | `/api/auth/refresh` | POST | No |
| | `/api/auth/me` | GET | Yes |
| | `/api/auth/logout` | POST | Yes |
| **Wallet** | `/api/wallet/balance` | GET | Yes |
| | `/api/wallet/deposit/initiate` | POST | Yes |
| | `/api/wallet/withdraw` | POST | Yes |
| | `/api/wallet/transactions` | GET | Yes |
| **Categories** | `/api/categories/tree` | GET | No |
| | `/api/admin/categories` | POST | Admin |
| **Permissions** | `/api/permissions/roles` | GET | Admin |
| | `/api/permissions/roles` | POST | Admin |

Full API documentation: http://localhost:8081/swagger-ui/

---

## Need Help?

- Check logs: `docker-compose logs -f`
- Check API logs: `RUST_LOG=debug cargo run`
- View Swagger docs: http://localhost:8081/swagger-ui/
- Read the main README: [README.md](README.md)

---

**Happy Coding! 🚀**
