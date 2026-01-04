# Wallet V2 - Platform Wallet Architecture

## 1. Trust Currency

```
┌─────────────────────────────────────────┐
│            TRUST CURRENCY                │
├─────────────────────────────────────────┤
│  1000 VND = 1 Trust (Cố định)           │
│                                          │
│  • Nạp tiền: VND → Trust (3rd party)    │
│  • Rút tiền: Trust → VND (bank transfer)│
│  • Giao dịch: Trust only (float)        │
│  • Làm tròn: đến 0.001 Trust            │
└─────────────────────────────────────────┘
```

**Lưu ý:**
- Trust type: **Float** (không phải integer)
- Rounding: **3 chữ số thập phân** (0.001)
- 0.001 Trust = 1 VND

---

## 2. Tổng quan Escrow System

**Escrow System trong V2 với Platform Wallet:**

Khác với V1 nơi seller giữ tiền trong `pending_balance`, V2 sử dụng **Platform Wallet** làm trung gian để giữ escrow. Điều này đảm bảo:
- ✅ Platform kiểm soát hoàn toàn luồng tiền
- ✅ Buyer được bảo vệ trong 3 ngày
- ✅ Seller chỉ nhận tiền khi Platform Wallet release
- ✅ Commission được trừ và giữ lại ngay khi release

---

## 1. Cấu trúc Escrow V2

### 1.1 So sánh V1 vs V2

```
┌─────────────────────────────────────────────────────────────────┐
│              ESCROW V1 vs V2 ARCHITECTURE                        │
└─────────────────────────────────────────────────────────────────┘

V1 (Seller Wallet Escrow):
┌─────────────────────────────────────────────────────────────────┐
│  BUYER WALLET              VENDOR WALLET                         │
│  ┌─────────────┐           ┌─────────────────────────────────┐  │
│  │ Available   │           │  Available                      │  │
│  │   500 Trust │           │    2,000 Trust                  │  │
│  └─────────────┘           │  ─────────────────────────────  │  │
│                            │  Pending (ESCROW)               │  │
│                            │    500 Trust                    │  │
│                            │  ─────────────────────────────  │  │
│                            │  Total: 2,500 Trust             │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                 │
│  Buyer mua → Trừ buyer → Cộng vào vendor.pending               │
│  Sau 3 ngày → vendor.pending → vendor.available                │
└─────────────────────────────────────────────────────────────────┘

V2 (Platform Wallet Escrow):
┌─────────────────────────────────────────────────────────────────┐
│  BUYER WALLET     PLATFORM WALLET       SELLER WALLET            │
│  ┌─────────────┐  ┌─────────────────┐  ┌─────────────────────┐ │
│  │ Available   │  │ Available       │  │ Available            │ │
│  │   500 Trust │  │   500 Trust     │  │   2,000 Trust        │ │
│  │             │  │ (Holding escrow)│  │ Commission debt: 25  │ │
│  └─────────────┘  └─────────────────┘  └─────────────────────┘ │
│                                                                 │
│  Buyer mua → Trừ buyer → Cộng vào Platform (hold)              │
│  Sau 3 ngày → Platform trừ → Cộng seller 95%                   │
│              → Platform giữ 5% commission                       │
└─────────────────────────────────────────────────────────────────┘
```

### 1.2 EscrowHold Data Model

```
EscrowHold {
    id: "ESC-20250115-001"

    // Order info
    order_id: "ORD-12345"
    buyer_id: "buyer_001"
    seller_id: "seller_002"

    // Amount
    amount: 100 Trust  // Số tiền đang giữ

    // Status
    status: "HOLDING"  // HOLDING | RELEASED | REFUNDED | DISPUTED

    // Timeline
    created_at: "2025-01-15 10:00:00"
    release_at: "2025-01-18 10:00:00"  // 3 days later
    released_at: null

    // Commission
    commission_amount: null,  // Sẽ tính khi release
    commission_rate: 0.05,    // 5%

    // Early release
    early_release: false,
    early_release_by: null,

    // Dispute
    dispute_id: null,
    locked_at: null
}
```

