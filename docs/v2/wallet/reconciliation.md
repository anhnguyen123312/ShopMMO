# Reconciliation - Đối soát hệ thống

## Tổng quan

**Reconciliation đảm bảo:**
- Không có Trust bị leak hoặc tạo từ không khí
- Platform Wallet balance khớp với tổng escrows + withdrawable_commission
- Tổng Trust trong hệ thống = Tổng VND đã nạp / 1000
- Phát hiện bất thường để alert admin

**3 loại reconciliation:**
1. **Real-time Balance Check** - Mỗi transaction kiểm tra balance invariants
2. **Monthly Snapshot** - Tạo verified snapshot mỗi tháng
3. **Daily Full Reconciliation** - Reconcile toàn hệ thống mỗi ngày

**Actors:**
- System - Auto-running reconciliation jobs
- Admin - Review alerts, investigate discrepancies

---

## 1. Real-time Balance Check

### 1.1 Overview

Chạy: **SAU MỌI TRANSACTION** (in transaction)

Mục đích: Phát hiện ngay lập tức nếu có balance mismatch

### 1.2 Balance Invariants

```
┌─────────────────────────────────────────┐
│      REAL-TIME BALANCE CHECKS           │
└─────────────────────────────────────────┘

[CHECK 1] Total == Sum of States?

        Formula:
        total_trust == available_trust + withdrawal_locked + dispute_locked

        Ví dụ:
        total_trust = 500
        available = 300
        withdrawal_locked = 100
        dispute_locked = 100
        admin_debt = 0
        → 500 == 300 + 100 + 100 + 0 ✅

        ❌ Nếu FAIL:
        🚨 CRITICAL ALERT
        "Balance state mismatch detected!
         Wallet: WLT-USER-123
         total_trust: 500
         sum_of_states: 450 (calculated)
         Possible: Database corruption or concurrent modification"

        Action:
        → Consider ROLLBACK transaction
        → Log full context
        → Alert engineering team

[CHECK 2] All States >= 0?

        Formula:
        available_trust >= 0 AND
        withdrawal_locked >= 0 AND
        dispute_locked >= 0 AND
        admin_debt >= 0

        ❌ Nếu FAIL:
        🚨 CRITICAL ALERT
        "Negative balance detected!
         Wallet: WLT-USER-123
         available_trust: -50
         Possible: Logic bug or double-spend"

        Action:
        → IMMEDIATE ROLLBACK
        → Alert engineering team
        → Block wallet operations

[CHECK 3] Available <= Total?

        Formula:
        available_trust <= total_trust

        ❌ Nếu FAIL:
        🚨 CRITICAL ALERT
        "Available exceeds total!
         Wallet: WLT-USER-123
         available: 600
         total: 500
         Possible: Calculation error"

        Action:
        → Consider ROLLBACK
        → Alert engineering team
```

---

## 2. Monthly Snapshot Flow

### 2.1 Conditions

```
┌─────────────────────────────────────────┐
│         ĐIỀU KIỆN THỰC HIỆN            │
└─────────────────────────────────────────┘

1. **Actors**: System (Cron job)
2. **Schedule**: Ngày 1 hàng tháng, 2:00 AM
3. **Duration**: ~30 phút (tùy số lượng wallets)
4. **Business Rules**:
   ├── Tạo snapshot cho tất cả ACTIVE và SUSPENDED wallets
   ├── Calculate balance từ transactions
   ├── Compare với actual balance
   ├── Discrepancy > 100 Trust → CRITICAL alert
   └── Discrepancy <= 100 Trust → WARNING alert
```

### 2.2 Flow Diagram

