//! Authentication service
//!
//! Business logic for authentication operations.

use bson::oid::ObjectId;
use std::sync::Arc;

use crate::{
    config::AppConfig,
    core::ServiceError,
    utils::{self, hash_password, verify_password},
};

use super::{
    domain::{RefreshToken, User},
    dto::*,
    repository::{RefreshTokenRepository, UserRepository},
};

/// Authentication service
#[derive(Clone)]
pub struct AuthService {
    user_repo: Arc<UserRepository>,
    token_repo: Arc<RefreshTokenRepository>,
    config: Arc<AppConfig>,
}

impl AuthService {
    pub fn new(
        user_repo: Arc<UserRepository>,
        token_repo: Arc<RefreshTokenRepository>,
        config: Arc<AppConfig>,
    ) -> Self {
        Self {
            user_repo,
            token_repo,
            config,
        }
    }

    /// Registers a new user
    ///
    /// # Arguments
    /// * `req` - Registration request
    ///
    /// # Returns
    /// * `Result<AuthResponse, ServiceError>` - Authentication response with tokens
    pub async fn register(&self, req: RegisterRequest) -> Result<AuthResponse, ServiceError> {
        // Check if email already exists
        if self.user_repo.email_exists(&req.email).await.map_err(|e| ServiceError::DatabaseError(e.to_string()))? {
            return Err(ServiceError::ValidationFailed(
                "Email already registered".to_string(),
            ));
        }

        // Hash password
        let password_hash = hash_password(&req.password, Some(self.config.security.bcrypt_cost))
            .map_err(|e| ServiceError::InternalError(format!("Failed to hash password: {}", e)))?;

        // Create user
        let user = User::new(req.email, password_hash, req.name, None);
        let created_user = self
            .user_repo
            .create(user)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        // Generate tokens
        let auth_response = self.generate_tokens(&created_user, None, None).await?;

        tracing::info!(
            user_id = %created_user.id.unwrap().to_hex(),
            email = %created_user.email,
            "User registered successfully"
        );

        Ok(auth_response)
    }

    /// Logs in a user
    ///
    /// # Arguments
    /// * `req` - Login request
    /// * `ip_address` - Optional IP address
    /// * `user_agent` - Optional user agent
    ///
    /// # Returns
    /// * `Result<AuthResponse, ServiceError>` - Authentication response with tokens
    pub async fn login(
        &self,
        req: LoginRequest,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<AuthResponse, ServiceError> {
        // Find user by email
        let user = self
            .user_repo
            .find_by_email(&req.email)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ServiceError::ValidationFailed("Invalid credentials".to_string()))?;

        // Verify password
        let is_valid = verify_password(&req.password, &user.password_hash)
            .map_err(|e| ServiceError::InternalError(format!("Failed to verify password: {}", e)))?;

        if !is_valid {
            return Err(ServiceError::ValidationFailed(
                "Invalid credentials".to_string(),
            ));
        }

        // Check if account is active
        if !user.is_active() {
            return Err(ServiceError::ValidationFailed(
                "Account is not active".to_string(),
            ));
        }

        // Update last login
        if let Some(id) = user.id {
            self.user_repo
                .update_last_login(&id)
                .await
                .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        }

        // Generate tokens
        let auth_response = self.generate_tokens(&user, ip_address, user_agent).await?;

        tracing::info!(
            user_id = %user.id.unwrap().to_hex(),
            email = %user.email,
            "User logged in successfully"
        );

        Ok(auth_response)
    }

