//! Wallet repository
//!
//! Database operations for wallet system.

use bson::{doc, oid::ObjectId};
use mongodb::Collection;
use std::sync::Arc;

use crate::{
    core::DbError,
    database::{mongodb::collections, MongoDB},
};

use super::domain::Wallet;

/// Wallet repository
#[derive(Clone)]
pub struct WalletRepository {
    collection: Collection<Wallet>,
}

impl WalletRepository {
    pub fn new(db: Arc<MongoDB>) -> Self {
        let collection = db.database().collection::<Wallet>(collections::WALLETS);
        Self { collection }
    }

    /// Creates a new wallet
    pub async fn create(&self, wallet: Wallet) -> Result<Wallet, DbError> {
        self.collection.insert_one(&wallet, None).await?;
        Ok(wallet)
    }

    /// Finds wallet by user ID
    pub async fn find_by_user_id(&self, user_id: &ObjectId) -> Result<Option<Wallet>, DbError> {
        self.collection
            .find_one(doc! { "user_id": user_id }, None)
            .await
            .map_err(DbError::from)
    }

    /// Gets or creates wallet for user
    pub async fn get_or_create(&self, user_id: &ObjectId) -> Result<Wallet, DbError> {
        if let Some(wallet) = self.find_by_user_id(user_id).await? {
            return Ok(wallet);
        }

        let wallet = Wallet::new(*user_id);
        self.create(wallet).await
    }

    // TODO: Add other repository methods:
    // - update_balance (with session for transactions)
    // - transfer (atomic operation)
    // - freeze/unfreeze
}
