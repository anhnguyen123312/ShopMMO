# Trust Wallet System V3

## Tổng quan

**Mục đích**: Thiết kế wallet system với Trust Currency (1 Trust = 1000 VND) hỗ trợ 3 loại wallet (User, Seller, Platform) với escrow mechanism và validation engine mạnh mẽ.

**Scope**:
- User Wallet: Nạp/rút tiền, mua hàng
- Seller Wallet: Nhận escrow, rút với commission
- Platform Wallet: Escrow pool, commission collection
- Admin Operations: Manual adjustments, approvals, commission setup

**Actors**:
- **Buyer**: User mua hàng, nạp/rút Trust
- **Seller**: Vendor bán hàng, nhận escrow, trả commission
- **Admin**: Manage wallets, approve withdrawals, setup commission
- **System**: Auto escrow release, validation engine, reconciliation

---

## 1. Trust Currency & Wallet Architecture

### 1.1 Trust Currency

```
┌─────────────────────────────────────────┐
│            TRUST CURRENCY                │
├─────────────────────────────────────────┤
│  1000 VND = 1 Trust (Cố định)           │
│                                          │
│  • Nạp tiền: VND → Trust (3rd party)    │
│  • Rút tiền: Trust → VND (bank transfer)│
│  • Giao dịch: Trust only (i64, no float)│
└─────────────────────────────────────────┘
```

### 1.2 Wallet Types

| Type | Description | Key Features |
|------|-------------|--------------|
| **USER** | Buyer wallet | Mua hàng, nạp/rút cơ bản |
| **SELLER** | Vendor wallet | Nhận escrow, commission debt |
| **PLATFORM** | System wallet | Escrow pool, commission collected |

### 1.3 Commission Setup

```
┌─────────────────────────────────────────┐
│        COMMISSION BY SHOP               │
├─────────────────────────────────────────┤
│  Default: 5%                            │
│  Per-shop: 1% - 20%                     │
│                                         │
│  shop_commission_config {               │
│    shop_id: ObjectId                    │
│    rate: f64 (0.01 - 0.20)             │
│    effective_from: DateTime            │
│    effective_to: DateTime (nullable)   │
│    created_by: admin_id                │
│  }                                      │
└─────────────────────────────────────────┘
```

---

## 2. Data Models

### 2.1 Wallet Collection

```javascript
{
  _id: ObjectId,
  wallet_id: String,              // "WLT-{ULID}"
  user_id: String,                // User ID or "PLATFORM"
  wallet_type: String,            // "USER" | "SELLER" | "PLATFORM"

  // Balance States
  available_trust: i64,           // Có thể dùng ngay
  withdrawal_locked: i64,         // Đang chờ rút
  dispute_locked: i64,            // Đang tranh chấp

  // Computed
  total_trust: i64,               // = available + withdrawal_locked + dispute_locked

  // Running Totals (for validation)
  lifetime_deposited: i64,        // Tổng đã nạp
  lifetime_withdrawn: i64,        // Tổng đã rút
  lifetime_spent: i64,            // Tổng đã chi (mua hàng)
  lifetime_received: i64,         // Tổng đã nhận (bán hàng)

  // Seller-specific
  commission_rate: f64,           // Override rate (nullable)
  commission_debt: i64,           // Tích lũy commission chưa trả

  // Monthly Snapshot Reference
  last_snapshot_month: String,    // "2026-01"
  last_snapshot_balance: i64,     // Balance tại cuối tháng
  last_snapshot_verified: bool,

  status: String,                 // "ACTIVE" | "SUSPENDED" | "FROZEN"
  freeze_reason: String,

  created_at: DateTime,
  updated_at: DateTime
}

// Indexes
db.wallets.createIndex({ "user_id": 1 }, { unique: true })
db.wallets.createIndex({ "wallet_type": 1, "status": 1 })
db.wallets.createIndex({ "last_snapshot_month": 1 })
```

### 2.2 Transaction Collection

```javascript
{
  _id: ObjectId,
  tx_id: String,                  // "TXN-{ULID}"
  wallet_id: String,
  user_id: String,

  // Type & Direction
  tx_type: String,                // DEPOSIT_*, WITHDRAWAL_*, PURCHASE_*, etc.
  direction: String,              // "CREDIT" (+) or "DEBIT" (-)

  // Amounts
  amount: i64,                    // Trust amount (always positive)
  vnd_amount: i64,                // VND equivalent (nullable)
  fee_amount: i64,                // Fee/commission deducted

  // Balance Tracking
  balance_before: i64,
  balance_after: i64,
  balance_type: String,           // "AVAILABLE" | "WITHDRAWAL_LOCKED" | "DISPUTE_LOCKED"

  // Running Totals After Tx
  running_deposited: i64,
  running_withdrawn: i64,

  // Status
  status: String,                 // "PENDING" | "PROCESSING" | "COMPLETED" | "FAILED" | etc.
  status_history: Array<{
    from_status: String,
    to_status: String,
    changed_at: DateTime,
    changed_by: String,
    reason: String
  }>,

  // References
  reference_type: String,         // "order" | "withdrawal" | "refund"
  reference_id: String,
  external_ref: String,           // Bank ref, payment gateway ref

  // Admin/System
  initiated_by: String,           // user_id or "SYSTEM" or admin_id
  admin_note: String,

  created_at: DateTime,
  updated_at: DateTime,
  completed_at: DateTime
}

// Indexes
db.transactions.createIndex({ "wallet_id": 1, "created_at": -1 })
db.transactions.createIndex({ "wallet_id": 1, "status": 1, "created_at": -1 })
db.transactions.createIndex({ "tx_type": 1, "created_at": -1 })
db.transactions.createIndex({ "reference_type": 1, "reference_id": 1 })
```

**Transaction Types**:

| Category | Type | Direction | Description |
|----------|------|-----------|-------------|
| **Deposit** | DEPOSIT_PENDING | - | Waiting for payment |
| | DEPOSIT_VND_RECEIVED | CREDIT | VND received from gateway |
| | DEPOSIT_TRUST_CREDITED | CREDIT | Trust added to wallet |
| | DEPOSIT_MANUAL | CREDIT | Admin manual deposit |
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

### 2.3 Monthly Snapshot Collection

```javascript
{
  _id: ObjectId,
  snapshot_id: String,            // "SNAP-{wallet_id}-{YYYY-MM}"
  wallet_id: String,
  user_id: String,
  month: String,                  // "2026-01"

  // Balances at End of Month
  closing_balance: i64,           // Calculated from transactions
  actual_balance: i64,            // From wallet at snapshot time

  // Running Totals
  total_deposited: i64,
  total_withdrawn: i64,
  total_spent: i64,
  total_received: i64,

  // Verification
  discrepancy: i64,               // closing - actual
  status: String,                 // "PENDING" | "VERIFIED" | "DISCREPANCY" | "CRITICAL"
  verified_at: DateTime,
  verified_by: String,

  // Transaction Summary
  tx_count: i64,
  first_tx_id: String,
  last_tx_id: String,

  created_at: DateTime
}

// Indexes
db.monthly_snapshots.createIndex(
  { "wallet_id": 1, "month": -1 },
  { unique: true }
)
```

### 2.4 Withdrawal Request Collection

```javascript
{
  _id: ObjectId,
  request_id: String,             // "WD-{ULID}"
  wallet_id: String,
  user_id: String,

  // Amounts
  trust_amount: i64,              // Trust to withdraw
  commission_deduct: i64,         // Commission to pay (seller only)
  net_trust: i64,                 // trust_amount - commission_deduct
  vnd_amount: i64,                // net_trust * 1000

  // Bank Info
  bank_code: String,
  bank_name: String,
  account_number: String,
  account_name: String,

  // Status
  status: String,                 // "PENDING" | "VALIDATING" | "APPROVED" | "COMPLETED" | etc.
  status_history: Array,

  // Validation
  validation_result: {
    balance_check: { passed: bool, details: String, severity: String },
    flow_check: { passed: bool, details: String, severity: String },
    fraud_check: { passed: bool, details: String, severity: String },
    limit_check: { passed: bool, details: String, severity: String },
    overall_passed: bool,
    risk_score: f64
  },

  // Processing
  approved_by: String,
  approved_at: DateTime,
  bank_transfer_ref: String,
  bank_transfer_at: DateTime,

  created_at: DateTime,
  updated_at: DateTime,
  completed_at: DateTime,
  expires_at: DateTime
}

// Indexes
db.withdrawal_requests.createIndex({ "status": 1, "created_at": -1 })
db.withdrawal_requests.createIndex({ "wallet_id": 1, "status": 1 })
```

### 2.5 Admin Operation Log Collection

```javascript
{
  _id: ObjectId,
  log_id: String,                 // "ALOG-{ULID}"

  // Admin Info
  admin_id: String,
  admin_email: String,
  admin_role: String,

  // Operation
  operation: String,              // "MANUAL_DEPOSIT" | "MANUAL_DEBIT" | "WITHDRAWAL_APPROVE" | etc.
  target_type: String,            // "WALLET" | "USER" | "SHOP" | "WITHDRAWAL"
  target_id: String,

  // Before/After
  before_state: Object,
  after_state: Object,

  // Details
  amount: i64,
  reason: String,
  note: String,

  // Related
  transaction_id: String,

  // Metadata
  ip_address: String,
  user_agent: String,
  created_at: DateTime
}

// Indexes
db.admin_operation_logs.createIndex({ "admin_id": 1, "created_at": -1 })
db.admin_operation_logs.createIndex({ "target_type": 1, "target_id": 1 })
```

---

## 3. User Wallet Flows

### 3.1 Deposit Flow (via 3rd Party)

#### 3.1.1 Conditions/Requirements

┌─────────────────────────────────────────┐
│         ĐIỀU KIỆN THỰC HIỆN            │
└─────────────────────────────────────────┘

1. **Actors**: Buyer, 3rd Party Payment Gateway, System
2. **Preconditions**:
   ├── User logged in
   ├── Wallet exists (status = ACTIVE)
   └── Payment gateway configured (VNPay/MoMo/etc.)

3. **Input Requirements**:
   ├── amount_vnd: Number (10,000 - 50,000,000)
   ├── Must be divisible by 1,000
   └── Payment method selected

4. **Business Rules**:
   ├── Min deposit: 10,000 VND (10 Trust)
   ├── Max deposit: 50,000,000 VND (50,000 Trust)
   ├── Payment expires: 15 minutes
   └── Webhook must be validated (signature check)

5. **Edge Cases**:
   ├── Payment expired ──► Mark transaction as EXPIRED
   ├── Payment cancelled ──► Mark as CANCELLED
   ├── Webhook signature invalid ──► Reject, log suspicious
   └── Duplicate webhook ──► Ignore (idempotent)

#### 3.1.2 Flow

┌─────────────────────────────────────────┐
│        USER DEPOSIT FLOW                │
└─────────────────────────────────────────┘

[Bước 1] User nhấn "Nạp tiền"
         │
         ▼
