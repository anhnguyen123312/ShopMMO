# Hệ thống Authorization P2P MMO

**Version 2.0 | January 2026**

---

## Tổng quan

Hệ thống Authorization của P2P MMO sử dụng **Hybrid RBAC + ABAC** với dynamic permissions được quản lý qua Admin Dashboard và lưu trữ trong MongoDB.

### Mục tiêu

- Tạo và quản lý roles/permissions động qua dashboard
- Role hierarchy với inheritance (Admin → Seller → Buyer)
- Ownership-based access control cho resources
- Redis caching để đạt latency < 5ms

### Actors

| Actor | Description | Permissions |
|-------|-------------|-------------|
| **super_admin** | Full system access | Tất cả permissions |
| **admin** | Platform management | Quản lý users, roles, content |
| **moderator** | Content moderation | Duyệt bài viết, xử lý report |
| **seller** | Sell products, manage shop | Quản lý products, orders của mình |
| **buyer** | Browse, purchase products | Xem, mua sản phẩm |

---

## 1. Tech Stack

| Component | Technology | Version |
|-----------|------------|---------|
| Framework | actix-web | 4.9 |
| Authorization | actix-web-grants | Latest |
| Database | MongoDB | 4.4+ |
| Cache | Redis | 6.0+ |
| Authentication | JWT | jsonwebtoken crate |

---

## 2. Role Hierarchy

Hệ thống có 4 role levels với inheritance từ trên xuống:

```
┌─────────────────────────────────────────────────────────────┐
│                    ROLE HIERARCHY                           │
└─────────────────────────────────────────────────────────────┘

Level 3: super_admin
   │
   ├── Inherits: admin, moderator, seller, buyer
   │
   ▼
Level 2: admin
   │
   ├── Inherits: moderator, seller, buyer
   │
   ▼
Level 1: moderator
   │
   ├── Inherits: buyer
   │
   ▼
Level 0: seller
   │
   ├── Inherits: buyer
   │
   ▼
Level 0: buyer
```

### 2.1 Role Levels Table

| Level | Role | Inherits From | Description |
|-------|------|---------------|-------------|
| 3 | `super_admin` | admin, moderator, seller, buyer | Full system access |
| 2 | `admin` | moderator, seller, buyer | Platform management |
| 1 | `moderator` | seller, buyer | Content moderation |
| 1 | `seller` | buyer | Sell products, manage shop |
| 0 | `buyer` | (none) | Browse, purchase products |

---

## 3. MongoDB Schema Design

### 3.1 Permissions Collection

Lưu trữ tất cả permissions có thể assign cho roles.

**Collection:** `permissions`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `_id` | ObjectId | ✅ | Primary key |
| `name` | String | ✅ | Unique: `"resource:action"` |
| `display_name` | String | ✅ | Human-readable name |
| `description` | String | ❌ | Detailed description |
| `resource` | String | ✅ | Resource type |
| `action` | String | ✅ | Action type |
| `category` | String | ✅ | Grouping: marketplace, admin, finance |
| `is_active` | Boolean | ✅ | Soft delete flag |
| `created_at` | DateTime | ✅ | Creation timestamp |
| `updated_at` | DateTime | ✅ | Last update timestamp |

**Example Document:**

```json
{
  "_id": ObjectId("..."),
  "name": "products:create",
  "display_name": "Create Products",
  "description": "Allows creation of new product listings",
  "resource": "products",
  "action": "create",
  "category": "marketplace",
  "is_active": true,
  "created_at": ISODate("2025-01-01T00:00:00Z"),
  "updated_at": ISODate("2025-01-01T00:00:00Z")
}
```

**Indexes:**

```javascript
db.permissions.createIndex({ "name": 1 }, { unique: true })
db.permissions.createIndex({ "resource": 1, "action": 1 })
db.permissions.createIndex({ "category": 1, "is_active": 1 })
```

---

### 3.2 Roles Collection

Định nghĩa roles với hierarchy và permissions.

