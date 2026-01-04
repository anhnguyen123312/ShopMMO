# MMO API Server

Production-ready Rust API server built with **actix-web** and **MongoDB**.

## 🚀 Features

- ✅ **JWT Authentication** - Access + Refresh token flow
- ✅ **Role-Based Authorization** - Admin, User, Seller roles
- ✅ **MongoDB** - Document database with connection pooling
- ✅ **Redis** - Caching and session management
- ✅ **Structured Logging** - Using tracing crate
- ✅ **Error Handling** - Type-safe error handling with custom types
- ✅ **Input Validation** - Request validation with validator crate
- ✅ **CORS Support** - Configurable cross-origin requests
- ✅ **Clean Architecture** - Modular design with clear separation of concerns
- ✅ **Transaction Support** - MongoDB multi-document transactions
- ✅ **Request Tracking** - Unique request IDs for tracing

## 📁 Project Structure

```
mmo-api/
├── src/
│   ├── main.rs                 # Application entry point
│   ├── config/                 # Configuration management
│   │   └── app.rs             # Environment-based config
│   ├── core/                   # Core infrastructure
│   │   ├── errors.rs          # Error types & handling
│   │   ├── response.rs        # Standard API responses
│   │   ├── logger.rs          # Logging setup
│   │   └── validator.rs       # Custom validators
│   ├── database/               # Database connections
│   │   ├── mongodb.rs         # MongoDB client
│   │   └── redis.rs           # Redis client
│   ├── middleware/             # Middleware components
│   │   ├── auth.rs            # JWT authentication
│   │   ├── authorization.rs   # Role-based access control
│   │   ├── cors.rs            # CORS configuration
│   │   └── request_id.rs      # Request ID tracking
│   ├── modules/                # Feature modules
│   │   ├── auth/              # Authentication module
│   │   │   ├── domain.rs      # Domain models (User, RefreshToken)
│   │   │   ├── dto.rs         # Request/Response DTOs
│   │   │   ├── handler.rs     # HTTP handlers
│   │   │   ├── service.rs     # Business logic
│   │   │   ├── repository.rs  # Database operations
│   │   │   └── routes.rs      # Route definitions
│   │   └── wallet/            # Wallet module (template)
│   │       └── ...
│   └── utils/                  # Utility functions
│       ├── hash.rs            # Password hashing
│       ├── jwt.rs             # JWT utilities
│       ├── number_generator.rs # ID generation
│       └── datetime.rs        # Date/time helpers
├── Cargo.toml
├── .env.example
└── README.md
```

## 🛠️ Prerequisites

- **Rust** 1.75+ (2021 edition)
- **MongoDB** 4.4+
- **Redis** 6.0+

## 📦 Installation

### 1. Clone the repository

```bash
git clone <repository-url>
cd mmo-api
```

### 2. Set up environment variables

```bash
cp .env.example .env
```

Edit `.env` with your configuration:

```env
# Server
HOST=127.0.0.1
PORT=8080
RUST_LOG=info,mmo_api=debug

# MongoDB
MONGODB_URI=mongodb://localhost:27017
MONGODB_DATABASE=mmo_db

# Redis
REDIS_URI=redis://localhost:6379

# JWT
JWT_SECRET=your-super-secret-jwt-key-change-in-production
JWT_ACCESS_TOKEN_EXPIRES_IN=15m
JWT_REFRESH_TOKEN_EXPIRES_IN=7d
```

### 3. Install dependencies

```bash
cargo build
```

### 4. Run the server

**Development:**
```bash
cargo run
```

**Production (optimized):**
```bash
cargo build --release
./target/release/mmo-api
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with logging
RUST_LOG=debug cargo test

# Run specific test
cargo test test_name
```

## 📚 API Documentation

### Authentication Endpoints

#### Register
```http
POST /api/auth/register
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "Password123",
  "name": "John Doe"
}
```

#### Login
```http
POST /api/auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "Password123"
}
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
      "email": "user@example.com",
      "name": "John Doe",
      "role": "user"
    }
  }
}
```

#### Refresh Token
```http
POST /api/auth/refresh
Content-Type: application/json

{
  "refreshToken": "eyJhbGc..."
}
```

#### Get Current User
```http
GET /api/auth/me
Authorization: Bearer <access_token>
```

#### Change Password
```http
POST /api/auth/change-password
Authorization: Bearer <access_token>
Content-Type: application/json

{
  "currentPassword": "OldPassword123",
  "newPassword": "NewPassword456"
}
```

#### Logout
```http
POST /api/auth/logout
Authorization: Bearer <access_token>
Content-Type: application/json

{
  "refreshToken": "eyJhbGc..."
}
```

### Wallet Endpoints

#### Get Balance
```http
GET /api/wallet/balance
Authorization: Bearer <access_token>
```

