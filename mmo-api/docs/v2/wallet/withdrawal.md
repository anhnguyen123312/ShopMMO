# Withdrawal Flows - Vendor

## Tổng quan

**Withdrawal Flow** cho phép Vendor (Seller) rút tiền từ wallet về tài khoản ngân hàng. Seller sẽ bị trừ commission khi rút tiền.

**Actors:**
- Vendor (Seller) - Yêu cầu rút tiền
- System - Validation engine, xử lý rút tiền
- Bank - Chuyển tiền VND
- Admin - Phê duyệt các yêu cầu cần review

**Key Features:**
- Commission được trừ tự động khi seller rút tiền
- Validation engine: Balance + Flow + Fraud + Limits
- Auto-approve nếu risk_score < 0.3
- Manual review nếu 0.3 <= risk_score < 0.7
- Auto-reject nếu risk_score >= 0.7

---

## 1. Commission Deduction khi Withdraw

### 1.1 Tổng quan

```
┌─────────────────────────────────────────┐
│      COMMISSION WHEN WITHDRAWING        │
└─────────────────────────────────────────┘

KEY PRINCIPLE:
Commission được trừ TỰ ĐỘNG từ seller withdrawal_amount.

Commission tracking:
- Escrow release: Ghi nhận commission_debt (+)
- Seller withdraw: Trừ commission_debt (-) + Transfer cho Platform
```

### 1.2 Commission Calculation

```
┌─────────────────────────────────────────┐
│      COMMISSION CALCULATION             │
└─────────────────────────────────────────┘

[STEP 1] Tính commission cần trừ:

         withdrawal_amount = 100 Trust
         commission_rate = 5%
         expected_commission = 100 × 5% = 5 Trust
         current_commission_debt = 20 Trust

         commission_to_deduct = min(5, 20) = 5 Trust

         Reason: Trừ minimum giữa:
         - 5% của withdrawal (5 Trust)
         - Commission debt hiện có (20 Trust)

[STEP 2] Tính actual receive:

         actual_trust = 100 - 5 = 95 Trust
         vnd_amount = 95 × 1000 = 95,000 VND

[STEP 3] Bank Transfer:

         Call Bank API → Transfer 95,000 VND to seller account
```

---

## 2. Seller Withdrawal Flow

### 2.1 Conditions/Requirements

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

### 2.2 Flow Diagram

┌─────────────────────────────────────────┐
│      SELLER WITHDRAWAL FLOW             │
└─────────────────────────────────────────┘

[B1] Seller nhấn "Rút tiền"
         │
         ▼
[B2] Hệ thống hiển thị thông tin:

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
[B3] Seller nhập số Trust muốn rút
         │
         ├── Invalid ──► Show error
         ├── Valid ──► Continue
         │
         ▼
[B4] Get commission rate:
         │
         ├── Query shop_commission_config
         ├── Not found ──► Use default 5%
         ├── Found ──► Use config.rate
         │
         ▼
[B5] Calculate commission:
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
[B6] Hệ thống hiển thị xác nhận:

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
[B7] BEGIN TRANSACTION
         │
         ▼
[B8] Tạo WithdrawalRequest:
         │
         ├── Status: PENDING
         ├── trust_amount: 500
         ├── commission_deduct: 25
         ├── net_trust: 475
         └── vnd_amount: 475,000
         │
         ▼
[B9] Lock funds:
         │
         ├── Tạo Transaction: WITHDRAWAL_REQUEST
         ├── Move: available (-500) → withdrawal_locked (+500)
         └── COMMIT
         │
         ▼
[B10] Enqueue validation job
         │
         └── (Go to Validation Flow)
         │
         ▼
         ─────────────────────────────────────────
         [After validation passed and bank transfer success]
         ─────────────────────────────────────────
         │
         ▼
[B11] BEGIN TRANSACTION
         │
         ▼
[B12] Tạo Transaction: WITHDRAWAL_COMPLETED
         │
         ├── Direction: DEBIT
         ├── Amount: 500 (full trust_amount)
         └── balance_type: WITHDRAWAL_LOCKED
         │
         ▼
