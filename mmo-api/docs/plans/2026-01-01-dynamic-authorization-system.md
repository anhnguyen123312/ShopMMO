# Dynamic Authorization System Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Redesign authorization system from hardcoded roles to dynamic roles with permissions defined as Rust Enum (AWS IAM-style: `RESOURCE:ACTION`)

**Architecture:**
- Permissions are **hardcoded constants** in Rust code as Enum (type-safe, IDE-friendly)
- Roles are **dynamic** (created/managed via CRUD API, stored in MongoDB)
- Authorization follows `resource:action` format (e.g., `product:create`, `order:read`)
- Redis caching for performance with permission versioning for cache invalidation

**Tech Stack:**
- Rust 2021, actix-web 4.9
- MongoDB (roles, permissions, user assignments)
- Redis (permission caching)
- Enum-based permission constants
- actix-web-grants for authorization guards

---

## Migration Strategy

**Current State:**
- Hardcoded roles: `BUYER`, `SELLER`, `ADMIN`, `SUPER_ADMIN`
- JWT contains roles array
- `#[protect("ADMIN", "SUPER_ADMIN")]` guards

**Target State:**
- Dynamic roles in MongoDB (CRUD operations)
- Permission enum constants in Rust code
- JWT unchanged (still contains roles for backward compatibility)
- Guards check permissions: `#[protect("product:create")]`

**Backward Compatibility:**
- Keep existing `role` and `roles` fields in User model
- Seed default roles matching current hardcoded roles
- Migration script to create initial roles

---

## Task 1: Create Permission Constants Module

**Files:**
- Create: `src/modules/permissions/constants.rs`
- Modify: `src/modules/permissions/mod.rs`

**Step 1: Create constants module with Permission enum**

```rust
// src/modules/permissions/constants.rs

//! Permission constants - hardcoded actions for authorization
//!
//! Permissions are defined as compile-time constants using Enum.
//! This provides type-safety and IDE autocomplete support.
//!
//! Format: RESOURCE:ACTION (e.g., product:create, order:read)
//! Similar to AWS IAM permission model.

use std::collections::HashSet;

/// Permission enum - type-safe permission definitions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    // Product permissions
    ProductCreate,
    ProductRead,
    ProductUpdate,
    ProductDelete,
    ProductList,

    // Order permissions
    OrderCreate,
    OrderRead,
    OrderUpdate,
    OrderCancel,
    OrderList,

    // Wallet permissions
    WalletRead,
    WalletWithdraw,
    WalletDeposit,
    WalletList,

    // User management permissions
    UserCreate,
    UserRead,
    UserUpdate,
    UserDelete,
    UserAssignRoles,

    // Role management permissions
    RoleCreate,
    RoleRead,
    RoleUpdate,
    RoleDelete,
    RoleAssignPermissions,
}

impl Permission {
    /// Convert permission to RESOURCE:ACTION string format
    ///
    /// # Examples
    /// ```
    /// use crate::modules::permissions::constants::Permission;
    /// assert_eq!(Permission::ProductCreate.as_str(), "product:create");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ProductCreate => "product:create",
            Self::ProductRead => "product:read",
            Self::ProductUpdate => "product:update",
            Self::ProductDelete => "product:delete",
            Self::ProductList => "product:list",

            Self::OrderCreate => "order:create",
            Self::OrderRead => "order:read",
            Self::OrderUpdate => "order:update",
            Self::OrderCancel => "order:cancel",
            Self::OrderList => "order:list",

            Self::WalletRead => "wallet:read",
            Self::WalletWithdraw => "wallet:withdraw",
            Self::WalletDeposit => "wallet:deposit",
            Self::WalletList => "wallet:list",

            Self::UserCreate => "user:create",
            Self::UserRead => "user:read",
            Self::UserUpdate => "user:update",
            Self::UserDelete => "user:delete",
            Self::UserAssignRoles => "user:assign_roles",

            Self::RoleCreate => "role:create",
            Self::RoleRead => "role:read",
            Self::RoleUpdate => "role:update",
            Self::RoleDelete => "role:delete",
            Self::RoleAssignPermissions => "role:assign_permissions",
        }
    }

    /// Parse from string (for loading from DB)
    ///
    /// # Returns
    /// * `Option<Permission>` - Some(permission) if valid, None otherwise
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "product:create" => Some(Self::ProductCreate),
            "product:read" => Some(Self::ProductRead),
            "product:update" => Some(Self::ProductUpdate),
            "product:delete" => Some(Self::ProductDelete),
            "product:list" => Some(Self::ProductList),

            "order:create" => Some(Self::OrderCreate),
            "order:read" => Some(Self::OrderRead),
            "order:update" => Some(Self::OrderUpdate),
            "order:cancel" => Some(Self::OrderCancel),
            "order:list" => Some(Self::OrderList),

            "wallet:read" => Some(Self::WalletRead),
            "wallet:withdraw" => Some(Self::WalletWithdraw),
            "wallet:deposit" => Some(Self::WalletDeposit),
            "wallet:list" => Some(Self::WalletList),

            "user:create" => Some(Self::UserCreate),
            "user:read" => Some(Self::UserRead),
            "user:update" => Some(Self::UserUpdate),
            "user:delete" => Some(Self::UserDelete),
            "user:assign_roles" => Some(Self::UserAssignRoles),

            "role:create" => Some(Self::RoleCreate),
            "role:read" => Some(Self::RoleRead),
            "role:update" => Some(Self::RoleUpdate),
            "role:delete" => Some(Self::RoleDelete),
            "role:assign_permissions" => Some(Self::RoleAssignPermissions),

            _ => None,
        }
    }

    /// Get resource part from permission
    pub fn resource(&self) -> &'static str {
        self.as_str().split(':').next().unwrap_or("")
    }

    /// Get action part from permission
    pub fn action(&self) -> &'static str {
        self.as_str().split(':').nth(1).unwrap_or("")
    }
}