**Collection:** `roles`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `_id` | ObjectId | ✅ | Primary key |
| `name` | String | ✅ | Unique: buyer, seller, admin, super_admin |
| `display_name` | String | ✅ | Human-readable name |
| `level` | i32 | ✅ | Hierarchy level (0=lowest) |
| `parent_role_id` | ObjectId? | ❌ | Reference to parent role |
| `inherits_from` | [String] | ✅ | List of inherited role names |
| `direct_permissions` | [ObjectId] | ✅ | Directly assigned permissions |
| `flattened_permissions` | [String] | ✅ | All permissions (direct + inherited) |
| `is_system` | Boolean | ✅ | Built-in role flag |
| `is_active` | Boolean | ✅ | Soft delete flag |
| `version` | i32 | ✅ | Optimistic locking & cache invalidation |

**Example Document:**

```json
{
  "_id": ObjectId("..."),
  "name": "seller",
  "display_name": "Seller",
  "level": 1,
  "parent_role_id": ObjectId("..."),
  "inherits_from": ["buyer"],
  "direct_permissions": [
    ObjectId("..."),
    ObjectId("...")
  ],
  "flattened_permissions": [
    "products:create",
    "products:read",
    "products:update",
    "products:delete",
    "orders:read",
    "shops:create",
    "shops:update"
  ],
  "is_system": true,
  "is_active": true,
  "version": 3
}
```

---

### 3.3 Users Collection - Authorization Fields

Các fields liên quan đến authorization trong users collection:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `roles` | [RoleAssignment] | ✅ | Array of assigned roles |
| `roles[].role_id` | ObjectId | ✅ | Reference to roles collection |
| `roles[].role_name` | String | ✅ | Denormalized for quick access |
| `roles[].assigned_at` | DateTime | ✅ | When role was assigned |
| `roles[].assigned_by` | ObjectId? | ❌ | Who assigned (null = system) |
| `direct_permissions` | [String] | ✅ | User-specific permissions |
| `effective_permissions` | [String] | ✅ | All permissions (roles + direct) |
| `perm_version` | i32 | ✅ | Cache invalidation version |

**Example Document (partial):**

```json
{
  "_id": ObjectId("..."),
  "email": "seller@example.com",
  "roles": [
    {
      "role_id": ObjectId("..."),
      "role_name": "seller",
      "assigned_at": ISODate("2025-01-15T00:00:00Z"),
      "assigned_by": null
    }
  ],
  "direct_permissions": [],
  "effective_permissions": [
    "products:create",
    "products:read",
    "products:update",
    "orders:read",
    "shops:create"
  ],
  "perm_version": 5
}
```

---

## 4. Permission Resolution Flow