[Bước 2] Hệ thống hiển thị form nhập:

         ╔═══════════════════════════════════════╗
         ║  Nạp tiền vào Wallet                 ║
         ╠═══════════════════════════════════════╣
         ║  Số tiền VND: [______________]       ║
         ║  Min: 10,000 | Max: 50,000,000       ║
         ║                                       ║
         ║  Phương thức:                         ║
         ║  ○ VNPay                              ║
         ║  ○ MoMo                              ║
         ║  ○ Bank Transfer                     ║
         ║                                       ║
         ║  [Hủy]          [Tiếp tục]            ║
         ╚═══════════════════════════════════════╝
         │
         ▼
[Bước 3] User nhập số tiền, chọn phương thức
         │
         ├── Invalid (min/max/divisible) ──► Show error, return to Bước 2
         ├── Valid ──► Continue
         │
         ▼
[Bước 4] Hệ thống validate và tính Trust:
         │
         ├── trust_amount = amount_vnd / 1000
         ├── Ví dụ: 100,000 VND = 100 Trust
         │
         ▼
[Bước 5] Tạo Transaction #1:
         │
         ├── Type: DEPOSIT_PENDING
         ├── Status: PENDING
         ├── amount: trust_amount
         ├── vnd_amount: amount_vnd
         └── expires_at: now + 15 minutes
         │
         ▼
[Bước 6] Gọi 3rd Party API, nhận payment_url
         │
         ▼
[Bước 7] Redirect user đến payment gateway
         │
         ▼
         ─────────────────────────────────────────
         [User pays at gateway - external flow]
         ─────────────────────────────────────────
         │
         ├── Payment SUCCESS ──► Gateway sends webhook
         ├── Payment CANCELLED ──► User returns, mark CANCELLED
         ├── Payment TIMEOUT ──► Cron marks EXPIRED
         │
         ▼
[Bước 8] Gateway gửi webhook callback
         │
         ├── Validate signature ──► Continue
         ├── Invalid signature ──► Reject webhook, alert security
         │
         ▼
[Bước 9] BEGIN DATABASE TRANSACTION
         │
         ▼
[Bước 10] Update Transaction #1:
         │
         ├── Status: PENDING → COMPLETED
         ├── external_ref: gateway_transaction_id
         └── completed_at: now
         │
         ▼
[Bước 11] Tạo Transaction #2:
         │
         ├── Type: DEPOSIT_TRUST_CREDITED
         ├── Status: COMPLETED
         ├── direction: CREDIT
         ├── amount: trust_amount
         ├── balance_before: old_balance
         ├── balance_after: old_balance + trust_amount
         └── running_deposited: lifetime_deposited + trust_amount
         │
         ▼
[Bước 12] Update Wallet:
         │
         ├── available_trust += trust_amount
         ├── total_trust += trust_amount
         └── lifetime_deposited += trust_amount
         │
         ▼
[Bước 13] Validate Invariant:
         │
         ├── new_balance == old_balance + trust_amount?
         ├── Passed ──► COMMIT
         ├── Failed ──► ROLLBACK, alert admin CRITICAL
         │
         ▼
[Bước 14] Send notification to user:
         │
         ├── "Nạp tiền thành công"
         ├── Amount: +100 Trust
         └── Current balance: X Trust

#### 3.1.3 Edge Cases & Error Handling

| Case | Condition | Handling | User Message |
|------|-----------|----------|--------------|
| Invalid amount | < 10,000 or > 50,000,000 or not divisible by 1,000 | Reject with 400 | "Số tiền không hợp lệ. Min: 10,000, Max: 50,000,000 VND" |
| Payment expired | No webhook after 15 min | Mark as EXPIRED | "Giao dịch đã hết hạn. Vui lòng thử lại." |
| Payment cancelled | User cancels at gateway | Mark as CANCELLED | "Bạn đã hủy giao dịch." |
| Webhook duplicate | Same ref received twice | Ignore (idempotent) | - |
| Invalid signature | HMAC mismatch | Reject webhook, log | - (System alert) |
| Invariant failed | Balance doesn't match | ROLLBACK, alert admin | - (Admin intervention) |

---

### 3.2 User Withdrawal Flow

#### 3.2.1 Conditions/Requirements

┌─────────────────────────────────────────┐
│         ĐIỀU KIỆN THỰC HIỆN            │
└─────────────────────────────────────────┘

1. **Actors**: Buyer, System, Bank (optional Admin for approval)
2. **Preconditions**:
   ├── User logged in
   ├── Wallet exists, status = ACTIVE
   ├── Bank info configured
   └── available_trust >= withdrawal amount

3. **Input Requirements**:
   ├── trust_amount: Number (10 - 100,000)
   └── Bank info (if not saved)

4. **Business Rules**:
   ├── Min withdrawal: 10 Trust (10,000 VND)
   ├── Max withdrawal: 100,000 Trust (100,000,000 VND)
   ├── Daily limit: 500,000 Trust
   ├── Monthly limit: 5,000,000 Trust
   ├── Velocity: Max 5 withdrawals/day
   └── NO commission for buyer withdrawals

5. **Edge Cases**:
   ├── Insufficient balance ──► Reject
   ├── Daily limit exceeded ──► Require manual review
   ├── High fraud score ──► Require manual review
   ├── Bank transfer fails ──► Retry 3 times, then manual
   └── Validation failed ──► Unlock funds, notify user

#### 3.2.2 Flow

┌─────────────────────────────────────────┐
│      USER WITHDRAWAL FLOW               │
└─────────────────────────────────────────┘

[Bước 1] User nhấn "Rút tiền"
         │
         ▼
[Bước 2] Hệ thống hiển thị thông tin:

         ╔═══════════════════════════════════════╗
         ║  Rút tiền                           ║
         ╠═══════════════════════════════════════╣
         ║  Số dư khả dụng: 500 Trust          ║
         ║  Có thể rút: 500,000 VND            ║
         ║  ────────────────────────────────   ║
         ║  Số Trust rút: [______] Trust       ║
         ║  Min: 10 | Max: 100,000             ║
         ║                                       ║
         ║  Ngân hàng: Vietcombank ****1234     ║
         ║  [Thay đổi]                         ║
         ║                                       ║
         ║  [Hủy]          [Tiếp tục]            ║
         ╚═══════════════════════════════════════╝
         │
         ▼
[Bước 3] User nhập số Trust muốn rút
         │
         ├── Invalid (min/max/not number) ──► Show error
         ├── Valid ──► Continue
         │
         ▼
[Bước 4] Hệ thống validate:
         │
         ├── Check balance: available >= trust_amount?
         ├── ├── No ──► "Số dư không đủ"
         ├── ├── Yes ──► Continue
         │
         ├── Check daily limit: today_withdrawn + amount <= 500,000?
         ├── ├── No ──► Flag for review
         ├── ├── Yes ──► Continue
         │
         └── Check velocity: today_count < 5?
             ├── No ──► "Bạn đã đạt giới hạn rút tiền hôm nay"
             └── Yes ──► Continue
         │
         ▼
[Bước 5] Calculate:
         │
         ├── trust_amount: X Trust
         ├── commission_deduct: 0 (buyer không mất commission)
         ├── net_trust: X Trust
         └── vnd_amount: X * 1000 VND
         │
         ▼
[Bước 6] Hệ thống hiển thị xác nhận:

         ╔═══════════════════════════════════════╗
         ║  Xác nhận rút tiền                   ║
         ╠═══════════════════════════════════════╣
         ║  Số Trust rút: 100 Trust             ║
         ║  Commission: 0 Trust                 ║
         ║  Số tiền nhận: 100,000 VND           ║
         ║                                       ║
         ║  Ngân hàng: Vietcombank              ║
         ║  Số tài khoản: 1234567890            ║
         ║  Chủ tài khoản: NGUYEN VAN A         ║
         ║                                       ║
         ║  Thời gian dự kiến: 1-2 giờ          ║
         ║                                       ║
         ║  [Hủy]          [Xác nhận]           ║
         ╚═══════════════════════════════════════╝
         │
         ├── User cancels ──► Return to dashboard
         ├── User confirms ──► Continue
         │
         ▼
[Bước 7] BEGIN DATABASE TRANSACTION
         │
         ▼
[Bước 8] Tạo WithdrawalRequest:
         │
         ├── Status: PENDING
         ├── trust_amount: X
         ├── net_trust: X
         ├── vnd_amount: X * 1000
         └── Bank info
         │
         ▼
[Bước 9] Tạo Transaction:
         │
         ├── Type: WITHDRAWAL_REQUEST
         ├── Direction: DEBIT
         ├── Amount: X
         ├── balance_type: WITHDRAWAL_LOCKED
         ├── balance_before: old_available
         └── balance_after: old_available - X
         │
         ▼
[Bước 10] Update Wallet:
         │
         ├── available_trust -= X
         └── withdrawal_locked += X
         │
         ▼
[Bước 11] COMMIT
         │
         ▼
[Bước 12] Enqueue background job: validate_withdrawal
         │
         ├── Response to user: "Yêu cầu đang được xử lý"
         │
         ▼
         ─────────────────────────────────────────
         [Async Validation Flow - Background Job]
         ─────────────────────────────────────────
         │
         ▼
[V1] Update WithdrawalRequest: PENDING → VALIDATING
         │
         ▼
[V2] Run Validation Engine (xem Section 6):
         │
         ├── Check 1: Balance Integrity
         ├── Check 2: Flow Validation
         ├── Check 3: Fraud Pattern Detection
         └── Check 4: Limits
         │
         ▼
[V3] Aggregate results, calculate risk_score
         │
         ├── risk_score < 0.3 ──► PASSED (Auto-approve)
         ├── risk_score 0.3-0.7 ──► REVIEW (Manual approval)
         ├── risk_score >= 0.7 ──► FAILED (Auto-reject)
         └── Any CRITICAL check failed ──► FAILED
         │
         ├── FAILED ──► Go to [VF1]
         ├── REVIEW ──► Go to [VF2]
         ├── PASSED ──► Go to [VF3]
         │
         ▼
         ─────────────────────────────────────────
         [VF1: Validation Failed]
         ─────────────────────────────────────────
         │
         ▼
[VF1.1] Update Request: VALIDATING → VALIDATION_FAILED
         │
         ▼
[VF1.2] BEGIN TRANSACTION
         │
         ▼
[VF1.3] Tạo Transaction: WITHDRAWAL_REJECTED
         │
         ├── Direction: CREDIT
         ├── Amount: X
         └── Refund: withdrawal_locked → available
         │
         ▼
[VF1.4] Update Wallet:
         │
         ├── withdrawal_locked -= X
         └── available_trust += X
         │
         ▼
[VF1.5] COMMIT
         │
         ▼
[VF1.6] Notify user: "Yêu cầu bị từ chối: <reason>"
         │
         ▼
         END
         │
         ▼
         ─────────────────────────────────────────
         [VF2: Needs Manual Review]
         ─────────────────────────────────────────
         │
         ▼
[VF2.1] Update Request: VALIDATING → AWAITING_APPROVAL
         │
         ▼
