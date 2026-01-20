use bson::DateTime as BsonDateTime;

use crate::core::error::ServiceError;
use super::{dto::*, domain::*, service::WalletService};

impl WalletService {
    pub async fn manual_debit(
        &self,
        req: ManualDebitRequest,
        admin_id: String,
    ) -> Result<AdminDebitResponse, ServiceError> {
        let mut wallet = self
            .repo
            .find_wallet_by_user_id(&req.user_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Wallet not found".to_string()))?;

        let available = wallet.available_trust;
        let requested = req.trust_amount;

        let (actual_deduct, debt_amount) = if available >= requested {
            (requested, 0)
        } else if req.allow_debt {
            (available, requested - available)
        } else {
            return Err(ServiceError::BadRequest(format!(
                "Insufficient balance. Available: {} Trust, Requested: {} Trust. Enable allow_debt to create debt.",
                available, requested
            )));
        };

        let mut session = self.repo.start_session().await?;
        session.start_transaction().await
            .map_err(|e| ServiceError::DatabaseError(format!("Failed to start transaction: {}", e)))?;

        let balance_before = wallet.available_trust;
        let mut debt_id: Option<String> = None;

        if actual_deduct > 0 {
            let tx_id = Self::generate_id("TXN");
            let balance_after = balance_before - actual_deduct;

            let tx = Transaction::new(
                tx_id.clone(),
                wallet.wallet_id.clone(),
                wallet.user_id.clone(),
                TransactionType::AdminDebit,
                Direction::Debit,
                actual_deduct,
                balance_before,
                balance_after,
                BalanceType::Available,
                admin_id.clone(),
            );

            wallet.available_trust -= actual_deduct;
            wallet.total_trust -= actual_deduct;

            self.repo.create_transaction_with_session(tx, &mut session).await?;
        }

        if debt_amount > 0 {
            let new_debt_id = Self::generate_id("DEBT");
            debt_id = Some(new_debt_id.clone());

            let debt_tx = AdminDebtTransaction::new(
                new_debt_id.clone(),
                wallet.wallet_id.clone(),
                wallet.user_id.clone(),
                requested,
                actual_deduct,
                debt_amount,
                req.reason.clone(),
                admin_id.clone(),
            );

            wallet.admin_debt += debt_amount;
            wallet.admin_debt_reason = Some(req.reason.clone());
            wallet.admin_debt_created_by = Some(admin_id.clone());
            wallet.admin_debt_created_at = Some(BsonDateTime::now());

            self.repo.create_admin_debt_transaction_with_session(debt_tx, &mut session).await?;

            let debt_tx_record = Transaction::new(
                Self::generate_id("TXN"),
                wallet.wallet_id.clone(),
                wallet.user_id.clone(),
                TransactionType::AdminDebit,
                Direction::Debit,
                0,
                wallet.available_trust,
                wallet.available_trust,
                BalanceType::Available,
                admin_id.clone(),
            );
            self.repo.create_transaction_with_session(debt_tx_record, &mut session).await?;
        }

        Self::validate_invariant(&wallet)?;
        self.repo.update_wallet_with_session(&wallet, &mut session).await?;

        let log = AdminOperationLog {
            id: None,
            log_id: Self::generate_id("ALOG"),
            admin_id: admin_id.clone(),
            admin_email: "admin@example.com".to_string(),
            admin_role: "ADMIN".to_string(),
            operation: AdminOperation::ManualDebit,
            target_type: TargetType::Wallet,
            target_id: wallet.wallet_id.clone(),
            before_state: serde_json::json!({
                "available_trust": balance_before,
                "admin_debt": wallet.admin_debt - debt_amount
            }),
            after_state: serde_json::json!({
                "available_trust": wallet.available_trust,
                "admin_debt": wallet.admin_debt
            }),
            amount: Some(requested),
            reason: req.reason,
            note: req.note,
            transaction_id: debt_id.clone(),
            ip_address: "0.0.0.0".to_string(),
            user_agent: "".to_string(),
            created_at: BsonDateTime::now(),
        };
        self.repo.create_admin_log_with_session(log, &mut session).await?;

        session.commit_transaction().await
            .map_err(|e| ServiceError::DatabaseError(format!("Failed to commit transaction: {}", e)))?;

        Ok(AdminDebitResponse {
            wallet_id: wallet.wallet_id,
            user_id: wallet.user_id,
            requested_amount: requested,
            actual_deducted: actual_deduct,
            debt_created: debt_amount,
            new_available: wallet.available_trust,
            new_admin_debt: wallet.admin_debt,
            debt_id,
        })
    }

    // ========================================================================
    // ADMIN: FREEZE/UNFREEZE WALLET
    // ========================================================================

    pub async fn freeze_wallet(
        &self,
        req: FreezeWalletRequest,
        admin_id: String,
    ) -> Result<SuccessResponse, ServiceError> {
        let mut wallet = self
            .repo
            .find_wallet_by_user_id(&req.user_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Wallet not found".to_string()))?;

        let lock_amount = req.amount.unwrap_or(wallet.available_trust);

        if lock_amount > wallet.available_trust {
            return Err(ServiceError::BadRequest(format!(
                "Cannot lock {} Trust. Available: {} Trust",
                lock_amount, wallet.available_trust
            )));
        }

        let mut session = self.repo.start_session().await?;
        session.start_transaction().await
            .map_err(|e| ServiceError::DatabaseError(format!("Failed to start transaction: {}", e)))?;

        let before_available = wallet.available_trust;
        let before_dispute_locked = wallet.dispute_locked;

        wallet.available_trust -= lock_amount;
        wallet.dispute_locked += lock_amount;

        let full_freeze = wallet.available_trust == 0;
        if full_freeze {
            wallet.status = WalletStatus::Suspended;
            wallet.freeze_reason = Some(req.reason.clone());
        }

        Self::validate_invariant(&wallet)?;

        let lock_id = Self::generate_id("LOCK");
        let dispute_lock = DisputeLock::new(
            lock_id.clone(),
            wallet.wallet_id.clone(),
            wallet.user_id.clone(),
            lock_amount,
            req.reason.clone(),
            req.case_reference.clone(),
            admin_id.clone(),
        );

        self.repo.create_dispute_lock_with_session(dispute_lock, &mut session).await?;
        self.repo.update_wallet_with_session(&wallet, &mut session).await?;

        let log = AdminOperationLog {
            id: None,
            log_id: Self::generate_id("ALOG"),
            admin_id: admin_id.clone(),
            admin_email: "admin@example.com".to_string(),
            admin_role: "ADMIN".to_string(),
            operation: AdminOperation::FreezeWallet,
            target_type: TargetType::Wallet,
            target_id: wallet.wallet_id.clone(),
            before_state: serde_json::json!({
                "available_trust": before_available,
                "dispute_locked": before_dispute_locked
            }),
            after_state: serde_json::json!({
                "available_trust": wallet.available_trust,
                "dispute_locked": wallet.dispute_locked
            }),
            amount: Some(lock_amount),
            reason: req.reason,
            note: Some(req.case_reference),
            transaction_id: Some(lock_id.clone()),
            ip_address: "0.0.0.0".to_string(),
            user_agent: "".to_string(),
            created_at: BsonDateTime::now(),
        };
        self.repo.create_admin_log_with_session(log, &mut session).await?;

        session.commit_transaction().await
            .map_err(|e| ServiceError::DatabaseError(format!("Failed to commit transaction: {}", e)))?;

        Ok(SuccessResponse::new(format!(
            "Locked {} Trust for wallet {}. Lock ID: {}",
            lock_amount, wallet.wallet_id, lock_id
        )))
    }

    pub async fn unfreeze_wallet(
        &self,
        req: UnfreezeWalletRequest,
        admin_id: String,
    ) -> Result<SuccessResponse, ServiceError> {
        let mut wallet = self
            .repo
            .find_wallet_by_user_id(&req.user_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Wallet not found".to_string()))?;

        let active_locks = self.repo.find_active_dispute_locks(&wallet.wallet_id).await?;

        if active_locks.is_empty() {
            return Err(ServiceError::BadRequest(
                "No active locks on this wallet".to_string(),
            ));
        }

        let mut session = self.repo.start_session().await?;
        session.start_transaction().await
            .map_err(|e| ServiceError::DatabaseError(format!("Failed to start transaction: {}", e)))?;

        let before_available = wallet.available_trust;
        let before_dispute_locked = wallet.dispute_locked;

        let mut total_released = 0i64;
        for mut lock in active_locks {
            lock.resolve(admin_id.clone(), req.resolution_note.clone());
            self.repo.update_dispute_lock_with_session(&lock, &mut session).await?;

            wallet.dispute_locked -= lock.amount;
            wallet.available_trust += lock.amount;
            total_released += lock.amount;
        }

        if wallet.dispute_locked == 0 && wallet.status == WalletStatus::Suspended {
            wallet.status = WalletStatus::Active;
            wallet.freeze_reason = None;
        }

        Self::validate_invariant(&wallet)?;
        self.repo.update_wallet_with_session(&wallet, &mut session).await?;

        let log = AdminOperationLog {
            id: None,
            log_id: Self::generate_id("ALOG"),
            admin_id: admin_id.clone(),
            admin_email: "admin@example.com".to_string(),
            admin_role: "ADMIN".to_string(),
            operation: AdminOperation::UnfreezeWallet,
            target_type: TargetType::Wallet,
            target_id: wallet.wallet_id.clone(),
            before_state: serde_json::json!({
                "available_trust": before_available,
                "dispute_locked": before_dispute_locked
            }),
            after_state: serde_json::json!({
                "available_trust": wallet.available_trust,
                "dispute_locked": wallet.dispute_locked
            }),
            amount: Some(total_released),
            reason: "Wallet unlocked".to_string(),
            note: Some(req.resolution_note),
            transaction_id: None,
            ip_address: "0.0.0.0".to_string(),
            user_agent: "".to_string(),
            created_at: BsonDateTime::now(),
        };
        self.repo.create_admin_log_with_session(log, &mut session).await?;

        session.commit_transaction().await
            .map_err(|e| ServiceError::DatabaseError(format!("Failed to commit transaction: {}", e)))?;

        Ok(SuccessResponse::new(format!(
            "Released {} Trust for wallet {}",
            total_released, wallet.wallet_id
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
            shop_id: req.shop_id.clone(),
            rate: req.commission_rate,
            effective_from: BsonDateTime::now(),
            effective_to: None,
            created_by: admin_id.clone(),
            reason: req.reason.clone(),
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

    /// Get transaction history for a wallet (admin version)
    pub async fn get_transaction_history_admin(
        &self,
        wallet_id: &str,
        limit: Option<i64>,
    ) -> Result<TransactionHistoryResponse, ServiceError> {
        let limit = limit.unwrap_or(50).min(100); // Max 100
        let transactions = self.repo.find_transactions_by_wallet(wallet_id, None, None, limit, 0).await?;

        let count = transactions.len() as i64;
        Ok(TransactionHistoryResponse {
            wallet_id: wallet_id.to_string(),
            transactions,
            count,
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
            self.repo.find_admin_logs_by_target(&tid, limit).await?
        } else {
            self.repo.get_recent_admin_logs(limit).await?
        };

        let count = logs.len() as i64;
        Ok(AdminLogResponse {
            logs,
            count,
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
        if req.status != WithdrawalStatus::Pending && req.status != WithdrawalStatus::Validating {
            return Err(ServiceError::BadRequest(
                "Can only approve pending or validating withdrawals".to_string(),
            ));
        }

        // Update status
        req.status = WithdrawalStatus::Approved;
        req.approved_by = Some(admin_id.clone());
        req.approved_at = Some(BsonDateTime::now());
        req.updated_at = BsonDateTime::now();

        self.repo.update_withdrawal_request(&req).await?;

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
        session.start_transaction().await
            .map_err(|e| ServiceError::DatabaseError(format!("Failed to start transaction: {}", e)))?;

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
        withdrawal_req.validation_errors.push(ValidationError {
            error_type: "REJECTED".to_string(),
            message: req.rejection_reason.clone(),
            severity: Severity::Error,
        });

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

        session.commit_transaction().await
            .map_err(|e| ServiceError::DatabaseError(format!("Failed to commit transaction: {}", e)))?;

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
        session.start_transaction().await
            .map_err(|e| ServiceError::DatabaseError(format!("Failed to start transaction: {}", e)))?;

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
        withdrawal_req.bank_transfer_ref = Some(req.bank_transfer_ref.clone());
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

        session.commit_transaction().await
            .map_err(|e| ServiceError::DatabaseError(format!("Failed to commit transaction: {}", e)))?;

        Ok(SuccessResponse::new(format!(
            "Bank transfer completed. {} VND sent to user. Commission deducted: {} Trust.",
            withdrawal_req.vnd_amount, withdrawal_req.commission_deduct
        )))
    }
}
