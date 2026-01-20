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

    /// User's username (unique)
    pub username: String,

    /// User's email (unique)
    pub email: String,

    /// Hashed password (bcrypt)
    pub password_hash: String,

    /// User's display name
    pub name: String,

    /// User's role (deprecated - kept for backward compatibility)
    pub role: String,

    /// User's roles - array of role names for V2 authorization
    #[serde(default)]
    pub roles: Vec<String>,

    /// Permission version - increments when permissions change for cache invalidation
    #[serde(default)]
    pub perm_version: u32,

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
#[derive(Default)]
pub enum UserStatus {
    Active,
    Suspended,
    #[default]
    PendingVerification,
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
    /// * `username` - User's unique username
    /// * `email` - User's email
    /// * `password_hash` - Hashed password
    /// * `name` - User's display name
    /// * `role` - User's primary role (default: "BUYER")
    /// * `roles` - Optional array of additional roles
    pub fn new(
        username: String,
        email: String,
        password_hash: String,
        name: String,
        role: Option<String>,
        roles: Option<Vec<String>>,
    ) -> Self {
        let now = DateTime::now();
        let default_role = role.unwrap_or_else(|| "BUYER".to_string());
        let default_roles = roles.unwrap_or_else(|| vec![default_role.clone()]);
        Self {
            id: None,
            username,
            email,
            password_hash,
            name,
            role: default_role,
            roles: default_roles,
            perm_version: 1,
            status: UserStatus::PendingVerification,
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

    /// Checks if token is expired
    pub fn is_expired(&self) -> bool {
        self.expires_at < DateTime::now()
    }

    /// Checks if token is valid (not revoked and not expired)
    pub fn is_valid(&self) -> bool {
        !self.revoked && !self.is_expired()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== UserStatus Tests ====================

    #[test]
    fn test_user_status_default_is_pending_verification() {
        let status = UserStatus::default();
        assert_eq!(status, UserStatus::PendingVerification);
    }

    #[test]
    fn test_user_status_equality() {
        assert_eq!(UserStatus::Active, UserStatus::Active);
        assert_eq!(UserStatus::Suspended, UserStatus::Suspended);
        assert_eq!(
            UserStatus::PendingVerification,
            UserStatus::PendingVerification
        );
        assert_ne!(UserStatus::Active, UserStatus::Suspended);
        assert_ne!(UserStatus::Active, UserStatus::PendingVerification);
    }

    // ==================== User::new Tests ====================

    #[test]
    fn test_user_new_with_defaults() {
        let user = User::new(
            "testuser".to_string(),
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            "Test User".to_string(),
            None,
            None,
        );

        assert!(user.id.is_none());
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.password_hash, "hashed_password");
        assert_eq!(user.name, "Test User");
        assert_eq!(user.role, "BUYER");
        assert_eq!(user.roles, vec!["BUYER"]);
        assert_eq!(user.perm_version, 1);
        assert_eq!(user.status, UserStatus::PendingVerification);
        assert!(!user.email_verified);
        assert!(user.last_login_at.is_none());
    }

    #[test]
    fn test_user_new_with_custom_role() {
        let user = User::new(
            "seller1".to_string(),
            "seller@example.com".to_string(),
            "hashed_password".to_string(),
            "Seller One".to_string(),
            Some("SELLER".to_string()),
            None,
        );

        assert_eq!(user.role, "SELLER");
        assert_eq!(user.roles, vec!["SELLER"]);
    }

    #[test]
    fn test_user_new_with_custom_roles_array() {
        let user = User::new(
            "admin1".to_string(),
            "admin@example.com".to_string(),
            "hashed_password".to_string(),
            "Admin One".to_string(),
            Some("ADMIN".to_string()),
            Some(vec!["ADMIN".to_string(), "MODERATOR".to_string()]),
        );

        assert_eq!(user.role, "ADMIN");
        assert_eq!(user.roles, vec!["ADMIN", "MODERATOR"]);
    }

    #[test]
    fn test_user_new_timestamps_are_set() {
        let before = DateTime::now();
        let user = User::new(
            "timeuser".to_string(),
            "time@example.com".to_string(),
            "hash".to_string(),
            "Time User".to_string(),
            None,
            None,
        );
        let after = DateTime::now();

        // created_at and updated_at should be between before and after
        assert!(user.created_at >= before);
        assert!(user.created_at <= after);
        assert!(user.updated_at >= before);
        assert!(user.updated_at <= after);
    }

    // ==================== User::is_active Tests ====================

    #[test]
    fn test_user_is_active_when_active() {
        let mut user = User::new(
            "activeuser".to_string(),
            "active@example.com".to_string(),
            "hash".to_string(),
            "Active User".to_string(),
            None,
            None,
        );
        user.status = UserStatus::Active;

        assert!(user.is_active());
    }

    #[test]
    fn test_user_is_not_active_when_suspended() {
        let mut user = User::new(
            "suspendeduser".to_string(),
            "suspended@example.com".to_string(),
            "hash".to_string(),
            "Suspended User".to_string(),
            None,
            None,
        );
        user.status = UserStatus::Suspended;

        assert!(!user.is_active());
    }

    #[test]
    fn test_user_is_not_active_when_pending_verification() {
        let user = User::new(
            "pendinguser".to_string(),
            "pending@example.com".to_string(),
            "hash".to_string(),
            "Pending User".to_string(),
            None,
            None,
        );
        // Default status is PendingVerification

        assert!(!user.is_active());
    }

    // ==================== RefreshToken::new Tests ====================

    #[test]
    fn test_refresh_token_new_basic() {
        let user_id = ObjectId::new();
        let expires_at = DateTime::now();

        let token = RefreshToken::new(
            user_id,
            "jwt_token_string".to_string(),
            expires_at,
            None,
            None,
        );

        assert!(token.id.is_none());
        assert_eq!(token.user_id, user_id);
        assert_eq!(token.token, "jwt_token_string");
        assert_eq!(token.expires_at, expires_at);
        assert!(!token.revoked);
        assert!(token.ip_address.is_none());
        assert!(token.user_agent.is_none());
    }

    #[test]
    fn test_refresh_token_new_with_metadata() {
        let user_id = ObjectId::new();
        let expires_at = DateTime::now();

        let token = RefreshToken::new(
            user_id,
            "jwt_token_string".to_string(),
            expires_at,
            Some("192.168.1.1".to_string()),
            Some("Mozilla/5.0".to_string()),
        );

        assert_eq!(token.ip_address, Some("192.168.1.1".to_string()));
        assert_eq!(token.user_agent, Some("Mozilla/5.0".to_string()));
    }

    #[test]
    fn test_refresh_token_created_at_is_set() {
        let user_id = ObjectId::new();
        let before = DateTime::now();

        let token = RefreshToken::new(
            user_id,
            "jwt_token".to_string(),
            DateTime::now(),
            None,
            None,
        );

        let after = DateTime::now();

        assert!(token.created_at >= before);
        assert!(token.created_at <= after);
    }

    // ==================== RefreshToken::is_expired Tests ====================

    #[test]
    fn test_refresh_token_is_expired_when_past() {
        let user_id = ObjectId::new();
        // Set expires_at to 1 hour ago
        let past = DateTime::from_millis(DateTime::now().timestamp_millis() - 3600 * 1000);

        let token = RefreshToken::new(user_id, "expired_token".to_string(), past, None, None);

        assert!(token.is_expired());
    }

    #[test]
    fn test_refresh_token_is_not_expired_when_future() {
        let user_id = ObjectId::new();
        // Set expires_at to 1 hour from now
        let future = DateTime::from_millis(DateTime::now().timestamp_millis() + 3600 * 1000);

        let token = RefreshToken::new(user_id, "valid_token".to_string(), future, None, None);

        assert!(!token.is_expired());
    }

    // ==================== RefreshToken::is_valid Tests ====================

    #[test]
    fn test_refresh_token_is_valid_when_not_revoked_and_not_expired() {
        let user_id = ObjectId::new();
        let future = DateTime::from_millis(DateTime::now().timestamp_millis() + 3600 * 1000);

        let token = RefreshToken::new(user_id, "valid_token".to_string(), future, None, None);

        assert!(token.is_valid());
    }

    #[test]
    fn test_refresh_token_is_not_valid_when_revoked() {
        let user_id = ObjectId::new();
        let future = DateTime::from_millis(DateTime::now().timestamp_millis() + 3600 * 1000);

        let mut token = RefreshToken::new(user_id, "revoked_token".to_string(), future, None, None);
        token.revoked = true;

        assert!(!token.is_valid());
    }

    #[test]
    fn test_refresh_token_is_not_valid_when_expired() {
        let user_id = ObjectId::new();
        let past = DateTime::from_millis(DateTime::now().timestamp_millis() - 3600 * 1000);

        let token = RefreshToken::new(user_id, "expired_token".to_string(), past, None, None);

        assert!(!token.is_valid());
    }

    #[test]
    fn test_refresh_token_is_not_valid_when_both_revoked_and_expired() {
        let user_id = ObjectId::new();
        let past = DateTime::from_millis(DateTime::now().timestamp_millis() - 3600 * 1000);

        let mut token = RefreshToken::new(user_id, "bad_token".to_string(), past, None, None);
        token.revoked = true;

        assert!(!token.is_valid());
    }
}
