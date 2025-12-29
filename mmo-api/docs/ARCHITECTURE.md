# MMO API Architecture

## Overview

This document describes the architecture of the MMO API server, built with Rust, actix-web, and MongoDB.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                         Client                               │
└────────────────────────┬────────────────────────────────────┘
                         │ HTTP/HTTPS
                         ├─────────────────────────┐
                         │                          │
                         ▼                          ▼
┌────────────────────────────────────┐   ┌──────────────────┐
│          Middleware Layer           │   │   Health Check   │
├─────────────────────────────────────┤   └──────────────────┘
│  - Request ID                       │
│  - Logging (Tracing)                │
│  - CORS                             │
│  - Authentication (JWT)             │
│  - Authorization (Roles)            │
└────────────────┬───────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────┐
│              Handler Layer                  │
├─────────────────────────────────────────────┤
│  - Parse Request                            │
│  - Validate Input                           │
│  - Call Service                             │
│  - Format Response                          │
└────────────────┬────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────┐
│             Service Layer                   │
├─────────────────────────────────────────────┤
│  - Business Logic                           │
│  - Validation (Business Rules)              │
│  - Transaction Management                   │
│  - Error Handling                           │
└────────────────┬────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────┐
│           Repository Layer                  │
├─────────────────────────────────────────────┤
│  - Database Operations (CRUD)               │
│  - Query Building                           │
│  - Data Mapping                             │
└────────────────┬────────────────────────────┘
                 │
         ┌───────┴────────┐
         ▼                 ▼