[VF2.2] Notify admin: "New withdrawal requires review"
         │
         ├── Go to Admin Flow (Section 5.5)
         │
         ▼
         END (wait for admin decision)
         │
         ▼
         ─────────────────────────────────────────
         [VF3: Validation Passed - Auto Approve]
         ─────────────────────────────────────────
         │
         ▼
[VF3.1] Update Request: VALIDATING → APPROVED
         │
         ▼
[VF3.2] Enqueue job: process_withdrawal
         │
         ▼
         ─────────────────────────────────────────
         [Bank Transfer Flow]
         ─────────────────────────────────────────
         │
         ▼
[P1] Update Request: APPROVED → PROCESSING
         │
         ▼
[P2] Call Bank API to transfer VND
         │
         ├── Success ──► Go to [P3]
         ├── Failed ──► Retry (max 3 times)
         │   ├── Still failed after 3 retries ──► Go to [P4]
         │
         ▼
[P3] Update Request:
         │
         ├── Status: PROCESSING → COMPLETED
         ├── bank_transfer_ref: bank_transaction_id
         └── completed_at: now
         │
         ▼
[P4] BEGIN TRANSACTION
         │
         ▼
[P5] Tạo Transaction: WITHDRAWAL_COMPLETED
         │
         ├── Direction: DEBIT
         ├── Amount: X
         ├── balance_type: WITHDRAWAL_LOCKED
         └── running_withdrawn: lifetime_withdrawn + X
         │
         ▼
[P6] Update Wallet:
         │
         ├── withdrawal_locked -= X
         ├── total_trust -= X
         └── lifetime_withdrawn += X
         │
         ▼
[P7] COMMIT
         │
         ▼
[P8] Notify user: "Rút tiền thành công 100,000 VND"
         │
         ▼
         END
         │
         ▼
         ─────────────────────────────────────────
         [P4: Bank Transfer Failed]
         ─────────────────────────────────────────
         │
         ▼
[P4.1] Update Request: PROCESSING → FAILED
         │
         ▼
[P4.2] Refund funds (same as VF1)
         │
         ▼
[P4.3] Notify user: "Giao dịch thất bại. Tiền đã hoàn lại."
         │
         ▼
[P4.4] Escalate to admin for manual review
         │
         ▼
         END

#### 3.2.3 Edge Cases & Error Handling

| Case | Condition | Handling | User Message |
|------|-----------|----------|--------------|
| Insufficient balance | available < amount | Reject with 400 | "Số dư không đủ. Vui lòng nhập số tiền nhỏ hơn." |
| Daily limit exceeded | today_total + amount > 500,000 | Flag for review | "Yêu cầu cần được admin phê duyệt." |
| Monthly limit exceeded | month_total + amount > 5,000,000 | Block | "Bạn đã đạt giới hạn rút tiền tháng này." |
| Velocity exceeded | today_count >= 5 | Block | "Bạn đã đạt số lần rút tối đa trong ngày." |
| Balance mismatch | Invariant validation failed | Auto-reject, freeze wallet | "Yêu cầu bị từ chối. Vui lòng liên hệ hỗ trợ." |
| High fraud score | risk_score >= 0.7 | Auto-reject | "Yêu cầu bị từ chối. Vui lòng liên hệ hỗ trợ." |
| Bank transfer failed | API returns error | Retry 3x, then fail with refund | "Giao dịch thất bại. Tiền đã hoàn lại." |
| No bank info | Bank account not configured | Redirect to setup | "Vui lòng cấu hình tài khoản ngân hàng trước." |

---

### 3.3 User Purchase Flow

#### 3.3.1 Conditions/Requirements

┌─────────────────────────────────────────┐
│         ĐIỀU KIỆN THỰC HIỆN            │
└─────────────────────────────────────────┘

1. **Actors**: Buyer, Seller, System
2. **Preconditions**:
   ├── Buyer logged in
   ├── Buyer wallet ACTIVE
   ├── Product exists, in stock
   └── Buyer available_trust >= product_price

3. **Input Requirements**:
   ├── product_id: String
   ├── quantity: Number
   └── Shipping address (if physical)

4. **Business Rules**:
   ├── Check stock before deduct
   ├── Buyer pays full price to Platform
   ├── Platform holds in escrow (3 days)
   ├── Seller receives after escrow release
   └── Commission deducted from seller's portion

5. **Edge Cases**:
   ├── Out of stock ──► Reject purchase
   ├── Insufficient balance ──► Reject
   ├── Concurrent purchase ──► Handle race condition
   └── Dispute opened ──► Hold escrow indefinitely

#### 3.3.2 Flow

┌─────────────────────────────────────────┐
│        USER PURCHASE FLOW               │
└─────────────────────────────────────────┘

[Bước 1] User chọn sản phẩm, nhấn "Mua ngay"
         │
         ▼
[Bước 2] Hệ thống validate:
         │
         ├── Check stock: quantity >= requested?
         ├── ├── No ──► "Sản phẩm đã hết hàng"
         ├── ├── Yes ──► Continue
         │
         └── Check balance: available >= total_price?
             ├── No ──► "Số dư không đủ. Vui lòng nạp thêm tiền."
             └── Yes ──► Continue
         │
         ▼
[Bước 3] BEGIN DATABASE TRANSACTION
         │
         ▼
[Bước 4] Tạo Order:
         │
         ├── order_id: "ORD-{ULID}"
         ├── buyer_id: user_id
         ├── seller_id: product.seller_id
         ├── items: [{product_id, quantity, price}]
         ├── total_amount: total_price
         ├── payment_status: PENDING
         └── order_status: PENDING
         │
         ▼
[Bước 5] Deduct Buyer Wallet:
         │
         ├── Tạo Transaction Buyer:
         │   ├── Type: PURCHASE_DEBIT
         │   ├── Direction: DEBIT
         │   ├── Amount: total_price
         │   ├── balance_type: AVAILABLE
         │   ├── balance_before: old_balance
         │   └── balance_after: old_balance - total_price
         │
         └── Update Buyer Wallet:
             ├── available_trust -= total_price
             ├── total_trust -= total_price
             └── lifetime_spent += total_price
         │
         ▼
[Bước 6] Credit Platform Wallet (Escrow):
         │
         ├── Tạo Transaction Platform:
         │   ├── Type: ESCROW_HOLD
         │   ├── Direction: CREDIT
         │   ├── Amount: total_price
         │   ├── balance_type: AVAILABLE
         │   └── reference_id: order_id
         │
         └── Update Platform Wallet:
             ├── available_trust += total_price
             └── total_trust += total_price
         │
         ▼
[Bước 7] Tạo EscrowHold:
         │
         ├── escrow_id: "ESC-{ULID}"
         ├── order_id: order_id
         ├── seller_id: seller_id
         ├── buyer_id: buyer_id
         ├── escrow_amount: total_price
         ├── status: HOLDING
         ├── created_at: now
         └── release_at: now + 3 days
         │
         ▼
[Bước 8] Update Order:
         │
         ├── payment_status: PENDING → PAID
         └── order_status: PENDING → CONFIRMED
         │
         ▼
[Bước 9] Decrease product stock:
         │
         └── product.stock -= quantity
         │
         ▼
[Bước 10] Validate Invariants:
         │
         ├── Buyer invariant passed?
         ├── Platform invariant passed?
         └── Escrow amount matches?
         │
         ├── Any failed ──► ROLLBACK, alert admin
         ├── All passed ──► COMMIT
         │
         ▼
[Bước 11] Send notifications:
         │
         ├── To Buyer: "Đặt hàng thành công. Mã đơn: ORD-xxx"
         └── To Seller: "Bạn có đơn hàng mới. ORD-xxx"
         │
         ▼
         END

#### 3.3.3 Edge Cases & Error Handling

| Case | Condition | Handling | User Message |
|------|-----------|----------|--------------|
| Out of stock | stock < quantity | Reject with 400 | "Sản phẩm đã hết hàng." |
| Insufficient balance | available < total_price | Reject with 400 | "Số dư không đủ. Vui lòng nạp thêm." |
| Race condition | Concurrent purchase | Use optimistic lock with version | "Đã có người khác mua ngay trước đó. Vui lòng thử lại." |
| Transaction failed | DB error | Rollback all changes | "Giao dịch thất bại. Vui lòng thử lại." |
| Invariant failed | Balance mismatch | Rollback, alert admin CRITICAL | - (System alert) |

---

## 4. Seller Wallet Flows

### 4.1 Escrow Release Flow

#### 4.1.1 Conditions/Requirements

┌─────────────────────────────────────────┐
│         ĐIỀU KIỆN THỰC HIỆN            │
└─────────────────────────────────────────┘

1. **Actors**: System (Cron job), Seller, Platform
2. **Preconditions**:
   ├── EscrowHold exists with status = HOLDING
   ├── release_at <= NOW
   └── No dispute opened on order

3. **Input Requirements**:
   └── EscrowHold records (queried by cron)

4. **Business Rules**:
   ├── Escrow period: 3 days (72 hours)
   ├── Auto-release if no dispute
   ├── Commission deducted from escrow amount
   ├── Commission rate: Get from shop_commission_config (default 5%)
   └── Seller receives: escrow_amount - commission

5. **Edge Cases**:
   ├── Dispute exists ──► Skip, keep holding
   ├── Platform insufficient funds ──► CRITICAL alert
   ├── Commission calculation error ──► Log, use default
   └── Seller wallet frozen ──► Still credit, but can't withdraw

#### 4.1.2 Flow

┌─────────────────────────────────────────┐
│      ESCROW RELEASE FLOW (Cron)         │
└─────────────────────────────────────────┘

[C1] Cron runs every 5 minutes
         │
         ▼
[C2] Query EscrowHold records:
         │
         ├── WHERE status = "HOLDING"
         └── AND release_at <= NOW
         │
         ├── No records ──► END
         ├── Has records ──► Continue
         │
         ▼
[C3] Loop through each EscrowHold
         │
         ▼
[C4] Check if dispute exists:
         │
         ├── Query Dispute WHERE order_id AND status IN ("OPEN", "INVESTIGATING")
         ├── Has dispute ──► Skip this escrow, continue to next
         ├── No dispute ──► Continue
         │
         ▼
[C5] Get commission rate:
         │
         ├── Query shop_commission_config:
         │   ├── shop_id = escrow.seller_id
         │   ├── effective_from <= now
         │   ├── effective_to >= now OR NULL
         │
         ├── Not found ──► Use default: 0.05 (5%)
         ├── Found ──► Use config.rate
         │
         ▼
[C6] Calculate amounts:
         │
         ├── escrow_amount: X Trust
         ├── commission_rate: r (0.05)
         ├── commission_amount: X * r
         └── seller_receives: X - commission_amount
         │
         ├── Example:
         │   ├── escrow: 1000 Trust
         │   ├── rate: 5%
         │   ├── commission: 50 Trust
         │   └── seller receives: 950 Trust
         │
         ▼
[C7] Validate Platform Wallet:
         │
         ├── Platform.available_trust >= escrow_amount?
         ├── ├── No ──► CRITICAL ALERT, skip, continue next
         ├── ├── Yes ──► Continue
         │
         ▼
