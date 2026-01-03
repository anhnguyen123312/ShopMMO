---
name: rust-api-standards
description: Use when coding Rust REST APIs with actix-web, utoipa, MongoDB - implementing handlers, services, repositories, DTOs, OpenAPI docs, JWT auth, RBAC, middleware, or error handling
---

# Rust API Standards

## Overview

Production patterns for **Actix-web 4.9** + **MongoDB 3.1** + **utoipa 5.4**. Layered architecture: **Handler → Service → Repository**. 3 roles: **Admin, Vendor, Buyer**.

**Core principles:**
- Define shared types ONCE in `common/`
- ALL public APIs MUST have `#[utoipa::path]`
- Error chain: `DbError → ServiceError → ApiError`
- User URLs: `/api/{module}/*`
- Admin URLs: `/admin/api/{module}/*`
- **ALWAYS add `#[protect()]` with permissions to handlers**
- **Define permissions in `common/permissions.rs` first**

## Project Structure

```
src/
├── common/                     # SHARED - define once, use everywhere
│   ├── status.rs               # ALL status enums with ToSchema
│   ├── errors.rs               # ApiError, ServiceError, DbError
│   ├── responses.rs            # ApiResponse<T>, ListResponse<T>
│   └── auth.rs                 # Role enum, AuthUser, JwtClaims
│
├── modules/{feature}/
│   ├── domain.rs               # MongoDB models
│   ├── dto.rs                  # Request/Response DTOs with ToSchema
│   ├── service.rs              # Business logic
│   ├── repository.rs           # DB operations
│   └── api/
│       ├── admin/              # /admin/api/{feature}/*
│       │   └── routes.rs
│       └── user/               # /api/{feature}/*
│           └── routes.rs
│
└── middleware/
    ├── jwt.rs                  # JWT validation
    └── admin_guard.rs          # Admin-only middleware
```

## 1. Common Types (Define Once)

### 1.1 Status Enums

```rust
// common/status.rs
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum UserStatus { Active, Inactive, Suspended, PendingVerification }

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum WalletStatus { Active, Frozen, Closed }

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus { Pending, Paid, Delivered, Disputed, Completed, Refunded, Cancelled }

// Add more as needed...
```

### 1.2 Role Enum (3 Roles Only)

```rust
// common/auth.rs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Role { Admin, Vendor, Buyer }

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Admin => "admin",
            Role::Vendor => "vendor",
            Role::Buyer => "buyer",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: ObjectId,
    pub email: String,
    pub role: Role,
}

impl AuthUser {
    pub fn is_admin(&self) -> bool { self.role == Role::Admin }
    pub fn is_vendor(&self) -> bool { self.role == Role::Vendor }
}
```

### 1.3 Error Chain

```rust
// common/errors.rs
use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use thiserror::Error;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    #[schema(example = "VALIDATION_ERROR")]
    pub code: String,
    #[schema(example = "Invalid request")]
    pub message: String,
}

#[derive(Debug, Error)]
pub enum DbError {
    #[error("MongoDB error: {0}")]
    Mongo(#[from] mongodb::error::Error),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Duplicate: {0}")]
    Duplicate(String),
}

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("DB error: {0}")]
    Database(#[from] DbError),
    #[error("Validation: {0}")]
    Validation(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Insufficient balance")]
    InsufficientBalance,
}

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Service: {0}")]
    Service(#[from] ServiceError),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Unauthorized")]
    Unauthorized,
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(ErrorResponse {
            code: match self {
                ApiError::NotFound(_) => "NOT_FOUND",
                ApiError::BadRequest(_) => "BAD_REQUEST",
                ApiError::Unauthorized => "UNAUTHORIZED",
                _ => "INTERNAL_ERROR",
            }.to_string(),
            message: self.to_string(),
        })
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
```

### 1.4 Response Wrappers

```rust
// common/responses.rs
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: T,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ListResponse<T: Serialize> {
    pub success: bool,
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginationMeta {
    pub page: u32,
    pub per_page: u32,
    pub total: u64,
    pub total_pages: u32,
}
```

### 1.5 Permissions (REQUIRED)

**Rule:** ALWAYS define permissions in `common/permissions.rs` BEFORE using them in handlers.

