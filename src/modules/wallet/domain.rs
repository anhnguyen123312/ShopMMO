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

    // === Admin Debt System (V2) ===
    /// Admin-imposed debt (auto-repaid from escrow releases and deposits)
    #[serde(default)]
    pub admin_debt: i64,

    /// Reason for admin debt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_debt_reason: Option<String>,

    /// Admin who created the debt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_debt_created_by: Option<String>,

    /// When admin debt was created
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_debt_created_at: Option<BsonDateTime>,

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
    DepositUsdt,

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
    PartialRefund,
    CancelledBySeller,
    Extended,
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
// DISPUTE LOCK MODEL
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisputeLock {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub lock_id: String,
    pub wallet_id: String,
    pub user_id: String,

    pub amount: i64,
    pub reason: String,
    pub case_reference: String,

    pub admin_id: String,

    pub status: DisputeLockStatus,

    pub created_at: BsonDateTime,
    pub updated_at: BsonDateTime,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<BsonDateTime>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_by: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DisputeLockStatus {
    Active,
    Resolved,
    Released,
}

impl DisputeLock {
    pub fn new(
        lock_id: String,
        wallet_id: String,
        user_id: String,
        amount: i64,
        reason: String,
        case_reference: String,
        admin_id: String,
    ) -> Self {
        let now = BsonDateTime::now();
        Self {
            id: None,
            lock_id,
            wallet_id,
            user_id,
            amount,
            reason,
            case_reference,
            admin_id,
            status: DisputeLockStatus::Active,
            created_at: now,
            updated_at: now,
            resolved_at: None,
            resolved_by: None,
            resolution_note: None,
        }
    }

    pub fn resolve(&mut self, resolved_by: String, note: String) {
        self.status = DisputeLockStatus::Resolved;
        self.resolved_at = Some(BsonDateTime::now());
        self.resolved_by = Some(resolved_by);
        self.resolution_note = Some(note);
        self.updated_at = BsonDateTime::now();
    }
}

// ============================================================================
// ADMIN DEBT TRANSACTION MODEL
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminDebtTransaction {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub debt_id: String,
    pub wallet_id: String,
    pub user_id: String,

    pub original_amount: i64,
    pub actual_deducted: i64,
    pub debt_amount: i64,

    pub reason: String,
    pub admin_id: String,

    pub total_repaid: i64,
    pub remaining_debt: i64,

    #[serde(default)]
    pub repayment_history: Vec<DebtRepayment>,

    pub status: AdminDebtStatus,

    pub created_at: BsonDateTime,
    pub updated_at: BsonDateTime,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cleared_at: Option<BsonDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtRepayment {
    pub order_id: Option<String>,
    pub deposit_id: Option<String>,
    pub amount: i64,
    pub source: DebtRepaymentSource,
    pub repaid_at: BsonDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DebtRepaymentSource {
    EscrowRelease,
    Deposit,
    ManualRepayment,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AdminDebtStatus {
    Pending,
    Partial,
    Cleared,
}

impl AdminDebtTransaction {
    pub fn new(
        debt_id: String,
        wallet_id: String,
        user_id: String,
        original_amount: i64,
        actual_deducted: i64,
        debt_amount: i64,
        reason: String,
        admin_id: String,
    ) -> Self {
        let now = BsonDateTime::now();
        Self {
            id: None,
            debt_id,
            wallet_id,
            user_id,
            original_amount,
            actual_deducted,
            debt_amount,
            reason,
            admin_id,
            total_repaid: 0,
            remaining_debt: debt_amount,
            repayment_history: vec![],
            status: AdminDebtStatus::Pending,
            created_at: now,
            updated_at: now,
            cleared_at: None,
        }
    }

    pub fn add_repayment(
        &mut self,
        amount: i64,
        source: DebtRepaymentSource,
        order_id: Option<String>,
        deposit_id: Option<String>,
    ) {
        let repayment = DebtRepayment {
            order_id,
            deposit_id,
            amount,
            source,
            repaid_at: BsonDateTime::now(),
        };
        self.repayment_history.push(repayment);
        self.total_repaid += amount;
        self.remaining_debt -= amount;
        self.updated_at = BsonDateTime::now();

        if self.remaining_debt <= 0 {
            self.status = AdminDebtStatus::Cleared;
            self.cleared_at = Some(BsonDateTime::now());
        } else {
            self.status = AdminDebtStatus::Partial;
        }
    }
}

// ============================================================================
// HELPER IMPLEMENTATIONS
// ============================================================================

impl Wallet {
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
            admin_debt: 0,
            admin_debt_reason: None,
            admin_debt_created_by: None,
            admin_debt_created_at: None,
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

    pub fn can_withdraw(&self, amount: i64) -> bool {
        self.is_active() && self.available_trust >= amount && self.admin_debt == 0
    }

    pub fn has_admin_debt(&self) -> bool {
        self.admin_debt > 0
    }

    pub fn total_debt(&self) -> i64 {
        self.commission_debt + self.admin_debt
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
    pub fn add_status_change(
        &mut self,
        to_status: TransactionStatus,
        changed_by: String,
        reason: Option<String>,
    ) {
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

// ============================================================================
// USDT DEPOSIT MODEL - TRC20 Network Support
// ============================================================================

/// USDT Deposit document for TRC20 network deposits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsdtDeposit {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// Unique deposit ID: USDT-{ULID}
    pub deposit_id: String,

    pub wallet_id: String,
    pub user_id: String,

    // === USDT Details ===
    /// USDT amount (float precision for crypto)
    pub usdt_amount: f64,

    /// Blockchain network
    pub network: UsdtNetwork,

    /// Sender's blockchain address
    pub sender_address: String,

    /// Transaction hash on blockchain
    pub transaction_hash: String,

    /// Block number for confirmation tracking
    pub block_number: i64,

    // === Conversion ===
    /// VND amount (usdt_amount * exchange_rate)
    pub vnd_amount: i64,

    /// Trust amount (vnd_amount / 1000)
    pub trust_amount: i64,

    /// Exchange rate used (USDT to VND)
    pub exchange_rate: f64,

    // === Status & Confirmations ===
    pub status: UsdtDepositStatus,

    /// Current confirmation count
    #[serde(default)]
    pub confirmations: i32,

    /// Required confirmations (usually 20 for TRC20)
    #[serde(default)]
    pub required_confirmations: i32,

    // === Processing ===
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credited_at: Option<BsonDateTime>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_reason: Option<String>,

    // === Memo/Reference ===
    /// User-provided memo for tracking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<String>,

    // === Timestamps ===
    pub created_at: BsonDateTime,
    pub updated_at: BsonDateTime,
}

/// Supported USDT networks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UsdtNetwork {
    /// TRC20 (Tron network) - Low fees, fast
    Trc20,
    /// BEP20 (BSC) - Future support
    Bec20,
    /// ERC20 (Ethereum) - Future support
    Erc20,
}

/// USDT deposit status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UsdtDepositStatus {
    /// Deposit detected, waiting for confirmations
    Pending,
    /// Gathering confirmations
    Confirming,
    /// Required confirmations reached, ready to credit
    Confirmed,
    /// Successfully credited to wallet
    Credited,
    /// Deposit failed (invalid amount, network error, etc.)
    Failed,
    /// Ignored (no memo or invalid memo)
    Ignored,
}

impl UsdtDeposit {
    /// Create new USDT deposit
    pub fn new(
        deposit_id: String,
        wallet_id: String,
        user_id: String,
        usdt_amount: f64,
        network: UsdtNetwork,
        sender_address: String,
        transaction_hash: String,
        block_number: i64,
        exchange_rate: f64,
    ) -> Self {
        let vnd_amount = (usdt_amount * exchange_rate) as i64;
        let trust_amount = vnd_amount / 1000;

        let now = BsonDateTime::now();
        Self {
            id: None,
            deposit_id,
            wallet_id,
            user_id,
            usdt_amount,
            network,
            sender_address,
            transaction_hash,
            block_number,
            vnd_amount,
            trust_amount,
            exchange_rate,
            status: UsdtDepositStatus::Pending,
            confirmations: 0,
            required_confirmations: 20, // TRC20 standard
            credited_at: None,
            failed_reason: None,
            memo: None,
            transaction_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if deposit has enough confirmations
    pub fn has_enough_confirmations(&self) -> bool {
        self.confirmations >= self.required_confirmations
    }

    /// Check if deposit can be credited
    pub fn can_credit(&self) -> bool {
        self.status == UsdtDepositStatus::Confirmed
            || (self.status == UsdtDepositStatus::Confirming && self.has_enough_confirmations())
    }
}

// ============================================================================
// DISPUTE CASE MODEL - Enhanced Dispute System V2
// ============================================================================

/// Dispute case document - Full dispute tracking with multi-exchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisputeCase {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// Unique dispute ID: DSP-{ULID}
    pub dispute_id: String,

    /// Reference to escrow
    pub escrow_id: String,
    pub escrow_status_at_dispute: EscrowStatus,

    /// Order info
    pub order_id: String,
    pub buyer_id: String,
    pub seller_id: String,

    /// Amount in dispute
    pub amount: i64,

    // === Type & Status ===
    /// Dispute type
    pub dispute_type: DisputeType,

    /// Current status
    pub status: DisputeStatus,

    // === Buyer Initial Request ===
    pub buyer_reason: String,

    /// Evidence images (max 5 initially)
    #[serde(default)]
    pub buyer_evidence_images: Vec<String>,

    /// Buyer updates (multi-exchange)
    #[serde(default)]
    pub buyer_updates: Vec<DisputeUpdate>,

    /// When buyer submitted dispute
    pub buyer_created_at: BsonDateTime,

    /// Last time buyer responded
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buyer_responded_at: Option<BsonDateTime>,

    // === Seller Response ===
    /// Seller's action type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seller_action: Option<SellerAction>,

    /// Seller's response message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seller_response: Option<String>,

    /// Seller's evidence images
    #[serde(default)]
    pub seller_evidence_images: Vec<String>,

    /// Seller updates (multi-exchange)
    #[serde(default)]
    pub seller_updates: Vec<DisputeUpdate>,

    /// When seller responded
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seller_responded_at: Option<BsonDateTime>,

    // === Partial Offer (for PARTIAL_ACCEPT) ===
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seller_offer_amount: Option<i64>,

    // === Replacement Items (for REPLACEMENT) ===
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seller_replacement_items: Option<String>,

    // === Deadlines ===
    /// Seller must respond within 48 hours (2 days)
    pub seller_deadline: BsonDateTime,

    /// Buyer must respond within 24 hours after seller
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buyer_deadline: Option<BsonDateTime>,

    // === Escalation ===
    /// Total exchanges (messages from both sides)
    #[serde(default)]
    pub exchange_count: i32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub escalated_at: Option<BsonDateTime>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub escalated_by: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub escalate_reason: Option<String>,

    // === Extension ===
    /// Admin extended deadline
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extended_at: Option<BsonDateTime>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub extended_by: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub extension_days: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_deadline: Option<BsonDateTime>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub extension_reason: Option<String>,

    // === Resolution ===
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<BsonDateTime>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_by: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,

    /// Final refund amount
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refund_amount: Option<i64>,

    /// Final amount to seller (after commission)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seller_amount: Option<i64>,

    /// Commission deducted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commission_amount: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub refund_processed_at: Option<BsonDateTime>,

    // === Admin Review ===
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_review_started_at: Option<BsonDateTime>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_decision: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_note: Option<String>,

    // === Timestamps ===
    pub created_at: BsonDateTime,
    pub updated_at: BsonDateTime,
}

/// Dispute type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DisputeType {
    /// Buyer requests refund
    RefundRequest,
    /// Seller requests cancellation (V1 only, removed in V2)
    #[serde(rename = "seller_cancel")]
    SellerCancel,
    /// Other dispute types
    Other,
}

/// Dispute status (V2 - enhanced flow)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DisputeStatus {
    /// Buyer created dispute, waiting for seller
    Pending,
    /// Seller responded
    SellerResponded,
    /// Buyer responded to seller
    BuyerResponded,
    /// Escalated to admin
    Escalated,
    /// Admin reviewing
    AdminReview,
    /// Resolved by agreement
    Resolved,
    /// Full refund to buyer
    Refunded,
    /// Partial refund
    PartialRefund,
    /// Rejected, money to seller
    Rejected,
    /// Closed without resolution
    Closed,
    /// Extended by admin
    Extended,
}

/// Seller action types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SellerAction {
    /// Accept full refund
    Accept,
    /// Accept partial refund with offer
    PartialAccept,
    /// Reject dispute
    Reject,
    /// Offer replacement items
    Replacement,
}

/// Dispute update (multi-exchange)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisputeUpdate {
    /// Update message
    pub message: String,

    /// Additional evidence images (max 3 per update)
    #[serde(default)]
    pub images: Vec<String>,

    /// Who sent this update
    pub sent_by: String,

    /// When sent
    pub sent_at: BsonDateTime,
}

/// Dispute reason codes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DisputeReason {
    WrongItem,
    NotWorking,
    Duplicate,
    MissingItems,
    QualityIssue,
    PartialWorking,
    Other,
}