[C8] BEGIN DATABASE TRANSACTION
         │
         ▼
[C9] Deduct Platform Wallet:
         │
         ├── Tạo Transaction Platform:
         │   ├── Type: ESCROW_RELEASE
         │   ├── Direction: DEBIT
         │   ├── Amount: escrow_amount
         │   ├── reference_id: escrow_id
         │   └── balance_type: AVAILABLE
         │
         └── Update Platform Wallet:
             ├── available_trust -= escrow_amount
             └── total_trust -= escrow_amount
         │
         ▼
[C10] Credit Seller Wallet:
         │
         ├── Tạo Transaction Seller:
         │   ├── Type: ESCROW_RELEASE
         │   ├── Direction: CREDIT
         │   ├── Amount: seller_receives
         │   ├── reference_id: escrow_id
         │   └── balance_type: AVAILABLE
         │
         └── Update Seller Wallet:
             ├── available_trust += seller_receives
             ├── total_trust += seller_receives
             └── lifetime_received += seller_receives
         │
         ▼
[C11] Record Commission Debt:
         │
         ├── Tạo Transaction Seller:
         │   ├── Type: COMMISSION_ACCRUE
         │   ├── Amount: commission_amount
         │   └── reference_id: escrow_id
         │
         └── Update Seller Wallet:
             └── commission_debt += commission_amount
         │
         ▼
[C12] Record Commission Collected:
         │
         ├── Tạo Transaction Platform:
         │   ├── Type: COMMISSION_COLLECTED
         │   ├── Direction: CREDIT
         │   ├── Amount: commission_amount
         │   └── reference_id: escrow_id
         │
         └── Update Platform Wallet:
             ├── available_trust += commission_amount
             └── total_trust += commission_amount
         │
         ▼
[C13] Update EscrowHold:
         │
         ├── status: HOLDING → RELEASED
         └── released_at: now
         │
         ▼
[C14] Update Order:
         │
         ├── order_status: CONFIRMED → COMPLETED
         └── completed_at: now
         │
         ▼
[C15] Validate Invariants:
         │
         ├── Platform invariant OK?
         ├── Seller invariant OK?
         └── Commission amounts match?
         │
         ├── Any failed ──► ROLLBACK, alert admin
         ├── All passed ──► COMMIT
         │
         ▼
[C16] Notify Seller:
         │
         ├── "Đơn hàng ORD-xxx đã hoàn thành"
         ├── "Bạn nhận được: 950 Trust"
         └── "Commission: 50 Trust (đã ghi nợ)"
         │
         ▼
[C17] Move to next EscrowHold
         │
         └── Loop back to [C4]
         │
         ▼
         END

#### 4.1.3 Edge Cases & Error Handling

| Case | Condition | Handling | User Message |
|------|-----------|----------|--------------|
| Dispute exists | Dispute.status IN (OPEN, INVESTIGATING) | Skip release, keep holding | - (Seller notified when dispute resolves) |
| Platform insufficient | Platform.available < escrow_amount | CRITICAL alert, skip | - (Admin intervention required) |
| Seller wallet frozen | Wallet.status = FROZEN | Still credit, but can't withdraw | "Tiền đã nhận nhưng ví đang bị khóa." |
| Config not found | No commission config for shop | Use default 5% | - (System uses default) |
| Transaction failed | DB error | Rollback, retry next cron | - (Automatic retry) |

---

### 4.2 Seller Withdrawal Flow

#### 4.2.1 Conditions/Requirements

┌─────────────────────────────────────────┐
│         ĐIỀU KIỆN THỰC HIỆN            │
└─────────────────────────────────────────┘

1. **Actors**: Seller, System, Bank, Admin (for approval)
2. **Preconditions**:
   ├── Seller logged in
   ├── Wallet exists, status = ACTIVE
   ├── Bank info configured
   └── available_trust >= withdrawal amount

3. **Input Requirements**:
   ├── trust_amount: Number (10 - 100,000)
   └── Bank info (if not saved)

4. **Business Rules**:
   ├── Min: 10 Trust, Max: 100,000 Trust
   ├── Daily limit: 500,000 Trust
   ├── Monthly limit: 5,000,000 Trust
   ├── Velocity: Max 5/day
   ├── **COMMISSION APPLIED**: Deduct from commission_debt
   └── commission_deduct = min(amount * rate, commission_debt)

5. **Edge Cases**:
   ├── Commission debt = 0 ──► No commission deducted
   ├── Commission debt < calculated ──► Deduct actual debt only
   ├── Commission debt > calculated ──► Deduct calculated, debt remains
   └── All validation same as buyer withdrawal

#### 4.2.2 Flow

┌─────────────────────────────────────────┐
│      SELLER WITHDRAWAL FLOW             │
└─────────────────────────────────────────┘

[Bước 1] Seller nhấn "Rút tiền"
         │
         ▼
[Bước 2] Hệ thống hiển thị thông tin:

         ╔═══════════════════════════════════════════╗
         ║  Rút tiền                              ║
         ╠═══════════════════════════════════════════╣
         ║  Số dư khả dụng: 1,000 Trust           ║
         ║  Commission nợ: 50 Trust               ║
         ║  ───────────────────────────────────   ║
         ║  Ước tính nhận: 950,000 VND            ║
         ║                                         ║
         ║  Số Trust rút: [______] Trust          ║
         ║  Min: 10 | Max: 100,000                ║
         ║                                         ║
         ║  Ngân hàng: Vietcombank ****1234       ║
         ║  [Thay đổi]                           ║
         ║                                         ║
         ║  [Hủy]            [Tiếp tục]           ║
         ╚═══════════════════════════════════════════╝
         │
         ▼
[Bước 3] Seller nhập số Trust muốn rút
         │
         ├── Invalid ──► Show error
         ├── Valid ──► Continue
         │
         ▼
[Bước 4] Get commission rate:
         │
         ├── Query shop_commission_config
         ├── Not found ──► Use default 5%
         ├── Found ──► Use config.rate
         │
         ▼
[Bước 5] Calculate commission:
         │
         ├── trust_amount: X Trust (VD: 500)
         ├── commission_rate: r (VD: 0.05)
         ├── calculated_commission: X * r = 25 Trust
         ├── commission_debt: 50 Trust
         ├── commission_deduct: min(25, 50) = 25 Trust
         ├── net_trust: 500 - 25 = 475 Trust
         └── vnd_amount: 475 * 1000 = 475,000 VND
         │
         ▼
[Bước 6] Hệ thống hiển thị xác nhận:

         ╔═══════════════════════════════════════════╗
         ║  Xác nhận rút tiền                     ║
         ╠═══════════════════════════════════════════╣
         ║  Số Trust rút: 500 Trust                ║
         ║  Commission: 25 Trust (5%)              ║
         ║  Số tiền nhận: 475,000 VND              ║
         ║                                         ║
         ║  Ngân hàng: Vietcombank                 ║
         ║  Số tài khoản: 1234567890               ║
         ║  Chủ tài khoản: NGUYEN VAN A            ║
         ║                                         ║
         ║  Thời gian: 1-2 giờ                    ║
         ║                                         ║
         ║  [Hủy]            [Xác nhận]           ║
         ╚═══════════════════════════════════════════╝
         │
         ├── Seller confirms ──► Continue
         │
         ▼
[Bước 7] BEGIN TRANSACTION
         │
         ▼
[Bước 8] Tạo WithdrawalRequest:
         │
         ├── Status: PENDING
         ├── trust_amount: 500
         ├── commission_deduct: 25
         ├── net_trust: 475
         └── vnd_amount: 475,000
         │
         ▼
[Bước 9] Lock funds:
         │
         ├── Tạo Transaction: WITHDRAWAL_REQUEST
         ├── Move: available (-500) → withdrawal_locked (+500)
         └── COMMIT
         │
         ▼
[Bước 10] Enqueue validation job
         │
         └── (Same as buyer withdrawal validation flow)
         │
         ▼
         ─────────────────────────────────────────
         [After validation passed and bank transfer success]
         ─────────────────────────────────────────
         │
         ▼
[Final] BEGIN TRANSACTION
         │
         ▼
[Final.1] Tạo Transaction: WITHDRAWAL_COMPLETED
         │
         ├── Direction: DEBIT
         ├── Amount: 500 (full trust_amount)
         └── balance_type: WITHDRAWAL_LOCKED
         │
         ▼
[Final.2] Update Seller Wallet:
         │
         ├── withdrawal_locked -= 500
         ├── total_trust -= 500
         ├── lifetime_withdrawn += 500
         └── commission_debt -= 25
         │
         ▼
[Final.3] Commission to Platform:
         │
         ├── Tạo Transaction Platform:
         │   ├── Type: COMMISSION_COLLECTED
         │   ├── Direction: CREDIT
         │   └── Amount: 25
         │
         └── Update Platform Wallet:
             ├── available_trust += 25
             └── total_trust += 25
         │
         ▼
[Final.4] COMMIT
         │
         ▼
[Final.5] Notify Seller:
         │
         ├── "Rút tiền thành công: 475,000 VND"
         └── "Commission đã trả: 25 Trust"
         │
         ▼
         END

#### 4.2.3 Edge Cases & Error Handling

| Case | Condition | Handling | User Message |
|------|-----------|----------|--------------|
| Commission debt = 0 | commission_debt = 0 | No commission deducted | "Không có commission nợ. Rút đầy đủ." |
| Partial debt | commission_debt < calculated | Deduct actual debt only | "Commission: 25 Trust (nợ còn 25 Trust)" |
| Full debt coverage | commission_debt >= calculated | Deduct calculated, debt remains | "Commission: 25 Trust (nợ còn 25 Trust)" |
| Other validations | Same as buyer | Same as buyer | Same as buyer |

---

## 5. Admin Wallet Flows

### 5.1 Admin Dashboard Overview

#### 5.1.1 Dashboard Layout