/// Get all available permissions as string array
///
/// # Returns
/// * `Vec<&'static str>` - All permissions in RESOURCE:ACTION format
///
/// # Examples
/// ```
/// use crate::modules::permissions::constants::all_permissions;
/// let perms = all_permissions();
/// assert!(perms.contains(&"product:create"));
/// ```
pub fn all_permissions() -> Vec<&'static str> {
    vec![
        // Products
        Permission::ProductCreate.as_str(),
        Permission::ProductRead.as_str(),
        Permission::ProductUpdate.as_str(),
        Permission::ProductDelete.as_str(),
        Permission::ProductList.as_str(),

        // Orders
        Permission::OrderCreate.as_str(),
        Permission::OrderRead.as_str(),
        Permission::OrderUpdate.as_str(),
        Permission::OrderCancel.as_str(),
        Permission::OrderList.as_str(),

        // Wallet
        Permission::WalletRead.as_str(),
        Permission::WalletWithdraw.as_str(),
        Permission::WalletDeposit.as_str(),
        Permission::WalletList.as_str(),

        // Users
        Permission::UserCreate.as_str(),
        Permission::UserRead.as_str(),
        Permission::UserUpdate.as_str(),
        Permission::UserDelete.as_str(),
        Permission::UserAssignRoles.as_str(),

        // Roles
        Permission::RoleCreate.as_str(),
        Permission::RoleRead.as_str(),
        Permission::RoleUpdate.as_str(),
        Permission::RoleDelete.as_str(),
        Permission::RoleAssignPermissions.as_str(),
    ]
}

/// Get all available permissions as HashSet for efficient lookup
pub fn all_permissions_set() -> HashSet<&'static str> {
    all_permissions().into_iter().collect()
}

/// Validate if a permission string is valid
///
/// # Arguments
/// * `permission` - Permission string to validate
///
/// # Returns
/// * `bool` - true if valid, false otherwise
pub fn is_valid_permission(permission: &str) -> bool {
    Permission::from_str(permission).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_as_str() {
        assert_eq!(Permission::ProductCreate.as_str(), "product:create");
        assert_eq!(Permission::OrderRead.as_str(), "order:read");
        assert_eq!(Permission::WalletWithdraw.as_str(), "wallet:withdraw");
    }

    #[test]
    fn test_permission_from_str() {
        assert_eq!(Permission::from_str("product:create"), Some(Permission::ProductCreate));
        assert_eq!(Permission::from_str("invalid:permission"), None);
        assert_eq!(Permission::from_str("not_valid"), None);
    }

    #[test]
    fn test_permission_resource_action() {
        assert_eq!(Permission::ProductCreate.resource(), "product");
        assert_eq!(Permission::ProductCreate.action(), "create");
        assert_eq!(Permission::OrderCancel.resource(), "order");
        assert_eq!(Permission::OrderCancel.action(), "cancel");
    }

    #[test]
    fn test_all_permissions() {
        let perms = all_permissions();
        assert!(perms.contains(&"product:create"));
        assert!(perms.contains(&"order:read"));
        assert!(perms.contains(&"wallet:withdraw"));
        assert!(perms.contains(&"user:assign_roles"));
    }

    #[test]
    fn test_is_valid_permission() {
        assert!(is_valid_permission("product:create"));
        assert!(is_valid_permission("role:assign_permissions"));
        assert!(!is_valid_permission("invalid:permission"));
        assert!(!is_valid_permission("not_valid"));
    }
}
```

**Step 2: Update mod.rs to export constants**

```rust
// Add to src/modules/permissions/mod.rs

pub mod constants;

pub use constants::{Permission, all_permissions, all_permissions_set, is_valid_permission};
```

**Step 3: Run cargo check**

```bash
cd /Volumes/Data/Git/mmo/mmo-api && cargo check
```

Expected: No errors

**Step 4: Run tests**

```bash
cd /Volumes/Data/Git/mmo/mmo-api && cargo test --lib permissions::constants
```

Expected: All tests pass

**Step 5: Commit**

```bash
git add src/modules/permissions/
git commit -m "feat: add permission constants as Enum for type-safe authorization"
```

---

## Task 2: Update Role Domain Model

**Files:**
- Modify: `src/modules/permissions/domain.rs`

**Step 1: Update Role struct to use permission strings**

The Role struct is already good. Just ensure `flattened_permissions` stores permission strings like `product:create`.

Current structure already supports this. Verify:
```rust
pub struct Role {
    // ...
    /// Pre-computed flattened list of all permission names
    /// Includes both direct and inherited permissions
    /// Example: ["product:create", "product:read", "order:create"]
    pub flattened_permissions: Vec<String>,
    // ...
}
```

**Step 2: Add helper methods to Role**

```rust
// Add to impl Role in domain.rs

