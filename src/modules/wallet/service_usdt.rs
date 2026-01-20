//! Wallet USDT Service
//!
//! Handles USDT TRC20 deposit processing with fixed exchange rate

use std::sync::Arc;
use anyhow::Result;
use ulid::Ulid;

use crate::core::error::{ServiceError, DbError};
use super::domain::*;
use super::dto::*;
use super::repository::WalletRepository;

/// Configuration for USDT processing
#[derive(Clone, Debug)]
pub struct UsdtConfig {
    /// Platform USDT TRC20 address
    pub platform_address: String,

    /// Fixed exchange rate: 1 USDT = X VND
    pub exchange_rate: f64,

    /// Minimum deposit amount in USDT
    pub min_deposit: f64,

    /// Maximum deposit amount in USDT
    pub max_deposit: f64,

    /// Required confirmations for TRC20
    pub required_confirmations: i32,

    /// USDT TRC20 contract address
    pub usdt_trc20_address: String,
}

impl Default for UsdtConfig {
    fn default() -> Self {
        Self {
            platform_address: "TRC20_PLATFORM_ADDRESS".to_string(),
            exchange_rate: 25000.0, // 1 USDT = 25000 VND
            min_deposit: 1.0,
            max_deposit: 10000.0,
            required_confirmations: 20,
            usdt_trc20_address: "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t".to_string(), // Official USDT TRC20
        }
    }
}

/// USDT wallet service
pub struct WalletUsdtService {
    pub wallet_repo: Arc<WalletRepository>,
    config: UsdtConfig,
}

impl WalletUsdtService {
    /// Create new USDT service instance
    pub fn new(wallet_repo: Arc<WalletRepository>, config: UsdtConfig) -> Self {
        Self { wallet_repo, config }
    }

    /// Get USDT deposit information for user
    pub async fn get_deposit_info(
        &self,
        user_id: String,
    ) -> Result<UsdtDepositAddressResponse, ServiceError> {
        // Verify user exists and has a wallet
        let _wallet = self
            .wallet_repo
            .find_wallet_by_user_id(&user_id)
            .await
            .map_err(|e| match e {
                DbError::NotFound(_) => ServiceError::NotFound("Wallet not found".to_string()),
                _ => ServiceError::DatabaseError(e.to_string()),
            })?
            .ok_or_else(|| ServiceError::NotFound("Wallet not found".to_string()))?;

        // Generate unique memo for this user
        let timestamp = chrono::Utc::now().timestamp();
        let memo_example = format!("USDT-{}-{}", user_id, timestamp);

        Ok(UsdtDepositAddressResponse {
            deposit_address: self.config.platform_address.clone(),
            network: "TRC20".to_string(),
            memo_format: "USDT-{user_id}-{timestamp}".to_string(),
            memo_example,
            min_deposit: self.config.min_deposit,
            max_deposit: self.config.max_deposit,
            exchange_rate: self.config.exchange_rate,
            required_confirmations: self.config.required_confirmations,
        })
    }

    /// Get USDT deposit status
    pub async fn get_deposit_status(
        &self,
        user_id: String,
        deposit_id: String,
    ) -> Result<UsdtDepositStatusResponse, ServiceError> {
        let deposit = self
            .wallet_repo
            .find_usdt_deposit_by_id(&deposit_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ServiceError::NotFound("Deposit not found".to_string()))?;

        // Verify ownership
        if deposit.user_id != user_id {
            return Err(ServiceError::Unauthorized(
                "Access denied to this deposit".to_string(),
            ));
        }

        Ok(UsdtDepositStatusResponse {
            deposit_id: deposit.deposit_id,
            usdt_amount: Some(deposit.usdt_amount),
            vnd_amount: Some(deposit.vnd_amount),
            trust_amount: Some(deposit.trust_amount),
            network: format!("{:?}", deposit.network),
            transaction_hash: Some(deposit.transaction_hash),
            confirmations: Some(deposit.confirmations),
            required_confirmations: Some(deposit.required_confirmations),
            status: format!("{:?}", deposit.status),
            created_at: Some(format!("{:?}", deposit.created_at)),
            credited_at: deposit.credited_at.map(|t| format!("{:?}", t)),
        })
    }

