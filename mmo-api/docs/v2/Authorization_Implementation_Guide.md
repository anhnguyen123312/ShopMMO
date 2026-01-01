# Authorization System V2 - Implementation Guide

**Version 2.0 | January 2026**

This guide describes how the authorization system is implemented in the codebase and how to use it.

---

## Table of Contents

1. [Implementation Overview](#1-implementation-overview)
2. [Code Structure](#2-code-structure)
3. [Using Permission Guards](#3-using-permission-guards)
4. [Ownership Checks](#4-ownership-checks)
5. [Adding New Permissions](#5-adding-new-permissions)
6. [Database Setup](#6-database-setup)

---

## 1. Implementation Overview

The authorization system uses:
- **actix-web-grants** for declarative permission guards via `#[protect]` macro
- **JWT** with `roles` array for role-based access
- **MongoDB** for storing permissions and roles
- **Redis** for caching user permissions
- **Ownership utilities** for resource-level authorization

### Architecture Layers

```
Request → AuthMiddleware (JWT) → GrantsMiddleware (permissions) → #[protect] → Handler
                                                        ↓
                                                  Ownership Check
```

---

## 2. Code Structure

### Key Files

| File | Purpose |
|------|---------|
| `src/middleware/auth.rs` | JWT validation, AuthUser extractor |
| `src/middleware/permissions.rs` | Permission extractor for actix-web-grants |
| `src/core/ownership.rs` | Ownership check utilities |
| `src/modules/permissions/` | Permission domain models, repository, cache |
| `src/scripts/check_permission.lua` | Redis atomic permission check |
| `migrations/001_init_permissions.js` | Database migration script |
| `migrations/src/seed_permissions.rs` | Rust seed script |

### AuthUser Structure

```rust
pub struct AuthUser {
    pub user_id: String,      // MongoDB ObjectId
    pub wallet_id: String,    // From JWT (backward compatibility)
    pub email: String,        // From JWT (backward compatibility)
    pub role: String,         // Primary role (backward compatibility)
    pub roles: Vec<String>,   // All assigned roles (V2)
    pub perm_version: u32,    // For cache invalidation
}
```

---

## 3. Using Permission Guards

### Import the protect macro

```rust
use actix_web_grants::protect;
```

### Apply guards to handlers

```rust
// Single role requirement
#[get("/api/wallet/balance")]
#[protect("BUYER", "SELLER")]
pub async fn get_balance(
    service: web::Data<Arc<WalletService>>,
    auth: AuthUser,
) -> Result<HttpResponse, ApiError> {
    let balance = service.get_wallet_balance(&auth.wallet_id).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(balance)))
}

// Admin-only endpoints
#[post("/api/wallet/admin/freeze")]
#[protect("ADMIN", "SUPER_ADMIN")]
pub async fn freeze_wallet(
    service: web::Data<Arc<WalletService>>,
    admin: AdminUser,
    req: web::Json<FreezeWalletRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = service.freeze_wallet(req.into_inner(), admin.user_id).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}
```

### Role Hierarchy

| Role | Level | Inherits | Description |
|------|-------|----------|-------------|
| SUPER_ADMIN | 3 | none | Full system access |
| ADMIN | 2 | SELLER | Platform management |
| SELLER | 1 | BUYER | Products, orders management |
| BUYER | 0 | none | Browse, purchase |

---

## 4. Ownership Checks

For resource-level authorization (user can only access their own resources, admins can access all):

```rust
use crate::core::ownership::{check_ownership, is_admin, has_role};

// In a handler
pub async fn get_wallet_detail(
    auth: AuthUser,
    wallet_id: Path<String>,
    service: web::Data<WalletService>,
) -> Result<HttpResponse, ApiError> {
    // First, fetch the resource
    let wallet = service.find_by_id(&wallet_id).await?;

    // Check ownership (admin bypass)
    check_ownership(&auth, &wallet.user_id)?;

    // Process request
    Ok(HttpResponse::Ok().json(ApiResponse::success(wallet)))
}
```

### Available ownership functions

```rust
// Check if user owns resource (admin bypass)
check_ownership(auth_user, resource_user_id) -> Result<(), ApiError>

// Check if user is admin
is_admin(auth_user) -> bool

// Check if user has specific role
has_role(auth_user, role) -> bool

// Check if user has any of the specified roles
has_any_role(auth_user, roles) -> bool
```

---

## 5. Adding New Permissions

### Step 1: Add permission to database

Via MongoDB shell or migration:

```javascript
db.permissions.insertOne({
  name: "products:approve",
  display_name: "Approve Products",
  description: "Approve or reject product listings",
  resource: "products",
  action: "approve",
  category: "product_management",
  is_active: true,
  created_at: new Date(),
  updated_at: new Date()
});
```

### Step 2: Add to role

```javascript
// Get permission ID
var permId = db.permissions.findOne({name: "products:approve"})._id;

// Add to ADMIN role
db.roles.updateOne(
  {name: "ADMIN"},
  {
    $push: {direct_permissions: permId},
    $inc: {version: 1}
  }
);

// Recompute flattened permissions
var allPerms = db.roles.findOne({name: "ADMIN"}).direct_permissions;
var permNames = db.permissions.find({_id: {$in: allPerms}}).map(p => p.name).toArray();
db.roles.updateOne(
  {name: "ADMIN"},
  {$set: {flattened_permissions: permNames}}
);
```

### Step 3: Use in handler

```rust
#[post("/api/products/{id}/approve")]
#[protect("ADMIN", "SUPER_ADMIN")]
pub async fn approve_product(
    auth: AdminUser,
    product_id: Path<String>,
) -> Result<HttpResponse, ApiError> {
    // ... approval logic
}
```

---

## 6. Database Setup

### Option 1: JavaScript Migration

```bash
mongo "mongodb://localhost:27017/mmo_api" migrations/001_init_permissions.js
```

### Option 2: Rust Seed Script

```bash
# Set MongoDB URL (optional, defaults to mongodb://localhost:27017)
export MONGODB_URL="mongodb://localhost:27017"

# Run seed script
cargo run --bin seed_permissions
```

### Manual: Add roles to a user

```javascript
// Via MongoDB shell
db.users.updateOne(
  {_id: ObjectId("user_id_here")},
  {
    $set: {
      roles: ["BUYER", "SELLER"],
      perm_version: 1
    }
  }
);
```

---

## Quick Reference

### Common patterns

```rust
// Public endpoint (no auth)
#[get("/health")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

// Authenticated endpoint (any logged-in user)
#[get("/api/profile")]
pub async fn get_profile(auth: AuthUser) -> Result<HttpResponse, ApiError> {
    // auth.user_id is available
}

// Role-restricted endpoint
#[get("/api/admin/dashboard")]
#[protect("ADMIN", "SUPER_ADMIN")]
pub async fn admin_dashboard(admin: AdminUser) -> Result<HttpResponse, ApiError> {
    // Only admins
}

// Ownership-protected endpoint
#[put("/api/products/{id}")]
#[protect("SELLER")]
pub async fn update_product(
    auth: AuthUser,
    id: Path<String>,
) -> Result<HttpResponse, ApiError> {
    let product = repo.find_by_id(&id).await?;
    check_ownership(&auth, &product.seller_id)?;
    // ... update logic
}
```

### Error responses

| Status | Meaning |
|--------|---------|
| 401 | No token or invalid token |
| 403 | Valid token but missing required role |
| 403 | Valid token but doesn't own resource |

---

*— End of Implementation Guide —*
