# Authorization System V2 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Migrate from hardcoded role-based authorization to dynamic RBAC + ABAC hybrid system with Redis caching

**Architecture:**
- JWT contains `roles: Vec<String>` and `perm_version` for cache invalidation
- MongoDB stores dynamic permissions, roles with hierarchy, and user role assignments
- Redis caches user permissions with atomic Lua script verification
- actix-web-grants provides `#[protect()]` macro for handler guards

**Tech Stack:**
- actix-web 4.9 + actix-web-grants
- MongoDB 4.4+ (permissions, roles collections)
- Redis 6.0+ (permission caching with Lua scripts)
- JWT with role arrays and permission versioning

---

## Phase 1: Core Authorization Infrastructure

### Task 1: Add actix-web-grants Dependency

**Files:**
- Modify: `mmo-api/Cargo.toml`

**Step 1: Add dependency to Cargo.toml**

```toml
# In [dependencies] section
actix-web-grants = "4"
```

**Step 2: Run cargo check**

```bash
cd mmo-api && cargo check
```

Expected: Dependencies resolve successfully

**Step 3: Commit**

```bash
git add mmo-api/Cargo.toml
git commit -m "deps: add actix-web-grants for permission-based authorization"
```

---

### Task 2: Update JWT Claims Structure

**Files:**
- Modify: `mmo-api/src/utils/jwt.rs`

**Step 1: Write failing test for new claims structure**

Create `mmo-api/src/utils/jwt_tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    #[test]
    fn test_jwt_claims_with_roles_array() {
        let claims = TokenClaims {
            sub: "user123".to_string(),
            roles: vec!["buyer".to_string(), "seller".to_string()],
            perm_version: 5,
            iat: 1704067200,
            exp: 1735689600,
        };

        let token = encode_jwt(&claims).unwrap();
        let decoded = decode_jwt(&token).unwrap();

        assert_eq!(decoded.sub, "user123");
        assert_eq!(decoded.roles.len(), 2);
        assert!(decoded.roles.contains(&"seller".to_string()));
        assert_eq!(decoded.perm_version, 5);
    }
}
```

**Step 2: Run test to verify it fails**

```bash
cd mmo-api && cargo test jwt_claims_with_roles_array
```

Expected: FAIL - `TokenClaims` doesn't have `roles` or `perm_version` fields

**Step 3: Update TokenClaims struct**

In `mmo-api/src/utils/jwt.rs`, replace existing claims:

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenClaims {
    pub sub: String,              // user_id
    pub roles: Vec<String>,       // Array of role names (e.g., ["buyer", "seller"])
    pub perm_version: u32,        // Permission version for cache invalidation
    pub iat: i64,                 // Issued at timestamp
    pub exp: i64,                 // Expiration timestamp
}
```

**Step 4: Update encode_jwt function**

Remove `wallet_id` and `email` from claims (fetch from DB instead):

```rust
pub fn encode_jwt(claims: &TokenClaims) -> Result<String, JwtError> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp();

    let claims = TokenClaims {
        sub: claims.sub.clone(),
        roles: claims.roles.clone(),
        perm_version: claims.perm_version,
        iat: Utc::now().timestamp(),
        exp: expiration,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(get_jwt_secret()))
        .map_err(|_| JwtError::EncodingError)
}
```

**Step 5: Update decode_jwt function**

```rust
pub fn decode_jwt(token: &str) -> Result<TokenClaims, JwtError> {
    decode::<TokenClaims>(
        token,
        &DecodingKey::from_secret(get_jwt_secret()),
        &Validation::new(Algorithm::HS256),
    )
    .map(|data| data.claims)
    .map_err(|_| JwtError::DecodingError)
}
```

**Step 6: Update auth service to use new claims**

In `mmo-api/src/modules/auth/service.rs`, modify login/token generation:

```rust
impl AuthService {
    pub async fn generate_tokens(&self, user_id: &str, roles: Vec<String>, perm_version: u32) -> Result<TokenPair, ServiceError> {
        let claims = TokenClaims {
            sub: user_id.to_string(),
            roles,
            perm_version,
            iat: Utc::now().timestamp(),
            exp: 0, // Will be set in encode_jwt
        };

        let access_token = encode_jwt(&claims)?;
        let refresh_token = generate_refresh_token()?;

        // Store refresh token in Redis
        self.redis_store
            .set(&format!("refresh_token:{}", user_id), &refresh_token, 60 * 60 * 24 * 7)
            .await?;

        Ok(TokenPair {
            access_token,
            refresh_token,
        })
    }
}
```

**Step 7: Run test to verify it passes**

```bash
cd mmo-api && cargo test jwt_claims_with_roles_array
```

Expected: PASS

**Step 8: Commit**

```bash
git add mmo-api/src/utils/jwt.rs mmo-api/src/utils/jwt_tests.rs mmo-api/src/modules/auth/service.rs
git commit -m "feat: update JWT claims to support multiple roles and permission versioning"
```

---

### Task 3: Create Permission and Role Domain Models

**Files:**
- Create: `mmo-api/src/modules/permissions/mod.rs`
- Create: `mmo-api/src/modules/permissions/domain.rs`

**Step 1: Write tests for permission model**

Create `mmo-api/src/modules/permissions/domain_tests.rs`:

```rust
use super::*;