### 4.1 Request Authentication & Authorization Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    HTTP REQUEST                             │
│              Authorization: Bearer <JWT>                    │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│  [Bước 1] JWT Validation (~1-2ms)                          │
│         │                                                  │
│         ├── Verify signature                               │
│         ├── Check expiration                               │
│         └── Extract: user_id, roles, perm_version          │
│         │                                                  │
│         ▼                                                  │
│    Invalid ──► Return 401 UNAUTHORIZED                     │
│    Valid     ──► Continue                                  │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│  [Bước 2] Redis Cache Check (~0.5-1ms)                     │
│         Key: user:{user_id}:permissions                    │
│         │                                                  │
│         ├── Check perm_version matches JWT                 │
│         └── SISMEMBER for required permission              │
│         │                                                  │
│         ├── Cache Hit ──────────────────────────────┐       │
│         ├── Cache Miss ───────────────────────┐      │       │
│         ├── Cache Stale (version mismatch) ────┤      │       │
│         │                                │      │       │
│         ▼                                ▼      │       │
│    [Decision]                      [Query MongoDB]      │       │
│    │                                │      │       │
│    ├── Has permission ──► Authorized │      │       │
│    ├── No permission ──► Return 403  │      │       │
│    │                                │      │       │
│    │                                ▼      │       │
│    │                        [Bước 3] MongoDB Query   │       │
│    │                                (~5-10ms)          │       │
│    │                                │                  │       │
│    │                                ├── Get effective_permissions
│    │                                ├── Update Redis cache        │
│    │                                │                  │       │
│    │                                ▼                  │       │
│    │                           [Decision]            │       │
│    │                                │                  │       │
│    │                                ├── Has permission ──► Authorized
│    │                                └── No permission ──► Return 403
└─────────────────────────────────────────────────────────────┘
```

### 4.1.1 Conditions/Requirements

┌─────────────────────────────────────────┐
│         ĐIỀU KIỆN THỰC HIỆN            │
└─────────────────────────────────────────┘

**Preconditions:**
- Actor: Any authenticated user
- State: User must have valid JWT token
- Data: JWT contains user_id, roles, perm_version

**Input Requirements:**
- Header: `Authorization: Bearer <JWT>`
- JWT Claims: `sub` (user_id), `roles` (array), `perm_version` (number)

**Business Rules:**
- JWT signature must be valid
- JWT must not be expired
- perm_version in JWT must match cached version

**Edge Cases:**
- Expired JWT ──► Return 401 with "Token expired"
- Invalid signature ──► Return 401 with "Invalid token"
- Cache miss ──► Query MongoDB and update cache
- Version mismatch ──► Refresh cache from MongoDB

---

### 4.2 Seller Registration Flow

Khi user đăng ký và hoàn thành thủ tục seller:

```
┌─────────────────────────────────────────────────────────────┐
│  [Bước 1] User submits seller registration                  │
│         POST /api/v1/seller/register                       │
│         │                                                  │
│         ├── KYC data                                       │
│         ├── Business information                           │
│         └── Payment methods                                │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│  [Bước 2] Validate seller data                             │
│         │                                                  │
│         ├── Valid ──► Continue                             │
│         ├── Invalid ──► Return 400 with errors             │
│         └── Pending review ──► Return 202 (accepted)       │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│  [Bước 3] MongoDB: Add 'seller' role to user               │
│         │                                                  │
│         ├── Update user.roles[]                            │
│         ├── Increment perm_version                         │
│         └── Recompute effective_permissions                │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│  [Bước 4] Redis: Invalidate cache                          │
│         │                                                  │
│         ├── DEL user:{user_id}:permissions                 │
│         └── DEL user:{user_id}:perm_version                │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│  [Bước 5] Generate new JWT with updated roles              │
│         │                                                  │
│         └── Claims: { sub, roles: ["buyer", "seller"],     │
│                      perm_version }                         │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│  [Bước 6] Return response to client                        │
│         │                                                  │
│         └── { success: true, access_token: "..." }         │
└─────────────────────────────────────────────────────────────┘
```

### 4.2.1 Conditions/Requirements

┌─────────────────────────────────────────┐
│         ĐIỀU KIỆN THỰC HIỆN            │
└─────────────────────────────────────────┘

**Preconditions:**
- Actor: Buyer user
- State: User logged in, has buyer role
- Data: Valid KYC documents, business info

**Input Requirements:**
- Required: full_name, business_name, tax_id
- Optional: business_address, phone_number

**Business Rules:**
- User must complete KYC verification
- Business information must be valid
- Tax ID must be unique

**Edge Cases:**
- Duplicate tax_id ──► Return 409 "Tax ID already exists"
- Incomplete KYC ──► Return 400 "KYC documents required"
- Pending approval ──► Return 202 "Under review"

---

### 4.3 Role Permission Update Flow

Khi admin thay đổi permissions của một role:

```
┌─────────────────────────────────────────────────────────────┐
│  [Bước 1] Admin updates role permissions                    │
│         POST /api/v1/roles/:id/permissions                 │
│         │                                                  │
│         └── Body: { permissions: ["..."] }                 │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│  [Bước 2] Update role in MongoDB                           │
│         │                                                  │
│         ├── Update role.direct_permissions                 │
│         ├── Recompute role.flattened_permissions           │
│         └── Increment role.version                         │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│  [Bước 3] Find all users with this role                    │
│         │                                                  │
│         └── Query: { "roles.role_name": role_name }        │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│  [Bước 4] For each affected user:                          │
│         │                                                  │
│         ├── Recompute effective_permissions                │
│         └── Increment perm_version                         │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│  [Bước 5] Bulk invalidate Redis cache                      │
│         │                                                  │
│         ├── DEL role:{role_name}:permissions               │
│         ├── DEL user:{user_id}:permissions (all affected)  │
│         └── DEL user:{user_id}:perm_version (all affected) │
└─────────────────────────────────────────────────────────────┘
```

### 4.3.1 Conditions/Requirements

┌─────────────────────────────────────────┐
│         ĐIỀU KIỆN THỰC HIỆN            │
└─────────────────────────────────────────┘

**Preconditions:**
- Actor: Admin or super_admin
- State: Role must exist
- Data: List of permission IDs

**Input Requirements:**
- Required: permissions (array of ObjectIds)
- Validation: All permissions must exist and be active

**Business Rules:**
- System roles cannot be deleted
- Permission changes affect all users with that role
- Cache invalidation is atomic

**Edge Cases:**
- Role not found ──► Return 404 "Role not found"
- Invalid permission ID ──► Return 400 "Invalid permission"
- System role modification ──► Allow but log audit

---

## 5. Coding Conventions & Guidelines

### 5.1 Permission Naming Convention

Tất cả permissions phải tuân theo format: **`resource:action`**

| Component | Convention | Examples |
|-----------|------------|----------|
| Resource | Plural noun, snake_case | `products`, `orders`, `shop_settings` |
| Action | Verb, lowercase | `create`, `read`, `update`, `delete`, `manage` |
| Special | Wildcard for all actions | `products:*` |

#### Standard Actions

| Action | Description |
|--------|-------------|
| `create` | Create new resource |
| `read` | View resource (list and detail) |
| `update` | Modify existing resource |
| `delete` | Remove resource (soft/hard delete) |
| `manage` | Full control including admin actions |
| `approve` | Approve pending items |
| `export` | Export data |

---

### 5.2 Adding Permission Guard to Handlers

Sử dụng macro `#[protect()]` từ actix-web-grants:

#### Basic Usage

```rust
// Single permission required
#[get("/products")]
#[protect("products:read")]
async fn list_products() -> HttpResponse {
    // ... handler logic
}

// Multiple permissions - ANY (OR logic)
#[post("/products")]
#[protect(any("products:create", "products:manage"))]
async fn create_product() -> HttpResponse {
    // ... handler logic
}

// Multiple permissions - ALL (AND logic)
#[delete("/products/{id}")]
#[protect("products:delete", "products:manage")]
async fn delete_product() -> HttpResponse {
    // ... handler logic
}
```

---

### 5.3 Ownership Check Pattern

Khi cần check resource ownership:

```rust
#[put("/products/{id}")]
#[protect("products:update")]  // First check permission
async fn update_product(
    auth: AuthenticatedUser,
    product_id: Path<String>,
    product_service: Data<ProductService>,
) -> Result<HttpResponse, Error> {
    // Then check ownership (unless admin)
    product_service.check_ownership_or_admin(
        &product_id,
        &auth.user_id,
        &auth.roles
    ).await?;

    // ... proceed with update
}
```

#### Ownership Check Implementation

```rust
impl ProductService {
    pub async fn check_ownership_or_admin(
        &self,
        product_id: &str,
        user_id: &str,
        roles: &[String],
    ) -> Result<(), Error> {
        // Admin bypass
        if roles.contains(&"admin".to_string())
            || roles.contains(&"super_admin".to_string()) {
            return Ok(());
        }

        // Check ownership
        let product = self.repository.find_by_id(product_id).await?;
        if product.owner_id != user_id {
            return Err(Error::Forbidden("OWNERSHIP_REQUIRED"));
        }

        Ok(())
    }
}
```

---

### 5.4 New Module Permission Checklist

Khi tạo module mới, follow checklist sau:

1. **Define permissions** - Tạo permissions trong database
   ```
   // permissions to create:
   // {module}:create, {module}:read, {module}:update, {module}:delete
   ```

2. **Assign to roles** - Add permissions vào appropriate roles

3. **Add guards** - Thêm `#[protect()]` macro vào handlers

4. **Ownership check** - Implement nếu resource có owner

5. **Update docs** - Document trong module's CONTEXT.md

#### Example: Adding "reviews" module

