//! Wallet service
//!
//! Business logic for wallet operations.

use bson::oid::ObjectId;
use std::sync::Arc;

use crate::core::ServiceError;

use super::{dto::*, repository::WalletRepository};

/// Wallet service
#[derive(Clone)]
pub struct WalletService {
    repo: Arc<WalletRepository>,
}

impl WalletService {
    pub fn new(repo: Arc<WalletRepository>) -> Self {
        Self { repo }
    }

    /// Gets wallet balance
    pub async fn get_balance(
        &self,
        user_id: &ObjectId,
    ) -> Result<WalletBalanceResponse, ServiceError> {
        let wallet = self
            .repo
            .get_or_create(user_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let ap_total = wallet.total_balance();

        Ok(WalletBalanceResponse {
            ap_current: wallet.balances.ap_current,
            ap_pending_cashout: wallet.balances.ap_pending_cashout,
            ap_total,
            vnd_equivalent: ap_total * 1000, // 1 AP = 1000 VND
        })
    }

    // TODO: Implement other service methods:
    // - transfer_ap
    // - request_withdrawal
    // - get_transaction_history
    // - deposit (admin)
}
