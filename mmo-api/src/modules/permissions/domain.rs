//! Permission and Role domain models
//!
//! Defines the core data structures for the V2 authorization system.

use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

/// Permission represents a granular action on a resource
///
/// Permissions follow the `resource:action` naming convention.
/// Example: `products:create`, `orders:read`, `users:manage`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    /// MongoDB ObjectId
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// Unique permission identifier in "resource:action" format
    /// Examples: "products:create", "orders:read", "users:manage"
    pub name: String,

    /// Human-readable display name for UI
    pub display_name: String,

    /// Detailed description of what this permission allows
    pub description: String,

    /// Resource type (e.g., products, orders, users, shops)
    pub resource: String,

    /// Action type (e.g., create, read, update, delete, manage)
    pub action: String,

    /// Category for grouping in UI (e.g., marketplace, admin, finance)
    pub category: String,

    /// Soft delete flag - inactive permissions are ignored
    pub is_active: bool,

    /// Creation timestamp
    pub created_at: DateTime,

    /// Last update timestamp
    pub updated_at: DateTime,
}

/// Role with hierarchy and inheritance support
///
/// Roles inherit permissions from parent roles in the hierarchy:
/// super_admin (level 3) → admin (level 2) → seller (level 1) → buyer (level 0)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// MongoDB ObjectId
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// Unique role identifier (e.g., buyer, seller, admin, super_admin)
    pub name: String,

    /// Human-readable display name
    pub display_name: String,

    /// Hierarchy level (0 = lowest, higher = more access)
    /// Used for role comparison: level 3 can do everything level 2 can do
    pub level: i32,

    /// Reference to parent role for inheritance chain
    pub parent_role_id: Option<ObjectId>,

    /// List of role names this role inherits permissions from
    /// Example: seller inherits from ["buyer"]
    pub inherits_from: Vec<String>,

    /// Directly assigned permission IDs
    pub direct_permissions: Vec<ObjectId>,

    /// Pre-computed flattened list of all permission names
    /// Includes both direct and inherited permissions
    /// Example: ["products:read", "products:create", "orders:read"]
    pub flattened_permissions: Vec<String>,

    /// System roles cannot be deleted
    pub is_system: bool,

    /// Soft delete flag
    pub is_active: bool,

    /// Version for optimistic locking and cache invalidation
    /// Increment when permissions change
    pub version: i32,

    /// Creation timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime>,

    /// Last update timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime>,
}

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
            created_at: Some(now),
            updated_at: Some(now),
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

/// User role assignment with metadata
///
/// Tracks when and by whom a role was assigned to a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleAssignment {
    /// Reference to the role document
    pub role_id: ObjectId,

    /// Denormalized role name for quick access without lookup
    pub role_name: String,

    /// When this role was assigned to the user
    pub assigned_at: DateTime,

    /// Who assigned this role (null = system assigned)
    pub assigned_by: Option<ObjectId>,
}

/// User permission query result
///
/// Represents a user's complete permission set computed from
/// their assigned roles and direct permission grants.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermissions {
    /// User ID
    pub user_id: String,

    /// List of role names assigned to this user
    pub roles: Vec<String>,

    /// User-specific direct permissions (special grants outside roles)
    pub direct_permissions: Vec<String>,

    /// Complete flattened list of all permissions the user has
    /// Combines role permissions + direct permissions
    pub effective_permissions: Vec<String>,

    /// Permission version for cache invalidation
    pub perm_version: i32,
}

#[cfg(test)]
mod tests {
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
            created_at: DateTime::now(),
            updated_at: DateTime::now(),
        };

        assert_eq!(perm.resource, "products");
        assert_eq!(perm.action, "create");
        assert_eq!(perm.name, "products:create");
        assert!(perm.is_active);
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
                "products:update".to_string(),
                "products:delete".to_string(),
                "orders:read".to_string(),
                "shops:create".to_string(),
            ],
            is_system: true,
            is_active: true,
            version: 1,
        };

        assert_eq!(role.level, 1);
        assert!(role.inherits_from.contains(&"buyer".to_string()));
        assert_eq!(role.flattened_permissions.len(), 6);
        assert!(role.is_system);
    }

    #[test]
    fn test_role_assignment() {
        let role_id = ObjectId::new();
        let assigned_by = ObjectId::new();

        let assignment = RoleAssignment {
            role_id,
            role_name: "seller".to_string(),
            assigned_at: DateTime::now(),
            assigned_by: Some(assigned_by),
        };

        assert_eq!(assignment.role_name, "seller");
        assert!(assignment.assigned_by.is_some());
    }

    #[test]
    fn test_user_permissions() {
        let perms = UserPermissions {
            user_id: "user123".to_string(),
            roles: vec!["buyer".to_string(), "seller".to_string()],
            direct_permissions: vec![],
            effective_permissions: vec![
                "products:read".to_string(),
                "products:create".to_string(),
            ],
            perm_version: 5,
        };

        assert_eq!(perms.roles.len(), 2);
        assert_eq!(perms.perm_version, 5);
        assert!(perms.effective_permissions.contains(&"products:create".to_string()));
    }
}
