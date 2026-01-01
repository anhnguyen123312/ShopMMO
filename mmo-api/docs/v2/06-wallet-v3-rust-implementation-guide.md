# Wallet V3 Rust Implementation Guide

**Created:** 2026-01-01
**Status:** In Progress
**Based on:** TaphoaMMO Trust Wallet V3 Design

---

## Implementation Status

### ✅ Completed

1. **Domain Models** ([mmo-api/src/modules/wallet/domain.rs](../../mmo-api/src/modules/wallet/domain.rs))
   - ✅ Wallet (User, Seller, Platform types)
   - ✅ Transaction (Exness-style status tracking)
   - ✅ WithdrawalRequest
   - ✅ MonthlySnapshot
   - ✅ EscrowHold
   - ✅ AdminOperationLog
   - ✅ ShopCommissionConfig
   - ✅ DepositRequest
   - ✅ All enums and status types

2. **DTOs** ([mmo-api/src/modules/wallet/dto.rs](../../mmo-api/src/modules/wallet/dto.rs))
   - ✅ Wallet balance & info responses
   - ✅ Deposit DTOs (auto + manual)
   - ✅ Withdrawal DTOs
   - ✅ Purchase DTOs
   - ✅ Escrow DTOs
   - ✅ Refund DTOs
   - ✅ Admin operation DTOs
   - ✅ Transaction history DTOs
   - ✅ Snapshot & reconciliation DTOs
   - ✅ Admin dashboard DTOs

### 🚧 To Implement

3. **Repository Layer** (Next)
4. **Service Layer** (Business Logic)
5. **HTTP Handlers**
6. **Routes Configuration**
7. **Background Jobs** (Cron)
8. **Tests**

---

## Repository Layer Structure

Create `mmo-api/src/modules/wallet/repository.rs`:

```rust
//! Wallet Repository Layer
//!
//! MongoDB database operations

use mongodb::{Database, Collection, bson::doc};
use bson::oid::ObjectId;

use super::domain::*;
use crate::core::error::DbError;

pub struct WalletRepository {
    wallets: Collection<Wallet>,
    transactions: Collection<Transaction>,
    withdrawal_requests: Collection<WithdrawalRequest>,
    deposit_requests: Collection<DepositRequest>,
    escrow_holds: Collection<EscrowHold>,
    monthly_snapshots: Collection<MonthlySnapshot>,
    admin_operation_logs: Collection<AdminOperationLog>,
    shop_commission_configs: Collection<ShopCommissionConfig>,
}

impl WalletRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            wallets: db.collection("wallets"),
            transactions: db.collection("wallet_transactions"),
            withdrawal_requests: db.collection("withdrawal_requests"),
            deposit_requests: db.collection("deposit_requests"),
            escrow_holds: db.collection("escrow_holds"),
            monthly_snapshots: db.collection("monthly_snapshots"),
            admin_operation_logs: db.collection("admin_operation_logs"),
            shop_commission_configs: db.collection("shop_commission_configs"),
        }
    }

    // === Wallet Operations ===

    pub async fn create_wallet(&self, wallet: Wallet) -> Result<Wallet, DbError>;

    pub async fn find_wallet_by_id(&self, wallet_id: &str) -> Result<Option<Wallet>, DbError>;

    pub async fn find_wallet_by_user_id(&self, user_id: &str) -> Result<Option<Wallet>, DbError>;

    pub async fn update_wallet(&self, wallet: &Wallet) -> Result<(), DbError>;

    pub async fn lock_wallet_for_update(&self, wallet_id: &str) -> Result<Wallet, DbError>;

    // === Transaction Operations ===

    pub async fn create_transaction(&self, tx: Transaction) -> Result<Transaction, DbError>;

    pub async fn find_transactions_by_wallet(
        &self,
        wallet_id: &str,
        start_date: Option<DateTime>,
        end_date: Option<DateTime>,
        limit: i64,
        skip: i64,
    ) -> Result<Vec<Transaction>, DbError>;

    pub async fn count_transactions_by_wallet(&self, wallet_id: &str) -> Result<i64, DbError>;

    // === Withdrawal Operations ===

    pub async fn create_withdrawal_request(&self, req: WithdrawalRequest)
        -> Result<WithdrawalRequest, DbError>;

    pub async fn find_withdrawal_by_id(&self, request_id: &str)
        -> Result<Option<WithdrawalRequest>, DbError>;

    pub async fn update_withdrawal_request(&self, req: &WithdrawalRequest)
        -> Result<(), DbError>;

    pub async fn find_pending_withdrawals_for_review(&self, limit: i64)
        -> Result<Vec<WithdrawalRequest>, DbError>;

    // === Deposit Operations ===

    pub async fn create_deposit_request(&self, req: DepositRequest)
        -> Result<DepositRequest, DbError>;

    pub async fn find_deposit_by_id(&self, deposit_id: &str)
        -> Result<Option<DepositRequest>, DbError>;

    pub async fn find_deposit_by_gateway_ref(&self, gateway_ref: &str)
        -> Result<Option<DepositRequest>, DbError>;

    pub async fn update_deposit_request(&self, req: &DepositRequest)
        -> Result<(), DbError>;

    // === Escrow Operations ===

    pub async fn create_escrow_hold(&self, escrow: EscrowHold)
        -> Result<EscrowHold, DbError>;

    pub async fn find_escrow_by_order_id(&self, order_id: &str)
        -> Result<Option<EscrowHold>, DbError>;

    pub async fn find_escrows_ready_for_release(&self)
        -> Result<Vec<EscrowHold>, DbError>;

    pub async fn update_escrow_hold(&self, escrow: &EscrowHold)
        -> Result<(), DbError>;

    // === Snapshot Operations ===

    pub async fn create_monthly_snapshot(&self, snapshot: MonthlySnapshot)
        -> Result<MonthlySnapshot, DbError>;

    pub async fn find_snapshot(&self, wallet_id: &str, month: &str)
        -> Result<Option<MonthlySnapshot>, DbError>;

    pub async fn find_latest_verified_snapshot(&self, wallet_id: &str)
        -> Result<Option<MonthlySnapshot>, DbError>;

    // === Commission Config Operations ===

    pub async fn get_active_commission_config(&self, shop_id: &str)
        -> Result<Option<ShopCommissionConfig>, DbError>;

    pub async fn create_commission_config(&self, config: ShopCommissionConfig)
        -> Result<ShopCommissionConfig, DbError>;

    // === Admin Log Operations ===

    pub async fn create_admin_log(&self, log: AdminOperationLog)
        -> Result<AdminOperationLog, DbError>;
}
```