```
┌─────────────────────────────────────────────────────────────────┐
│                    ADMIN WALLET DASHBOARD                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │ TỔNG TRUST      │  │ ĐANG HOLD       │  │ COMMISSION      │  │
│  │ TRONG HỆ THỐNG  │  │ (ESCROW)        │  │ ĐÃ THU          │  │
│  │                 │  │                 │  │                 │  │
│  │ 50,000,000     │  │ 5,000,000      │  │ 2,500,000      │  │
│  │ Trust          │  │ Trust          │  │ Trust          │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘  │
│                                                                  │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ HOLD BY SHOP                                               │  │
│  ├───────────────────────────────────────────────────────────┤  │
│  │ Shop               | Hold Amount | Status    | Action     │  │
│  │ -------------------|-------------|-----------|----------- │  │
│  │ Shop A             | 2,000,000   | NORMAL    | [View]     │  │
│  │ Shop B             | 1,500,000   | NORMAL    | [View]     │  │
│  │ Shop C (⚠️)        | 1,000,000   | DISPUTE   | [Review]   │  │
│  │ Shop D             | 500,000     | NORMAL    | [View]     │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                  │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ PENDING WITHDRAWALS (Chờ duyệt)                           │  │
│  ├───────────────────────────────────────────────────────────┤  │
│  │ User     | Amount    | Risk Score | Status         | Act  │  │
│  │ ---------|-----------|------------|----------------|----- │  │
│  │ seller_1 | 10,000 T  | 0.45       | AWAITING_REVIEW| [✓][✗]│
│  │ user_2   | 50,000 T  | 0.65       | AWAITING_REVIEW| [✓][✗]│
│  └───────────────────────────────────────────────────────────┘  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 5.2 Admin Manual Deposit Flow

#### 5.2.1 Conditions/Requirements

┌─────────────────────────────────────────┐
│         ĐIỀU KIỆN THỰC HIỆN            │
└─────────────────────────────────────────┘

1. **Actors**: Admin, User (recipient)
2. **Preconditions**:
   ├── Admin logged in with WALLET_DEPOSIT permission
   └── Target user/seller exists

3. **Input Requirements**:
   ├── target_user_id: String
   ├── trust_amount: Number (1 - 1,000,000)
   └── reason: String (min 10 chars, required)

4. **Business Rules**:
   ├── Min: 1 Trust
   ├── Max: 1,000,000 Trust per transaction
   ├── Reason required (audit trail)
   └── Supervisor approval if amount > 100,000 Trust

5. **Edge Cases**:
   ├── Wallet not exist ──► Auto-create wallet first
   ├── Wallet frozen ──► Still credit, but remains frozen
   ├── Target is platform ──► Allow (for funding platform)
   └── Large amount ──► Require supervisor approval

#### 5.2.2 Flow

┌─────────────────────────────────────────┐
│      ADMIN MANUAL DEPOSIT FLOW          │
└─────────────────────────────────────────┘

[A1] Admin chọn "Nạp tiền thủ công"
         │
         ▼
[A2] Hệ thống hiển thị form:

         ╔═══════════════════════════════════════╗
         ║  Nạp tiền thủ công                   ║
         ╠═══════════════════════════════════════╣
         ║  User/Seller ID: [______________]     ║
         ║  [Tìm kiếm]                          ║
         ║                                       ║
         ║  Số Trust: [______] Trust            ║
         ║  Min: 1 | Max: 1,000,000             ║
         ║                                       ║
         ║  Lý do: [_____________________]      ║
         ║  (Bắt buộc, tối thiểu 10 ký tự)      ║
         ║                                       ║
         ║  Ghi chú (tùy chọn):                 ║
         ║  [_____________________________]    ║
         ║                                       ║
         ║  [Hủy]          [Xác nhận]           ║
         ╚═══════════════════════════════════════╝
         │
         ▼
[A3] Admin nhập thông tin
         │
         ├── Validate:
         │   ├── User exists?
         │   ├── Amount valid?
         │   └── Reason >= 10 chars?
         │
         ├── Invalid ──► Show error
         ├── Valid ──► Continue
         │
         ▼
[A4] Check permission:
         │
         ├── Admin has WALLET_DEPOSIT permission?
         ├── ├── No ──► "Bạn không có quyền thực hiện"
         ├── ├── Yes ──► Continue
         │
         ▼
[A5] Check amount threshold:
         │
         ├── amount > 100,000 Trust?
         ├── ├── Yes ──► Require supervisor approval
         ├── ├── No ──► Continue to [A7]
         │
         ▼
         ─────────────────────────────────────────
         [Supervisor Approval Flow]
         ─────────────────────────────────────────
         │
         ▼
[A6.1] Send approval request to supervisor:
         │
         ├── "Admin X muốn nạp 150,000 Trust cho user Y"
         ├── "Lý do: <reason>"
         └── Supervisor must approve via 2FA
         │
         ├── Supervisor rejects ──► END, notify admin
         ├── Supervisor approves ──► Continue to [A7]
         │
         ▼
         ─────────────────────────────────────────
         [Main Flow Continues]
         ─────────────────────────────────────────
         │
         ▼
[A7] Admin confirms deposit
         │
         ├── Cancel ──► END
         ├── Confirm ──► Continue
         │
         ▼
[A8] BEGIN TRANSACTION
         │
         ▼
[A9] Get or create target wallet:
         │
         ├── Query Wallet WHERE user_id = target_user_id
         ├── Not exist ──► Create new wallet (status: ACTIVE)
         ├── Exists ──► Use existing
         │
         ▼
[A10] Tạo Transaction:
         │
         ├── Type: ADMIN_CREDIT
         ├── Direction: CREDIT
         ├── Amount: trust_amount
         ├── initiated_by: admin_id
         ├── admin_note: reason
         ├── balance_before: old_balance
         └── balance_after: old_balance + trust_amount
         │
         ▼
[A11] Update Target Wallet:
         │
         ├── available_trust += trust_amount
         ├── total_trust += trust_amount
         └── lifetime_deposited += trust_amount
         │
         ▼
[A12] Tạo AdminOperationLog:
         │
         ├── operation: MANUAL_DEPOSIT
         ├── admin_id, admin_email, admin_role
         ├── target_type: "WALLET"
         ├── target_id: wallet_id
         ├── before_state: { available_trust: old }
         ├── after_state: { available_trust: new }
         ├── amount: trust_amount
         └── reason: reason
         │
         ▼
[A13] Validate Invariant:
         │
         ├── new_balance == old_balance + trust_amount?
         ├── Passed ──► COMMIT
         ├── Failed ──► ROLLBACK, alert supervisor
         │
         ▼
[A14] Send notifications:
         │
         ├── To target user: "Admin đã nạp X Trust vào ví của bạn. Lý do: <reason>"
         └── To supervisor (if amount > threshold): "Admin X đã nạp X Trust"
         │
         ▼
         END

#### 5.2.3 Edge Cases & Error Handling

| Case | Condition | Handling | Admin Message |
|------|-----------|----------|---------------|
| User not found | user_id doesn't exist | Return 404 | "User không tồn tại." |
| Wallet not exist | No wallet for user | Auto-create wallet | "Đã tạo wallet mới cho user." |
| Permission denied | No WALLET_DEPOSIT | Return 403 | "Bạn không có quyền thực hiện." |
| Supervisor rejected | Supervisor 2FA reject | Don't process | "Supervisor đã từ chối." |
| Invariant failed | Balance mismatch | Rollback, alert | - (System alert) |

### 5.3 Admin Manual Debit Flow

#### 5.3.1 Conditions/Requirements

┌─────────────────────────────────────────┐
│         ĐIỀU KIỆN THỰC HIỆN            │
└─────────────────────────────────────────┘

1. **Actors**: Admin, Supervisor (if large amount), User (target)
2. **Preconditions**:
   ├── Admin logged in with WALLET_DEBIT permission
   ├── Target wallet exists
   └── Wallet has sufficient balance

3. **Input Requirements**:
   ├── target_user_id: String
   ├── trust_amount: Number
   ├── reason: String (min 10 chars, required)
   └── reference_id: String (order_id, etc.)

4. **Business Rules**:
   ├── Cannot debit more than available_trust
   ├── Amount > 10,000 Trust requires supervisor approval
   ├── Reason required (audit trail)
   └── Reference recommended for traceability

5. **Edge Cases**:
   ├── Insufficient balance ──► Show max debitable
   ├── Wallet frozen ──► Allow, but log warning
   ├── Negative balance attempt ──► Prevent
   └── Zero balance ──► Allow (no effect)

#### 5.3.2 Flow

┌─────────────────────────────────────────┐
│      ADMIN MANUAL DEBIT FLOW            │
└─────────────────────────────────────────┘

[A1] Admin chọn "Trừ tiền"
         │
         ▼
[A2] Admin nhập target user_id
         │
         ▼
[A3] Hệ thống hiển thị wallet info:

         ╔═══════════════════════════════════════════╗
         ║  Trừ tiền                              ║
         ╠═══════════════════════════════════════════╣
         ║  User: user_123 (nguyenvan@email.com)   ║
         ║  Wallet: WLT-ABC123                      ║
         ║  Status: ACTIVE                         ║
         ║  ───────────────────────────────────   ║
         ║  Số dư khả dụng: 5,000 Trust            ║
         ║  Đang khóa rút: 0 Trust                 ║
         ║  Đang tranh chấp: 0 Trust               ║
         ║  Tổng: 5,000 Trust                      ║
         ║  ───────────────────────────────────   ║
         ║  Có thể trừ tối đa: 5,000 Trust         ║
         ║                                         ║
         ║  Số Trust trừ: [______] Trust           ║
         ║                                         ║
         ║  Lý do: [_________________________]    ║
         ║  (Bắt buộc)                            ║
         ║                                         ║
         ║  Reference: [__________________]       ║
         ║  (VD: order_id, transaction_id)        ║
         ║                                         ║
         ║  Ghi chú: [________________________]   ║
         ║                                         ║
         ║  [Hủy]            [Tiếp tục]           ║
         ╚═══════════════════════════════════════════╝
         │
         ▼
[A4] Admin enters amount and reason
         │
         ├── Validate:
         │   ├── amount <= available_trust?
         │   ├── amount > 0?
         │   └── reason >= 10 chars?
         │
         ├── Invalid ──► Show error
         ├── Valid ──► Continue
         │
         ▼
[A5] Check permission:
         │
         ├── Admin has WALLET_DEBIT permission?
         ├── ├── No ──► Return 403
         ├── ├── Yes ──► Continue
         │
         ▼
[A6] Check amount threshold:
         │
         ├── amount > 10,000 Trust?
         ├── ├── Yes ──► Require supervisor approval
         ├── ├── No ──► Continue to [A8]
         │
         ▼
         ─────────────────────────────────────────
         [Supervisor Approval]
         ─────────────────────────────────────────
         │
         ▼
[A7] Supervisor approves via 2FA
         │
         ├── Reject ──► END, notify admin
         ├── Approve ──► Continue
         │
         ▼
         ─────────────────────────────────────────
         [Main Flow]
         ─────────────────────────────────────────
         │
         ▼
[A8] Admin confirms debit
         │
         ├── Cancel ──► END
         ├── Confirm ──► Continue
         │
         ▼
[A9] BEGIN TRANSACTION
         │
         ▼
[A10] Tạo Transaction:
         │
         ├── Type: ADMIN_DEBIT
         ├── Direction: DEBIT
         ├── Amount: trust_amount
         ├── initiated_by: admin_id
         ├── admin_note: reason
         └── reference_id: reference_id
         │
         ▼
[A11] Update Target Wallet:
         │
         ├── available_trust -= trust_amount
         └── total_trust -= trust_amount
         │
         ▼
[A12] Tạo AdminOperationLog:
         │
         ├── operation: MANUAL_DEBIT
         ├── before_state: { available_trust: old }
         ├── after_state: { available_trust: new }
         ├── amount, reason, reference
         └── admin_id, admin_email
         │
         ▼
[A13] Validate Invariant
         │
         ├── Passed ──► COMMIT
         ├── Failed ──► ROLLBACK
         │
         ▼
[A14] Notify user:
         │
         ├── "Tài khoản của bạn bị trừ X Trust"
         ├── "Lý do: <reason>"
         └── "Liên hệ hỗ trợ nếu có thắc mắc"
         │
         ▼
         END

### 5.4 Admin Commission Setup Flow

#### 5.4.1 Conditions/Requirements

┌─────────────────────────────────────────┐
│         ĐIỀU KIỆN THỰC HIỆN            │
└─────────────────────────────────────────┘

1. **Actors**: Admin
2. **Preconditions**:
   ├── Admin logged in with COMMISSION_MANAGE permission
   └── Target shop exists

3. **Input Requirements**:
   ├── shop_id: String
   ├── rate: f64 (0.01 - 0.20, i.e., 1% - 20%)
   ├── effective_from: DateTime
   ├── effective_to: DateTime (nullable)
   └── reason: String

4. **Business Rules**:
   ├── Rate range: 1% - 20%
   ├── effective_from cannot be in past
   ├── effective_to optional (NULL = indefinite)
   ├── Only one active config per shop at a time
   └── Old configs auto-deactivated when new one created

5. **Edge Cases**:
   ├── Overlapping dates ──► Deactivate old, activate new
   ├── Invalid rate ──► Reject
   ├── Shop not found ──► Return 404

#### 5.4.2 Flow

┌─────────────────────────────────────────┐
│    ADMIN COMMISSION SETUP FLOW          │
└─────────────────────────────────────────┘

[A1] Admin selects "Cài đặt Commission"
         │
         ▼
[A2] System shows all shops with current rates:

         ╔═════════════════════════════════════════════════╗
         ║  Cài đặt Commission theo Shop                   ║
         ╠═════════════════════════════════════════════════╣
         ║  Shop                  | Rate      | Action    ║
         ║  ----------------------|-----------|---------- ║
         ║  Shop A (Gaming)       | 5% (default)| [Chỉnh] ║
         ║  Shop B (Accounts)     | 3% (custom) | [Chỉnh] ║
         ║  Shop C (Services)     | 5% (default)| [Chỉnh] ║
         ║  Shop D (Digital)      | 10% (custom)| [Chỉnh] ║
         ║                                       ║
         ║  [Tạo mới cho shop khác]               ║
         ╚═════════════════════════════════════════════════╝
         │
         ▼
[A3] Admin clicks "Chỉnh" on a shop
         │
         ▼
[A4] System shows form:

         ╔═════════════════════════════════════════════════╗
         ║  Cài đặt Commission - Shop B                   ║
         ╠═════════════════════════════════════════════════╣
         ║  Shop: Shop B (Accounts Shop)                  ║
         ║  Rate hiện tại: 3% (hiệu lực từ 2025-01-01)    ║
         ║  ──────────────────────────────────────────   ║
         ║  Rate mới (%): [____] (1 - 20)                ║
         ║                                       ║
         ║  Hiệu lực từ: [__/__/____]                   ║
         ║  Hiệu lực đến: [__/__/____] (optional)       ║
         ║                                       ║
         ║  Lý do thay đổi:                           ║
         ║  [____________________________________]     ║
         ║                                       ║
         ║  [Hủy]            [Lưu]                ║
         ╚═════════════════════════════════════════════════╝
         │
         ▼
[A5] Admin enters new rate
         │
         ├── Validate:
         │   ├── Rate in range 1-20?
         │   ├── effective_from >= today?
         │   └── Reason provided?
         │
         ├── Invalid ──► Show error
         ├── Valid ──► Show impact preview
         │
         ▼
[A6] System shows impact:

         ╔═════════════════════════════════════════════════╗
         ║  Xác nhận thay đổi Commission                 ║
         ╠═════════════════════════════════════════════════╣
         ║  Shop B - Accounts Shop                       ║
         ║  ──────────────────────────────────────────   ║
         ║  Rate cũ: 3%                                 ║
         ║  Rate mới: 5%                                ║
         ║                                       ║
         ║  Ước tính tác động:                          ║
         ║  • Số orders/tháng: ~100                     ║
         ║  • Giá trị trung bình/order: 1,000 Trust     ║
         ║  • Tổng giá trị: 100,000 Trust/tháng        ║
         ║                                       ║
         ║  Commission cũ: 3,000 Trust/tháng            ║
         ║  Commission mới: 5,000 Trust/tháng            ║
         ║  Chênh lệch: +2,000 Trust/tháng              ║
         ║                                       ║
         ║  [Hủy]            [Xác nhận thay đổi]       ║
         ╚═════════════════════════════════════════════════╝
         │
         ▼
[A7] Admin confirms
         │
         ├── Cancel ──► END
         ├── Confirm ──► Continue
         │
         ▼
[A8] BEGIN TRANSACTION
         │
         ▼
[A9] Deactivate old config:
         │
         ├── Query shop_commission_config
         ├── WHERE shop_id = X AND effective_to IS NULL
         └── UPDATE effective_to = now
         │
         ▼
[A10] Create new config:
         │
         ├── Insert shop_commission_config:
         │   ├── shop_id: shop_id
         │   ├── rate: new_rate
         │   ├── effective_from: effective_from
         │   ├── effective_to: effective_to (or NULL)
         │   └── created_by: admin_id
         │
         ▼
[A11] Tạo AdminOperationLog:
         │
         ├── operation: COMMISSION_OVERRIDE
         ├── target_type: "SHOP"
         ├── target_id: shop_id
         ├── before_state: { rate: old_rate }
         ├── after_state: { rate: new_rate }
         └── reason: reason
         │
         ▼
[A12] COMMIT
         │
         ▼
[A13] Notify shop owner:
         │
         ├── "Commission rate đã thay đổi"
         ├── "Từ: old_rate → new_rate"
         └── "Hiệu lực từ: effective_from"
         │
         ▼
         END

### 5.5 Admin Withdrawal Review Flow

#### 5.5.1 Conditions/Requirements

┌─────────────────────────────────────────┐
│         ĐIỀU KIỆN THỰC HIỆN            │
└─────────────────────────────────────────┘

1. **Actors**: Admin
2. **Preconditions**:
   ├── Admin logged in with WITHDRAWAL_APPROVE permission
   └── WithdrawalRequest with status = AWAITING_APPROVAL

3. **Input Requirements**:
   └── Decision: APPROVE | REJECT | HOLD

4. **Business Rules**:
   ├── Must review validation results before deciding
   ├── Reject requires reason
   ├── Hold requires reason and investigation ticket
   └── All actions logged to AdminOperationLog

5. **Edge Cases**:
   ├── Already processed ──► Show error
   ├── Withdrawal expired ──► Auto-cancel
   └── User account frozen ──► Flag for review

#### 5.5.2 Flow

┌─────────────────────────────────────────┐
│     ADMIN WITHDRAWAL REVIEW FLOW        │
└─────────────────────────────────────────┘

[A1] Admin selects "Pending Withdrawals"
         │
         ▼
[A2] System shows list:

         ╔═════════════════════════════════════════════════╗
         ║  Withdrawals chờ duyệt                        ║
         ╠═════════════════════════════════════════════════╣
         ║  User     | Amount | Risk | Status    | Action ║
         ║  ---------|--------|------|-----------|-------- ║
         ║  seller_1 | 10K T  | 0.45 | AWAITING  | [Xem]  ║
         ║  user_2   | 50K T  | 0.65 | AWAITING  | [Xem]  ║
         ║  seller_3 | 5K T   | 0.25 | AWAITING  | [Xem]  ║
         ╚═════════════════════════════════════════════════╝
         │
         ▼
[A3] Admin clicks "Xem" on a withdrawal
         │
         ▼
[A4] System shows details:

         ╔═════════════════════════════════════════════════╗
         ║  Chi tiết Withdrawal Request                  ║
         ╠═════════════════════════════════════════════════╣
         ║  Request ID: WD-ABC123                        ║
         ║  User: seller_1 (email@example.com)           ║
         ║  Wallet: WLT-XYZ789                           ║
         ║  ──────────────────────────────────────────   ║
         ║  Amount: 10,000 Trust                         ║
         ║  Commission: 500 Trust                        ║
         ║  Net: 9,500,000 VND                           ║
         ║  ──────────────────────────────────────────   ║
         ║  Bank: Vietcombank                           ║
         ║  Account: ****1234                           ║
         ║  Name: NGUYEN VAN A                          ║
         ║  ──────────────────────────────────────────   ║
         ║  VALIDATION RESULTS:                         ║
         ║  ✓ Balance Check: PASS                       ║
         ║  ✓ Flow Check: PASS                          ║
         ║  ⚠ Fraud Check: WARNING                      ║
         ║    - First withdrawal                        ║
         ║    - Large amount                            ║
         ║  ✓ Limit Check: PASS                         ║
         ║  ──────────────────────────────────────────   ║
         ║  RISK SCORE: 0.45 (MEDIUM)                   ║
         ║  ──────────────────────────────────────────   ║
         ║  RECENT TRANSACTIONS:                        ║
         ║  | Jan 1 | ESCROW_RELEASE | +950 T | ✓     | ║
         ║  | Jan 1 | ESCROW_RELEASE | +475 T | ✓     | ║
         ║  | Dec 31| WITHDRAWAL     | -5000 T| ✓     | ║
         ║                                       ║
         ║  [Approve] [Reject] [Hold]                 ║
         ╚═════════════════════════════════════════════════╝
         │
         ▼
[A5] Admin makes decision
         │
         ├── Approve ──► Go to [A6]
         ├── Reject ──► Go to [A7]
         ├── Hold ──► Go to [A8]
         │
         ▼
         ─────────────────────────────────────────
         [Approve Path]
         ─────────────────────────────────────────
         │
         ▼
[A6.1] Check permission:
         │
         ├── Admin has WITHDRAWAL_APPROVE?
         ├── ├── No ──► Return 403
         ├── ├── Yes ──► Continue
         │
         ▼
[A6.2] Confirm approve:
         │
         ├── "Bạn có chắc muốn phê duyệt?"
         ├── Cancel ──► Return
         ├── Confirm ──► Continue
         │
         ▼
[A6.3] BEGIN TRANSACTION
         │
         ▼
[A6.4] Update WithdrawalRequest:
         │
         ├── status: AWAITING_APPROVAL → APPROVED
         ├── approved_by: admin_id
         └── approved_at: now
         │
         ▼
[A6.5] Tạo AdminOperationLog:
         │
         ├── operation: WITHDRAWAL_APPROVE
         ├── target_id: request_id
         └── admin_id, admin_email
         │
         ▼
[A6.6] COMMIT
         │
         ▼
[A6.7] Enqueue process_withdrawal job
         │
         ▼
[A6.8] Notify user:
         │
         └── "Yêu cầu rút tiền đã được duyệt. Đang xử lý."
         │
         ▼
         END
         │
         ▼
         ─────────────────────────────────────────
         [Reject Path]
         ─────────────────────────────────────────
         │
         ▼
[A7.1] Admin enters reject reason
         │
         ├── Form: "Lý do từ chối: [_____________]"
         ├── Required
         │
         ▼
[A7.2] BEGIN TRANSACTION
         │
         ▼
[A7.3] Update WithdrawalRequest:
         │
         ├── status: AWAITING_APPROVAL → REJECTED
         └── reject_reason: reason
         │
         ▼
[A7.4] Unlock funds:
         │
         ├── Tạo Transaction: WITHDRAWAL_REJECTED
         ├── Move: withdrawal_locked → available
         └── Update Wallet
         │
         ▼
[A7.5] Tạo AdminOperationLog:
         │
         ├── operation: WITHDRAWAL_REJECT
         ├── target_id: request_id
         └── reason: reason
         │
         ▼
[A7.6] COMMIT
         │
         ▼
[A7.7] Notify user:
         │
         └── "Yêu cầu rút tiền bị từ chối. Lý do: <reason>"
         │
         ▼
         END
         │
         ▼
         ─────────────────────────────────────────
         [Hold Path]
         ─────────────────────────────────────────
         │
         ▼
[A8.1] Admin enters hold reason
         │
         ├── Form: "Lý do hold: [_____________]"
         ├── Required
         │
         ▼
[A8.2] Update WithdrawalRequest:
         │
         ├── status: AWAITING_APPROVAL → HOLD
         ├── hold_reason: reason
         ├── hold_at: now
         └── hold_by: admin_id
         │
         ▼
[A8.3] Note: Funds REMAIN in withdrawal_locked
         │
         ▼
[A8.4] Tạo AdminOperationLog
         │
         ▼
[A8.5] Tạo Investigation Ticket:
         │
         ├── type: WITHDRAWAL_HOLD
         ├── priority: HIGH (if amount > 50K)
         ├── priority: MEDIUM (if amount <= 50K)
         └── assigned_to: security_team
         │
         ▼
[A8.6] Notify user:
         │
         └── "Yêu cầu đang được xem xét. Vui lòng chờ 1-3 ngày làm việc."
         │
         ▼
[A8.7] Notify security team:
         │
         └── "New hold case: WD-ABC123"
         │
         ▼
         END (wait for investigation)

---

## 6. Validation Engine

### 6.1 Validation Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    VALIDATION ENGINE                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Input: withdrawal_request, wallet                              │
│                                                                  │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐         │
│  │   Check 1   │    │   Check 2   │    │   Check 3   │         │
│  │  Balance    │───→│   Flow      │───→│   Fraud     │         │
│  │  Integrity  │    │  Validation │    │   Pattern   │         │
│  └─────────────┘    └─────────────┘    └─────────────┘         │
│        │                  │                  │                  │
│        ↓                  ↓                  ↓                  │
│  ┌─────────────────────────────────────────────────────┐       │
│  │              Check 4: Limits                        │       │
│  └─────────────────────────────────────────────────────┘       │
│                          │                                      │
│                          ↓                                      │
│  ┌─────────────────────────────────────────────────────┐       │
│  │              Aggregate Results                       │       │
│  │              Calculate Risk Score                    │       │
│  │              Determine: PASS / REVIEW / REJECT       │       │
│  └─────────────────────────────────────────────────────┘       │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 6.2 Check 1: Balance Integrity (Monthly Snapshot)

#### 6.2.1 Conditions/Requirements

┌─────────────────────────────────────────┐
│         VALIDATION LOGIC                │
└─────────────────────────────────────────┘

1. **Purpose**: Verify wallet balance matches transaction history
2. **Method**: Incremental calculation from monthly snapshot
3. **Performance**: Only query current month transactions
4. **Severity**: CRITICAL if mismatch > 100 Trust

#### 6.2.2 Flow

┌─────────────────────────────────────────┐
│     BALANCE INTEGRITY CHECK             │
└─────────────────────────────────────────┘

[V1] Get MonthlySnapshot:
         │
         ├── Query: WHERE wallet_id = X AND month = previous_month
         ├── Found ──► Use snapshot
         ├── Not found ──► Calculate from all transactions (slower)
         │
         ▼
[V2] If snapshot exists:
         │
         ├── last_snapshot_balance: X (from snapshot)
         ├── Query transactions WHERE created_at >= month_start AND status = COMPLETED
         ├── Calculate delta:
         │   ├── delta_credit = Σ(amount WHERE direction = CREDIT)
         │   └── delta_debit = Σ(amount WHERE direction = DEBIT)
         │
         └── expected_balance = last_snapshot_balance + delta_credit - delta_debit
         │
         ▼
[V3] Compare:
         │
         ├── expected_balance == wallet.total_trust?
         │
         ├── Yes ──► PASS
         │
         ├── No ──► Calculate discrepancy:
         │   ├── discrepancy = expected_balance - actual_balance
         │   ├── abs(discrepancy) > 100?
         │   │   ├── Yes ──► CRITICAL FAIL
         │   │   └── No ──► WARNING (minor discrepancy)
         │
         ▼
[V4] Return CheckResult:
         │
         ├── passed: true/false
         ├── details: "Balance verified" or "Discrepancy: X Trust"
         └── severity: INFO / WARNING / CRITICAL

**SQL Query**:
```sql
SELECT
    SUM(CASE WHEN direction = 'CREDIT' THEN amount ELSE 0 END) as total_credits,
    SUM(CASE WHEN direction = 'DEBIT' THEN amount ELSE 0 END) as total_debits,
    COUNT(*) as tx_count