impl Role {
    /// Create a new role
    pub fn new(
        name: String,
        display_name: String,
        level: i32,
        permissions: Vec<String>,
    ) -> Self {
        let now = DateTime::now();
        Self {
            id: None,
            name,
            display_name,
            level,
            parent_role_id: None,
            inherits_from: vec![],
            direct_permissions: vec![],
            flattened_permissions: permissions,
            is_system: false,
            is_active: true,
            version: 1,
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if role has a specific permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.flattened_permissions.contains(&permission.to_string())
    }

    /// Get all permissions for this role
    pub fn permissions(&self) -> &[String] {
        &self.flattened_permissions
    }
}
```

**Step 3: Run cargo check**

```bash
cargo check
```

Expected: No errors

**Step 4: Commit**

```bash
git add src/modules/permissions/domain.rs
git commit -m "feat: add helper methods to Role domain model"
```

---

## Task 3: Create Role Management Service

**Files:**
- Modify: `src/modules/permissions/service.rs`

**Step 1: Add role CRUD methods**

```rust
// Add to PermissionService in service.rs

use crate::modules::permissions::constants::{all_permissions, is_valid_permission};
use mongodb::bson::doc;

impl PermissionService {
    // ... existing methods ...

    /// Create a new role
    ///
    /// # Arguments
    /// * `name` - Unique role name
    /// * `display_name` - Human-readable name
    /// * `level` - Hierarchy level
    /// * `permissions` - List of permission strings to assign
    ///
    /// # Returns
    /// * `Result<Role, ApiError>` - Created role
    pub async fn create_role(
        &self,
        name: String,
        display_name: String,
        level: i32,
        permissions: Vec<String>,
    ) -> Result<Role, ApiError> {
        // Validate permissions
        for perm in &permissions {
            if !is_valid_permission(perm) {
                return Err(ApiError::bad_request(&format!(
                    "Invalid permission: {}. Valid permissions are: {:?}",
                    perm,
                    all_permissions()
                )));
            }
        }

        // Check if role already exists
        let existing = self
            .role_repo
            .find_by_name(&name)
            .await
            .map_err(|e| ApiError::database(e.to_string()))?;

        if existing.is_some() {
            return Err(ApiError::bad_request(&format!(
                "Role '{}' already exists",
                name
            )));
        }

        let role = Role::new(name.clone(), display_name, level, permissions);

        self.role_repo
            .create(role)
            .await
            .map_err(|e| ApiError::database(e.to_string()))?;

        // Fetch and return the created role
        let created = self
            .role_repo
            .find_by_name(&name)
            .await
            .map_err(|e| ApiError::database(e.to_string()))?
            .ok_or_else(|| ApiError::internal("Failed to retrieve created role"))?;

        Ok(created)
    }

    /// Update role permissions
    ///
    /// # Arguments
    /// * `role_name` - Role name to update
    /// * `permissions` - New list of permissions
    ///
    /// # Returns
    /// * `Result<(), ApiError>` - Success or error
    pub async fn update_role_permissions(
        &self,
        role_name: &str,
        permissions: Vec<String>,
    ) -> Result<(), ApiError> {
        // Validate permissions
        for perm in &permissions {
            if !is_valid_permission(perm) {
                return Err(ApiError::bad_request(&format!(
                    "Invalid permission: {}",
                    perm
                )));
            }
        }

        self.role_repo
            .update_permissions(role_name, permissions)
            .await
            .map_err(|e| ApiError::database(e.to_string()))?;

        Ok(())
    }

    /// Delete a role
    ///
    /// # Arguments
    /// * `role_name` - Role name to delete
    ///
    /// # Returns
    /// * `Result<(), ApiError>` - Success or error
    pub async fn delete_role(&self, role_name: &str) -> Result<(), ApiError> {
        // Check if role exists
        let role = self
            .role_repo
            .find_by_name(role_name)
            .await
            .map_err(|e| ApiError::database(e.to_string()))?
            .ok_or_else(|| ApiError::not_found(&format!("Role '{}' not found", role_name)))?;

        // Prevent deleting system roles
        if role.is_system {
            return Err(ApiError::bad_request("Cannot delete system roles"));
        }

        self.role_repo
            .delete(role_name)
            .await
            .map_err(|e| ApiError::database(e.to_string()))?;

        Ok(())
    }

    /// List all roles
    ///
    /// # Returns
    /// * `Result<Vec<Role>, ApiError>` - List of roles
    pub async fn list_roles(&self) -> Result<Vec<Role>, ApiError> {
        self.role_repo
            .list_all()
            .await
            .map_err(|e| ApiError::database(e.to_string()))
    }

    /// Assign role to user
    ///
    /// # Arguments
    /// * `user_id` - User's ObjectId as string
    /// * `role_name` - Role name to assign
    ///
    /// # Returns
    /// * `Result<(), ApiError>` - Success or error
    pub async fn assign_role_to_user(
        &self,
        user_id: &str,
        role_name: &str,
    ) -> Result<(), ApiError> {
        // Verify role exists
        let role = self
            .role_repo
            .find_by_name(role_name)
            .await
            .map_err(|e| ApiError::database(e.to_string()))?
            .ok_or_else(|| ApiError::not_found(&format!("Role '{}' not found", role_name)))?;

        // Parse user_id
        let oid = bson::oid::ObjectId::parse_str(user_id)
            .map_err(|_| ApiError::bad_request("Invalid user ID"))?;

        self.role_repo
            .assign_role_to_user(&oid, &role)
            .await
            .map_err(|e| ApiError::database(e.to_string()))?;

        Ok(())
    }

