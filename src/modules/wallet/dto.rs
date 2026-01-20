//! Wallet V3 DTOs
//!
//! Request and Response structures for all wallet endpoints
//! Based on TaphoaMMO Trust Wallet V3 Design

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use super::domain::{
    BalanceType, Direction, EscrowStatus, Severity, SnapshotStatus, TransactionStatus,
    TransactionType, ValidationResult, WalletStatus, WalletType, WithdrawalStatus,
};

// ============================================================================
// WALLET BALANCE & INFO RESPONSES
// ============================================================================

/// Get wallet balance response
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WalletBalanceResponse {
    pub wallet_id: String,
    pub wallet_type: WalletType,

    // Balance states
    pub available_trust: i64,
    pub withdrawal_locked: i64,
    pub dispute_locked: i64,
    pub total_trust: i64,

    // VND equivalent
    pub available_vnd: i64,
    pub total_vnd: i64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub commission_debt: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub commission_rate: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_debt: Option<i64>,

    pub status: WalletStatus,
}

/// Wallet detailed info response
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WalletInfoResponse {
    pub wallet_id: String,
    pub user_id: String,
    pub wallet_type: WalletType,

    // Balance states
    pub available_trust: i64,
    pub withdrawal_locked: i64,
    pub dispute_locked: i64,
    pub total_trust: i64,

    // Running totals
    pub lifetime_deposited: i64,
    pub lifetime_withdrawn: i64,
    pub lifetime_spent: i64,
    pub lifetime_received: i64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub commission_debt: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub commission_rate: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_debt: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_debt_reason: Option<String>,

    // Snapshot info
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_snapshot_month: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_snapshot_balance: Option<i64>,

    pub status: WalletStatus,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeze_reason: Option<String>,

    pub created_at: String,
    pub updated_at: String,
}

// ============================================================================
// DEPOSIT DTOs
// ============================================================================

/// Auto deposit request (via payment gateway)
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AutoDepositRequest {
    /// VND amount (must be divisible by 1000)
    #[validate(range(
        min = 10000,
        max = 50000000,
        message = "Amount must be between 10,000 and 50,000,000 VND"
    ))]
    #[validate(custom(function = "validate_divisible_by_1000"))]
    pub vnd_amount: i64,

    /// Payment gateway: VNPay, MoMo, etc.
    #[validate(length(min = 1, max = 50))]
    pub payment_gateway: String,
}

/// Manual deposit request (admin only)
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ManualDepositRequest {
    /// Target user ID
    #[validate(length(min = 1))]
    pub user_id: String,

    /// Trust amount to deposit
    #[validate(range(
        min = 1,
        max = 1000000,
        message = "Amount must be between 1 and 1,000,000 Trust"
    ))]
    pub trust_amount: i64,

    /// Reason for manual deposit (required)
    #[validate(length(min = 10, max = 500))]
    pub reason: String,

    /// Optional note
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(max = 1000))]
    pub note: Option<String>,
}

/// Deposit response
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DepositResponse {
    pub deposit_id: String,
    pub wallet_id: String,
    pub vnd_amount: i64,
    pub trust_amount: i64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_gateway: Option<String>,

    pub status: String,
    pub expires_at: Option<String>,
    pub created_at: String,
}

/// Deposit initiate request (3rd party payment)
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DepositInitiateRequest {
    /// VND amount (must be divisible by 1000)
    #[validate(range(
        min = 10000,
        max = 50000000,
        message = "Amount must be between 10,000 and 50,000,000 VND"
    ))]
    #[validate(custom(function = "validate_divisible_by_1000"))]
    pub amount_vnd: i64,

    /// Payment method: VNPay, MoMo, BankTransfer
    #[validate(length(min = 1, max = 50))]
    pub payment_method: String,

    /// Return URL after payment
    #[validate(url)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_url: Option<String>,

    /// Cancel URL
    #[validate(url)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancel_url: Option<String>,
}

/// Deposit initiate response
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DepositInitiateResponse {
    pub deposit_id: String,
    pub transaction_id: String,
    pub amount_vnd: i64,
    pub trust_amount: i64,
    pub payment_method: String,
    pub payment_url: String,
    pub expires_at: String,
    pub status: String,
    pub created_at: String,
}