---

## Service Layer Structure

Create `mmo-api/src/modules/wallet/service.rs`:

```rust
//! Wallet Service Layer
//!
//! Business logic for all wallet operations

use std::sync::Arc;
use ulid::Ulid;

use super::repository::WalletRepository;
use super::domain::*;
use super::dto::*;
use crate::core::error::ServiceError;

pub struct WalletService {
    repo: Arc<WalletRepository>,
    default_commission_rate: f64, // 0.05 = 5%
}

impl WalletService {
    pub fn new(repo: Arc<WalletRepository>) -> Self {
        Self {
            repo,
            default_commission_rate: 0.05,
        }
    }

    // === Wallet Management ===

    /// Create new wallet for user
    pub async fn create_wallet(&self, user_id: String, wallet_type: WalletType)
        -> Result<Wallet, ServiceError>;

    /// Get wallet balance
    pub async fn get_wallet_balance(&self, wallet_id: &str)
        -> Result<WalletBalanceResponse, ServiceError>;

    /// Get detailed wallet info
    pub async fn get_wallet_info(&self, wallet_id: &str)
        -> Result<WalletInfoResponse, ServiceError>;

    // === Deposit Flow ===

    /// Auto deposit - create deposit request with payment gateway
    pub async fn create_auto_deposit(&self, wallet_id: &str, req: AutoDepositRequest)
        -> Result<DepositResponse, ServiceError>;

    /// Process deposit webhook from payment gateway
    pub async fn process_deposit_webhook(
        &self,
        gateway_ref: &str,
        vnd_amount: i64,
        signature: &str,
    ) -> Result<(), ServiceError>;

    /// Manual deposit (admin only)
    pub async fn manual_deposit(&self, req: ManualDepositRequest, admin_id: String)
        -> Result<SuccessResponse, ServiceError>;

    // === Purchase Flow ===

    /// Process purchase (buyer → platform escrow)
    pub async fn process_purchase(&self, buyer_id: &str, req: PurchaseRequest)
        -> Result<PurchaseResponse, ServiceError>;

    // === Escrow Flow ===

    /// Auto-release escrows (cron job)
    pub async fn auto_release_escrows(&self) -> Result<i64, ServiceError>;

    /// Early release (buyer confirms)
    pub async fn early_release_escrow(&self, buyer_id: &str, order_id: &str)
        -> Result<SuccessResponse, ServiceError>;

    // === Withdrawal Flow ===

    /// Create withdrawal request
    pub async fn create_withdrawal(&self, wallet_id: &str, req: WithdrawalRequest)
        -> Result<WithdrawalResponse, ServiceError>;

    /// Validate withdrawal (background job)
    pub async fn validate_withdrawal(&self, request_id: &str)
        -> Result<ValidationResult, ServiceError>;

    /// Process approved withdrawal
    pub async fn process_withdrawal(&self, request_id: &str)
        -> Result<SuccessResponse, ServiceError>;

    // === Admin Operations ===

    /// Admin manual debit
    pub async fn admin_debit(&self, req: AdminDebitRequest, admin_info: AdminInfo)
        -> Result<SuccessResponse, ServiceError>;

    /// Admin freeze wallet
    pub async fn admin_freeze_wallet(&self, req: AdminFreezeRequest, admin_info: AdminInfo)
        -> Result<SuccessResponse, ServiceError>;

    /// Admin approve/reject withdrawal
    pub async fn admin_withdrawal_decision(
        &self,
        req: AdminWithdrawalDecisionRequest,
        admin_info: AdminInfo,
    ) -> Result<SuccessResponse, ServiceError>;

    // === Validation Engine ===

    /// Check 1: Balance integrity
    async fn validate_balance_integrity(&self, wallet: &Wallet)
        -> Result<CheckResult, ServiceError>;

    /// Check 2: Flow validation
    async fn validate_flow(&self, wallet: &Wallet, withdrawal_amount: i64)
        -> Result<CheckResult, ServiceError>;

    /// Check 3: Fraud detection
    async fn validate_fraud_patterns(&self, wallet: &Wallet, withdrawal_amount: i64)
        -> Result<CheckResult, ServiceError>;

    /// Check 4: Daily/monthly limits
    async fn validate_limits(&self, wallet: &Wallet, withdrawal_amount: i64)
        -> Result<CheckResult, ServiceError>;

    // === Helper Methods ===

    /// Generate unique ID with ULID
    fn generate_id(prefix: &str) -> String {
        format!("{}-{}", prefix, Ulid::new())
    }

    /// Calculate commission for shop/seller
    async fn get_commission_rate(&self, shop_id: &str) -> Result<f64, ServiceError>;

    /// Validate balance invariant
    fn validate_invariant(&self, wallet: &Wallet) -> Result<(), ServiceError>;
}

#[derive(Debug, Clone)]
pub struct AdminInfo {
    pub admin_id: String,
    pub admin_email: String,
    pub admin_role: String,
    pub ip_address: String,
    pub user_agent: String,
}
```

