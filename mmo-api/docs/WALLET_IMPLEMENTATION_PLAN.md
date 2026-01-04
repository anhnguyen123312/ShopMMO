# Wallet Module Implementation Plan

## Overview
Implementation of the Wallet module for P2PMMO V2 with Trust Currency (1 Trust = 1000 VND), escrow system, and USDT TRC20 payment support.

**Tech Stack**: Rust + actix-web + MongoDB + Redis + TRC20 Blockchain Monitor

**Module Location**: `src/modules/wallet/`

---

## Architecture

### Directory Structure
```
src/modules/wallet/
├── mod.rs              # Module exports
├── domain.rs           # MongoDB models (Wallet, Transaction, EscrowHold, WithdrawalRequest, etc.)
├── dto.rs              # Request/Response DTOs with validation
├── handler.rs          # HTTP handlers with utoipa documentation
├── service.rs          # Business logic layer
├── repository.rs       # MongoDB operations
└── routes.rs           # Route configuration

src/services/
└── usdt_monitor.rs     # TRC20 blockchain monitoring service
```

---

## Data Models

### 1. Wallet Collection
```rust
pub struct Wallet {
    pub wallet_id: String,              // "WLT-{ULID}"
    pub user_id: String,                // User ID or "PLATFORM"
    pub wallet_type: WalletType,        // USER | SELLER | PLATFORM

    // Balance States (i64 for Trust, 1000 VND = 1 Trust)
    pub available_trust: i64,           // Available to use
    pub withdrawal_locked: i64,         // Locked for withdrawal
    pub dispute_locked: i64,            // Locked in dispute

    // Computed
    pub total_trust: i64,               // Sum of above

    // Running Totals
    pub lifetime_deposited: i64,
    pub lifetime_withdrawn: i64,
    pub lifetime_spent: i64,
    pub lifetime_received: i64,

    // Seller-specific
    pub commission_rate: Option<f64>,   // Override rate (nullable)
    pub commission_debt: i64,           // Accumulated commission debt

    // Monthly Snapshot
    pub last_snapshot_month: Option<String>,
    pub last_snapshot_balance: Option<i64>,
    pub last_snapshot_verified: bool,

    pub status: WalletStatus,           // ACTIVE | SUSPENDED | FROZEN
    pub freeze_reason: Option<String>,

    pub created_at: DateTime,
    pub updated_at: DateTime,
}
```

### 2. Transaction Collection
```rust
pub struct Transaction {
    pub tx_id: String,                  // "TXN-{ULID}"
    pub wallet_id: String,
    pub user_id: String,

    pub tx_type: TransactionType,
    pub direction: TransactionDirection, // CREDIT | DEBIT

    pub amount: i64,                    // Trust amount
    pub vnd_amount: Option<i64>,        // VND equivalent
    pub fee_amount: Option<i64>,

    pub balance_before: i64,
    pub balance_after: i64,
    pub balance_type: BalanceType,

    pub running_deposited: Option<i64>,
    pub running_withdrawn: Option<i64>,

    pub status: TransactionStatus,
    pub status_history: Vec<StatusHistory>,

    pub reference_type: Option<String>,
    pub reference_id: Option<String>,
    pub external_ref: Option<String>,   // Bank/gateway reference

    pub initiated_by: String,
    pub admin_note: Option<String>,

    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub completed_at: Option<DateTime>,
}
```

### 3. EscrowHold Collection
```rust
pub struct EscrowHold {
    pub escrow_id: String,              // "ESC-{ULID}"
    pub order_id: String,
    pub buyer_id: String,
    pub seller_id: String,

    pub escrow_amount: i64,
    pub commission_rate: f64,
    pub commission_amount: Option<i64>,

    pub status: EscrowStatus,           // HOLDING | RELEASED | REFUNDED | DISPUTED
    pub created_at: DateTime,
    pub release_at: DateTime,
    pub released_at: Option<DateTime>,

    pub early_release: bool,
    pub early_release_by: Option<String>,

    pub dispute_id: Option<String>,
    pub locked_at: Option<DateTime>,
}
```

### 4. WithdrawalRequest Collection
```rust
pub struct WithdrawalRequest {
    pub request_id: String,             // "WD-{ULID}"
    pub wallet_id: String,
    pub user_id: String,

    pub trust_amount: i64,
    pub commission_deduct: i64,
    pub net_trust: i64,
    pub vnd_amount: i64,

    // Bank Info
    pub bank_code: String,
    pub bank_name: String,
    pub account_number: String,
    pub account_name: String,

    pub status: WithdrawalStatus,
    pub status_history: Vec<StatusHistory>,

    pub validation_result: Option<ValidationResult>,

    pub approved_by: Option<String>,
    pub approved_at: Option<DateTime>,
    pub bank_transfer_ref: Option<String>,
    pub bank_transfer_at: Option<DateTime>,

    pub reject_reason: Option<String>,

    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub completed_at: Option<DateTime>,
    pub expires_at: DateTime,
}
```

