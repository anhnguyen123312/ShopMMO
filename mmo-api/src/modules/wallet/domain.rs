//! Wallet V3 Domain Models
//!
//! Trust Currency System - MongoDB document structures
//! Based on TaphoaMMO Trust Wallet V3 Design

use bson::{oid::ObjectId, DateTime as BsonDateTime};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ============================================================================
// WALLET MODEL - Unified for all user types
// ============================================================================

/// Main Wallet document
/// Supports USER (Buyer), SELLER (Vendor), and PLATFORM wallet types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// Unique wallet identifier: WLT-{ULID}
    pub wallet_id: String,

    /// User ID or "PLATFORM" for system wallet
    pub user_id: String,

    /// Wallet type
    pub wallet_type: WalletType,

    // === Balance States ===
    /// Available for immediate use
    pub available_trust: i64,

    /// Locked for pending withdrawal
    pub withdrawal_locked: i64,

    /// Locked due to dispute/investigation
    pub dispute_locked: i64,

    /// Computed total (available + withdrawal_locked + dispute_locked)
    pub total_trust: i64,

    // === Running Totals (for validation) ===
    /// Total deposited from beginning
    pub lifetime_deposited: i64,

    /// Total withdrawn from beginning
    pub lifetime_withdrawn: i64,

    /// Total spent on purchases
    pub lifetime_spent: i64,

    /// Total received from sales/refunds
    pub lifetime_received: i64,

    // === Seller-specific ===
    /// Custom commission rate override (default from config if None)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commission_rate: Option<f64>,

    /// Accumulated commission debt (paid on withdrawal)
    #[serde(default)]
    pub commission_debt: i64,

    // === Monthly Snapshot Reference ===
    /// Last snapshot month: "2026-01"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_snapshot_month: Option<String>,

    /// Balance at end of that month
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_snapshot_balance: Option<i64>,

    /// Whether snapshot was verified
    #[serde(default)]
    pub last_snapshot_verified: bool,

    // === Status & Metadata ===
    pub status: WalletStatus,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeze_reason: Option<String>,

    pub created_at: BsonDateTime,
    pub updated_at: BsonDateTime,
}

/// Wallet type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WalletType {
    /// Regular user/buyer wallet
    User,
    /// Seller/vendor wallet with commission tracking
    Seller,
    /// Platform system wallet (holds escrow + commission)
    Platform,
}

/// Wallet status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WalletStatus {
    /// Normal operation
    Active,
    /// Temporarily suspended, can be unlocked
    Suspended,
    /// Frozen, requires admin intervention
    Frozen,
}

// ============================================================================
// TRANSACTION MODEL - Immutable ledger with Exness-style status tracking
// ============================================================================

/// Transaction document - Immutable ledger entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// Unique transaction ID: TXN-{ULID}
    pub tx_id: String,

    /// Reference to wallet
    pub wallet_id: String,

    /// Owner of wallet
    pub user_id: String,

    // === Type & Direction ===
    pub tx_type: TransactionType,
    pub direction: Direction,

    // === Amounts ===
    /// Trust amount (always positive)
    pub amount: i64,

    /// VND equivalent if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vnd_amount: Option<i64>,

    /// Fee/commission deducted
    #[serde(default)]
    pub fee_amount: i64,

    // === Balance Tracking ===
    /// Wallet balance before this transaction
    pub balance_before: i64,

    /// Wallet balance after this transaction
    pub balance_after: i64,

    /// Which balance field was affected
    pub balance_type: BalanceType,

    // === Running Totals After Transaction ===
    pub running_deposited: i64,
    pub running_withdrawn: i64,

    // === Status (Exness-style) ===
    pub status: TransactionStatus,

    /// Status change history
    #[serde(default)]
    pub status_history: Vec<StatusChange>,

    // === References ===
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_type: Option<ReferenceType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,

    /// External reference (bank, gateway)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_ref: Option<String>,

    // === Admin/System ===
    /// User ID, "SYSTEM", or admin ID
    pub initiated_by: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_note: Option<String>,

    // === Timestamps ===
    pub created_at: BsonDateTime,
    pub updated_at: BsonDateTime,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<BsonDateTime>,
}

