//! JWT utilities for token generation and validation
//!
//! Handles creation and verification of JSON Web Tokens for authentication.

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::core::ApiError;

/// JWT token claims
///
/// Contains the standard JWT claims plus custom user information.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenClaims {
    /// Subject (user ID)
    pub sub: String,

    /// User's wallet ID (kept for backward compatibility)
    #[serde(default)]
    pub wallet_id: String,

    /// User email (kept for backward compatibility)
    pub email: String,

    /// User roles - array of role names (e.g., ["buyer", "seller"])
    /// DEPRECATED: Use `roles` instead. Single `role` field kept for backward compatibility.
    #[serde(default)]
    pub role: String,

    /// Multiple roles support for V2 authorization
    #[serde(default)]
    pub roles: Vec<String>,

    /// Permission version for cache invalidation
    /// When user's permissions change, this increments to invalidate Redis cache
    #[serde(default)]
    pub perm_version: u32,

    /// Issued at (timestamp)
    pub iat: i64,

    /// Expiration time (timestamp)
    pub exp: i64,

    /// Token type (access or refresh)
    pub token_type: String,
}

/// Generates an access token (legacy - single role)
///
/// # Arguments
/// * `user_id` - User's MongoDB ObjectId as string
/// * `wallet_id` - User's wallet ID
/// * `email` - User's email
/// * `role` - User's single role
/// * `secret` - JWT secret key
/// * `expires_in_minutes` - Token expiration time in minutes
///
/// # Returns
/// * `Result<String, ApiError>` - JWT token or error
///
/// # Examples
/// ```
/// let token = generate_access_token(
///     "507f1f77bcf86cd799439011",
///     "WLT-507f1f77bcf86cd799439011",
///     "user@example.com",
///     "user",
///     &config.jwt.secret,
///     15
/// )?;
/// ```
pub fn generate_access_token(
    user_id: &str,
    wallet_id: &str,
    email: &str,
    role: &str,
    secret: &str,
    expires_in_minutes: i64,
) -> Result<String, ApiError> {
    let now = Utc::now();
    let exp = now + Duration::minutes(expires_in_minutes);

    let claims = TokenClaims {
        sub: user_id.to_string(),
        wallet_id: wallet_id.to_string(),
        email: email.to_string(),
        role: role.to_string(),
        roles: vec![role.to_string()], // Also populate roles array
        perm_version: 0,
        iat: now.timestamp(),
        exp: exp.timestamp(),
        token_type: "access".to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| ApiError::internal(format!("Failed to generate access token: {}", e)))
}

/// Generates an access token with V2 authorization support
///
/// # Arguments
/// * `user_id` - User's MongoDB ObjectId as string
/// * `wallet_id` - User's wallet ID
/// * `email` - User's email
/// * `roles` - Array of role names (e.g., ["buyer", "seller"])
/// * `perm_version` - Permission version for cache invalidation
/// * `secret` - JWT secret key
/// * `expires_in_minutes` - Token expiration time in minutes
///
/// # Returns
/// * `Result<String, ApiError>` - JWT token or error
pub fn generate_access_token_v2(
    user_id: &str,
    wallet_id: &str,
    email: &str,
    roles: Vec<String>,
    perm_version: u32,
    secret: &str,
    expires_in_minutes: i64,
) -> Result<String, ApiError> {
    let now = Utc::now();
    let exp = now + Duration::minutes(expires_in_minutes);

    // For backward compatibility, set role to the first/highest priority role
    let primary_role = roles.first().unwrap_or(&String::from("user")).clone();

    let claims = TokenClaims {
        sub: user_id.to_string(),
        wallet_id: wallet_id.to_string(),
        email: email.to_string(),
        role: primary_role,
        roles,
        perm_version,
        iat: now.timestamp(),
        exp: exp.timestamp(),
        token_type: "access".to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| ApiError::internal(format!("Failed to generate access token: {}", e)))
}

/// Generates a refresh token
///
/// # Arguments
/// * `user_id` - User's MongoDB ObjectId as string
/// * `wallet_id` - User's wallet ID
/// * `email` - User's email
/// * `role` - User's single role (for backward compatibility)
/// * `secret` - JWT secret key
/// * `expires_in_days` - Token expiration time in days
///
/// # Returns
/// * `Result<String, ApiError>` - JWT token or error
///
/// # Examples
/// ```
/// let token = generate_refresh_token(
///     "507f1f77bcf86cd799439011",
///     "WLT-507f1f77bcf86cd799439011",
///     "user@example.com",
///     "user",
///     &config.jwt.secret,
///     7
/// )?;
/// ```
pub fn generate_refresh_token(
    user_id: &str,
    wallet_id: &str,
    email: &str,
    role: &str,
    secret: &str,
    expires_in_days: i64,
) -> Result<String, ApiError> {
    let now = Utc::now();
    let exp = now + Duration::days(expires_in_days);

    let claims = TokenClaims {
        sub: user_id.to_string(),
        wallet_id: wallet_id.to_string(),
        email: email.to_string(),
        role: role.to_string(),
        roles: vec![role.to_string()],
        perm_version: 0,
        iat: now.timestamp(),
        exp: exp.timestamp(),
        token_type: "refresh".to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| ApiError::internal(format!("Failed to generate refresh token: {}", e)))
}