┌─────────────┐   ┌──────────────┐
│   MongoDB   │   │    Redis     │
└─────────────┘   └──────────────┘
```

## Layered Architecture

### 1. Handler Layer

**Responsibility**: HTTP request/response handling

**Files**: `modules/*/handler.rs`

**Rules**:
- Parse and validate request DTOs
- Extract authentication info
- Call service layer
- Format responses using `ApiResponse`
- NO business logic
- NO direct database access

**Example**:
```rust
pub async fn create_wallet(
    service: web::Data<Arc<WalletService>>,
    req: web::Json<CreateWalletRequest>,
    auth: AuthUser,
) -> Result<HttpResponse, ApiError> {
    req.validate()?;  // Input validation
    let wallet = service.create_wallet(req.into_inner()).await?;
    Ok(HttpResponse::Created().json(ApiResponse::success(wallet)))
}
```

### 2. Service Layer

**Responsibility**: Business logic and orchestration

**Files**: `modules/*/service.rs`

**Rules**:
- Implement business logic
- Validate business rules
- Manage transactions
- Coordinate multiple repositories
- Error handling and logging
- NO HTTP-specific code
- NO direct database queries

**Example**:
```rust
pub async fn transfer_ap(&self, params: TransferParams) -> Result<(), ServiceError> {
    // Business validation
    self.validate_transfer(&params).await?;

    // Start transaction
    let session = self.db.start_session().await?;

    // Execute transfer
    self.repo.deduct_balance(&params.from, params.amount, &session).await?;
    self.repo.add_balance(&params.to, params.amount, &session).await?;

    // Commit
    session.commit().await?;
    Ok(())
}
```

### 3. Repository Layer

**Responsibility**: Database operations

**Files**: `modules/*/repository.rs`

**Rules**:
- Pure database CRUD operations
- Query building
- Return domain models
- Support transactions (sessions)
- NO business logic
- NO validation

**Example**:
```rust
pub async fn find_by_id(&self, id: &ObjectId) -> Result<Option<User>, DbError> {
    self.collection
        .find_one(doc! { "_id": id }, None)
        .await
        .map_err(DbError::from)
}
```

### 4. Domain Layer

**Responsibility**: Domain models and business entities

**Files**: `modules/*/domain.rs`

**Rules**:
- Define MongoDB document structures
- Use `serde` for serialization
- No logic (pure data structures)
- Domain-specific helper methods allowed

### 5. DTO Layer

**Responsibility**: Data transfer objects

**Files**: `modules/*/dto.rs`

**Rules**:
- Request/Response structures
- Validation annotations
- Type conversions (From/Into)
- Separate from domain models

## Data Flow

### Request Flow

```
1. HTTP Request
   ↓
2. Middleware (Auth, Logging, CORS)
   ↓
3. Handler (Parse, Validate)
   ↓
4. Service (Business Logic)
   ↓
5. Repository (Database)
   ↓
6. MongoDB/Redis
```

### Response Flow

```
1. Database Result
   ↓
2. Repository (Domain Model)
   ↓
3. Service (Business Processing)
   ↓
4. Handler (Convert to DTO)
   ↓
5. ApiResponse Wrapper
   ↓
6. HTTP Response (JSON)
```

## Module Structure

Each feature module follows this structure:

```
modules/feature_name/
├── mod.rs          # Module exports
├── domain.rs       # MongoDB models
├── dto.rs          # Request/Response DTOs
├── handler.rs      # HTTP handlers
├── service.rs      # Business logic
├── repository.rs   # Database operations
└── routes.rs       # Route configuration
```

### Module Communication

```
┌──────────────┐
│  Auth Module │
└──────┬───────┘
       │ depends on
       ▼
┌──────────────┐
│ Wallet Module│
└──────┬───────┘
       │ depends on
       ▼
┌──────────────┐
│ Order Module │
└──────────────┘
```

**Rules**:
- Modules can depend on other modules via services
- Use dependency injection
- Avoid circular dependencies

## Error Handling

### Error Types Hierarchy

```
ApiError (HTTP layer)
    ↑
    | converts from
    |
ServiceError (Business layer)
    ↑
    | converts from
    |
DbError (Database layer)
```

### Error Flow

```rust
// Repository returns DbError
Err(DbError::MongoError(...))
    ↓
// Service converts to ServiceError
Err(ServiceError::DatabaseError(...))
    ↓
// Handler converts to ApiError
Err(ApiError::InternalError(...))
    ↓
// ApiError implements ResponseError
HttpResponse::InternalServerError().json(...)
```

## Authentication & Authorization

### JWT Flow

```
1. Login Request
   ↓
2. Service validates credentials
   ↓
3. Generate Access Token (15m) + Refresh Token (7d)
   ↓
4. Store Refresh Token in MongoDB
   ↓
5. Return both tokens to client
```

### Token Refresh Flow

```
1. POST /api/auth/refresh with refresh_token
   ↓
2. Verify refresh token (JWT + DB check)
   ↓
3. Generate new Access Token + Refresh Token
   ↓
4. Revoke old refresh token
   ↓
5. Store new refresh token
   ↓
6. Return new tokens
```

### Protected Route Flow

```
1. Request with Authorization: Bearer <token>
   ↓
2. AuthMiddleware extracts token
   ↓
3. Verify JWT signature & expiration
   ↓
4. Add AuthUser to request extensions
   ↓
5. Handler extracts AuthUser
   ↓
6. Process request
```

## Database Design

### MongoDB Collections

- `users` - User accounts
- `refresh_tokens` - JWT refresh tokens
- `wallets` - User wallets
- `wallet_transactions` - Transaction ledger
- `escrow_holds` - Escrow management
- `withdrawal_requests` - Withdrawal requests
- `deposit_requests` - Deposit requests

### Transaction Pattern

```rust
// Start session
let session = db.start_session().await?;

// Execute operations within transaction
session.with_transaction(async {
    repo1.operation1(&session).await?;
    repo2.operation2(&session).await?;
    Ok(())
}).await?;

// Auto-commit on success, auto-rollback on error
```

## Dependency Injection

### Main Setup

```rust
// Initialize dependencies in main.rs
let db = MongoDB::connect(&config).await?;
let repo = Arc::new(Repository::new(db));
let service = Arc::new(Service::new(repo));

// Inject into handlers
App::new()
    .app_data(web::Data::from(service.clone()))
```

### Handler Usage

```rust
pub async fn handler(
    service: web::Data<Arc<Service>>,  // Auto-injected
) -> Result<HttpResponse, ApiError> {
    service.method().await?;
    // ...
}
```

## Configuration Management

### Environment-Based Config

```
.env (development)
  ↓
AppConfig::from_env()
  ↓
Arc<AppConfig> shared across app
```

### Config Access

```rust
// In service
impl Service {
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self { config }
    }

    fn use_config(&self) {
        let timeout = self.config.database.timeout;
    }
}
```

## Logging Strategy

### Structured Logging

```rust
tracing::info!(
    user_id = %user.id,
    email = %user.email,
    "User registered successfully"
);
```

### Log Levels

- `error` - Critical errors requiring immediate attention
- `warn` - Warning conditions
- `info` - Informational messages (business events)
- `debug` - Debug information
- `trace` - Very detailed tracing

### Request Tracing

```
Request ID: 550e8400-e29b-41d4-a716-446655440000
  ↓
All logs for this request include request_id
  ↓
Easy to trace entire request flow
```

## Performance Considerations

### Connection Pooling

- **MongoDB**: Min 10, Max 100 connections
- **Redis**: Pool size 10

### Caching Strategy

- Use Redis for:
  - Session storage
  - Frequently accessed data
  - Rate limiting counters

### Query Optimization

- Create indexes for frequently queried fields
- Use projection to limit returned fields
- Pagination for list endpoints

## Security

### Authentication

- JWT with short-lived access tokens (15m)
- Long-lived refresh tokens (7d) stored in DB
- Token revocation support

### Authorization

- Role-based access control (RBAC)
- Admin, User, Seller roles
- Middleware-based enforcement

### Input Validation

- All requests validated using `validator` crate
- Custom validators for business rules
- Type-safe parsing

### Password Security

- bcrypt hashing (cost: 12)
- Minimum 8 characters
- Complexity requirements

## Scalability

### Horizontal Scaling

```
Load Balancer
    ↓
┌───────┬───────┬───────┐
│ API 1 │ API 2 │ API 3 │  ← Stateless instances
└───┬───┴───┬───┴───┬───┘
    └───────┼───────┘
            ↓
    ┌───────────────┐
    │   MongoDB     │  ← Shared database
    │   (Replica)   │
    └───────────────┘
```

### Considerations

- **Stateless design**: No in-memory sessions
- **Distributed locking**: Use Redis for coordination
- **Sequence generation**: Use MongoDB or distributed service

## Monitoring & Observability

### Metrics to Track

- Request rate
- Response time (p50, p95, p99)
- Error rate
- Database query time
- Cache hit rate

### Health Checks

- `/health` endpoint
- Database connectivity check
- Redis connectivity check

## Testing Strategy

### Unit Tests

- Test individual functions
- Mock dependencies
- Fast execution

### Integration Tests

- Test API endpoints
- Use test database
- Test authentication flow

### Test Organization

```
tests/
├── common/
│   └── mod.rs      # Test utilities
├── auth_tests.rs   # Auth integration tests
└── wallet_tests.rs # Wallet integration tests
```

## Deployment

### Build Process

```bash
cargo build --release
```

### Environment Variables

- Development: `.env` file
- Production: System environment or secrets manager

### Docker Deployment

```
Build → Docker Image → Container Registry → Deploy
```

## Future Enhancements

- [ ] Rate limiting middleware
- [ ] Background job processing
- [ ] Email notifications
- [ ] WebSocket support
- [ ] GraphQL API
- [ ] Metrics exporter (Prometheus)
- [ ] Distributed tracing (Jaeger)