```
┌─────────────────────────────────────────┐
│       MONTHLY SNAPSHOT - VERIFICATION    │
└─────────────────────────────────────────┘

[START] Cron triggered: 2025-02-01 02:00:00
        │
        ▼
[MS1] Xác định tháng cần snapshot:

        target_month = 2025-01 (previous month)
        start_date = 2025-01-01 00:00:00
        end_date = 2025-01-31 23:59:59
        │
        ▼
[MS2] Query tất cả wallets:

        SELECT * FROM wallets
        WHERE status IN ('ACTIVE', 'SUSPENDED')
        │
        Result: 1,250 wallets
        │
        ▼
[MS3] Loop qua từng wallet (batch 100):
        │
        ▼
[MS4] Get wallet info:

        wallet_id = "WLT-USER-123"
        total_trust = 500000
        │
        ▼
[MS5] Query tất cả transactions của wallet:

        SELECT * FROM transactions
        WHERE wallet_id = 'WLT-USER-123'
          AND created_at <= '2025-01-31 23:59:59'
        │
        Result: 245 transactions
        │
        ▼
[MS6] Calculate balance from transactions:

        calculated_balance = 0

        FOR EACH transaction:
          IF amount > 0:  // Credit
            calculated_balance += amount
          ELSE:  // Debit
            calculated_balance += amount  // amount is negative

        Ví dụ:
        Opening balance (from last snapshot): 400,000
        Credits this month: +200,000
        Debits this month: -100,000
        calculated_balance = 400,000 + 200,000 - 100,000 = 500,000
        │
        ▼
[MS7] Compare calculated vs actual:

        actual_balance = wallet.total_trust = 500,000
        calculated_balance = 500,000

        IF calculated == actual:
        └──→ VERIFIED ✅
             discrepancy = 0
             status = "VERIFIED"

        ELSE:
        └──→ DISCREPANCY FOUND ❌
             discrepancy = calculated - actual
                    = 500,000 - 485,000 = 15,000

             severity = discrepancy > 100 ? "CRITICAL" : "WARNING"
        │
        ▼
[MS8] Tạo MonthlySnapshot record:

        INSERT INTO monthly_snapshots {
          wallet_id: "WLT-USER-123",
          month: "2025-01",
          opening_balance: 400000,
          credits: 200000,
          debits: -100000,
          calculated_balance: 500000,
          actual_balance: 485000,
          discrepancy: 15000,
          status: "REQUIRE_MANUAL_REVIEW",
          created_at: NOW()
        }
        │
        ▼
[MS9] Handle discrepancy:

        IF discrepancy > 100:
        └──→ 🚨 CRITICAL ALERT
              "Wallet WLT-USER-123 has discrepancy!
               Expected: 500,000 Trust
               Actual: 485,000 Trust
               Discrepancy: 15,000 Trust
               Status: REQUIRE_MANUAL_REVIEW
               Action: Wallet operations BLOCKED"

              → Block wallet operations
              → Email urgent to admin team
              → Create ticket for investigation

        ELSE IF discrepancy > 0:
        └──→ ⚠️ WARNING
              "Wallet WLT-USER-123 has minor discrepancy
               Expected: 500,000 Trust
               Actual: 485,000 Trust
               Discrepancy: 15,000 Trust
               Status: FLAGGED_FOR_REVIEW"

              → Flag for review
              → Email warning to admin team
        │
        ▼
[MS10] Continue next wallet:
        │
        ├── Vẫn còn wallets ──► Quay lại [MS4]
        │
        └── Hết wallets ──► [MS11]
        │
        ▼
[MS11] Generate monthly report:

        ╔═══════════════════════════════════════════╗
        ║  MONTHLY SNAPSHOT REPORT - JANUARY 2025    ║
        ╠═══════════════════════════════════════════╣
        ║                                           ║
        ║  Total wallets: 1,250                     ║
        ║  Verified: 1,240 (99.2%)                 ║
        ║  Discrepancy found: 10 (0.8%)             ║
        ║                                           ║
        ║  Discrepancy details:                      ║
        ║  • Critical (>100): 2 wallets             ║
        ║    - WLT-USER-123: 15,000 Trust          ║
        ║    - WLT-USER-456: 8,500 Trust           ║
        ║                                           ║
        ║  • Warning (1-100): 8 wallets             ║
        ║                                           ║
        ║  Total discrepancy: 23,500 Trust          ║
        ║                                           ║
        ║  Actions:                                 ║
        ║  • Investigate WLT-USER-123               ║
        ║  • Investigate WLT-USER-456               ║
        ║                                           ║
        ╚═══════════════════════════════════════════╝
        │
        ▼
[MS12] Email report to admin team:

        TO: admin-team@company.com
        SUBJECT: Monthly Wallet Snapshot - January 2025

        ... (report above) ...

        ▼
[END] Monthly snapshot completed
```

---

## 3. Daily Full Reconciliation Flow

### 3.1 Conditions