```rust
// Step 1: Create permissions (via API or migration)
// reviews:create, reviews:read, reviews:update, reviews:delete

// Step 2: Assign to roles
// - buyer: reviews:create, reviews:read, reviews:update (own), reviews:delete (own)
// - admin: reviews:manage

// Step 3: Add guards to handlers
#[post("/reviews")]
#[protect("reviews:create")]
async fn create_review() -> HttpResponse { ... }

#[get("/reviews")]
#[protect("reviews:read")]
async fn list_reviews() -> HttpResponse { ... }

#[put("/reviews/{id}")]
#[protect("reviews:update")]
async fn update_review(auth: AuthenticatedUser, ...) -> HttpResponse {
    // Step 4: Ownership check
    review_service.check_ownership_or_admin(...).await?;
    // ...
}
```

---

## 6. API Endpoints

### 6.1 Permission Management APIs

| Method | Endpoint | Permission | Description |
|--------|----------|------------|-------------|
| GET | `/api/v1/permissions` | `permissions:read` | List all permissions |
| GET | `/api/v1/permissions/:id` | `permissions:read` | Get permission detail |
| POST | `/api/v1/permissions` | `permissions:create` | Create new permission |
| PUT | `/api/v1/permissions/:id` | `permissions:update` | Update permission |
| DELETE | `/api/v1/permissions/:id` | `permissions:delete` | Soft delete permission |

---

### 6.2 Role Management APIs

| Method | Endpoint | Permission | Description |
|--------|----------|------------|-------------|
| GET | `/api/v1/roles` | `roles:read` | List all roles |
| GET | `/api/v1/roles/:id` | `roles:read` | Get role detail |
| POST | `/api/v1/roles` | `roles:create` | Create new role |
| PUT | `/api/v1/roles/:id` | `roles:update` | Update role |
| DELETE | `/api/v1/roles/:id` | `roles:delete` | Delete role |
| POST | `/api/v1/roles/:id/permissions` | `roles:manage` | Add permissions to role |
| DELETE | `/api/v1/roles/:id/permissions/:permId` | `roles:manage` | Remove permission from role |

---

### 6.3 User Role Assignment APIs

| Method | Endpoint | Permission | Description |
|--------|----------|------------|-------------|
| GET | `/api/v1/users/:id/roles` | `users:read` | Get user's roles |
| POST | `/api/v1/users/:id/roles` | `users:manage_roles` | Assign role to user |
| DELETE | `/api/v1/users/:id/roles/:roleId` | `users:manage_roles` | Remove role from user |
| GET | `/api/v1/users/:id/permissions` | `users:read` | Get effective permissions |

---

## 7. Default Permissions Matrix

### 7.1 Products Module

| Permission | Buyer | Seller | Admin | Super Admin |
|------------|-------|--------|-------|-------------|
| `products:read` | ✓ | ✓ | ✓ | ✓ |
| `products:create` | | ✓ | ✓ | ✓ |
| `products:update` | | ✓ (own) | ✓ | ✓ |
| `products:delete` | | ✓ (own) | ✓ | ✓ |
| `products:approve` | | | ✓ | ✓ |
| `products:manage` | | | ✓ | ✓ |

---

### 7.2 Orders Module

| Permission | Buyer | Seller | Admin | Super Admin |
|------------|-------|--------|-------|-------------|
| `orders:create` | ✓ | ✓ | ✓ | ✓ |
| `orders:read` | ✓ (own) | ✓ (own) | ✓ | ✓ |
| `orders:cancel` | ✓ (own) | ✓ (own) | ✓ | ✓ |
| `orders:refund` | | | ✓ | ✓ |
| `orders:manage` | | | ✓ | ✓ |

---

### 7.3 Shops Module

| Permission | Buyer | Seller | Admin | Super Admin |
|------------|-------|--------|-------|-------------|
| `shops:read` | ✓ | ✓ | ✓ | ✓ |
| `shops:create` | | ✓ | ✓ | ✓ |
| `shops:update` | | ✓ (own) | ✓ | ✓ |
| `shops:delete` | | | ✓ | ✓ |
| `shops:manage` | | | ✓ | ✓ |

