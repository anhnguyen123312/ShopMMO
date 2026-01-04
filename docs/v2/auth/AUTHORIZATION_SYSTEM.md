# Authorization System V2 - Dynamic RBAC

## Overview

The MMO API uses a **dynamic Role-Based Access Control (RBAC)** system with:
- **Hardcoded permissions** in Rust code (type-safe enum constants)
- **Dynamic roles** stored in MongoDB (manageable via CRUD API)
- **Redis caching** for fast permission checks
- **JWT with roles array** for authentication

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            Request Flow                                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  1. Client Request                                                          │
│     │                                                                       │
│     ▼                                                                       │
│  2. AuthMiddleware (validates JWT)                                         │
│     │                                                                       │
│     ▼                                                                       │
│  3. GrantsMiddleware (extracts permissions from Redis/DB)                   │
│     │                                                                       │
│     ▼                                                                       │
│  4. Permission Guard #[protect("resource:action")]                         │
│     │                                                                       │
│     ▼                                                                       │
│  5. Handler (business logic)                                               │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Permission Format

**Format:** `resource:action` (lowercase, colon-separated)

### Examples

| Permission | Description |
|------------|-------------|
| `product:create` | Create new product |
| `product:read` | View product details |
| `product:update` | Update product |
| `product:delete` | Delete product |
| `product:list` | List all products |
| `order:create` | Create order |
| `order:cancel` | Cancel order |
| `wallet:read` | View wallet balance |
| `wallet:withdraw` | Withdraw from wallet |
| `role:assign` | Assign roles to users |

## Default Roles

### BUYER (Level 0)
```
Permissions:
  - product:list
  - product:read
  - order:create
  - order:read
  - order:list
  - wallet:read
  - wallet:deposit
```

### SELLER (Level 1)
```
Inherits: BUYER
Additional Permissions:
  - product:create
  - product:update
  - product:delete
  - order:update
  - wallet:withdraw
```

### ADMIN (Level 2)
```
Inherits: SELLER
Additional Permissions:
  - order:cancel
  - wallet:list
  - user:read
  - user:update
  - user:assign_roles
  - role:read
```

### SUPER_ADMIN (Level 3)
```
Permissions: ALL (*)
```

## File Structure

```
src/modules/permissions/
├── mod.rs           # Module exports
├── constants.rs     # Permission enum definitions (hardcoded)
├── domain.rs        # Role, Permission, UserPermissions models
├── dto.rs           # Request/Response DTOs for role management
├── repository.rs    # Database operations for roles/permissions
├── service.rs       # Business logic for role management
├── handler.rs       # HTTP handlers with #[protect()] guards
├── routes.rs        # Route configuration
└── cache.rs         # Redis caching for permissions
```

## API Endpoints

### Public Endpoints

#### List All Permissions
```http
GET /api/v1/permissions
Authorization: Bearer <token>
```

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "id": "...",
      "name": "product:create",
      "displayName": "Create Product",
      "description": "Create new product listings",
      "resource": "product",
      "action": "create",
      "category": "marketplace",
      "isActive": true
    }
  ]
}
```

### Protected Endpoints (Admin)

#### Create Role
```http
POST /api/permissions/roles
Authorization: Bearer <admin_token>
Content-Type: application/json

{
  "name": "CONTENT_MANAGER",
  "displayName": "Content Manager",
  "level": 1,
  "permissions": ["product:create", "product:update", "product:read"]
}
```

#### List Roles
```http
GET /api/permissions/roles
Authorization: Bearer <token>
```

#### Update Role Permissions
```http
PUT /api/permissions/roles/{role_name}/permissions
Authorization: Bearer <admin_token>
Content-Type: application/json

{
  "permissions": ["product:create", "product:update", "product:delete"]
}
```

#### Delete Role
```http
DELETE /api/permissions/roles/{role_name}
Authorization: Bearer <admin_token>
```

#### Assign Role to User
```http
POST /api/permissions/roles/assign
Authorization: Bearer <admin_token>
Content-Type: application/json

