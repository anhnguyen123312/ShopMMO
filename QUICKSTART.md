# Quick Start Guide

## 🚀 Get Started in 5 Minutes

### Prerequisites

- Rust 1.75+ installed
- MongoDB running on `localhost:27017`
- Redis running on `localhost:6379`

### Step 1: Clone and Setup

```bash
cd mmo-api
cp .env.example .env  # Or use the existing .env file
```

### Step 2: Run the Server

```bash
cargo run
```

You should see:
```
INFO Starting MMO API Server
INFO Configuration loaded host=127.0.0.1 port=8080
INFO Successfully connected to MongoDB
INFO Successfully connected to Redis
INFO Starting HTTP server at 127.0.0.1:8080
```

### Step 3: Test the API

#### Health Check
```bash
curl http://localhost:8080/health
# Output: OK
```

#### Register a User
```bash
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "Password123",
    "name": "Test User"
  }'
```

**Response:**
```json
{
  "success": true,
  "data": {
    "accessToken": "eyJhbGc...",
    "refreshToken": "eyJhbGc...",
    "tokenType": "Bearer",
    "expiresIn": 900,
    "user": {
      "id": "507f1f77bcf86cd799439011",
      "email": "test@example.com",
      "name": "Test User",
      "role": "user",
      "emailVerified": false
    }
  }
}
```

#### Login
```bash
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "Password123"
  }'
```

#### Get Current User (Protected Route)
```bash
# Save access token from login/register response
export TOKEN="your_access_token_here"

curl http://localhost:8080/api/auth/me \
  -H "Authorization: Bearer $TOKEN"
```

**Response:**
```json
{
  "success": true,
  "data": {
    "user_id": "507f1f77bcf86cd799439011",
    "email": "test@example.com",
    "role": "user"
  }
}
```

#### Get Wallet Balance
```bash
curl http://localhost:8080/api/wallet/balance \
  -H "Authorization: Bearer $TOKEN"
```

**Response:**
```json
{
  "success": true,
  "data": {
    "apCurrent": 0,
    "apPendingCashout": 0,
    "apTotal": 0,
    "vndEquivalent": 0
  }
}
```

## 🧪 Run Tests

```bash
cargo test
```

## 📝 Next Steps

1. **Read the docs:**
   - [README.md](README.md) - Full documentation
   - [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) - Architecture guide
   - [docs/CODING_STANDARDS.md](docs/CODING_STANDARDS.md) - Coding rules

2. **Create your first module:**
   - Copy the `wallet` module as a template
   - Follow the structure: domain → dto → repository → service → handler → routes

3. **Implement Wallet V2:**
   - See [../../docs/v2/01-wallet-system-design.md](../../docs/v2/01-wallet-system-design.md)
   - Implement transaction models
   - Add deposit/withdrawal flows
   - Implement escrow system

## 🛠️ Common Issues

### MongoDB Connection Failed
```bash
# Start MongoDB
mongod --dbpath /path/to/data

# Or use Docker
docker run -d -p 27017:27017 mongo:latest
```

### Redis Connection Failed
```bash
# Start Redis
redis-server

# Or use Docker
docker run -d -p 6379:6379 redis:latest
```

### Port Already in Use
Change `PORT` in `.env` file to a different port (e.g., 8081)

## 📚 API Endpoints

### Public (No Auth Required)
- `POST /api/auth/register` - Register new user
- `POST /api/auth/login` - Login
- `POST /api/auth/refresh` - Refresh access token
- `GET /health` - Health check

### Protected (Auth Required)
- `GET /api/auth/me` - Get current user
- `POST /api/auth/logout` - Logout
- `POST /api/auth/change-password` - Change password
- `GET /api/wallet/balance` - Get wallet balance

## 🎯 Development Workflow

1. **Make changes** to code
2. **Format code**: `cargo fmt`
3. **Check linting**: `cargo clippy`
4. **Run tests**: `cargo test`
5. **Run server**: `cargo run`
6. **Test endpoint** with curl/Postman

## 💡 Tips

- Use `RUST_LOG=debug` for detailed logs
- Check logs for request IDs to trace requests
- MongoDB data stored in `mmo_db` database
- Redis keys use prefixes: `session:*`, `cache:*`, etc.

Happy coding! 🚀