[B13] Update Seller Wallet:
         │
         ├── withdrawal_locked -= 500
         ├── total_trust -= 500
         ├── lifetime_withdrawn += 500
         └── commission_debt -= 25
         │
         ▼
[B14] Commission to Platform:
         │
         ├── Tạo Transaction Platform:
         │   ├── Type: COMMISSION_RELEASED
         │   ├── Direction: CREDIT
         │   └── Amount: 25
         │
         └── Update Platform Wallet:
             ├── available_trust += 25
             ├── withdrawable_commission += 25
             └── total_trust += 25
         │
         ▼
[B15] COMMIT
         │
         ▼
[B16] Notify Seller:
         │
         ├── "Rút tiền thành công: 475,000 VND"
         └── "Commission đã trả: 25 Trust"
         │
         ▼
         END

### 2.3 Edge Cases & Error Handling

| Case | Condition | Handling | User Message |
|------|-----------|----------|--------------|
| Commission debt = 0 | commission_debt = 0 | No commission deducted | "Không có commission nợ. Rút đầy đủ." |
| Partial debt | commission_debt < calculated | Deduct actual debt only | "Commission: 25 Trust (nợ còn 25 Trust)" |
| Full debt coverage | commission_debt >= calculated | Deduct calculated, debt remains | "Commission: 25 Trust (nợ còn 25 Trust)" |
| Insufficient balance | available < amount | Reject with 400 | "Số dư không đủ. Vui lòng nhập số tiền nhỏ hơn." |
| Daily limit exceeded | today_total + amount > 500,000 | Flag for review | "Yêu cầu cần được admin phê duyệt." |
| Monthly limit exceeded | month_total + amount > 5,000,000 | Block | "Bạn đã đạt giới hạn rút tiền tháng này." |
| Velocity exceeded | today_count >= 5 | Block | "Bạn đã đạt số lần rút tối đa trong ngày." |
| High fraud score | risk_score >= 0.7 | Auto-reject | "Yêu cầu bị từ chối. Vui lòng liên hệ hỗ trợ." |
| Bank transfer failed | API returns error | Retry 3x, then fail with refund | "Giao dịch thất bại. Tiền đã hoàn lại." |
| No bank info | Bank account not configured | Redirect to setup | "Vui lòng cấu hình tài khoản ngân hàng trước." |

---

## 3. Validation Engine

### 3.1 Validation Architecture

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

### 3.2 Check 1: Balance Integrity

```
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
```

### 3.3 Check 2: Flow Validation

```
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
```

### 3.4 Check 3: Fraud Pattern Detection

| Pattern | Condition | Risk Increase |
|---------|-----------|---------------|
| **Too many withdrawals** | today_withdrawals >= 5 | +0.3 |
| **Large sudden withdrawal** | amount > avg_balance_30d * 5 | +0.4 |
| **New account rapid withdrawal** | age < 7 days AND amount > 1000 | +0.5 |
| **First withdrawal** | prev_withdrawals == 0 | +0.2 |
| **Unusual timing** | hour >= 0 AND hour < 6 (midnight-6am) | +0.1 |

### 3.5 Check 4: Limits

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

---

## 4. Admin Withdrawal Review

### 4.1 Conditions

┌─────────────────────────────────────────┐
│         ĐIỀU KIỆN THỰC HIỆN            │
└─────────────────────────────────────────┘

1. **Actors**: Admin
2. **Preconditions**:
   ├── Admin logged in with WITHDRAWAL_APPROVE permission
   └── WithdrawalRequest with status = AWAITING_APPROVAL

3. **Business Rules**:
   ├── Must review validation results before deciding
   ├── Reject requires reason
   ├── Hold requires reason and investigation ticket
   └── All actions logged to AdminOperationLog

### 4.2 Flow Diagram

┌─────────────────────────────────────────┐
│     ADMIN WITHDRAWAL REVIEW FLOW        │
└─────────────────────────────────────────┘

[A1] Admin selects "Pending Withdrawals"
         │
         ▼
