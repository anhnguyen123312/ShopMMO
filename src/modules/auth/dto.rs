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
    /// User's unique username
    #[validate(length(min = 3, max = 30, message = "Username must be 3-30 characters"))]
    #[schema(example = "johndoe123", min_length = 3, max_length = 30)]
    pub username: String,

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
    /// User's username or email
    #[validate(length(min = 1, message = "Username or email is required"))]
    #[schema(example = "johndoe123")]
    pub identifier: String,

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

    /// User's username
    #[schema(example = "johndoe123")]
    pub username: String,

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
            username: user.username,
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

/// Assign role request (admin only)
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AssignRoleRequest {
    /// User ID to assign role to
    #[validate(length(min = 1, message = "User ID is required"))]
    pub user_id: String,

    /// Roles to assign (e.g., ["BUYER", "SELLER"])
    #[validate(length(min = 1, message = "At least one role is required"))]
    pub roles: Vec<String>,
}

/// User with roles response
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserRolesResponse {
    /// User ID
    pub id: String,

    /// User's username
    pub username: String,

    /// User's email
    pub email: String,

    /// User's display name
    pub name: String,

    /// Primary role (backward compatibility)
    pub role: String,

    /// All assigned roles
    pub roles: Vec<String>,

    /// Permission version
    pub perm_version: u32,

    /// Account status
    pub status: String,
}