---

## Handler Layer Structure

Create `mmo-api/src/modules/wallet/handler.rs`:

```rust
//! Wallet HTTP Handlers
//!
//! Actix-web request handlers

use actix_web::{web, HttpResponse};
use validator::Validate;

use super::service::WalletService;
use super::dto::*;
use crate::core::error::ApiError;
use crate::middleware::auth::AuthUser;

// === User Endpoints ===

pub async fn get_wallet_balance(
    service: web::Data<WalletService>,
    auth: AuthUser,
) -> Result<HttpResponse, ApiError> {
    let wallet_id = &auth.wallet_id;
    let response = service.get_wallet_balance(wallet_id).await?;
    Ok(HttpResponse::Ok().json(response))
}

pub async fn create_deposit(
    service: web::Data<WalletService>,
    auth: AuthUser,
    req: web::Json<AutoDepositRequest>,
) -> Result<HttpResponse, ApiError> {
    req.validate()?;
    let response = service
        .create_auto_deposit(&auth.wallet_id, req.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(response))
}

pub async fn create_withdrawal(
    service: web::Data<WalletService>,
    auth: AuthUser,
    req: web::Json<WithdrawalRequest>,
) -> Result<HttpResponse, ApiError> {
    req.validate()?;
    let response = service
        .create_withdrawal(&auth.wallet_id, req.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(response))
}

pub async fn process_purchase(
    service: web::Data<WalletService>,
    auth: AuthUser,
    req: web::Json<PurchaseRequest>,
) -> Result<HttpResponse, ApiError> {
    req.validate()?;
    let response = service
        .process_purchase(&auth.user_id, req.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(response))
}

// === Admin Endpoints ===

pub async fn admin_manual_deposit(
    service: web::Data<WalletService>,
    auth: AuthUser,
    req: web::Json<ManualDepositRequest>,
) -> Result<HttpResponse, ApiError> {
    // Verify admin role
    auth.require_admin()?;

    req.validate()?;
    let response = service
        .manual_deposit(req.into_inner(), auth.user_id)
        .await?;
    Ok(HttpResponse::Ok().json(response))
}

// ... more handlers
```

