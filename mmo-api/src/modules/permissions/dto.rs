//! Permission and Role DTOs
//!
//! Data Transfer Objects for permission API requests and responses.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use mongodb::bson::oid::ObjectId;

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