---

## 2. Platform Wallet Model

```
PlatformWallet {
    user_id: "PLATFORM"

    // Balance
    available_trust: 5_000_000,           // Tổng available (escrow + commission)
    total_trust: 5_000_000,

    // Escrow tracking
    escrow_holding: 4_500_000,            // Tổng escrow đang giữ

    // Commission tracking
    total_commission_collected: 500_000,   // Tổng commission đã collect (all time)
    withdrawable_commission: 450_000,      // Commission có thể rút ngay
    withdrawn_commission: 50_000,          // Commission đã rút

    // Validation
    // available_trust PHẢI = escrow_holding + withdrawable_commission
}
```

---

## 3. Commission Flow V2

### 3.1 Tổng quan

```
┌─────────────────────────────────────────────────────────────────┐
│              COMMISSION FLOW - PLATFORM WALLET V2               │
└─────────────────────────────────────────────────────────────────┘

KEY PRINCIPLE:
Commission KHÔNG trừ ngay khi release escrow.
Commission chỉ được THỰC SỤ trừ khi seller WITHDRAW.

Flow:
┌─────────────┐         ┌──────────────┐         ┌─────────────┐
│  Escrow     │         │   Seller     │         │  Seller     │
│  Release    │────────>│   Wallet     │────────>│  Withdraw   │
│             │         │              │         │             │
│ +95 Seller  │         │ +95 Available│         │ -100 Locked │
│ +5 Debt     │         │ +5 Debt      │         │ -5 Debt     │
└─────────────┘         └──────────────┘         └─────────────┘
                                                        │
                                                        ▼
                                               ┌──────────────┐
                                               │   Platform   │
                                               │   Wallet     │
                                               │              │
                                               │  +5 Trust    │
                                               └──────────────┘

Commission tracking:
- Escrow release: Ghi nhận commission_debt (+)
- Seller withdraw: Trừ commission_debt (-) + Transfer cho Platform
```

### 3.2 Commission Accrual (Khi Release Escrow)

```
┌─────────────────────────────────────────────────────────────────┐
│          COMMISSION ACCRUAL - ESCROW RELEASE                    │
└─────────────────────────────────────────────────────────────────┘

Scenario: Escrow 100 Trust được release

[STEP 1] Tính commission:

         escrow_amount = 100 Trust
         commission_rate = 5%
         commission = 5 Trust
         seller_receives = 95 Trust

[STEP 2] Platform deduct 100%:

         Platform Wallet: available_trust -= 100

[STEP 3] Credit Seller 95%:

         Seller Wallet: available_trust += 95

[STEP 4] Ghi nhận Commission Debt:

         CREATE Transaction {
           type: "CommissionAccrue",
           amount: +5,  // Positive = ghi nhận debt (NOT real money move)
           wallet_id: "WLT-SELLER-002",
           order_id: "ORD-123",
           description: "Commission debt accrued"
         }

         UPDATE wallets SET
           commission_debt += 5
         WHERE user_id = 'seller_002'

         Result:
         - Seller available: +95 Trust (thực nhận)
         - Seller commission_debt: +5 Trust (nợ platform)

[STEP 5] Platform ghi nhận (cho tracking):

         CREATE Transaction {
           type: "CommissionCollected",  // NOTE: Chỉ ghi nhận
           amount: +5,  // KHÔNG thực sự move tiền vào Platform
           wallet_id: "PLATFORM",
           description: "Commission recorded from seller"
         }

         UPDATE wallets SET
           total_commission_collected += 5
         WHERE user_id = 'PLATFORM'

⚠️ LƯU Ý QUAN TRỌNG:
- Commission KHÔNG transfer vào Platform available_trust
- Chỉ ghi nhận vào total_commission_collected (tracking field)
- Platform sẽ nhận tiền thực khi SELLER WITHDRAW
```