{
  "userId": "507f1f77bcf86cd799439011",
  "roleName": "SELLER"
}
```

## Usage in Handlers

### Step 1: Import the protect macro
```rust
use actix_web_grants::protect;
```

### Step 2: Add permission guard
```rust
/// Create product
#[utoipa::path(
    post,
    path = "/api/products",
    tag = "Products",
    security(("bearer_auth" = [])),
    request_body = CreateProductRequest,
    responses(
        (status = 201, description = "Product created"),
        (status = 403, description = "Forbidden - missing product:create permission")
    )
)]
#[protect("product:create")]  // ← Permission guard
pub async fn create_product(
    service: web::Data<Arc<ProductService>>,
    req: web::Json<CreateProductRequest>,
) -> Result<HttpResponse, ApiError> {
    req.validate()?;
    let product = service.create_product(req.into_inner()).await?;
    Ok(HttpResponse::Created().json(ApiResponse::success(product)))
}
```

### Step 3: Multiple permissions (any of)
```rust
#[protect(any("order:approve", "order:manage"))]
pub async fn approve_order(...) { }
```

### Step 4: Ownership check pattern
```rust
#[protect("product:update")]
pub async fn update_product(
    service: web::Data<Arc<ProductService>>,
    auth: AuthUser,  // Authenticated user
    path: web::Path<String>,
    req: web::Json<UpdateProductRequest>,
) -> Result<HttpResponse, ApiError> {
    let product_id = path.into_inner();

    // Get product
    let product = service.get_product(&product_id).await?;

    // Check ownership or admin
    if !auth.roles.contains(&"SUPER_ADMIN".to_string())
        && product.owner_id != auth.user_id {
        return Err(ApiError::forbidden("You don't own this product"));
    }

    // Update
    let updated = service.update_product(&product_id, req.into_inner()).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(updated)))
}
```

## JWT Structure

```json
{
  "sub": "507f1f77bcf86cd799439011",
  "roles": ["BUYER", "SELLER"],
  "permVersion": 1,
  "iat": 1704067200,
  "exp": 1735689600
}
```

- `sub`: User ID (ObjectId)
- `roles`: Array of role names
- `permVersion`: Permission version for cache invalidation

## Seeding Default Roles

```bash
# Run seed script
MONGODB_URI="mongodb://mmo_admin:mmo_secret_password@localhost:27017" \
  cargo run --bin seed_roles
```

This creates:
- BUYER (7 permissions)
- SELLER (12 permissions)
- ADMIN (18 permissions)
- SUPER_ADMIN (24 permissions)

## Testing

### 1. Create test role
```bash
curl -X POST http://localhost:8080/api/permissions/roles \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "TEST_ROLE",
    "displayName": "Test Role",
    "level": 1,
    "permissions": ["product:create", "product:read"]
  }'
```

### 2. Assign role to user
```bash
curl -X POST http://localhost:8080/api/permissions/roles/assign \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "userId": "'$USER_ID'",
    "roleName": "TEST_ROLE"
  }'
```

### 3. Test API access
```bash
# Should succeed (has permission)
curl -X POST http://localhost:8080/api/products \
  -H "Authorization: Bearer $USER_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name": "Test", "price": 100}'

# Should fail (no permission)
curl -X DELETE http://localhost:8080/api/products/123 \
  -H "Authorization: Bearer $USER_TOKEN"
```

## Common Patterns

### Admin-only endpoint
```rust
#[protect("user:manage")]
pub async fn delete_user(...) { }
```

### Owner or admin
```rust
#[protect("product:update")]
pub async fn update_product(...) {
    // Check if user is admin
    if auth.roles.contains(&"SUPER_ADMIN".to_string()) {
        // Allow
    } else if product.owner_id == auth.user_id {
        // Allow
    } else {
        return Err(ApiError::forbidden("Not authorized"));
    }
}
```

### Multiple role options
```rust
#[protect(any("order:approve", "order:manage"))]
pub async fn approve_order(...) { }
```

## Adding New Permissions

### 1. Define in constants.rs
```rust
// src/modules/permissions/constants.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    // ... existing
    ProductCreate,
    ProductRead,
}

impl Permission {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ProductCreate => "product:create",
            Self::ProductRead => "product:read",
        }
    }
}
```

### 2. Update validation
```rust
pub fn is_valid_permission(permission: &str) -> bool {
    matches!(permission,
        "product:create" | "product:read" | // add new ones
        // ... existing
    )
}
```

### 3. Use in handler
```rust
#[protect("product:create")]
pub async fn create_product(...) { }
```

See [WORKFLOW_ADD_NEW_API.md](./WORKFLOW_ADD_NEW_API.md) for complete guide.

## Troubleshooting

### 403 Forbidden on valid request
- Check if user has required role: `GET /api/auth/me`
- Check if role has required permission: `GET /api/permissions/roles`
- Check permission string format: `resource:action` (lowercase)

### Role not found
- Run seed script: `cargo run --bin seed_roles`
- Check MongoDB: `db.roles.find({name: "ROLE_NAME"})`

### Permission not working
- Check permission is defined in `constants.rs`
- Check `is_valid_permission()` includes the permission
- Check handler has `#[protect("permission:action")]`

### Swagger not showing new endpoints
- Check handler has `#[utoipa::path]` annotation
- Check handler is added to `openapi.rs` paths
- Check DTOs have `#[derive(ToSchema)]`
- Check DTOs are added to `openapi.rs` schemas
- Restart server after changes

## References

- [Workflow: Add New API](./WORKFLOW_ADD_NEW_API.md) - Complete guide for adding APIs
- [Swagger UI](http://localhost:8080/swagger-ui/) - Interactive API documentation
- [Constants](../src/modules/permissions/constants.rs) - Permission definitions
- [Coding Standards](./CODING_STANDARDS.md) - General coding guidelines