/// Deposit status response
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DepositStatusResponse {
    pub deposit_id: String,
    pub transaction_id: String,
    pub amount_vnd: i64,
    pub trust_amount: i64,
    pub payment_method: String,
    pub status: String,
    pub payment_gateway_ref: Option<String>,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

/// Webhook payload from payment gateway
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PaymentWebhookPayload {
    /// Gateway transaction ID
    pub transaction_id: String,

    /// Our deposit ID
    pub deposit_id: String,

    /// Payment status: success, cancelled, failed
    pub status: String,

    /// Amount in VND
    pub amount_vnd: i64,

    /// Signature for verification
    pub signature: String,

    /// Additional gateway data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway_data: Option<serde_json::Value>,
}

/// Deposit history item
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DepositHistoryItem {
    pub deposit_id: String,
    pub transaction_id: String,
    pub amount_vnd: i64,
    pub trust_amount: i64,
    pub payment_method: String,
    pub status: String,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
}

/// Deposit history response
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DepositHistoryResponse {
    pub deposits: Vec<DepositHistoryItem>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

// ============================================================================
// WITHDRAWAL DTOs
// ============================================================================

/// Withdrawal request
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawalRequest {
    /// Trust amount to withdraw
    #[validate(range(
        min = 10,
        max = 100000,
        message = "Amount must be between 10 and 100,000 Trust"
    ))]
    pub trust_amount: i64,

    /// Bank information
    #[validate(length(min = 1, max = 50))]
    pub bank_code: String,

    #[validate(length(min = 1, max = 100))]
    pub bank_name: String,

    #[validate(length(min = 1, max = 50))]
    pub account_number: String,

    #[validate(length(min = 1, max = 100))]
    pub account_name: String,
}

/// Withdrawal response
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawalResponse {
    pub request_id: String,
    pub wallet_id: String,
    pub trust_amount: i64,
    pub commission_deduct: i64,
    pub net_trust: i64,
    pub vnd_amount: i64,
    pub bank_info: BankInfo,
    pub status: WithdrawalStatus,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_result: Option<ValidationResult>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bank_transfer_ref: Option<String>,

    pub created_at: String,
    pub expires_at: String,
}

/// Bank info in response
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BankInfo {
    pub bank_code: String,
    pub bank_name: String,
    pub account_number: String,
    pub account_name: String,
}

// ============================================================================
// PURCHASE DTOs
// ============================================================================

/// Purchase request (buyer buys product)
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseRequest {
    #[validate(length(min = 1))]
    pub order_id: String,

    #[validate(length(min = 1))]
    pub seller_user_id: String,

    #[validate(length(min = 1))]
    pub product_id: String,

    #[validate(length(min = 1))]
    pub product_name: String,

    #[validate(range(min = 1))]
    pub trust_amount: i64,
}

/// Purchase response
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseResponse {
    pub order_id: String,
    pub escrow_id: String,
    pub buyer_wallet_id: String,
    pub seller_id: String,
    pub amount: i64,
    pub release_at: String,
    pub created_at: String,
}

// ============================================================================
// ESCROW DTOs
// ============================================================================

/// Early release request (buyer confirms receipt)
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EarlyReleaseRequest {
    #[validate(length(min = 1))]
    pub order_id: String,
}

/// Escrow info response
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EscrowInfoResponse {
    pub escrow_id: String,
    pub order_id: String,
    pub buyer_id: String,
    pub seller_id: String,
    pub amount: i64,
    pub status: EscrowStatus,
    pub release_at: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub released_at: Option<String>,

    pub early_release: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub commission_amount: Option<i64>,

    pub created_at: String,
}

// ============================================================================
// REFUND DTOs
// ============================================================================

/// Refund request (buyer requests refund)
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RefundRequest {
    #[validate(length(min = 1))]
    pub order_id: String,

    #[validate(length(min = 20, max = 500))]
    pub reason: String,
}

/// Seller cancel order request
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SellerCancelRequest {
    #[validate(length(min = 1))]
    pub order_id: String,

    #[validate(length(min = 20, max = 500))]
    pub reason: String,
}

// ============================================================================
// ADMIN OPERATION DTOs
// ============================================================================

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminDebitRequest {
    #[validate(length(min = 1))]
    pub user_id: String,

    #[serde(alias = "amount")]
    #[validate(range(min = 1))]
    pub trust_amount: i64,

    #[validate(length(min = 20, max = 500))]
    pub reason: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(max = 1000))]
    pub note: Option<String>,

    #[serde(default)]
    pub allow_debt: bool,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminDebitResponse {
    pub wallet_id: String,
    pub user_id: String,
    pub requested_amount: i64,
    pub actual_deducted: i64,
    pub debt_created: i64,
    pub new_available: i64,
    pub new_admin_debt: i64,
    pub debt_id: Option<String>,
}