### 3.3 Commission Deduction (Khi Withdraw)

```
┌─────────────────────────────────────────────────────────────────┐
│          COMMISSION DEDUCTION - SELLER WITHDRAW                 │
└─────────────────────────────────────────────────────────────────┘

Scenario: Seller withdraw 100 Trust, commission_debt = 20 Trust

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

[STEP 4] Complete Seller Withdrawal:

         CREATE Transaction {
           type: "WithdrawalComplete",
           amount: -100,  // Trừ từ withdrawal_locked
           wallet_id: "WLT-SELLER-002"
         }

         UPDATE wallets SET
           withdrawal_locked -= 100,
           total_trust -= 100,
           commission_debt -= 5  // Trừ nợ commission
         WHERE user_id = 'seller_002'

         Result:
         - withdrawal_locked: -100 (unlock)
         - total_trust: -100
         - commission_debt: 20 → 15 (còn nợ 15)

[STEP 5] RELEASE FEE TO PLATFORM:

         ┌─────────────────────────────────────────────────────────┐
         │  FEE ĐƯỢC GIẢI PHÓNG HOÀN TOÀN VÀO PLATFORM WALLET      │
         └─────────────────────────────────────────────────────────┘

         CREATE Transaction {
           type: "CommissionReleased",
           amount: +5,
           wallet_id: "PLATFORM",
           seller_id: "seller_002",
           withdrawal_id: "WDL-001",
           description: "Commission released from withdrawal"
         }

         UPDATE wallets SET
           available_trust += 5,
           total_trust += 5,
           total_commission_collected += 5,
           withdrawable_commission += 5    // Số commission có thể rút
         WHERE user_id = 'PLATFORM'

         Result:
         - Platform available_trust: +5
         - Platform withdrawable_commission: +5 (có thể rút)
         - Platform total_commission_collected: +5 (tracking)

✅ KẾT QUẢ:
- Seller nhận: 95,000 VND vào ngân hàng
- Seller wallet: -100 Trust, -5 commission debt
- Platform wallet: +5 Trust available (có thể rút)
```

---

## 4. Ví dụ Thực tế

### 4.1 Kịch bản hoàn chỉnh

