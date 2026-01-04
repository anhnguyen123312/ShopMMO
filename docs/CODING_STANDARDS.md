# Coding Standards & Best Practices

## Overview

This document defines the coding standards for the MMO API project. Following these rules ensures consistency, maintainability, and code quality.

## Table of Contents

1. [Module Structure Rules](#module-structure-rules)
2. [Naming Conventions](#naming-conventions)
3. [Layer Responsibilities](#layer-responsibilities)
4. [Error Handling](#error-handling)
5. [Documentation](#documentation)
6. [Testing](#testing)
7. [Security](#security)
8. [Performance](#performance)

---

## Module Structure Rules

### ✅ ALWAYS Follow This Structure

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

### ❌ NEVER

- Mix layers (e.g., database code in handlers)
- Skip layers (e.g., handler calling repository directly)
- Create additional files without good reason

---

## Naming Conventions

### Files

| Type | Convention | Example |
|------|-----------|---------|
| Module | `snake_case` | `auth`, `wallet`, `order_management` |
| File | `snake_case.rs` | `domain.rs`, `service.rs` |

### Code

| Type | Convention | Example |
|------|-----------|---------|
| Struct | `PascalCase` | `User`, `WalletService` |
| Enum | `PascalCase` | `UserRole`, `WalletStatus` |
| Function | `snake_case` | `create_user`, `find_by_id` |
| Variable | `snake_case` | `user_id`, `access_token` |
| Constant | `SCREAMING_SNAKE_CASE` | `MAX_PAGE_SIZE`, `DEFAULT_CURRENCY` |
| Trait | `PascalCase` | `Repository`, `Authenticator` |

### DTOs

| Type | Pattern | Example |
|------|---------|---------|
| Request | `{Action}{Resource}Request` | `CreateUserRequest`, `LoginRequest` |
| Response | `{Resource}Response` | `UserResponse`, `WalletBalanceResponse` |
| Internal | `{Purpose}Params` | `TransferParams`, `SearchCriteria` |

### Functions

| Type | Pattern | Example |
|------|---------|---------|
| Get single | `find_by_{field}` | `find_by_id`, `find_by_email` |
| Get multiple | `find_all`, `list_{resource}` | `find_all_users`, `list_transactions` |
| Create | `create`, `create_{resource}` | `create`, `create_wallet` |
| Update | `update`, `update_{field}` | `update`, `update_balance` |
| Delete | `delete`, `delete_{resource}` | `delete`, `delete_user` |
| Check existence | `exists`, `{field}_exists` | `exists`, `email_exists` |

---

## Layer Responsibilities

### Rule 1: Handler Layer

**✅ DO:**
```rust
pub async fn create_user(
    service: web::Data<Arc<UserService>>,
    req: web::Json<CreateUserRequest>,
    auth: AuthUser,
) -> Result<HttpResponse, ApiError> {
    // 1. Validate input
    req.validate()?;

    // 2. Call service
    let user = service.create_user(req.into_inner()).await?;

    // 3. Return response
    Ok(HttpResponse::Created().json(ApiResponse::success(user)))
}
```

**❌ DON'T:**
```rust
// ❌ NO business logic
if user.age < 18 {
    return Err(ApiError::bad_request("Too young"));
}

// ❌ NO database access
let user = db.collection.find_one(...).await?;

// ❌ NO password hashing
let hash = bcrypt::hash(password)?;
```

### Rule 2: Service Layer

**✅ DO:**
```rust
impl UserService {
    pub async fn create_user(&self, req: CreateUserRequest) -> Result<User, ServiceError> {
        // 1. Business validation
        if self.repo.email_exists(&req.email).await? {
            return Err(ServiceError::ValidationFailed("Email exists".to_string()));
        }

        // 2. Business logic
        let password_hash = hash_password(&req.password)?;
        let user = User::new(req.email, password_hash, req.name);

        // 3. Call repository
        let created = self.repo.create(user).await?;

        // 4. Additional operations
        self.send_welcome_email(&created).await?;

        Ok(created)
    }
}
```

**❌ DON'T:**
```rust
// ❌ NO HTTP-specific code
fn create_user(&self) -> HttpResponse { }

// ❌ NO direct database queries
db.collection.insert_one(...).await?;

// ❌ NO validation.validate() (that's handler's job)
req.validate()?;
```

### Rule 3: Repository Layer

**✅ DO:**
```rust
impl UserRepository {
    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, DbError> {
        self.collection
            .find_one(doc! { "email": email }, None)
            .await
            .map_err(DbError::from)
    }

    pub async fn update_with_session(
        &self,
        id: &ObjectId,
        update: Document,
        session: &ClientSession,
    ) -> Result<(), DbError> {
        self.collection
            .update_one_with_session(
                doc! { "_id": id },
                update,
                None,
                session,
            )
            .await?;
        Ok(())
    }
}
```

**❌ DON'T:**
```rust
// ❌ NO business logic
if user.balance < amount {
    return Err(DbError::InsufficientFunds);
}

// ❌ NO validation
if email.is_empty() {
    return Err(DbError::InvalidInput);
}

// ❌ NO HTTP responses
fn find_user(&self) -> HttpResponse { }
```

---

## Error Handling

### Rule 1: Error Type Per Layer

```rust
// Database Layer
pub enum DbError {
    MongoError(String),
    RedisError(String),
    // ...
}

// Service Layer
pub enum ServiceError {
    NotFound(String),
    ValidationFailed(String),
    DatabaseError(String),
    // ...
}

// API Layer
pub enum ApiError {
    NotFound { message: String },
    BadRequest { message: String },
    // ...
}
```

### Rule 2: Error Conversion

```rust
// DbError → ServiceError
impl From<DbError> for ServiceError {
    fn from(err: DbError) -> Self {
        ServiceError::DatabaseError(err.to_string())
    }
}

// ServiceError → ApiError
impl From<ServiceError> for ApiError {
    fn from(err: ServiceError) -> Self {
        match err {
            ServiceError::NotFound(msg) => ApiError::not_found(msg),
            ServiceError::ValidationFailed(msg) => ApiError::bad_request(msg),
            // ...
        }
    }
}
```

### Rule 3: Error Logging

```rust
// ✅ Log errors before returning
pub async fn create_user(&self, req: CreateUserRequest) -> Result<User, ServiceError> {
    match self.repo.create(user).await {
        Ok(user) => Ok(user),
        Err(e) => {
            tracing::error!(
                error = %e,
                email = %req.email,
                "Failed to create user"
            );
            Err(ServiceError::from(e))
        }
    }
}
```

---

## Documentation

### Rule 1: Module-Level Documentation

```rust
//! Authentication module
//!
//! Handles user authentication, registration, and token management.
//!
//! # Features
//! - User registration with email/password
//! - JWT-based authentication
//! - Refresh token support
//! - Password change functionality
```

### Rule 2: Public Function Documentation

```rust
/// Creates a new user account
///
/// # Arguments
/// * `email` - User's email address (must be unique)
/// * `password` - Plain text password (will be hashed)
/// * `name` - User's display name
///
/// # Returns
/// * `Result<User, ServiceError>` - Created user or error
///
/// # Errors
/// * `ServiceError::ValidationFailed` - If email already exists
/// * `ServiceError::InternalError` - If password hashing fails
///
/// # Examples
/// ```
/// let user = service.create_user(
///     "user@example.com",
///     "Password123",
///     "John Doe"
/// ).await?;
/// ```
pub async fn create_user(
    &self,
    email: String,
    password: String,
    name: String,
) -> Result<User, ServiceError> {
    // ...
}
```

### Rule 3: Complex Logic Comments

```rust
// Calculate escrow hold period based on order type and amount
// - Base hold: 3 days for digital goods
// - +2 days if amount > 1000 AP
// - +4 days if seller reputation < 4.0
let hold_days = match order.order_type {
    OrderType::Digital => {
        let mut days = 3;
        if order.amount > 1000 { days += 2; }
        if seller.reputation < 4.0 { days += 4; }
        days
    }
    // ...
};
```

### Rule 4: TODO Comments

```rust
// TODO: Add email verification before allowing login
// TODO: Implement rate limiting for login attempts
// TODO(username): Optimize this query with proper index
// TODO: Issue #123 - Fix edge case when user has multiple wallets
```

---

## Testing

### Rule 1: Test Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        // Arrange
        let email = "test@example.com";
        let password = "Password123";

        // Act
        let user = User::new(email.to_string(), password.to_string(), "Test".to_string());

        // Assert
        assert_eq!(user.email, email);
        assert!(user.is_active());
    }
}
```

### Rule 2: Test Naming

```rust
#[test]
fn test_{function}_{scenario}_{expected_result}() { }

// Examples:
#[test]
fn test_create_user_with_valid_data_succeeds() { }

#[test]
fn test_create_user_with_existing_email_fails() { }

#[test]
fn test_login_with_wrong_password_returns_unauthorized() { }
```

### Rule 3: Test Coverage

- ✅ Test happy path
- ✅ Test error cases
- ✅ Test edge cases
- ✅ Test validation failures
- ✅ Test business rule violations

---

## Security

### Rule 1: Input Validation

```rust
// ✅ Always validate input
#[derive(Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(email)]
    pub email: String,

    #[validate(length(min = 8))]
    pub password: String,

    #[validate(length(min = 2, max = 50))]
    pub name: String,
}

// ✅ Validate in handler
pub async fn create_user(req: web::Json<CreateUserRequest>) -> Result<HttpResponse, ApiError> {
    req.validate()?;  // ALWAYS validate
    // ...
}
```

### Rule 2: Password Handling

```rust
// ✅ DO
let hash = hash_password(&password, Some(12))?;  // Bcrypt with cost 12
user.password_hash = hash;

// ❌ NEVER
user.password = password;  // Never store plain text
log::info!("Password: {}", password);  // Never log passwords
```

### Rule 3: Authentication Required

```rust
// ✅ Require auth for protected endpoints
pub async fn transfer_money(
    service: web::Data<Arc<WalletService>>,
    auth: AuthUser,  // ← Required auth
) -> Result<HttpResponse, ApiError> {
    // User is authenticated
}

// ✅ Use role checking
pub async fn admin_action(auth: AuthUser) -> Result<HttpResponse, ApiError> {
    require_role!(auth, UserRole::Admin)?;
    // Only admin can proceed
}
```

### Rule 4: SQL Injection Prevention

```rust
// ✅ Use BSON (MongoDB is safe)
doc! { "email": email }  // Safe - BSON prevents injection

// ❌ Avoid string concatenation in queries (not applicable to MongoDB)
// This is more relevant for SQL databases
```

---

## Performance

### Rule 1: Database Queries

```rust
// ✅ Use indexes
db.users.createIndex({ "email": 1 }, { unique: true })

// ✅ Use projection to limit fields
let options = FindOptions::builder()
    .projection(doc! { "email": 1, "name": 1 })
    .build();

// ✅ Use pagination
let options = FindOptions::builder()
    .skip((page - 1) * page_size)
    .limit(page_size)
    .build();

// ❌ Avoid N+1 queries
for user in users {
    let wallet = find_wallet(user.id).await?;  // ❌ N queries
}

// ✅ Use aggregation or bulk operations
let wallets = find_wallets_by_user_ids(&user_ids).await?;  // 1 query
```

### Rule 2: Caching

```rust
// ✅ Cache frequently accessed data
pub async fn get_user(&self, id: &ObjectId) -> Result<User, ServiceError> {
    // Check cache first
    if let Some(cached) = self.redis.get(&format!("user:{}", id)).await? {
        return Ok(serde_json::from_str(&cached)?);
    }

    // Fetch from database
    let user = self.repo.find_by_id(id).await?;

    // Cache for 1 hour
    self.redis.set(
        &format!("user:{}", id),
        &serde_json::to_string(&user)?,
        Some(3600)
    ).await?;

    Ok(user)
}
```

### Rule 3: Connection Pooling

```rust
// ✅ Set appropriate pool sizes
MONGODB_MAX_POOL_SIZE=100
MONGODB_MIN_POOL_SIZE=10

// ✅ Reuse connections (automatic with actix-web)
```

---

## Code Review Checklist

Before committing code, verify:

- [ ] Follows layer architecture (handler → service → repository)
- [ ] All public functions have documentation
- [ ] Input validation implemented
- [ ] Error handling with proper error types
- [ ] Logging for important operations
- [ ] Tests written (unit + integration)
- [ ] No hardcoded values (use config)
- [ ] No passwords/secrets in code
- [ ] Naming follows conventions
- [ ] No unused imports/variables
- [ ] `cargo fmt` applied
- [ ] `cargo clippy` passes with no warnings

---

## Common Mistakes to Avoid

### ❌ Mistake 1: Skipping Layers

```rust
// ❌ Handler calling repository directly
pub async fn get_user(repo: web::Data<UserRepository>) {
    let user = repo.find_by_id(&id).await?;  // WRONG
}

// ✅ Correct
pub async fn get_user(service: web::Data<UserService>) {
    let user = service.get_user(&id).await?;  // RIGHT
}
```

### ❌ Mistake 2: Business Logic in Handlers

```rust
// ❌ Handler with business logic
pub async fn transfer(req: web::Json<TransferRequest>) {
    if wallet.balance < req.amount {  // WRONG - business logic
        return Err(ApiError::bad_request("Insufficient balance"));
    }
}

// ✅ Correct - business logic in service
pub async fn transfer(&self, req: TransferRequest) -> Result<(), ServiceError> {
    if wallet.balance < req.amount {  // RIGHT - in service
        return Err(ServiceError::InsufficientBalance);
    }
}
```

### ❌ Mistake 3: Missing Validation

```rust
// ❌ No validation
pub async fn create_user(req: web::Json<CreateUserRequest>) {
    let user = service.create_user(req.into_inner()).await?;  // WRONG
}

// ✅ Correct
pub async fn create_user(req: web::Json<CreateUserRequest>) {
    req.validate()?;  // RIGHT - always validate
    let user = service.create_user(req.into_inner()).await?;
}
```

---

## Best Practices Summary

1. **Follow the architecture** - Never skip layers
2. **Validate everything** - All user input must be validated
3. **Document thoroughly** - Public APIs need documentation
4. **Handle errors properly** - Use appropriate error types
5. **Test comprehensively** - Cover happy path and errors
6. **Log important events** - Use structured logging
7. **Keep it simple** - Don't over-engineer
8. **Be consistent** - Follow naming conventions
9. **Secure by default** - Never trust user input
10. **Performance matters** - Use indexes, caching, and pagination

---

## Questions?

If you're unsure about any coding standard, ask the team or refer to existing code in the `auth` module as a reference implementation.