[A2] System shows details with validation results
         │
         ├── Admin reviews: Balance, Flow, Fraud, Limits checks
         ├── Risk score displayed
         └── Recent transactions shown
         │
         ▼
[A3] Admin makes decision
         │
         ├── Approve ──► Update status → APPROVED → Enqueue bank transfer
         ├── Reject ──► Require reason → Unlock funds → Notify user
         ├── Hold ──► Require reason → Create investigation ticket
         │
         ▼
         END

---

## 5. Transaction Types

### 5.1 Withdrawal Transaction Types

| Type | Direction | Description |
|------|-----------|-------------|
| **WITHDRAWAL_REQUEST** | DEBIT | Lock funds for withdrawal |
| **WITHDRAWAL_COMPLETED** | DEBIT | Finalize withdrawal |
| **WITHDRAWAL_REJECTED** | CREDIT | Refund rejected withdrawal |
| **COMMISSION_RELEASED** | CREDIT | Platform receives commission |

### 5.2 Withdrawal Request Data Model

```javascript
{
  _id: ObjectId,
  request_id: String,             // "WD-{ULID}"
  wallet_id: String,
  user_id: String,

  // Amounts
  trust_amount: Number,            // Trust to withdraw
  commission_deduct: Number,       // Commission to pay (seller only)
  net_trust: Number,               // trust_amount - commission_deduct
  vnd_amount: Number,              // net_trust * 1000

  // Bank Info
  bank_code: String,
  bank_name: String,
  account_number: String,
  account_name: String,

  // Status
  status: String,                 // "PENDING" | "VALIDATING" | "APPROVED" | "COMPLETED" | "REJECTED"

  // Validation
  validation_result: {
    balance_check: { passed: bool, details: String, severity: String },
    flow_check: { passed: bool, details: String, severity: String },
    fraud_check: { passed: bool, details: String, severity: String },
    limit_check: { passed: bool, details: String, severity: String },
    overall_passed: bool,
    risk_score: Number
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
```

---

## 6. API Endpoints

| Method | Endpoint | Description | Access |
|--------|----------|-------------|--------|
| POST | /api/v3/seller/withdraw/request | Request withdrawal | Seller |
| GET | /api/v3/seller/withdraw/status/:id | Check withdrawal status | Seller |
| GET | /api/v3/seller/withdrawals/history | List withdrawal history | Seller |
| GET | /api/v3/admin/withdrawals/pending | List pending withdrawals | Admin |
| POST | /api/v3/admin/withdrawals/:id/approve | Approve withdrawal | Admin |
| POST | /api/v3/admin/withdrawals/:id/reject | Reject withdrawal | Admin |
| POST | /api/v3/admin/withdrawals/:id/hold | Hold for investigation | Admin |

---

## 7. Business Rules Summary

| # | Rule |
|---|------|
| **BR_WITHDRAW_1** | Min: 10 Trust, Max: 100,000 Trust per transaction |
| **BR_WITHDRAW_2** | Daily limit: 500,000 Trust, Monthly: 5,000,000 Trust |
| **BR_WITHDRAW_3** | Velocity: Max 5/day, 20/month |
| **BR_WITHDRAW_4** | Commission deducted: min(amount × rate, commission_debt) |
| **BR_WITHDRAW_5** | Risk score < 0.3: Auto-approve |
| **BR_WITHDRAW_6** | Risk score 0.3-0.7: Manual review |
| **BR_WITHDRAW_7** | Risk score >= 0.7: Auto-reject |
| **BR_WITHDRAW_8** | Validation: Balance + Flow + Fraud + Limits |
| **BR_WITHDRAW_9** | Bank transfer retry: 3 times before fail |
| **BR_WITHDRAW_10** | Commission transferred to Platform on completion |

---

## Related Documents

- [Wallet Overview](wallet-overview.md) - Tổng quan hệ thống
- [Deposit Flows](deposit.md) - Nạp tiền
- [Escrow System](escrow.md) - Escrow auto-release
- [Admin Operations](adjustment.md) - Admin operations