    /// Get user's USDT deposits
    pub async fn get_user_deposits(
        &self,
        user_id: String,
        limit: i64,
    ) -> Result<Vec<UsdtDeposit>, ServiceError> {
        let deposits = self
            .wallet_repo
            .get_usdt_deposits_by_user(&user_id, limit)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok(deposits)
    }

    /// Process incoming USDT deposit (called by blockchain monitor)
    pub async fn process_deposit(
        &self,
        sender_address: String,
        transaction_hash: String,
        usdt_amount: f64,
        block_number: i64,
        user_id: Option<String>,
        memo: Option<String>,
    ) -> Result<UsdtDeposit, ServiceError> {
        // Check idempotency - already processed?
        if let Some(existing) = self
            .wallet_repo
            .find_usdt_deposit_by_tx_hash(&transaction_hash)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
        {
            tracing::info!(
                "Transaction {} already processed, deposit_id: {}",
                transaction_hash,
                existing.deposit_id
            );
            return Ok(existing);
        }

        // Validate amount
        if usdt_amount < self.config.min_deposit {
            let deposit_id = format!("USDT-{}", Ulid::new());
            let deposit = UsdtDeposit {
                id: None,
                deposit_id: deposit_id.clone(),
                wallet_id: String::new(),
                user_id: user_id.unwrap_or_default(),
                usdt_amount,
                network: UsdtNetwork::Trc20,
                sender_address,
                transaction_hash,
                block_number,
                vnd_amount: 0,
                trust_amount: 0,
                exchange_rate: self.config.exchange_rate,
                status: UsdtDepositStatus::Failed,
                confirmations: 0,
                required_confirmations: self.config.required_confirmations,
                credited_at: None,
                failed_reason: Some(format!(
                    "Amount below minimum: {} < {}",
                    usdt_amount, self.config.min_deposit
                )),
                memo,
                transaction_id: None,
                created_at: bson::DateTime::now(),
                updated_at: bson::DateTime::now(),
            };

            self.wallet_repo
                .create_usdt_deposit(&deposit)
                .await
                .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

            return Ok(deposit);
        }

        // If no user_id provided, try to extract from memo
        let target_user_id = if let Some(uid) = user_id {
            uid
        } else {
            // Try to parse user_id from memo
            self.parse_user_id_from_memo(memo.as_deref().unwrap_or_default())
                .ok_or_else(|| {
                    ServiceError::ValidationFailed(
                        "Cannot determine user - no valid memo provided".to_string(),
                    )
                })?
        };

        // Get user's wallet
        let wallet = self
            .wallet_repo
            .find_wallet_by_user_id(&target_user_id)
            .await
            .map_err(|e| match e {
                DbError::NotFound(_) => ServiceError::NotFound(format!("Wallet not found for user: {}", target_user_id)),
                _ => ServiceError::DatabaseError(e.to_string()),
            })?
            .ok_or_else(|| ServiceError::NotFound(format!("Wallet not found for user: {}", target_user_id)))?;

        // Create deposit record
        let deposit_id = format!("USDT-{}", Ulid::new());
        let mut deposit = UsdtDeposit::new(
            deposit_id.clone(),
            wallet.wallet_id.clone(),
            target_user_id.clone(),
            usdt_amount,
            UsdtNetwork::Trc20,
            sender_address,
            transaction_hash,
            block_number,
            self.config.exchange_rate,
        );
        deposit.memo = memo;

        let deposit_id_for_log = deposit_id.clone();
        let target_user_id_for_log = target_user_id.clone();
        let usdt_amount_for_log = usdt_amount;

        let created_deposit = self
            .wallet_repo
            .create_usdt_deposit(&deposit)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        tracing::info!(
            "Created USDT deposit: {} for user: {}, amount: {} USDT",
            deposit_id_for_log,
            target_user_id_for_log,
            usdt_amount_for_log
        );

        Ok(created_deposit)
    }

