# MMO API Server

Production-ready Rust API server for a marketplace/wallet system built with **actix-web**, **MongoDB**, and **Redis**.

## Features

### Core Features
- **JWT Authentication** - Access + Refresh token flow with V2 multi-role support
- **Role-Based Authorization** - Dynamic RBAC with permission-based access control
- **MongoDB** - Document database with connection pooling and transaction support
- **Redis** - Caching and session management
- **Structured Logging** - Using tracing crate with JSON format support
- **Error Handling** - Type-safe 3-layer error handling (ApiError, ServiceError, DbError)
- **Input Validation** - Request validation with validator crate
- **CORS Support** - Configurable cross-origin requests
- **OpenAPI/Swagger** - Auto-generated API documentation with utoipa
- **Request Tracking** - Unique request IDs for distributed tracing

### Business Modules
- **Auth Module** - User registration, login, token refresh, password management
- **Wallet Module** - Trust currency system with deposits, withdrawals, escrow
- **Permissions Module** - Dynamic role and permission management
- **Category Module** - Hierarchical category management
- **Shop Module** - Marketplace shop management (in development)

## Tech Stack

| Component | Technology | Version |
|-----------|-----------|---------|
| Language | Rust | 2021 edition |
| Web Framework | actix-web | 4.9 |
| Database | MongoDB | 3.1 driver |
| Cache | Redis | 0.27 |
| Authentication | jsonwebtoken | 9.3 |
| Password Hashing | bcrypt + argon2 | 0.15 / 0.5 |
| Validation | validator | 0.18 |
| Logging | tracing | 0.1 |
| API Docs | utoipa | 5.4.0 |
| Async Runtime | Tokio | 1.42 |

## Project Structure

```
mmo-api/
├── src/
│   ├── main.rs                    # Application entry point
│   ├── lib.rs                     # Library exports
│   ├── config/                    # Configuration management
│   │   ├── mod.rs
│   │   └── app.rs                 # Environment-based config
│   ├── core/                      # Core infrastructure
│   │   ├── mod.rs
│   │   ├── errors.rs              # 3-layer error types (Api/Service/Db)
│   │   ├── response.rs            # Standard API response wrapper
│   │   ├── logger.rs              # Structured logging setup
│   │   ├── validator.rs           # Custom validators
│   │   └── ownership.rs           # Authorization helpers
│   ├── database/                  # Database connections
│   │   ├── mod.rs
│   │   ├── mongodb.rs             # MongoDB client with pooling
│   │   └── redis.rs               # Redis client
│   ├── middleware/                # HTTP middleware
│   │   ├── mod.rs
│   │   ├── auth.rs                # JWT authentication
│   │   ├── authorization.rs       # Role-based access control
│   │   ├── permissions.rs         # Permission extraction
│   │   ├── request_id.rs          # Request ID tracking
│   │   └── cors.rs                # CORS configuration
│   ├── modules/                   # Feature modules
│   │   ├── mod.rs
│   │   ├── auth/                  # Authentication module
│   │   │   ├── domain.rs          # User, RefreshToken models
│   │   │   ├── dto.rs             # Request/Response DTOs
│   │   │   ├── handler.rs         # HTTP handlers
│   │   │   ├── service.rs         # Business logic
│   │   │   ├── repository.rs      # Database operations
│   │   │   ├── routes.rs          # Route definitions
│   │   │   └── mod.rs
│   │   ├── wallet/                # Wallet module (V3)
│   │   │   ├── domain.rs          # Wallet, Transaction, Escrow models
│   │   │   ├── dto.rs             # DTOs
│   │   │   ├── handler.rs         # HTTP handlers
│   │   │   ├── service.rs         # Core wallet operations
│   │   │   ├── service_escrow.rs  # Escrow operations
│   │   │   ├── service_admin.rs   # Admin operations
│   │   │   ├── service_usdt.rs    # USDT integration
│   │   │   ├── service_cron.rs    # Background jobs
│   │   │   ├── repository.rs      # Database operations
│   │   │   ├── routes.rs          # Route definitions
│   │   │   └── mod.rs
│   │   ├── permissions/           # Permission management
│   │   ├── category/              # Category management
│   │   └── shop/                  # Shop management
│   ├── utils/                     # Utility functions
│   │   ├── mod.rs
│   │   ├── jwt.rs                 # JWT token utilities
│   │   ├── hash.rs                # Password hashing
│   │   ├── datetime.rs            # Date/time helpers
│   │   └── number_generator.rs    # ID generation
│   └── openapi.rs                 # OpenAPI spec definition
├── migrations/                    # Database migrations
├── scripts/                       # Utility scripts
├── docs/                          # Documentation
│   └── v2/                        # V2 design documents
├── Cargo.toml                     # Dependencies
├── docker-compose.yml             # Docker Compose for local dev
└── .env.example                   # Environment template
```