/// Transaction type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionType {
    // === Deposit ===
    DepositPending,
    DepositVndReceived,
    DepositTrustCredited,
    DepositManual,

    // === Withdrawal ===
    WithdrawalRequest,
    WithdrawalValidating,
    WithdrawalApproved,
    WithdrawalProcessing,
    WithdrawalCompleted,
    WithdrawalRejected,
    WithdrawalCancelled,

    // === Purchase/Order ===
    Purchase,
    PurchaseDebit,
    PurchaseEscrow,
    EscrowHold,
    EscrowRelease,
    EscrowReceive,

    // === Refund ===
    RefundEscrow,
    RefundSeller,
    DisputeRefund,

    // === Commission ===
    CommissionAccrue,
    CommissionDeduct,
    CommissionCollected,

    // === Admin Operations ===
    AdminCredit,
    AdminDebit,
    DebitManual,
    AdminAdjustment,
    AdminFreeze,
    AdminUnfreeze,
}

/// Transaction direction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Direction {
    /// Money in (+)
    Credit,
    /// Money out (-)
    Debit,
}

/// Balance type affected
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BalanceType {
    Available,
    WithdrawalLocked,
    DisputeLocked,
}

/// Transaction status (Exness-style)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionStatus {
    Pending,
    Processing,
    Validating,
    AwaitingReview,
    Approved,
    Rejected,
    Completed,
    Failed,
    Cancelled,
    Reversed,
}

/// Status change history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusChange {
    pub from_status: TransactionStatus,
    pub to_status: TransactionStatus,
    pub changed_at: BsonDateTime,
    /// User ID, admin ID, or "SYSTEM"
    pub changed_by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Reference type for linking transactions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReferenceType {
    Order,
    Withdrawal,
    Deposit,
    Escrow,
    Commission,
    AdminOperation,
}

// ============================================================================
// WITHDRAWAL REQUEST MODEL
// ============================================================================

/// Withdrawal request document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawalRequest {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// Unique request ID: WD-{ULID}
    pub request_id: String,

    pub wallet_id: String,
    pub user_id: String,

    // === Amounts ===
    /// Trust amount to withdraw
    pub trust_amount: i64,

    /// Commission to be deducted
    pub commission_deduct: i64,

    /// Net trust after commission
    pub net_trust: i64,

    /// VND amount (net_trust * 1000)
    pub vnd_amount: i64,

    // === Bank Info ===
    pub bank_code: String,
    pub bank_name: String,
    pub account_number: String,
    pub account_name: String,

    // === Status ===
    pub status: WithdrawalStatus,

    #[serde(default)]
    pub status_history: Vec<WithdrawalStatusChange>,

    // === Validation Results ===
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_result: Option<ValidationResult>,

    #[serde(default)]
    pub validation_errors: Vec<ValidationError>,

    // === Processing ===
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved_by: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved_at: Option<BsonDateTime>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bank_transfer_ref: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bank_transfer_at: Option<BsonDateTime>,

    // === Timestamps ===
    pub created_at: BsonDateTime,
    pub updated_at: BsonDateTime,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<BsonDateTime>,

    /// Auto-cancel if not processed by this time
    pub expires_at: BsonDateTime,
}

/// Withdrawal status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WithdrawalStatus {
    Pending,
    Validating,
    ValidationFailed,
    AwaitingApproval,
    Approved,
    Processing,
    Completed,
    Rejected,
    Cancelled,
    Expired,
    Hold,
}

/// Withdrawal status change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawalStatusChange {
    pub from_status: WithdrawalStatus,
    pub to_status: WithdrawalStatus,
    pub changed_at: BsonDateTime,
    pub changed_by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Validation result for withdrawal
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ValidationResult {
    pub balance_check: CheckResult,
    pub flow_check: CheckResult,
    pub fraud_check: CheckResult,
    pub limit_check: CheckResult,
    pub overall_passed: bool,
    pub risk_score: f64,
}

/// Individual check result
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CheckResult {
    pub passed: bool,
    pub details: String,
    pub severity: Severity,
}

/// Validation error
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ValidationError {
    pub error_type: String,
    pub message: String,
    pub severity: Severity,
}