FROM transactions
WHERE wallet_id = :wallet_id
  AND created_at >= :month_start
  AND status = 'COMPLETED';
```

### 6.3 Check 2: Flow Validation

#### 6.3.1 Flow Invariant Formula

```
┌─────────────────────────────────────────┐
│         FLOW INVARIANT                  │
└─────────────────────────────────────────┘

DEPOSITED - WITHDRAWN + RECEIVED - SPENT - ESCROW_OUT = BALANCE

Where:
• DEPOSITED   = lifetime_deposited
• WITHDRAWN   = lifetime_withdrawn
• RECEIVED    = lifetime_received
• SPENT       = lifetime_spent
• ESCROW_OUT  = Active escrows where user is buyer
• BALANCE     = total_trust
```

#### 6.3.2 Flow

┌─────────────────────────────────────────┐
│        FLOW VALIDATION CHECK            │
└─────────────────────────────────────────┘

[V1] Get wallet running totals:
         │
         ├── lifetime_deposited: D
         ├── lifetime_withdrawn: W
         ├── lifetime_spent: S
         ├── lifetime_received: R
         └── total_trust: B
         │
         ▼
[V2] Get active escrow out:
         │
         ├── Query EscrowHold WHERE buyer_id = user_id AND status = HOLDING
         └── escrow_out: E
         │
         ▼
[V3] Calculate expected balance:
         │
         └── expected = D - W + R - S - E
         │
         ▼
[V4] Compare:
         │
         ├── expected >= B?
         │
         ├── Yes ──► PASS
         ├── No ──► FAIL (flow doesn't match, possible hidden transaction)
         │
         ▼
[V5] Return CheckResult:
         │
         ├── passed: true/false
         ├── details: "Flow validation OK" or "Flow mismatch: expected X, actual Y"
         └── severity: INFO / ERROR

### 6.4 Check 3: Fraud Pattern Detection

#### 6.4.1 Fraud Rules

| Pattern | Condition | Risk Increase |
|---------|-----------|---------------|
| **Too many withdrawals** | today_withdrawals >= 5 | +0.3 |
| **Large sudden withdrawal** | amount > avg_balance_30d * 5 | +0.4 |
| **New account rapid withdrawal** | age < 7 days AND amount > 1000 | +0.5 |
| **First withdrawal** | prev_withdrawals == 0 | +0.2 |
| **Unusual timing** | hour >= 0 AND hour < 6 (midnight-6am) | +0.1 |

#### 6.4.2 Flow

┌─────────────────────────────────────────┐
│       FRAUD PATTERN CHECK               │
└─────────────────────────────────────────┘

[V1] Initialize:
         │
         └── risk_score = 0.0
         │
         ▼
[V2] Pattern 1: Too many withdrawals today
         │
         ├── Query: COUNT(*) WHERE type LIKE 'WITHDRAWAL%' AND created_at >= today_start
         ├── count >= 5?
         ├── ├── Yes ──► risk_score += 0.3
         └── ├── No ──► Continue
         │
         ▼
[V3] Pattern 2: Large sudden withdrawal
         │
         ├── Query: AVG(daily_balance) for last 30 days
         ├── amount > avg * 5?
         ├── ├── Yes ──► risk_score += 0.4
         └── ├── No ──► Continue
         │
         ▼
[V4] Pattern 3: New account rapid withdrawal
         │
         ├── account_age = now - wallet.created_at
         ├── age < 7 days AND amount > 1000?
         ├── ├── Yes ──► risk_score += 0.5
         └── ├── No ──► Continue
         │
         ▼
[V5] Pattern 4: First withdrawal
         │
         ├── Query: COUNT(*) WHERE type LIKE 'WITHDRAWAL%'
         ├── count == 0?
         ├── ├── Yes ──► risk_score += 0.2
         └── ├── No ──► Continue
         │
         ▼
[V6] Pattern 5: Unusual timing
         │
         ├── hour = now.hour()
         ├── hour >= 0 AND hour < 6?
         ├── ├── Yes ──► risk_score += 0.1
         └── ├── No ──► Continue
         │
         ▼
[V7] Determine result:
         │
         ├── risk_score < 0.3 ──► PASS (Auto-approve)
         ├── risk_score 0.3 - 0.7 ──► REVIEW (Manual check needed)
         └── risk_score >= 0.7 ──► FAIL (Auto-reject)
         │
         ▼
[V8] Return CheckResult:
         │
         ├── passed: true/false
         ├── details: "Risk score: X (patterns detected)"
         └── severity: INFO / WARNING / ERROR

### 6.5 Check 4: Daily/Monthly Limits

#### 6.5.1 Limit Rules

```
┌─────────────────────────────────────────┐
│            WITHDRAWAL LIMITS             │
└─────────────────────────────────────────┘

