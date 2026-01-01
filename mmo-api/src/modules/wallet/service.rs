//! Wallet V3 Service Layer
//!
//! Business logic for all wallet operations - PART 1: Core & Deposit/Withdrawal
//! See service_escrow.rs for escrow operations
//! See service_admin.rs for admin operations

use bson::DateTime as BsonDateTime;
use std::sync::Arc;
use ulid::Ulid;

use crate::core::error::ServiceError;
use super::{repository::WalletRepository, domain::*};
use super::dto::{self, *};

const VND_TO_TRUST_RATE: i64 = 1000;
const DEFAULT_COMMISSION_RATE: f64 = 0.05; // 5%
const ESCROW_HOLD_HOURS: i64 = 72; // 3 days

/// Wallet service
#[derive(Clone)]
pub struct WalletService {
    pub(super) repo: Arc<WalletRepository>,
}

impl WalletService {
    pub fn new(repo: Arc<WalletRepository>) -> Self {
        Self { repo }
    }

    // ========================================================================
    // WALLET MANAGEMENT
    // ========================================================================

    /// Get wallet balance
    pub async fn get_wallet_balance(
        &self,
        wallet_id: &str,
    ) -> Result<WalletBalanceResponse, ServiceError> {
        let wallet = self
            .repo
            .find_wallet_by_id(wallet_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Wallet not found".to_string()))?;

        let commission_debt = if wallet.wallet_type == WalletType::Seller {
            Some(wallet.commission_debt)
        } else {
            None
        };

        Ok(WalletBalanceResponse {
            wallet_id: wallet.wallet_id,
            wallet_type: wallet.wallet_type,
            available_trust: wallet.available_trust,
            withdrawal_locked: wallet.withdrawal_locked,
            dispute_locked: wallet.dispute_locked,
            total_trust: wallet.total_trust,
            available_vnd: wallet.available_trust * VND_TO_TRUST_RATE,
            total_vnd: wallet.total_trust * VND_TO_TRUST_RATE,
            commission_debt,
            commission_rate: wallet.commission_rate,
            status: wallet.status,
        })
    }

    /// Create wallet for user
    pub async fn create_wallet(
        &self,
        user_id: String,
        wallet_type: WalletType,
    ) -> Result<Wallet, ServiceError> {
        // Check if wallet already exists
        if let Some(_) = self.repo.find_wallet_by_user_id(&user_id).await? {
            return Err(ServiceError::BadRequest(
                "Wallet already exists for this user".to_string(),
            ));
        }

        let wallet_id = Self::generate_id("WLT");
        let wallet = match wallet_type {
            WalletType::User => Wallet::new_user(user_id, wallet_id),
            WalletType::Seller => Wallet::new_seller(user_id, wallet_id, None),
            WalletType::Platform => Wallet::new_platform(wallet_id),
        };

        let created = self.repo.create_wallet(wallet).await?;
        Ok(created)
    }

    // ========================================================================
    // DEPOSIT FLOW
    // ========================================================================

    /// Create auto deposit request
    pub async fn create_auto_deposit(
        &self,
        wallet_id: &str,
        req: AutoDepositRequest,
    ) -> Result<DepositResponse, ServiceError> {
        // Get wallet
        let wallet = self
            .repo
            .find_wallet_by_id(wallet_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Wallet not found".to_string()))?;

        // Check wallet status
        if !wallet.is_active() {
            return Err(ServiceError::BadRequest(
                "Wallet is not active".to_string(),
            ));
        }

        // Calculate Trust amount
        let trust_amount = req.vnd_amount / VND_TO_TRUST_RATE;

        // Create deposit request
        let deposit_id = Self::generate_id("DEP");
        let now = BsonDateTime::now();
        let expires_at = BsonDateTime::from_millis(now.timestamp_millis() + (15 * 60 * 1000)); // 15 minutes

        let deposit_req = DepositRequest {
            id: None,
            deposit_id: deposit_id.clone(),
            wallet_id: wallet.wallet_id.clone(),
            user_id: wallet.user_id.clone(),
            vnd_amount: req.vnd_amount,
            trust_amount,
            payment_gateway: req.payment_gateway.clone(),
            payment_url: None, // Will be set after calling payment gateway API
            payment_gateway_ref: None,
            status: DepositStatus::Pending,
            created_at: now,
            updated_at: now,
            completed_at: None,
            expires_at,
        };

        let created = self.repo.create_deposit_request(deposit_req).await?;

        // TODO: Call payment gateway API to get payment_url
        // For now, return mock response
        Ok(DepositResponse {
            deposit_id: created.deposit_id,
            wallet_id: created.wallet_id,
            vnd_amount: created.vnd_amount,
            trust_amount: created.trust_amount,
            payment_url: Some("https://payment-gateway.example.com/pay/123".to_string()),
            payment_gateway: Some(created.payment_gateway),
            status: "PENDING".to_string(),
            expires_at: Some(created.expires_at.to_string()),
            created_at: created.created_at.to_string(),
        })
    }

    /// Manual deposit (admin only)
    pub async fn manual_deposit(
        &self,
        req: ManualDepositRequest,
        admin_id: String,
    ) -> Result<SuccessResponse, ServiceError> {
        // Get wallet
        let mut wallet = self
            .repo
            .find_wallet_by_user_id(&req.user_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Wallet not found".to_string()))?;

        // Create transaction
        let tx_id = Self::generate_id("TXN");
        let balance_before = wallet.available_trust;
        let balance_after = balance_before + req.trust_amount;

        let tx = Transaction::new(
            tx_id,
            wallet.wallet_id.clone(),
            wallet.user_id.clone(),
            TransactionType::DepositManual,
            Direction::Credit,
            req.trust_amount,
            balance_before,
            balance_after,
            BalanceType::Available,
            admin_id.clone(),
        );

        // Update wallet
        wallet.available_trust += req.trust_amount;
        wallet.total_trust += req.trust_amount;
        wallet.lifetime_deposited += req.trust_amount;

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
            admin_email: "admin@example.com".to_string(), // TODO: Get from context
            admin_role: "ADMIN".to_string(),
            operation: AdminOperation::ManualDeposit,
            target_type: TargetType::Wallet,
            target_id: wallet.wallet_id.clone(),
            before_state: serde_json::json!({"available_trust": balance_before}),
            after_state: serde_json::json!({"available_trust": balance_after}),
            amount: Some(req.trust_amount),
            reason: req.reason,
            note: req.note,
            transaction_id: None,
            ip_address: "0.0.0.0".to_string(), // TODO: Get from request
            user_agent: "".to_string(),
            created_at: BsonDateTime::now(),
        };
        self.repo.create_admin_log(log).await?;

        Ok(SuccessResponse::new(format!(
            "Successfully deposited {} Trust to wallet",
            req.trust_amount
        )))
    }

    // ========================================================================
    // WITHDRAWAL FLOW
    // ========================================================================

    /// Create withdrawal request
    pub async fn create_withdrawal(
        &self,
        wallet_id: &str,
        req: dto::WithdrawalRequest,
    ) -> Result<WithdrawalResponse, ServiceError> {
        // Get wallet
        let mut wallet = self
            .repo
            .find_wallet_by_id(wallet_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Wallet not found".to_string()))?;

        // Check if can withdraw
        if !wallet.can_withdraw(req.trust_amount) {
            return Err(ServiceError::BadRequest(
                "Insufficient balance or wallet not active".to_string(),
            ));
        }

        // Calculate commission to deduct
        let commission_deduct = if wallet.wallet_type == WalletType::Seller {
            let max_commission = (req.trust_amount as f64 * DEFAULT_COMMISSION_RATE) as i64;
            max_commission.min(wallet.commission_debt)
        } else {
            0
        };

        let net_trust = req.trust_amount - commission_deduct;
        let vnd_amount = net_trust * VND_TO_TRUST_RATE;

        // Create withdrawal request
        let request_id = Self::generate_id("WD");
        let now = BsonDateTime::now();
        let expires_at = BsonDateTime::from_millis(now.timestamp_millis() + (24 * 60 * 60 * 1000)); // 24 hours

        let withdrawal_req = super::domain::WithdrawalRequest {
            id: None,
            request_id: request_id.clone(),
            wallet_id: wallet.wallet_id.clone(),
            user_id: wallet.user_id.clone(),
            trust_amount: req.trust_amount,
            commission_deduct,
            net_trust,
            vnd_amount,
            bank_code: req.bank_code.clone(),
            bank_name: req.bank_name.clone(),
            account_number: req.account_number.clone(),
            account_name: req.account_name.clone(),
            status: WithdrawalStatus::Pending,
            status_history: vec![],
            validation_result: None,
            validation_errors: vec![],
            approved_by: None,
            approved_at: None,
            bank_transfer_ref: None,
            bank_transfer_at: None,
            created_at: now,
            updated_at: now,
            completed_at: None,
            expires_at,
        };

        let created_req = self.repo.create_withdrawal_request(withdrawal_req).await?;

        // Lock funds
        let tx_id = Self::generate_id("TXN");
        let balance_before = wallet.available_trust;
        let balance_after = balance_before - req.trust_amount;

        let lock_tx = Transaction::new(
            tx_id,
            wallet.wallet_id.clone(),
            wallet.user_id.clone(),
            TransactionType::WithdrawalRequest,
            Direction::Debit,
            req.trust_amount,
            balance_before,
            balance_after,
            BalanceType::Available,
            wallet.user_id.clone(),
        );

        // Update wallet
        wallet.available_trust -= req.trust_amount;
        wallet.withdrawal_locked += req.trust_amount;
        Self::validate_invariant(&wallet)?;

        // Save
        self.repo.create_transaction(lock_tx).await?;
        self.repo.update_wallet(&wallet).await?;

        // TODO: Enqueue background validation job

        Ok(WithdrawalResponse {
            request_id: created_req.request_id,
            wallet_id: created_req.wallet_id,
            trust_amount: created_req.trust_amount,
            commission_deduct: created_req.commission_deduct,
            net_trust: created_req.net_trust,
            vnd_amount: created_req.vnd_amount,
            bank_info: BankInfo {
                bank_code: created_req.bank_code,
                bank_name: created_req.bank_name,
                account_number: created_req.account_number,
                account_name: created_req.account_name,
            },
            status: created_req.status,
            validation_result: None,
            bank_transfer_ref: None,
            created_at: created_req.created_at.to_string(),
            expires_at: created_req.expires_at.to_string(),
        })
    }

    /// Validate withdrawal (simplified version - full implementation in validation module)
    pub async fn validate_withdrawal(
        &self,
        request_id: &str,
    ) -> Result<ValidationResult, ServiceError> {
        let req = self
            .repo
            .find_withdrawal_by_id(request_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Withdrawal request not found".to_string()))?;

        let wallet = self
            .repo
            .find_wallet_by_id(&req.wallet_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Wallet not found".to_string()))?;

        // Run validation checks
        let balance_check = self.validate_balance_integrity(&wallet).await?;
        let flow_check = self.validate_flow(&wallet, req.trust_amount).await?;
        let fraud_check = self.validate_fraud_patterns(&wallet, req.trust_amount).await?;
        let limit_check = self.validate_limits(&wallet, req.trust_amount).await?;

        let overall_passed = balance_check.passed
            && flow_check.passed
            && fraud_check.passed
            && limit_check.passed;

        // Calculate risk score (simplified)
        let mut risk_score = 0.0;
        if !fraud_check.passed {
            risk_score += 0.5;
        }
        if fraud_check.severity == Severity::Warning {
            risk_score += 0.3;
        }

        Ok(ValidationResult {
            balance_check,
            flow_check,
            fraud_check,
            limit_check,
            overall_passed,
            risk_score,
        })
    }

    // ========================================================================
    // VALIDATION CHECKS (Simplified)
    // ========================================================================

    async fn validate_balance_integrity(&self, wallet: &Wallet) -> Result<CheckResult, ServiceError> {
        // Check balance invariant
        let is_valid = wallet.validate_balance_invariant();

        Ok(CheckResult {
            passed: is_valid,
            details: if is_valid {
                "Balance integrity check passed".to_string()
            } else {
                format!(
                    "Balance mismatch: total={} but sum={}",
                    wallet.total_trust,
                    wallet.available_trust + wallet.withdrawal_locked + wallet.dispute_locked
                )
            },
            severity: if is_valid {
                Severity::Info
            } else {
                Severity::Critical
            },
        })
    }

    async fn validate_flow(&self, wallet: &Wallet, withdrawal_amount: i64) -> Result<CheckResult, ServiceError> {
        // Simplified flow validation
        let expected_balance =
            wallet.lifetime_deposited - wallet.lifetime_withdrawn + wallet.lifetime_received - wallet.lifetime_spent;

        let is_valid = expected_balance >= wallet.total_trust;

        Ok(CheckResult {
            passed: is_valid,
            details: if is_valid {
                "Flow validation passed".to_string()
            } else {
                "Flow validation failed: money flow doesn't match".to_string()
            },
            severity: if is_valid {
                Severity::Info
            } else {
                Severity::Error
            },
        })
    }

    async fn validate_fraud_patterns(
        &self,
        wallet: &Wallet,
        withdrawal_amount: i64,
    ) -> Result<CheckResult, ServiceError> {
        // Count today's withdrawals
        let today_count = self.repo.count_today_withdrawals(&wallet.wallet_id).await?;

        let mut risk_score = 0.0;
        let mut details = vec![];

        // Pattern 1: Too many withdrawals today
        if today_count >= 5 {
            risk_score += 0.3;
            details.push(format!("High frequency: {} withdrawals today", today_count));
        }

        // Pattern 2: Large amount (simplified)
        if withdrawal_amount > 100000 {
            risk_score += 0.2;
            details.push("Large withdrawal amount".to_string());
        }

        let severity = if risk_score >= 0.7 {
            Severity::Critical
        } else if risk_score >= 0.3 {
            Severity::Warning
        } else {
            Severity::Info
        };

        Ok(CheckResult {
            passed: risk_score < 0.7,
            details: if details.is_empty() {
                "No fraud patterns detected".to_string()
            } else {
                details.join(", ")
            },
            severity,
        })
    }

    async fn validate_limits(&self, wallet: &Wallet, withdrawal_amount: i64) -> Result<CheckResult, ServiceError> {
        // Check per-transaction limits
        if withdrawal_amount < 10 {
            return Ok(CheckResult {
                passed: false,
                details: "Below minimum withdrawal amount (10 Trust)".to_string(),
                severity: Severity::Error,
            });
        }

        if withdrawal_amount > 100000 {
            return Ok(CheckResult {
                passed: false,
                details: "Exceeds maximum withdrawal amount (100,000 Trust)".to_string(),
                severity: Severity::Error,
            });
        }

        Ok(CheckResult {
            passed: true,
            details: "Withdrawal limits check passed".to_string(),
            severity: Severity::Info,
        })
    }

    // ========================================================================
    // HELPER METHODS
    // ========================================================================

    pub(super) fn generate_id(prefix: &str) -> String {
        format!("{}-{}", prefix, Ulid::new())
    }

    pub(super) fn validate_invariant(wallet: &Wallet) -> Result<(), ServiceError> {
        if !wallet.validate_balance_invariant() {
            return Err(ServiceError::InternalError(format!(
                "Balance invariant violated for wallet {}",
                wallet.wallet_id
            )));
        }
        Ok(())
    }

    pub fn get_repository(&self) -> Arc<WalletRepository> {
        Arc::clone(&self.repo)
    }
}
