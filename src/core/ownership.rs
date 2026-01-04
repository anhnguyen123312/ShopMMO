//! Ownership check utilities
//!
//! Provides helper functions for checking if a user owns a resource
//! or has admin privileges to access any resource.

use crate::middleware::AuthUser;

/// Check if user can access a resource
///
/// Users can access their own resources, admins can access all resources.
///
/// # Arguments
/// * `auth_user` - The authenticated user from request
/// * `resource_user_id` - The owner ID of the resource being accessed
///
/// # Returns
/// * `Ok(())` - User has access (owns resource or is admin)
/// * `Err(ApiError)` - User does not have access (403 Forbidden)
///
/// # Example
/// ```rust
/// use crate::core::ownership::check_ownership;
///
/// pub async fn get_wallet(
///     auth: AuthUser,
///     wallet_id: String,
/// ) -> Result<Wallet, ApiError> {
///     let wallet = repository.find_by_id(&wallet_id).await?;
///     check_ownership(&auth, &wallet.user_id)?;
///     Ok(wallet)
/// }
/// ```
pub fn check_ownership(auth_user: &AuthUser, resource_user_id: &str) -> Result<(), crate::core::ApiError> {
    // Admin and SUPER_ADMIN can access any resource
    if auth_user.role == "ADMIN" || auth_user.role == "SUPER_ADMIN" {
        return Ok(());
    }

    // Check if user owns the resource
    if auth_user.user_id == resource_user_id {
        return Ok(());
    }

    Err(crate::core::ApiError::forbidden(
        "You do not have permission to access this resource"
    ))
}

/// Check if user can modify a resource
///
/// Similar to check_ownership but specifically for write operations.
/// Can be extended with additional checks like resource locking.
///
/// # Arguments
/// * `auth_user` - The authenticated user from request
/// * `resource_user_id` - The owner ID of the resource being modified
///
/// # Returns
/// * `Ok(())` - User can modify the resource
/// * `Err(ApiError)` - User cannot modify the resource
pub fn check_modify_permission(auth_user: &AuthUser, resource_user_id: &str) -> Result<(), crate::core::ApiError> {
    check_ownership(auth_user, resource_user_id)
}

/// Check if user has admin privileges
///
/// # Arguments
/// * `auth_user` - The authenticated user from request
///
/// # Returns
/// * `true` - User is ADMIN or SUPER_ADMIN
/// * `false` - User is not an admin
pub fn is_admin(auth_user: &AuthUser) -> bool {
    auth_user.role == "ADMIN" || auth_user.role == "SUPER_ADMIN"
}

/// Check if user has a specific role
///
/// # Arguments
/// * `auth_user` - The authenticated user from request
/// * `role` - The role to check for
///
/// # Returns
/// * `true` - User has the specified role
/// * `false` - User does not have the role
pub fn has_role(auth_user: &AuthUser, role: &str) -> bool {
    auth_user.role == role || auth_user.roles.contains(&role.to_string())
}

/// Check if user has any of the specified roles
///
/// # Arguments
/// * `auth_user` - The authenticated user from request
/// * `roles` - Slice of roles to check for
///
/// # Returns
/// * `true` - User has at least one of the specified roles
/// * `false` - User does not have any of the specified roles
pub fn has_any_role(auth_user: &AuthUser, roles: &[&str]) -> bool {
    roles.iter().any(|&role| has_role(auth_user, role))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_user(user_id: &str, role: &str, roles: Vec<String>) -> AuthUser {
        AuthUser {
            user_id: user_id.to_string(),
            wallet_id: format!("WLT-{}", user_id),
            email: format!("{}@test.com", user_id),
            role: role.to_string(),
            roles,
            perm_version: 1,
        }
    }

    #[test]
    fn test_check_ownership_owner() {
        let user = create_test_user("user123", "BUYER", vec!["BUYER".to_string()]);
        assert!(check_ownership(&user, "user123").is_ok());
    }

    #[test]
    fn test_check_ownership_not_owner() {
        let user = create_test_user("user123", "BUYER", vec!["BUYER".to_string()]);
        assert!(check_ownership(&user, "user456").is_err());
    }

    #[test]
    fn test_check_ownership_admin() {
        let admin = create_test_user("admin123", "ADMIN", vec!["ADMIN".to_string()]);
        // Admin can access any resource
        assert!(check_ownership(&admin, "user999").is_ok());
    }

    #[test]
    fn test_is_admin() {
        let admin = create_test_user("admin123", "ADMIN", vec!["ADMIN".to_string()]);
        let super_admin = create_test_user("super123", "SUPER_ADMIN", vec!["SUPER_ADMIN".to_string()]);
        let user = create_test_user("user123", "BUYER", vec!["BUYER".to_string()]);

        assert!(is_admin(&admin));
        assert!(is_admin(&super_admin));
        assert!(!is_admin(&user));
    }

    #[test]
    fn test_has_role() {
        let user = create_test_user("user123", "BUYER", vec!["BUYER".to_string(), "SELLER".to_string()]);

        assert!(has_role(&user, "BUYER"));
        assert!(has_role(&user, "SELLER"));
        assert!(!has_role(&user, "ADMIN"));
    }

    #[test]
    fn test_has_any_role() {
        let user = create_test_user("user123", "BUYER", vec!["BUYER".to_string(), "SELLER".to_string()]);

        assert!(has_any_role(&user, &["BUYER", "ADMIN"]));
        assert!(has_any_role(&user, &["SELLER", "ADMIN"]));
        assert!(!has_any_role(&user, &["ADMIN", "SUPER_ADMIN"]));
    }
}