---

## Routes Configuration

Create `mmo-api/src/modules/wallet/routes.rs`:

```rust
//! Wallet Routes Configuration

use actix_web::web;

use super::handler;

pub fn configure(cfg: &web::ServiceConfig) {
    cfg.service(
        web::scope("/api/wallet")
            // User endpoints
            .route("/balance", web::get().to(handler::get_wallet_balance))
            .route("/deposit", web::post().to(handler::create_deposit))
            .route("/withdrawal", web::post().to(handler::create_withdrawal))
            .route("/purchase", web::post().to(handler::process_purchase))
            .route("/transactions", web::get().to(handler::get_transaction_history))

            // Escrow endpoints
            .route("/escrow/{order_id}/early-release", web::post().to(handler::early_release_escrow))
            .route("/escrow/{order_id}", web::get().to(handler::get_escrow_info))

            // Refund endpoints
            .route("/refund", web::post().to(handler::request_refund))

            // Admin endpoints
            .service(
                web::scope("/admin")
                    .route("/deposit", web::post().to(handler::admin_manual_deposit))
                    .route("/debit", web::post().to(handler::admin_debit))
                    .route("/freeze", web::post().to(handler::admin_freeze_wallet))
                    .route("/unfreeze", web::post().to(handler::admin_unfreeze_wallet))
                    .route("/withdrawal/decision", web::post().to(handler::admin_withdrawal_decision))
                    .route("/commission/set", web::post().to(handler::admin_set_commission))
                    .route("/dashboard", web::get().to(handler::admin_dashboard))
                    .route("/withdrawals/pending", web::get().to(handler::admin_pending_withdrawals))
            )
    );
}
```

---

## Background Jobs Structure

Create `mmo-api/src/jobs/wallet_jobs.rs`:

```rust
//! Wallet Background Jobs
//!
//! Cron jobs for escrow release, snapshots, and reconciliation

use std::sync::Arc;
use crate::modules::wallet::service::WalletService;

/// Auto-release escrows (runs every hour)
pub async fn escrow_auto_release_job(service: Arc<WalletService>) {
    match service.auto_release_escrows().await {
        Ok(count) => {
            log::info!("Auto-released {} escrows", count);
        }
        Err(e) => {
            log::error!("Escrow auto-release failed: {:?}", e);
        }
    }
}

/// Create monthly snapshots (runs on 1st of month at 2:00 AM)
pub async fn monthly_snapshot_job(service: Arc<WalletService>) {
    // Implementation
}

/// Daily reconciliation (runs at 3:00 AM)
pub async fn daily_reconciliation_job(service: Arc<WalletService>) {
    // Implementation
}
```

---

## MongoDB Indexes

