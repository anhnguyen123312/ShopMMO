//! Wallet V3 Service Layer - PART 2: Escrow Operations
//!
//! Purchase flow, escrow release, and dispute handling

use bson::DateTime as BsonDateTime;
use chrono::{Duration, Utc};
use std::sync::Arc;

use crate::core::error::ServiceError;
use super::{dto::*, repository::WalletRepository, domain::*, service::WalletService};

const VND_TO_TRUST_RATE: i64 = 1000;
const ESCROW_HOLD_HOURS: i64 = 72; // 3 days
const DEFAULT_COMMISSION_RATE: f64 = 0.05; // 5%

impl WalletService {
    // ========================================================================
    // PURCHASE FLOW (Buyer → Platform Escrow)
    // ========================================================================

    /// Create purchase (buyer pays, money locked in platform wallet)
    pub async fn create_purchase(
        &self,
        buyer_wallet_id: &str,
        req: PurchaseRequest,
    ) -> Result<PurchaseResponse, ServiceError> {
        // Get buyer wallet
        let mut buyer_wallet = self
            .repo
            .find_wallet_by_id(buyer_wallet_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Buyer wallet not found".to_string()))?;

        // Verify buyer has enough balance
        if !buyer_wallet.can_withdraw(req.trust_amount) {
            return Err(ServiceError::BadRequest(
                "Insufficient balance".to_string(),
            ));
        }

        // Get or create platform wallet
        let mut platform_wallet = self
            .repo
            .find_platform_wallet()
            .await?
            .ok_or_else(|| ServiceError::NotFound("Platform wallet not found".to_string()))?;

        // Get seller wallet to determine commission rate
        let seller_wallet = self
            .repo
            .find_wallet_by_user_id(&req.seller_user_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Seller wallet not found".to_string()))?;

        // Calculate commission
        let commission_rate = seller_wallet.commission_rate.unwrap_or(DEFAULT_COMMISSION_RATE);
        let commission_amount = (req.trust_amount as f64 * commission_rate) as i64;

        // Create escrow hold
        let escrow_id = Self::generate_id("ESC");
        let now = BsonDateTime::now();
        let release_at = BsonDateTime::from_millis(
            now.timestamp_millis() + (ESCROW_HOLD_HOURS * 60 * 60 * 1000),
        );

        let escrow = EscrowHold {
            id: None,
            escrow_id: escrow_id.clone(),
            order_id: req.order_id.clone(),
            buyer_wallet_id: buyer_wallet.wallet_id.clone(),
            buyer_user_id: buyer_wallet.user_id.clone(),
            seller_wallet_id: seller_wallet.wallet_id.clone(),
            seller_user_id: seller_wallet.user_id.clone(),
            product_id: req.product_id.clone(),
            product_name: req.product_name.clone(),
            trust_amount: req.trust_amount,
            commission_rate,
            commission_amount,
            status: EscrowStatus::Holding,
            auto_release_at: release_at,
            released_at: None,
            release_type: None,
            dispute_reason: None,
            created_at: now,
            updated_at: now,
        };

        // Use MongoDB transaction
        let mut session = self.repo.start_session().await?;
        session.start_transaction(None).await?;

        // 1. Deduct from buyer: available -> spent
        let buyer_balance_before = buyer_wallet.available_trust;
        buyer_wallet.available_trust -= req.trust_amount;
        buyer_wallet.lifetime_spent += req.trust_amount;
        buyer_wallet.total_trust -= req.trust_amount;
        Self::validate_invariant(&buyer_wallet)?;

        let buyer_tx_id = Self::generate_id("TXN");
        let buyer_tx = Transaction::new(
            buyer_tx_id.clone(),
            buyer_wallet.wallet_id.clone(),
            buyer_wallet.user_id.clone(),
            TransactionType::Purchase,
            Direction::Debit,
            req.trust_amount,
            buyer_balance_before,
            buyer_wallet.available_trust,
            BalanceType::Available,
            buyer_wallet.user_id.clone(),
        );

        // 2. Add to platform wallet: holding for escrow
        let platform_balance_before = platform_wallet.available_trust;
        platform_wallet.available_trust += req.trust_amount;
        platform_wallet.total_trust += req.trust_amount;
        Self::validate_invariant(&platform_wallet)?;

        let platform_tx_id = Self::generate_id("TXN");
        let platform_tx = Transaction::new(
            platform_tx_id.clone(),
            platform_wallet.wallet_id.clone(),
            platform_wallet.user_id.clone(),
            TransactionType::PurchaseEscrow,
            Direction::Credit,
            req.trust_amount,
            platform_balance_before,
            platform_wallet.available_trust,
            BalanceType::Available,
            buyer_wallet.user_id.clone(),
        );

        // Save to database
        self.repo
            .create_transaction_with_session(buyer_tx, &mut session)
            .await?;
        self.repo
            .create_transaction_with_session(platform_tx, &mut session)
            .await?;
        self.repo
            .update_wallet_with_session(&buyer_wallet, &mut session)
            .await?;
        self.repo
            .update_wallet_with_session(&platform_wallet, &mut session)
            .await?;
        self.repo
            .create_escrow_with_session(escrow, &mut session)
            .await?;

        session.commit_transaction().await?;

        Ok(PurchaseResponse {
            escrow_id,
            order_id: req.order_id,
            buyer_wallet_id: buyer_wallet.wallet_id,
            seller_wallet_id: seller_wallet.wallet_id,
            trust_amount: req.trust_amount,
            commission_amount,
            status: EscrowStatus::Holding,
            auto_release_at: release_at.to_string(),
            created_at: now.to_string(),
        })
    }

    // ========================================================================
    // ESCROW RELEASE FLOW (Platform → Seller)
    // ========================================================================

    /// Auto release escrow after 3 days
    pub async fn auto_release_escrow(
        &self,
        escrow_id: &str,
    ) -> Result<SuccessResponse, ServiceError> {
        self.release_escrow_internal(escrow_id, ReleaseType::Auto, None)
            .await
    }

    /// Early release by buyer
    pub async fn early_release_escrow(
        &self,
        escrow_id: &str,
        buyer_user_id: String,
    ) -> Result<SuccessResponse, ServiceError> {
        // Verify escrow belongs to buyer
        let escrow = self
            .repo
            .find_escrow_by_id(escrow_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Escrow not found".to_string()))?;

        if escrow.buyer_user_id != buyer_user_id {
            return Err(ServiceError::Unauthorized(
                "Not authorized to release this escrow".to_string(),
            ));
        }

        self.release_escrow_internal(escrow_id, ReleaseType::EarlyRelease, None)
            .await
    }

    /// Internal release logic
    async fn release_escrow_internal(
        &self,
        escrow_id: &str,
        release_type: ReleaseType,
        admin_id: Option<String>,
    ) -> Result<SuccessResponse, ServiceError> {
        // Get escrow
        let mut escrow = self
            .repo
            .find_escrow_by_id(escrow_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Escrow not found".to_string()))?;

        // Check if already released
        if escrow.status != EscrowStatus::Holding {
            return Err(ServiceError::BadRequest(
                "Escrow is not in holding status".to_string(),
            ));
        }

        // Get wallets
        let mut platform_wallet = self
            .repo
            .find_platform_wallet()
            .await?
            .ok_or_else(|| ServiceError::NotFound("Platform wallet not found".to_string()))?;

        let mut seller_wallet = self
            .repo
            .find_wallet_by_id(&escrow.seller_wallet_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Seller wallet not found".to_string()))?;

        // Calculate amounts
        let net_to_seller = escrow.trust_amount - escrow.commission_amount;

        // Use MongoDB transaction
        let mut session = self.repo.start_session().await?;
        session.start_transaction(None).await?;

        let now = BsonDateTime::now();

        // 1. Deduct from platform wallet
        let platform_balance_before = platform_wallet.available_trust;
        platform_wallet.available_trust -= escrow.trust_amount;
        platform_wallet.total_trust -= escrow.trust_amount;
        Self::validate_invariant(&platform_wallet)?;

        let platform_tx_id = Self::generate_id("TXN");
        let platform_tx = Transaction::new(
            platform_tx_id,
            platform_wallet.wallet_id.clone(),
            platform_wallet.user_id.clone(),
            TransactionType::EscrowRelease,
            Direction::Debit,
            escrow.trust_amount,
            platform_balance_before,
            platform_wallet.available_trust,
            BalanceType::Available,
            admin_id.clone().unwrap_or_else(|| "SYSTEM".to_string()),
        );

        // 2. Add net amount to seller
        let seller_balance_before = seller_wallet.available_trust;
        seller_wallet.available_trust += net_to_seller;
        seller_wallet.total_trust += net_to_seller;
        seller_wallet.lifetime_received += net_to_seller;

        // 3. Add commission to seller's debt
        seller_wallet.commission_debt += escrow.commission_amount;

        Self::validate_invariant(&seller_wallet)?;

        let seller_tx_id = Self::generate_id("TXN");
        let seller_tx = Transaction::new(
            seller_tx_id,
            seller_wallet.wallet_id.clone(),
            seller_wallet.user_id.clone(),
            TransactionType::EscrowReceive,
            Direction::Credit,
            net_to_seller,
            seller_balance_before,
            seller_wallet.available_trust,
            BalanceType::Available,
            admin_id.clone().unwrap_or_else(|| "SYSTEM".to_string()),
        );

        // 4. Update escrow status
        escrow.status = EscrowStatus::Released;
        escrow.released_at = Some(now);
        escrow.release_type = Some(release_type);
        escrow.updated_at = now;

        // Save to database
        self.repo
            .create_transaction_with_session(platform_tx, &mut session)
            .await?;
        self.repo
            .create_transaction_with_session(seller_tx, &mut session)
            .await?;
        self.repo
            .update_wallet_with_session(&platform_wallet, &mut session)
            .await?;
        self.repo
            .update_wallet_with_session(&seller_wallet, &mut session)
            .await?;
        self.repo
            .update_escrow_with_session(&escrow, &mut session)
            .await?;

        session.commit_transaction().await?;

        Ok(SuccessResponse::new(format!(
            "Escrow {} released to seller. Net: {} Trust, Commission: {} Trust",
            escrow_id, net_to_seller, escrow.commission_amount
        )))
    }

    // ========================================================================
    // DISPUTE FLOW
    // ========================================================================

    /// Create dispute (freeze escrow)
    pub async fn create_dispute(
        &self,
        escrow_id: &str,
        req: DisputeRequest,
        user_id: String,
    ) -> Result<SuccessResponse, ServiceError> {
        // Get escrow
        let mut escrow = self
            .repo
            .find_escrow_by_id(escrow_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Escrow not found".to_string()))?;

        // Verify user is buyer or seller
        if escrow.buyer_user_id != user_id && escrow.seller_user_id != user_id {
            return Err(ServiceError::Unauthorized(
                "Not authorized to dispute this escrow".to_string(),
            ));
        }

        // Check if in holding status
        if escrow.status != EscrowStatus::Holding {
            return Err(ServiceError::BadRequest(
                "Can only dispute escrow in holding status".to_string(),
            ));
        }

        // Update escrow to disputed
        escrow.status = EscrowStatus::Disputed;
        escrow.dispute_reason = Some(req.reason);
        escrow.updated_at = BsonDateTime::now();

        self.repo.update_escrow(&escrow).await?;

        Ok(SuccessResponse::new(format!(
            "Dispute created for escrow {}. Awaiting admin resolution.",
            escrow_id
        )))
    }

    /// Admin resolve dispute - refund to buyer
    pub async fn resolve_dispute_refund(
        &self,
        escrow_id: &str,
        admin_id: String,
        reason: String,
    ) -> Result<SuccessResponse, ServiceError> {
        // Get escrow
        let mut escrow = self
            .repo
            .find_escrow_by_id(escrow_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Escrow not found".to_string()))?;

        if escrow.status != EscrowStatus::Disputed {
            return Err(ServiceError::BadRequest(
                "Escrow is not in disputed status".to_string(),
            ));
        }

        // Get wallets
        let mut platform_wallet = self
            .repo
            .find_platform_wallet()
            .await?
            .ok_or_else(|| ServiceError::NotFound("Platform wallet not found".to_string()))?;

        let mut buyer_wallet = self
            .repo
            .find_wallet_by_id(&escrow.buyer_wallet_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Buyer wallet not found".to_string()))?;

        // Use MongoDB transaction
        let mut session = self.repo.start_session().await?;
        session.start_transaction(None).await?;

        let now = BsonDateTime::now();

        // 1. Deduct from platform
        let platform_balance_before = platform_wallet.available_trust;
        platform_wallet.available_trust -= escrow.trust_amount;
        platform_wallet.total_trust -= escrow.trust_amount;
        Self::validate_invariant(&platform_wallet)?;

        let platform_tx_id = Self::generate_id("TXN");
        let platform_tx = Transaction::new(
            platform_tx_id,
            platform_wallet.wallet_id.clone(),
            platform_wallet.user_id.clone(),
            TransactionType::DisputeRefund,
            Direction::Debit,
            escrow.trust_amount,
            platform_balance_before,
            platform_wallet.available_trust,
            BalanceType::Available,
            admin_id.clone(),
        );

        // 2. Refund to buyer (reverse the spent)
        let buyer_balance_before = buyer_wallet.available_trust;
        buyer_wallet.available_trust += escrow.trust_amount;
        buyer_wallet.total_trust += escrow.trust_amount;
        buyer_wallet.lifetime_spent -= escrow.trust_amount; // Reverse
        Self::validate_invariant(&buyer_wallet)?;

        let buyer_tx_id = Self::generate_id("TXN");
        let buyer_tx = Transaction::new(
            buyer_tx_id,
            buyer_wallet.wallet_id.clone(),
            buyer_wallet.user_id.clone(),
            TransactionType::DisputeRefund,
            Direction::Credit,
            escrow.trust_amount,
            buyer_balance_before,
            buyer_wallet.available_trust,
            BalanceType::Available,
            admin_id.clone(),
        );

        // 3. Update escrow
        escrow.status = EscrowStatus::Refunded;
        escrow.released_at = Some(now);
        escrow.release_type = Some(ReleaseType::DisputeRefund);
        escrow.updated_at = now;

        // Save to database
        self.repo
            .create_transaction_with_session(platform_tx, &mut session)
            .await?;
        self.repo
            .create_transaction_with_session(buyer_tx, &mut session)
            .await?;
        self.repo
            .update_wallet_with_session(&platform_wallet, &mut session)
            .await?;
        self.repo
            .update_wallet_with_session(&buyer_wallet, &mut session)
            .await?;
        self.repo
            .update_escrow_with_session(&escrow, &mut session)
            .await?;

        // Create admin log
        let log = AdminOperationLog {
            id: None,
            log_id: Self::generate_id("ALOG"),
            admin_id: admin_id.clone(),
            admin_email: "admin@example.com".to_string(),
            admin_role: "ADMIN".to_string(),
            operation: AdminOperation::DisputeRefund,
            target_type: TargetType::Escrow,
            target_id: escrow.escrow_id.clone(),
            before_state: serde_json::json!({"status": "DISPUTED"}),
            after_state: serde_json::json!({"status": "REFUNDED"}),
            amount: Some(escrow.trust_amount),
            reason,
            note: None,
            transaction_id: Some(buyer_tx_id),
            ip_address: "0.0.0.0".to_string(),
            user_agent: "".to_string(),
            created_at: now,
        };
        self.repo
            .create_admin_log_with_session(log, &mut session)
            .await?;

        session.commit_transaction().await?;

        Ok(SuccessResponse::new(format!(
            "Dispute resolved with refund. {} Trust returned to buyer.",
            escrow.trust_amount
        )))
    }

    /// Admin resolve dispute - release to seller
    pub async fn resolve_dispute_release(
        &self,
        escrow_id: &str,
        admin_id: String,
        reason: String,
    ) -> Result<SuccessResponse, ServiceError> {
        // Get escrow
        let mut escrow = self
            .repo
            .find_escrow_by_id(escrow_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Escrow not found".to_string()))?;

        if escrow.status != EscrowStatus::Disputed {
            return Err(ServiceError::BadRequest(
                "Escrow is not in disputed status".to_string(),
            ));
        }

        // Update to holding so we can release
        escrow.status = EscrowStatus::Holding;
        self.repo.update_escrow(&escrow).await?;

        // Use internal release logic
        self.release_escrow_internal(escrow_id, ReleaseType::DisputeRelease, Some(admin_id.clone()))
            .await?;

        // Create admin log
        let log = AdminOperationLog {
            id: None,
            log_id: Self::generate_id("ALOG"),
            admin_id: admin_id.clone(),
            admin_email: "admin@example.com".to_string(),
            admin_role: "ADMIN".to_string(),
            operation: AdminOperation::DisputeRelease,
            target_type: TargetType::Escrow,
            target_id: escrow.escrow_id.clone(),
            before_state: serde_json::json!({"status": "DISPUTED"}),
            after_state: serde_json::json!({"status": "RELEASED"}),
            amount: Some(escrow.trust_amount),
            reason,
            note: None,
            transaction_id: None,
            ip_address: "0.0.0.0".to_string(),
            user_agent: "".to_string(),
            created_at: BsonDateTime::now(),
        };
        self.repo.create_admin_log(log).await?;

        Ok(SuccessResponse::new(format!(
            "Dispute resolved with release. Money released to seller.",
        )))
    }

    // ========================================================================
    // BACKGROUND JOB: Auto-release escrows
    // ========================================================================

    /// Process auto-release for all escrows past their hold period
    pub async fn process_auto_releases(&self) -> Result<ProcessAutoReleaseResponse, ServiceError> {
        let now = BsonDateTime::now();
        let escrows = self.repo.find_escrows_ready_for_release(now).await?;

        let mut released_count = 0;
        let mut failed_count = 0;
        let mut released_ids = vec![];
        let mut errors = vec![];

        for escrow in escrows {
            match self.auto_release_escrow(&escrow.escrow_id).await {
                Ok(_) => {
                    released_count += 1;
                    released_ids.push(escrow.escrow_id);
                }
                Err(e) => {
                    failed_count += 1;
                    errors.push(format!("{}: {}", escrow.escrow_id, e));
                }
            }
        }

        Ok(ProcessAutoReleaseResponse {
            total_processed: released_count + failed_count,
            released_count,
            failed_count,
            released_ids,
            errors,
        })
    }
}