```
┌─────────────────────────────────────────┐
│         ĐIỀU KIỆN THỰC HIỆN            │
└─────────────────────────────────────────┘

1. **Actors**: System (Cron job)
2. **Schedule**: Mỗi ngày 3:00 AM
3. **Duration**: ~15 phút
4. **Business Rules**:
   ├── Chạy 5 checks độc lập
   ├── Bất kỳ check fail → URGENT email
   └── Report lưu vào database để audit
```

### 3.2 Flow Diagram - 5 Checks

```
┌─────────────────────────────────────────┐
│     DAILY FULL RECONCILIATION - 5 CHECKS │
└─────────────────────────────────────────┘

[START] Cron triggered: 2025-01-16 03:00:00
        │
        ▼
════════════════════════════════════════════════════════
[CHECK 1] System Total Trust
════════════════════════════════════════════════════════

[D1.1] Calculate total all wallets:

        total_all_wallets = Σ(wallet.total_trust)
                          = 50,000,000 Trust

[D1.2] Calculate total deposits:

        total_deposits = Σ(transactions WHERE type = 'DepositConvert')
                       = 60,000,000 Trust

[D1.3] Calculate total withdrawals:

        total_withdrawals = Σ(transactions WHERE type = 'WithdrawalComplete')
                          = 10,000,000 Trust

[D1.4] Compare:

        Expected: total_wallets == deposits - withdrawals
                  50,000,000 == 60,000,000 - 10,000,000
                  50,000,000 == 50,000,000 ✅

        ❌ IF FAIL:
        🚨 ALERT 1: System total mismatch
        "Expected: 50,000,000 Trust
         Actual: 49,000,000 Trust
         Missing: 1,000,000 Trust
         Possible: Trust leak or double-spend"

        Action:
        → URGENT email to admin
        → Block all wallet operations
        → Run full investigation

        ✅ IF PASS:
        → Continue to Check 2

════════════════════════════════════════════════════════
[CHECK 2] Platform Wallet Balance
════════════════════════════════════════════════════════

[D2.1] Get Platform Wallet:

        SELECT * FROM wallets WHERE user_id = 'PLATFORM'

        available_trust = 5,000,000 Trust
        total_trust = 5,100,000 Trust
        withdrawable_commission = 500,000 Trust
        escrow_holding = 4,500,000 Trust (calculated)

[D2.2] Query active escrows:

        SELECT SUM(amount) FROM escrow_holds
        WHERE status = 'HOLDING'

        total_escrows = 4,500,000 Trust

[D2.3] Compare:

        Expected: platform.available == total_escrows + withdrawable_commission
                  5,000,000 == 4,500,000 + 500,000 ✅

        ❌ IF FAIL:
        🚨 ALERT 2: Platform wallet mismatch
        "Expected: 5,000,000 Trust
         Actual: 4,500,000 Trust
         Missing: 500,000 Trust
         Possible: Escrow not created or released incorrectly"

        Action:
        → URGENT email to admin
        → Check recent escrow operations
        → Possible bug in escrow system

        ✅ IF PASS:
        → Continue to Check 3

════════════════════════════════════════════════════════
[CHECK 3] VND ↔ Trust Reconciliation
════════════════════════════════════════════════════════

[D3.1] Sum VND deposits:

        total_vnd_deposits = Σ(DepositVND.vnd_amount)
                          = 50,000,000,000 VND

[D3.2] Sum Trust deposits:

        total_trust_deposits = Σ(DepositConvert.amount)
                             = 50,000,000 Trust

[D3.3] Compare:

        Expected: total_vnd / 1000 == total_trust
                  50,000,000,000 / 1000 == 50,000,000
                  50,000,000 == 50,000,000 ✅

        ❌ IF FAIL:
        🚨 ALERT 3: VND-Trust conversion mismatch
        "Total VND deposited: 50,000,000,000
         Total Trust created: 50,100,000
         Mismatch: 100,000 Trust (100,000,000 VND)
         Possible: Double credit or wrong conversion rate"

        Action:
        → URGENT email to admin
        → Check deposit webhook processing
        → Verify conversion rate

        ✅ IF PASS:
        → Continue to Check 4

════════════════════════════════════════════════════════
[CHECK 4] Withdrawal VND Reconciliation
════════════════════════════════════════════════════════

[D4.1] Sum withdrawal Trust:

        total_withdrawal_trust = Σ(WithdrawalComplete.amount)
                              = 8,000,000 Trust

[D4.2] Sum withdrawal VND:

        total_withdrawal_vnd = Σ(WithdrawalRequest.vnd_amount)
                            = 7,600,000,000 VND

[D4.3] Sum commission deducted:

        total_commission = Σ(CommissionDeduct.amount)
                         = 400,000 Trust

[D4.4] Calculate expected VND:

        expected_vnd = (total_withdrawal_trust - total_commission) * 1000
                     = (8,000,000 - 400,000) * 1000
                     = 7,600,000,000 VND

[D4.5] Compare:

        Expected: total_withdrawal_vnd == expected_vnd
                  7,600,000,000 == 7,600,000,000 ✅

        ❌ IF FAIL:
        🚨 ALERT 4: Withdrawal VND mismatch
        "Total VND withdrawn: 7,600,000,000
         Expected: 7,500,000,000
         Mismatch: 100,000,000 VND
         Possible: Commission not deducted correctly"

        Action:
        → URGENT email to admin
        → Check withdrawal processing
        → Verify commission calculation

        ✅ IF PASS:
        → Continue to Check 5

════════════════════════════════════════════════════════
[CHECK 5] Money Flow Balance
════════════════════════════════════════════════════════

[D5.1] Calculate inflow:

        inflow = Σ(DepositConvert.amount)
               = 60,000,000 Trust

[D5.2] Calculate outflow:

        outflow = Σ(WithdrawalComplete.amount)
                = 10,000,000 Trust

[D5.3] Calculate remaining:

        remaining = inflow - outflow
                 = 60,000,000 - 10,000,000
                 = 50,000,000 Trust

[D5.4] Get actual total wallets:

        total_all_wallets = 50,000,000 Trust

[D5.5] Compare:

        Expected: remaining == total_all_wallets
                  50,000,000 == 50,000,000 ✅

        ❌ IF FAIL:
        🚨 ALERT 5: Money flow doesn't balance
        "Inflow - Outflow: 50,000,000 Trust
         Actual total wallets: 49,500,000 Trust
         Mismatch: 500,000 Trust
         Possible: Unaccounted transaction"

        Action:
        → URGENT email to admin
        → Full audit trail review
        → Check for missing transactions

        ✅ IF PASS:
        → All checks passed!

════════════════════════════════════════════════════════

[D6] Generate daily reconciliation report:

        ╔═══════════════════════════════════════════╗
        ║  DAILY RECONCILIATION - 2025-01-16         ║
        ╠═══════════════════════════════════════════╣
        ║                                           ║
        ║  Check 1 - System Total Trust: ✅ PASSED  ║
        ║  Check 2 - Platform Wallet: ✅ PASSED     ║
        ║  Check 3 - VND ↔ Trust: ✅ PASSED         ║
        ║  Check 4 - Withdrawal VND: ✅ PASSED      ║
        ║  Check 5 - Money Flow: ✅ PASSED          ║
        ║                                           ║
        ║  All checks passed!                       ║
        ║                                           ║
        ║  System summary:                          ║
        ║  • Total wallets: 1,250                   ║
        ║  • Total Trust: 50,000,000               ║
        ║  • Total VND deposited: 50,000,000,000    ║
        ║  • Platform escrows: 5,000,000           ║
        ║                                           ║
        ╚═══════════════════════════════════════════╝

        IF any alerts:
        └──→ 📧 Send URGENT email with alert details

        ELSE:
        └──→ 📧 Send normal daily report

        ▼
[D7] Save report to database:

        INSERT INTO reconciliation_reports {
          date: "2025-01-16",
          check1_passed: true,
          check2_passed: true,
          check3_passed: true,
          check4_passed: true,
          check5_passed: true,
          has_alerts: false,
          report_data: {...},
          created_at: NOW()
        }

        ▼
[END] Daily reconciliation completed
```