/// Generates a refresh token with V2 authorization support
///
/// # Arguments
/// * `user_id` - User's MongoDB ObjectId as string
/// * `wallet_id` - User's wallet ID
/// * `email` - User's email
/// * `roles` - Array of role names
/// * `perm_version` - Permission version
/// * `secret` - JWT secret key
/// * `expires_in_days` - Token expiration time in days
///
/// # Returns
/// * `Result<String, ApiError>` - JWT token or error
pub fn generate_refresh_token_v2(
    user_id: &str,
    wallet_id: &str,
    email: &str,
    roles: Vec<String>,
    perm_version: u32,
    secret: &str,
    expires_in_days: i64,
) -> Result<String, ApiError> {
    let now = Utc::now();
    let exp = now + Duration::days(expires_in_days);

    let primary_role = roles.first().unwrap_or(&String::from("user")).clone();

    let claims = TokenClaims {
        sub: user_id.to_string(),
        wallet_id: wallet_id.to_string(),
        email: email.to_string(),
        role: primary_role,
        roles,
        perm_version,
        iat: now.timestamp(),
        exp: exp.timestamp(),
        token_type: "refresh".to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| ApiError::internal(format!("Failed to generate refresh token: {}", e)))
}

/// Validates and decodes a JWT token
///
/// # Arguments
/// * `token` - JWT token string
/// * `secret` - JWT secret key
///
/// # Returns
/// * `Result<TokenClaims, ApiError>` - Decoded claims or error
///
/// # Examples
/// ```
/// let claims = verify_token(&token, &config.jwt.secret)?;
/// println!("User ID: {}", claims.sub);
/// ```
pub fn verify_token(token: &str, secret: &str) -> Result<TokenClaims, ApiError> {
    let validation = Validation::default();

    decode::<TokenClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|e| {
        tracing::warn!(error = %e, "Token verification failed");
        ApiError::unauthorized(format!("Invalid token: {}", e))
    })
}

/// Parses duration string (e.g., "15m", "7d") to minutes
///
/// # Arguments
/// * `duration_str` - Duration string (format: number + unit)
///   - Supported units: m (minutes), h (hours), d (days)
///
/// # Returns
/// * `i64` - Duration in minutes
///
/// # Examples
/// ```
/// assert_eq!(parse_duration("15m"), 15);
/// assert_eq!(parse_duration("2h"), 120);
/// assert_eq!(parse_duration("7d"), 10080);
/// ```
pub fn parse_duration(duration_str: &str) -> i64 {
    let len = duration_str.len();
    if len < 2 {
        return 15; // Default to 15 minutes
    }

    let (num_str, unit) = duration_str.split_at(len - 1);
    let num = num_str.parse::<i64>().unwrap_or(15);

    match unit {
        "m" => num,
        "h" => num * 60,
        "d" => num * 60 * 24,
        _ => 15,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SECRET: &str = "test_secret_key_for_jwt";

    #[test]
    fn test_generate_and_verify_access_token() {
        let token = generate_access_token(
            "507f1f77bcf86cd799439011",
            "WLT-507f1f77bcf86cd799439011",
            "test@example.com",
            "user",
            TEST_SECRET,
            15,
        )
        .unwrap();

        let claims = verify_token(&token, TEST_SECRET).unwrap();
        assert_eq!(claims.sub, "507f1f77bcf86cd799439011");
        assert_eq!(claims.wallet_id, "WLT-507f1f77bcf86cd799439011");
        assert_eq!(claims.email, "test@example.com");
        assert_eq!(claims.role, "user");
        assert_eq!(claims.token_type, "access");
    }

    #[test]
    fn test_generate_and_verify_access_token_v2() {
        let roles = vec!["buyer".to_string(), "seller".to_string()];
        let token = generate_access_token_v2(
            "507f1f77bcf86cd799439011",
            "WLT-507f1f77bcf86cd799439011",
            "test@example.com",
            roles.clone(),
            5,
            TEST_SECRET,
            15,
        )
        .unwrap();

        let claims = verify_token(&token, TEST_SECRET).unwrap();
        assert_eq!(claims.sub, "507f1f77bcf86cd799439011");
        assert_eq!(claims.wallet_id, "WLT-507f1f77bcf86cd799439011");
        assert_eq!(claims.email, "test@example.com");
        assert_eq!(claims.role, "buyer"); // Primary role
        assert_eq!(claims.roles, roles);
        assert_eq!(claims.perm_version, 5);
        assert_eq!(claims.token_type, "access");
    }

    #[test]
    fn test_jwt_claims_with_roles_array() {
        let claims = TokenClaims {
            sub: "user123".to_string(),
            wallet_id: "WLT-user123".to_string(),
            email: "user@example.com".to_string(),
            role: "seller".to_string(),
            roles: vec!["buyer".to_string(), "seller".to_string()],
            perm_version: 5,
            iat: 1704067200,
            exp: 1735689600,
            token_type: "access".to_string(),
        };

        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.roles.len(), 2);
        assert!(claims.roles.contains(&"seller".to_string()));
        assert_eq!(claims.perm_version, 5);
        assert_eq!(claims.role, "seller");
    }

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("15m"), 15);
        assert_eq!(parse_duration("2h"), 120);
        assert_eq!(parse_duration("7d"), 10080);
        assert_eq!(parse_duration("x"), 15); // Invalid unit returns default
        assert_eq!(parse_duration(""), 15); // Empty string returns default
    }

    #[test]
    fn test_verify_invalid_token() {
        let result = verify_token("invalid.token.here", TEST_SECRET);
        assert!(result.is_err());
    }
}