### 5. UsdtDeposit Collection (New for USDT)
```rust
pub struct UsdtDeposit {
    pub deposit_id: String,             // "USDT-{ULID}"
    pub wallet_id: String,
    pub user_id: String,

    pub usdt_amount: f64,               // USDT amount
    pub network: UsdtNetwork,           // TRC20
    pub sender_address: String,
    pub transaction_hash: String,
    pub block_number: i64,

    pub vnd_amount: i64,                // Converted to VND
    pub trust_amount: i64,              // Converted to Trust

    pub exchange_rate: f64,             // USDT to VND rate

    pub status: UsdtDepositStatus,      // PENDING | CONFIRMING | CONFIRMED | CREDITED | FAILED
    pub confirmations: i32,             // Current confirmations
    pub required_confirmations: i32,    // Usually 20 for TRC20

    pub credited_at: Option<DateTime>,
    pub failed_reason: Option<String>,

    pub created_at: DateTime,
    pub updated_at: DateTime,
}
```

---

## Transaction Types

| Category | Type | Direction | Description |
|----------|------|-----------|-------------|
| **Deposit** | DEPOSIT_PENDING | - | Waiting for payment |
| | DEPOSIT_VND_RECEIVED | CREDIT | VND received from gateway |
| | DEPOSIT_TRUST_CREDITED | CREDIT | Trust added to wallet |
| | DEPOSIT_MANUAL | CREDIT | Admin manual deposit |
| | DEPOSIT_USDT | CREDIT | USDT deposit credited |
| **Withdrawal** | WITHDRAWAL_REQUEST | DEBIT | Lock funds for withdrawal |
| | WITHDRAWAL_COMPLETED | DEBIT | Finalize withdrawal |
| | WITHDRAWAL_REJECTED | CREDIT | Refund rejected withdrawal |
| **Purchase** | PURCHASE_DEBIT | DEBIT | Buyer pays |
| | ESCROW_HOLD | CREDIT | Platform receives |
| | ESCROW_RELEASE | DEBIT | Platform pays seller |
| **Refund** | REFUND_ESCROW | DEBIT | Refund from escrow |
| **Commission** | COMMISSION_ACCRUE | - | Record commission debt |
| | COMMISSION_DEDUCT | DEBIT | Deduct on withdrawal |
| | COMMISSION_COLLECTED | CREDIT | Platform receives |
| **Admin** | ADMIN_CREDIT | CREDIT | Admin adds trust |
| | ADMIN_DEBIT | DEBIT | Admin removes trust |
| | ADMIN_FREEZE | DEBIT | Freeze funds to dispute_locked |
| | ADMIN_UNFREEZE | CREDIT | Unfreeze funds |

---

## API Endpoints

### User Wallet APIs

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| GET | `/api/v3/wallet` | Get wallet info | User |
| GET | `/api/v3/wallet/balance` | Get current balance | User |
| GET | `/api/v3/wallet/transactions` | Get transaction history | User |
| POST | `/api/v3/wallet/deposit/initiate` | Initiate deposit via gateway | User |
| POST | `/api/v3/wallet/deposit/usdt/generate-address` | Generate USDT deposit address | User |
| GET | `/api/v3/wallet/deposit/usdt/status/{deposit_id}` | Check USDT deposit status | User |
| POST | `/api/v3/wallet/withdrawal/request` | Request withdrawal | User |
| GET | `/api/v3/wallet/withdrawal/{request_id}` | Get withdrawal status | User |

### Seller Wallet APIs

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| GET | `/api/v3/seller/wallet` | Get seller wallet | Seller |
| GET | `/api/v3/seller/escrows` | Get pending escrows | Seller |
| GET | `/api/v3/seller/commission` | Get commission debt | Seller |
| POST | `/api/v3/seller/withdraw` | Request withdrawal | Seller |

### Admin Wallet APIs