## Prerequisites

- **Rust** 1.75+ (2021 edition)
- **MongoDB** 4.4+
- **Redis** 6.0+

## Quick Start

### 1. Clone and Setup

```bash
git clone <repository-url>
cd mmo-api

# Copy environment template
cp .env.example .env
```

### 2. Configure Environment

Edit `.env` with your configuration:

```env
# Server
HOST=127.0.0.1
PORT=8080
RUST_LOG=info,mmo_api=debug
SERVER_WORKERS=4

# MongoDB
MONGODB_URI=mongodb://localhost:27017
MONGODB_DATABASE=mmo_db
MONGODB_MAX_POOL_SIZE=100
MONGODB_MIN_POOL_SIZE=10

# Redis
REDIS_URI=redis://localhost:6379

# JWT
JWT_SECRET=your-super-secret-jwt-key-change-in-production
JWT_ACCESS_TOKEN_EXPIRES_IN=15m
JWT_REFRESH_TOKEN_EXPIRES_IN=7d

# Security
BCRYPT_COST=12
```

### 3. Start Dependencies

Using Docker Compose:
```bash
docker-compose up -d mongodb redis
```

Or start manually:
```bash
# MongoDB
mongod --dbpath /data/db

# Redis
redis-server
```

### 4. Build and Run

```bash
# Development
cargo run

# Production (optimized)
cargo build --release
./target/release/mmo-api
```

### 5. Seed Initial Data (Optional)

```bash
# Seed permissions
cargo run --bin seed_permissions

# Seed roles
cargo run --bin seed_roles

# Create super admin
cargo run --bin create_super_admin
```

## API Documentation

### Swagger UI

Start the Swagger server:
```bash
cargo run --bin swagger_server
```

Then visit: http://localhost:8081/swagger-ui/

### Generate OpenAPI Spec

```bash
cargo run --bin generate_openapi
# Output: swagger/openapi.json
```

## API Endpoints

### Authentication

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| POST | `/api/auth/register` | Register new user | No |
| POST | `/api/auth/login` | Login | No |
| POST | `/api/auth/refresh` | Refresh access token | No |
| POST | `/api/auth/logout` | Logout | Yes |
| GET | `/api/auth/me` | Get current user | Yes |
| POST | `/api/auth/change-password` | Change password | Yes |
| POST | `/api/auth/admin/assign-roles` | Assign roles to user | Admin |
| GET | `/api/auth/admin/users/{id}/roles` | Get user roles | Admin |

### Wallet

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| GET | `/api/wallet/balance` | Get wallet balance | Yes |
| POST | `/api/wallet/deposit/initiate` | Initiate deposit | Yes |
| POST | `/api/wallet/withdraw` | Create withdrawal | Yes |
| GET | `/api/wallet/transactions` | Transaction history | Yes |
| POST | `/api/wallet/escrow/create` | Create escrow | Yes |
| POST | `/api/wallet/escrow/release` | Release escrow | Yes |

### Categories

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| GET | `/api/categories/tree` | Get category tree | No |
| GET | `/api/categories/{id}` | Get category by ID | No |
| POST | `/api/admin/categories` | Create category | Admin |
| PUT | `/api/admin/categories/{id}` | Update category | Admin |
| DELETE | `/api/admin/categories/{id}` | Delete category | Admin |

### Permissions

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| GET | `/api/permissions/roles` | List all roles | Admin |
| POST | `/api/permissions/roles` | Create role | Admin |
| PUT | `/api/permissions/roles/{name}` | Update role | Admin |
| DELETE | `/api/permissions/roles/{name}` | Delete role | Admin |

## Request/Response Format

### Standard Success Response

```json
{
  "success": true,
  "message": null,
  "data": { /* payload */ },
  "error": null
}
```

### Standard Error Response

```json
{
  "success": false,
  "message": "Error description",
  "data": null,
  "error": {
    "error": "Detailed error message",
    "status_code": 400
  }
}
```

### Authentication Example

**Register:**
```bash
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "johndoe",
    "email": "john@example.com",
    "password": "SecurePass123",
    "name": "John Doe"
  }'
```

**Login:**
```bash
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "identifier": "john@example.com",
    "password": "SecurePass123"
  }'
```

**Authenticated Request:**
```bash
curl -X GET http://localhost:8080/api/auth/me \
  -H "Authorization: Bearer <access_token>"
```

## Development

### Code Quality

```bash
# Format code
cargo fmt

# Lint with clippy
cargo clippy

# Run tests
cargo test

# Run with verbose logging
RUST_LOG=debug cargo run
```

### Adding a New Module

