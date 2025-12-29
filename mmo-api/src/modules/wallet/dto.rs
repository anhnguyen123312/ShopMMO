//! Wallet DTOs
//!
//! Request and response structures for wallet endpoints.

use serde::{Deserialize, Serialize};
use validator::Validate;

/// Get wallet balance response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletBalanceResponse {
    pub ap_current: i64,
    pub ap_pending_cashout: i64,
    pub ap_total: i64,
    pub vnd_equivalent: i64,
}

/// Transfer AP request
#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct TransferRequest {
    #[validate(length(equal = 24, message = "Invalid user ID format"))]
    pub to_user_id: String,

    #[validate(range(min = 1, message = "Amount must be positive"))]
    pub amount: i64,

    #[validate(length(max = 200, message = "Message too long"))]
    pub message: Option<String>,
}

// TODO: Add other DTOs:
// - DepositRequest
// - WithdrawalRequest
// - TransactionHistoryQuery
// - TransactionResponse