```javascript
// wallets collection
db.wallets.createIndex({ "wallet_id": 1 }, { unique: true });
db.wallets.createIndex({ "user_id": 1 });
db.wallets.createIndex({ "wallet_type": 1 });
db.wallets.createIndex({ "status": 1 });

// wallet_transactions collection
db.wallet_transactions.createIndex({ "tx_id": 1 }, { unique: true });
db.wallet_transactions.createIndex({ "wallet_id": 1, "created_at": -1 });
db.wallet_transactions.createIndex({ "wallet_id": 1, "status": 1, "created_at": -1 });
db.wallet_transactions.createIndex({ "tx_type": 1, "created_at": -1 });
db.wallet_transactions.createIndex({ "reference_id": 1 });

// withdrawal_requests collection
db.withdrawal_requests.createIndex({ "request_id": 1 }, { unique: true });
db.withdrawal_requests.createIndex({ "wallet_id": 1, "status": 1 });
db.withdrawal_requests.createIndex({ "status": 1, "created_at": -1 });

// deposit_requests collection
db.deposit_requests.createIndex({ "deposit_id": 1 }, { unique: true });
db.deposit_requests.createIndex({ "payment_gateway_ref": 1 }, { unique: true, sparse: true });
db.deposit_requests.createIndex({ "wallet_id": 1, "status": 1 });

// escrow_holds collection
db.escrow_holds.createIndex({ "escrow_id": 1 }, { unique: true });
db.escrow_holds.createIndex({ "order_id": 1 }, { unique: true });
db.escrow_holds.createIndex({ "status": 1, "release_at": 1 });
db.escrow_holds.createIndex({ "seller_id": 1, "status": 1 });
db.escrow_holds.createIndex({ "buyer_id": 1, "status": 1 });

// monthly_snapshots collection
db.monthly_snapshots.createIndex({ "snapshot_id": 1 }, { unique: true });
db.monthly_snapshots.createIndex({ "wallet_id": 1, "month": -1 });
db.monthly_snapshots.createIndex({ "status": 1 });

// shop_commission_configs collection
db.shop_commission_configs.createIndex({ "shop_id": 1, "effective_from": -1 });
db.shop_commission_configs.createIndex({ "shop_id": 1, "effective_to": 1 }, { sparse: true });

// admin_operation_logs collection
db.admin_operation_logs.createIndex({ "log_id": 1 }, { unique: true });
db.admin_operation_logs.createIndex({ "admin_id": 1, "created_at": -1 });
db.admin_operation_logs.createIndex({ "target_id": 1, "created_at": -1 });
db.admin_operation_logs.createIndex({ "operation": 1, "created_at": -1 });
```

---

## Next Steps

1. ✅ **Implement Repository Layer** - Complete all MongoDB operations
2. **Implement Service Layer** - Business logic for all flows
3. **Implement Handlers** - HTTP request handlers
4. **Implement Routes** - API endpoint configuration
5. **Implement Background Jobs** - Cron jobs for escrow/snapshots/reconciliation
6. **Write Tests** - Unit + integration tests
7. **Add Logging** - Comprehensive logging with tracing
8. **Add Metrics** - Prometheus metrics for monitoring
9. **Performance Testing** - Load testing with k6
10. **Security Audit** - Review for vulnerabilities

---

## Key Implementation Notes

### Trust Currency Conversion
```rust
const VND_TO_TRUST_RATE: i64 = 1000;

fn vnd_to_trust(vnd: i64) -> i64 {
    vnd / VND_TO_TRUST_RATE
}

fn trust_to_vnd(trust: i64) -> i64 {
    trust * VND_TO_TRUST_RATE
}
```

### Balance Invariant Validation
```rust
fn validate_balance_invariant(wallet: &Wallet) -> Result<(), ServiceError> {
    let calculated = wallet.available_trust
        + wallet.withdrawal_locked
        + wallet.dispute_locked;

    if calculated != wallet.total_trust {
        return Err(ServiceError::BalanceInvariantViolation {
            wallet_id: wallet.wallet_id.clone(),
            expected: calculated,
            actual: wallet.total_trust,
        });
    }

    Ok(())
}
```

### Transaction Atomicity
All multi-step operations must use MongoDB transactions:
```rust
let mut session = client.start_session(None).await?;
session.start_transaction(None).await?;

// Perform operations with session
// ...

session.commit_transaction().await?;
```

---

## References

- [Platform Wallet Flows](./05-wallet-v2-platform-wallet-flows.md)
- [Trust Wallet V3 Design](./TaphoaMMO_Trust_Wallet_V3_Design.md)
- [Architecture](../ARCHITECTURE.md)
- [Coding Standards](../CODING_STANDARDS.md)
