//! Wallet V3 Service Layer - PART 2: Escrow Operations
//!
//! Purchase flow, escrow release, and dispute handling

use bson::{doc, DateTime as BsonDateTime};

use crate::core::error::ServiceError;
use super::{dto::*, domain::*, service::WalletService};

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
            .await?;

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
            buyer_id: buyer_wallet.user_id.clone(),
            seller_id: seller_wallet.user_id.clone(),
            amount: req.trust_amount,
            status: EscrowStatus::Holding,
            release_at,
            released_at: None,
            early_release: false,
            early_release_by: None,
            commission_amount: Some(commission_amount),
            created_at: now,
            updated_at: now,
        };

        // Use MongoDB transaction
        let mut session = self.repo.start_session().await?;
        session.start_transaction().await
            .map_err(|e| ServiceError::DatabaseError(format!("Failed to start transaction: {}", e)))?;

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
            .create_escrow_hold_with_session(escrow, &mut session)
            .await?;

        session.commit_transaction().await
            .map_err(|e| ServiceError::DatabaseError(format!("Failed to commit transaction: {}", e)))?;

        Ok(PurchaseResponse {
            escrow_id,
            order_id: req.order_id,
            buyer_wallet_id: buyer_wallet.wallet_id,
            seller_id: seller_wallet.user_id,
            amount: req.trust_amount,
            release_at: release_at.to_string(),
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

        if escrow.buyer_id != buyer_user_id {
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
        _release_type: ReleaseType,
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
            .await?;

        let mut seller_wallet = self
            .repo
            .find_wallet_by_user_id(&escrow.seller_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Seller wallet not found".to_string()))?;

        let commission_amount = escrow.commission_amount.unwrap_or(0);
        let net_to_seller = escrow.amount - commission_amount;

        // Use MongoDB transaction
        let mut session = self.repo.start_session().await?;
        session.start_transaction().await
            .map_err(|e| ServiceError::DatabaseError(format!("Failed to start transaction: {}", e)))?;

        let now = BsonDateTime::now();

        // 1. Deduct from platform wallet
        let platform_balance_before = platform_wallet.available_trust;
        platform_wallet.available_trust -= escrow.amount;
        platform_wallet.total_trust -= escrow.amount;
        Self::validate_invariant(&platform_wallet)?;

        let platform_tx_id = Self::generate_id("TXN");
        let platform_tx = Transaction::new(
            platform_tx_id,
            platform_wallet.wallet_id.clone(),
            platform_wallet.user_id.clone(),
            TransactionType::EscrowRelease,
            Direction::Debit,
            escrow.amount,
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
        seller_wallet.commission_debt += commission_amount;

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
            .update_escrow_hold_with_session(&escrow, &mut session)
            .await?;

        session.commit_transaction().await
            .map_err(|e| ServiceError::DatabaseError(format!("Failed to commit transaction: {}", e)))?;

        Ok(SuccessResponse::new(format!(
            "Escrow {} released to seller. Net: {} Trust, Commission: {} Trust",
            escrow_id, net_to_seller, commission_amount
        )))
    }

    // ========================================================================
    // DISPUTE FLOW (Legacy - kept for backward compatibility)
    // ========================================================================

    /// Create dispute (freeze escrow) - Legacy method
    pub async fn create_dispute(
        &self,
        escrow_id: &str,
        _req: CreateDisputeRequest,
        user_id: String,
    ) -> Result<SuccessResponse, ServiceError> {
        // Get escrow
        let mut escrow = self
            .repo
            .find_escrow_by_id(escrow_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Escrow not found".to_string()))?;

        // Verify user is buyer or seller
        if escrow.buyer_id != user_id && escrow.seller_id != user_id {
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
        // Note: dispute_reason is not stored in EscrowHold struct
        // Could be stored in a separate dispute record if needed
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
            .await?;

        let mut buyer_wallet = self
            .repo
            .find_wallet_by_user_id(&escrow.buyer_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Buyer wallet not found".to_string()))?;

        // Use MongoDB transaction
        let mut session = self.repo.start_session().await?;
        session.start_transaction().await
            .map_err(|e| ServiceError::DatabaseError(format!("Failed to start transaction: {}", e)))?;

        let now = BsonDateTime::now();

        // 1. Deduct from platform
        let platform_balance_before = platform_wallet.available_trust;
        platform_wallet.available_trust -= escrow.amount;
        platform_wallet.total_trust -= escrow.amount;
        Self::validate_invariant(&platform_wallet)?;

        let platform_tx_id = Self::generate_id("TXN");
        let platform_tx = Transaction::new(
            platform_tx_id,
            platform_wallet.wallet_id.clone(),
            platform_wallet.user_id.clone(),
            TransactionType::DisputeRefund,
            Direction::Debit,
            escrow.amount,
            platform_balance_before,
            platform_wallet.available_trust,
            BalanceType::Available,
            admin_id.clone(),
        );

        // 2. Refund to buyer (reverse the spent)
        let buyer_balance_before = buyer_wallet.available_trust;
        buyer_wallet.available_trust += escrow.amount;
        buyer_wallet.total_trust += escrow.amount;
        buyer_wallet.lifetime_spent -= escrow.amount; // Reverse
        Self::validate_invariant(&buyer_wallet)?;

        let buyer_tx_id = Self::generate_id("TXN");
        let buyer_tx = Transaction::new(
            buyer_tx_id.clone(),
            buyer_wallet.wallet_id.clone(),
            buyer_wallet.user_id.clone(),
            TransactionType::DisputeRefund,
            Direction::Credit,
            escrow.amount,
            buyer_balance_before,
            buyer_wallet.available_trust,
            BalanceType::Available,
            admin_id.clone(),
        );

        // 3. Update escrow
        escrow.status = EscrowStatus::Refunded;
        escrow.released_at = Some(now);
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
            .update_escrow_hold_with_session(&escrow, &mut session)
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
            amount: Some(escrow.amount),
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

        session.commit_transaction().await
            .map_err(|e| ServiceError::DatabaseError(format!("Failed to commit transaction: {}", e)))?;

        Ok(SuccessResponse::new(format!(
            "Dispute resolved with refund. {} Trust returned to buyer.",
            escrow.amount
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
            amount: Some(escrow.amount),
            reason,
            note: None,
            transaction_id: None,
            ip_address: "0.0.0.0".to_string(),
            user_agent: "".to_string(),
            created_at: BsonDateTime::now(),
        };
        self.repo.create_admin_log(log).await?;

        Ok(SuccessResponse::new("Dispute resolved with release. Money released to seller.".to_string()))
    }

    // ========================================================================
    // BACKGROUND JOB: Auto-release escrows
    // ========================================================================

    /// Process auto-release for all escrows past their hold period
    pub async fn process_auto_releases(&self) -> Result<ProcessAutoReleaseResponse, ServiceError> {
        let _now = BsonDateTime::now();
        let escrows = self.repo.find_escrows_ready_for_release().await?;

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

    // ========================================================================
    // DISPUTE CASE SYSTEM V2 - Enhanced Dispute Flow
    // ========================================================================

    /// Buyer creates dispute
    pub async fn create_dispute_case(
        &self,
        escrow_id: &str,
        buyer_id: String,
        req: CreateDisputeRequest,
    ) -> Result<DisputeInfoResponse, ServiceError> {
        // Get escrow
        let escrow = self
            .repo
            .find_escrow_by_id(escrow_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Escrow not found".to_string()))?;

        // Verify buyer owns this escrow
        if escrow.buyer_id != buyer_id {
            return Err(ServiceError::Forbidden("Not your escrow".to_string()));
        }

        // Check escrow status - only can dispute when HOLDING
        if escrow.status != EscrowStatus::Holding {
            return Err(ServiceError::BadRequest(
                format!("Cannot dispute escrow with status: {:?}", escrow.status),
            ));
        }

        // Check for existing dispute
        if let Some(_) = self.repo.find_dispute_by_escrow_id(escrow_id).await? {
            return Err(ServiceError::BadRequest(
                "Dispute already exists for this escrow".to_string(),
            ));
        }

        // Validate evidence images (max 5)
        if req.evidence_images.len() > 5 {
            return Err(ServiceError::BadRequest(
                "Maximum 5 evidence images allowed".to_string(),
            ));
        }

        // Create dispute case
        let dispute_id = Self::generate_id("DSP");
        let dispute = DisputeCase::new_buyer_dispute(
            dispute_id.clone(),
            escrow_id.to_string(),
            escrow.status.clone(),
            escrow.order_id.clone(),
            buyer_id.clone(),
            escrow.seller_id.clone(),
            escrow.amount,
            req.reason.clone(),
            req.evidence_images.clone(),
        );

        self.repo.create_dispute_case(dispute).await?;

        // Update escrow status to DISPUTED
        let mut updated_escrow = escrow.clone();
        updated_escrow.status = EscrowStatus::Disputed;
        self.repo.update_escrow_hold(&updated_escrow).await?;

        Ok(DisputeInfoResponse {
            dispute_id,
            escrow_id: escrow.escrow_id,
            order_id: escrow.order_id,
            buyer_id: escrow.buyer_id,
            seller_id: escrow.seller_id,
            amount: escrow.amount,
            status: DisputeStatus::Pending,
            dispute_type: DisputeType::RefundRequest,
            buyer_reason: req.reason,
            buyer_evidence_images: req.evidence_images,
            buyer_updates_count: 0,
            buyer_created_at: BsonDateTime::now().try_to_rfc3339_string().unwrap_or_default(),
            seller_action: None,
            seller_response: None,
            seller_evidence_images: vec![],
            seller_updates_count: 0,
            seller_offer_amount: None,
            seller_deadline: BsonDateTime::now().try_to_rfc3339_string().unwrap_or_default(),
            buyer_deadline: None,
            escalated_at: None,
            escalated_by: None,
            exchange_count: 0,
            exchange_remaining: 6,
            refund_amount: None,
            seller_amount: None,
            commission_amount: None,
            created_at: BsonDateTime::now().try_to_rfc3339_string().unwrap_or_default(),
            updated_at: BsonDateTime::now().try_to_rfc3339_string().unwrap_or_default(),
        })
    }

    /// Seller responds to dispute
    pub async fn seller_respond_dispute(
        &self,
        escrow_id: &str,
        seller_id: String,
        req: SellerDisputeResponseRequest,
    ) -> Result<DisputeInfoResponse, ServiceError> {
        // Get dispute
        let mut dispute = self
            .repo
            .find_dispute_by_escrow_id(escrow_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Dispute not found".to_string()))?;

        // Verify seller owns this dispute
        if dispute.seller_id != seller_id {
            return Err(ServiceError::Forbidden("Not your dispute".to_string()));
        }

        // Check status - only PENDING allows seller response
        if dispute.status != DisputeStatus::Pending {
            return Err(ServiceError::BadRequest(
                format!("Cannot respond to dispute with status: {:?}", dispute.status),
            ));
        }

        // Validate evidence images
        if req.evidence_images.len() > 5 {
            return Err(ServiceError::BadRequest(
                "Maximum 5 evidence images allowed".to_string(),
            ));
        }

        // Validate partial offer amount
        if req.seller_action == SellerAction::PartialAccept {
            if let Some(offer) = req.offer_amount {
                if offer <= 0 || offer >= dispute.amount {
                    return Err(ServiceError::BadRequest(
                        "Offer amount must be between 0 and full amount".to_string(),
                    ));
                }
                dispute.seller_offer_amount = Some(offer);
            } else {
                return Err(ServiceError::BadRequest(
                    "Offer amount required for PARTIAL_ACCEPT".to_string(),
                ));
            }
        }

        // Validate replacement items for REPLACEMENT action
        if req.seller_action == SellerAction::Replacement {
            if req.replacement_items.is_none() {
                return Err(ServiceError::BadRequest(
                    "Replacement items file required for REPLACEMENT".to_string(),
                ));
            }
            dispute.seller_replacement_items = req.replacement_items;
        }

        // Set seller action
        dispute.set_seller_action(req.seller_action.clone());
        dispute.seller_response = Some(req.response);
        dispute.seller_evidence_images = req.evidence_images;

        // If ACCEPT, no buyer deadline needed (auto-resolve)
        // Otherwise set buyer deadline (24 hours)
        if !matches!(req.seller_action, SellerAction::Accept) {
            let buyer_deadline = BsonDateTime::from_millis(
                BsonDateTime::now().timestamp_millis() + (24 * 60 * 60 * 1000),
            );
            dispute.buyer_deadline = Some(buyer_deadline);
        }

        self.repo.update_dispute_case(&dispute).await?;

        Ok(self.convert_dispute_to_response(&dispute))
    }

    /// Buyer responds to dispute (multi-exchange)
    pub async fn buyer_respond_dispute(
        &self,
        dispute_id: &str,
        buyer_id: String,
        req: BuyerDisputeResponseRequest,
    ) -> Result<DisputeInfoResponse, ServiceError> {
        let mut dispute = self
            .repo
            .find_dispute_by_id(dispute_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Dispute not found".to_string()))?;

        // Verify buyer owns this dispute
        if dispute.buyer_id != buyer_id {
            return Err(ServiceError::Forbidden("Not your dispute".to_string()));
        }

        // Check status
        if !matches!(
            dispute.status,
            DisputeStatus::SellerResponded | DisputeStatus::BuyerResponded
        ) {
            return Err(ServiceError::BadRequest(
                format!("Cannot respond to dispute with status: {:?}", dispute.status),
            ));
        }

        // Validate additional images
        if req.additional_images.len() > 3 {
            return Err(ServiceError::BadRequest(
                "Maximum 3 additional images per update".to_string(),
            ));
        }

        match req.decision {
            BuyerDisputeDecision::AcceptOffer => {
                // Buyer accepts seller's offer
                if let Some(offer_amount) = dispute.seller_offer_amount {
                    // Process partial refund
                    self.process_partial_refund(&dispute, offer_amount).await?;
                    dispute.resolve_refunded(offer_amount, buyer_id);
                } else {
                    // No offer, but seller accepted - full refund
                    self.process_refund_from_dispute(&dispute).await?;
                    dispute.resolve_refunded(dispute.amount, buyer_id);
                }
            }
            BuyerDisputeDecision::Escalate => {
                // Validate escalation message
                if req.message.is_none() || req.message.as_ref().is_some_and(|m| m.len() < 20) {
                    return Err(ServiceError::BadRequest(
                        "Escalation message must be at least 20 characters".to_string(),
                    ));
                }

                // Check max exchanges
                if dispute.is_max_exchanges_reached() {
                    // Auto-escalate
                    dispute.escalate(
                        buyer_id.clone(),
                        req.message.unwrap_or_else(|| "Max exchanges reached".to_string()),
                    );
                } else {
                    // Add buyer update and continue
                    dispute.add_buyer_update(
                        req.message.unwrap_or_else(|| "Please escalate this".to_string()),
                        req.additional_images,
                    );
                    dispute.status = DisputeStatus::BuyerResponded;

                    // Set new seller deadline (48 hours)
                    let new_seller_deadline = BsonDateTime::from_millis(
                        BsonDateTime::now().timestamp_millis() + (48 * 60 * 60 * 1000),
                    );
                    dispute.seller_deadline = new_seller_deadline;
                }
            }
        }

        self.repo.update_dispute_case(&dispute).await?;

        Ok(self.convert_dispute_to_response(&dispute))
    }

    /// Admin extends deadline
    pub async fn admin_extend_deadline(
        &self,
        admin_id: String,
        req: AdminExtendDeadlineRequest,
    ) -> Result<DisputeInfoResponse, ServiceError> {
        let mut dispute = self
            .repo
            .find_dispute_by_id(&req.dispute_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Dispute not found".to_string()))?;

        // Extend deadline
        dispute.extend_deadline(req.extension_days, admin_id.clone(), req.reason.clone());

        self.repo.update_dispute_case(&dispute).await?;

        // Create admin log
        let log = AdminOperationLog {
            id: None,
            log_id: Self::generate_id("ALOG"),
            admin_id: admin_id.clone(),
            admin_email: "admin@example.com".to_string(),
            admin_role: "ADMIN".to_string(),
            operation: AdminOperation::ForceReleaseEscrow, // Using existing enum
            target_type: TargetType::Escrow,
            target_id: req.dispute_id.clone(),
            before_state: serde_json::json!({"deadline": dispute.seller_deadline}),
            after_state: serde_json::json!({"deadline": dispute.new_deadline}),
            amount: None,
            reason: req.reason,
            note: Some(format!("Extended {} days", req.extension_days)),
            transaction_id: None,
            ip_address: "0.0.0.0".to_string(),
            user_agent: "".to_string(),
            created_at: BsonDateTime::now(),
        };
        self.repo.create_admin_log(log).await?;

        Ok(self.convert_dispute_to_response(&dispute))
    }

    /// Admin partial refund
    pub async fn admin_partial_refund(
        &self,
        admin_id: String,
        req: AdminPartialRefundRequest,
    ) -> Result<DisputeInfoResponse, ServiceError> {
        let mut dispute = self
            .repo
            .find_dispute_by_id(&req.dispute_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Dispute not found".to_string()))?;

        // Calculate amounts
        let buyer_amount = (dispute.amount as f64 * (req.buyer_percent as f64 / 100.0)) as i64;
        let seller_amount = dispute.amount - buyer_amount;
        let commission_amount = if seller_amount > 0 {
            (seller_amount as f64 * DEFAULT_COMMISSION_RATE) as i64
        } else {
            0
        };

        // Process partial refund
        let mut session = self.repo.start_session().await?;
        session.start_transaction().await
            .map_err(|e| ServiceError::DatabaseError(format!("Failed to start transaction: {}", e)))?;

        // Get wallets
        let mut platform_wallet = self.repo.get_platform_wallet_with_session(&mut session).await?;
        let mut buyer_wallet = self.repo.find_wallet_by_user_id_with_session(&dispute.buyer_id, &mut session).await?
            .ok_or_else(|| ServiceError::NotFound("Buyer wallet not found".to_string()))?;
        let mut seller_wallet = self.repo.find_wallet_by_user_id_with_session(&dispute.seller_id, &mut session).await?
            .ok_or_else(|| ServiceError::NotFound("Seller wallet not found".to_string()))?;

        // Get escrow
        let mut escrow = self.repo.find_escrow_by_id(&dispute.escrow_id).await?
            .ok_or_else(|| ServiceError::NotFound("Escrow not found".to_string()))?;

        // 1. Refund to buyer
        buyer_wallet.available_trust += buyer_amount;
        buyer_wallet.total_trust += buyer_amount;
        Self::validate_invariant(&buyer_wallet)?;

        let buyer_tx = Transaction::new(
            Self::generate_id("TXN"),
            buyer_wallet.wallet_id.clone(),
            buyer_wallet.user_id.clone(),
            TransactionType::DisputeRefund,
            Direction::Credit,
            buyer_amount,
            buyer_wallet.available_trust - buyer_amount,
            buyer_wallet.available_trust,
            BalanceType::Available,
            admin_id.clone(),
        );

        // 2. Release to seller (after commission)
        seller_wallet.available_trust += seller_amount - commission_amount;
        seller_wallet.total_trust += seller_amount - commission_amount;
        Self::validate_invariant(&seller_wallet)?;

        let seller_tx = Transaction::new(
            Self::generate_id("TXN"),
            seller_wallet.wallet_id.clone(),
            seller_wallet.user_id.clone(),
            TransactionType::EscrowRelease,
            Direction::Credit,
            seller_amount - commission_amount,
            seller_wallet.available_trust - (seller_amount - commission_amount),
            seller_wallet.available_trust,
            BalanceType::Available,
            admin_id.clone(),
        );

        // 3. Platform deduct
        platform_wallet.available_trust -= dispute.amount;
        platform_wallet.total_trust -= dispute.amount;
        Self::validate_invariant(&platform_wallet)?;

        // Save everything
        self.repo.create_transaction_with_session(buyer_tx, &mut session).await?;
        self.repo.create_transaction_with_session(seller_tx, &mut session).await?;
        self.repo.update_wallet_with_session(&buyer_wallet, &mut session).await?;
        self.repo.update_wallet_with_session(&seller_wallet, &mut session).await?;
        self.repo.update_wallet_with_session(&platform_wallet, &mut session).await?;

        // Update escrow
        escrow.status = EscrowStatus::PartialRefund;
        escrow.released_at = Some(BsonDateTime::now());
        self.repo.update_escrow_hold_with_session(&escrow, &mut session).await?;

        session.commit_transaction().await
            .map_err(|e| ServiceError::DatabaseError(format!("Failed to commit: {}", e)))?;

        // Update dispute
        dispute.status = DisputeStatus::PartialRefund;
        dispute.resolved_at = Some(BsonDateTime::now());
        dispute.resolved_by = Some(admin_id.clone());
        dispute.refund_amount = Some(buyer_amount);
        dispute.seller_amount = Some(seller_amount - commission_amount);
        dispute.commission_amount = Some(commission_amount);
        dispute.admin_decision = Some(req.reason.clone());
        self.repo.update_dispute_case(&dispute).await?;

        Ok(self.convert_dispute_to_response(&dispute))
    }

    /// Process auto-escalate (cron job)
    pub async fn process_auto_escalate(&self) -> Result<ProcessAutoEscalateResponse, ServiceError> {
        let mut escalated_count = 0;
        let mut auto_resolved_count = 0;
        let mut failed_count = 0;
        let mut escalated_ids = vec![];
        let mut resolved_ids = vec![];
        let mut errors = vec![];

        // Check seller deadlines
        let seller_deadline_disputes = self.repo.find_disputes_past_seller_deadline().await?;
        for dispute in seller_deadline_disputes {
            match self.auto_escalate_seller(&dispute).await {
                Ok(_) => {
                    escalated_count += 1;
                    escalated_ids.push(dispute.dispute_id);
                }
                Err(e) => {
                    failed_count += 1;
                    errors.push(format!("{}: {}", dispute.dispute_id, e));
                }
            }
        }

        // Check buyer deadlines
        let buyer_deadline_disputes = self.repo.find_disputes_past_buyer_deadline().await?;
        for dispute in buyer_deadline_disputes {
            if dispute.can_auto_resolve() {
                // Auto-resolve (seller accepted, buyer no response)
                match self.auto_resolve_buyer(&dispute).await {
                    Ok(_) => {
                        auto_resolved_count += 1;
                        resolved_ids.push(dispute.dispute_id);
                    }
                    Err(e) => {
                        failed_count += 1;
                        errors.push(format!("{}: {}", dispute.dispute_id, e));
                    }
                }
            } else {
                // Auto-escalate
                match self.auto_escalate_buyer(&dispute).await {
                    Ok(_) => {
                        escalated_count += 1;
                        escalated_ids.push(dispute.dispute_id);
                    }
                    Err(e) => {
                        failed_count += 1;
                        errors.push(format!("{}: {}", dispute.dispute_id, e));
                    }
                }
            }
        }

        Ok(ProcessAutoEscalateResponse {
            total_processed: escalated_count + auto_resolved_count + failed_count,
            escalated_count,
            auto_resolved_count,
            failed_count,
            escalated_ids,
            resolved_ids,
            errors,
        })
    }

    // ========================================================================
    // HELPER METHODS
    // ========================================================================

    fn convert_dispute_to_response(&self, dispute: &DisputeCase) -> DisputeInfoResponse {
        DisputeInfoResponse {
            dispute_id: dispute.dispute_id.clone(),
            escrow_id: dispute.escrow_id.clone(),
            order_id: dispute.order_id.clone(),
            buyer_id: dispute.buyer_id.clone(),
            seller_id: dispute.seller_id.clone(),
            amount: dispute.amount,
            status: dispute.status.clone(),
            dispute_type: dispute.dispute_type.clone(),
            buyer_reason: dispute.buyer_reason.clone(),
            buyer_evidence_images: dispute.buyer_evidence_images.clone(),
            buyer_updates_count: dispute.buyer_updates.len() as i32,
            buyer_created_at: dispute.buyer_created_at.try_to_rfc3339_string().unwrap_or_default(),
            seller_action: dispute.seller_action.clone(),
            seller_response: dispute.seller_response.clone(),
            seller_evidence_images: dispute.seller_evidence_images.clone(),
            seller_updates_count: dispute.seller_updates.len() as i32,
            seller_offer_amount: dispute.seller_offer_amount,
            seller_deadline: dispute.seller_deadline.try_to_rfc3339_string().unwrap_or_default(),
            buyer_deadline: dispute.buyer_deadline.as_ref().map(|d| d.try_to_rfc3339_string().unwrap_or_default()),
            escalated_at: dispute.escalated_at.as_ref().map(|d| d.try_to_rfc3339_string().unwrap_or_default()),
            escalated_by: dispute.escalated_by.clone(),
            exchange_count: dispute.exchange_count,
            exchange_remaining: 6 - dispute.exchange_count,
            refund_amount: dispute.refund_amount,
            seller_amount: dispute.seller_amount,
            commission_amount: dispute.commission_amount,
            created_at: dispute.created_at.try_to_rfc3339_string().unwrap_or_default(),
            updated_at: dispute.updated_at.try_to_rfc3339_string().unwrap_or_default(),
        }
    }

    async fn process_partial_refund(&self, dispute: &DisputeCase, refund_amount: i64) -> Result<(), ServiceError> {
        let mut session = self.repo.start_session().await?;
        session.start_transaction().await
            .map_err(|e| ServiceError::DatabaseError(format!("Failed to start transaction: {}", e)))?;

        // Get wallets and escrow
        let mut platform_wallet = self.repo.get_platform_wallet_with_session(&mut session).await?;
        let mut buyer_wallet = self.repo.find_wallet_by_user_id_with_session(&dispute.buyer_id, &mut session).await?
            .ok_or_else(|| ServiceError::NotFound("Buyer wallet not found".to_string()))?;
        let mut seller_wallet = self.repo.find_wallet_by_user_id_with_session(&dispute.seller_id, &mut session).await?
            .ok_or_else(|| ServiceError::NotFound("Seller wallet not found".to_string()))?;
        let mut escrow = self.repo.find_escrow_by_id(&dispute.escrow_id).await?
            .ok_or_else(|| ServiceError::NotFound("Escrow not found".to_string()))?;

        // Calculate amounts
        let seller_amount = dispute.amount - refund_amount;
        let commission_amount = (seller_amount as f64 * DEFAULT_COMMISSION_RATE) as i64;

        // Refund buyer
        buyer_wallet.available_trust += refund_amount;
        buyer_wallet.total_trust += refund_amount;
        Self::validate_invariant(&buyer_wallet)?;

        // Release to seller (after commission)
        seller_wallet.available_trust += seller_amount - commission_amount;
        seller_wallet.total_trust += seller_amount - commission_amount;
        Self::validate_invariant(&seller_wallet)?;

        // Platform deduct
        platform_wallet.available_trust -= dispute.amount;
        platform_wallet.total_trust -= dispute.amount;

        let buyer_tx = Transaction::new(
            Self::generate_id("TXN"),
            buyer_wallet.wallet_id.clone(),
            buyer_wallet.user_id.clone(),
            TransactionType::DisputeRefund,
            Direction::Credit,
            refund_amount,
            buyer_wallet.available_trust - refund_amount,
            buyer_wallet.available_trust,
            BalanceType::Available,
            "SYSTEM".to_string(),
        );

        let seller_tx = Transaction::new(
            Self::generate_id("TXN"),
            seller_wallet.wallet_id.clone(),
            seller_wallet.user_id.clone(),
            TransactionType::EscrowRelease,
            Direction::Credit,
            seller_amount - commission_amount,
            seller_wallet.available_trust - (seller_amount - commission_amount),
            seller_wallet.available_trust,
            BalanceType::Available,
            "SYSTEM".to_string(),
        );

        // Save
        self.repo.create_transaction_with_session(buyer_tx, &mut session).await?;
        self.repo.create_transaction_with_session(seller_tx, &mut session).await?;
        self.repo.update_wallet_with_session(&buyer_wallet, &mut session).await?;
        self.repo.update_wallet_with_session(&seller_wallet, &mut session).await?;
        self.repo.update_wallet_with_session(&platform_wallet, &mut session).await?;

        // Update escrow
        escrow.status = EscrowStatus::PartialRefund;
        escrow.released_at = Some(BsonDateTime::now());
        self.repo.update_escrow_hold_with_session(&escrow, &mut session).await?;

        session.commit_transaction().await
            .map_err(|e| ServiceError::DatabaseError(format!("Failed to commit: {}", e)))?;

        Ok(())
    }

    async fn process_refund_from_dispute(&self, dispute: &DisputeCase) -> Result<(), ServiceError> {
        // Similar to resolve_dispute_refund but without changing escrow to refunded first
        let mut session = self.repo.start_session().await?;
        session.start_transaction().await
            .map_err(|e| ServiceError::DatabaseError(format!("Failed to start transaction: {}", e)))?;

        let mut platform_wallet = self.repo.get_platform_wallet_with_session(&mut session).await?;
        let mut buyer_wallet = self.repo.find_wallet_by_user_id_with_session(&dispute.buyer_id, &mut session).await?
            .ok_or_else(|| ServiceError::NotFound("Buyer wallet not found".to_string()))?;
        let mut escrow = self.repo.find_escrow_by_id(&dispute.escrow_id).await?
            .ok_or_else(|| ServiceError::NotFound("Escrow not found".to_string()))?;

        // Refund full amount
        buyer_wallet.available_trust += dispute.amount;
        buyer_wallet.total_trust += dispute.amount;
        buyer_wallet.lifetime_spent -= dispute.amount;
        Self::validate_invariant(&buyer_wallet)?;

        platform_wallet.available_trust -= dispute.amount;
        platform_wallet.total_trust -= dispute.amount;

        let buyer_tx = Transaction::new(
            Self::generate_id("TXN"),
            buyer_wallet.wallet_id.clone(),
            buyer_wallet.user_id.clone(),
            TransactionType::DisputeRefund,
            Direction::Credit,
            dispute.amount,
            buyer_wallet.available_trust - dispute.amount,
            buyer_wallet.available_trust,
            BalanceType::Available,
            "SYSTEM".to_string(),
        );

        self.repo.create_transaction_with_session(buyer_tx, &mut session).await?;
        self.repo.update_wallet_with_session(&buyer_wallet, &mut session).await?;
        self.repo.update_wallet_with_session(&platform_wallet, &mut session).await?;

        escrow.status = EscrowStatus::Refunded;
        escrow.released_at = Some(BsonDateTime::now());
        self.repo.update_escrow_hold_with_session(&escrow, &mut session).await?;

        session.commit_transaction().await
            .map_err(|e| ServiceError::DatabaseError(format!("Failed to commit: {}", e)))?;

        Ok(())
    }

    async fn auto_escalate_seller(&self, dispute: &DisputeCase) -> Result<(), ServiceError> {
        let mut updated = dispute.clone();
        updated.escalate(
            "SYSTEM".to_string(),
            "Seller did not respond within 48 hours".to_string(),
        );
        self.repo.update_dispute_case(&updated).await?;
        Ok(())
    }

    async fn auto_escalate_buyer(&self, dispute: &DisputeCase) -> Result<(), ServiceError> {
        let mut updated = dispute.clone();
        updated.escalate(
            "SYSTEM".to_string(),
            "Buyer did not respond within 24 hours".to_string(),
        );
        self.repo.update_dispute_case(&updated).await?;
        Ok(())
    }

    async fn auto_resolve_buyer(&self, dispute: &DisputeCase) -> Result<(), ServiceError> {
        // Seller accepted, buyer no response - auto-resolve with seller's offer
        if let Some(offer_amount) = dispute.seller_offer_amount {
            self.process_partial_refund(dispute, offer_amount).await?;
        } else {
            // Seller accepted with no offer = full refund
            self.process_refund_from_dispute(dispute).await?;
        }

        let mut updated = dispute.clone();
        updated.resolve_refunded(
            updated.seller_offer_amount.unwrap_or(updated.amount),
            "SYSTEM".to_string(),
        );
        self.repo.update_dispute_case(&updated).await?;

        Ok(())
    }

    /// Get disputes list for user
    pub async fn get_disputes_list(
        &self,
        _user_id: &str,
        query: DisputeListQuery,
    ) -> Result<DisputeListResponse, ServiceError> {
        let page = query.page.max(1);
        let per_page = query.per_page.clamp(1, 100);

        // Build filter
        let mut filter = doc! {};
        if let Some(status) = query.status {
            filter.insert("status", status);
        }
        if let Some(filter_user_id) = query.user_id {
            filter.insert("$or", vec![
                doc! { "buyer_id": &filter_user_id },
                doc! { "seller_id": &filter_user_id }
            ]);
        }
        if let Some(order_id) = query.order_id {
            filter.insert("order_id", order_id);
        }

        let (disputes, total) = self
            .repo
            .find_disputes_list(Some(filter), page, per_page)
            .await?;

        let items: Vec<DisputeInfoResponse> = disputes
            .iter()
            .map(|d| self.convert_dispute_to_response(d))
            .collect();

        Ok(DisputeListResponse {
            disputes: items,
            total,
            page,
            per_page,
        })
    }

    /// Get dispute detail by ID
    pub async fn get_dispute_detail(
        &self,
        dispute_id: &str,
    ) -> Result<DisputeInfoResponse, ServiceError> {
        let dispute = self
            .repo
            .find_dispute_by_id(dispute_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Dispute not found".to_string()))?;

        Ok(self.convert_dispute_to_response(&dispute))
    }
}