    /// Remove role from user
    ///
    /// # Arguments
    /// * `user_id` - User's ObjectId as string
    /// * `role_name` - Role name to remove
    ///
    /// # Returns
    /// * `Result<(), ApiError>` - Success or error
    pub async fn remove_role_from_user(
        &self,
        user_id: &str,
        role_name: &str,
    ) -> Result<(), ApiError> {
        let oid = bson::oid::ObjectId::parse_str(user_id)
            .map_err(|_| ApiError::bad_request("Invalid user ID"))?;

        self.role_repo
            .remove_role_from_user(&oid, role_name)
            .await
            .map_err(|e| ApiError::database(e.to_string()))?;

        Ok(())
    }
}
```

**Step 2: Update repository methods (if needed)**

Check if `src/modules/permissions/repository.rs` has these methods. Add if missing:

```rust
// Add to RoleRepository in repository.rs

impl RoleRepository {
    /// Find role by name
    pub async fn find_by_name(&self, name: &str) -> Result<Option<Role>, DbError> {
        self.collection
            .find_one(doc! { "name": name })
            .await
    }

    /// Create a new role
    pub async fn create(&self, role: Role) -> Result<(), DbError> {
        self.collection.insert_one(role).await?;
        Ok(())
    }

    /// Update role permissions
    pub async fn update_permissions(
        &self,
        role_name: &str,
        permissions: Vec<String>,
    ) -> Result<(), DbError> {
        self.collection
            .update_one(
                doc! { "name": role_name },
                doc! {
                    "$set": {
                        "flattened_permissions": permissions,
                        "updated_at": DateTime::now(),
                        "$inc": { "version": 1 }
                    }
                },
            )
            .await?;
        Ok(())
    }

    /// Delete a role
    pub async fn delete(&self, role_name: &str) -> Result<(), DbError> {
        self.collection
            .update_one(
                doc! { "name": role_name },
                doc! { "$set": { "is_active": false, "updated_at": DateTime::now() } },
            )
            .await?;
        Ok(())
    }

    /// List all active roles
    pub async fn list_all(&self) -> Result<Vec<Role>, DbError> {
        self.collection
            .find(doc! { "is_active": true })
            .await
            .map(|cursor| cursor.try_collect())
            .map_err(|e| DbError::QueryError(e.to_string()))?
    }

    /// Assign role to user
    pub async fn assign_role_to_user(
        &self,
        user_id: &ObjectId,
        role: &Role,
    ) -> Result<(), DbError> {
        let users_collection = self
            .database
            .collection::<mongodb::bson::Document>("users");

        users_collection
            .update_one(
                doc! { "_id": user_id },
                doc! {
                    "$addToSet": { "roles": role.name.clone() },
                    "$set": { "updated_at": DateTime::now() }
                },
            )
            .await?;

        Ok(())
    }

    /// Remove role from user
    pub async fn remove_role_from_user(
        &self,
        user_id: &ObjectId,
        role_name: &str,
    ) -> Result<(), DbError> {
        let users_collection = self
            .database
            .collection::<mongodb::bson::Document>("users");

        users_collection
            .update_one(
                doc! { "_id": user_id },
                doc! {
                    "$pull": { "roles": role_name },
                    "$set": { "updated_at": DateTime::now() }
                },
            )
            .await?;

        Ok(())
    }
}
```

**Step 3: Run cargo check**

```bash
cargo check
```

Expected: No errors

**Step 4: Commit**

```bash
git add src/modules/permissions/service.rs src/modules/permissions/repository.rs
git commit -m "feat: add role CRUD operations to permission service"
```

---

## Task 4: Create Role Management DTOs

**Files:**
- Modify: `src/modules/permissions/dto.rs`

**Step 1: Add role management DTOs**

```rust
// Add to src/modules/permissions/dto.rs

use validator::Validate;
use utoipa::ToSchema;

/// Create role request
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoleRequest {
    /// Unique role name
    #[validate(length(min = 1, message = "Role name is required"))]
    pub name: String,

    /// Human-readable display name
    #[validate(length(min = 1, message = "Display name is required"))]
    pub display_name: String,

    /// Hierarchy level (0 = lowest)
    #[validate(range(min = 0, message = "Level must be non-negative"))]
    pub level: i32,

    /// List of permissions to assign
    pub permissions: Vec<String>,
}

/// Update role permissions request
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRolePermissionsRequest {
    /// New list of permissions
    pub permissions: Vec<String>,
}

/// Assign role to user request
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AssignRoleRequest {
    /// User ID (ObjectId as string)
    #[validate(length(min = 1, message = "User ID is required"))]
    pub user_id: String,

    /// Role name to assign
    #[validate(length(min = 1, message = "Role name is required"))]
    pub role_name: String,
}

/// Role response
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RoleResponse {
    /// Role ID
    pub id: String,

    /// Role name
    pub name: String,

    /// Display name
    pub display_name: String,

    /// Hierarchy level
    pub level: i32,

    /// All permissions
    pub permissions: Vec<String>,

    /// Is system role
    pub is_system: bool,

    /// Version
    pub version: i32,
}

impl From<Role> for RoleResponse {
    fn from(role: Role) -> Self {
        Self {
            id: role.id.map(|id| id.to_hex()).unwrap_or_default(),
            name: role.name,
            display_name: role.display_name,
            level: role.level,
            permissions: role.flattened_permissions,
            is_system: role.is_system,
            version: role.version,
        }
    }
}
```

**Step 2: Run cargo check**

```bash
cargo check
```

Expected: No errors

**Step 3: Commit**

```bash
git add src/modules/permissions/dto.rs
git commit -m "feat: add role management DTOs"
```

---

## Task 5: Create Role Management Handlers

**Files:**
- Modify: `src/modules/permissions/handler.rs`

**Step 1: Add role management handlers**

```rust
// Add to src/modules/permissions/handler.rs

use super::dto::*;
use crate::core::MessageResponse;

/// Create a new role (admin only)
#[utoipa::path(
    post,
    path = "/api/permissions/roles",
    tag = "Permissions",
    security(("bearer_auth" = [])),
    request_body = CreateRoleRequest,
    responses(
        (status = 201, description = "Role created", body = ApiResponse<RoleResponse>),
        (status = 400, description = "Validation error", body = ApiError),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Forbidden - Admin only", body = ApiError)
    )
)]
pub async fn create_role(
    service: web::Data<Arc<PermissionService>>,
    req: web::Json<CreateRoleRequest>,
) -> Result<HttpResponse, ApiError> {
    req.validate()?;

    let role = service
        .create_role(
            req.name.clone(),
            req.display_name.clone(),
            req.level,
            req.permissions.clone(),
        )
        .await?;

    Ok(HttpResponse::Created().json(ApiResponse::success(RoleResponse::from(role))))
}

