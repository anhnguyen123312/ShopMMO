//! Permission extractor for actix-web-grants integration
//!
//! Extracts user permissions from JWT and cache/DB for authorization.

use actix_web::{dev::ServiceRequest, Error, FromRequest, HttpRequest, dev::Payload, HttpMessage};
use futures::future::{ready, Ready};
use std::collections::HashSet;

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

/// Permission extractor for actix-web-grants middleware
///
/// This function extracts user roles from the request (populated by AuthMiddleware).
/// Currently returns roles directly as permissions for simplicity.
///
/// # Signature Required by actix-web-grants
/// - Takes: `&ServiceRequest` (reference to incoming request)
/// - Returns: `Result<HashSet<String>, Error>` (set of permission/role strings)
///
/// # Example
/// ```rust
/// use actix_web_grants::GrantsMiddleware;
///
/// HttpServer::new(|| {
///     App::new()
///         .wrap(GrantsMiddleware::with_extractor(extract_permissions))
/// })
/// ```
pub async fn extract_permissions(req: &ServiceRequest) -> Result<HashSet<String>, Error> {
    // Get AuthUser from request extensions (populated by AuthMiddleware)
    match req.extensions().get::<crate::middleware::AuthUser>() {
        Some(user) => {
            // For now, return roles directly as permissions
            // This allows us to use #[protect("ROLE_ADMIN")] style guards
            // In production, you could fetch actual permissions from DB/cache here
            tracing::debug!(
                user_id = %user.user_id,
                roles = ?user.roles,
                "Extracted permissions for user"
            );
            Ok(user.roles.iter().cloned().collect())
        }
        None => {
            tracing::warn!("No AuthUser found in request extensions");
            // Return empty set instead of error - allows public routes to work
            Ok(HashSet::new())
        }
    }
}

/// Fetch permissions from database
///
/// This function fetches actual permissions from the database (not just roles).
/// It can be used in the future when we need fine-grained permission checking.
async fn get_permissions_from_db(
    auth_user: &AuthUserV2,
    db: &MongoDB,
    redis_url: &str,
) -> Vec<String> {
    // Try cache first with atomic check
    let cache = match PermissionCache::new(redis_url).await {
        Ok(c) => c,
        Err(_) => {
            // If Redis is unavailable, fall back to DB directly
            let role_repo = RoleRepository::new(db.database().clone());
            return match role_repo.get_user_permissions(&auth_user.user_id).await {
                Ok(user_perms) => user_perms.effective_permissions,
                Err(_) => {
                    tracing::warn!(
                        user_id = %auth_user.user_id,
                        "Failed to get user permissions from DB"
                    );
                    vec![]
                }
            };
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
                    let role_repo = RoleRepository::new(db.database().clone());
                    let perms = match role_repo.get_user_permissions(&auth_user.user_id).await {
                        Ok(user_perms) => user_perms.effective_permissions,
                        Err(_) => vec![],
                    };
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
            let role_repo = RoleRepository::new(db.database().clone());
            let perms = match role_repo.get_user_permissions(&auth_user.user_id).await {
                Ok(user_perms) => user_perms.effective_permissions,
                Err(_) => vec![],
            };
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
