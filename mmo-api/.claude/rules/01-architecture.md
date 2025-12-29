# Architecture Rules

## Layer Architecture (STRICT)

You MUST follow this layered architecture for ALL code:

```
Handler → Service → Repository → Database
```

### Rule 1: NEVER Skip Layers

❌ FORBIDDEN:
- Handler calling Repository directly
- Handler accessing Database directly
- Service accessing Database directly (must use Repository)

✅ REQUIRED:
- Handler ONLY calls Service
- Service ONLY calls Repository
- Repository ONLY accesses Database

### Rule 2: Layer Responsibilities

**Handler Layer** (`modules/*/handler.rs`):
- Parse HTTP requests
- Validate DTOs with `req.validate()?`
- Call Service methods
- Format responses with `ApiResponse`
- Extract `AuthUser` from middleware
- Return `Result<HttpResponse, ApiError>`

**NEVER** in handlers:
- Business logic
- Database queries
- Password hashing
- Transaction management

**Service Layer** (`modules/*/service.rs`):
- Business logic implementation
- Business rule validation
- Transaction management (MongoDB sessions)
- Coordinate multiple repositories
- Error handling and logging
- Return `Result<T, ServiceError>`

**NEVER** in services:
- HTTP-specific code (`HttpResponse`)
- DTO validation (`.validate()`)
- Direct database queries

**Repository Layer** (`modules/*/repository.rs`):
- Pure CRUD operations
- Query building with `doc!` macro
- Return domain models
- Support MongoDB sessions for transactions
- Return `Result<T, DbError>`

**NEVER** in repositories:
- Business logic
- Input validation
- HTTP responses

## Module Structure (STRICT)

Every feature module MUST have exactly these files:

```
modules/feature_name/
├── mod.rs          # Module exports only
├── domain.rs       # MongoDB models only
├── dto.rs          # Request/Response DTOs only
├── handler.rs      # HTTP handlers only
├── service.rs      # Business logic only
├── repository.rs   # Database operations only
└── routes.rs       # Route configuration only
```

### Rule 3: File Responsibilities

**domain.rs**:
- MongoDB document structures with `#[derive(Serialize, Deserialize)]`
- Domain-specific helper methods
- NO validation logic
- NO business logic

**dto.rs**:
- Request DTOs with `#[derive(Deserialize, Validate)]`
- Response DTOs with `#[derive(Serialize)]`
- Validation annotations using `validator` crate
- Conversion implementations (`From<Domain> for DTO`)

**handler.rs**:
- One function per endpoint
- Function signature: `pub async fn name(service, req, auth) -> Result<HttpResponse, ApiError>`
- ALWAYS validate: `req.validate()?`
- ALWAYS wrap response: `ApiResponse::success(data)`

**service.rs**:
- Struct with repository dependencies
- Public methods for business operations
- Private helper methods for complex logic
- ALWAYS use transactions for multi-step operations

**repository.rs**:
- Struct with MongoDB collection
- Methods: `find_by_*`, `create`, `update_*`, `delete`
- Support `session` parameter for transactions

**routes.rs**:
- Single `configure()` function
- Use `web::scope()` for grouping
- Attach handlers to routes

## Error Handling (STRICT)

### Rule 4: Error Type Hierarchy

```
DbError → ServiceError → ApiError
```

ALWAYS convert errors at layer boundaries:

```rust
// In repository: return DbError
Err(DbError::MongoError(err.to_string()))

// In service: convert DbError → ServiceError
.map_err(|e| ServiceError::DatabaseError(e.to_string()))?

// In handler: ServiceError auto-converts to ApiError via From trait
service.method().await?  // Auto-converts
```

### Rule 5: Error Logging

ALWAYS log errors in Service layer before returning:

```rust
tracing::error!(
    error = %e,
    context_field = value,
    "Operation failed"
);
```

## Dependency Injection

### Rule 6: Initialize in main.rs

```rust
// 1. Create repositories
let repo = Arc::new(Repository::new(db));

// 2. Create services with repository dependencies
let service = Arc::new(Service::new(repo));

// 3. Inject into app
App::new().app_data(web::Data::from(service))
```

### Rule 7: Handler Parameters

Order of parameters:

```rust
pub async fn handler(
    service: web::Data<Arc<Service>>,  // Required
    req: web::Json<Request>,           // If needed
    auth: AuthUser,                    // If protected
) -> Result<HttpResponse, ApiError>
```

## MongoDB Transactions

### Rule 8: Transaction Pattern

For operations modifying multiple documents:

```rust
let session = self.db.start_session().await?;

session.withTransaction(async {
    repo1.operation(&session).await?;
    repo2.operation(&session).await?;
    Ok(())
}).await?;
```

## Authentication & Authorization

### Rule 9: Protected Routes

All protected endpoints MUST:
1. Have `auth: AuthUser` parameter
2. Be wrapped in `AuthMiddleware` in routes

```rust
// In routes.rs
web::scope("")
    .wrap(AuthMiddleware::new(config))
    .route("/protected", web::get().to(handler))
```

### Rule 10: Role-Based Access

For admin-only endpoints:

```rust
// Method 1: Middleware
.wrap(RequireRole::admin())

// Method 2: In handler
require_role!(auth, UserRole::Admin)?;
```

## Validation

### Rule 11: Input Validation

ALWAYS validate in handlers:

```rust
req.validate()?;  // First line after function declaration
```

For custom validation, add to `core/validator.rs`.

## Logging

### Rule 12: Structured Logging

Use tracing with structured fields:

```rust
tracing::info!(
    user_id = %id,
    operation = "create",
    "User created successfully"
);
```

Log levels:
- `error!` - Critical errors
- `warn!` - Warnings
- `info!` - Business events
- `debug!` - Debug info

## Testing

### Rule 13: Test Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_scenario_result() {
        // Arrange
        // Act
        // Assert
    }
}
```

## Code Review Checklist

Before committing, verify:

- [ ] Follows layer architecture (no layer skipping)
- [ ] All public functions have doc comments
- [ ] Input validation implemented
- [ ] Errors logged in service layer
- [ ] Transactions used for multi-document operations
- [ ] `cargo fmt` and `cargo clippy` pass
- [ ] Tests written for new code