---

## 4. Data Models

### 4.1 MonthlySnapshot Data Model

```javascript
{
  _id: ObjectId,
  id: String,                      // "SNAP-{ULID}"

  // Wallet info
  wallet_id: String,
  user_id: String,

  // Month
  month: String,                   // "2025-01" (YYYY-MM)

  // Balances
  opening_balance: Number,         // Số dư đầu tháng
  credits: Number,                 // Tổng credit trong tháng
  debits: Number,                  // Tổng debit trong tháng
  calculated_balance: Number,      // Balance tính từ transactions
  actual_balance: Number,          // Balance thực tế trong wallet

  // Discrepancy
  discrepancy: Number,             // calculated - actual
  status: String,                  // "VERIFIED" | "FLAGGED" | "REQUIRE_MANUAL_REVIEW"

  // Audit
  created_at: DateTime,
  verified_at: DateTime,

  // Review (nếu có discrepancy)
  reviewed_by: String,             // admin_id
  reviewed_at: DateTime,
  resolution: String,
  adjustment_made: Number          // Số Trust đã adjust để fix
}
```

### 4.2 ReconciliationReport Data Model

```javascript
{
  _id: ObjectId,
  id: String,                      // "REC-{ULID}"

  // Report info
  date: Date,                      // "2025-01-16"
  type: String,                    // "DAILY" | "MONTHLY"

  // Check results
  checks: {
    system_total_trust: {
      passed: Boolean,
      expected: Number,
      actual: Number,
      discrepancy: Number
    },
    platform_wallet: {
      passed: Boolean,
      expected: Number,
      actual: Number,
      discrepancy: Number
    },
    vnd_trust_conversion: {
      passed: Boolean,
      expected_vnd: Number,
      actual_vnd: Number,
      discrepancy: Number
    },
    withdrawal_vnd: {
      passed: Boolean,
      expected_vnd: Number,
      actual_vnd: Number,
      discrepancy: Number
    },
    money_flow: {
      passed: Boolean,
      inflow: Number,
      outflow: Number,
      expected_remaining: Number,
      actual_remaining: Number,
      discrepancy: Number
    }
  },

  // Overall
  all_passed: Boolean,
  has_alerts: Boolean,

  // System summary
  summary: {
    total_wallets: Number,
    total_trust: Number,
    total_vnd_deposited: Number,
    platform_escrows: Number
  },

  // Audit
  created_at: DateTime,
  created_by: String               // "SYSTEM"
}
```