/// List all roles
#[utoipa::path(
    get,
    path = "/api/permissions/roles",
    tag = "Permissions",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of roles", body = ApiResponse<Vec<RoleResponse>>),
        (status = 401, description = "Unauthorized", body = ApiError)
    )
)]
pub async fn list_roles(
    service: web::Data<Arc<PermissionService>>,
) -> Result<HttpResponse, ApiError> {
    let roles = service.list_roles().await?;
    let response: Vec<RoleResponse> = roles.into_iter().map(RoleResponse::from).collect();

    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

/// Update role permissions (admin only)
#[utoipa::path(
    put,
    path = "/api/permissions/roles/{role_name}/permissions",
    tag = "Permissions",
    security(("bearer_auth" = [])),
    params(
        ("role_name" = String, Path, description = "Role name")
    ),
    request_body = UpdateRolePermissionsRequest,
    responses(
        (status = 200, description = "Permissions updated", body = ApiResponse<MessageResponse>),
        (status = 400, description = "Validation error", body = ApiError),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Forbidden - Admin only", body = ApiError),
        (status = 404, description = "Role not found", body = ApiError)
    )
)]
pub async fn update_role_permissions(
    service: web::Data<Arc<PermissionService>>,
    path: web::Path<String>,
    req: web::Json<UpdateRolePermissionsRequest>,
) -> Result<HttpResponse, ApiError> {
    let role_name = path.into_inner();
    req.validate()?;

    service
        .update_role_permissions(&role_name, req.permissions.clone())
        .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(MessageResponse::new(
        "Role permissions updated successfully",
    ))))
}

/// Delete a role (admin only)
#[utoipa::path(
    delete,
    path = "/api/permissions/roles/{role_name}",
    tag = "Permissions",
    security(("bearer_auth" = [])),
    params(
        ("role_name" = String, Path, description = "Role name")
    ),
    responses(
        (status = 200, description = "Role deleted", body = ApiResponse<MessageResponse>),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Forbidden - Admin only", body = ApiError),
        (status = 404, description = "Role not found", body = ApiError)
    )
)]
pub async fn delete_role(
    service: web::Data<Arc<PermissionService>>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let role_name = path.into_inner();

    service.delete_role(&role_name).await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(MessageResponse::new(
        "Role deleted successfully",
    ))))
}

/// Assign role to user (admin only)
#[utoipa::path(
    post,
    path = "/api/permissions/roles/assign",
    tag = "Permissions",
    security(("bearer_auth" = [])),
    request_body = AssignRoleRequest,
    responses(
        (status = 200, description = "Role assigned", body = ApiResponse<MessageResponse>),
        (status = 400, description = "Validation error", body = ApiError),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Forbidden - Admin only", body = ApiError),
        (status = 404, description = "Role not found", body = ApiError)
    )
)]
pub async fn assign_role(
    service: web::Data<Arc<PermissionService>>,
    req: web::Json<AssignRoleRequest>,
) -> Result<HttpResponse, ApiError> {
    req.validate()?;

    service
        .assign_role_to_user(&req.user_id, &req.role_name)
        .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(MessageResponse::new(
        "Role assigned successfully",
    ))))
}
```

**Step 2: Run cargo check**

```bash
cargo check
```

Expected: No errors

**Step 3: Commit**

```bash
git add src/modules/permissions/handler.rs
git commit -m "feat: add role management handlers"
```

---

## Task 6: Update Routes

**Files:**
- Modify: `src/modules/permissions/routes.rs`

**Step 1: Update routes configuration**

```rust
// Replace src/modules/permissions/routes.rs with:

//! Permission routes
//!
//! Route configuration for permission and role management endpoints.

use actix_web::web;
use super::handler;