impl DisputeCase {
    /// Create new dispute from buyer
    pub fn new_buyer_dispute(
        dispute_id: String,
        escrow_id: String,
        escrow_status: EscrowStatus,
        order_id: String,
        buyer_id: String,
        seller_id: String,
        amount: i64,
        reason: String,
        evidence_images: Vec<String>,
    ) -> Self {
        let now = BsonDateTime::now();
        let seller_deadline = BsonDateTime::from_millis(
            now.timestamp_millis() + (48 * 60 * 60 * 1000), // 48 hours
        );

        Self {
            id: None,
            dispute_id,
            escrow_id,
            escrow_status_at_dispute: escrow_status,
            order_id,
            buyer_id,
            seller_id,
            amount,
            dispute_type: DisputeType::RefundRequest,
            status: DisputeStatus::Pending,
            buyer_reason: reason,
            buyer_evidence_images: evidence_images,
            buyer_updates: vec![],
            buyer_created_at: now,
            buyer_responded_at: None,
            seller_action: None,
            seller_response: None,
            seller_evidence_images: vec![],
            seller_updates: vec![],
            seller_responded_at: None,
            seller_offer_amount: None,
            seller_replacement_items: None,
            seller_deadline,
            buyer_deadline: None,
            exchange_count: 0,
            escalated_at: None,
            escalated_by: None,
            escalate_reason: None,
            extended_at: None,
            extended_by: None,
            extension_days: None,
            new_deadline: None,
            extension_reason: None,
            resolved_at: None,
            resolved_by: None,
            resolution: None,
            refund_amount: None,
            seller_amount: None,
            commission_amount: None,
            refund_processed_at: None,
            admin_review_started_at: None,
            admin_decision: None,
            admin_note: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if seller deadline has passed
    pub fn is_seller_deadline_passed(&self) -> bool {
        BsonDateTime::now().timestamp_millis() > self.seller_deadline.timestamp_millis()
    }

    /// Check if buyer deadline has passed
    pub fn is_buyer_deadline_passed(&self) -> bool {
        if let Some(deadline) = self.buyer_deadline {
            BsonDateTime::now().timestamp_millis() > deadline.timestamp_millis()
        } else {
            false
        }
    }

    /// Check if max exchanges reached (3 rounds = 6 messages)
    pub fn is_max_exchanges_reached(&self) -> bool {
        self.exchange_count >= 6
    }

    /// Check if dispute can be auto-escalated (seller no response)
    pub fn should_auto_escalate_seller(&self) -> bool {
        self.status == DisputeStatus::Pending
            && self.seller_responded_at.is_none()
            && self.is_seller_deadline_passed()
    }

    /// Check if dispute can be auto-escalated (buyer no response)
    pub fn should_auto_escalate_buyer(&self) -> bool {
        if let Some(_deadline) = self.buyer_deadline {
            (self.status == DisputeStatus::SellerResponded
                || self.status == DisputeStatus::BuyerResponded)
                && self.is_buyer_deadline_passed()
        } else {
            false
        }
    }

    /// Check if dispute can be auto-resolved (buyer no response, seller accepted)
    pub fn can_auto_resolve(&self) -> bool {
        if let Some(_deadline) = self.buyer_deadline {
            if let Some(action) = &self.seller_action {
                return matches!(action, SellerAction::Accept | SellerAction::PartialAccept)
                    && self.is_buyer_deadline_passed();
            }
        }
        false
    }

    /// Add buyer update
    pub fn add_buyer_update(&mut self, message: String, images: Vec<String>) {
        let update = DisputeUpdate {
            message,
            images,
            sent_by: self.buyer_id.clone(),
            sent_at: BsonDateTime::now(),
        };
        self.buyer_updates.push(update);
        self.buyer_responded_at = Some(BsonDateTime::now());
        self.exchange_count += 1;
        self.updated_at = BsonDateTime::now();
    }

    /// Add seller update
    pub fn add_seller_update(&mut self, message: String, images: Vec<String>) {
        let update = DisputeUpdate {
            message,
            images,
            sent_by: self.seller_id.clone(),
            sent_at: BsonDateTime::now(),
        };
        self.seller_updates.push(update);
        self.seller_responded_at = Some(BsonDateTime::now());
        self.exchange_count += 1;
        self.updated_at = BsonDateTime::now();
    }

    /// Set seller action and update status
    pub fn set_seller_action(&mut self, action: SellerAction) {
        self.seller_action = Some(action.clone());
        self.seller_responded_at = Some(BsonDateTime::now());

        // Update status based on action
        self.status = match action {
            SellerAction::Accept => {
                // Auto-resolve, no need for buyer response
                DisputeStatus::Resolved
            }
            _ => {
                // Buyer needs to respond
                DisputeStatus::SellerResponded
            }
        };

        self.updated_at = BsonDateTime::now();
    }

    /// Extend deadline (admin action)
    pub fn extend_deadline(&mut self, days: i32, admin_id: String, reason: String) {
        let now = BsonDateTime::now();
        let current_deadline = if let Some(existing) = self.new_deadline {
            existing
        } else {
            self.seller_deadline
        };

        let new_deadline = BsonDateTime::from_millis(
            current_deadline.timestamp_millis() + (days as i64 * 24 * 60 * 60 * 1000),
        );

        self.extended_at = Some(now);
        self.extended_by = Some(admin_id);
        self.extension_days = Some(days);
        self.new_deadline = Some(new_deadline);
        self.extension_reason = Some(reason);
        self.seller_deadline = new_deadline;
        self.status = DisputeStatus::Extended;
        self.updated_at = now;
    }

    /// Escalate to admin
    pub fn escalate(&mut self, escalated_by: String, reason: String) {
        self.escalated_at = Some(BsonDateTime::now());
        self.escalated_by = Some(escalated_by.clone());
        self.escalate_reason = Some(reason);
        self.status = DisputeStatus::Escalated;
        self.updated_at = BsonDateTime::now();
    }

    /// Resolve as refunded
    pub fn resolve_refunded(&mut self, refund_amount: i64, resolved_by: String) {
        let now = BsonDateTime::now();
        self.status = if refund_amount == self.amount {
            DisputeStatus::Refunded
        } else {
            DisputeStatus::PartialRefund
        };
        self.resolved_at = Some(now);
        self.resolved_by = Some(resolved_by);
        self.refund_amount = Some(refund_amount);
        self.resolution = Some(if refund_amount == self.amount {
            "FULL_REFUND".to_string()
        } else {
            format!(
                "PARTIAL_REFUND_{}%",
                (refund_amount as f64 / self.amount as f64 * 100.0) as i32
            )
        });
        self.updated_at = now;
    }

    /// Resolve as rejected (money to seller)
    pub fn resolve_rejected(&mut self, resolved_by: String, reason: String) {
        let now = BsonDateTime::now();
        self.status = DisputeStatus::Rejected;
        self.resolved_at = Some(now);
        self.resolved_by = Some(resolved_by);
        self.resolution = Some("REJECTED_SELLER_FAVORED".to_string());
        self.admin_note = Some(reason);
        self.updated_at = now;
    }
}