/// Admin wallet freeze request
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminFreezeRequest {
    #[validate(length(min = 1))]
    pub user_id: String,

    /// Amount to freeze (or None for full freeze)
    pub amount: Option<i64>,

    #[validate(length(min = 10, max = 500))]
    pub reason: String,

    #[validate(length(min = 1, max = 100))]
    pub case_reference: String,
}

/// Admin wallet unfreeze request
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminUnfreezeRequest {
    #[validate(length(min = 1))]
    pub user_id: String,

    #[validate(length(min = 20, max = 500))]
    pub resolution_note: String,
}

/// Admin withdrawal approval/rejection
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminWithdrawalDecisionRequest {
    #[validate(length(min = 1))]
    pub request_id: String,

    /// "approve", "reject", or "hold"
    #[validate(length(min = 1))]
    pub decision: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(min = 10, max = 500))]
    pub reason: Option<String>,
}

/// Shop commission config request
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SetCommissionRateRequest {
    #[validate(length(min = 1))]
    pub shop_id: String,

    #[validate(length(min = 1))]
    pub seller_user_id: String,

    /// Rate between 0.01 and 0.20 (1% to 20%)
    #[serde(alias = "rate")]
    #[validate(range(min = 0.01, max = 0.20))]
    pub commission_rate: f64,

    #[validate(length(min = 10, max = 500))]
    pub reason: String,

    /// Effective from (ISO date string)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_from: Option<String>,
}

// ============================================================================
// TRANSACTION HISTORY DTOs
// ============================================================================

/// Transaction history query
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TransactionHistoryQuery {
    /// Page number (1-indexed)
    #[validate(range(min = 1))]
    pub page: Option<i64>,

    /// Page size (default 20, max 100)
    #[validate(range(min = 1, max = 100))]
    pub page_size: Option<i64>,

    /// Filter by transaction type
    pub tx_type: Option<String>,

    /// Filter by status
    pub status: Option<String>,

    /// Start date (ISO format)
    pub start_date: Option<String>,

    /// End date (ISO format)
    pub end_date: Option<String>,
}

/// Transaction response
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TransactionResponse {
    pub tx_id: String,
    pub wallet_id: String,
    pub tx_type: TransactionType,
    pub direction: Direction,
    pub amount: i64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub vnd_amount: Option<i64>,

    pub fee_amount: i64,
    pub balance_before: i64,
    pub balance_after: i64,
    pub balance_type: BalanceType,
    pub status: TransactionStatus,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_ref: Option<String>,

    pub created_at: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
}

/// Paginated transaction list
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TransactionListResponse {
    pub transactions: Vec<TransactionResponse>,
    pub page: i64,
    pub page_size: i64,
    pub total_count: i64,
    pub total_pages: i64,
}

// ============================================================================
// SNAPSHOT & RECONCILIATION DTOs
// ============================================================================

/// Monthly snapshot response
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotResponse {
    pub snapshot_id: String,
    pub wallet_id: String,
    pub month: String,
    pub closing_balance: i64,
    pub actual_balance: i64,
    pub discrepancy: i64,
    pub status: SnapshotStatus,
    pub tx_count: i64,
    pub created_at: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified_at: Option<String>,
}

/// Reconciliation report response
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationReportResponse {
    pub date: String,
    pub system_total_trust: i64,
    pub platform_escrow_balance: i64,
    pub total_active_escrows: i64,
    pub total_deposits_vnd: i64,
    pub total_deposits_trust: i64,
    pub total_withdrawals_trust: i64,
    pub total_withdrawals_vnd: i64,
    pub commission_collected: i64,
    pub checks: Vec<ReconciliationCheck>,
    pub overall_status: String,
}

/// Individual reconciliation check
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationCheck {
    pub check_name: String,
    pub passed: bool,
    pub severity: Severity,
    pub details: String,
}

// ============================================================================
// ADMIN DASHBOARD DTOs
// ============================================================================