---

### 7.4 Admin Module

| Permission | Buyer | Seller | Admin | Super Admin |
|------------|-------|--------|-------|-------------|
| `permissions:read` | | | ✓ | ✓ |
| `permissions:create` | | | | ✓ |
| `permissions:update` | | | | ✓ |
| `permissions:delete` | | | | ✓ |
| `permissions:manage` | | | | ✓ |
| `roles:read` | | | ✓ | ✓ |
| `roles:create` | | | | ✓ |
| `roles:update` | | | | ✓ |
| `roles:delete` | | | | ✓ |
| `roles:manage` | | | | ✓ |
| `users:read` | | | ✓ | ✓ |
| `users:manage_roles` | | | ✓ | ✓ |
| `system:*` | | | | ✓ |

---

## 8. Redis Caching Strategy

### 8.1 Key Structure

| Key Pattern | Type | Description |
|-------------|------|-------------|
| `user:{user_id}:permissions` | SET | User's effective permissions |
| `user:{user_id}:perm_version` | STRING | Permission version for cache validation |
| `role:{role_name}:permissions` | SET | Role's flattened permissions |

---

### 8.2 TTL Configuration

| Key Type | TTL | Rationale |
|----------|-----|-----------|
| User permissions | 10 minutes | Balance freshness & performance |
| Role definitions | 2 hours | Roles change infrequently |

---

### 8.3 Cache Invalidation Events

| Event | Actions |
|-------|---------|
| User role change | Delete `user:{id}:permissions`, `user:{id}:perm_version` |
| Role permission change | Delete `role:{name}:permissions` + all affected users |
| Permission deleted | Rebuild all roles containing it + invalidate users |

---

### 8.4 Lua Script for Atomic Permission Check

```lua
-- check_permission.lua
-- KEYS[1] = user:{user_id}:permissions
-- KEYS[2] = user:{user_id}:perm_version
-- ARGV[1] = required_permission
-- ARGV[2] = jwt_perm_version
-- Returns: 1 = authorized, 0 = denied, -1 = cache miss/stale

local cached_version = redis.call('GET', KEYS[2])

if not cached_version then
    return -1  -- Cache miss
end

if tonumber(cached_version) ~= tonumber(ARGV[2]) then
    return -1  -- Version mismatch, need refresh
end

return redis.call('SISMEMBER', KEYS[1], ARGV[1])
```

---

## 9. Authorization Error Responses

### 9.1 Error Codes

| Status | Code | Description |
|--------|------|-------------|
| 401 | `UNAUTHORIZED` | Missing or invalid JWT token |
| 403 | `FORBIDDEN` | Valid token but insufficient permissions |
| 403 | `OWNERSHIP_REQUIRED` | User doesn't own the resource |
| 403 | `ROLE_REQUIRED` | Specific role needed for this action |

---

### 9.2 Error Response Format

```json
{
  "success": false,
  "error": {
    "code": "FORBIDDEN",
    "message": "You don't have permission to perform this action",
    "required_permission": "products:delete"
  }
}
```

---

## 10. Quick Reference

### 10.1 Handler Permission Guard Cheat Sheet

```rust
// Read operations
#[protect("resource:read")]

// Write operations (create/update)
#[protect("resource:create")]  // or resource:update

// Delete operations
#[protect("resource:delete")]

// Admin-only operations
#[protect("resource:manage")]

// Multiple permissions (OR)
#[protect(any("perm1", "perm2"))]

// Multiple permissions (AND)
#[protect("perm1", "perm2")]
```

---

### 10.2 JWT Claims Structure

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JWTClaims {
    pub sub: String,           // user_id
    pub roles: Vec<String>,    // ["buyer", "seller"]
    pub perm_version: u32,     // For cache invalidation
    pub iat: i64,              // Issued at
    pub exp: i64,              // Expiration
}
```

---

### 10.3 Permission Middleware Setup

```rust
use actix_web::{App, HttpServer};
use actix_web_grants::GrantsMiddleware;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(GrantsMiddleware::with_extractor(extract_permissions))
            .configure(routes::configure)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
```

---

*— End of Document —*