```rust
// common/permissions.rs
use std::fmt;

/// Permission format: resource:action
#[derive(Debug, Clone, PartialEq)]
pub struct Permission {
    pub resource: String,
    pub action: String,
}

impl Permission {
    pub fn new(resource: &str, action: &str) -> Self {
        Self {
            resource: resource.to_string(),
            action: action.to_string(),
        }
    }

    pub fn as_str(&self) -> String {
        format!("{}:{}", self.resource, self.action)
    }
}

// Standard permissions (add module-specific ones here)
pub mod perms {
    use super::Permission;

    // User module
    pub const USERS_READ: &str = "users:read";
    pub const USERS_UPDATE: &str = "users:update";
    pub const USERS_DELETE: &str = "users:delete";
    pub const USERS_MANAGE: &str = "users:manage";

    // Wallet module
    pub const WALLETS_READ: &str = "wallets:read";
    pub const WALLETS_UPDATE: &str = "wallets:update";
    pub const WALLETS_MANAGE: &str = "wallets:manage";

    // Product module
    pub const PRODUCTS_CREATE: &str = "products:create";
    pub const PRODUCTS_READ: &str = "products:read";
    pub const PRODUCTS_UPDATE: &str = "products:update";
    pub const PRODUCTS_DELETE: &str = "products:delete";
    pub const PRODUCTS_MANAGE: &str = "products:manage";

    // Order module
    pub const ORDERS_CREATE: &str = "orders:create";
    pub const ORDERS_READ: &str = "orders:read";
    pub const ORDERS_UPDATE: &str = "orders:update";
    pub const ORDERS_MANAGE: &str = "orders:manage";

    // Shop module
    pub const SHOPS_CREATE: &str = "shops:create";
    pub const SHOPS_READ: &str = "shops:read";
    pub const SHOPS_UPDATE: &str = "shops:update";
    pub const SHOPS_DELETE: &str = "shops:delete";
    pub const SHOPS_MANAGE: &str = "shops:manage";

    // Dispute module
    pub const DISPUTES_CREATE: &str = "disputes:create";
    pub const DISPUTES_READ: &str = "disputes:read";
    pub const DISPUTES_RESPOND: &str = "disputes:respond";
    pub const DISPUTES_RESOLVE: &str = "disputes:resolve";
    pub const DISPUTES_MANAGE: &str = "disputes:manage";
}

// Role permissions mapping
pub fn role_permissions(role: &str) -> Vec<&'static str> {
    match role {
        "admin" => vec![
            perms::USERS_MANAGE,
            perms::WALLETS_MANAGE,
            perms::PRODUCTS_MANAGE,
            perms::ORDERS_MANAGE,
            perms::SHOPS_MANAGE,
            perms::DISPUTES_MANAGE,
        ],
        "vendor" => vec![
            perms::PRODUCTS_CREATE,
            perms::PRODUCTS_READ,
            perms::PRODUCTS_UPDATE,
            perms::ORDERS_READ,
            perms::SHOPS_READ,
            perms::SHOPS_UPDATE,
            perms::DISPUTES_RESPOND,
            perms::WALLETS_READ,
        ],
        "buyer" => vec![
            perms::PRODUCTS_READ,
            perms::ORDERS_CREATE,
            perms::ORDERS_READ,
            perms::SHOPS_READ,
            perms::DISPUTES_CREATE,
            perms::WALLETS_READ,
        ],
        _ => vec![],
    }
}
```

**Permission naming convention:**

| Resource | Actions | Permission Format |
|----------|---------|-------------------|
| users | create, read, update, delete, manage | `users:create`, `users:manage` |
| wallets | read, update, manage | `wallets:read` |
| products | create, read, update, delete, manage | `products:create` |
| orders | create, read, update, manage | `orders:create` |
| shops | create, read, update, delete, manage | `shops:update` |
| disputes | create, read, respond, resolve, manage | `disputes:respond` |