| Method | Endpoint | Description | Auth | Permission |
|--------|----------|-------------|------|------------|
| GET | `/api/v3/admin/wallets/dashboard` | Dashboard overview | Admin | DASHBOARD |
| POST | `/api/v3/admin/wallets/deposit` | Manual deposit | Admin | WALLET_DEPOSIT |
| POST | `/api/v3/admin/wallets/debit` | Manual debit | Admin | WALLET_DEBIT |
| GET | `/api/v3/admin/wallets/{user_id}` | Get user wallet | Admin | WALLET_VIEW |
| GET | `/api/v3/admin/withdrawals/pending` | List pending withdrawals | Admin | WITHDRAWAL_VIEW |
| POST | `/api/v3/admin/withdrawals/{id}/approve` | Approve withdrawal | Admin | WITHDRAWAL_APPROVE |
| POST | `/api/v3/admin/withdrawals/{id}/reject` | Reject withdrawal | Admin | WITHDRAWAL_APPROVE |
| POST | `/api/v3/admin/commission/setup` | Setup commission rate | Admin | COMMISSION_MANAGE |
| GET | `/api/v3/admin/reconcile/daily` | Trigger daily reconciliation | Admin | RECONCILE |
| GET | `/api/v3/admin/usdt/deposits` | List USDT deposits | Admin | USDT_VIEW |
| POST | `/api/v3/admin/usdt/deposits/{id}/credit` | Manually credit USDT deposit | Admin | USDT_CREDIT |

---

## USDT TRC20 Integration

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    USDT DEPOSIT FLOW                            │
└─────────────────────────────────────────────────────────────────┘

[Step 1] User requests deposit address
         │
         ▼
[Step 2] System generates unique payment ID
         │
         ▼
[Step 3] Returns platform TRC20 address with memo:
         "USDT-{user_id}-{ulid}"
         │
         ▼
[Step 4] User sends USDT to address
         │
         ▼
───────────────────────────────────────────────────────────────────
[Background Monitor - Cron every 30 seconds]
───────────────────────────────────────────────────────────────────
         │
         ▼
[Step 5] Query TRC20 transactions for platform address
         │
         ▼
[Step 6] For each new transaction:
         │
         ├── Parse transaction hash, amount, sender
         ├── Parse memo from transaction data
         ├── Extract user_id from memo
         │
         ├── Valid memo format ──► Continue
         ├── Invalid/No memo ──► Log warning, skip
         │
         ▼
[Step 7] Check if already processed
         │
         ├── Already exists ──► Skip (idempotent)
         ├── New ──► Continue
         │
         ▼
[Step 8] Get exchange rate (USDT → VND)
         │
         ├── Call external price API or use fixed rate
         │
         ▼
[Step 9] Create UsdtDeposit record:
         │
         ├── status: CONFIRMING
         ├── confirmations: current
         ├── required_confirmations: 20
         │
         ▼
[Step 10] Wait for required confirmations (poll)
         │
         ├── confirmations >= 20 ──► CREDIT
         │
         ▼
[Step 11] BEGIN TRANSACTION
         │
         ▼
[Step 12] Create transaction (DEPOSIT_USDT)
         │
         ▼
[Step 13] Update wallet:
         │
         ├── available_trust += trust_amount
         ├── total_trust += trust_amount
         ├── lifetime_deposited += trust_amount
         │
         ▼
[Step 14] Update UsdtDeposit:
         │
         ├── status: CREDITED
         ├── credited_at: now
         │
         ▼
[Step 15] COMMIT
         │
         ▼
[Step 16] Notify user: "Nạp thành công {trust_amount} Trust"
```

### TronGrid API Integration

**Base URL**: `https://api.trongrid.io`

**Endpoints**:
- Get transaction by hash: `/v1/transactions/{tx_id}`
- Get transactions by account: `/v1/accounts/{address}/transactions/trc20`
- Get block: `/wallet/getblock`

**Rate Limits**: Free tier: 3 requests/second

**Example Flow**:
```rust
// 1. Get latest TRC20 transactions for platform address
let url = format!("{}/v1/accounts/{}/transactions/trc20?only_confirmed=true&limit=50",
    TRONGRID_URL, PLATFORM_USDT_ADDRESS);

let response = reqwest::get(&url).await?.json::<TronTransactionsResponse>().await?;

// 2. For each transaction, check if it's USDT (TRC21)
for tx in response.data {
    if tx.token_address == USDT_TRC20_ADDRESS {
        // Check memo for user_id
        let memo = extract_memo(&tx)?;

        if let Some(user_id) = parse_user_id_from_memo(&memo) {
            // Process deposit
            process_usdt_deposit(tx, user_id).await?;
        }
    }
}
```

### Configuration

```env
# USDT Configuration
USDT_NETWORK=trc20
USDT_TRC20_ADDRESS=TRC20_ADDRESS_HERE
USDT_PLATFORM_ADDRESS=PLATFORM_TRC20_ADDRESS
USDT_REQUIRED_CONFIRMATIONS=20
USDT_EXCHANGE_RATE_API=https://api.example.com/usdt/vnd
USDT_FIXED_RATE=25000  # Fallback fixed rate (1 USDT = 25000 VND)
USDT_MIN_DEPOSIT=1.0   # Minimum 1 USDT
USDT_MAX_DEPOSIT=10000.0  # Maximum 10000 USDT

# TronGrid API
TRONGRID_API_KEY=your_api_key
TRONGRID_API_URL=https://api.trongrid.io
```

