//! Authentication domain models
//!
//! Contains MongoDB document structures for users and refresh tokens.

use bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

/// User document in MongoDB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// MongoDB ObjectId
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// User's email (unique)
    pub email: String,

    /// Hashed password (bcrypt)
    pub password_hash: String,

    /// User's display name
    pub name: String,

    /// User's role
    pub role: String,

    /// Account status
    pub status: UserStatus,

    /// Email verification status
    pub email_verified: bool,

    /// Last login timestamp
    pub last_login_at: Option<DateTime>,

    /// Created timestamp
    pub created_at: DateTime,

    /// Updated timestamp
    pub updated_at: DateTime,
}

/// User account status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    Active,
    Suspended,
    Inactive,
}

impl Default for UserStatus {
    fn default() -> Self {
        Self::Active
    }
}

/// Refresh token document in MongoDB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshToken {
    /// MongoDB ObjectId
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// User ID this token belongs to
    pub user_id: ObjectId,

    /// The actual JWT refresh token
    pub token: String,

    /// Expiration timestamp
    pub expires_at: DateTime,

    /// Whether token has been revoked
    pub revoked: bool,

    /// Created timestamp
    pub created_at: DateTime,

    /// IP address when token was created
    pub ip_address: Option<String>,

    /// User agent when token was created
    pub user_agent: Option<String>,
}

impl User {
    /// Creates a new user
    ///
    /// # Arguments
    /// * `email` - User's email
    /// * `password_hash` - Hashed password
    /// * `name` - User's display name
    /// * `role` - User's role (default: "user")
    pub fn new(email: String, password_hash: String, name: String, role: Option<String>) -> Self {
        let now = DateTime::now();
        Self {
            id: None,
            email,
            password_hash,
            name,
            role: role.unwrap_or_else(|| "user".to_string()),
            status: UserStatus::Active,
            email_verified: false,
            last_login_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Checks if user is active
    pub fn is_active(&self) -> bool {
        self.status == UserStatus::Active
    }
}

impl RefreshToken {
    /// Creates a new refresh token
    ///
    /// # Arguments
    /// * `user_id` - User's ObjectId
    /// * `token` - JWT refresh token string
    /// * `expires_at` - Token expiration time
    /// * `ip_address` - Optional IP address
    /// * `user_agent` - Optional user agent
    pub fn new(
        user_id: ObjectId,
        token: String,
        expires_at: DateTime,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        Self {
            id: None,
            user_id,
            token,
            expires_at,
            revoked: false,
            created_at: DateTime::now(),
            ip_address,
            user_agent,
        }
    }
}