---

## 5. API Endpoints

| Method | Endpoint | Description | Access |
|--------|----------|-------------|--------|
| GET | /api/v3/admin/reconciliation/daily | Get daily reconciliation report | Admin |
| GET | /api/v3/admin/reconciliation/monthly | Get monthly snapshot report | Admin |
| GET | /api/v3/admin/reconciliation/wallet/:id | Get wallet reconciliation history | Admin |
| POST | /api/v3/admin/reconciliation/run/manual | Manually trigger reconciliation | Admin |
| PUT | /api/v3/admin/reconciliation/snapshot/:id/resolve | Resolve snapshot discrepancy | Admin |

---

## 6. Business Rules Summary

| # | Rule |
|---|------|
| **BR_RECON_1** | Real-time check chạy sau MỌI transaction (in transaction) |
| **BR_RECON_2** | Nếu real-time check fail → Consider rollback transaction |
| **BR_RECON_3** | Monthly snapshot chạy ngày 1 hàng tháng lúc 2:00 AM |
| **BR_RECON_4** | Snapshot discrepancy > 100 Trust → CRITICAL alert + manual review |
| **BR_RECON_5** | Snapshot discrepancy <= 100 Trust → WARNING alert |
| **BR_RECON_6** | Daily reconciliation chạy lúc 3:00 AM mỗi ngày |
| **BR_RECON_7** | Daily reconciliation có 5 checks độc lập |
| **BR_RECON_8** | Bất kỳ check nào fail → URGENT email to admin team |
| **BR_RECON_9** | Reconciliation report lưu vào database để audit |
| **BR_RECON_10** | Platform Wallet balance PHẢI luôn == Tổng active escrows + withdrawable_commission |

---

## Related Documents

- [Wallet Overview](wallet-overview.md) - Tổng quan hệ thống
- [Deposit Flows](deposit.md) - Nạp tiền (VND → Trust conversion)
- [Withdrawal Flows](withdrawal.md) - Rút tiền (Trust → VND conversion)
- [Escrow System](escrow.md) - Escrow hold/release
- [Admin Operations](adjustment.md) - Admin adjustments