#[test]
fn test_permission_creation() {
    let perm = Permission {
        id: None,
        name: "products:create".to_string(),
        display_name: "Create Products".to_string(),
        description: "Allows creation of new product listings".to_string(),
        resource: "products".to_string(),
        action: "create".to_string(),
        category: "marketplace".to_string(),
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    assert_eq!(perm.resource, "products");
    assert_eq!(perm.action, "create");
    assert_eq!(perm.name, "products:create");
}

#[test]
fn test_role_with_inheritance() {
    let role = Role {
        id: None,
        name: "seller".to_string(),
        display_name: "Seller".to_string(),
        level: 1,
        parent_role_id: None,
        inherits_from: vec!["buyer".to_string()],
        direct_permissions: vec![],
        flattened_permissions: vec![
            "products:create".to_string(),
            "products:read".to_string(),
        ],
        is_system: true,
        is_active: true,
        version: 1,
    };

    assert_eq!(role.level, 1);
    assert!(role.inherits_from.contains(&"buyer".to_string()));
    assert_eq!(role.flattened_permissions.len(), 2);
}
```

**Step 2: Run test to verify it fails**

```bash
cd mmo-api && cargo test permission_creation
```

Expected: FAIL - modules don't exist yet

**Step 3: Create permission domain models**

In `mmo-api/src/modules/permissions/domain.rs`:

```rust
use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

/// Permission represents a granular action on a resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,                  // "resource:action" format
    pub display_name: String,          // Human-readable name
    pub description: String,           // Detailed description
    pub resource: String,              // Resource type: products, orders, etc.
    pub action: String,                // Action type: create, read, update, delete
    pub category: String,              // Grouping: marketplace, admin, finance
    pub is_active: bool,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

/// Role with hierarchy and inheritance support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,                  // buyer, seller, admin, super_admin
    pub display_name: String,
    pub level: i32,                    // Hierarchy level (0=lowest)
    pub parent_role_id: Option<ObjectId>,
    pub inherits_from: Vec<String>,    // List of role names to inherit from
    pub direct_permissions: Vec<ObjectId>,  // Directly assigned permissions
    pub flattened_permissions: Vec<String>,  // Pre-computed all permissions
    pub is_system: bool,               // Built-in roles cannot be deleted
    pub is_active: bool,
    pub version: i32,                  // For optimistic locking
}

/// User role assignment with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleAssignment {
    pub role_id: ObjectId,
    pub role_name: String,             // Denormalized for quick access
    pub assigned_at: DateTime,
    pub assigned_by: Option<ObjectId>, // null = system assigned
}

/// User permission query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermissions {
    pub user_id: String,
    pub roles: Vec<String>,
    pub direct_permissions: Vec<String>,
    pub effective_permissions: Vec<String>,
    pub perm_version: i32,
}
```

**Step 4: Create module exports**

In `mmo-api/src/modules/permissions/mod.rs`:

```rust
pub mod domain;
pub mod dto;
pub mod repository;
pub mod service;
pub mod handler;
pub mod routes;

pub use domain::*;
```

**Step 5: Add to main module tree**

In `mmo-api/src/modules/mod.rs`:

```rust
pub mod permissions;
```

**Step 6: Run test to verify it passes**

```bash
cd mmo-api && cargo test permission_creation role_with_inheritance
```

Expected: PASS

**Step 7: Commit**

```bash
git add mmo-api/src/modules/permissions/
git commit -m "feat: add permission and role domain models with hierarchy support"
```

---

### Task 4: Create Permission Repository

**Files:**
- Create: `mmo-api/src/modules/permissions/repository.rs`

**Step 1: Write tests for permission repository**

Create `mmo-api/src/modules/permissions/repository_tests.rs`:

```rust
use super::*;
use crate::database::mongodb::MongoDB;
use mongodb::bson::doc;