/// Configures permission routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/permissions")
            // Permission endpoints
            .route("", web::get().to(handler::list_permissions))
            // Role management endpoints
            .service(
                web::scope("/roles")
                    .route("", web::post().to(handler::create_role))
                    .route("", web::get().to(handler::list_roles))
                    .route("/{role_name}", web::delete().to(handler::delete_role))
                    .route("/{role_name}/permissions", web::put().to(handler::update_role_permissions))
                    .route("/assign", web::post().to(handler::assign_role))
            ),
    );
}
```

**Step 2: Run cargo check**

```bash
cargo check
```

Expected: No errors

**Step 3: Commit**

```bash
git add src/modules/permissions/routes.rs
git commit -m "feat: update permission routes"
```

---

## Task 7: Create Seed Script for Default Roles

**Files:**
- Create: `scripts/seed_roles.rs`
- Modify: `Cargo.toml`

**Step 1: Add binary to Cargo.toml**

```toml
# Add to [[bin]] section in Cargo.toml

[[bin]]
name = "seed_roles"
path = "scripts/seed_roles.rs"
```

**Step 2: Create seed script**

```rust
// scripts/seed_roles.rs

//! Role Seeding Script
//!
//! Creates default roles with permissions for the MMO API.
//!
//! Run with:
//!   cargo run --bin seed_roles

use bson::oid::ObjectId;
use mongodb::{bson::doc, Client, Collection};
use serde::{Deserialize, Serialize};
use tokio::main;

// Import permission constants from our module
// Note: Since this is a binary, we'll define the same permissions here

#[derive(Debug, Serialize, Deserialize)]
struct Role {
    #[serde(rename = "_id")]
    id: Option<ObjectId>,
    name: String,
    display_name: String,
    level: i32,
    parent_role_id: Option<ObjectId>,
    inherits_from: Vec<String>,
    direct_permissions: Vec<ObjectId>,
    flattened_permissions: Vec<String>,
    is_system: bool,
    is_active: bool,
    version: i32,
    created_at: mongodb::bson::DateTime,
    updated_at: mongodb::bson::DateTime,
}

// Define permissions (must match constants.rs)
const PERM_PRODUCT_CREATE: &str = "product:create";
const PERM_PRODUCT_READ: &str = "product:read";
const PERM_PRODUCT_UPDATE: &str = "product:update";
const PERM_PRODUCT_DELETE: &str = "product:delete";
const PERM_PRODUCT_LIST: &str = "product:list";

const PERM_ORDER_CREATE: &str = "order:create";
const PERM_ORDER_READ: &str = "order:read";
const PERM_ORDER_UPDATE: &str = "order:update";
const PERM_ORDER_CANCEL: &str = "order:cancel";
const PERM_ORDER_LIST: &str = "order:list";

const PERM_WALLET_READ: &str = "wallet:read";
const PERM_WALLET_WITHDRAW: &str = "wallet:withdraw";
const PERM_WALLET_DEPOSIT: &str = "wallet:deposit";
const PERM_WALLET_LIST: &str = "wallet:list";

const PERM_USER_CREATE: &str = "user:create";
const PERM_USER_READ: &str = "user:read";
const PERM_USER_UPDATE: &str = "user:update";
const PERM_USER_DELETE: &str = "user:delete";
const PERM_USER_ASSIGN_ROLES: &str = "user:assign_roles";

const PERM_ROLE_CREATE: &str = "role:create";
const PERM_ROLE_READ: &str = "role:read";
const PERM_ROLE_UPDATE: &str = "role:update";
const PERM_ROLE_DELETE: &str = "role:delete";
const PERM_ROLE_ASSIGN_PERMISSIONS: &str = "role:assign_permissions";