1. Create module directory: `src/modules/your_module/`
2. Add standard files:
   - `domain.rs` - MongoDB models
   - `dto.rs` - Request/Response DTOs
   - `handler.rs` - HTTP handlers
   - `service.rs` - Business logic
   - `repository.rs` - Database operations
   - `routes.rs` - Route configuration
   - `mod.rs` - Module exports
3. Register in `src/modules/mod.rs`
4. Configure routes in `src/main.rs`

See `docs/WORKFLOW_ADD_NEW_API.md` for detailed guide.

## Database Indexes

Run these in MongoDB shell for optimal performance:

```javascript
// Users
db.users.createIndex({ "email": 1 }, { unique: true })
db.users.createIndex({ "username": 1 }, { unique: true })

// Refresh tokens
db.refresh_tokens.createIndex({ "token": 1 }, { unique: true })
db.refresh_tokens.createIndex({ "user_id": 1 })
db.refresh_tokens.createIndex({ "expires_at": 1 }, { expireAfterSeconds: 0 })

// Wallets
db.wallets.createIndex({ "wallet_id": 1 }, { unique: true })
db.wallets.createIndex({ "user_id": 1 }, { unique: true })

// Transactions
db.wallet_transactions.createIndex({ "tx_id": 1 }, { unique: true })
db.wallet_transactions.createIndex({ "wallet_id": 1, "created_at": -1 })

// Categories
db.categories.createIndex({ "slug": 1 }, { unique: true })
db.categories.createIndex({ "parent_id": 1 })
```

## Deployment

### Docker

```dockerfile
FROM rust:1.75-slim as builder
WORKDIR /app
COPY . .
RUN apt-get update && apt-get install -y pkg-config libssl-dev && \
    cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/mmo-api /usr/local/bin/
EXPOSE 8080
CMD ["mmo-api"]
```

### Docker Compose (Production)

```yaml
version: '3.8'
services:
  api:
    build: .
    ports:
      - "8080:8080"
    environment:
      - RUST_LOG=info
      - MONGODB_URI=mongodb://mongodb:27017
      - REDIS_URI=redis://redis:6379
    depends_on:
      - mongodb
      - redis
    restart: always

  mongodb:
    image: mongo:7
    volumes:
      - mongodb_data:/data/db
    restart: always

  redis:
    image: redis:7-alpine
    restart: always

volumes:
  mongodb_data:
```

## Architecture

### Request Flow

```
HTTP Request
    │
    ▼
┌─────────────────────────┐
│     Middleware Stack    │
│  ├─ TracingLogger       │
│  ├─ RequestId           │
│  ├─ AuthMiddleware      │
│  └─ GrantsMiddleware    │
└─────────────────────────┘
    │
    ▼
┌─────────────────────────┐
│       Handler           │
│  (validates input)      │
└─────────────────────────┘
    │
    ▼
┌─────────────────────────┐
│       Service           │
│  (business logic)       │
└─────────────────────────┘
    │
    ▼
┌─────────────────────────┐
│      Repository         │
│  (database operations)  │
└─────────────────────────┘
    │
    ▼
┌─────────────────────────┐
│    MongoDB / Redis      │
└─────────────────────────┘
```

### Error Handling Chain

```
DbError → ServiceError → ApiError → HTTP Response
```

## Documentation

- [Workflow: Add New API](docs/WORKFLOW_ADD_NEW_API.md)
- [Wallet System Design](docs/v2/wallet/wallet-overview.md)
- [Escrow Flow](docs/v2/wallet/escrow.md)
- [Full Implementation Order](docs/v2/full-flows-implementation-order.md)

## Troubleshooting

### Common Issues

1. **MongoDB connection failed**
   - Check `MONGODB_URI` in `.env`
   - Ensure MongoDB is running: `mongosh --eval "db.runCommand({ping:1})"`

2. **Redis connection failed**
   - Check `REDIS_URI` in `.env`
   - Ensure Redis is running: `redis-cli ping`

3. **JWT errors**
   - Verify `JWT_SECRET` is set
   - Check token expiration settings

4. **Permission denied**
   - Verify user has required roles
   - Check role-permission mappings in database

### Debug Logging

```bash
# All debug logs
RUST_LOG=debug cargo run

# Module-specific logging
RUST_LOG=mmo_api::modules::auth=debug cargo run

# JSON format (for production)
LOG_FORMAT=json cargo run
```

## License

[MIT License](LICENSE)

## Contributing

1. Fork the repository
2. Create feature branch: `git checkout -b feature/amazing-feature`
3. Commit changes: `git commit -m 'Add amazing feature'`
4. Push to branch: `git push origin feature/amazing-feature`
5. Open a Pull Request

## Support

For issues and questions, please open an issue on GitHub.