PER TRANSACTION:
• Min: 10 Trust (10,000 VND)
• Max: 100,000 Trust (100,000,000 VND)

DAILY LIMIT:
• Total: 500,000 Trust (500,000,000 VND)
• If exceeded: Require manual review

MONTHLY LIMIT:
• Total: 5,000,000 Trust (5 billion VND)
• If exceeded: Block + Admin notification

VELOCITY LIMIT:
• Max 5 withdrawals per day
• Max 20 withdrawals per month
```

#### 6.5.2 Flow

┌─────────────────────────────────────────┐
│         LIMITS CHECK                    │
└─────────────────────────────────────────┘

[V1] Check transaction amount:
         │
         ├── amount >= 10 AND amount <= 100,000?
         ├── ├── No ──► FAIL (outside limits)
         └── ├── Yes ──► Continue
         │
         ▼
[V2] Check daily total:
         │
         ├── Query: SUM(amount) WHERE type LIKE 'WITHDRAWAL%' AND created_at >= today_start
         ├── today_total + amount <= 500,000?
         ├── ├── No ──► WARNING (exceeds daily limit, flag for review)
         └── ├── Yes ──► Continue
         │
         ▼
[V3] Check monthly total:
         │
         ├── Query: SUM(amount) WHERE type LIKE 'WITHDRAWAL%' AND created_at >= month_start
         ├── month_total + amount <= 5,000,000?
         ├── ├── No ──► FAIL (exceeds monthly limit, block)
         └── ├── Yes ──► Continue
         │
         ▼
[V4] Check daily velocity:
         │
         ├── Query: COUNT(*) WHERE type LIKE 'WITHDRAWAL%' AND created_at >= today_start
         ├── today_count < 5?
         ├── ├── No ──► FAIL (max daily withdrawals reached)
         └── ├── Yes ──► Continue
         │
         ▼
[V5] Check monthly velocity:
         │
         ├── Query: COUNT(*) WHERE type LIKE 'WITHDRAWAL%' AND created_at >= month_start
         ├── month_count < 20?
         ├── ├── No ──► FAIL (max monthly withdrawals reached)
         └── ├── Yes ──► PASS
         │
         ▼
[V6] Return CheckResult:
         │
         ├── passed: true/false
         ├── details: "All limits OK" or "Limit exceeded: X"
         └── severity: INFO / WARNING / ERROR

### 6.6 Aggregation & Decision

┌─────────────────────────────────────────┐
│      AGGREGATE & DECIDE                 │
└─────────────────────────────────────────┘

[Final1] Collect all CheckResults:
         │
         ├── check1: Balance Integrity
         ├── check2: Flow Validation
         ├── check3: Fraud Pattern
         └── check4: Limits
         │
         ▼
[Final2] Calculate overall risk_score:
         │
         ├── If any check.severity = CRITICAL ──► overall_risk = 1.0 (auto-reject)
         ├── Else if any check.severity = ERROR ──► overall_risk = 0.8 (likely reject)
         ├── Else if fraud_check.risk_score exists ──► Use that
         └── Else ──► overall_risk = 0.0 (safe)
         │
         ▼
[Final3] Determine final decision:
         │
         ├── Any CRITICAL or overall_risk >= 0.7 ──► REJECT
         ├── Any WARNING or 0.3 <= overall_risk < 0.7 ──► REVIEW
         ├── All PASS or overall_risk < 0.3 ──► PASS
         │
         ▼
[Final4] Return ValidationResult:
         │
         ├── balance_check: CheckResult
         ├── flow_check: CheckResult
         ├── fraud_check: CheckResult
         ├── limit_check: CheckResult
         ├── overall_passed: true/false
         └── risk_score: overall_risk

---

## 7. Reconciliation Formulas & Daily Checks

### 7.1 System-wide Invariant

```
┌─────────────────────────────────────────┐
│         SYSTEM INVARIANT                │
└─────────────────────────────────────────┘