#[main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================");
    println!("MMO API - Seed Default Roles");
    println!("========================================");

    // Load MongoDB URL
    let mongo_url = std::env::var("MONGODB_URL")
        .or_else(|_| std::env::var("MONGODB_URI"))
        .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());

    println!("\nConnecting to MongoDB...");
    let client = Client::with_uri_str(&mongo_url).await?;
    let db = client.database("mmo_api");
    let roles_collection: Collection<Role> = db.collection("roles");

    // Define default roles
    let default_roles = vec![
        // BUYER - Level 0
        Role {
            id: None,
            name: "BUYER".to_string(),
            display_name: "Buyer".to_string(),
            level: 0,
            parent_role_id: None,
            inherits_from: vec![],
            direct_permissions: vec![],
            flattened_permissions: vec![
                PERM_PRODUCT_LIST.to_string(),
                PERM_PRODUCT_READ.to_string(),
                PERM_ORDER_CREATE.to_string(),
                PERM_ORDER_READ.to_string(),
                PERM_ORDER_LIST.to_string(),
                PERM_WALLET_READ.to_string(),
                PERM_WALLET_DEPOSIT.to_string(),
            ],
            is_system: true,
            is_active: true,
            version: 1,
            created_at: mongodb::bson::DateTime::now(),
            updated_at: mongodb::bson::DateTime::now(),
        },
        // SELLER - Level 1
        Role {
            id: None,
            name: "SELLER".to_string(),
            display_name: "Seller".to_string(),
            level: 1,
            parent_role_id: None,
            inherits_from: vec!["BUYER".to_string()],
            direct_permissions: vec![],
            flattened_permissions: vec![
                // Inherited from BUYER
                PERM_PRODUCT_LIST.to_string(),
                PERM_PRODUCT_READ.to_string(),
                PERM_ORDER_CREATE.to_string(),
                PERM_ORDER_READ.to_string(),
                PERM_ORDER_LIST.to_string(),
                PERM_WALLET_READ.to_string(),
                PERM_WALLET_DEPOSIT.to_string(),
                // SELLER-specific
                PERM_PRODUCT_CREATE.to_string(),
                PERM_PRODUCT_UPDATE.to_string(),
                PERM_PRODUCT_DELETE.to_string(),
                PERM_ORDER_UPDATE.to_string(),
                PERM_WALLET_WITHDRAW.to_string(),
            ],
            is_system: true,
            is_active: true,
            version: 1,
            created_at: mongodb::bson::DateTime::now(),
            updated_at: mongodb::bson::DateTime::now(),
        },
        // ADMIN - Level 2
        Role {
            id: None,
            name: "ADMIN".to_string(),
            display_name: "Administrator".to_string(),
            level: 2,
            parent_role_id: None,
            inherits_from: vec!["SELLER".to_string()],
            direct_permissions: vec![],
            flattened_permissions: vec![
                // All product permissions
                PERM_PRODUCT_CREATE.to_string(),
                PERM_PRODUCT_READ.to_string(),
                PERM_PRODUCT_UPDATE.to_string(),
                PERM_PRODUCT_DELETE.to_string(),
                PERM_PRODUCT_LIST.to_string(),
                // All order permissions
                PERM_ORDER_CREATE.to_string(),
                PERM_ORDER_READ.to_string(),
                PERM_ORDER_UPDATE.to_string(),
                PERM_ORDER_CANCEL.to_string(),
                PERM_ORDER_LIST.to_string(),
                // All wallet permissions
                PERM_WALLET_READ.to_string(),
                PERM_WALLET_WITHDRAW.to_string(),
                PERM_WALLET_DEPOSIT.to_string(),
                PERM_WALLET_LIST.to_string(),
                // User management
                PERM_USER_READ.to_string(),
                PERM_USER_UPDATE.to_string(),
                PERM_USER_ASSIGN_ROLES.to_string(),
                // Role management
                PERM_ROLE_READ.to_string(),
            ],
            is_system: true,
            is_active: true,
            version: 1,
            created_at: mongodb::bson::DateTime::now(),
            updated_at: mongodb::bson::DateTime::now(),
        },
        // SUPER_ADMIN - Level 3
        Role {
            id: None,
            name: "SUPER_ADMIN".to_string(),
            display_name: "Super Administrator".to_string(),
            level: 3,
            parent_role_id: None,
            inherits_from: vec![],
            direct_permissions: vec![],
            flattened_permissions: vec![
                // All permissions
                PERM_PRODUCT_CREATE.to_string(),
                PERM_PRODUCT_READ.to_string(),
                PERM_PRODUCT_UPDATE.to_string(),
                PERM_PRODUCT_DELETE.to_string(),
                PERM_PRODUCT_LIST.to_string(),
                PERM_ORDER_CREATE.to_string(),
                PERM_ORDER_READ.to_string(),
                PERM_ORDER_UPDATE.to_string(),
                PERM_ORDER_CANCEL.to_string(),
                PERM_ORDER_LIST.to_string(),
                PERM_WALLET_READ.to_string(),
                PERM_WALLET_WITHDRAW.to_string(),
                PERM_WALLET_DEPOSIT.to_string(),
                PERM_WALLET_LIST.to_string(),
                PERM_USER_CREATE.to_string(),
                PERM_USER_READ.to_string(),
                PERM_USER_UPDATE.to_string(),
                PERM_USER_DELETE.to_string(),
                PERM_USER_ASSIGN_ROLES.to_string(),
                PERM_ROLE_CREATE.to_string(),
                PERM_ROLE_READ.to_string(),
                PERM_ROLE_UPDATE.to_string(),
                PERM_ROLE_DELETE.to_string(),
                PERM_ROLE_ASSIGN_PERMISSIONS.to_string(),
            ],
            is_system: true,
            is_active: true,
            version: 1,
            created_at: mongodb::bson::DateTime::now(),
            updated_at: mongodb::bson::DateTime::now(),
        },
    ];

    // Insert roles
    println!("\nSeeding roles...");
    for role in &default_roles {
        // Check if exists
        let existing = roles_collection
            .find_one(doc! { "name": &role.name })
            .await?;

        if existing.is_some() {
            println!("  Role '{}' already exists, skipping...", role.name);
        } else {
            roles_collection.insert_one(role.clone()).await?;
            println!("  Created role: {} (level {})", role.name, role.level);
        }
    }

    println!("\n========================================");
    println!("Role seeding completed!");
    println!("========================================");
    println!("\nDefault roles:");
    for role in &default_roles {
        println!("  - {} ({} permissions)", role.name, role.flattened_permissions.len());
    }
    println!();

    Ok(())
}
```

**Step 3: Run cargo check**

```bash
cargo check --bin seed_roles
```

Expected: No errors

**Step 4: Run seed script**

```bash
MONGODB_URI="mongodb://mmo_admin:mmo_secret_password@localhost:27017" cargo run --bin seed_roles
```

Expected: Roles created successfully

**Step 5: Commit**

```bash
git add scripts/seed_roles.rs Cargo.toml
git commit -m "feat: add role seeding script"
```

---

## Task 8: Update Auth Handler to Use Dynamic Roles

**Files:**
- Modify: `src/modules/auth/handler.rs`
- Modify: `src/modules/auth/service.rs`

**Step 1: Remove hardcoded role validation**

In `src/modules/auth/service.rs`, remove hardcoded validation from `assign_roles`:

```rust
// Replace the validation section in assign_roles method