**Standard actions:**
- `create` - Create new resources
- `read` - View resources
- `update` - Edit own resources
- `delete` - Delete resources
- `manage` - Full control (including others' resources)
- `respond` - Special action (e.g., dispute response)
- `resolve` - Admin resolution actions
- `approve` - Approval actions
- `export` - Export data

## 2. Module Pattern

### 2.1 Domain Model

```rust
// modules/wallet/domain.rs
use serde::{Deserialize, Serialize};
use mongodb::bson::{oid::ObjectId, DateTime};
use crate::common::status::WalletStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    #[serde(rename = "_id")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    pub balance: i64,
    pub status: WalletStatus,
    pub created_at: DateTime,
}
```

### 2.2 DTOs with ToSchema

```rust
// modules/wallet/dto.rs
use serde::{Deserialize, Serialize};
use validator::Validate;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateWalletRequest {
    #[validate(length(min = 1))]
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WalletResponse {
    pub id: String,
    pub user_id: String,
    pub balance: i64,
    pub status: String,
}

impl From<Wallet> for WalletResponse {
    fn from(w: Wallet) -> Self {
        Self {
            id: w.id.map(|i| i.to_hex()).unwrap_or_default(),
            user_id: w.user_id.to_hex(),
            balance: w.balance,
            status: format!("{:?}", w.status).to_lowercase(),
        }
    }
}
```

### 2.3 Repository

```rust
// modules/wallet/repository.rs
use mongodb::{Collection, Database, bson::{doc, oid::ObjectId}};
use crate::modules::wallet::domain::Wallet;
use crate::common::errors::DbError;

pub struct WalletRepository {
    collection: Collection<Wallet>,
}

impl WalletRepository {
    pub fn new(db: &Database) -> Self {
        Self { collection: db.collection("wallets") }
    }

    pub async fn find_by_user(&self, user_id: &ObjectId) -> Result<Option<Wallet>, DbError> {
        self.collection
            .find_one(doc! { "user_id": user_id }, None)
            .await
            .map_err(DbError::from)
    }

    pub async fn insert(&self, wallet: Wallet) -> Result<Wallet, DbError> {
        self.collection
            .insert_one(wallet, None)
            .await
            .map(|_| wallet)
            .map_err(DbError::from)
    }
}
```

### 2.4 Service

```rust
// modules/wallet/service.rs
use mongodb::bson::oid::ObjectId;
use crate::modules::wallet::{repository::WalletRepository, domain::Wallet};
use crate::common::errors::ServiceError;

pub struct WalletService {
    repo: WalletRepository,
}

impl WalletService {
    pub fn new(repo: WalletRepository) -> Self {
        Self { repo }
    }

    pub async fn create_wallet(&self, user_id: ObjectId) -> Result<Wallet, ServiceError> {
        if self.repo.find_by_user(&user_id).await?.is_some() {
            return Err(ServiceError::Validation("Wallet exists".into()));
        }

        let wallet = Wallet {
            id: None,
            user_id,
            balance: 0,
            status: crate::common::status::WalletStatus::Active,
            created_at: mongodb::bson::DateTime::now(),
        };

        self.repo.insert(wallet).await.map_err(ServiceError::from)
    }
}
```

### 2.5 Handler with OpenAPI + Permissions (REQUIRED)

```rust
// modules/wallet/api/user/handlers.rs
use actix_web::{web, HttpResponse, Responder};
use actix_web_grants::protect;
use crate::common::{errors::ApiResult, responses::ApiResponse, auth::AuthUser, permissions::perms};
use crate::modules::wallet::{dto::{CreateWalletRequest, WalletResponse}, service::WalletService};

/// Create wallet
#[utoipa::path(
    post,
    path = "/api/wallet",
    tag = "wallet",
    request_body = CreateWalletRequest,
    responses(
        (status = 201, description = "Wallet created", body = ApiResponse<WalletResponse>),
        (status = 400, description = "Validation error", body = crate::common::errors::ErrorResponse),
        (status = 401, description = "Unauthorized", body = crate::common::errors::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::common::errors::ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
#[protect(perms::WALLETS_UPDATE)]
pub async fn create_wallet(
    service: web::Data<WalletService>,
    auth_user: AuthUser,
) -> ApiResult<impl Responder> {
    let wallet = service.create_wallet(auth_user.id).await?;
    Ok(HttpResponse::Created().json(ApiResponse::new(WalletResponse::from(wallet))))
}

/// Get balance
#[utoipa::path(
    get,
    path = "/api/wallet/balance",
    tag = "wallet",
    responses(
        (status = 200, description = "Success", body = ApiResponse<i64>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
#[protect(perms::WALLETS_READ)]
pub async fn get_balance(
    service: web::Data<WalletService>,
    auth_user: AuthUser,
) -> ApiResult<impl Responder> {
    let balance = service.get_balance(auth_user.id).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::new(balance)))
}

/// Admin: Get user wallet
#[utoipa::path(
    get,
    path = "/admin/api/wallet/users/{user_id}",
    tag = "wallet",
    params(
        ("user_id" = String, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "Success", body = ApiResponse<WalletResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    ),
    security(("bearer_auth" = []))
)]
#[protect(perms::WALLETS_MANAGE)]
pub async fn get_user_wallet(
    service: web::Data<WalletService>,
    path: web::Path<String>,
) -> ApiResult<impl Responder> {
    let user_id = path.into_inner();
    let wallet = service.get_wallet(&user_id).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::new(WalletResponse::from(wallet))))
}
```

**Permission patterns:**

```rust
// Single permission
#[protect("products:read")]

// Multiple permissions (OR logic - user needs ONE of these)
#[protect(any("products:create", "products:manage"))]

// Role-based (admin only)
#[protect("admin")]

// Ownership check pattern
#[protect("products:update")]
pub async fn update_product(
    auth_user: AuthUser,
    product_id: web::Path<String>,
) -> ApiResult<impl Responder> {
    // Service checks if user owns product OR is admin
    let product = service.get_product_for_update(&product_id, &auth_user).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::new(product)))
}
```

### 2.6 Routes

```rust
// modules/wallet/api/user/routes.rs
use actix_web::web;

pub fn configure_wallet_user_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/wallet")
            .route("", web::post().to(crate::modules::wallet::api::user::handlers::create_wallet))
            .route("/balance", web::get().to(crate::modules::wallet::api::user::handlers::get_balance))
    );
}

// modules/wallet/api/admin/routes.rs
pub fn configure_wallet_admin_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin/api/wallet")
            .wrap(crate::middleware::admin_guard::AdminGuard)
            .route("/users/{user_id}", web::get().to(handlers::get_user_wallet))
    );
}
```

## 3. Main Configuration

```rust
// main.rs
use actix_web::App;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Init MongoDB, Redis...

    HttpServer::new(move || {
        App::new()
            // User routes: /api/{module}/*
            .configure(crate::modules::wallet::api::user::routes::configure_wallet_user_routes)
            .configure(crate::modules::auth::api::user::routes::configure_auth_user_routes)

            // Admin routes: /admin/api/{module}/*
            .configure(crate::modules::wallet::api::admin::routes::configure_wallet_admin_routes)

            // Swagger UI
            .configure(swagger_ui_config)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
```

## Quick Reference

| Task | Pattern |
|------|---------|
| Status enum | Define in `common/status.rs`, import everywhere |
| Permissions | Define in `common/permissions.rs`, use `#[protect()]` |
| Errors | `DbError → ServiceError → ApiError` chain |
| Handler | `#[utoipa::path]` + `#[protect()]` + validate → service → `ApiResponse` |
| Repository | Pure MongoDB operations, return domain models |
| Service | Business logic, validation, coordinate repos |
| User routes | `/api/{module}/*` |
| Admin routes | `/admin/api/{module}/*` with guard |

## Common Mistakes

| ❌ Mistake | ✅ Fix |
|------------|-------|
| Inline status enum | Use from `common/status.rs` |
| Skip repository layer | Always: Handler → Service → Repository |
| Missing `#[utoipa::path]` | ALL public handlers need OpenAPI docs |
| Missing `#[protect()]` | ALL protected handlers need permissions |
| Wrong route structure | User: `/api/*`, Admin: `/admin/api/*` |
| Hardcode permissions | Define in `common/permissions.rs` first |

## Dependencies

```toml
[dependencies]
actix-web = "4.9"
actix-web-grants = "4"
mongodb = "3.1"
redis = "0.27"
utoipa = { version = "5.4", features = ["actix_extras"] }
utoipa-swagger-ui = "8"
validator = { version = "0.18", features = ["derive"] }
thiserror = "2.0"
jsonwebtoken = "9.3"
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
```