---

## Business Rules Summary

| # | Rule |
|---|------|
| **BR1** | 1000 VND = 1 Trust (fixed conversion) |
| **BR2** | All transactions go through Platform Wallet for escrow |
| **BR3** | Escrow hold: 3 days (72 hours) |
| **BR4** | Commission default: 5%, overridable 1-20% |
| **BR5** | Withdrawal validation: Balance + Flow + Fraud + Limits |
| **BR6** | Monthly snapshot: 1st of month, 2:00 AM |
| **BR7** | Daily reconciliation: 3:00 AM daily |
| **BR8** | Discrepancy > 100 Trust: CRITICAL alert |
| **BR9** | Risk score >= 0.7: Auto-reject withdrawal |
| **BR10** | All admin operations must have audit log |
| **BR11** | USDT minimum: 1 USDT, maximum: 10000 USDT |
| **BR12** | USDT confirmations required: 20 blocks |
| **BR13** | USDT memo format: "USDT-{user_id}-{timestamp}" |

---

## Implementation Order

### Phase 1: Core Models & Repository (Day 1)
1. Create domain models (Wallet, Transaction, EscrowHold, etc.)
2. Create base repository with MongoDB operations
3. Create indexes for collections
4. Create error types

### Phase 2: Basic Wallet Operations (Day 2)
1. Implement wallet creation (auto-create on user registration)
2. Implement balance retrieval
3. Implement transaction history
4. Implement basic deposit (admin manual)
5. Implement basic withdrawal (request)

### Phase 3: Escrow System (Day 3)
1. Implement escrow hold on purchase
2. Implement escrow release (cron)
3. Implement commission accrual
4. Implement commission deduction on withdrawal
5. Implement platform wallet operations

### Phase 4: Validation Engine (Day 4)
1. Implement balance integrity check
2. Implement flow validation check
3. Implement fraud pattern detection
4. Implement limits check
5. Implement risk score aggregation

### Phase 5: Admin Operations (Day 5)
1. Implement manual deposit/debit
2. Implement withdrawal approval/rejection
3. Implement commission setup
4. Implement dashboard stats
5. Implement reconciliation

### Phase 6: USDT Integration (Day 6-7)
1. Implement USDT deposit model
2. Implement TRC20 monitor service
3. Implement exchange rate fetcher
4. Implement confirmation polling
5. Implement manual credit for USDT

### Phase 7: Testing & Documentation (Day 8)
1. Write unit tests
2. Write integration tests
3. Complete utoipa OpenAPI docs
4. Test all flows end-to-end
5. Performance testing

---

## Dependencies

```toml
[dependencies]
# Core
actix-web = "4.9"
tokio = { version = "1.42", features = ["full"] }
mongodb = "3.1"
redis = "0.27"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Validation
validator = { version = "0.18", features = ["derive"] }

# OpenAPI
utoipa = { version = "5.4", features = ["actix_extras", "macros"] }
utoipa-swagger-ui = { version = "9", features = ["actix-web"] }

# Error handling
thiserror = "2.0"
anyhow = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Authentication
jsonwebtoken = "9"
bcrypt = "0.16"

# IDs
ulid = "1.1"

# Date/time
chrono = { version = "0.4", features = ["serde"] }

# HTTP client (for blockchain APIs)
reqwest = { version = "0.12", features = ["json"] }

# Environment
dotenv = "0.15"

# Async utilities
futures = "0.3"
```

---

## Security Considerations

1. **Transaction Integrity**: All wallet operations use database transactions
2. **Idempotency**: All deposits use idempotency keys (ULID)
3. **Validation**: Multi-layer validation (handler → service → repository)
4. **Audit Trail**: All admin operations logged
5. **Rate Limiting**: API endpoints rate limited per user
6. **Encryption**: Sensitive data encrypted at rest
7. **Webhook Validation**: Payment gateway webhooks signature verified
8. **Blockchain Verification**: USDT transactions verified on-chain

---

## Performance Considerations

1. **Connection Pooling**: MongoDB connection pool (100 max, 10 min)
2. **Caching**: Redis cache for wallet balances (5 min TTL)
3. **Monthly Snapshots**: Incremental balance validation
4. **Indexing**: Proper indexes on all query fields
5. **Pagination**: All list APIs paginated
6. **Async Processing**: USDT monitoring runs as background job

---

## Next Steps

1. Review this plan and approve
2. Create the wallet module structure
3. Implement domain models
4. Implement repository layer
5. Implement service layer
6. Implement handlers with utoipa docs
7. Implement USDT monitor service
8. Test all endpoints

---

**Document Version**: 1.0
**Last Updated**: 2026-01-04
**Author**: Claude Code
**Status**: Ready for Implementation