/// Severity level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

// ============================================================================
// MONTHLY SNAPSHOT MODEL
// ============================================================================

/// Monthly snapshot for performance optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlySnapshot {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// Unique snapshot ID: SNAP-{wallet_id}-{YYYY-MM}
    pub snapshot_id: String,

    pub wallet_id: String,
    pub user_id: String,

    /// Month: "2026-01"
    pub month: String,

    // === Balances at End of Month ===
    /// Calculated from transactions
    pub closing_balance: i64,

    /// From wallet at snapshot time
    pub actual_balance: i64,

    // === Running Totals at End of Month ===
    pub total_deposited: i64,
    pub total_withdrawn: i64,
    pub total_spent: i64,
    pub total_received: i64,

    // === Verification ===
    /// closing_balance - actual_balance
    pub discrepancy: i64,

    pub status: SnapshotStatus,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified_at: Option<BsonDateTime>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified_by: Option<String>,

    // === Transaction Summary ===
    pub tx_count: i64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_tx_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_tx_id: Option<String>,

    pub created_at: BsonDateTime,
}

/// Snapshot status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SnapshotStatus {
    Pending,
    Verified,
    Discrepancy,
    Critical,
    ManualOverride,
}

// ============================================================================
// ESCROW HOLD MODEL
// ============================================================================

/// Escrow hold document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscrowHold {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// Unique escrow ID: ESC-{ULID}
    pub escrow_id: String,

    pub order_id: String,
    pub buyer_id: String,
    pub seller_id: String,

    /// Amount held in Platform wallet
    pub amount: i64,

    pub status: EscrowStatus,

    /// Auto-release after 3 days
    pub release_at: BsonDateTime,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub released_at: Option<BsonDateTime>,

    /// If buyer confirmed early
    #[serde(default)]
    pub early_release: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub early_release_by: Option<String>,

    /// Commission amount calculated on release
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commission_amount: Option<i64>,

    pub created_at: BsonDateTime,
    pub updated_at: BsonDateTime,
}

/// Escrow status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EscrowStatus {
    Holding,
    Released,
    Refunded,
    Disputed,
    CancelledBySeller,
}

/// Release type for escrow
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReleaseType {
    Auto,
    EarlyRelease,
    DisputeRelease,
    DisputeRefund,
    AdminForce,
}

// ============================================================================
// ADMIN OPERATION LOG
// ============================================================================

/// Admin operation audit log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminOperationLog {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// Unique log ID: ALOG-{ULID}
    pub log_id: String,

    // === Admin Info ===
    pub admin_id: String,
    pub admin_email: String,
    pub admin_role: String,

    // === Operation ===
    pub operation: AdminOperation,
    pub target_type: TargetType,
    pub target_id: String,

    // === Before/After States ===
    pub before_state: serde_json::Value,
    pub after_state: serde_json::Value,

    // === Details ===
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<i64>,

    pub reason: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,

    // === Related ===
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<String>,

    // === Metadata ===
    pub ip_address: String,
    pub user_agent: String,

    pub created_at: BsonDateTime,
}

/// Admin operation type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AdminOperation {
    ManualDeposit,
    ManualDebit,
    BalanceAdjustment,
    CommissionOverride,
    FreezeWallet,
    UnfreezeWallet,
    SetCommission,
    ApproveWithdrawal,
    RejectWithdrawal,
    CompleteBankTransfer,
    ForceReleaseEscrow,
    DisputeRefund,
    DisputeRelease,
}

/// Target type for admin operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TargetType {
    Wallet,
    User,
    Shop,
    Withdrawal,
    Escrow,
    Order,
}

// ============================================================================
// SHOP COMMISSION CONFIG
// ============================================================================

/// Shop commission configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopCommissionConfig {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub shop_id: String,

    /// Commission rate (0.01 to 0.20 = 1% to 20%)
    pub rate: f64,

    pub effective_from: BsonDateTime,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_to: Option<BsonDateTime>,

    pub created_by: String,
    pub reason: String,

    pub created_at: BsonDateTime,
}

// ============================================================================
// DEPOSIT REQUEST MODEL
// ============================================================================

