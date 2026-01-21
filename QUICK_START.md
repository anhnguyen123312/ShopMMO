# 🚀 Quick Start Guide

Get the MMO API running in 5 minutes!

## Prerequisites Check

```bash
# Check if you have everything installed
rustc --version    # Should show 1.75+
docker --version   # Should work
cargo --version    # Should work
```

---

## 🎯 Step-by-Step Setup

### 1️⃣ Start Databases (MongoDB + Redis)

```bash
# Start Docker Desktop first, then run:
docker-compose up -d

# Verify they're running (should show "healthy"):
docker-compose ps
```

**Expected Output:**
```
NAME           IMAGE             STATUS
mmo-mongodb    mongo:7.0         Up (healthy)
mmo-redis      redis:7-alpine    Up (healthy)
```

---

### 2️⃣ Generate Swagger Documentation

```bash
# Generate OpenAPI spec file
cargo run --bin generate_openapi
```

**Result:** Creates `swagger/openapi.json`

---

### 3️⃣ (Optional) Seed Database

```bash
# Run these in order:
cargo run --bin seed_permissions
cargo run --bin seed_roles
cargo run --bin create_super_admin
```

---

### 4️⃣ Run the API

```bash
# Start the API server
cargo run
```

**You should see:**
```
🚀 Starting MMO API Server...
✅ MongoDB connected: mmo_db
✅ Redis connected
🎯 Server started at http://127.0.0.1:8080
```

---

### 5️⃣ (Optional) Run Swagger UI

**In a separate terminal:**

```bash
cargo run --bin swagger_server
```

Then open: **http://localhost:8081/swagger-ui/**

---

## 🧪 Test It Works

```bash
# Test health endpoint
curl http://localhost:8080/health

# Expected: {"status":"ok","timestamp":"..."}
```

---

## 📚 What's Configured

Your `.env` file is already set up with:

- **MongoDB:** `mongodb://mmo_admin:mmo_secret_password@localhost:27017`
- **Redis:** `redis://:mmo_redis_password@localhost:6379`
- **API Port:** `8080`
- **Swagger Port:** `8081`

---

## 🔗 Access Points

| Service | URL | Status |
|---------|-----|--------|
| API | http://localhost:8080 | ✅ Ready |
| Swagger UI | http://localhost:8081/swagger-ui/ | ✅ Ready |
| MongoDB | localhost:27017 | ✅ Running in Docker |
| Redis | localhost:6379 | ✅ Running in Docker |

---

## 🛑 Stop Everything

```bash
# Stop API - Press Ctrl+C in the terminal

# Stop databases (keeps data)
docker-compose stop

# Stop and remove containers (keeps data)
docker-compose down

# Remove everything including data
docker-compose down -v
```

---

## ⚠️ Common Issues

### "Cannot connect to Docker daemon"
→ Start Docker Desktop

### "Port 8080 already in use"
→ Change `PORT=8081` in `.env` file

### "MongoDB connection failed"
→ Run `docker-compose up -d` first

### "Build errors"
→ Run `cargo clean && cargo build`

---

## 📖 Full Documentation

See [SETUP_GUIDE.md](SETUP_GUIDE.md) for complete details and troubleshooting.

---

**That's it! You're ready to develop! 🎉**
