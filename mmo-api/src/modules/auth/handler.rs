//! Authentication handlers
//!
//! HTTP request handlers for authentication endpoints.

use actix_web::{web, HttpRequest, HttpResponse};
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