    /// Refreshes access token
    ///
    /// # Arguments
    /// * `req` - Refresh token request
    ///
    /// # Returns
    /// * `Result<AuthResponse, ServiceError>` - New authentication response
    pub async fn refresh_token(&self, req: RefreshTokenRequest) -> Result<AuthResponse, ServiceError> {
        // Verify refresh token JWT
        let claims = utils::verify_token(&req.refresh_token, &self.config.jwt.secret)
            .map_err(|_| ServiceError::Unauthorized("Invalid refresh token".to_string()))?;

        if claims.token_type != "refresh" {
            return Err(ServiceError::Unauthorized("Invalid token type".to_string()));
        }

        // Check if token exists and not revoked in database
        let token_doc = self
            .token_repo
            .find_by_token(&req.refresh_token)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or(ServiceError::Unauthorized("Token not found or revoked".to_string()))?;

        // Get user
        let user = self
            .user_repo
            .find_by_id(&token_doc.user_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or(ServiceError::NotFound("User not found".to_string()))?;

        // Generate new tokens
        let auth_response = self
            .generate_tokens(&user, token_doc.ip_address, token_doc.user_agent)
            .await?;

        // Revoke old refresh token
        self.token_repo
            .revoke(&req.refresh_token)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok(auth_response)
    }

    /// Logs out a user by revoking refresh token
    ///
    /// # Arguments
    /// * `refresh_token` - Refresh token to revoke
    pub async fn logout(&self, refresh_token: &str) -> Result<(), ServiceError> {
        self.token_repo
            .revoke(refresh_token)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        tracing::info!("User logged out successfully");
        Ok(())
    }

    /// Changes user password
    ///
    /// # Arguments
    /// * `user_id` - User's ObjectId
    /// * `req` - Change password request
    pub async fn change_password(
        &self,
        user_id: &ObjectId,
        req: ChangePasswordRequest,
    ) -> Result<(), ServiceError> {
        // Get user
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ServiceError::NotFound("User not found".to_string()))?;

        // Verify current password
        let is_valid = verify_password(&req.current_password, &user.password_hash)
            .map_err(|e| ServiceError::InternalError(format!("Failed to verify password: {}", e)))?;

        if !is_valid {
            return Err(ServiceError::ValidationFailed(
                "Current password is incorrect".to_string(),
            ));
        }

        // Hash new password
        let new_hash = hash_password(&req.new_password, Some(self.config.security.bcrypt_cost))
            .map_err(|e| ServiceError::InternalError(format!("Failed to hash password: {}", e)))?;

        // Update password
        self.user_repo
            .update_password(user_id, &new_hash)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        // Revoke all refresh tokens for security
        self.token_repo
            .revoke_all_for_user(user_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        tracing::info!(user_id = %user_id.to_hex(), "Password changed successfully");
        Ok(())
    }

    /// Generates access and refresh tokens
    async fn generate_tokens(
        &self,
        user: &User,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<AuthResponse, ServiceError> {
        let user_id = user.id.unwrap().to_hex();

        // Parse token expiration
        let access_expires = utils::jwt::parse_duration(&self.config.jwt.access_token_expires_in);
        let refresh_expires_days = utils::jwt::parse_duration(&self.config.jwt.refresh_token_expires_in) / (60 * 24);

        // TODO: Get actual wallet_id from wallet service
        // For now, use user_id as wallet_id (wallet will be created on first access)
        let wallet_id = format!("WLT-{}", user_id);

        // Generate access token
        let access_token = utils::generate_access_token(
            &user_id,
            &wallet_id,
            &user.email,
            &user.role,
            &self.config.jwt.secret,
            access_expires,
        )
        .map_err(|e| ServiceError::InternalError(e.to_string()))?;

        // Generate refresh token
        let refresh_token = utils::generate_refresh_token(
            &user_id,
            &wallet_id,
            &user.email,
            &user.role,
            &self.config.jwt.secret,
            refresh_expires_days,
        )
        .map_err(|e| ServiceError::InternalError(e.to_string()))?;

        // Store refresh token in database
        let refresh_token_doc = RefreshToken::new(
            user.id.unwrap(),
            refresh_token.clone(),
            utils::add_days(refresh_expires_days),
            ip_address,
            user_agent,
        );

        self.token_repo
            .create(refresh_token_doc)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok(AuthResponse::new(
            access_token,
            refresh_token,
            access_expires * 60, // Convert to seconds
            UserResponse::from(user.clone()),
        ))
    }
}