```
┌─────────────────────────────────────────────────────────────────┐
│            SCENARIO: FROM PURCHASE TO WITHDRAWAL                 │
└─────────────────────────────────────────────────────────────────┘

───────────────────────────────────────────────────────────────────
DAY 0: Purchase
───────────────────────────────────────────────────────────────────

Order #12345:
- Product: Gmail US Account
- Price: 200 Trust
- Buyer: buyer_001 (available: 500 Trust)
- Seller: seller_002 (available: 1000 Trust, commission_debt: 0, admin_debt: 300)

Transactions:
1. Buyer Wallet: available_trust = 500 - 200 = 300 Trust
2. Platform Wallet: available_trust = 0 + 200 = 200 Trust
3. EscrowHold created: amount = 200, status = HOLDING

State:
- Buyer: 300 Trust available
- Platform: 200 Trust available (holding escrow)
- Seller: 1000 Trust available, 300 Trust admin_debt (unchanged)

───────────────────────────────────────────────────────────────────
DAY 3: Auto-Release Escrow
───────────────────────────────────────────────────────────────────

Escrow release:
- escrow_amount = 200 Trust
- commission = 200 × 5% = 10 Trust
- seller_receives_before_debt = 190 Trust

Debt repayment:
- seller_debt = 300 Trust
- debt_to_repay = min(190, 300) = 190 Trust
- actual_credit = 190 - 190 = 0 Trust

Transactions:
1. Platform: available_trust = 200 - 200 = 0 Trust
2. Seller: available_trust = 1000 + 0 = 1000 Trust (không đổi)
3. Seller: admin_debt = 300 - 190 = 110 Trust
4. Seller: commission_debt = 0 + 10 = 10 Trust
5. Platform: total_commission_collected = 0 + 10 (tracking only)

State:
- Buyer: 300 Trust available (unchanged)
- Platform: 0 Trust available
- Seller: 1000 Trust available, 10 Trust commission debt, 110 Trust admin debt

───────────────────────────────────────────────────────────────────
DAY 5: Seller Withdraw 500 Trust
───────────────────────────────────────────────────────────────────

Withdrawal request:
- withdrawal_amount = 500 Trust
- commission_to_deduct = min(500 × 5%, 10) = min(25, 10) = 10 Trust
- actual_trust = 500 - 10 = 490 Trust
- vnd_amount = 490,000 VND

Transactions:
1. Lock: available_trust = 1000 - 500 = 500
           withdrawal_locked = 0 + 500 = 500
2. Validate: Pass (risk < 0.3)
3. Approve: Bank transfer 490,000 VND
4. Complete:
   - withdrawal_locked = 500 - 500 = 0
   - total_trust = 1000 - 500 = 500
   - commission_debt = 10 - 10 = 0
5. Platform:
   - available_trust = 0 + 10 = 10 Trust
   - withdrawable_commission = 0 + 10 = 10 Trust
   - total_commission_collected = 10 + 10 = 20

State:
- Buyer: 300 Trust available (unchanged)
- Platform: 10 Trust available (commission collected)
- Seller: 500 Trust available, 0 commission debt, 110 Trust admin debt
- Bank: Seller received 490,000 VND

───────────────────────────────────────────────────────────────────
SUMMARY
───────────────────────────────────────────────────────────────────

Money Flow:
1. Buyer deposit 500 Trust → Buyer wallet: 500 Trust
2. Buyer purchase 200 Trust → Buyer: 300, Platform: 200 (escrow)
3. Escrow release → Platform: 0, Seller: +0 (all to debt), debt: 300 → 110
4. Seller withdraw 500 Trust → Seller: 500, Platform: +10
5. Bank: Seller receives 490,000 VND

Commission:
- Total earned: 10 Trust (from 200 Trust sale)
- Collected: 10 Trust (when seller withdrew)
- Platform balance: +10 Trust available to use

Debt Repayment:
- Initial debt: 300 Trust
- Repaid from escrow: 190 Trust
- Remaining debt: 110 Trust
- Next sale will continue to repay debt

Key Points:
✅ Platform giữ escrow 200 Trust trong 3 ngày
✅ Seller nhận 0/190 (0%) khi release vì phải trả nợ
✅ Commission 10 Trust được ghi nhận debt
✅ Commission 10 Trust được trừ khi seller withdraw
✅ Platform thực sự nhận 10 Trust vào available_balance
✅ Debt system hoạt động: trừ tự động từ tiền bán hàng
```

---

## 5. Trạng thái Escrow

### 5.1 State Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│              ESCROW STATE TRANSITION DIAGRAM                     │
└─────────────────────────────────────────────────────────────────┘

                    [CREATED]
                        │
                        ▼
                    [HOLDING] ◄────────────────────┐
                        │                           │
                        │                           │
         ┌───────────────┼───────────────┐          │
         │               │               │          │
    [Buyer        [Time      [Admin    (Seller Cancel
     Confirm]     Pass]      Dispute]      REMOVED)
         │               │               │
         ▼               ▼               ▼
     [RELEASED]    [RELEASED]      [DISPUTED] ──────┤
         │               │               │
         └───────────────┴───────────────┘
                                 │
                                 ▼
                            [REFUNDED]

State Transitions:
1. CREATED → HOLDING: Escrow created after purchase
2. HOLDING → RELEASED: After 3 days OR buyer confirms
3. HOLDING → DISPUTED: Buyer raise dispute
4. DISPUTED → RELEASED: Admin resolve in favor of seller
5. DISPUTED → REFUNDED: Admin resolve in favor of buyer OR Auto-refund (2 days)