Σ(All User Wallets) + Platform_Escrow = Σ(Deposits) - Σ(Withdrawals)

Where:
• Σ(All User Wallets) = Total trust in all user + seller wallets
• Platform_Escrow = Platform wallet available (escrow pool)
• Σ(Deposits) = Sum of all deposits (auto + manual)
• Σ(Withdrawals) = Sum of all completed withdrawals

If mismatch → CRITICAL ALERT
```

### 7.2 Platform Wallet Invariant

```
┌─────────────────────────────────────────┐
│      PLATFORM WALLET INVARIANT          │
└─────────────────────────────────────────┘

Platform.available_trust = Σ(Active Escrows) + Σ(Commission Collected, chưa rút)

Check hourly:
1. Query all EscrowHold WHERE status = HOLDING
2. Sum amounts → total_escrow
3. Query commission collected - commission paid → net_commission
4. Verify: Platform.available >= total_escrow + net_commission
5. If mismatch → Investigate
```

### 7.3 User Wallet Invariant

```
┌─────────────────────────────────────────┐
│       USER WALLET INVARIANT             │
└─────────────────────────────────────────┘

For each User/Seller wallet:

total_trust = available + withdrawal_locked + dispute_locked

AND

lifetime_deposited - lifetime_withdrawn + lifetime_received - lifetime_spent
    - active_escrow_as_buyer = total_trust

Check:
• Real-time: After each transaction
• Batch: Daily reconciliation job
```

### 7.4 Daily Reconciliation Flow

┌─────────────────────────────────────────┐
│     DAILY RECONCILIATION (3:00 AM)      │
└─────────────────────────────────────────┘

[R1] Cron starts at 3:00 AM daily
         │
         ▼
[R2] Check 1: System Total
         │
         ├── total_wallets = Σ(wallet.total_trust) WHERE status != FROZEN
         ├── total_deposits = Σ(transactions) WHERE type LIKE 'DEPOSIT%' AND status = COMPLETED
         ├── total_withdrawals = Σ(transactions) WHERE type LIKE 'WITHDRAWAL%' AND status = COMPLETED
         └── platform_escrow = Platform.available_trust
         │
         ├── Verify: total_wallets + platform_escrow == total_deposits - total_withdrawals?
         │
         ├── No match ──► CRITICAL ALERT: "System leak detected"
         ├── Match ──► PASS
         │
         ▼
[R3] Check 2: Platform Escrow
         │
         ├── platform_balance = Platform.available_trust
         ├── active_escrows = Σ(escrow_amount) WHERE status = HOLDING
         └── commission_collected = Σ(commission_debt) across all sellers
         │
         ├── Verify: platform_balance >= active_escrows?
         │
         ├── No ──► CRITICAL ALERT: "Platform shortage"
         ├── Yes ──► PASS
         │
         ▼
[R4] Check 3: VND ↔ Trust Conversion
         │
         ├── vnd_in = Σ(vnd_amount) WHERE type LIKE 'DEPOSIT%' AND status = COMPLETED
         ├── trust_in = Σ(amount) WHERE type LIKE 'DEPOSIT%' AND status = COMPLETED
         └── vnd_in / 1000 == trust_in?
         │
         ├── No ──► ALERT: "Conversion mismatch"
         ├── Yes ──► PASS
         │
         ▼
[R5] Check 4: Commission Balance
         │
         ├── commission_collected = Σ(amount) WHERE type = COMMISSION_COLLECTED
         ├── commission_debt = Σ(commission_debt) across all sellers
         └── Note: collected may be less than debt (debt decreases on withdrawal)
         │
         ├── Verify: No negative commission_debt?
         │
         ├── Has negative ──► ALERT: "Commission debt negative"
         ├── All positive ──► PASS
         │
         ▼
[R6] Generate daily report:
         │
         ├── Timestamp
         ├── All check results
         ├── Total wallets, transactions
         ├── Platform balance
         ├── Any alerts
         │
         ▼
[R7] Send notifications:
         │
         ├── Has alerts ──► Send URGENT email to admin
         ├── No alerts ──► Send normal daily report email
         │
         ▼
         END

---

## 8. Performance Optimization

### 8.1 Monthly Snapshot Strategy

```
┌─────────────────────────────────────────┐
│      MONTHLY SNAPSHOT STRATEGY          │
└─────────────────────────────────────────┘

Problem: Querying all transactions is slow (millions of rows)

Solution: Monthly snapshot + incremental calculation

┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐
│ Nov '25 │──│ Dec '25 │──│ Jan '26 │──│  Now    │
│ Snapshot│  │ Snapshot│  │ Snapshot│  │ Query   │
│ 10,000  │  │ 15,000  │  │ 18,000  │  │ +2,000  │
└─────────┘  └─────────┘  └─────────┘  └─────────┘
              ↑                           ↑
         Verified                    Only query
         checkpoint                  this month

Validation Query:
expected = Jan_Snapshot + Σ(Feb_transactions)
         = 18,000 + 2,000 = 20,000

Instead of: Σ(all transactions since account creation)
```

### 8.2 Redis Caching Strategy

```
┌─────────────────────────────────────────┐
│         REDIS CACHING                   │
└─────────────────────────────────────────┘

KEY: wallet:{wallet_id}
VALUE: {
  available_trust: 10000,
  withdrawal_locked: 0,
  dispute_locked: 0,
  total_trust: 10000,
  commission_debt: 500,
  updated_at: "2026-01-01T10:00:00Z"
}
TTL: 5 minutes
INVALIDATE: On any wallet update

─────────────────────────────────────────

KEY: monthly_snapshot:{wallet_id}:{month}
VALUE: {snapshot_data}
TTL: 1 month (immutable once created)

─────────────────────────────────────────

KEY: shop_commission:{shop_id}
VALUE: { rate: 0.03, effective_from: "..." }
TTL: 1 hour

─────────────────────────────────────────

KEY: daily_withdrawal_total:{wallet_id}:{date}
VALUE: 50000 (Trust)
TTL: 24 hours
INCREMENT: Atomic on each withdrawal
```

---

## 9. Business Rules Summary

| # | Rule |
|---|------|
| **BR1** | 1000 VND = 1 Trust (cố định) |
| **BR2** | Mọi giao dịch phải qua Platform Wallet |
| **BR3** | Escrow hold: 3 ngày (72 giờ) |
| **BR4** | Commission default: 5%, có thể override 1-20% |
| **BR5** | Withdrawal validation: Balance + Flow + Fraud + Limits |
| **BR6** | Monthly snapshot: Ngày 1 mỗi tháng, 2:00 AM |
| **BR7** | Daily reconciliation: 3:00 AM mỗi ngày |
| **BR8** | Discrepancy > 100 Trust: CRITICAL alert |
| **BR9** | Risk score >= 0.7: Auto-reject withdrawal |
| **BR10** | Mọi admin operation phải có audit log |

---

## 10. API Endpoints Overview

### 10.1 User Wallet APIs

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | /api/v3/wallet | Get wallet info |
| POST | /api/v3/wallet/deposit/initiate | Initiate deposit |
| POST | /api/v3/wallet/withdrawal/request | Request withdrawal |
| GET | /api/v3/wallet/transactions | Get transaction history |
| GET | /api/v3/wallet/balance | Get current balance |

### 10.2 Seller Wallet APIs

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | /api/v3/seller/wallet | Get seller wallet |
| GET | /api/v3/seller/escrows | Get pending escrows |
| GET | /api/v3/seller/commission | Get commission debt |
| POST | /api/v3/seller/withdraw | Request withdrawal |

### 10.3 Admin Wallet APIs

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | /api/v3/admin/wallets/dashboard | Dashboard overview |
| POST | /api/v3/admin/wallets/deposit | Manual deposit |
| POST | /api/v3/admin/wallets/debit | Manual debit |
| GET | /api/v3/admin/withdrawals/pending | List pending withdrawals |
| POST | /api/v3/admin/withdrawals/:id/approve | Approve withdrawal |
| POST | /api/v3/admin/withdrawals/:id/reject | Reject withdrawal |
| POST | /api/v3/admin/commission/setup | Setup commission rate |
| GET | /api/v3/admin/reconcile/daily | Trigger daily reconciliation |

---

**End of Document**