#[tokio::test]
async fn test_create_permission() {
    let db = MongoDB::new("mongodb://localhost:27017/test").await.unwrap();
    let repo = PermissionRepository::new(db);

    let perm = Permission {
        id: None,
        name: "test:create".to_string(),
        display_name: "Test Create".to_string(),
        description: "Test permission".to_string(),
        resource: "test".to_string(),
        action: "create".to_string(),
        category: "test".to_string(),
        is_active: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let created = repo.create_permission(perm).await.unwrap();
    assert!(created.id.is_some());
    assert_eq!(created.name, "test:create");
}

#[tokio::test]
async fn test_get_role_flattened_permissions() {
    let db = MongoDB::new("mongodb://localhost:27017/test").await.unwrap();
    let repo = RoleRepository::new(db);

    let permissions = repo.get_role_permissions("seller").await.unwrap();
    assert!(!permissions.is_empty());
}
```

**Step 2: Run test to verify it fails**

```bash
cd mmo-api && cargo test create_permission
```

Expected: FAIL - repository doesn't exist

**Step 3: Implement PermissionRepository**

In `mmo-api/src/modules/permissions/repository.rs`:

```rust
use mongodb::{
    bson::{doc, oid::ObjectId},
    Collection, Database,
};
use super::domain::*;

pub struct PermissionRepository {
    collection: Collection<Permission>,
}

impl PermissionRepository {
    pub fn new(db: Database) -> Self {
        Self {
            collection: db.collection("permissions"),
        }
    }

    pub async fn create_permission(&self, perm: Permission) -> Result<Permission, DbError> {
        let now = chrono::Utc::now();
        let mut perm = perm;
        perm.created_at = now;
        perm.updated_at = now;

        self.collection
            .insert_one(&perm, None)
            .await
            .map(|_| perm)
            .map_err(|e| DbError::InsertError(e.to_string()))
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Option<Permission>, DbError> {
        self.collection
            .find_one(doc! { "name": name, "is_active": true }, None)
            .await
            .map_err(|e| DbError::QueryError(e.to_string()))
    }

    pub async fn list_all(&self) -> Result<Vec<Permission>, DbError> {
        self.collection
            .find(doc! { "is_active": true }, None)
            .await
            .map_err(|e| DbError::QueryError(e.to_string()))?
            .try_collect()
            .await
            .map_err(|e| DbError::QueryError(e.to_string()))
    }
}

pub struct RoleRepository {
    collection: Collection<Role>,
    users_collection: Collection<mongodb::bson::Document>,
}

impl RoleRepository {
    pub fn new(db: Database) -> Self {
        Self {
            collection: db.collection("roles"),
            users_collection: db.collection("users"),
        }
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Option<Role>, DbError> {
        self.collection
            .find_one(doc! { "name": name, "is_active": true }, None)
            .await
            .map_err(|e| DbError::QueryError(e.to_string()))
    }

    pub async fn get_role_permissions(&self, role_name: &str) -> Result<Vec<String>, DbError> {
        let role = self.find_by_name(role_name).await?
            .ok_or_else(|| DbError::NotFound("Role not found".to_string()))?;

        Ok(role.flattened_permissions)
    }

    pub async fn get_user_permissions(&self, user_id: &str) -> Result<UserPermissions, DbError> {
        let user_doc = self.users_collection
            .find_one(doc! { "_id": user_id }, None)
            .await
            .map_err(|e| DbError::QueryError(e.to_string()))?
            .ok_or_else(|| DbError::NotFound("User not found".to_string()))?;

        let roles: Vec<String> = user_doc
            .get_array("roles")
            .unwrap_or(&mongodb::bson::Bson::Array(vec![]))
            .iter()
            .filter_map(|b| b.as_str().map(String::from))
            .collect();

        let effective_permissions: Vec<String> = user_doc
            .get_array("effective_permissions")
            .unwrap_or(&mongodb::bson::Bson::Array(vec![]))
            .iter()
            .filter_map(|b| b.as_str().map(String::from))
            .collect();

        let perm_version = user_doc
            .get_i32("perm_version")
            .unwrap_or(0);

        Ok(UserPermissions {
            user_id: user_id.to_string(),
            roles,
            direct_permissions: vec![],
            effective_permissions,
            perm_version,
        })
    }
}
```

**Step 4: Run test to verify it passes**

```bash
cd mmo-api && cargo test create_permission get_role_flattened_permissions
```

Expected: PASS

**Step 5: Commit**

```bash
git add mmo-api/src/modules/permissions/repository.rs
git commit -m "feat: implement permission and role repositories"
```

---

## Phase 2: Redis Permission Caching

### Task 5: Add Redis Permission Cache Layer

**Files:**
- Create: `mmo-api/src/modules/permissions/cache.rs`
- Modify: `mmo-api/src/database/redis.rs`

**Step 1: Write tests for permission cache**

Create `mmo-api/src/modules/permissions/cache_tests.rs`:

```rust
use super::*;

#[tokio::test]
async fn test_cache_and_retrieve_permissions() {
    let cache = PermissionCache::new("redis://localhost:6379").await.unwrap();
    let user_id = "test_user_123";

    let permissions = vec![
        "products:read".to_string(),
        "products:create".to_string(),
    ];

    cache.set_permissions(user_id, &permissions, 1).await.unwrap();

    let cached = cache.get_permissions(user_id).await.unwrap();
    assert_eq!(cached.len(), 2);
    assert!(cached.contains(&"products:read".to_string()));
}

#[tokio::test]
async fn test_check_permission_with_version() {
    let cache = PermissionCache::new("redis://localhost:6379").await.unwrap();

    cache.set_permissions("user_1", &vec!["products:read".to_string()], 5).await.unwrap();

    // Correct version
    let result = cache.check_permission("user_1", "products:read", 5).await.unwrap();
    assert_eq!(result, true);

    // Stale version
    let result = cache.check_permission("user_1", "products:read", 6).await.unwrap();
    assert_eq!(result, false); // Should return false for stale
}
```

**Step 2: Run test to verify it fails**

```bash
cd mmo-api && cargo test cache_and_retrieve_permissions
```

Expected: FAIL - cache module doesn't exist

**Step 3: Implement PermissionCache**

In `mmo-api/src/modules/permissions/cache.rs`:

```rust
use redis::{AsyncCommands, Client};
use serde::{Deserialize, Serialize};

pub struct PermissionCache {
    client: Client,
}

impl PermissionCache {
    pub async fn new(url: &str) -> Result<Self, redis::RedisError> {
        let client = Client::open(url)?;
        // Test connection
        let mut conn = client.get_async_connection().await?;
        conn.ping().await?;
        Ok(Self { client })
    }

    /// Set user permissions in Redis
    pub async fn set_permissions(
        &self,
        user_id: &str,
        permissions: &[String],
        version: i32,
    ) -> Result<(), redis::RedisError> {
        let mut conn = self.client.get_async_connection().await?;

        // Store permissions as SET
        let key = format!("user:{}:permissions", user_id);
        conn.del(&key).await?; // Clear old data

        for perm in permissions {
            conn.sadd(&key, perm).await?;
        }

        // Set expiration (10 minutes)
        conn.expire(&key, 600).await?;

        // Store version
        let version_key = format!("user:{}:perm_version", user_id);
        conn.set(&version_key, version).await?;
        conn.expire(&version_key, 600).await?;

        Ok(())
    }

    /// Get all user permissions from cache
    pub async fn get_permissions(&self, user_id: &str) -> Result<Vec<String>, redis::RedisError> {
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("user:{}:permissions", user_id);

        let perms: Vec<String> = conn.smembers(&key).await?;
        Ok(perms)
    }

    /// Check if user has specific permission with version validation
    pub async fn check_permission(
        &self,
        user_id: &str,
        permission: &str,
        jwt_version: i32,
    ) -> Result<bool, redis::RedisError> {
        let mut conn = self.client.get_async_connection().await?;

        // Check version first
        let version_key = format!("user:{}:perm_version", user_id);
        let cached_version: Option<String> = conn.get(&version_key).await?;

        match cached_version {
            Some(v) => {
                let cached: i32 = v.parse().unwrap_or(0);
                if cached != jwt_version {
                    return Ok(false); // Stale cache
                }
            }
            None => return Ok(false), // Cache miss
        }

        // Check permission
        let key = format!("user:{}:permissions", user_id);
        let exists: bool = conn.sismember(&key, permission).await?;
        Ok(exists)
    }

    /// Invalidate user permissions cache
    pub async fn invalidate_user(&self, user_id: &str) -> Result<(), redis::RedisError> {
        let mut conn = self.client.get_async_connection().await?;

        conn.del(format!("user:{}:permissions", user_id)).await?;
        conn.del(format!("user:{}:perm_version", user_id)).await?;

        Ok(())
    }

    /// Invalidate role permissions cache
    pub async fn invalidate_role(&self, role_name: &str) -> Result<(), redis::RedisError> {
        let mut conn = self.client.get_async_connection().await?;
        conn.del(format!("role:{}:permissions", role_name)).await?;
        Ok(())
    }
}
```

**Step 4: Run test to verify it passes**

```bash
cd mmo-api && cargo test cache_and_retrieve_permissions check_permission_with_version
```

Expected: PASS

**Step 5: Commit**

```bash
git add mmo-api/src/modules/permissions/cache.rs
git commit -m "feat: implement Redis permission cache with version validation"
```

---

### Task 6: Create Lua Script for Atomic Permission Check

**Files:**
- Create: `mmo-api/src/scripts/check_permission.lua`
- Modify: `mmo-api/src/modules/permissions/cache.rs`

**Step 1: Create Lua script**

In `mmo-api/src/scripts/check_permission.lua`:

```lua
-- check_permission.lua
-- KEYS[1] = user:{user_id}:permissions (SET)
-- KEYS[2] = user:{user_id}:perm_version (STRING)
-- ARGV[1] = required_permission (STRING)
-- ARGV[2] = jwt_perm_version (NUMBER)
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

**Step 2: Load Lua script in cache module**

Add to `mmo-api/src/modules/permissions/cache.rs`:

```rust
impl PermissionCache {
    pub async fn check_permission_atomic(
        &self,
        user_id: &str,
        permission: &str,
        jwt_version: i32,
    ) -> Result<i32, redis::RedisError> {
        let mut conn = self.client.get_async_connection().await?;

        // Load Lua script
        let script = redis::Script::new(include_str!("../../scripts/check_permission.lua"));

        let result: i32 = script
            .key(format!("user:{}:permissions", user_id))
            .key(format!("user:{}:perm_version", user_id))
            .arg(permission)
            .arg(jwt_version)
            .invoke_async(&mut conn)
            .await?;

        Ok(result) // 1 = authorized, 0 = denied, -1 = cache miss
    }
}
```

**Step 3: Commit**

```bash
git add mmo-api/src/scripts/ mmo-api/src/modules/permissions/cache.rs
git commit -m "feat: add Lua script for atomic permission checks"
```

---

## Phase 3: Authorization Middleware Integration

### Task 7: Implement Permission Extractor for actix-web-grants

**Files:**
- Create: `mmo-api/src/middleware/permission_extractor.rs`
- Modify: `mmo-api/src/middleware/mod.rs`

**Step 1: Write tests for permission extractor**

Create `mmo-api/src/middleware/permission_extractor_tests.rs`:

```rust
use super::*;

#[actix_web::test]
async fn test_extract_permissions_from_auth_user() {
    let permissions = extract_permissions(&AuthUser {
        user_id: "test123".to_string(),
        roles: vec!["buyer".to_string(), "seller".to_string()],
        perm_version: 5,
    }).await;

    assert!(permissions.contains(&"products:read".to_string()));
    assert!(permissions.contains(&"products:create".to_string()));
}
```

**Step 2: Run test to verify it fails**

```bash
cd mmo-api && cargo test extract_permissions
```

Expected: FAIL - extractor doesn't exist

**Step 3: Implement permission extractor**

In `mmo-api/src/middleware/permission_extractor.rs`:

```rust
use actix_web::{dev::Payload, Error, FromRequest, HttpRequest};
use actix_web_grants::permissions::{PermissionsCheck, PermissionsStorage};
use futures::future::{ready, Ready};
use std::sync::Arc;

use crate::modules::permissions::{cache::PermissionCache, repository::RoleRepository};
use crate::database::mongodb::MongoDB;

/// Authenticated user with role information
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub roles: Vec<String>,
    pub perm_version: u32,
}

impl FromRequest for AuthUser {
    type Error = Error;
    type Future = Ready<Result<Self, Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        // Extract from JWT middleware (to be implemented)
        ready(Ok(AuthUser {
            user_id: "test".to_string(), // Placeholder
            roles: vec!["buyer".to_string()],
            perm_version: 1,
        }))
    }
}

/// Permission extractor that integrates with actix-web-grants
pub async fn extract_permissions(
    auth_user: &AuthUser,
) -> Vec<String> {
    // Try cache first, fallback to DB
    let cache = PermissionCache::new("redis://localhost:6379").await.unwrap();

    match cache.get_permissions(&auth_user.user_id).await {
        Ok(permissions) if !permissions.is_empty() => permissions,
        _ => {
            // Cache miss - fetch from DB
            let db = MongoDB::new("mongodb://localhost:27017/test").await.unwrap();
            let repo = RoleRepository::new(db.database());

            match repo.get_user_permissions(&auth_user.user_id).await {
                Ok(user_perms) => {
                    // Update cache
                    let _ = cache.set_permissions(
                        &auth_user.user_id,
                        &user_perms.effective_permissions,
                        auth_user.perm_version as i32,
                    ).await;
                    user_perms.effective_permissions
                }
                Err(_) => vec![],
            }
        }
    }
}
```

**Step 4: Update AuthUser in auth.rs to match new structure**

In `mmo-api/src/middleware/auth.rs`, update `AuthUser` struct:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub user_id: String,
    pub roles: Vec<String>,      // Changed from single role
    pub perm_version: u32,       // Added for cache invalidation
}
```

**Step 5: Update AuthMiddleware to extract from new JWT**

```rust
impl AuthMiddleware {
    pub async fn validate_token(&self, token: &str) -> Result<AuthUser, ApiError> {
        let claims = decode_jwt(token)?;

        Ok(AuthUser {
            user_id: claims.sub,
            roles: claims.roles,
            perm_version: claims.perm_version,
        })
    }
}
```

**Step 6: Run test to verify it passes**

```bash
cd mmo-api && cargo test extract_permissions
```

Expected: PASS

**Step 7: Commit**

```bash
git add mmo-api/src/middleware/auth.rs mmo-api/src/middleware/permission_extractor.rs
git commit -m "feat: implement permission extractor for actix-web-grants integration"
```

---

### Task 8: Configure Grants Middleware in main.rs

**Files:**
- Modify: `mmo-api/src/main.rs`

**Step 1: Add grants middleware to application**

```rust
use actix_web_grants::GrantsMiddleware;
use crate::middleware::permission_extractor::{extract_permissions, AuthUser};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // ... existing setup ...

    HttpServer::new(move || {
        App::new()
            // ... existing middleware ...
            .wrap(GrantsMiddleware::with_extractor(extract_permissions))
            .configure(routes::configure)
    })
    .bind(&config.server_addr)?
    .run()
    .await
}
```

**Step 2: Commit**

```bash
git add mmo-api/src/main.rs
git commit -m "feat: integrate actix-web-grants middleware"
```

---

## Phase 4: Update Handler Protection

### Task 9: Add Permission Guards to Existing Handlers

**Files:**
- Modify: `mmo-api/src/modules/wallet/handler.rs`
- Modify: `mmo-api/src/modules/auth/handler.rs`

**Step 1: Update wallet handlers with permission guards**

In `mmo-api/src/modules/wallet/handler.rs`:

```rust
use actix_web_grants::protect;

/// Get wallet balance - any authenticated user
#[get("/wallet/balance")]
#[protect("wallet:read")]
pub async fn get_balance(
    service: web::Data<Arc<WalletService>>,
    auth: AuthUser,
) -> Result<HttpResponse, ApiError> {
    let result = service.get_balance(&auth.user_id).await?;
    Ok(HttpResponse::Ok().json(result))
}

/// Manual debit - admin only
#[post("/wallet/admin/debit")]
#[protect("wallet:manage")]
pub async fn manual_debit(
    service: web::Data<Arc<WalletService>>,
    auth: AuthUser,
    req: web::Json<ManualDebitRequest>,
) -> Result<HttpResponse, ApiError> {
    let result = service.manual_debit(&req.user_id, req.amount, &req.reason).await?;
    Ok(HttpResponse::Ok().json(result))
}

/// Approve withdrawal - admin or moderator
#[post("/withdrawals/{id}/approve")]
#[protect(any("wallet:approve", "wallet:manage"))]
pub async fn approve_withdrawal(
    service: web::Data<Arc<WalletService>>,
    auth: AuthUser,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let withdrawal_id = path.into_inner();
    let result = service.approve_withdrawal(&withdrawal_id, &auth.user_id).await?;
    Ok(HttpResponse::Ok().json(result))
}
```

**Step 2: Update auth handlers**

In `mmo-api/src/modules/auth/handler.rs`:

```rust
/// Assign role to user - admin only
#[post("/users/{id}/roles")]
#[protect("users:manage_roles")]
pub async fn assign_role(
    service: web::Data<Arc<AuthService>>,
    auth: AuthUser,
    path: web::Path<String>,
    req: web::Json<AssignRoleRequest>,
) -> Result<HttpResponse, ApiError> {
    let user_id = path.into_inner();
    let result = service.assign_role(&user_id, &req.role_name, &auth.user_id).await?;
    Ok(HttpResponse::Ok().json(result))
}

/// Get user permissions
#[get("/users/{id}/permissions")]
#[protect("users:read")]
pub async fn get_user_permissions(
    service: web::Data<Arc<AuthService>>,
    auth: AuthUser,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let user_id = path.into_inner();
    let result = service.get_user_permissions(&user_id).await?;
    Ok(HttpResponse::Ok().json(result))
}
```

**Step 3: Commit**

```bash
git add mmo-api/src/modules/wallet/handler.rs mmo-api/src/modules/auth/handler.rs
git commit -m "feat: add permission guards to wallet and auth handlers"
```

---

### Task 10: Implement Ownership Check Pattern

**Files:**
- Create: `mmo-api/src/modules/ownership.rs`
- Modify: `mmo-api/src/modules/products/handler.rs` (when products module exists)

**Step 1: Create ownership check utility**

In `mmo-api/src/modules/ownership.rs`:

```rust
use actix_web::{error::ErrorUnauthorized, Error};
use crate::middleware::permission_extractor::AuthUser;

/// Check if user owns resource or is admin
pub async fn check_ownership_or_admin(
    resource_owner_id: &str,
    auth_user: &AuthUser,
) -> Result<(), Error> {
    // Admin bypass
    if auth_user.roles.contains(&"admin".to_string())
        || auth_user.roles.contains(&"super_admin".to_string())
    {
        return Ok(());
    }

    // Check ownership
    if resource_owner_id != auth_user.user_id {
        return Err(ErrorUnauthorized("OWNERSHIP_REQUIRED"));
    }

    Ok(())
}

/// Check if user can modify resource (owns it OR has manage permission)
pub async fn can_modify_resource(
    resource_owner_id: &str,
    auth_user: &AuthUser,
    required_permission: &str,
) -> Result<(), Error> {
    // Check permission first
    let has_permission = check_user_permission(auth_user, required_permission).await;

    if has_permission {
        return Ok(());
    }

    // Fallback to ownership check
    check_ownership_or_admin(resource_owner_id, auth_user).await
}

async fn check_user_permission(auth_user: &AuthUser, permission: &str) -> bool {
    // Use permission cache or DB to check
    // Implementation depends on extract_permissions
    true // Placeholder
}
```

**Step 2: Example usage in product handler**

```rust
#[put("/products/{id}")]
#[protect("products:update")]
pub async fn update_product(
    service: web::Data<Arc<ProductService>>,
    auth: AuthUser,
    path: web::Path<String>,
    req: web::Json<UpdateProductRequest>,
) -> Result<HttpResponse, Error> {
    let product_id = path.into_inner();

    // Get product to check ownership
    let product = service.get_product(&product_id).await?;

    // Verify ownership or admin
    check_ownership_or_admin(&product.owner_id, &auth).await?;

    // Proceed with update
    let result = service.update_product(&product_id, req.into_inner()).await?;
    Ok(HttpResponse::Ok().json(result))
}
```

**Step 3: Commit**

```bash
git add mmo-api/src/modules/ownership.rs
git commit -m "feat: implement ownership check pattern for resource access control"
```

---

## Phase 5: Database Migration & Seeding

### Task 11: Create Database Migration Script

**Files:**
- Create: `mmo-api/migrations/001_create_permissions_collections.rs`

**Step 1: Create permission indexes**

```rust
use mongodb::{
    bson::doc,
    Client,
    IndexModel,
    options::IndexOptions,
};

pub async fn up(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let db = client.database("mmo");

    // Create indexes for permissions collection
    let perms = db.collection("permissions");

    let indexes = vec![
        IndexModel::builder()
            .keys(doc! { "name": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build(),
        IndexModel::builder()
            .keys(doc! { "resource": 1, "action": 1 })
            .build(),
        IndexModel::builder()
            .keys(doc! { "category": 1, "is_active": 1 })
            .build(),
    ];

    perms.create_indexes(indexes, None).await?;

    // Create indexes for roles collection
    let roles = db.collection("roles");

    let role_indexes = vec![
        IndexModel::builder()
            .keys(doc! { "name": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build(),
        IndexModel::builder()
            .keys(doc! { "level": -1 })
            .build(),
    ];

    roles.create_indexes(role_indexes, None).await?;

    println!("Migration 001: Created permissions and roles indexes");
    Ok(())
}
```

**Step 2: Create seeding script**

Create `mmo-api/migrations/seed_permissions.rs`:

```rust
use mongodb::Client;
use crate::modules::permissions::domain::Permission;

pub async fn seed_default_permissions(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let db = client.database("mmo");
    let perms = db.collection::<Permission>("permissions");

    let default_permissions = vec![
        // Products
        Permission {
            id: None,
            name: "products:read".into(),
            display_name: "View Products".into(),
            description: "View product listings and details".into(),
            resource: "products".into(),
            action: "read".into(),
            category: "marketplace".into(),
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        Permission {
            id: None,
            name: "products:create".into(),
            display_name: "Create Products".into(),
            description: "Create new product listings".into(),
            resource: "products".into(),
            action: "create".into(),
            category: "marketplace".into(),
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        // ... more permissions
    ];

    perms.insert_many(default_permissions, None).await?;

    println!("Seeded default permissions");
    Ok(())
}
```

**Step 3: Commit**

```bash
git add mmo-api/migrations/
git commit -m "feat: add database migration scripts for authorization system"
```

---

### Task 12: Create Default Roles and Permissions

**Files:**
- Create: `mmo-api/migrations/seed_roles.rs`

**Step 1: Seed default roles**

```rust
use mongodb::{Client, bson::oid::ObjectId};
use crate::modules::permissions::domain::Role;

pub async fn seed_default_roles(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let db = client.database("mmo");
    let roles = db.collection::<Role>("roles");

    let buyer_role = Role {
        id: None,
        name: "buyer".into(),
        display_name: "Buyer".into(),
        level: 0,
        parent_role_id: None,
        inherits_from: vec![],
        direct_permissions: vec![],
        flattened_permissions: vec![
            "products:read".into(),
            "orders:create".into(),
            "orders:read".into(),
        ],
        is_system: true,
        is_active: true,
        version: 1,
    };

    let seller_role = Role {
        id: None,
        name: "seller".into(),
        display_name: "Seller".into(),
        level: 1,
        parent_role_id: None, // Will be set after buyer is inserted
        inherits_from: vec!["buyer".into()],
        direct_permissions: vec![],
        flattened_permissions: vec![
            "products:read".into(),
            "products:create".into(),
            "products:update".into(),
            "products:delete".into(),
            "orders:read".into(),
            "shops:create".into(),
        ],
        is_system: true,
        is_active: true,
        version: 1,
    };

    let admin_role = Role {
        id: None,
        name: "admin".into(),
        display_name: "Administrator".into(),
        level: 2,
        parent_role_id: None,
        inherits_from: vec!["seller".into(), "moderator".into()],
        direct_permissions: vec![],
        flattened_permissions: vec!["*".into()], // All permissions
        is_system: true,
        is_active: true,
        version: 1,
    };

    roles.insert_many(vec![buyer_role, seller_role, admin_role], None).await?;

    println!("Seeded default roles: buyer, seller, admin");
    Ok(())
}
```

**Step 2: Commit**

```bash
git add mmo-api/migrations/seed_roles.rs
git commit -m "feat: seed default roles with hierarchy"
```

---

## Phase 6: Cleanup & Documentation

### Task 13: Remove Legacy Authorization Code

**Files:**
- Modify: `mmo-api/src/middleware/authorization.rs` (deprecate)
- Modify: All handler files using AdminUser

**Step 1: Deprecate old RequireRole middleware**

```rust
// In mmo-api/src/middleware/authorization.rs

#[deprecated(since = "2.0", note = "Use #[protect()] macro instead")]
pub struct RequireRole {
    // Keep for backward compatibility but mark as deprecated
}
```

**Step 2: Replace AdminUser with AuthUser + permission guard**

Find all uses of `AdminUser` extractor and replace with `AuthUser` + `#[protect("...")]`:

```rust
// Before
pub async fn admin_handler(admin: AdminUser) -> Result<HttpResponse, ApiError> {

// After
#[protect("admin:action")]
pub async fn admin_handler(auth: AuthUser) -> Result<HttpResponse, ApiError> {
```

**Step 3: Commit**

```bash
git add mmo-api/src/middleware/authorization.rs
git commit -m "deprecate: remove legacy RequireRole middleware in favor of permission guards"
```

---

### Task 14: Update Documentation

**Files:**
- Create: `.claude/context/permissions.md`
- Update: `docs/ARCHITECTURE.md`

**Step 1: Create permissions context**

Create `.claude/context/permissions.md`:

```markdown
# Permissions Module Context

## Overview
Dynamic RBAC + ABAC authorization system with Redis caching.

## Key Concepts

### Permission Format
`resource:action` (e.g., `products:create`, `orders:read`)

### Role Hierarchy
```
super_admin (level 3)
    inherits: admin, moderator, seller, buyer
admin (level 2)
    inherits: moderator, seller, buyer
seller (level 1)
    inherits: buyer
buyer (level 0)
    base role
```

### Permission Resolution
1. JWT contains roles + perm_version
2. Redis cache checked first (using Lua script)
3. MongoDB fallback on cache miss
4. Ownership checks for resource-level access

## Usage

### Adding Permission to Handler
```rust
#[protect("products:read")]
pub async fn list_products() -> HttpResponse { }
```

### Ownership Check
```rust
check_ownership_or_admin(&resource.owner_id, &auth).await?;
```

## Files
- Domain: `src/modules/permissions/domain.rs`
- Repository: `src/modules/permissions/repository.rs`
- Cache: `src/modules/permissions/cache.rs`
- Middleware: `src/middleware/permission_extractor.rs`
```

**Step 2: Update architecture docs**

Add to `docs/ARCHITECTURE.md`:

```markdown
## Authorization System

The P2PMMO uses a hybrid RBAC + ABAC system:

- Dynamic permissions stored in MongoDB
- Role hierarchy with inheritance
- Redis caching for <5ms permission checks
- Ownership-based resource access control

See [docs/v2/P2P_MMO_Authorization_System_Documentation.md](../v2/P2P_MMO_Authorization_System_Documentation.md) for full details.
```

**Step 3: Commit**

```bash
git add .claude/context/permissions.md docs/ARCHITECTURE.md
git commit -m "docs: add permissions module documentation"
```

---

## Summary

This plan implements a complete authorization system migration:

1. **JWT Structure**: Single role → Multiple roles array + perm_version
2. **Database Schema**: New permissions and roles collections with hierarchy
3. **Caching Layer**: Redis with atomic Lua script checks
4. **Middleware**: actix-web-grants with #[protect()] macros
5. **Handler Guards**: Permission-based instead of role-based
6. **Ownership Pattern**: Resource-level access control
7. **Migration**: Database scripts for seeding default data

**Estimated tasks**: 14 major tasks
**Estimated files created**: ~15 new files
**Estimated files modified**: ~10 existing files

---

*For execution, use superpowers:executing-plans skill*