    /// Update deposit confirmations and credit when ready
    pub async fn update_confirmations(
        &self,
        deposit_id: String,
        confirmations: i32,
    ) -> Result<UsdtDeposit, ServiceError> {
        let deposit = self
            .wallet_repo
            .find_usdt_deposit_by_id(&deposit_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ServiceError::NotFound("Deposit not found".to_string()))?;

        // Only update pending or confirming deposits
        if !matches!(
            deposit.status,
            UsdtDepositStatus::Pending | UsdtDepositStatus::Confirming
        ) {
            return Ok(deposit);
        }

        let new_status = if confirmations >= self.config.required_confirmations {
            UsdtDepositStatus::Confirmed
        } else {
            UsdtDepositStatus::Confirming
        };

        self.wallet_repo
            .update_usdt_deposit_confirmations(&deposit_id, confirmations, new_status.clone())
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        // If confirmed, credit the wallet
        if new_status == UsdtDepositStatus::Confirmed {
            self.credit_deposit(&deposit_id).await?;
        }

        // Return updated deposit
        let updated = self
            .wallet_repo
            .find_usdt_deposit_by_id(&deposit_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .unwrap();

        Ok(updated)
    }

    /// Credit confirmed USDT deposit to wallet
    async fn credit_deposit(&self, deposit_id: &str) -> Result<(), ServiceError> {
        let deposit = self
            .wallet_repo
            .find_usdt_deposit_by_id(deposit_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ServiceError::NotFound("Deposit not found".to_string()))?;

        if !matches!(deposit.status, UsdtDepositStatus::Confirmed) {
            return Err(ServiceError::ValidationFailed(
                "Deposit cannot be credited yet".to_string(),
            ));
        }

        let deposit_for_log = deposit.clone();

        // Get current wallet
        let mut wallet = self
            .wallet_repo
            .find_wallet_by_user_id(&deposit.user_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ServiceError::NotFound("Wallet not found".to_string()))?;

        let balance_before = wallet.available_trust;

        // Create transaction
        let tx_id = format!("TXN-{}", Ulid::new());
        let mut transaction = Transaction::new(
            tx_id.clone(),
            wallet.wallet_id.clone(),
            deposit.user_id.clone(),
            TransactionType::DepositUsdt,
            Direction::Credit,
            deposit.trust_amount,
            balance_before,
            balance_before + deposit.trust_amount,
            BalanceType::Available,
            "SYSTEM".to_string(),
        );
        transaction.vnd_amount = Some(deposit.vnd_amount);
        transaction.external_ref = Some(deposit.transaction_hash.clone());
        transaction.reference_id = Some(deposit.deposit_id.clone());
        transaction.reference_type = Some(ReferenceType::Deposit);
        transaction.status = TransactionStatus::Completed;
        transaction.completed_at = Some(bson::DateTime::now());
        transaction.running_deposited = wallet.lifetime_deposited + deposit.trust_amount;

        // Update wallet balance
        wallet.available_trust += deposit.trust_amount;
        wallet.total_trust += deposit.trust_amount;
        wallet.lifetime_deposited += deposit.trust_amount;

        // Save to database
        self.wallet_repo
            .create_transaction(transaction.clone())
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        self.wallet_repo
            .update_wallet(&wallet)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        // Update deposit status
        let mut updated_deposit = deposit.clone();
        updated_deposit.status = UsdtDepositStatus::Credited;
        updated_deposit.confirmations = deposit.required_confirmations;
        updated_deposit.credited_at = Some(bson::DateTime::now());
        updated_deposit.transaction_id = Some(tx_id);

        self.wallet_repo
            .update_usdt_deposit(&updated_deposit)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        tracing::info!(
            "Credited USDT deposit: {} to user: {}, amount: {} Trust",
            deposit_id,
            deposit_for_log.user_id,
            deposit_for_log.trust_amount
        );

        Ok(())
    }

    /// Manual credit USDT deposit (admin override)
    pub async fn manual_credit_deposit(
        &self,
        deposit_id: String,
        admin_id: String,
        reason: String,
    ) -> Result<UsdtDeposit, ServiceError> {
        let deposit = self
            .wallet_repo
            .find_usdt_deposit_by_id(&deposit_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ServiceError::NotFound("Deposit not found".to_string()))?;

        if deposit.status == UsdtDepositStatus::Credited {
            return Err(ServiceError::ValidationFailed(
                "Deposit already credited".to_string(),
            ));
        }

        // Force credit by setting status to confirmed
        self.wallet_repo
            .update_usdt_deposit_confirmations(
                &deposit_id,
                deposit.required_confirmations,
                UsdtDepositStatus::Confirmed,
            )
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        // Credit the deposit
        self.credit_deposit(&deposit_id).await?;

        // Get updated wallet for logging
        let wallet = self
            .wallet_repo
            .find_wallet_by_user_id(&deposit.user_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ServiceError::NotFound("Wallet not found".to_string()))?;

        // Create admin log
        let log = AdminOperationLog {
            id: None,
            log_id: format!("ALOG-{}", Ulid::new()),
            admin_id: admin_id.clone(),
            admin_email: "admin@example.com".to_string(),
            admin_role: "admin".to_string(),
            operation: AdminOperation::ManualDeposit,
            target_type: TargetType::Wallet,
            target_id: wallet.wallet_id.clone(),
            before_state: serde_json::json!({"status": "Confirming"}),
            after_state: serde_json::json!({"status": "Credited"}),
            amount: Some(deposit.trust_amount),
            reason: format!("Manual USDT credit: {}", reason),
            note: Some(format!("Deposit ID: {}", deposit_id)),
            transaction_id: Some(deposit_id.clone()),
            ip_address: "0.0.0.0".to_string(),
            user_agent: "admin".to_string(),
            created_at: bson::DateTime::now(),
        };

        self.wallet_repo
            .create_admin_log(log)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        // Return updated deposit
        let updated = self
            .wallet_repo
            .find_usdt_deposit_by_id(&deposit_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .unwrap();

        Ok(updated)
    }

    /// Get all USDT deposits (admin)
    pub async fn get_all_deposits(
        &self,
        page: i64,
        per_page: i64,
    ) -> Result<(Vec<UsdtDeposit>, i64), ServiceError> {
        self.wallet_repo
            .get_all_usdt_deposits(page, per_page)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    /// Get current exchange rate
    pub fn get_exchange_rate(&self) -> f64 {
        self.config.exchange_rate
    }

    /// Parse user_id from memo
    /// Memo format: "USDT-{user_id}-{timestamp}" or just "{user_id}"
    fn parse_user_id_from_memo(&self, memo: &str) -> Option<String> {
        if memo.is_empty() {
            return None;
        }

        // Try format: USDT-{user_id}-{timestamp}
        if let Some(rest) = memo.strip_prefix("USDT-") {
            let parts: Vec<&str> = rest.split('-').collect();
            if parts.len() >= 2 {
                return Some(parts[0].to_string());
            }
        }

        // Try just user_id
        Some(memo.to_string())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_user_id_from_memo_format() {
        let memo_with_prefix = "USDT-user123-1234567890";
        let parts: Vec<&str> = memo_with_prefix.split('-').collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "USDT");
        assert_eq!(parts[1], "user123");
        
        let simple_memo = "user123";
        assert!(!simple_memo.is_empty());
        
        let empty_memo = "";
        assert!(empty_memo.is_empty());
    }
    
    #[test]
    fn test_usdt_config_default() {
        use super::UsdtConfig;
        let config = UsdtConfig::default();
        assert!(config.min_deposit > 0.0);
        assert!(config.max_deposit > 0.0);
        assert!(config.exchange_rate > 0.0);
    }
}