/// Admin dashboard stats
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminDashboardStats {
    pub total_trust_in_system: i64,
    pub platform_wallet_balance: i64,
    pub total_active_escrows: i64,
    pub total_escrow_amount: i64,
    pub total_commission_collected: i64,
    pub pending_withdrawals_count: i64,
    pub pending_withdrawals_amount: i64,
    pub active_disputes_count: i64,
}

/// Pending withdrawal for admin review
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PendingWithdrawalItem {
    pub request_id: String,
    pub user_id: String,
    pub wallet_id: String,
    pub trust_amount: i64,
    pub vnd_amount: i64,
    pub risk_score: f64,
    pub status: WithdrawalStatus,
    pub validation_result: Option<ValidationResult>,
    pub created_at: String,
}

// ============================================================================
// VALIDATION HELPERS
// ============================================================================

fn validate_divisible_by_1000(value: i64) -> Result<(), validator::ValidationError> {
    if value % 1000 != 0 {
        return Err(validator::ValidationError::new("not_divisible_by_1000"));
    }
    Ok(())
}

// ============================================================================
// SUCCESS RESPONSES
// ============================================================================

/// Generic success response
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessResponse {
    pub success: bool,
    pub message: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl SuccessResponse {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: None,
        }
    }

    pub fn with_data(message: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: Some(data),
        }
    }
}

// ============================================================================
// ADDITIONAL DTOs FOR HANDLERS
// ============================================================================

/// Create wallet request
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateWalletRequest {
    pub wallet_type: super::domain::WalletType,
}

// ============================================================================
// DISPUTE DTOs V2 - Enhanced Dispute System
// ============================================================================

/// Buyer creates dispute request
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateDisputeRequest {
    /// Reason for dispute (min 20 chars)
    #[validate(length(min = 20, max = 500))]
    pub reason: String,

    /// Evidence images (max 5 images)
    #[validate(length(max = 5))]
    pub evidence_images: Vec<String>,

    /// Dispute reason code
    pub dispute_reason: Option<super::domain::DisputeReason>,
}

/// Seller dispute response request
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SellerDisputeResponseRequest {
    /// Action type: ACCEPT, PARTIAL_ACCEPT, REJECT, REPLACEMENT
    #[serde(rename = "action")]
    pub seller_action: super::domain::SellerAction,

    /// Response message (min 20 chars)
    #[validate(length(min = 20, max = 500))]
    pub response: String,

    /// Evidence images (max 5 images)
    #[validate(length(max = 5))]
    pub evidence_images: Vec<String>,

    /// For PARTIAL_ACCEPT: offer amount
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(range(min = 1))]
    pub offer_amount: Option<i64>,

    /// For REPLACEMENT: items file path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replacement_items: Option<String>,
}

/// Buyer dispute response request (multi-exchange)
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BuyerDisputeResponseRequest {
    /// Buyer's decision: ACCEPT_OFFER or ESCALATE
    pub decision: BuyerDisputeDecision,

    /// Additional message (min 20 chars if escalating)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(min = 20, max = 500))]
    pub message: Option<String>,

    /// Additional evidence images (max 3 per update)
    #[validate(length(max = 3))]
    pub additional_images: Vec<String>,
}

/// Buyer decision types
#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BuyerDisputeDecision {
    /// Accept seller's offer
    AcceptOffer,
    /// Escalate to admin
    Escalate,
}

/// Admin extend deadline request
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminExtendDeadlineRequest {
    /// Dispute ID
    #[validate(length(min = 1))]
    pub dispute_id: String,

    /// Extension days (1-7)
    #[validate(range(min = 1, max = 7))]
    pub extension_days: i32,

    /// Reason for extension (min 20 chars)
    #[validate(length(min = 20, max = 500))]
    pub reason: String,
}

/// Admin partial refund request
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminPartialRefundRequest {
    /// Dispute ID or Escrow ID
    #[validate(length(min = 1))]
    pub dispute_id: String,

    /// Buyer percentage (0-100)
    #[validate(range(min = 0, max = 100))]
    pub buyer_percent: i32,

    /// Admin decision reason (min 20 chars)
    #[validate(length(min = 20, max = 500))]
    pub reason: String,
}

/// Resolve dispute request (legacy - kept for compatibility)
#[derive(Debug, Deserialize, ToSchema)]
pub struct ResolveDisputeRequest {
    pub reason: String,
}