/// Deposit request document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositRequest {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// Unique deposit ID: DEP-{ULID}
    pub deposit_id: String,

    pub wallet_id: String,
    pub user_id: String,

    /// VND amount requested
    pub vnd_amount: i64,

    /// Trust to be credited (vnd / 1000)
    pub trust_amount: i64,

    /// Payment gateway (VNPay, MoMo, etc.)
    pub payment_gateway: String,

    /// Payment URL from gateway
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_url: Option<String>,

    /// Gateway transaction reference
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_gateway_ref: Option<String>,

    pub status: DepositStatus,

    pub created_at: BsonDateTime,
    pub updated_at: BsonDateTime,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<BsonDateTime>,

    /// Payment URL expires after 15 minutes
    pub expires_at: BsonDateTime,
}

/// Deposit status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DepositStatus {
    Pending,
    Processing,
    Completed,
    Cancelled,
    Expired,
    Failed,
}

// ============================================================================
// HELPER IMPLEMENTATIONS
// ============================================================================

impl Wallet {
    /// Create new user wallet
    pub fn new_user(user_id: String, wallet_id: String) -> Self {
        let now = BsonDateTime::now();
        Self {
            id: None,
            wallet_id,
            user_id,
            wallet_type: WalletType::User,
            available_trust: 0,
            withdrawal_locked: 0,
            dispute_locked: 0,
            total_trust: 0,
            lifetime_deposited: 0,
            lifetime_withdrawn: 0,
            lifetime_spent: 0,
            lifetime_received: 0,
            commission_rate: None,
            commission_debt: 0,
            last_snapshot_month: None,
            last_snapshot_balance: None,
            last_snapshot_verified: false,
            status: WalletStatus::Active,
            freeze_reason: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create new seller wallet
    pub fn new_seller(user_id: String, wallet_id: String, commission_rate: Option<f64>) -> Self {
        let mut wallet = Self::new_user(user_id, wallet_id);
        wallet.wallet_type = WalletType::Seller;
        wallet.commission_rate = commission_rate;
        wallet
    }

    /// Create platform wallet
    pub fn new_platform(wallet_id: String) -> Self {
        let mut wallet = Self::new_user("PLATFORM".to_string(), wallet_id);
        wallet.wallet_type = WalletType::Platform;
        wallet
    }

    /// Validate balance invariant
    pub fn validate_balance_invariant(&self) -> bool {
        self.total_trust == (self.available_trust + self.withdrawal_locked + self.dispute_locked)
    }

    /// Check if wallet is active
    pub fn is_active(&self) -> bool {
        self.status == WalletStatus::Active
    }

    /// Check if wallet can withdraw
    pub fn can_withdraw(&self, amount: i64) -> bool {
        self.is_active() && self.available_trust >= amount
    }
}

impl Transaction {
    /// Create new transaction
    pub fn new(
        tx_id: String,
        wallet_id: String,
        user_id: String,
        tx_type: TransactionType,
        direction: Direction,
        amount: i64,
        balance_before: i64,
        balance_after: i64,
        balance_type: BalanceType,
        initiated_by: String,
    ) -> Self {
        let now = BsonDateTime::now();
        Self {
            id: None,
            tx_id,
            wallet_id,
            user_id,
            tx_type,
            direction,
            amount,
            vnd_amount: None,
            fee_amount: 0,
            balance_before,
            balance_after,
            balance_type,
            running_deposited: 0,
            running_withdrawn: 0,
            status: TransactionStatus::Pending,
            status_history: vec![],
            reference_type: None,
            reference_id: None,
            external_ref: None,
            initiated_by,
            admin_note: None,
            created_at: now,
            updated_at: now,
            completed_at: None,
        }
    }

    /// Add status change to history
    pub fn add_status_change(&mut self, to_status: TransactionStatus, changed_by: String, reason: Option<String>) {
        let change = StatusChange {
            from_status: self.status.clone(),
            to_status: to_status.clone(),
            changed_at: BsonDateTime::now(),
            changed_by,
            reason,
        };
        self.status_history.push(change);
        self.status = to_status;
        self.updated_at = BsonDateTime::now();
    }
}