Note: Seller cancel removed in V2 - not applicable for digital goods
```

### 5.2 Bảng Trạng thái

| State | Mô tả | Platform Money | Seller Money | Buyer Money |
|-------|-------|----------------|--------------|-------------|
| CREATED | Mới tạo escrow | +amount | - | -amount |
| HOLDING | Đang giữ 3 ngày | Hold | - | - |
| RELEASED | Đã release cho seller | -amount | +(amount - commission - debt) | - |
| DISPUTED | Đang tranh chấp | Hold (locked) | - | - |
| REFUNDED | Đã hoàn tiền | -amount | - | +amount |

---

## 6. Key Differences from V1

| Aspect | V1 (Seller Wallet Escrow) | V2 (Platform Wallet) |
|--------|---------------------------|----------------------|
| **Escrow location** | Seller pending_balance | **Platform Wallet** |
| **Purchase flow** | Buyer → Seller.pending | Buyer → **Platform** |
| **Release flow** | Seller.pending → Seller.available | **Platform → Seller** (95%) |
| **Commission timing** | Ghi nhận debt, tracking only | Ghi nhận debt → Trừ khi withdraw |
| **Commission transfer** | Không move tiền | **Platform nhận khi seller withdraw** |
| **Refund** | Seller.pending → Buyer | **Platform → Buyer** |
| **Debt system** | Không có | **Admin debt với auto-repayment** |
| **Control** | Seller có tiền (locked state) | **Platform kiểm soát hoàn toàn** |
| **Security** | Seller có thể thấy tiền (locked) | **Seller không thấy cho đến khi release** |
| **Audit** | State changes | **Real money movements** |

---

## 7. Platform Wallet Balance Formula

```
Platform_Available_Trust = Σ(Active Escrows) + Withdrawable_Commission + Withdrawal_Locked

Trong đó:
- Active Escrows = Σ(EscrowHold WHERE status = HOLDING)
- Withdrawable_Commission = Tổng commission đã collect từ seller withdrawals
- Withdrawal_Locked = Số commission đang trong quá trình rút (pending bank transfer)
- Total_Commission_Collected = Tracking field (all time)

Validation:
available_trust == escrow_holding + withdrawable_commission + platform_withdrawal_locked
```

**Lưu ý quan trọng:**
- Commission được giải phóng HOÀN TOÀN vào Platform available_trust khi seller withdraw
- withdrawable_commission là số commission có thể rút ngay
- Platform có thể rút commission bằng Admin Withdraw Platform Fee flow
- Total_Commission_Collected = tracking field cho thống kê, không ảnh hưởng đến available

---

## 8. Summary of All Flows V2

| Flow | Luồng tiền | Platform Wallet Role |
|------|-----------|---------------------|
| **Deposit** | Bank → User Wallet | Không tham gia |
| **Purchase** | Buyer → **Platform** | **Nhận tiền**, giữ escrow |
| **Escrow Release** | **Platform** → Seller (after debt repayment) | **Trả tiền**, giữ lại 5% commission, trừ admin debt |
| **Withdrawal** | Seller → Bank<br>Commission → **Platform** | **Nhận commission thực** |
| **Refund** | **Platform** → Buyer | **Trả lại tiền** escrow |
| **Admin Deduct** | User → Void (or create debt) | Không tham gia |
| **Platform Withdraw** | **Platform** → Company Bank | **Rút commission** ra tài khoản công ty |

---

## Related Documents

- [Deposit Flows](deposit.md) - Nạp tiền cho Buyer và Vendor
- [Withdrawal Flows](withdrawal.md) - Rút tiền cho Vendor
- [Transaction Flows](transactions.md) - Mua hàng (Buyer)
- [Escrow System](escrow.md) - Escrow tự động release
- [Admin Operations](adjustment.md) - Admin deduct, lock, unlock
- [Reconciliation](reconciliation.md) - Đối soát hệ thống