/// Dispute info response
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DisputeInfoResponse {
    pub dispute_id: String,
    pub escrow_id: String,
    pub order_id: String,
    pub buyer_id: String,
    pub seller_id: String,
    pub amount: i64,
    pub status: super::domain::DisputeStatus,
    pub dispute_type: super::domain::DisputeType,

    // Buyer info
    pub buyer_reason: String,
    pub buyer_evidence_images: Vec<String>,
    pub buyer_updates_count: i32,
    pub buyer_created_at: String,

    // Seller info
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seller_action: Option<super::domain::SellerAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seller_response: Option<String>,
    pub seller_evidence_images: Vec<String>,
    pub seller_updates_count: i32,

    // Offer info
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seller_offer_amount: Option<i64>,

    // Deadlines
    pub seller_deadline: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buyer_deadline: Option<String>,

    // Escalation info
    #[serde(skip_serializing_if = "Option::is_none")]
    pub escalated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub escalated_by: Option<String>,

    // Exchange count
    pub exchange_count: i32,
    pub exchange_remaining: i32,

    // Resolution info
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refund_amount: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seller_amount: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commission_amount: Option<i64>,

    pub created_at: String,
    pub updated_at: String,
}

/// Dispute list query
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DisputeListQuery {
    #[serde(default = "default_page")]
    pub page: i64,

    #[serde(default = "default_per_page")]
    pub per_page: i64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_id: Option<String>,
}

/// Dispute list response
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DisputeListResponse {
    pub disputes: Vec<DisputeInfoResponse>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

/// Process auto-escalate response
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProcessAutoEscalateResponse {
    pub total_processed: i32,
    pub escalated_count: i32,
    pub auto_resolved_count: i32,
    pub failed_count: i32,
    pub escalated_ids: Vec<String>,
    pub resolved_ids: Vec<String>,
    pub errors: Vec<String>,
}

// ============================================================================
// LEGACY DTOs (for backward compatibility)
// ============================================================================

/// Manual debit request (alias for AdminDebitRequest)
pub type ManualDebitRequest = AdminDebitRequest;

/// Freeze wallet request (alias for AdminFreezeRequest)
pub type FreezeWalletRequest = AdminFreezeRequest;

/// Unfreeze wallet request (alias for AdminUnfreezeRequest)
pub type UnfreezeWalletRequest = AdminUnfreezeRequest;

/// Set shop commission request (alias for SetCommissionRateRequest)
pub type SetShopCommissionRequest = SetCommissionRateRequest;

/// Reject withdrawal request
#[derive(Debug, Deserialize, ToSchema)]
pub struct RejectWithdrawalRequest {
    pub rejection_reason: String,
}

/// Complete bank transfer request
#[derive(Debug, Deserialize, ToSchema)]
pub struct CompleteBankTransferRequest {
    pub bank_transfer_ref: String,
}

/// Admin log query
#[derive(Debug, Deserialize, ToSchema)]
pub struct AdminLogQuery {
    pub target_id: Option<String>,
    pub limit: Option<i64>,
}

/// Transaction history response
#[derive(Debug, Serialize)]
pub struct TransactionHistoryResponse {
    pub wallet_id: String,
    pub transactions: Vec<super::domain::Transaction>,
    pub count: i64,
}

/// Admin log response
#[derive(Debug, Serialize)]
pub struct AdminLogResponse {
    pub logs: Vec<super::domain::AdminOperationLog>,
    pub count: i64,
}

/// Process auto-release response
#[derive(Debug, Serialize, ToSchema)]
pub struct ProcessAutoReleaseResponse {
    pub total_processed: i32,
    pub released_count: i32,
    pub failed_count: i32,
    pub released_ids: Vec<String>,
    pub errors: Vec<String>,
}

// ============================================================================
// USDT DEPOSIT DTOs
// ============================================================================

/// Get USDT deposit address response
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UsdtDepositAddressResponse {
    /// Platform USDT address (TRC20)
    pub deposit_address: String,

    /// Network type
    pub network: String,

    /// Memo format for user to include
    pub memo_format: String,

    /// Example memo
    pub memo_example: String,

    /// Minimum deposit amount
    pub min_deposit: f64,

    /// Maximum deposit amount
    pub max_deposit: f64,

    /// Current exchange rate
    pub exchange_rate: f64,

    /// Required confirmations
    pub required_confirmations: i32,
}

/// USDT deposit status response
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UsdtDepositStatusResponse {
    pub deposit_id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub usdt_amount: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub vnd_amount: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub trust_amount: Option<i64>,

    pub network: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_hash: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirmations: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_confirmations: Option<i32>,

    pub status: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub credited_at: Option<String>,
}

/// List USDT deposits response (admin)
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UsdtDepositsListResponse {
    pub deposits: Vec<UsdtDepositItemResponse>,
    pub count: i64,
    pub total_usdt: f64,
    pub total_vnd: i64,
}

/// Single USDT deposit item
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UsdtDepositItemResponse {
    pub deposit_id: String,
    pub user_id: String,
    pub wallet_id: String,

    pub usdt_amount: f64,
    pub vnd_amount: i64,
    pub trust_amount: i64,

    pub network: String,
    pub sender_address: String,
    pub transaction_hash: String,
    pub block_number: i64,

    pub exchange_rate: f64,

    pub confirmations: i32,
    pub required_confirmations: i32,

    pub status: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub credited_at: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_reason: Option<String>,

    pub created_at: String,
    pub updated_at: String,
}

/// Manual credit USDT deposit request (admin)
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ManualCreditUsdtRequest {
    /// Deposit ID to credit
    #[validate(length(min = 1))]
    pub deposit_id: String,

    /// Reason for manual credit
    #[validate(length(min = 10, max = 500))]
    pub reason: String,

    /// Optional note
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(max = 1000))]
    pub note: Option<String>,
}

