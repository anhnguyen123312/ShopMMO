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
