//! Permission and Role DTOs
//!
//! Data Transfer Objects for permission API requests and responses.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use super::domain::Role;

/// Request to create a new permission
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePermissionRequest {
    #[schema(example = "wallet:withdraw:own")]
    pub name: String,

    #[schema(example = "Withdraw Own Funds")]
    pub display_name: String,

    #[schema(example = "Allows user to withdraw funds from their own wallet")]
    pub description: String,

    #[schema(example = "wallet")]
    pub resource: String,

    #[schema(example = "withdraw")]
    pub action: String,

    #[schema(example = "wallet")]
    pub category: String,
}

/// Response containing permission details
#[derive(Debug, Serialize, ToSchema)]
pub struct PermissionResponse {
    #[schema(example = "65f8a1b2c3d4e5f6a7b8c9d0")]
    pub id: String,

    #[schema(example = "wallet:withdraw:own")]
    pub name: String,

    #[schema(example = "Withdraw Own Funds")]
    pub display_name: String,

    #[schema(example = "Allows user to withdraw funds from their own wallet")]
    pub description: String,

    #[schema(example = "wallet")]
    pub resource: String,

    #[schema(example = "withdraw")]
    pub action: String,

    #[schema(example = "wallet")]
    pub category: String,

    #[schema(example = true)]
    pub is_active: bool,
}

/// Request to assign a role to a user
#[derive(Debug, Deserialize, ToSchema)]
pub struct AssignRoleRequest {
    #[schema(example = "SELLER")]
    pub role_name: String,
}

/// Response containing user's permissions
#[derive(Debug, Serialize, ToSchema)]
pub struct UserPermissionsResponse {
    #[schema(example = "65f8a1b2c3d4e5f6a7b8c9d0")]
    pub user_id: String,

    #[schema(example = json!(["BUYER", "SELLER"]))]
    pub roles: Vec<String>,

    #[schema(example = json!(["product:read:all", "product:create:own", "order:create:own", "wallet:read:own", "wallet:withdraw:own", "shop:create:own"]))]
    pub permissions: Vec<String>,

    #[schema(example = 1)]
    pub perm_version: i32,
}

// ========== Role Management DTOs ==========

/// Create role request
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoleRequest {
    /// Unique role name (uppercase, no spaces)
    #[validate(length(min = 1, message = "Role name is required"))]
    #[schema(example = "MODERATOR")]
    pub name: String,

    /// Human-readable display name
    #[validate(length(min = 1, message = "Display name is required"))]
    #[schema(example = "Content Moderator")]
    pub display_name: String,

    /// Hierarchy level (0 = lowest, higher = more authority)
    #[validate(range(min = 0, message = "Level must be non-negative"))]
    #[schema(example = 2)]
    pub level: i32,

    /// List of permissions to assign to this role
    #[schema(example = json!(["product:read:all", "order:read:all", "dispute:read:all", "user:read:all"]))]
    pub permissions: Vec<String>,
}

/// Update role permissions request
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRolePermissionsRequest {
    /// Complete list of permissions (replaces existing)
    #[schema(example = json!(["product:read:all", "product:update:all", "order:read:all", "dispute:read:all", "dispute:resolve:refund", "user:read:all"]))]
    pub permissions: Vec<String>,
}

/// Assign role to user request
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AssignUserRoleRequest {
    /// User ID (MongoDB ObjectId)
    #[validate(length(min = 1, message = "User ID is required"))]
    #[schema(example = "65f8a1b2c3d4e5f6a7b8c9d0")]
    pub user_id: String,

    /// Role name to assign
    #[validate(length(min = 1, message = "Role name is required"))]
    #[schema(example = "SELLER")]
    pub role_name: String,
}

/// Role response
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RoleResponse {
    /// Role ID
    #[schema(example = "65f8a1b2c3d4e5f6a7b8c9d1")]
    pub id: String,

    /// Role name
    #[schema(example = "SELLER")]
    pub name: String,

    /// Display name
    #[schema(example = "Seller/Vendor")]
    pub display_name: String,

    /// Hierarchy level
    #[schema(example = 1)]
    pub level: i32,

    /// All permissions (including inherited)
    #[schema(example = json!(["product:read:all", "product:create:own", "product:update:own", "order:create:own", "order:read:own", "wallet:read:own", "wallet:withdraw:own", "shop:create:own"]))]
    pub permissions: Vec<String>,

    /// Is system role (cannot be deleted)
    #[schema(example = true)]
    pub is_system: bool,

    /// Version number (for cache invalidation)
    #[schema(example = 1)]
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
