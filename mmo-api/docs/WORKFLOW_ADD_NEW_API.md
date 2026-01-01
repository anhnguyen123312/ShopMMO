# Workflow: Thêm API Mới

> Mọi API mới đều phải có Swagger documentation và Permission control.

## Table of Contents

1. [Quick Checklist](#quick-checklist)
2. [Step-by-Step Workflow](#step-by-step-workflow)
3. [Example: Thêm Product API](#example-thêm-product-api)
4. [Common Mistakes](#common-mistakes)

---

## Quick Checklist

Khi thêm API mới, đảm bảo:

- [ ] **1. Define Permission** trong `src/modules/permissions/constants.rs`
- [ ] **2. Add Swagger** với `#[utoipa::path]` trong handler
- [ ] **3. Add DTOs** với `#[derive(ToSchema)]` trong dto.rs
- [ ] **4. Update OpenAPI** trong `src/openapi.rs`
- [ ] **5. Register Handler** trong routes.rs
- [ ] **6. Add Permission Guard** với `#[protect("permission:action")]`

---

## Step-by-Step Workflow

### Step 1: Define Permission trong Constants

File: `src/modules/permissions/constants.rs`

```rust
// Add permission enum variant
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    // ... existing permissions
    ProductCreate,  // ← NEW
    ProductRead,    // ← NEW
    ProductUpdate,  // ← NEW
    ProductDelete,  // ← NEW
    ProductList,    // ← NEW
}

impl Permission {
    pub fn as_str(&self) -> &'static str {
        match self {
            // ... existing
            Self::ProductCreate => "product:create",
            Self::ProductRead => "product:read",
            Self::ProductUpdate => "product:update",
            Self::ProductDelete => "product:delete",
            Self::ProductList => "product:list",
        }
    }
}

pub fn is_valid_permission(permission: &str) -> bool {
    matches!(permission,
        "product:create" | "product:read" | "product:update" |
        "product:delete" | "product:list"
        // ... existing permissions
    )
}
```

### Step 2: Create Request/Response DTOs với Swagger Schema

File: `src/modules/{feature}/dto.rs`

```rust
use serde::{Deserialize, Serialize};
use validator::Validate;
use utoipa::ToSchema;  // ← REQUIRED for Swagger

/// Request DTO
#[derive(Debug, Deserialize, Validate, ToSchema)]  // ← ToSchema is REQUIRED
#[serde(rename_all = "camelCase")]
pub struct CreateProductRequest {
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: String,

    #[validate(length(min = 1, message = "Description is required"))]
    pub description: String,

    #[validate(range(min = 1))]
    pub price: i64,
}

/// Response DTO
#[derive(Debug, Serialize, ToSchema)]  // ← ToSchema is REQUIRED
#[serde(rename_all = "camelCase")]
pub struct ProductResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub price: i64,
    pub created_at: String,
}
```

### Step 3: Create Handler với Swagger Documentation

File: `src/modules/{feature}/handler.rs`

```rust
use actix_web::{web, HttpResponse};
use actix_web_grants::protect;  // ← REQUIRED for permission guard
use std::sync::Arc;
use validator::Validate;

use super::service::ProductService;
use super::dto::*;
use crate::core::{ApiError, ApiResponse};

/// Create product - requires product:create permission
#[utoipa::path(
    post,
    path = "/api/products",
    tag = "Products",  // ← Swagger tag grouping
    security(
        ("bearer_auth" = [])  // ← Requires JWT
    ),
    request_body = CreateProductRequest,
    responses(
        (status = 201, description = "Product created", body = ApiResponse<ProductResponse>),
        (status = 400, description = "Validation error", body = ApiError),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Forbidden - missing product:create permission", body = ApiError)
    )
)]
#[protect("product:create")]  // ← REQUIRED: Permission guard
pub async fn create_product(
    service: web::Data<Arc<ProductService>>,
    req: web::Json<CreateProductRequest>,
) -> Result<HttpResponse, ApiError> {
    // 1. Validate input
    req.validate()?;

    // 2. Call service
    let product = service
        .create_product(req.into_inner())
        .await?;

    // 3. Return response
    Ok(HttpResponse::Created().json(ApiResponse::success(product)))
}

/// List products - requires product:list permission
#[utoipa::path(
    get,
    path = "/api/products",
    tag = "Products",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of products", body = ApiResponse<Vec<ProductResponse>>),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Forbidden - missing product:list permission", body = ApiError)
    )
)]
#[protect("product:list")]  // ← REQUIRED: Permission guard
pub async fn list_products(
    service: web::Data<Arc<ProductService>>,
) -> Result<HttpResponse, ApiError> {
    let products = service.list_products().await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(products)))
}

/// Get product by ID - requires product:read permission
#[utoipa::path(
    get,
    path = "/api/products/{id}",
    tag = "Products",
    security(("bearer_auth" = [])),
    params(
        ("id" = String, Path, description = "Product ID")
    ),
    responses(
        (status = 200, description = "Product details", body = ApiResponse<ProductResponse>),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Forbidden - missing product:read permission", body = ApiError),
        (status = 404, description = "Product not found", body = ApiError)
    )
)]
#[protect("product:read")]  // ← REQUIRED: Permission guard
pub async fn get_product(
    service: web::Data<Arc<ProductService>>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let product_id = path.into_inner();
    let product = service.get_product(&product_id).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(product)))
}
```

### Step 4: Update OpenAPI Documentation

File: `src/openapi.rs`

```rust
#[derive(OpenApi)]
#[openapi(
    info(
        title = "MMO API",
        version = "1.0.0",
        description = "Production-ready Rust API server with JWT authentication, MongoDB, and Redis",
        // ...
    ),
    tags(
        (name = "Auth", description = "Authentication endpoints"),
        (name = "Wallet", description = "Wallet management endpoints"),
        (name = "Permissions", description = "Permission and role management endpoints"),
        (name = "Products", description = "Product management endpoints"),  // ← NEW
    ),
    paths(
        // ... existing paths
        // Product endpoints  ← NEW
        crate::modules::products::handler::create_product,
        crate::modules::products::handler::list_products,
        crate::modules::products::handler::get_product,
    ),
    components(
        schemas(
            // ... existing schemas
            // Product DTOs  ← NEW
            crate::modules::products::dto::CreateProductRequest,
            crate::modules::products::dto::ProductResponse,
        ),
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;
```

### Step 5: Register Routes

File: `src/modules/{feature}/routes.rs`

```rust
use actix_web::web;

use super::handler::*;

/// Configure product routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/products")
            .service(
                web::resource("")
                    .route(web::post().to(create_product))
                    .route(web::get().to(list_products))
            )
            .route("/{id}", web::get().to(get_product))
    );
}
```

File: `src/main.rs`

```rust
.service(
    web::scope("")
        .wrap(middleware::AuthMiddleware::new(config.clone()))
        .configure(modules::wallet::routes::configure)
        .configure(modules::permissions::routes::configure)
        .configure(modules::products::routes::configure),  // ← NEW
)
```

### Step 6: Verify Swagger UI

```bash
# 1. Build
cargo build

# 2. Run server
MONGODB_URI="mongodb://..." cargo run --bin mmo-api

# 3. Open Swagger UI
# http://localhost:8080/swagger-ui/

# 4. Check:
#    - New "Products" tag appears
#    - All endpoints documented
#    - Request/Response schemas visible
#    - "Authorize" button works (JWT auth)
```

---

## Example: Thêm Product API

### 1. Define Permissions

```rust
// src/modules/permissions/constants.rs
const PERM_PRODUCT_CREATE: &str = "product:create";
const PERM_PRODUCT_READ: &str = "product:read";
const PERM_PRODUCT_UPDATE: &str = "product:update";
const PERM_PRODUCT_DELETE: &str = "product:delete";
const PERM_PRODUCT_LIST: &str = "product:list";
```

### 2. Create DTOs

```rust
// src/modules/products/dto.rs
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateProductRequest {
    #[validate(length(min = 1))]
    pub name: String,
    pub description: String,
    #[validate(range(min = 1))]
    pub price: i64,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProductResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub price: i64,
}
```

### 3. Create Handler

```rust
// src/modules/products/handler.rs
#[utoipa::path(
    post,
    path = "/api/products",
    tag = "Products",
    security(("bearer_auth" = [])),
    request_body = CreateProductRequest,
    responses(
        (status = 201, description = "Product created", body = ApiResponse<ProductResponse>),
        (status = 403, description = "Forbidden - missing product:create permission")
    )
)]
#[protect("product:create")]
pub async fn create_product(
    service: web::Data<Arc<ProductService>>,
    req: web::Json<CreateProductRequest>,
) -> Result<HttpResponse, ApiError> {
    req.validate()?;
    let product = service.create_product(req.into_inner()).await?;
    Ok(HttpResponse::Created().json(ApiResponse::success(product)))
}
```

### 4. Update OpenAPI

```rust
// src/openapi.rs
#[openapi(
    tags(
        (name = "Products", description = "Product management endpoints"),
    ),
    paths(
        crate::modules::products::handler::create_product,
    ),
    components(
        schemas(
            crate::modules::products::dto::CreateProductRequest,
            crate::modules::products::dto::ProductResponse,
        ),
    )
)]
```

### 5. Test API

```bash
# 1. Login to get token
TOKEN=$(curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"password"}' \
  | jq -r '.data.accessToken')

# 2. Create role with permission
curl -X POST http://localhost:8080/api/permissions/roles \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "PRODUCT_MANAGER",
    "displayName": "Product Manager",
    "level": 1,
    "permissions": ["product:create", "product:read", "product:list"]
  }'

# 3. Assign role to user
curl -X POST http://localhost:8080/api/permissions/roles/assign \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "userId": "<USER_ID>",
    "roleName": "PRODUCT_MANAGER"
  }'

# 4. Create product
curl -X POST http://localhost:8080/api/products \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Test Product",
    "description": "A test product",
    "price": 100
  }'
```

---

## Common Mistakes

### ❌ Mistake 1: Quên thêm `#[derive(ToSchema)]`

```rust
// ❌ WRONG
#[derive(Debug, Deserialize)]
pub struct CreateProductRequest {
    pub name: String,
}

// ✅ CORRECT
#[derive(Debug, Deserialize, ToSchema)]  // ← ToSchema is REQUIRED for Swagger
pub struct CreateProductRequest {
    pub name: String,
}
```

### ❌ Mistake 2: Quên `#[protect("permission")]`

```rust
// ❌ WRONG - No permission guard
#[utoipa::path(...)]
pub async fn create_product(...) -> Result<HttpResponse, ApiError> {
    // Anyone can access!
}

// ✅ CORRECT - Has permission guard
#[utoipa::path(...)]
#[protect("product:create")]  // ← REQUIRED
pub async fn create_product(...) -> Result<HttpResponse, ApiError> {
    // Only users with product:create can access
}
```

### ❌ Mistake 3: Quên update `openapi.rs`

```rust
// ❌ WRONG - Handler exists but not in OpenAPI
// Handler is defined but not added to paths()

// ✅ CORRECT - Handler in OpenAPI
#[openapi(
    paths(
        crate::modules::products::handler::create_product,  // ← ADD THIS
    ),
    components(
        schemas(
            crate::modules::products::dto::CreateProductRequest,  // ← AND THIS
        ),
    )
)]
```

### ❌ Mistake 4: Sai permission string format

```rust
// ❌ WRONG - Wrong format
#[protect("PRODUCT_CREATE")]  // PascalCase
#[protect("product-create")]  // dash instead of colon
#[protect("products_create")]  // plural resource

// ✅ CORRECT - resource:action format (lowercase)
#[protect("product:create")]
#[protect("product:read")]
#[protect("order:update")]
```

### ❌ Mistake 5: Quên validate input

```rust
// ❌ WRONG - No validation
pub async fn create_product(req: web::Json<CreateProductRequest>) {
    let product = service.create_product(req.into_inner()).await?;
}

// ✅ CORRECT - Always validate
pub async fn create_product(req: web::Json<CreateProductRequest>) {
    req.validate()?;  // ← REQUIRED
    let product = service.create_product(req.into_inner()).await?;
}
```

---

## Permission Naming Convention

### Format: `resource:action`

| Resource | Action | Permission String | Description |
|----------|--------|-------------------|-------------|
| `product` | `create` | `product:create` | Create new product |
| `product` | `read` | `product:read` | View product details |
| `product` | `update` | `product:update` | Update product |
| `product` | `delete` | `product:disable` | Delete product |
| `product` | `list` | `product:list` | List all products |
| `order` | `create` | `order:create` | Create order |
| `order` | `cancel` | `order:cancel` | Cancel order |
| `wallet` | `read` | `wallet:read` | View wallet balance |
| `wallet` | `withdraw` | `wallet:withdraw` | Withdraw money |
| `user` | `manage` | `user:manage` | Manage users (admin) |

### Rules:
1. Resource = **singular** noun (product, not products)
2. Action = **verb** (create, read, update, delete, list)
3. Separator = **colon** (`:`)
4. All = **lowercase**

---

## Testing Checklist

Before marking API as complete:

- [ ] Swagger UI shows endpoint with correct documentation
- [ ] Request body schema is correct
- [ ] Response schema is correct
- [ ] Permission guard blocks unauthorized access
- [ ] Authorization header is required
- [ ] Input validation works
- [ ] Error responses are documented
- [ ] Test with valid JWT + permission → 200/201
- [ ] Test with valid JWT but missing permission → 403
- [ ] Test without JWT → 401
- [ ] Test with invalid input → 400

---

## References

- [Swagger UI](http://localhost:8080/swagger-ui/) - API documentation
- [Permission Constants](../src/modules/permissions/constants.rs) - Define permissions here
- [Coding Standards](./CODING_STANDARDS.md) - General coding guidelines
- [Auth System V2](../docs/plans/2026-01-01-authorization-system-v2.md) - Authorization details
