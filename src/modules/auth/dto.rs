//! Authentication DTOs (Data Transfer Objects)
//!
//! Request and response structures for authentication endpoints.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// Register request
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    /// User's unique username (3-30 characters, alphanumeric and underscore)
    #[validate(length(min = 3, max = 30, message = "Username must be 3-30 characters"))]
    #[schema(example = "johndoe123", min_length = 3, max_length = 30)]
    pub username: String,

    /// User's email address
    #[validate(email(message = "Invalid email format"))]
    #[schema(example = "john.doe@example.com")]
    pub email: String,

    /// User's password (minimum 8 characters)
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    #[schema(example = "SecurePass123!", min_length = 8)]
    pub password: String,

    /// User's display name
    #[validate(length(min = 2, max = 50, message = "Name must be 2-50 characters"))]
    #[schema(example = "John Doe", min_length = 2, max_length = 50)]
    pub name: String,
}

/// Login request - authenticate with username/email and password
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    /// User's username or email address
    #[validate(length(min = 1, message = "Username or email is required"))]
    #[schema(example = "johndoe123")]
    pub identifier: String,

    /// User's password
    #[validate(length(min = 1, message = "Password is required"))]
    #[schema(example = "SecurePass123!")]
    pub password: String,
}

/// Refresh token request - exchange refresh token for new access token
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RefreshTokenRequest {
    /// Valid refresh token (JWT)
    #[validate(length(min = 1, message = "Refresh token is required"))]
    #[schema(
        example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiI2NWY4YTFiMmMzZDRlNWY2YTdiOGM5ZDAiLCJ0b2tlbl90eXBlIjoicmVmcmVzaCIsImV4cCI6MTcxMTIzNDU2N30.abc123"
    )]
    pub refresh_token: String,
}

/// Authentication response (login/register)
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    /// Access token (JWT) - use in Authorization header
    #[schema(
        example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiI2NWY4YTFiMmMzZDRlNWY2YTdiOGM5ZDAiLCJ1c2VybmFtZSI6ImpvaG5kb2UxMjMiLCJyb2xlIjoiQlVZRVIiLCJleHAiOjE3MTEyMzQ1Njd9.xyz789"
    )]
    pub access_token: String,

    /// Refresh token (JWT) - use to get new access token
    #[schema(
        example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiI2NWY4YTFiMmMzZDRlNWY2YTdiOGM5ZDAiLCJ0b2tlbl90eXBlIjoicmVmcmVzaCIsImV4cCI6MTcxMTgzOTM2N30.def456"
    )]
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
    /// User ID (MongoDB ObjectId)
    #[schema(example = "65f8a1b2c3d4e5f6a7b8c9d0")]
    pub id: String,

    /// User's username
    #[schema(example = "johndoe123")]
    pub username: String,

    /// User's email
    #[schema(example = "john.doe@example.com")]
    pub email: String,

    /// User's display name
    #[schema(example = "John Doe")]
    pub name: String,

    /// User's primary role
    #[schema(example = "BUYER")]
    pub role: String,

    /// Email verification status
    #[schema(example = true)]
    pub email_verified: bool,

    /// Account creation timestamp (ISO 8601)
    #[schema(example = "2024-01-15T10:30:00Z")]
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
    #[schema(example = "OldPass123!")]
    pub current_password: String,

    /// New password
    #[validate(length(min = 8, message = "New password must be at least 8 characters"))]
    #[schema(example = "NewSecurePass456!", min_length = 8)]
    pub new_password: String,
}

/// Logout request
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LogoutRequest {
    /// Refresh token to revoke
    #[validate(length(min = 1, message = "Refresh token is required"))]
    #[schema(
        example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiI2NWY4YTFiMmMzZDRlNWY2YTdiOGM5ZDAiLCJ0b2tlbl90eXBlIjoicmVmcmVzaCIsImV4cCI6MTcxMTgzOTM2N30.def456"
    )]
    pub refresh_token: String,
}

/// Assign role request (admin only)
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AssignRoleRequest {
    /// User ID to assign role to
    #[validate(length(min = 1, message = "User ID is required"))]
    #[schema(example = "65f8a1b2c3d4e5f6a7b8c9d0")]
    pub user_id: String,

    /// Roles to assign (e.g., ["BUYER", "SELLER"])
    #[validate(length(min = 1, message = "At least one role is required"))]
    #[schema(example = json!(["BUYER", "SELLER"]))]
    pub roles: Vec<String>,
}

/// User with roles response
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserRolesResponse {
    /// User ID
    #[schema(example = "65f8a1b2c3d4e5f6a7b8c9d0")]
    pub id: String,

    /// User's username
    #[schema(example = "johndoe123")]
    pub username: String,

    /// User's email
    #[schema(example = "john.doe@example.com")]
    pub email: String,

    /// User's display name
    #[schema(example = "John Doe")]
    pub name: String,

    /// Primary role (backward compatibility)
    #[schema(example = "SELLER")]
    pub role: String,

    /// All assigned roles
    #[schema(example = json!(["BUYER", "SELLER"]))]
    pub roles: Vec<String>,

    /// Permission version
    #[schema(example = 1)]
    pub perm_version: u32,

    /// Account status
    #[schema(example = "Active")]
    pub status: String,
}
