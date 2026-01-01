//! Wallet V3 DTOs
//!
//! Request and Response structures for all wallet endpoints
//! Based on TaphoaMMO Trust Wallet V3 Design

use serde::{Deserialize, Serialize};
use validator::Validate;
use utoipa::ToSchema;

use super::domain::{
    BalanceType, CheckResult, Direction, EscrowStatus, Severity, SnapshotStatus,
    TransactionStatus, TransactionType, ValidationResult, WalletStatus, WalletType,
    WithdrawalStatus,
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

    // Seller-specific
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commission_debt: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub commission_rate: Option<f64>,

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

    // Seller-specific
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commission_debt: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub commission_rate: Option<f64>,

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
    #[validate(range(min = 10000, max = 50000000, message = "Amount must be between 10,000 and 50,000,000 VND"))]
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
    #[validate(range(min = 1, max = 1000000, message = "Amount must be between 1 and 1,000,000 Trust"))]
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

// ============================================================================
// WITHDRAWAL DTOs
// ============================================================================

/// Withdrawal request
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawalRequest {
    /// Trust amount to withdraw
    #[validate(range(min = 10, max = 100000, message = "Amount must be between 10 and 100,000 Trust"))]
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

/// Admin manual debit request
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminDebitRequest {
    #[validate(length(min = 1))]
    pub user_id: String,

    #[serde(alias = "amount")]
    #[validate(range(min = 1))]
    pub trust_amount: i64,

    #[validate(length(min = 10, max = 500))]
    pub reason: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(max = 1000))]
    pub note: Option<String>,
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

/// Dispute request
#[derive(Debug, Deserialize, ToSchema)]
pub struct DisputeRequest {
    pub reason: String,
}

/// Resolve dispute request
#[derive(Debug, Deserialize, ToSchema)]
pub struct ResolveDisputeRequest {
    pub reason: String,
}

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

