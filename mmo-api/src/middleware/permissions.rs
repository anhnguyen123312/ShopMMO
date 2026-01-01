//! Permission extractor for actix-web-grants integration
//!
//! Extracts user permissions from JWT and cache/DB for authorization.

use actix_web::{dev::Payload, Error, FromRequest, HttpRequest};
use futures::future::{ready, Ready};
use std::sync::Arc;

use crate::modules::permissions::{cache::PermissionCache, repository::RoleRepository};
use crate::database::mongodb::MongoDB;

/// Authenticated user with role information
///
/// This is the V2 version with support for multiple roles and permission versioning.
#[derive(Debug, Clone)]
pub struct AuthUserV2 {
    /// User's MongoDB ObjectId
    pub user_id: String,

    /// User's wallet ID (from JWT, kept for backward compatibility)
    pub wallet_id: String,

    /// User's email (from JWT, kept for backward compatibility)
    pub email: String,

    /// Array of role names assigned to this user
    pub roles: Vec<String>,

    /// Permission version for cache validation
    pub perm_version: u32,
}

/// Permission extractor function for actix-web-grants
///
/// This function is called by actix-web-grants middleware to get
/// the list of permissions for the current user.
///
/// # Returns
/// Vector of permission strings (e.g., ["products:read", "orders:create"])
pub async fn extract_permissions(
    auth_user: &AuthUserV2,
    db: &MongoDB,
    redis_url: &str,
) -> Vec<String> {
    // Try cache first with atomic check
    let cache = match PermissionCache::new(redis_url).await {
        Ok(c) => c,
        Err(_) => {
            // If Redis is unavailable, fall back to DB
            return get_permissions_from_db(auth_user, db).await;
        }
    };

    // Check cache
    match cache.get_permissions(&auth_user.user_id).await {
        Ok(permissions) if !permissions.is_empty() => {
            // Cache hit - verify version
            match cache.check_permission_atomic(
                &auth_user.user_id,
                &permissions.first().unwrap_or(&String::new()),
                auth_user.perm_version as i32,
            ).await {
                Ok(Some(_)) => permissions, // Cache valid
                _ => {
                    // Cache stale or miss - fetch from DB
                    let perms = get_permissions_from_db(auth_user, db).await;
                    // Update cache
                    if !perms.is_empty() {
                        let _ = cache.set_permissions(
                            &auth_user.user_id,
                            &perms,
                            auth_user.perm_version as i32,
                        ).await;
                    }
                    perms
                }
            }
        }
        _ => {
            // Cache miss - fetch from DB
            let perms = get_permissions_from_db(auth_user, db).await;
            // Update cache
            if !perms.is_empty() {
                let _ = cache.set_permissions(
                    &auth_user.user_id,
                    &perms,
                    auth_user.perm_version as i32,
                ).await;
            }
            perms
        }
    }
}

/// Fetch permissions from database
async fn get_permissions_from_db(
    auth_user: &AuthUserV2,
    db: &MongoDB,
) -> Vec<String> {
    let role_repo = RoleRepository::new(db.database().clone());

    match role_repo.get_user_permissions(&auth_user.user_id).await {
        Ok(user_perms) => user_perms.effective_permissions,
        Err(_) => {
            tracing::warn!(
                user_id = %auth_user.user_id,
                "Failed to get user permissions from DB"
            );
            vec![]
        }
    }
}

/// FromRequest implementation for AuthUserV2
///
/// This extractor reads from request extensions where the auth middleware
/// should have placed the user info.
impl FromRequest for AuthUserV2 {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &HttpRequest,
        _: &mut Payload,
    ) -> Self::Future {
        // For now, return error if not authenticated
        // The actual implementation will read from request extensions
        // after the auth middleware populates it
        ready(Err(actix_web::error::ErrorUnauthorized(
            "User not authenticated - use V2 auth middleware"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_user_v2_creation() {
        let user = AuthUserV2 {
            user_id: "user123".to_string(),
            wallet_id: "WLT-user123".to_string(),
            email: "user@example.com".to_string(),
            roles: vec!["buyer".to_string(), "seller".to_string()],
            perm_version: 5,
        };

        assert_eq!(user.user_id, "user123");
        assert_eq!(user.roles.len(), 2);
        assert_eq!(user.perm_version, 5);
    }
}
