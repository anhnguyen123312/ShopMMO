//! Authentication DTOs (Data Transfer Objects)
//!
//! Request and response structures for authentication endpoints.

use serde::{Deserialize, Serialize};
use validator::Validate;
use utoipa::ToSchema;

/// Register request
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    /// User's email
    #[validate(email(message = "Invalid email format"))]
    #[schema(example = "user@example.com")]
    pub email: String,

    /// User's password
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    #[schema(example = "password123", min_length = 8)]
    pub password: String,

    /// User's display name
    #[validate(length(min = 2, max = 50, message = "Name must be 2-50 characters"))]
    #[schema(example = "John Doe", min_length = 2, max_length = 50)]
    pub name: String,
}

/// Login request
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    /// User's email
    #[validate(email(message = "Invalid email format"))]
    #[schema(example = "user@example.com")]
    pub email: String,

    /// User's password
    #[validate(length(min = 1, message = "Password is required"))]
    #[schema(example = "password123")]
    pub password: String,
}

/// Refresh token request
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RefreshTokenRequest {
    /// Refresh token
    #[validate(length(min = 1, message = "Refresh token is required"))]
    pub refresh_token: String,
}

/// Authentication response (login/register)
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    /// Access token (JWT)
    pub access_token: String,

    /// Refresh token (JWT)
    pub refresh_token: String,

    /// Token type (always "Bearer")
    #[schema(example = "Bearer")]
    pub token_type: String,

    /// Access token expiration in seconds
    #[schema(example = 900)]
    pub expires_in: i64,

    /// User information
    pub user: UserResponse,
}

impl AuthResponse {
    pub fn new(
        access_token: String,
        refresh_token: String,
        expires_in: i64,
        user: UserResponse,
    ) -> Self {
        Self {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
            user,
        }
    }
}

/// User response
#[derive(Debug, Serialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    /// User ID
    #[schema(example = "60f1b5b5b5b5b5b5b5b5b5b5")]
    pub id: String,

    /// User's email
    #[schema(example = "user@example.com")]
    pub email: String,

    /// User's display name
    #[schema(example = "John Doe")]
    pub name: String,

    /// User's role
    #[schema(example = "user")]
    pub role: String,

    /// Email verification status
    pub email_verified: bool,

    /// Account creation timestamp
    pub created_at: String,
}

impl From<crate::modules::auth::domain::User> for UserResponse {
    fn from(user: crate::modules::auth::domain::User) -> Self {
        Self {
            id: user.id.unwrap().to_hex(),
            email: user.email,
            name: user.name,
            role: user.role,
            email_verified: user.email_verified,
            created_at: user.created_at.to_chrono().to_rfc3339(),
        }
    }
}

/// Change password request
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChangePasswordRequest {
    /// Current password
    #[validate(length(min = 1, message = "Current password is required"))]
    pub current_password: String,

    /// New password
    #[validate(length(min = 8, message = "New password must be at least 8 characters"))]
    #[schema(min_length = 8)]
    pub new_password: String,
}

/// Logout request
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LogoutRequest {
    /// Refresh token to revoke
    #[validate(length(min = 1, message = "Refresh token is required"))]
    pub refresh_token: String,
}