**Response:**
```json
{
  "success": true,
  "data": {
    "apCurrent": 5000,
    "apPendingCashout": 2000,
    "apTotal": 7000,
    "vndEquivalent": 7000000
  }
}
```

## 🏗️ Module Development Guide

### Creating a New Module

Follow this structure for consistency:

```
modules/your_module/
├── domain.rs      # MongoDB models
├── dto.rs         # Request/Response DTOs
├── handler.rs     # HTTP handlers
├── service.rs     # Business logic
├── repository.rs  # Database operations
├── routes.rs      # Route configuration
└── mod.rs         # Module exports
```

### Example: Adding a Product Module

1. **Create domain models** (`domain.rs`):
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Product {
    #[serde(rename = "_id")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub price: i64,
    // ...
}
```

2. **Define DTOs** (`dto.rs`):
```rust
#[derive(Debug, Deserialize, Validate)]
pub struct CreateProductRequest {
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(range(min = 0))]
    pub price: i64,
}
```

3. **Implement repository** (`repository.rs`):
```rust
pub struct ProductRepository {
    collection: Collection<Product>,
}

impl ProductRepository {
    pub async fn create(&self, product: Product) -> Result<Product, DbError> {
        // ...
    }
}
```

4. **Implement service** (`service.rs`):
```rust
pub struct ProductService {
    repo: Arc<ProductRepository>,
}

impl ProductService {
    pub async fn create_product(&self, req: CreateProductRequest) -> Result<Product, ServiceError> {
        // Business logic here
    }
}
```

5. **Implement handlers** (`handler.rs`):
```rust
pub async fn create_product(
    service: web::Data<Arc<ProductService>>,
    req: web::Json<CreateProductRequest>,
    auth: AuthUser,
) -> Result<HttpResponse, ApiError> {
    req.validate()?;
    let product = service.create_product(req.into_inner()).await?;
    Ok(HttpResponse::Created().json(ApiResponse::success(product)))
}
```

6. **Define routes** (`routes.rs`):
```rust
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/products")
            .route("", web::post().to(handler::create_product))
            .route("", web::get().to(handler::list_products))
    );
}
```

7. **Register in main.rs**:
```rust
.configure(modules::product::routes::configure)
```

## 🔒 Security Best Practices

1. **Environment Variables**: Never commit `.env` file
2. **JWT Secret**: Use strong, random secret in production
3. **Password Hashing**: Uses bcrypt with configurable cost
4. **Input Validation**: All requests validated before processing
5. **SQL Injection**: MongoDB uses BSON, no SQL injection risk
6. **Rate Limiting**: Implement using actix-governor (TODO)

## 📊 Database Indexes

### Required Indexes

```javascript
// Users collection
db.users.createIndex({ "email": 1 }, { unique: true })
db.users.createIndex({ "created_at": -1 })

// Refresh tokens collection
db.refresh_tokens.createIndex({ "token": 1 }, { unique: true })
db.refresh_tokens.createIndex({ "user_id": 1 })
db.refresh_tokens.createIndex({ "expires_at": 1 })

// Wallets collection
db.wallets.createIndex({ "user_id": 1 }, { unique: true })
db.wallets.createIndex({ "status": 1 })
```

## 🐛 Debugging

### Enable debug logging

```bash
RUST_LOG=debug cargo run
```

### JSON logging (for production)

```bash
LOG_FORMAT=json cargo run
```

### Common issues

1. **MongoDB connection failed**: Check `MONGODB_URI` in `.env`
2. **Redis connection failed**: Ensure Redis is running on port 6379
3. **JWT errors**: Verify `JWT_SECRET` is set correctly

## 📝 Code Style

This project follows Rust standard conventions:

- Run `cargo fmt` before committing
- Run `cargo clippy` to check for common mistakes
- Follow the coding rules in `docs/CODING_STANDARDS.md`

## 🚀 Deployment

### Docker (Recommended)

```dockerfile
# Dockerfile example
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/mmo-api /usr/local/bin/
CMD ["mmo-api"]
```

### Systemd Service

```ini
[Unit]
Description=MMO API Server
After=network.target

[Service]
Type=simple
User=mmo-api
WorkingDirectory=/opt/mmo-api
Environment="RUST_LOG=info"
EnvironmentFile=/opt/mmo-api/.env
ExecStart=/opt/mmo-api/mmo-api
Restart=always

[Install]
WantedBy=multi-user.target
```

## 📖 Additional Documentation

- [Architecture Guide](docs/ARCHITECTURE.md)
- [Coding Standards](docs/CODING_STANDARDS.md)
- [Wallet V2 Design](../../docs/v2/01-wallet-system-design.md)

## 📄 License

[Your License Here]

## 👥 Contributors

[Your Team]

## 📞 Support

For issues and questions, please open an issue on GitHub.
