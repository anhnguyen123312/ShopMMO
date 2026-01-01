//! Authentication handlers
//!
//! HTTP request handlers for authentication endpoints.

use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_grants::protect;
use std::sync::Arc;
use validator::Validate;

use crate::{
    core::{ApiError, ApiResponse, MessageResponse},
    middleware::AuthUser,
};

use super::{dto::*, service::AuthService};

/// Register a new user
///
/// POST /api/auth/register
#[utoipa::path(
    post,
    path = "/api/auth/register",
    tag = "Auth",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully", body = ApiResponse<AuthResponse>),
        (status = 400, description = "Validation error", body = ApiError),
        (status = 409, description = "Email already exists", body = ApiError)
    )
)]
pub async fn register(
    service: web::Data<Arc<AuthService>>,
    req: web::Json<RegisterRequest>,
) -> Result<HttpResponse, ApiError> {
    // Validate request
    req.validate()?;

    // Call service
    let result = service.register(req.into_inner()).await?;

    Ok(HttpResponse::Created().json(ApiResponse::success(result)))
}

/// Login
///
/// POST /api/auth/login
#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "Auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = ApiResponse<AuthResponse>),
        (status = 400, description = "Invalid credentials", body = ApiError)
    )
)]
pub async fn login(
    service: web::Data<Arc<AuthService>>,
    req: web::Json<LoginRequest>,
    http_req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    // Validate request
    req.validate()?;

    // Extract IP and user agent
    let ip_address = http_req
        .connection_info()
        .realip_remote_addr()
        .map(|s| s.to_string());

    let user_agent = http_req
        .headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    // Call service
    let result = service.login(req.into_inner(), ip_address, user_agent).await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(result)))
}

/// Refresh access token
///
/// POST /api/auth/refresh
#[utoipa::path(
    post,
    path = "/api/auth/refresh",
    tag = "Auth",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "Token refreshed", body = ApiResponse<AuthResponse>),
        (status = 401, description = "Invalid or expired refresh token", body = ApiError)
    )
)]
pub async fn refresh_token(
    service: web::Data<Arc<AuthService>>,
    req: web::Json<RefreshTokenRequest>,
) -> Result<HttpResponse, ApiError> {
    // Validate request
    req.validate()?;

    // Call service
    let result = service.refresh_token(req.into_inner()).await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(result)))
}

/// Logout
///
/// POST /api/auth/logout
#[utoipa::path(
    post,
    path = "/api/auth/logout",
    tag = "Auth",
    security(
        ("bearer_auth" = [])
    ),
    request_body = LogoutRequest,
    responses(
        (status = 200, description = "Logged out successfully", body = ApiResponse<MessageResponse>),
        (status = 401, description = "Unauthorized", body = ApiError)
    )
)]
pub async fn logout(
    service: web::Data<Arc<AuthService>>,
    req: web::Json<LogoutRequest>,
    _auth: AuthUser, // Require authentication
) -> Result<HttpResponse, ApiError> {
    // Validate request
    req.validate()?;

    // Call service
    service.logout(&req.refresh_token).await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(MessageResponse::new(
        "Logged out successfully",
    ))))
}

/// Get current user profile
///
/// GET /api/auth/me
#[utoipa::path(
    get,
    path = "/api/auth/me",
    tag = "Auth",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Current user profile", body = ApiResponse<UserResponse>),
        (status = 401, description = "Unauthorized", body = ApiError)
    )
)]
pub async fn get_me(auth: AuthUser) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({
        "user_id": auth.user_id,
        "email": auth.email,
        "role": auth.role,
    }))))
}

/// Change password
///
/// POST /api/auth/change-password
#[utoipa::path(
    post,
    path = "/api/auth/change-password",
    tag = "Auth",
    security(
        ("bearer_auth" = [])
    ),
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "Password changed successfully", body = ApiResponse<MessageResponse>),
        (status = 400, description = "Validation error", body = ApiError),
        (status = 401, description = "Unauthorized", body = ApiError)
    )
)]
pub async fn change_password(
    service: web::Data<Arc<AuthService>>,
    req: web::Json<ChangePasswordRequest>,
    auth: AuthUser,
) -> Result<HttpResponse, ApiError> {
    // Validate request
    req.validate()?;

    // Parse user ID
    let user_id = bson::oid::ObjectId::parse_str(&auth.user_id)
        .map_err(|_| ApiError::bad_request("Invalid user ID"))?;

    // Call service
    service.change_password(&user_id, req.into_inner()).await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(MessageResponse::new(
        "Password changed successfully",
    ))))
}

/// Assign roles to user (admin only)
///
/// POST /api/auth/admin/assign-roles
#[utoipa::path(
    post,
    path = "/api/auth/admin/assign-roles",
    tag = "Auth",
    security(
        ("bearer_auth" = [])
    ),
    request_body = AssignRoleRequest,
    responses(
        (status = 200, description = "Roles assigned successfully", body = ApiResponse<MessageResponse>),
        (status = 400, description = "Validation error", body = ApiError),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Forbidden - Admin only", body = ApiError)
    )
)]
#[protect("ADMIN", "SUPER_ADMIN")]
pub async fn assign_roles(
    service: web::Data<Arc<AuthService>>,
    req: web::Json<AssignRoleRequest>,
) -> Result<HttpResponse, ApiError> {
    // Validate request
    req.validate()?;

    let req = req.into_inner();

    // Parse user ID
    let user_id = bson::oid::ObjectId::parse_str(&req.user_id)
        .map_err(|_| ApiError::bad_request("Invalid user ID"))?;

    // Call service
    service.assign_roles(&user_id, req.roles).await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(MessageResponse::new(
        "Roles assigned successfully",
    ))))
}

/// Get user roles (admin only)
///
/// GET /api/auth/admin/users/{user_id}/roles
#[utoipa::path(
    get,
    path = "/api/auth/admin/users/{user_id}/roles",
    tag = "Auth",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("user_id" = String, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User roles retrieved", body = ApiResponse<UserRolesResponse>),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Forbidden - Admin only", body = ApiError),
        (status = 404, description = "User not found", body = ApiError)
    )
)]
#[protect("ADMIN", "SUPER_ADMIN")]
pub async fn get_user_roles(
    service: web::Data<Arc<AuthService>>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let user_id_str = path.into_inner();

    // Parse user ID
    let user_id = bson::oid::ObjectId::parse_str(&user_id_str)
        .map_err(|_| ApiError::bad_request("Invalid user ID"))?;

    // Get user from service
    let user = service
        .get_user_with_roles(&user_id)
        .await
        .map_err(|e| ApiError::database(e.to_string()))?;

    let response = UserRolesResponse {
        id: user.id.unwrap().to_hex(),
        email: user.email,
        name: user.name,
        role: user.role.clone(),
        roles: if user.roles.is_empty() { vec![user.role] } else { user.roles },
        perm_version: user.perm_version,
        status: format!("{:?}", user.status),
    };

    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}
