//! Authentication repository
//!
//! Handles database operations for users and refresh tokens.

use bson::{doc, oid::ObjectId, DateTime};
use mongodb::Collection;
use std::sync::Arc;

use crate::{
    core::DbError,
    database::{mongodb::collections, MongoDB},
};

use super::domain::{RefreshToken, User};

/// User repository
#[derive(Clone)]
pub struct UserRepository {
    collection: Collection<User>,
}

impl UserRepository {
    /// Creates a new user repository
    pub fn new(db: Arc<MongoDB>) -> Self {
        let collection = db.database().collection::<User>(collections::USERS);
        Self { collection }
    }

    /// Creates a new user
    ///
    /// # Arguments
    /// * `user` - User to create
    ///
    /// # Returns
    /// * `Result<User, DbError>` - Created user with ID
    pub async fn create(&self, mut user: User) -> Result<User, DbError> {
        let result = self.collection.insert_one(&user).await?;
        user.id = Some(result.inserted_id.as_object_id().unwrap());
        Ok(user)
    }

    /// Finds user by email
    ///
    /// # Arguments
    /// * `email` - User's email
    ///
    /// # Returns
    /// * `Result<Option<User>, DbError>` - User if found
    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, DbError> {
        self.collection
            .find_one(doc! { "email": email })
            .await
            .map_err(DbError::from)
    }

    /// Finds user by ID
    ///
    /// # Arguments
    /// * `id` - User's ObjectId
    ///
    /// # Returns
    /// * `Result<Option<User>, DbError>` - User if found
    pub async fn find_by_id(&self, id: &ObjectId) -> Result<Option<User>, DbError> {
        self.collection
            .find_one(doc! { "_id": id })
            .await
            .map_err(DbError::from)
    }

    /// Updates user's last login timestamp
    ///
    /// # Arguments
    /// * `id` - User's ObjectId
    pub async fn update_last_login(&self, id: &ObjectId) -> Result<(), DbError> {
        self.collection
            .update_one(
                doc! { "_id": id },
                doc! {
                    "$set": {
                        "last_login_at": DateTime::now(),
                        "updated_at": DateTime::now()
                    }
                },
            )
            .await?;
        Ok(())
    }

    /// Updates user's password
    ///
    /// # Arguments
    /// * `id` - User's ObjectId
    /// * `password_hash` - New password hash
    pub async fn update_password(&self, id: &ObjectId, password_hash: &str) -> Result<(), DbError> {
        self.collection
            .update_one(
                doc! { "_id": id },
                doc! {
                    "$set": {
                        "password_hash": password_hash,
                        "updated_at": DateTime::now()
                    }
                },
            )
            .await?;
        Ok(())
    }

    /// Checks if email exists
    ///
    /// # Arguments
    /// * `email` - Email to check
    ///
    /// # Returns
    /// * `Result<bool, DbError>` - True if exists
    pub async fn email_exists(&self, email: &str) -> Result<bool, DbError> {
        let count = self
            .collection
            .count_documents(doc! { "email": email })
            .await?;
        Ok(count > 0)
    }

    /// Updates user roles
    ///
    /// # Arguments
    /// * `id` - User's ObjectId
    /// * `role` - Primary role
    /// * `roles` - Array of all roles
    /// * `perm_version` - New permission version
    pub async fn update_roles(
        &self,
        id: &ObjectId,
        role: String,
        roles: Vec<String>,
        perm_version: u32,
    ) -> Result<(), DbError> {
        self.collection
            .update_one(
                doc! { "_id": id },
                doc! {
                    "$set": {
                        "role": role,
                        "roles": roles,
                        "perm_version": perm_version,
                        "updated_at": DateTime::now()
                    }
                },
            )
            .await?;
        Ok(())
    }
}

/// Refresh token repository
#[derive(Clone)]
pub struct RefreshTokenRepository {
    collection: Collection<RefreshToken>,
}

impl RefreshTokenRepository {
    /// Creates a new refresh token repository
    pub fn new(db: Arc<MongoDB>) -> Self {
        let collection = db
            .database()
            .collection::<RefreshToken>(collections::REFRESH_TOKENS);
        Self { collection }
    }

    /// Creates a new refresh token
    ///
    /// # Arguments
    /// * `token` - Refresh token to create
    pub async fn create(&self, token: RefreshToken) -> Result<RefreshToken, DbError> {
        self.collection.insert_one(&token).await?;
        Ok(token)
    }

    /// Finds refresh token by token string
    ///
    /// # Arguments
    /// * `token` - Token string to find
    ///
    /// # Returns
    /// * `Result<Option<RefreshToken>, DbError>` - Token if found
    pub async fn find_by_token(&self, token: &str) -> Result<Option<RefreshToken>, DbError> {
        self.collection
            .find_one(doc! { "token": token, "revoked": false })
            .await
            .map_err(DbError::from)
    }

    /// Revokes a refresh token
    ///
    /// # Arguments
    /// * `token` - Token string to revoke
    pub async fn revoke(&self, token: &str) -> Result<(), DbError> {
        self.collection
            .update_one(
                doc! { "token": token },
                doc! { "$set": { "revoked": true } },
            )
            .await?;
        Ok(())
    }

    /// Revokes all refresh tokens for a user
    ///
    /// # Arguments
    /// * `user_id` - User's ObjectId
    pub async fn revoke_all_for_user(&self, user_id: &ObjectId) -> Result<(), DbError> {
        self.collection
            .update_many(
                doc! { "user_id": user_id },
                doc! { "$set": { "revoked": true } },
            )
            .await?;
        Ok(())
    }

    /// Deletes expired tokens (cleanup job)
    pub async fn delete_expired(&self) -> Result<u64, DbError> {
        let now = DateTime::now();
        let result = self
            .collection
            .delete_many(doc! { "expires_at": { "$lt": now } })
            .await?;
        Ok(result.deleted_count)
    }
}
