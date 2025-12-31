//! Wallet V3 Service Layer - PART 3: Admin Operations
//!
//! Admin-only operations: manual debit, freeze/unfreeze, commission config

use bson::DateTime as BsonDateTime;
use std::sync::Arc;

use crate::core::error::ServiceError;
use super::{dto::*, repository::WalletRepository, domain::*, service::WalletService};

impl WalletService {
    // ========================================================================
    // ADMIN: MANUAL DEBIT
    // ========================================================================

    /// Manual debit from wallet (admin only)
    pub async fn manual_debit(
        &self,
        req: ManualDebitRequest,
        admin_id: String,
    ) -> Result<SuccessResponse, ServiceError> {
        // Get wallet
        let mut wallet = self
            .repo
            .find_wallet_by_user_id(&req.user_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Wallet not found".to_string()))?;

        // Check if sufficient balance
        if wallet.available_trust < req.trust_amount {
            return Err(ServiceError::BadRequest(
                "Insufficient available balance".to_string(),
            ));
        }

        // Create transaction
        let tx_id = Self::generate_id("TXN");
        let balance_before = wallet.available_trust;
        let balance_after = balance_before - req.trust_amount;

        let tx = Transaction::new(
            tx_id.clone(),
            wallet.wallet_id.clone(),
            wallet.user_id.clone(),
            TransactionType::DebitManual,
            Direction::Debit,
            req.trust_amount,
            balance_before,
            balance_after,
            BalanceType::Available,
            admin_id.clone(),
        );

        // Update wallet
        wallet.available_trust -= req.trust_amount;
        wallet.total_trust -= req.trust_amount;

        // Validate invariant
        Self::validate_invariant(&wallet)?;

        // Save to database
        self.repo.create_transaction(tx).await?;
        self.repo.update_wallet(&wallet).await?;

        // Create admin log
        let log = AdminOperationLog {
            id: None,
            log_id: Self::generate_id("ALOG"),
            admin_id: admin_id.clone(),
            admin_email: "admin@example.com".to_string(),
            admin_role: "ADMIN".to_string(),
            operation: AdminOperation::ManualDebit,
            target_type: TargetType::Wallet,
            target_id: wallet.wallet_id.clone(),
            before_state: serde_json::json!({"available_trust": balance_before}),
            after_state: serde_json::json!({"available_trust": balance_after}),
            amount: Some(req.trust_amount),
            reason: req.reason,
            note: req.note,
            transaction_id: Some(tx_id),
            ip_address: "0.0.0.0".to_string(),
            user_agent: "".to_string(),
            created_at: BsonDateTime::now(),
        };
        self.repo.create_admin_log(log).await?;

        Ok(SuccessResponse::new(format!(
            "Successfully debited {} Trust from wallet",
            req.trust_amount
        )))
    }

    // ========================================================================
    // ADMIN: FREEZE/UNFREEZE WALLET
    // ========================================================================

    /// Freeze wallet (prevent all operations)
    pub async fn freeze_wallet(
        &self,
        req: FreezeWalletRequest,
        admin_id: String,
    ) -> Result<SuccessResponse, ServiceError> {
        // Get wallet
        let mut wallet = self
            .repo
            .find_wallet_by_user_id(&req.user_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Wallet not found".to_string()))?;

        // Check if already frozen
        if wallet.status == WalletStatus::Frozen {
            return Err(ServiceError::BadRequest(
                "Wallet is already frozen".to_string(),
            ));
        }

        let before_status = wallet.status.clone();
        wallet.status = WalletStatus::Frozen;
        wallet.updated_at = BsonDateTime::now();

        self.repo.update_wallet(&wallet).await?;

        // Create admin log
        let log = AdminOperationLog {
            id: None,
            log_id: Self::generate_id("ALOG"),
            admin_id: admin_id.clone(),
            admin_email: "admin@example.com".to_string(),
            admin_role: "ADMIN".to_string(),
            operation: AdminOperation::FreezeWallet,
            target_type: TargetType::Wallet,
            target_id: wallet.wallet_id.clone(),
            before_state: serde_json::json!({"status": before_status}),
            after_state: serde_json::json!({"status": WalletStatus::Frozen}),
            amount: None,
            reason: req.reason,
            note: req.note,
            transaction_id: None,
            ip_address: "0.0.0.0".to_string(),
            user_agent: "".to_string(),
            created_at: BsonDateTime::now(),
        };
        self.repo.create_admin_log(log).await?;

        Ok(SuccessResponse::new(format!(
            "Wallet {} has been frozen",
            wallet.wallet_id
        )))
    }

    /// Unfreeze wallet
    pub async fn unfreeze_wallet(
        &self,
        req: UnfreezeWalletRequest,
        admin_id: String,
    ) -> Result<SuccessResponse, ServiceError> {
        // Get wallet
        let mut wallet = self
            .repo
            .find_wallet_by_user_id(&req.user_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Wallet not found".to_string()))?;

        // Check if frozen
        if wallet.status != WalletStatus::Frozen {
            return Err(ServiceError::BadRequest(
                "Wallet is not frozen".to_string(),
            ));
        }

        let before_status = wallet.status.clone();
        wallet.status = WalletStatus::Active;
        wallet.updated_at = BsonDateTime::now();

        self.repo.update_wallet(&wallet).await?;

        // Create admin log
        let log = AdminOperationLog {
            id: None,
            log_id: Self::generate_id("ALOG"),
            admin_id: admin_id.clone(),
            admin_email: "admin@example.com".to_string(),
            admin_role: "ADMIN".to_string(),
            operation: AdminOperation::UnfreezeWallet,
            target_type: TargetType::Wallet,
            target_id: wallet.wallet_id.clone(),
            before_state: serde_json::json!({"status": before_status}),
            after_state: serde_json::json!({"status": WalletStatus::Active}),
            amount: None,
            reason: req.reason,
            note: req.note,
            transaction_id: None,
            ip_address: "0.0.0.0".to_string(),
            user_agent: "".to_string(),
            created_at: BsonDateTime::now(),
        };
        self.repo.create_admin_log(log).await?;

        Ok(SuccessResponse::new(format!(
            "Wallet {} has been unfrozen",
            wallet.wallet_id
        )))
    }

    // ========================================================================
    // ADMIN: COMMISSION CONFIGURATION
    // ========================================================================

    /// Set custom commission rate for a shop
    pub async fn set_shop_commission(
        &self,
        req: SetShopCommissionRequest,
        admin_id: String,
    ) -> Result<SuccessResponse, ServiceError> {
        // Validate rate
        if req.commission_rate < 0.01 || req.commission_rate > 0.20 {
            return Err(ServiceError::BadRequest(
                "Commission rate must be between 1% and 20%".to_string(),
            ));
        }

        // Get seller wallet
        let mut wallet = self
            .repo
            .find_wallet_by_user_id(&req.seller_user_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Seller wallet not found".to_string()))?;

        // Verify it's a seller wallet
        if wallet.wallet_type != WalletType::Seller {
            return Err(ServiceError::BadRequest(
                "Can only set commission for seller wallets".to_string(),
            ));
        }

        let before_rate = wallet.commission_rate;
        wallet.commission_rate = Some(req.commission_rate);
        wallet.updated_at = BsonDateTime::now();

        self.repo.update_wallet(&wallet).await?;

        // Save commission config
        let config = ShopCommissionConfig {
            id: None,
            config_id: Self::generate_id("COMM"),
            shop_id: req.shop_id.clone(),
            seller_user_id: req.seller_user_id.clone(),
            seller_wallet_id: wallet.wallet_id.clone(),
            commission_rate: req.commission_rate,
            effective_from: BsonDateTime::now(),
            reason: req.reason.clone(),
            set_by_admin_id: admin_id.clone(),
            created_at: BsonDateTime::now(),
        };
        self.repo.create_commission_config(config).await?;

        // Create admin log
        let log = AdminOperationLog {
            id: None,
            log_id: Self::generate_id("ALOG"),
            admin_id: admin_id.clone(),
            admin_email: "admin@example.com".to_string(),
            admin_role: "ADMIN".to_string(),
            operation: AdminOperation::SetCommission,
            target_type: TargetType::Wallet,
            target_id: wallet.wallet_id.clone(),
            before_state: serde_json::json!({"commission_rate": before_rate}),
            after_state: serde_json::json!({"commission_rate": req.commission_rate}),
            amount: None,
            reason: req.reason,
            note: None,
            transaction_id: None,
            ip_address: "0.0.0.0".to_string(),
            user_agent: "".to_string(),
            created_at: BsonDateTime::now(),
        };
        self.repo.create_admin_log(log).await?;

        Ok(SuccessResponse::new(format!(
            "Commission rate set to {:.1}% for shop {}",
            req.commission_rate * 100.0,
            req.shop_id
        )))
    }

    // ========================================================================
    // ADMIN: TRANSACTION HISTORY & LOGS
    // ========================================================================

    /// Get transaction history for a wallet
    pub async fn get_transaction_history(
        &self,
        wallet_id: &str,
        limit: Option<i64>,
    ) -> Result<TransactionHistoryResponse, ServiceError> {
        let limit = limit.unwrap_or(50).min(100); // Max 100
        let transactions = self.repo.get_recent_transactions(wallet_id, limit).await?;

        Ok(TransactionHistoryResponse {
            wallet_id: wallet_id.to_string(),
            transactions,
            count: transactions.len() as i64,
        })
    }

    /// Get admin operation logs
    pub async fn get_admin_logs(
        &self,
        target_id: Option<String>,
        limit: Option<i64>,
    ) -> Result<AdminLogResponse, ServiceError> {
        let limit = limit.unwrap_or(50).min(100);
        let logs = if let Some(tid) = target_id {
            self.repo.get_admin_logs_by_target(&tid, limit).await?
        } else {
            self.repo.get_recent_admin_logs(limit).await?
        };

        Ok(AdminLogResponse {
            logs,
            count: logs.len() as i64,
        })
    }

    // ========================================================================
    // ADMIN: WITHDRAWAL APPROVAL
    // ========================================================================

    /// Approve withdrawal request (admin)
    pub async fn approve_withdrawal(
        &self,
        request_id: &str,
        admin_id: String,
    ) -> Result<SuccessResponse, ServiceError> {
        // Get withdrawal request
        let mut req = self
            .repo
            .find_withdrawal_by_id(request_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Withdrawal request not found".to_string()))?;

        // Check status
        if req.status != WithdrawalStatus::Pending && req.status != WithdrawalStatus::Validated {
            return Err(ServiceError::BadRequest(
                "Can only approve pending or validated withdrawals".to_string(),
            ));
        }

        // Update status
        req.status = WithdrawalStatus::Approved;
        req.approved_by = Some(admin_id.clone());
        req.approved_at = Some(BsonDateTime::now());
        req.updated_at = BsonDateTime::now();

        self.repo.update_withdrawal(&req).await?;

        // Create admin log
        let log = AdminOperationLog {
            id: None,
            log_id: Self::generate_id("ALOG"),
            admin_id: admin_id.clone(),
            admin_email: "admin@example.com".to_string(),
            admin_role: "ADMIN".to_string(),
            operation: AdminOperation::ApproveWithdrawal,
            target_type: TargetType::Withdrawal,
            target_id: req.request_id.clone(),
            before_state: serde_json::json!({"status": "PENDING"}),
            after_state: serde_json::json!({"status": "APPROVED"}),
            amount: Some(req.trust_amount),
            reason: "Withdrawal approved".to_string(),
            note: None,
            transaction_id: None,
            ip_address: "0.0.0.0".to_string(),
            user_agent: "".to_string(),
            created_at: BsonDateTime::now(),
        };
        self.repo.create_admin_log(log).await?;

        Ok(SuccessResponse::new(format!(
            "Withdrawal {} approved. Ready for bank transfer.",
            request_id
        )))
    }

    /// Reject withdrawal request (admin)
    pub async fn reject_withdrawal(
        &self,
        request_id: &str,
        req: RejectWithdrawalRequest,
        admin_id: String,
    ) -> Result<SuccessResponse, ServiceError> {
        // Get withdrawal request
        let mut withdrawal_req = self
            .repo
            .find_withdrawal_by_id(request_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Withdrawal request not found".to_string()))?;

        // Check status
        if withdrawal_req.status == WithdrawalStatus::Completed
            || withdrawal_req.status == WithdrawalStatus::Rejected
        {
            return Err(ServiceError::BadRequest(
                "Cannot reject completed or already rejected withdrawal".to_string(),
            ));
        }

        // Get wallet to unlock funds
        let mut wallet = self
            .repo
            .find_wallet_by_id(&withdrawal_req.wallet_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Wallet not found".to_string()))?;

        // Use MongoDB transaction
        let mut session = self.repo.start_session().await?;
        session.start_transaction(None).await?;

        // Unlock funds: withdrawal_locked -> available
        let tx_id = Self::generate_id("TXN");
        let balance_before = wallet.withdrawal_locked;

        wallet.withdrawal_locked -= withdrawal_req.trust_amount;
        wallet.available_trust += withdrawal_req.trust_amount;
        Self::validate_invariant(&wallet)?;

        let unlock_tx = Transaction::new(
            tx_id.clone(),
            wallet.wallet_id.clone(),
            wallet.user_id.clone(),
            TransactionType::WithdrawalRejected,
            Direction::Credit,
            withdrawal_req.trust_amount,
            balance_before,
            wallet.withdrawal_locked,
            BalanceType::WithdrawalLocked,
            admin_id.clone(),
        );

        // Update withdrawal status
        withdrawal_req.status = WithdrawalStatus::Rejected;
        withdrawal_req.updated_at = BsonDateTime::now();
        withdrawal_req
            .validation_errors
            .push(req.rejection_reason.clone());

        // Save
        self.repo
            .create_transaction_with_session(unlock_tx, &mut session)
            .await?;
        self.repo
            .update_wallet_with_session(&wallet, &mut session)
            .await?;
        self.repo
            .update_withdrawal_with_session(&withdrawal_req, &mut session)
            .await?;

        // Create admin log
        let log = AdminOperationLog {
            id: None,
            log_id: Self::generate_id("ALOG"),
            admin_id: admin_id.clone(),
            admin_email: "admin@example.com".to_string(),
            admin_role: "ADMIN".to_string(),
            operation: AdminOperation::RejectWithdrawal,
            target_type: TargetType::Withdrawal,
            target_id: withdrawal_req.request_id.clone(),
            before_state: serde_json::json!({"status": "PENDING"}),
            after_state: serde_json::json!({"status": "REJECTED"}),
            amount: Some(withdrawal_req.trust_amount),
            reason: req.rejection_reason,
            note: None,
            transaction_id: Some(tx_id),
            ip_address: "0.0.0.0".to_string(),
            user_agent: "".to_string(),
            created_at: BsonDateTime::now(),
        };
        self.repo
            .create_admin_log_with_session(log, &mut session)
            .await?;

        session.commit_transaction().await?;

        Ok(SuccessResponse::new(format!(
            "Withdrawal {} rejected. Funds unlocked and returned to available balance.",
            request_id
        )))
    }

    /// Complete bank transfer (admin marks as completed)
    pub async fn complete_bank_transfer(
        &self,
        request_id: &str,
        req: CompleteBankTransferRequest,
        admin_id: String,
    ) -> Result<SuccessResponse, ServiceError> {
        // Get withdrawal request
        let mut withdrawal_req = self
            .repo
            .find_withdrawal_by_id(request_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Withdrawal request not found".to_string()))?;

        // Check status - must be approved
        if withdrawal_req.status != WithdrawalStatus::Approved {
            return Err(ServiceError::BadRequest(
                "Can only complete approved withdrawals".to_string(),
            ));
        }

        // Get wallet
        let mut wallet = self
            .repo
            .find_wallet_by_id(&withdrawal_req.wallet_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Wallet not found".to_string()))?;

        // Use MongoDB transaction
        let mut session = self.repo.start_session().await?;
        session.start_transaction(None).await?;

        let now = BsonDateTime::now();

        // Unlock and remove funds: withdrawal_locked -> (removed from total)
        let tx_id = Self::generate_id("TXN");
        let balance_before = wallet.withdrawal_locked;

        wallet.withdrawal_locked -= withdrawal_req.trust_amount;
        wallet.total_trust -= withdrawal_req.trust_amount;
        wallet.lifetime_withdrawn += withdrawal_req.trust_amount;

        // Deduct commission debt if any
        if withdrawal_req.commission_deduct > 0 {
            wallet.commission_debt -= withdrawal_req.commission_deduct;
        }

        Self::validate_invariant(&wallet)?;

        let complete_tx = Transaction::new(
            tx_id.clone(),
            wallet.wallet_id.clone(),
            wallet.user_id.clone(),
            TransactionType::WithdrawalCompleted,
            Direction::Debit,
            withdrawal_req.trust_amount,
            balance_before,
            wallet.withdrawal_locked,
            BalanceType::WithdrawalLocked,
            admin_id.clone(),
        );

        // Update withdrawal status
        withdrawal_req.status = WithdrawalStatus::Completed;
        withdrawal_req.bank_transfer_ref = Some(req.bank_transfer_ref);
        withdrawal_req.bank_transfer_at = Some(now);
        withdrawal_req.completed_at = Some(now);
        withdrawal_req.updated_at = now;

        // Save
        self.repo
            .create_transaction_with_session(complete_tx, &mut session)
            .await?;
        self.repo
            .update_wallet_with_session(&wallet, &mut session)
            .await?;
        self.repo
            .update_withdrawal_with_session(&withdrawal_req, &mut session)
            .await?;

        // Create admin log
        let log = AdminOperationLog {
            id: None,
            log_id: Self::generate_id("ALOG"),
            admin_id: admin_id.clone(),
            admin_email: "admin@example.com".to_string(),
            admin_role: "ADMIN".to_string(),
            operation: AdminOperation::CompleteBankTransfer,
            target_type: TargetType::Withdrawal,
            target_id: withdrawal_req.request_id.clone(),
            before_state: serde_json::json!({"status": "APPROVED"}),
            after_state: serde_json::json!({"status": "COMPLETED"}),
            amount: Some(withdrawal_req.net_trust),
            reason: "Bank transfer completed".to_string(),
            note: Some(format!("Bank ref: {}", req.bank_transfer_ref)),
            transaction_id: Some(tx_id),
            ip_address: "0.0.0.0".to_string(),
            user_agent: "".to_string(),
            created_at: now,
        };
        self.repo
            .create_admin_log_with_session(log, &mut session)
            .await?;

        session.commit_transaction().await?;

        Ok(SuccessResponse::new(format!(
            "Bank transfer completed. {} VND sent to user. Commission deducted: {} Trust.",
            withdrawal_req.vnd_amount, withdrawal_req.commission_deduct
        )))
    }
}
