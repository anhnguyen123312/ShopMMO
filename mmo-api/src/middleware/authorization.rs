//! Authorization middleware for role-based access control
//!
//! Checks if authenticated users have required roles for specific endpoints.

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorForbidden,
    Error,
};
use futures_util::future::LocalBoxFuture;
use std::{
    future::{ready, Ready},
    rc::Rc,
};

use super::auth::AuthUser;
use crate::core::ApiError;

/// User roles
#[derive(Debug, Clone, PartialEq)]
pub enum UserRole {
    Admin,
    User,
    Seller,
}

impl UserRole {
    pub fn from_str(role: &str) -> Option<Self> {
        match role.to_lowercase().as_str() {
            "admin" => Some(UserRole::Admin),
            "user" => Some(UserRole::User),
            "seller" => Some(UserRole::Seller),
            _ => None,
        }
    }
}

/// Authorization middleware factory
///
/// Checks if user has one of the required roles.
pub struct RequireRole {
    roles: Vec<UserRole>,
}

impl RequireRole {
    /// Creates a new RequireRole middleware
    ///
    /// # Arguments
    /// * `roles` - List of allowed roles
    ///
    /// # Examples
    /// ```
    /// // Only admin can access
    /// .wrap(RequireRole::one_of(vec![UserRole::Admin]))
    ///
    /// // Admin or Seller can access
    /// .wrap(RequireRole::one_of(vec![UserRole::Admin, UserRole::Seller]))
    /// ```
    pub fn one_of(roles: Vec<UserRole>) -> Self {
        Self { roles }
    }

    /// Helper: Require admin role only
    pub fn admin() -> Self {
        Self {
            roles: vec![UserRole::Admin],
        }
    }

    /// Helper: Require seller role (sellers and admins)
    pub fn seller() -> Self {
        Self {
            roles: vec![UserRole::Admin, UserRole::Seller],
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RequireRole
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequireRoleService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequireRoleService {
            service: Rc::new(service),
            roles: self.roles.clone(),
        }))
    }
}

pub struct RequireRoleService<S> {
    service: Rc<S>,
    roles: Vec<UserRole>,
}

impl<S, B> Service<ServiceRequest> for RequireRoleService<S>
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
        let required_roles = self.roles.clone();

        Box::pin(async move {
            // Get authenticated user from extensions
            let auth_user = req.extensions().get::<AuthUser>().cloned();

            match auth_user {
                Some(user) => {
                    // Check if user role matches any required role
                    let user_role = UserRole::from_str(&user.role);

                    match user_role {
                        Some(role) if required_roles.contains(&role) => {
                            // User has required role, continue
                            service.call(req).await
                        }
                        _ => {
                            tracing::warn!(
                                user_id = %user.user_id,
                                user_role = %user.role,
                                required_roles = ?required_roles,
                                "User does not have required role"
                            );
                            Err(ErrorForbidden(
                                ApiError::forbidden(
                                    "You do not have permission to access this resource",
                                )
                                .to_string(),
                            ))
                        }
                    }
                }
                None => {
                    // No authenticated user (should not happen if auth middleware is applied)
                    tracing::error!("Authorization middleware called without authentication");
                    Err(ErrorForbidden(
                        ApiError::forbidden("Authentication required").to_string(),
                    ))
                }
            }
        })
    }
}

/// Macro for easy role checking in handlers
///
/// # Examples
/// ```
/// async fn handler(auth: AuthUser) -> Result<HttpResponse, ApiError> {
///     require_role!(auth, UserRole::Admin)?;
///     // ... admin-only code
///     Ok(HttpResponse::Ok().finish())
/// }
/// ```
#[macro_export]
macro_rules! require_role {
    ($auth:expr, $role:expr) => {
        if UserRole::from_str(&$auth.role) != Some($role) {
            return Err(ApiError::forbidden(
                "You do not have permission to perform this action",
            ));
        }
    };
}
