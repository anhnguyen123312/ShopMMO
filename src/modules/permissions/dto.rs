//! Permission and Role DTOs
//!
//! Data Transfer Objects for permission API requests and responses.

use serde::{Deserialize, Serialize};
use validator::Validate;
use utoipa::ToSchema;

use super::domain::Role;

/// Request to create a new permission
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePermissionRequest {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub resource: String,
    pub action: String,
    pub category: String,
}

/// Response containing permission details
#[derive(Debug, Serialize, ToSchema)]
pub struct PermissionResponse {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub resource: String,
    pub action: String,
    pub category: String,
    pub is_active: bool,
}

/// Request to assign a role to a user
#[derive(Debug, Deserialize, ToSchema)]
pub struct AssignRoleRequest {
    pub role_name: String,
}

/// Response containing user's permissions
#[derive(Debug, Serialize, ToSchema)]
pub struct UserPermissionsResponse {
    pub user_id: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub perm_version: i32,
}

// ========== Role Management DTOs ==========

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
pub struct AssignUserRoleRequest {
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

