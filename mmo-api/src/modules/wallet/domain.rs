//! Wallet domain models
//!
//! MongoDB document structures for wallet system.

use bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

/// Wallet document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub user_id: ObjectId,
    pub balances: WalletBalances,
    pub lifetime: LifetimeStats,
    pub currency: String,
    pub status: WalletStatus,
    pub frozen_amount: i64,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

/// Wallet balances
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletBalances {
    pub ap_current: i64,
    pub ap_pending_cashout: i64,
}

/// Lifetime statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifetimeStats {
    pub total_deposited: i64,
    pub total_withdrawn: i64,
    pub total_earned: i64,
    pub total_spent: i64,
    pub total_sent: i64,
    pub total_received: i64,
}

/// Wallet status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum WalletStatus {
    Active,
    Frozen,
    Suspended,
}

impl Default for WalletBalances {
    fn default() -> Self {
        Self {
            ap_current: 0,
            ap_pending_cashout: 0,
        }
    }
}

impl Default for LifetimeStats {
    fn default() -> Self {
        Self {
            total_deposited: 0,
            total_withdrawn: 0,
            total_earned: 0,
            total_spent: 0,
            total_sent: 0,
            total_received: 0,
        }
    }
}

impl Wallet {
    pub fn new(user_id: ObjectId) -> Self {
        let now = DateTime::now();
        Self {
            id: None,
            user_id,
            balances: WalletBalances::default(),
            lifetime: LifetimeStats::default(),
            currency: "VND".to_string(),
            status: WalletStatus::Active,
            frozen_amount: 0,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn total_balance(&self) -> i64 {
        self.balances.ap_current + self.balances.ap_pending_cashout
    }

    pub fn is_active(&self) -> bool {
        self.status == WalletStatus::Active
    }
}

// TODO: Add other wallet-related models:
// - WalletTransaction
// - EscrowHold
// - WithdrawalRequest
// - DepositRequest
// - OrderTypeConfig
// - MoneyFlowSummary
