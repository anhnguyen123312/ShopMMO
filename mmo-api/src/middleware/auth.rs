//! Authentication middleware
//!
//! Validates JWT tokens and extracts user information from requests.

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorUnauthorized,
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use std::{
    future::{ready, Ready},
    rc::Rc,
};

use crate::{config::AppConfig, core::ApiError, utils::jwt};

/// Authenticated user information extracted from JWT
///
/// Available in handlers via the `AuthUser` extractor.
///
/// # V2 Updates
/// - Added `roles` array for multiple role support
/// - Added `perm_version` for cache invalidation
/// - Kept `role` for backward compatibility (primary role)
#[derive(Debug, Clone)]
pub struct AuthUser {
    /// User's MongoDB ObjectId
    pub user_id: String,

    /// User's wallet ID (from JWT)
    pub wallet_id: String,

    /// User's email (from JWT)
    pub email: String,

    /// Primary/single role (deprecated - use roles)
    /// Kept for backward compatibility
    pub role: String,

    /// All roles assigned to the user (V2)
    pub roles: Vec<String>,

    /// Permission version for cache invalidation (V2)
    pub perm_version: u32,
}

/// Admin user extractor (requires ADMIN or SUPER_ADMIN role)
#[derive(Debug, Clone)]
pub struct AdminUser {
    /// Admin's user ID
    pub user_id: String,

    /// Admin's wallet ID
    pub wallet_id: String,

    /// Admin's email
    pub email: String,

    /// Admin's primary role
    pub role: String,

    /// All admin roles
    pub roles: Vec<String>,

    /// Permission version
    pub perm_version: u32,
}

/// Authentication middleware factory
pub struct AuthMiddleware {
    config: Rc<AppConfig>,
}

impl AuthMiddleware {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config: Rc::new(config),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service: Rc::new(service),
            config: self.config.clone(),
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
    config: Rc<AppConfig>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let config = self.config.clone();

        Box::pin(async move {
            // Extract Authorization header
            let auth_header = req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok());

            let token = match auth_header {
                Some(header) if header.starts_with("Bearer ") => &header[7..],
                _ => {
                    tracing::warn!("Missing or invalid Authorization header");
                    return Err(ErrorUnauthorized(
                        ApiError::unauthorized("Missing or invalid authorization token").to_string(),
                    ));
                }
            };

            // Verify token
            let claims = match jwt::verify_token(token, &config.jwt.secret) {
                Ok(claims) => claims,
                Err(e) => {
                    tracing::warn!(error = %e, "Token verification failed");
                    return Err(ErrorUnauthorized(e.to_string()));
                }
            };

            // Validate token type
            if claims.token_type != "access" {
                return Err(ErrorUnauthorized(
                    ApiError::unauthorized("Invalid token type").to_string(),
                ));
            }

            // Create AuthUser and add to request extensions
            // Support both V1 (single role) and V2 (roles array) tokens
            let roles = if claims.roles.is_empty() {
                vec![claims.role.clone()]
            } else {
                claims.roles.clone()
            };

            let auth_user = AuthUser {
                user_id: claims.sub.clone(),
                wallet_id: claims.wallet_id.clone(),
                email: claims.email.clone(),
                role: claims.role.clone(),
                roles,
                perm_version: claims.perm_version,
            };

            req.extensions_mut().insert(auth_user);

            // Continue to next middleware/handler
            service.call(req).await
        })
    }
}

/// Extractor for authenticated user
///
/// # Examples
/// ```
/// async fn handler(auth: web::ReqData<AuthUser>) -> HttpResponse {
///     println!("User ID: {}", auth.user_id);
///     HttpResponse::Ok().finish()
/// }
/// ```
impl actix_web::FromRequest for AuthUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        match req.extensions().get::<AuthUser>() {
            Some(user) => ready(Ok(user.clone())),
            None => ready(Err(ErrorUnauthorized(
                ApiError::unauthorized("User not authenticated").to_string(),
            ))),
        }
    }
}

/// Extractor for admin user (requires ADMIN or SUPER_ADMIN role)
impl actix_web::FromRequest for AdminUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        match req.extensions().get::<AuthUser>() {
            Some(user) => {
                // Check if user has admin role (check both primary role and roles array)
                let is_admin = user.role == "ADMIN"
                    || user.role == "SUPER_ADMIN"
                    || user.roles.contains(&"ADMIN".to_string())
                    || user.roles.contains(&"SUPER_ADMIN".to_string());

                if is_admin {
                    ready(Ok(AdminUser {
                        user_id: user.user_id.clone(),
                        wallet_id: user.wallet_id.clone(),
                        email: user.email.clone(),
                        role: user.role.clone(),
                        roles: user.roles.clone(),
                        perm_version: user.perm_version,
                    }))
                } else {
                    ready(Err(ErrorUnauthorized(
                        ApiError::unauthorized("Admin access required").to_string(),
                    )))
                }
            }
            None => ready(Err(ErrorUnauthorized(
                ApiError::unauthorized("User not authenticated").to_string(),
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App, HttpResponse};

    async fn test_handler(auth: AuthUser) -> HttpResponse {
        HttpResponse::Ok().json(serde_json::json!({
            "user_id": auth.user_id,
            "email": auth.email,
        }))
    }

    // Note: Full integration tests require setting up test server with valid JWT
}