/// Get exchange rate response
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeRateResponse {
    pub usdt_to_vnd: f64,
    pub updated_at: String,
}

/// Deposit history query parameters
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DepositHistoryQuery {
    #[serde(default = "default_page")]
    pub page: i64,

    #[serde(default = "default_per_page")]
    pub per_page: i64,
}

/// Admin deposit history query parameters
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminDepositHistoryQuery {
    #[serde(default = "default_page")]
    pub page: i64,

    #[serde(default = "default_admin_per_page")]
    pub per_page: i64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
}

/// Admin manual deposit request
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminManualDepositRequest {
    /// Target user ID
    #[validate(length(min = 1))]
    pub target_user_id: String,

    /// Trust amount to deposit (min 1, max 1,000,000)
    #[validate(range(
        min = 1,
        max = 1000000,
        message = "Amount must be between 1 and 1,000,000 Trust"
    ))]
    pub trust_amount: i64,

    /// Reason for manual deposit (min 10 chars)
    #[validate(length(min = 10, max = 500))]
    pub reason: String,

    /// Optional note
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(max = 1000))]
    pub note: Option<String>,
}

fn default_page() -> i64 {
    1
}
fn default_per_page() -> i64 {
    20
}
fn default_admin_per_page() -> i64 {
    50
}

// ============================================================================
// DASHBOARD STATS DTOs
// ============================================================================

/// Dashboard statistics response for admin
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DashboardStatsResponse {
    // Platform wallet overview
    pub platform_balance: i64,
    pub platform_escrow_held: i64,

    // Today's activity
    pub today_transaction_count: i64,
    pub today_transaction_volume: i64,
    pub today_commission: Option<i64>,

    // Pending actions
    pub pending_withdrawals: i64,
    pub pending_withdrawal_amount: i64,

    // Escrow stats
    pub active_escrows: i64,

    // USDT stats
    pub usdt_deposits_today: i64,
    pub usdt_pending: i64,
    pub usdt_total_trust: i64,

    // System health
    pub system_status: String,
    pub last_updated: String,
}

/// Reconciliation discrepancy detail
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationDiscrepancy {
    pub wallet_id: String,
    pub discrepancy_type: String,
    pub expected: i64,
    pub actual: i64,
    pub details: String,
}

/// Reconciliation response
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationResponse {
    pub reconciliation_id: String,
    pub wallets_checked: i64,
    pub discrepancy_count: i64,
    pub discrepancies: Vec<ReconciliationDiscrepancy>,
    pub duration_ms: i64,
    pub status: String,
    pub performed_at: String,
}

/// USDT deposits summary (already exists, adding for reference)
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UsdtDepositsSummary {
    pub total_deposits: i64,
    pub pending_deposits: i64,
    pub confirmed_deposits: i64,
    pub total_usdt_amount: f64,
    pub total_trust_amount: i64,
}

// ============================================================================
// VALIDATION FUNCTIONS
// ============================================================================