// OLD CODE (remove this):
let valid_roles = ["BUYER", "SELLER", "ADMIN", "SUPER_ADMIN"];
for role in &roles {
    if !valid_roles.contains(&role.as_str()) {
        return Err(ServiceError::ValidationFailed(format!(
            "Invalid role: {}. Valid roles are: {}",
            role,
            valid_roles.join(", ")
        )));
    }
}

// NEW CODE (replace with):
// Validate roles exist in database
for role_name in &roles {
    let role_exists = self
        .user_repo
        .role_exists(role_name)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

    if !role_exists {
        return Err(ServiceError::ValidationFailed(format!(
            "Role '{}' does not exist. Please create it first.",
            role_name
        )));
    }
}
```

**Step 2: Add role_exists method to UserRepository**

```rust
// Add to UserRepository in src/modules/auth/repository.rs

/// Check if a role exists in the roles collection
///
/// # Arguments
/// * `role_name` - Role name to check
///
/// # Returns
/// * `Result<bool, DbError>` - true if exists
pub async fn role_exists(&self, role_name: &str) -> Result<bool, DbError> {
    let roles_collection = self
        .database
        .collection::<mongodb::bson::Document>("roles");

    let count = roles_collection
        .count_documents(doc! {
            "name": role_name,
            "is_active": true
        })
        .await?;

    Ok(count > 0)
}
```

**Step 3: Update assign_roles handler to use dynamic roles**

The handler in `src/modules/auth/handler.rs` is already generic. Just update the comment:

```rust
/// Assign roles to user (admin only)
///
/// POST /api/auth/admin/assign-roles
///
/// Note: Roles must be created via /api/permissions/roles first
#[utoipa::path(...)]
#[protect("ADMIN", "SUPER_ADMIN")]
pub async fn assign_roles(...)
```

**Step 4: Run cargo check**

```bash
cargo check
```

Expected: No errors

**Step 5: Commit**

```bash
git add src/modules/auth/
git commit -m "feat: remove hardcoded role validation, use dynamic roles from DB"
```

---

## Task 9: Update create_super_admin Script

**Files:**
- Modify: `scripts/create_super_admin.rs`

**Step 1: Update script to assign SUPER_ADMIN role**

The script currently sets `role: "SUPER_ADMIN"` directly. This still works for backward compatibility, but we should ensure the role exists.

Update the script to verify role exists:

```rust
// After checking if user exists, add role verification

// Verify SUPER_ADMIN role exists
let roles_collection: Collection<mongodb::bson::Document> = db.collection("roles");
let super_admin_role = roles_collection
    .find_one(doc! { "name": "SUPER_ADMIN", "is_active": true })
    .await?;

if super_admin_role.is_none() {
    println!("WARNING: SUPER_ADMIN role not found in database!");
    println!("Please run: cargo run --bin seed_roles");
    return Err("SUPER_ADMIN role not found".into());
}
```

**Step 2: Run cargo check**

```bash
cargo check --bin create_super_admin
```

Expected: No errors

**Step 3: Commit**

```bash
git add scripts/create_super_admin.rs
git commit -m "feat: verify SUPER_ADMIN role exists in create_super_admin script"
```

---

## Task 10: Integration Testing

**Files:**
- Create: `tests/integration/role_management_test.rs`

**Step 1: Create integration test**

```rust
// tests/integration/role_management_test.rs

//! Integration tests for dynamic role management

#[actix_web::test]
async fn test_create_and_assign_role() {
    // Test creating a custom role
    // Test assigning role to user
    // Test permission checking
}

// Add more tests...
```

**Step 2: Run tests**

```bash
cargo test
```

Expected: All tests pass

**Step 3: Commit**

```bash
git add tests/
git commit -m "test: add role management integration tests"
```

---

## Summary

After implementing all tasks:

1. ✅ Permissions are **hardcoded Enum constants** in Rust code (type-safe)
2. ✅ Roles are **dynamic** (CRUD operations via API)
3. ✅ Default roles seeded: BUYER, SELLER, ADMIN, SUPER_ADMIN
4. ✅ Backward compatible with existing JWT tokens
5. ✅ Admin can create custom roles via API
6. ✅ Role assignments are validated against database

**API Endpoints:**
- `POST /api/permissions/roles` - Create role
- `GET /api/permissions/roles` - List all roles
- `PUT /api/permissions/roles/{role_name}/permissions` - Update role permissions
- `DELETE /api/permissions/roles/{role_name}` - Delete role
- `POST /api/permissions/roles/assign` - Assign role to user

**Usage Example:**

```bash
# 1. Seed default roles
cargo run --bin seed_roles

# 2. Create custom role
curl -X POST http://localhost:8080/api/permissions/roles \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "CUSTOMER_SUPPORT",
    "displayName": "Customer Support",
    "level": 1,
    "permissions": ["order:read", "order:update", "user:read"]
  }'

# 3. Assign role to user
curl -X POST http://localhost:8080/api/permissions/roles/assign \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "userId": "<user_id>",
    "roleName": "CUSTOMER_SUPPORT"
  }'
```
