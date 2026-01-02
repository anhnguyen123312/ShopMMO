# Escrow Flow V2 - Platform Wallet Architecture

## Tổng quan

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

## 2. Flow Auto-Release Escrow

### 2.1 Cron Job Schedule

```
┌─────────────────────────────────────────────────────────────────┐
│            CRON JOB: AUTO-RELEASE ESCROW                         │
└─────────────────────────────────────────────────────────────────┘

Schedule: Mỗi giờ (0 * * * *)
Duration: Chạy tối đa 50 phút (timeout trước cron tiếp theo)
Batch size: 100 escrows/batch
```

### 2.2 Flow Chi Tiết

```
┌─────────────────────────────────────────────────────────────────┐
│              FLOW AUTO-RELEASE ESCROW V2                         │
└─────────────────────────────────────────────────────────────────┘

[START] Cron job triggered at 10:00:00
         │
         ▼
[B1] Query escrows cần release:

         SELECT *
         FROM escrow_holds
         WHERE status = 'HOLDING'
           AND release_at <= NOW()
           AND dispute_id IS NULL
         ORDER BY release_at ASC
         LIMIT 100
         │
         ├── Không có escrows ──► END (No action)
         │
         ▼
[B2] Loop qua từng escrow record
         │
         ▼
[B3] Lấy escrow info:
         {
           id: "ESC-001",
           order_id: "ORD-123",
           seller_id: "seller_002",
           amount: 100 Trust,
           release_at: "2025-01-18 09:00:00"
         }
         │
         ▼
[B4] Tính commission:

         escrow_amount = 100 Trust
         commission_rate = 5%
         commission = 100 × 5% = 5 Trust
         seller_receives = 100 - 5 = 95 Trust
         │
         ▼
[B5] Validate Platform Wallet:

         GET /wallets/PLATFORM
         │
         ├── available_trust < 100 ──► 🚨 CRITICAL ALERT
         │                                  "Platform wallet thiếu tiền!"
         │                                  "Expected: 100, Actual: {available}"
         │                                  → STOP processing
         │                                  → Notify admin IMMEDIATELY
         │                                  → Possible leak detected
         │
         ├── available_trust >= 100 ──► Continue
         │
         ▼
[B6] Check Seller Debt:

         GET /wallets/seller_002
         │
         ├── admin_debt > 0 ──► [DR1] Auto Debt Repayment
         │
         ▼
[B7] BEGIN TRANSACTION
         │
         ▼
[B8] Tạo Transaction cho Platform (Deduct):

         INSERT INTO transactions {
           wallet_id: "PLATFORM",
           type: "EscrowReleasePlatform",
           amount: -100,
           order_id: "ORD-123",
           escrow_id: "ESC-001",
           description: "Release escrow to seller"
         }
         │
         ▼
[B9] Update Platform Wallet:

         UPDATE wallets SET
           available_trust = available_trust - 100,
           total_trust = total_trust - 100
         WHERE user_id = 'PLATFORM'

         -- Platform available: 500 → 400 Trust
         │
         ▼
[B10] Tính seller receives sau debt repayment:

         seller_receives = 95 Trust
         seller_debt = 300 Trust
         debt_to_repay = min(95, 300) = 95 Trust
         actual_credit = 95 - 95 = 0 Trust
         │
         ▼
[B11] Tạo Transaction cho Seller (Credit):

         INSERT INTO transactions {
           wallet_id: "WLT-SELLER-002",
           type: "EscrowReleaseSeller",
           amount: +0,  // Actual credit sau khi trừ nợ
           order_id: "ORD-123",
           escrow_id: "ESC-001",
           description: "Receive from escrow (after debt repayment)"
         }
         │
         ▼
[B12] Update Seller Wallet:

         UPDATE wallets SET
           available_trust = available_trust + 0,  // Không nhận được gì
           admin_debt = admin_debt - 95           // Giảm nợ
         WHERE user_id = 'seller_002'

         -- Seller available: 2,000 → 2,000 (không đổi)
         -- Seller debt: 300 → 205 Trust
         │
         ▼
[B13] Tạo Commission Transaction (Accrue):

         INSERT INTO transactions {
           wallet_id: "WLT-SELLER-002",
           type: "CommissionAccrue",
           amount: +5,  // NOTE: Positive = ghi nhận debt
           order_id: "ORD-123",
           escrow_id: "ESC-001",
           description: "Commission accrued (will deduct on withdrawal)"
         }
         │
         ▼
[B14] Update Seller Commission Debt:

         UPDATE wallets SET
           commission_debt = commission_debt + 5
         WHERE user_id = 'seller_002'

         -- Seller commission_debt: 20 → 25 Trust
         │
         ▼
[B15] Update Debt Repayment History:

         UPDATE admin_debt_transactions SET
           total_repaid = total_repaid + 95,
           remaining_debt = remaining_debt - 95,
           repayment_history = repayment_history || [{
             order_id: "ORD-123",
             amount: 95,
             repaid_at: NOW()
           }]
         WHERE user_id = 'seller_002' AND status != 'CLEARED'
         │
         ▼
[B16] Tạo Transaction Platform Commission (Record keeping):

         INSERT INTO transactions {
           wallet_id: "PLATFORM",
           type: "CommissionCollected",
           amount: +5,  // NOTE: Chỉ ghi nhận, KHÔNG move tiền
           seller_id: "seller_002",
           order_id: "ORD-123",
           escrow_id: "ESC-001",
           description: "Commission accrued from seller"
         }
         │
         ▼
[B17] Update Platform Commission Tracking:

         UPDATE wallets SET
           total_commission_collected = total_commission_collected + 5
         WHERE user_id = 'PLATFORM'

         -- Platform total_commission_collected: 1000 → 1005 Trust
         │
         ▼
[B18] Update EscrowHold:

         UPDATE escrow_holds SET
           status = 'RELEASED',
           released_at = NOW(),
           commission_amount = 5
         WHERE id = 'ESC-001'
         │
         ▼
[B19] Update Order:

         UPDATE orders SET
           escrow_status = 'RELEASED',
           completed_at = NOW()
         WHERE id = 'ORD-123'
         │
         ▼
[B20] COMMIT TRANSACTION
         │
         ├── Transaction fail ──► ROLLBACK
         │                            → Log error
         │                            → Continue next escrow
         │
         ▼
[B21] Invalidate cache:
         - Delete cache: wallet:PLATFORM
         - Delete cache: wallet:seller_002
         │
         ▼
[B22] Gửi notification cho seller:

         📧 Email + Push Notification:
         "Bạn nhận được 95 Trust từ đơn hàng ORD-123
          Tiền dùng để trả nợ: 95 Trust
          Nợ còn lại: 205 Trust
          Commission: 5 Trust
          Commission debt: 25 Trust"
         │
         ▼
[B23] Continue next escrow trong batch
         │
         ├── Vẫn còn escrows ──► Quay lại [B3]
         │
         └── Hết escrows ──► END
         │
         ▼
[END] Cron job completed
       Processed: 100 escrows
       Duration: 45 seconds
       Next run: 11:00:00
```

---

## 3. Flow Early Release (Buyer confirms)

### 3.1 Kịch bản

**Buyer nhận hàng sớm → Muốn release tiền cho seller ngay**

```
┌─────────────────────────────────────────────────────────────────┐
│              FLOW EARLY RELEASE (BUYER CONFIRMS)                │
└─────────────────────────────────────────────────────────────────┘

Trigger: Buyer click "Đã nhận hàng" trên order detail
Condition: escrow_status == HOLDING
Timeline: Bất kỳ lúc nào trong 3 ngày escrow
```

### 3.2 Flow Chi Tiết

```
┌─────────────────────────────────────────────────────────────────┐
│            FLOW EARLY RELEASE - BUYER CONFIRMS                   │
└─────────────────────────────────────────────────────────────────┘

[START] Buyer click "Đã nhận hàng"
         │
         ▼
[B1] Validate user:
         │
         ├── current_user != order.buyer_id ──► "Không phải đơn của bạn"
         │
         ▼
[B2] Check order status:

         GET /orders/ORD-123
         │
         ├── escrow_status != 'HOLDING' ──► "Đơn hàng không trong escrow"
         │                                        "Có thể đã release/refund"
         │
         ▼
[B3] Get EscrowHold:

         GET /escrow-holds/by-order/ORD-123
         │
         ├── status != 'HOLDING' ──► "Escrow không active"
         │
         ▼
[B4] Hiển thị confirmation dialog:

         ╔═══════════════════════════════════════════════════════╗
         ║  XÁC NHẬN ĐÃ NHẬN HÀNG                                  ║
         ╠═══════════════════════════════════════════════════════╣
         ║                                                       ║
         ║  Đơn hàng: ORD-123                                    ║
         ║  Sản phẩm: Gmail US Account                           ║
         ║  Số tiền: 100 Trust                                   ║
         ║                                                       ║
         ║  ⚠️ Sau khi xác nhận:                                   ║
         ║  • Seller sẽ nhận 95 Trust                             ║
         ║  • Commission: 5 Trust                                ║
         ║  • Bạn KHÔNG thể khiếu nại nữa                        ║
         ║                                                       ║
         ║  [Xác nhận]  [Hủy]                                    ║
         ╚═══════════════════════════════════════════════════════╝
         │
         ├── Buyer Hủy ──► Stop
         │
         ▼
[B5] Buyer Xác nhận → BEGIN TRANSACTION
         │
         ▼
[B6] Update EscrowHold (Trigger early release):

         UPDATE escrow_holds SET
           release_at = NOW(),  -- Set to now instead of +3 days
           early_release = true,
           early_release_by = 'buyer_001',
           early_release_at = NOW()
         WHERE id = 'ESC-001'
         │
         ▼
[B7] Tính commission:

         escrow_amount = 100 Trust
         commission = 100 × 5% = 5 Trust
         seller_receives = 95 Trust
         │
         ▼
[B8] Execute same logic as Auto-Release:

         [Tương tự bước B8-B20 trong Auto-Release]
         │
         ▼
[B9] COMMIT TRANSACTION
         │
         ▼
[B10] Invalidate cache + Notify:

         ├── Notify Buyer:
         │  "Cảm ơn bạn đã xác nhận!
         │   Đánh giá seller để cải thiện chất lượng dịch vụ."

         ├── Notify Seller:
         │  "Buyer đã xác nhận nhận hàng sớm!
         │   Bạn nhận 95 Trust từ ORD-123"

         ▼
[END] Early release completed
```

---

## 4. Commission Flow

### 4.1 Tổng quan Commission V2

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

### 4.2 Commission Accrual (Khi Release Escrow)

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

### 4.3 Commission Deduction (Khi Withdraw) - UPDATED

```
┌─────────────────────────────────────────────────────────────────┐
│          COMMISSION DEDUCTION - SELLER WITHDRAW (UPDATED)        │
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

[STEP 4] BEGIN TRANSACTION

[STEP 5] Complete Seller Withdrawal:

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

[STEP 6] RELEASE FEE TO PLATFORM (UPDATED):

         ┌─────────────────────────────────────────────────────────┐
         │  FEE ĐƯỢC GIẢI PHÓNG HOÀN TOÀN VÀO PLATFORM WALLET      │
         └─────────────────────────────────────────────────────────┘

         CREATE Transaction {
           type: "CommissionReleased",  // Đổi từ CommissionCollected
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
           withdrawable_commission += 5    // NEW: Số commission có thể rút
         WHERE user_id = 'PLATFORM'

         Result:
         - Platform available_trust: +5
         - Platform withdrawable_commission: +5 (có thể rút)
         - Platform total_commission_collected: +5 (tracking)

[STEP 7] COMMIT TRANSACTION

✅ KẾT QUẢ:
- Seller nhận: 95,000 VND vào ngân hàng
- Seller wallet: -100 Trust, -5 commission debt
- Platform wallet: +5 Trust available (có thể rút)
```

### 4.4 Platform Wallet Model Update

```
PlatformWallet {
    user_id: "PLATFORM"

    // Balance
    available_trust: 5_000_000,           // Tổng available (escrow + commission)
    total_trust: 5_000_000,

    // Escrow tracking
    escrow_holding: 4_500_000,            // Tổng escrow đang giữ

    // Commission tracking (UPDATED)
    total_commission_collected: 500_000,   // Tổng commission đã collect (all time)
    withdrawable_commission: 450_000,      // Commission có thể rút ngay
    withdrawn_commission: 50_000,          // Commission đã rút

    // Validation
    // available_trust PHẢI = escrow_holding + withdrawable_commission
}
```

---

## 5. Ví dụ Thực tế

### 5.1 Kịch bản hoàn chỉnh

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

## 6. Trạng thái Escrow

### 6.1 State Diagram

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

### 6.2 Bảng Trạng thái

| State | Mô tả | Platform Money | Seller Money | Buyer Money |
|-------|-------|----------------|--------------|-------------|
| CREATED | Mới tạo escrow | +amount | - | -amount |
| HOLDING | Đang giữ 3 ngày | Hold | - | - |
| RELEASED | Đã release cho seller | -amount | +(amount - commission - debt) | - |
| DISPUTED | Đang tranh chấp | Hold (locked) | - | - |
| REFUNDED | Đã hoàn tiền | -amount | - | +amount |

---

## 7. Error Handling

### 7.1 Platform Wallet Insufficient

```
┌─────────────────────────────────────────────────────────────────┐
│          CRITICAL: PLATFORM WALLET INSUFFICIENT                  │
└─────────────────────────────────────────────────────────────────┘

Scenario:
- Escrow release: 100 Trust
- Platform available_trust: 80 Trust
- Expected: 100 Trust
- Actual: 80 Trust
- Missing: 20 Trust 🚨

Action:
1. STOP release immediately
2. Create CRITICAL ALERT:
   "Platform wallet leak detected!
    Escrow: ESC-001
    Expected: 100 Trust
    Actual: 80 Trust
    Missing: 20 Trust
    Possible cause: Database inconsistency, bug, or hack"
3. Send URGENT email to admin team
4. Log full context:
   - Escrow details
   - Platform wallet state
   - Recent transactions
5. Block all escrow releases until resolved
6. Run reconciliation immediately

Recovery:
1. Check transaction history for Platform wallet
2. Identify missing 20 Trust
3. Manual adjust Platform wallet
4. Verify reconciliation passes
5. Resume escrow releases
```

### 7.2 Seller Wallet Suspended

```
┌─────────────────────────────────────────────────────────────────┐
│            WARNING: SELLER WALLET SUSPENDED                      │
└─────────────────────────────────────────────────────────────────┘

Scenario:
- Escrow ready to release
- Seller wallet status: SUSPENDED
- Reason: Under investigation

Action:
1. Hold escrow (continue to keep money in Platform)
2. Update EscrowHold:
   status = 'HOLDING'  // Keep holding
   note = 'Seller wallet suspended, holding escrow'
3. Notify admin:
   "Escrow ESC-001 ready to release
    Seller seller_002 is SUSPENDED
    Money held in Platform awaiting resolution"
4. Monitor seller wallet status
5. Auto-release when seller reactivated

Do NOT:
❌ Release to suspended wallet
❌ Refund to buyer (not a dispute)
❌ Transfer commission yet

Correct flow:
✅ Keep holding in Platform
✅ Wait for seller investigation complete
✅ Release when seller ACTIVE again
```

---

## 8. Key Differences from V1

| Aspect | V1 (Seller Wallet Escrow) | V2 (Platform Wallet) |
|--------|---------------------------|----------------------|
| **Escrow location** | Seller pending_balance | **Platform Wallet** |
| **Purchase flow** | Buyer → Seller.pending | Buyer → **Platform** |
| **Release flow** | Seller.pending → Seller.available | **Platform → Seller** (95%) |
| **Commission timing** | Ghi nhận debt, tracking only | Ghi nhận debt → Trừ khi withdraw |
| **Commission transfer** | Không move tiền | **Platform nhận khi seller withdraw** |
| **Refund** | Seller.pending → Buyer | **Platform → Buyer** |
| **Debt system** | Không có | **Admin debt with auto-repayment** |
| **Control** | Seller có tiền (locked state) | **Platform kiểm soát hoàn toàn** |
| **Security** | Seller có thể thấy tiền (locked) | **Seller không thấy cho đến khi release** |
| **Audit** | State changes | **Real money movements** |

---

## 9. Dispute & Refund Flow - Hoàn tiền

### 9.1 Tổng quan

**Dispute/Refund xảy ra trong các trường hợp:**
- Buyer yêu cầu hoàn tiền với bằng chứng (trong 3 ngày escrow)
- Seller không phản hồi trong 2 ngày → Tự động hoàn tiền
- Admin xử lý tranh chấp theo hướng có lợi cho buyer/seller

**Refund flow V2:**
```
Platform Wallet (đang giữ escrow) → Buyer Wallet
```

**Key differences từ V1:**
- V1: Seller.pending → Buyer (seller trả lại)
- V2: Platform → Buyer (platform trả lại từ escrow)
- NEW: Seller có 2 ngày phản hồi, nếu không → auto-refund
- NEW: Cả buyer và seller có thể update thêm bằng chứng
- NEW: Admin có thể hold thêm 1-7 ngày

### 9.2 Buyer Request Refund Flow (ENHANCED)

```
┌─────────────────────────────────────────────────────────────────┐
│       FLOW BUYER REQUEST REFUND V2 (ENHANCED)                   │
└─────────────────────────────────────────────────────────────────┘

[START] Buyer click "Yêu cầu hoàn tiền"
         │
         ▼
[B1] Validate user:
         │
         ├── current_user != order.buyer_id ──► "Không phải đơn của bạn"
         │
         ▼
[B2] Check order status:
         │
         ├── escrow_status != 'HOLDING' ──► "Đơn hàng không trong escrow"
         │
         ▼
[B3] Get EscrowHold:

         GET /escrow-holds/by-order/ORD-123
         │
         ├── status != 'HOLDING' ──► "Escrow không active"
         │
         ▼
[B4] Check timeline:
         │
         ├── NOW > release_at ──► "Đã quá thời hạn yêu cầu hoàn tiền"
         │
         ▼
[B5] Hiển thị form yêu cầu hoàn tiền (ENHANCED):

         ╔═══════════════════════════════════════════════════════════════╗
         ║  YÊU CẦU HOÀN TIỀN                                            ║
         ╠═══════════════════════════════════════════════════════════════╣
         ║                                                               ║
         ║  Đơn hàng: ORD-123                                            ║
         ║  Sản phẩm: Gmail US Account                                   ║
         ║  Số tiền: 100 Trust                                           ║
         ║  Ngày mua: 15/01/2025                                         ║
         ║  Escrow đến: 18/01/2025                                       ║
         ║                                                               ║
         ║  ─────────────────────────────────────────────────────────── ║
         ║  LÝ DO HOÀN TIỀN * (tối thiểu 20 ký tự)                       ║
         ║  ┌───────────────────────────────────────────────────────────┐║
         ║  │  Sản phẩm không đúng như mô tả, không thể đăng nhập      │║
         ║  └───────────────────────────────────────────────────────────┘║
         ║                                                               ║
         ║  HÌNH ẢNH BẰNG CHỨNG (tối đa 5 ảnh, mỗi ảnh < 5MB)           ║
         ║  ┌─────────────────────────────────────────────────────────┐ ║
         ║  │  📷 [Chọn ảnh]                                          │ ║
         ║  │                                                         │ ║
         ║  │  ┌────────┐  ┌────────┐  ┌────────┐                    │ ║
         ║  │  │ img1   │  │ img2   │  │  + Add │                    │ ║
         ║  │  │  ❌    │  │  ❌    │  │        │                    │ ║
         ║  │  └────────┘  └────────┘  └────────┘                    │ ║
         ║  └─────────────────────────────────────────────────────────┘ ║
         ║                                                               ║
         ║  Định dạng: JPG, PNG, GIF, WEBP                               ║
         ║                                                               ║
         ║  ⚠️ Lưu ý:                                                    ║
         ║  • Seller có 2 ngày để phản hồi                               ║
         ║  • Nếu seller không phản hồi → Tự động hoàn tiền              ║
         ║  • Admin có thể gia hạn thời gian nếu cần thêm xác minh       ║
         ║                                                               ║
         ║  [Gửi yêu cầu]  [Hủy]                                         ║
         ╚═══════════════════════════════════════════════════════════════╝
         │
         ├── Buyer Hủy ──► Stop
         │
         ▼
[B6] Validate input:
         │
         ├── reason.length < 20 ──► "Lý do phải ít nhất 20 ký tự"
         ├── images.count > 5 ──► "Tối đa 5 ảnh"
         ├── any image.size > 5MB ──► "Ảnh không được quá 5MB"
         ├── invalid image format ──► "Chỉ chấp nhận JPG, PNG, GIF, WEBP"
         │
         ▼
[B7] Upload images to storage:

         FOR each image:
           upload_path = "disputes/DSP-{id}/buyer_{index}.{ext}"
           upload_to_s3(image, upload_path)
         │
         ▼
[B8] Buyer submits → BEGIN TRANSACTION
         │
         ▼
[B9] Tạo DisputeCase:

         INSERT INTO dispute_cases {
           type: "REFUND_REQUEST",
           status: "PENDING",
           order_id: "ORD-123",
           escrow_id: "ESC-001",
           buyer_id: "buyer_001",
           seller_id: "seller_002",
           buyer_reason: "Sản phẩm không đúng như mô tả...",
           buyer_evidence_images: ["disputes/DSP-001/buyer_1.jpg", ...],
           buyer_updates: [],
           seller_response: null,
           seller_evidence_images: [],
           seller_updates: [],
           seller_deadline: NOW() + 2 DAYS,  // ⚡ 2 ngày deadline
           created_at: NOW()
         }
         │
         ▼
[B10] Lock Escrow:

         UPDATE escrow_holds SET
           status = "DISPUTED",
           dispute_id = "DSP-001",
           locked_at = NOW()
         WHERE id = 'ESC-001'

         -- Escrow bị lock, không thể auto-release
         │
         ▼
[B11] COMMIT TRANSACTION
         │
         ▼
[B12] Notify Seller (URGENT):

         📧 Email + Push + SMS:
         "⚠️ KHẨN: Buyer yêu cầu hoàn tiền đơn ORD-123

          Lý do: Sản phẩm không đúng như mô tả...
          Số tiền: 100 Trust

          🕐 BẠN CÓ 2 NGÀY ĐỂ PHẢN HỒI
          Deadline: 17/01/2025 10:00:00

          ⚠️ Nếu không phản hồi, tiền sẽ TỰ ĐỘNG hoàn cho buyer

          [Xem chi tiết và phản hồi]"

         Notify Admin:
         "Có dispute mới DSP-001 cần xử lý"
         │
         ▼
[END1] Dispute created, waiting for seller response

═══════════════════════════════════════════════════════════════════
[SELLER RESPONSE FLOW]
═══════════════════════════════════════════════════════════════════

[SR1] Seller vào Disputes > DSP-001 (or click notification)
        │
        ▼
[SR2] Check deadline:
        │
        ├── NOW > seller_deadline ──► "Đã hết hạn phản hồi"
        │                                (Dispute đã auto-resolve)
        │
        ▼
[SR3] Hiển thị form phản hồi seller:

        ╔═══════════════════════════════════════════════════════════════╗
        ║  PHẢN HỒI YÊU CẦU HOÀN TIỀN                                   ║
        ╠═══════════════════════════════════════════════════════════════╣
        ║                                                               ║
        ║  Đơn hàng: ORD-123                                            ║
        ║  Buyer: buyer_001                                             ║
        ║  Số tiền: 100 Trust                                           ║
        ║                                                               ║
        ║  ─────────────────────────────────────────────────────────── ║
        ║  📋 LÝ DO CỦA BUYER:                                          ║
        ║  "Sản phẩm không đúng như mô tả, không thể đăng nhập"         ║
        ║                                                               ║
        ║  📷 BẰNG CHỨNG CỦA BUYER:                                     ║
        ║  ┌────────┐  ┌────────┐                                       ║
        ║  │ img1   │  │ img2   │  (click để xem lớn)                   ║
        ║  └────────┘  └────────┘                                       ║
        ║                                                               ║
        ║  ─────────────────────────────────────────────────────────── ║
        ║  ⏰ DEADLINE: 17/01/2025 10:00 (còn 23h 45m)                   ║
        ║  ─────────────────────────────────────────────────────────── ║
        ║                                                               ║
        ║  PHẢN HỒI CỦA BẠN * (tối thiểu 20 ký tự)                      ║
        ║  ┌───────────────────────────────────────────────────────────┐║
        ║  │  Sản phẩm hoàn toàn đúng mô tả. Tài khoản vẫn hoạt động  │║
        ║  │  bình thường. Buyer có thể đăng nhập sai mật khẩu.       │║
        ║  └───────────────────────────────────────────────────────────┘║
        ║                                                               ║
        ║  HÌNH ẢNH BẰNG CHỨNG (tối đa 5 ảnh)                           ║
        ║  ┌─────────────────────────────────────────────────────────┐ ║
        ║  │  📷 [Chọn ảnh]                                          │ ║
        ║  │  ┌────────┐  ┌────────┐                                 │ ║
        ║  │  │ proof1 │  │  + Add │                                 │ ║
        ║  │  └────────┘  └────────┘                                 │ ║
        ║  └─────────────────────────────────────────────────────────┘ ║
        ║                                                               ║
        ║  💡 Gợi ý bằng chứng:                                         ║
        ║  • Screenshot đăng nhập thành công                            ║
        ║  • Screenshot thông tin tài khoản                             ║
        ║  • Video màn hình (upload link)                               ║
        ║                                                               ║
        ║  [Gửi phản hồi]  [Hủy]                                        ║
        ╚═══════════════════════════════════════════════════════════════╝
        │
        ▼
[SR4] Validate & Upload images
        │
        ▼
[SR5] BEGIN TRANSACTION
        │
        ▼
[SR6] Update DisputeCase:

        UPDATE dispute_cases SET
          status = "SELLER_RESPONDED",
          seller_response = "Sản phẩm hoàn toàn đúng mô tả...",
          seller_evidence_images: ["disputes/DSP-001/seller_1.jpg", ...],
          seller_responded_at = NOW()
        WHERE id = 'DSP-001'
        │
        ▼
[SR7] COMMIT TRANSACTION
        │
        ▼
[SR8] Notify:

        ├── Buyer:
        "Seller đã phản hồi yêu cầu hoàn tiền của bạn
         Admin sẽ xem xét và đưa ra quyết định"

        ├── Admin:
        "Dispute DSP-001: Seller đã phản hồi
         Cần admin review và quyết định"
        │
        ▼
[END2] Waiting for admin decision

═══════════════════════════════════════════════════════════════════
[AUTO-REFUND CRON JOB]
═══════════════════════════════════════════════════════════════════

Schedule: Mỗi 30 phút (0,30 * * * *)

[AR1] Query disputes cần auto-refund:

        SELECT * FROM dispute_cases
        WHERE status = 'PENDING'           -- Seller CHƯA phản hồi
          AND seller_response IS NULL
          AND NOW() > seller_deadline      -- Quá 2 ngày
        │
        ├── Không có ──► END
        │
        ▼
[AR2] Loop qua từng dispute:
        │
        ▼
[AR3] Get related info:

        escrow = GET escrow_holds WHERE id = dispute.escrow_id
        order = GET orders WHERE id = dispute.order_id
        │
        ▼
[AR4] BEGIN TRANSACTION
        │
        ▼
[AR5] Validate Platform Wallet:

        GET /wallets/PLATFORM
        │
        ├── available_trust < escrow.amount ──► 🚨 CRITICAL ALERT
        │
        ▼
[AR6] Process Refund (same as manual refund):

        -- Platform Wallet - Escrow Amount
        -- Buyer Wallet + Escrow Amount
        -- Update escrow status = REFUNDED
        -- Update order status = REFUNDED
        │
        ▼
[AR7] Update DisputeCase:

        UPDATE dispute_cases SET
          status = "AUTO_REFUNDED",
          resolved_by = "SYSTEM_AUTO",
          resolution = "AUTO_REFUNDED",
          admin_note = "Seller không phản hồi trong 2 ngày, tự động hoàn tiền",
          resolved_at = NOW()
        WHERE id = dispute.id
        │
        ▼
[AR8] COMMIT TRANSACTION
        │
        ▼
[AR9] Notify:

        ├── Buyer:
        "✅ Tiền đã được tự động hoàn!

         Đơn hàng: ORD-123
         Số tiền: 100 Trust
         Lý do: Seller không phản hồi trong 2 ngày

         Số dư mới: 400 Trust"

        ├── Seller:
        "⚠️ Đơn hàng ORD-123 đã bị hoàn tiền tự động

         Lý do: Bạn không phản hồi dispute trong 2 ngày
         Số tiền: 100 Trust

         ⚠️ Việc không phản hồi dispute sẽ ảnh hưởng đến
         điểm uy tín của shop"

        ├── Admin:
        "Dispute DSP-001 đã tự động hoàn tiền
         Seller không phản hồi trong 2 ngày"
        │
        ▼
[END3] Auto-refund completed

═══════════════════════════════════════════════════════════════════
[ADMIN PROCESSING - ENHANCED]
═══════════════════════════════════════════════════════════════════

[A1] Admin vào Disputes > DSP-001
        │
        ▼
[A2] Xem chi tiết dispute (ENHANCED VIEW):

        ╔═══════════════════════════════════════════════════════════════╗
        ║  DISPUTE DETAILS - DSP-001                                    ║
        ╠═══════════════════════════════════════════════════════════════╣
        ║                                                               ║
        ║  📋 THÔNG TIN CHUNG                                           ║
        ║  ─────────────────────────────────────────────────────────── ║
        ║  Order: ORD-123         Status: SELLER_RESPONDED              ║
        ║  Buyer: buyer_001       Seller: seller_002                    ║
        ║  Amount: 100 Trust      Created: 15/01/2025 10:00             ║
        ║  Deadline: 17/01/2025 10:00 (còn 5h)                          ║
        ║                                                               ║
        ║  ─────────────────────────────────────────────────────────── ║
        ║  📜 TIMELINE TRAO ĐỔI                                         ║
        ║  ─────────────────────────────────────────────────────────── ║
        ║                                                               ║
        ║  ┌─ [15/01 10:00] 🛒 BUYER REQUEST ─────────────────────────┐ ║
        ║  │ "Sản phẩm không đúng như mô tả, không thể đăng nhập"     │ ║
        ║  │ 📷 [img1] [img2]                                         │ ║
        ║  └──────────────────────────────────────────────────────────┘ ║
        ║                                                               ║
        ║  ┌─ [16/01 12:00] 🏪 SELLER RESPONSE ───────────────────────┐ ║
        ║  │ "Sản phẩm hoàn toàn đúng mô tả. Tài khoản vẫn hoạt động" │ ║
        ║  │ 📷 [proof1]                                              │ ║
        ║  └──────────────────────────────────────────────────────────┘ ║
        ║                                                               ║
        ║  ┌─ [16/01 14:00] 🛒 BUYER UPDATE ──────────────────────────┐ ║
        ║  │ "Đã thử lại nhiều lần vẫn không được"                    │ ║
        ║  │ 📷 [error_screenshot]                                    │ ║
        ║  └──────────────────────────────────────────────────────────┘ ║
        ║                                                               ║
        ║  ─────────────────────────────────────────────────────────── ║
        ║  📊 THÔNG TIN BỔ SUNG                                         ║
        ║  ─────────────────────────────────────────────────────────── ║
        ║  Buyer history: 15 orders, 1 dispute (0% rate)               ║
        ║  Seller history: 234 orders, 5 disputes (2% rate)            ║
        ║  Product: Gmail US Account (ID: PRD-456)                      ║
        ║                                                               ║
        ╚═══════════════════════════════════════════════════════════════╝
        │
        ▼
[A3] Admin có 3 lựa chọn:

        ╔═══════════════════════════════════════════════════════════════╗
        ║  HÀNH ĐỘNG                                                    ║
        ╠═══════════════════════════════════════════════════════════════╣
        ║                                                               ║
        ║  ○ DUYỆT HOÀN TIỀN CHO BUYER                                  ║
        ║    → Tiền trả lại buyer, seller không nhận                    ║
        ║                                                               ║
        ║  ○ TỪ CHỐI, RELEASE CHO SELLER                                ║
        ║    → Escrow release bình thường, seller nhận 95%              ║
        ║                                                               ║
        ║  ○ GIA HẠN THỜI GIAN (Hold thêm N ngày)                       ║
        ║    → Cần thêm thời gian để 2 bên giải quyết                   ║
        ║                                                               ║
        ║  Ghi chú admin *:                                             ║
        ║  ┌───────────────────────────────────────────────────────────┐║
        ║  │                                                           │║
        ║  └───────────────────────────────────────────────────────────┘║
        ║                                                               ║
        ║  [Xác nhận]                                                   ║
        ╚═══════════════════════════════════════════════════════════════╝

─────────────────────────────────────────────────────────────────
[ADMIN EXTEND DEADLINE PATH] - MỚI
─────────────────────────────────────────────────────────────────

[EX1] Admin chọn "Gia hạn thời gian":

        ╔═══════════════════════════════════════════════════════════════╗
        ║  GIA HẠN THỜI GIAN DISPUTE                                    ║
        ╠═══════════════════════════════════════════════════════════════╣
        ║                                                               ║
        ║  Dispute: DSP-001                                             ║
        ║  Deadline hiện tại: 17/01/2025 10:00                          ║
        ║                                                               ║
        ║  Số ngày gia hạn *:                                           ║
        ║  ┌─────────┐                                                  ║
        ║  │    3    │ ngày (tối đa 7 ngày)                            ║
        ║  └─────────┘                                                  ║
        ║                                                               ║
        ║  Deadline mới: 20/01/2025 10:00                               ║
        ║                                                               ║
        ║  Lý do gia hạn *:                                             ║
        ║  ┌───────────────────────────────────────────────────────────┐║
        ║  │ Cần thêm thời gian để xác minh thông tin sản phẩm.       │║
        ║  │ Yêu cầu seller cung cấp thêm bằng chứng video.           │║
        ║  └───────────────────────────────────────────────────────────┘║
        ║                                                               ║
        ║  [Xác nhận gia hạn]  [Hủy]                                    ║
        ╚═══════════════════════════════════════════════════════════════╝
        │
        ▼
[EX2] Validate:
        │
        ├── extension_days < 1 ──► "Phải gia hạn ít nhất 1 ngày"
        ├── extension_days > 7 ──► "Tối đa 7 ngày"
        ├── reason.length < 20 ──► "Lý do phải ít nhất 20 ký tự"
        │
        ▼
[EX3] Update DisputeCase:

        UPDATE dispute_cases SET
          status = "EXTENDED",
          extended_by = "admin_001",
          extended_at = NOW(),
          extension_days = 3,
          new_deadline = seller_deadline + 3 DAYS,
          seller_deadline = seller_deadline + 3 DAYS,
          extension_reason = "Cần thêm thời gian xác minh..."
        WHERE id = 'DSP-001'
        │
        ▼
[EX4] Notify both parties:

        ├── Buyer:
        "Dispute DSP-001 đã được gia hạn

         Deadline mới: 20/01/2025 10:00
         Lý do: Cần thêm thời gian xác minh...

         Admin yêu cầu bạn cung cấp thêm thông tin nếu có"

        ├── Seller:
        "Dispute DSP-001 đã được gia hạn

         Deadline mới: 20/01/2025 10:00
         Lý do: Cần thêm thời gian xác minh...

         ⚠️ Vui lòng cung cấp thêm bằng chứng video"
        │
        ▼
[END4] Dispute extended, continue monitoring
```

### 9.3 Seller Cancel Order Flow - **REMOVED**

```
┌─────────────────────────────────────────────────────────────────┐
│              SELLER CANCEL ORDER - KHÔNG ÁP DỤNG                │
└─────────────────────────────────────────────────────────────────┘

❌ Flow này KHÔNG tồn tại trong V2

Lý do:
────────────────────────────────────────────────────────────────
Trong mô hình digital goods marketplace của TaphoaMMO:

1. Sản phẩm (tài khoản, key, etc.) được cung cấp NGAY LẬP TỨC
   sau khi buyer thanh toán thành công

2. Buyer đã nhận được sản phẩm ngay tại thời điểm purchase

3. Escrow period (3 ngày) chỉ là thời gian bảo vệ để buyer
   kiểm tra sản phẩm, KHÔNG phải thời gian để seller ship hàng

4. Seller không có lý do hợp lệ để cancel vì:
   - Sản phẩm đã giao
   - Không có shipping để delay
   - Stock đã được trừ tự động khi order

Kết luận:
────────────────────────────────────────────────────────────────
• Seller KHÔNG THỂ cancel order sau khi buyer đã nhận sản phẩm
• Nếu có vấn đề, phải thông qua Dispute flow
• Admin là người duy nhất có thể can thiệp
```

### 9.4 Dispute Business Rules (UPDATED)

| # | Rule |
|---|------|
| **BR8.1** | Buyer phải cung cấp lý do ít nhất 20 ký tự |
| **BR8.2** | Buyer có thể upload tối đa 5 ảnh làm bằng chứng (< 5MB) |
| **BR8.3** | Seller có **2 ngày** để phản hồi từ khi dispute tạo |
| **BR8.4** | Nếu seller không phản hồi trong 2 ngày → **Tự động hoàn tiền** |
| **BR8.5** | Admin có thể gia hạn thêm 1-7 ngày nếu cần xác minh |
| **BR8.6** | Cả buyer và seller có thể cập nhật thêm thông tin khi dispute đang xử lý |
| **BR8.7** | Mỗi update có thể đính kèm tối đa 3 ảnh |
| **BR8.8** | Ảnh bằng chứng được lưu trong S3/MinIO với path `disputes/{dispute_id}/` |
| **BR8.9** | Auto-refund cron job chạy mỗi 30 phút |
| **BR8.10** | Dispute timeline phải hiển thị đầy đủ cho cả 2 bên và admin |
| **BR8.11** | Seller KHỞI BỎ flow cancel order - không áp dụng cho digital goods |

---

## 10. Admin Operations - Thao tác quản trị

### 10.1 Tổng quan

Admin có thể thực hiện các thao tác thủ công trên wallet:

1. **Manual Deposit** - Nạp tiền trực tiếp Trust cho user
2. **Manual Deduct** - Trừ tiền từ user wallet (với Debt System)
3. **Lock Wallet** - Khóa ví user (dispute/investigation)
4. **Unlock Wallet** - Mở khóa ví user
5. **Withdraw Platform Fee** - Rút commission từ Platform Wallet

### 10.2 Admin Manual Deduct Flow (UPDATED - Debt System)

```
┌─────────────────────────────────────────────────────────────────┐
│         FLOW ADMIN MANUAL DEDUCT V2 (WITH DEBT SYSTEM)          │
└─────────────────────────────────────────────────────────────────┘

[START] Admin vào Users > Chi tiết user > Ví > Điều chỉnh số dư
         │
         ▼
[B1] Hiển thị form điều chỉnh (UPDATED):

         ╔═══════════════════════════════════════════════════════╗
         ║  ĐIỀU CHỈNH SỐ DƯ                                             ║
         ╠═══════════════════════════════════════════════════════╣
         ║                                                               ║
         ║  User: user123 (seller_002)                                   ║
         ║  Số dư hiện tại:                                              ║
         ║  • Available: 200 Trust                                       ║
         ║  • Withdrawal locked: 100 Trust                               ║
         ║  • Total: 300 Trust                                           ║
         ║  • Nợ hiện tại: 0 Trust                                       ║
         ║                                                               ║
         ║  ─────────────────────────────────────────────────────────── ║
         ║  Loại điều chỉnh:                                             ║
         ║  ○ Cộng tiền                                                  ║
         ║  ● Trừ tiền                                                   ║
         ║                                                               ║
         ║  Số Trust cần trừ: [___500___]                                ║
         ║                                                               ║
         ║  ⚠️ CẢNH BÁO: Số dư không đủ!                                 ║
         ║  • Cần trừ: 500 Trust                                         ║
         ║  • Available: 200 Trust                                       ║
         ║  • Thiếu: 300 Trust                                           ║
         ║                                                               ║
         ║  ☑️ Cho phép User nợ số tiền thiếu                            ║
         ║     → User sẽ nợ 300 Trust                                    ║
         ║     → Khi user bán hàng, tiền sẽ tự động trừ nợ               ║
         ║                                                               ║
         ║  Lý do * (tối thiểu 20 ký tự):                                ║
         ║  ┌───────────────────────────────────────────────────────────┐║
         ║  │ User vi phạm chính sách gian lận, cần bồi thường cho     │║
         ║  │ buyer theo case INV-2025-00123                            │║
         ║  └───────────────────────────────────────────────────────────┘║
         ║                                                               ║
         ║  [Xác nhận]                                                   ║
         ╚═══════════════════════════════════════════════════════════════╝
         │
         ▼
[B2] Validate input:
         │
         ├── amount <= 0 ──► "Số Trust phải lớn hơn 0"
         ├── reason.length < 20 ──► "Lý do phải ít nhất 20 ký tự"
         │
         ▼
[B3] Check permissions:

         Check admin has WALLET_DEDUCT permission
         │
         ├── No permission ──► "Bạn không có quyền thực hiện"
         │
         ▼
[B4] Calculate deduct breakdown:

         available = 200 Trust
         deduct_amount = 500 Trust

         IF available >= deduct_amount:
             actual_deduct = deduct_amount
             debt_amount = 0
         ELSE:
             actual_deduct = available      // 200 Trust
             debt_amount = deduct_amount - available  // 300 Trust
         │
         ▼
[B5] Nếu có debt → Hiển thị preview:

         ╔═══════════════════════════════════════════════════════╗
         ║  XÁC NHẬN TRỪTIỀN + TẠO NỢ                                   ║
         ╠═══════════════════════════════════════════════════════╣
         ║                                                               ║
         ║  Tổng số cần trừ: 500 Trust                                   ║
         ║                                                               ║
         ║  Chi tiết:                                                    ║
         ║  ├── Trừ từ available: 200 Trust                              ║
         ║  └── Chuyển thành NỢ: 300 Trust                               ║
         ║                                                               ║
         ║  Sau khi thực hiện:                                           ║
         ║  • Available: 0 Trust                                         ║
         ║  • Admin debt: 300 Trust                                      ║
         ║                                                               ║
         ║  ⚠️ User sẽ bị trừ nợ TỰ ĐỘNG khi:                            ║
         ║  • Bán hàng và nhận tiền từ escrow release                    ║
         ║  • Nạp tiền vào ví                                            ║
         ║                                                               ║
         ║  [Xác nhận]  [Hủy]                                            ║
         ╚═══════════════════════════════════════════════════════════════╝
         │
         ▼
[B6] Admin confirms → BEGIN TRANSACTION
         │
         ▼
[B7] Tạo Transaction trừ tiền thực:

         INSERT INTO transactions {
           wallet_id: "WLT-USER-123",
           type: "AdminDeduct",
           amount: -200,  // Chỉ trừ phần available
           admin_id: "admin_001",
           description: "Admin deduct (partial): Vi phạm chính sách",
           balance_before: 200,
           balance_after: 0
         }
         │
         ▼
[B8] Update User Wallet - Trừ available:

         UPDATE wallets SET
           available_trust = available_trust - 200,
           total_trust = total_trust - 200
         WHERE user_id = 'user123'
         │
         ▼
[B9] Tạo AdminDebtTransaction:

         INSERT INTO admin_debt_transactions {
           wallet_id: "WLT-USER-123",
           user_id: "user123",
           original_amount: 500,
           actual_deducted: 200,
           debt_amount: 300,
           reason: "Vi phạm chính sách gian lận...",
           admin_id: "admin_001",
           total_repaid: 0,
           remaining_debt: 300,
           repayment_history: [],
           status: "PENDING",
           created_at: NOW()
         }
         │
         ▼
[B10] Update Wallet Debt:

         UPDATE wallets SET
           admin_debt = admin_debt + 300,
           admin_debt_reason = "Vi phạm chính sách gian lận...",
           admin_debt_created_at = NOW(),
           admin_debt_created_by = "admin_001"
         WHERE user_id = 'user123'
         │
         ▼
[B11] Tạo Transaction ghi nhận nợ:

         INSERT INTO transactions {
           wallet_id: "WLT-USER-123",
           type: "AdminDebtCreated",
           amount: 0,  // Không thay đổi balance
           debt_amount: 300,
           admin_id: "admin_001",
           description: "Admin debt created: 300 Trust"
         }
         │
         ▼
[B12] Tạo AuditLog:

         INSERT INTO audit_logs {
           action: "ADMIN_DEDUCT_WITH_DEBT",
           admin_id: "admin_001",
           target_user: "user123",
           actual_deducted: 200,
           debt_created: 300,
           total_requested: 500,
           reason: "Vi phạm chính sách gian lận...",
           created_at: NOW()
         }
         │
         ▼
[B13] COMMIT TRANSACTION
         │
         ▼
[B14] Notify User:

         📧 Email + Push:
         "⚠️ THÔNG BÁO QUAN TRỌNG: Tài khoản bị điều chỉnh

          Admin đã trừ 500 Trust từ ví của bạn:
          • Trừ trực tiếp: 200 Trust
          • Số tiền NỢ: 300 Trust

          Lý do: Vi phạm chính sách gian lận...

          ⚠️ Số tiền nợ 300 Trust sẽ được tự động trừ khi
          bạn bán hàng và nhận tiền.

          Số dư hiện tại: 0 Trust
          Nợ hiện tại: 300 Trust

          Liên hệ support@taphoammo.com nếu có thắc mắc"
         │
         ▼
[END] Admin deduct with debt completed
```

### 10.3 Auto Debt Repayment (INTEGRATION WITH ESCROW RELEASE)

```
═══════════════════════════════════════════════════════════════════
[AUTO DEBT REPAYMENT - INTEGRATION WITH ESCROW RELEASE]
═══════════════════════════════════════════════════════════════════

Khi seller bán hàng và escrow release, hệ thống tự động trừ nợ:

[DR1] Escrow Release Flow đang chạy...
         │
         ▼
[DR2] Sau khi tính commission, trước khi credit seller:

         seller_receives = 95 Trust  // Sau commission
         seller_debt = GET wallet.admin_debt WHERE user_id = seller_id
         │
         ├── seller_debt == 0 ──► Continue normal flow
         │
         ▼
[DR3] Tính debt repayment:

         debt_to_repay = min(seller_receives, seller_debt)
                       = min(95, 300) = 95 Trust

         actual_credit = seller_receives - debt_to_repay
                       = 95 - 95 = 0 Trust
         │
         ▼
[DR4] Tạo Transaction Debt Repayment:

         INSERT INTO transactions {
           wallet_id: "WLT-SELLER-002",
           type: "AdminDebtRepayment",
           amount: -95,  // Trừ từ tiền nhận được
           order_id: "ORD-123",
           escrow_id: "ESC-001",
           description: "Auto debt repayment from sale"
         }
         │
         ▼
[DR5] Update Wallet Debt:

         UPDATE wallets SET
           admin_debt = admin_debt - 95,  // 300 - 95 = 205
           available_trust = available_trust + 0  // Không cộng
         WHERE user_id = 'seller_002'
         │
         ▼
[DR6] Update AdminDebtTransaction:

         UPDATE admin_debt_transactions SET
           total_repaid = total_repaid + 95,
           remaining_debt = remaining_debt - 95,
           status = CASE
             WHEN remaining_debt - 95 <= 0 THEN 'CLEARED'
             ELSE 'PARTIAL'
           END,
           repayment_history = repayment_history || [{
             order_id: "ORD-123",
             amount: 95,
             repaid_at: NOW()
           }]
         WHERE user_id = 'seller_002' AND status != 'CLEARED'
         │
         ▼
[DR7] Notify Seller:

         "💳 Tiền bán hàng đã được dùng để trả nợ

          Đơn hàng: ORD-123
          Tiền nhận được: 95 Trust
          Trừ trả nợ: 95 Trust
          Thực nhận: 0 Trust

          Nợ còn lại: 205 Trust"
         │
         ▼
[DR8] Continue escrow release với actual_credit
```

### 10.4 Admin Lock Wallet Flow

```
┌─────────────────────────────────────────────────────────────────┐
│               FLOW ADMIN LOCK WALLET V2                         │
└─────────────────────────────────────────────────────────────────┘

[START] Admin vào Users > Chi tiết user > Ví > Khóa ví
         │
         ▼
[B1] Hiển thị wallet info:

         ╔═══════════════════════════════════════════════════════╗
         ║  KHÓA VÍ - USER123                                    ║
         ╠═══════════════════════════════════════════════════════╣
         ║                                                       ║
         ║  Số dư hiện tại:                                       ║
         ║  • Available: 500,000 Trust                           ║
         ║  • Withdrawal locked: 0 Trust                         ║
         ║  • Dispute locked: 0 Trust                            ║
         ║  • Admin debt: 0 Trust                                ║
         ║  • Total: 500,000 Trust                               ║
         ║                                                       ║
         ║  Trạng thái ví: ACTIVE                                ║
         ║                                                       ║
         ║  Giao dịch gần đây:                                   ║
         ║  • 15/01: Deposit +100,000                            ║
         ║  • 14/01: Purchase -50,000                            ║
         ║  • 13/01: Withdrawal -200,000                         ║
         ╚═══════════════════════════════════════════════════════╝
         │
         ▼
[B2] Hiển thị form khóa ví:

         ╔═══════════════════════════════════════════════════════╗
         ║  KHÓA VÍ                                               ║
         ╠═══════════════════════════════════════════════════════╣
         ║                                                       ║
         ║  Số Trust muốn khóa:                                  ║
         ║  ┌───────────────────────────────────────────────────┐║
         ║  │ 500,000            [Khóa TOÀN BỘ]                │║
         ║  └───────────────────────────────────────────────────┘║
         ║                                                       ║
         ║  Lý do khóa *:                                        ║
         ║  ┌───────────────────────────────────────────────────┐║
         ║  │ Điều tra gian lận trong giao dịch, case ID:       │║
         ║  │   INV-2025-00123                                  │║
         ║  └───────────────────────────────────────────────────┘║
         ║                                                       ║
         ║  Case reference *:                                    ║
         ║  ┌───────────────────────────────────────────────────┐║
         ║  │ INV-2025-00123                                    │║
         ║  └───────────────────────────────────────────────────┘║
         ║                                                       ║
         ║  ⚠️ Lưu ý:                                           ║
         ║  • Số tiền bị khóa sẽ chuyển sang dispute_locked     ║
         ║  • User không thể dùng số tiền này                   ║
         ║  • Case reference là BẮT BUỘC                         ║
         ║                                                       ║
         ║  [Xác nhận khóa]  [Hủy]                                ║
         ╚═══════════════════════════════════════════════════════╝
         │
         ├── Admin Hủy ──► Stop
         │
         ▼
[B3] Validate input:
         │
         ├── amount <= 0 ──► "Số Trust phải lớn hơn 0"
         ├── amount > available_trust ──► "Không đủ available trust để lock"
         ├── reason.length < 10 ──► "Lý do bắt buộc"
         ├── case_ref empty ──► "Case reference bắt buộc khi lock wallet"
         │
         ▼
[B4] Check permissions:

         Check admin has WALLET_LOCK permission
         │
         ├── No ──► "Bạn không có quyền khóa ví"
         │
         ▼
[B5] Show preview:

         ╔═══════════════════════════════════════════════════════╗
         ║  XÁC NHẬN KHÓA VÍ                                     ║
         ╠═══════════════════════════════════════════════════════╣
         ║                                                       ║
         ║  Số tiền sẽ khóa: 500,000 Trust                       ║
         ║                                                       ║
         ║  Sau khi khóa:                                       ║
         ║  • Available: 0 Trust                                ║
         ║  • Dispute locked: 500,000 Trust                      ║
         ║  • Total: 500,000 Trust (không đổi)                  ║
         ║  • Status: SUSPENDED (toàn bộ bị khóa)              ║
         ║                                                       ║
         ║  User sẽ không thể:                                  ║
         ║  • Mua hàng                                           ║
         ║  • Rút tiền                                           ║
         ║  • Chuyển tiền                                        ║
         ║                                                       ║
         ║  [Xác nhận]  [Quay lại]                                ║
         ╚═══════════════════════════════════════════════════════╝
         │
         ├── Admin Quay lại ──► Stop
         │
         ▼
[B6] Admin confirms → BEGIN TRANSACTION
         │
         ▼
[B7] Tạo DisputeLock record:

         INSERT INTO dispute_locks {
           wallet_id: "WLT-USER-123",
           amount: 500000,
           reason: "Điều tra gian lận trong giao dịch",
           case_reference: "INV-2025-00123",
           locked_by: "admin_001",
           status: "ACTIVE",
           created_at: NOW()
         }
         │
         ▼
[B8] Tạo Transaction:

         INSERT INTO transactions {
           wallet_id: "WLT-USER-123",
           type: "AdminLock",
           amount: 0,  // Không thay đổi total
           description: "Lock available → dispute_locked",
           balance_before: 500000,
           balance_after: 500000
         }
         │
         ▼
[B9] Update Wallet:

         UPDATE wallets SET
           available_trust = available_trust - 500000,
           dispute_locked_trust = dispute_locked_trust + 500000
           -- total_trust không đổi
         WHERE user_id = 'user123'

         -- Available: 500,000 → 0
         -- Dispute locked: 0 → 500,000
         -- Total: 500,000 (unchanged)
         │
         ▼
[B10] Tạo AuditLog:

         INSERT INTO audit_logs {
           action: "WALLET_LOCK",
           admin_id: "admin_001",
           target_user: "user123",
           amount: 500000,
           reason: "Điều tra gian lận trong giao dịch",
           case_reference: "INV-2025-00123",
           created_at: NOW()
         }
         │
         ▼
[B11] COMMIT TRANSACTION
         │
         ▼
[B12] Check if lock toàn bộ available:

         IF available_trust == 0 AND dispute_locked > 0:
         └──→ Update wallet status = "SUSPENDED"
         │
         ▼
[B13] Invalidate cache + Notify:

        Notify User (email + push):
        "Ví của bạn đã bị khóa
         Lý do: Điều tra gian lận trong giao dịch
         Số tiền bị khóa: 500,000 Trust
         Case ID: INV-2025-00123
         Vui lòng liên hệ support để được giải đáp"

        ▼
[END] Wallet locked successfully
```

### 10.5 Admin Unlock Wallet Flow

```
┌─────────────────────────────────────────────────────────────────┐
│              FLOW ADMIN UNLOCK WALLET V2                        │
└─────────────────────────────────────────────────────────────────┘

[START] Admin vào Users > Chi tiết user > Ví > Mở khóa ví
         │
         ▼
[B1] Get danh sách DisputeLocks active:

         SELECT * FROM dispute_locks
         WHERE wallet_id = 'WLT-USER-123'
           AND status = 'ACTIVE'
         ORDER BY created_at DESC
         │
         ├── Không có locks ──► "Ví này không có lock nào"
         │
         ▼
[B2] Hiển thị danh sách locks:

         ╔═══════════════════════════════════════════════════════╗
         ║  MỞ KHÓA VÍ - USER123                                 ║
         ╠═══════════════════════════════════════════════════════╣
         ║                                                       ║
         ║  Các lượt khóa đang active:                           ║
         ║                                                       ║
         ║  ┌─────────────────────────────────────────────────┐ ║
         ║  │ [Radio] Lock #1 - 500,000 Trust                 │ ║
         ║  │         Lý do: Điều tra gian lận                │ ║
         ║  │         Case: INV-2025-00123                    │ ║
         ║  │         Locked by: admin_001                    │ ║
         ║  │         Locked at: 15/01/2025 10:00              │ ║
         ║  └─────────────────────────────────────────────────┘ ║
         ║                                                       ║
         ║  ┌─────────────────────────────────────────────────┐ ║
         ║  │ [Radio] Lock #2 - 100,000 Trust                 │ ║
         ║  │         Lý do: Dispute với seller               │ ║
         ║  │         Case: DSP-2025-00456                    │ ║
         ║  │         Locked by: admin_002                    │ ║
         ║  │         Locked at: 16/01/2025 14:30              │ ║
         ║  └─────────────────────────────────────────────────┘ ║
         ║                                                       ║
         ║  Total dispute locked: 600,000 Trust                 ║
         ║                                                       ║
         ║  ─────────────────────────────────────────────────── ║
         ║                                                       ║
         ║  Resolution note *:                                  ║
         ║  ┌───────────────────────────────────────────────────┐║
         ║  │ Điều tra hoàn tất, không phát hiện gian lận.     │║
         ║  │   User được mở khóa ví.                          │║
         ║  └───────────────────────────────────────────────────┘║
         ║                                                       ║
         ║  [Mở khóa]                                            ║
         ╚═══════════════════════════════════════════════════════╝
         │
         ▼
[B3] Admin chọn lock để unlock + enters resolution
         │
         ├── resolution.length < 20 ──► "Resolution note bắt buộc"
         │
         ▼
[B4] Check permissions:

         Check admin has WALLET_UNLOCK permission
         │
         ├── No ──► "Bạn không có quyền mở khóa ví"
         │
         ▼
[B5] Admin confirms → BEGIN TRANSACTION
         │
         ▼
[B6] Tạo Transaction:

         INSERT INTO transactions {
           wallet_id: "WLT-USER-123",
           type: "AdminUnlock",
           amount: 0,  // Không thay đổi total
           description: "Unlock dispute_locked → available",
           balance_before: 500000,
           balance_after: 500000
         }
         │
         ▼
[B7] Update Wallet:

         UPDATE wallets SET
           dispute_locked_trust = dispute_locked_trust - 500000,
           available_trust = available_trust + 500000
           -- total_trust không đổi
         WHERE user_id = 'user123'

         -- Dispute locked: 500,000 → 0
         -- Available: 0 → 500,000
         -- Total: 500,000 (unchanged)
         │
         ▼
[B8] Update DisputeLock:

         UPDATE dispute_locks SET
           status = "RESOLVED",
           resolved_by = "admin_001",
           resolved_at = NOW(),
           resolution: "Điều tra hoàn tất, không phát hiện gian lận"
         WHERE id = 'lock_id'
         │
         ▼
[B9] Tạo AuditLog:

         INSERT INTO audit_logs {
           action: "WALLET_UNLOCK",
           admin_id: "admin_001",
           target_user: "user123",
           amount: 500000,
           resolution: "Điều tra hoàn tất, không phát hiện gian lận",
           created_at: NOW()
         }
         │
         ▼
[B10] COMMIT TRANSACTION
         │
         ▼
[B11] Check if còn locks active:

         SELECT COUNT(*) FROM dispute_locks
         WHERE wallet_id = 'WLT-USER-123'
           AND status = 'ACTIVE'
         │
         ├── COUNT == 0 ──► Update wallet status = "ACTIVE"
         │                    "Ví đã được mở khóa hoàn toàn"
         │
         └── COUNT > 0 ──► Keep status = "SUSPENDED"
                        "Vẫn còn locks khác, ví vẫn bị khóa"
         │
         ▼
[B12] Invalidate cache + Notify:

        Notify User:
        "Ví của bạn đã được mở khóa
         Số tiền được mở khóa: 500,000 Trust
         Giải quyết: Điều tra hoàn tất, không phát hiện gian lận
         Bạn có thể giao dịch bình thường"

        ▼
[END] Wallet unlocked successfully
```

### 10.6 Admin Withdraw Platform Fee Flow (NEW)

```
┌─────────────────────────────────────────────────────────────────┐
│              FLOW ADMIN WITHDRAW PLATFORM FEE (NEW)             │
└─────────────────────────────────────────────────────────────────┘

[START] Admin vào Finance > Platform Wallet > Rút fee
         │
         ▼
[B1] Hiển thị Platform Wallet Overview:

         ╔═══════════════════════════════════════════════════════╗
         ║  PLATFORM WALLET                                              ║
         ╠═══════════════════════════════════════════════════════╣
         ║                                                               ║
         ║  💰 TỔNG QUAN                                                 ║
         ║  ─────────────────────────────────────────────────────────── ║
         ║  Available Trust: 5,000,000 Trust                             ║
         ║                                                               ║
         ║  Chi tiết:                                                    ║
         ║  ├── Escrow đang giữ: 4,500,000 Trust (không thể rút)        ║
         ║  └── Fee có thể rút: 500,000 Trust ✅                         ║
         ║                                                               ║
         ║  ─────────────────────────────────────────────────────────── ║
         ║  📊 THỐNG KÊ COMMISSION                                       ║
         ║  ─────────────────────────────────────────────────────────── ║
         ║  Tổng thu all time: 2,500,000 Trust                           ║
         ║  Đã rút: 2,000,000 Trust                                      ║
         ║  Có thể rút: 500,000 Trust                                    ║
         ║                                                               ║
         ║  ─────────────────────────────────────────────────────────── ║
         ║  📜 LỊCH SỬ RÚT GẦN ĐÂY                                       ║
         ║  ─────────────────────────────────────────────────────────── ║
         ║  [15/01] 100,000 Trust → VCB ****7890 ✅ Completed            ║
         ║  [10/01] 200,000 Trust → VCB ****7890 ✅ Completed            ║
         ║  [05/01] 150,000 Trust → VCB ****7890 ✅ Completed            ║
         ║                                                               ║
         ║  [Rút fee]  [Xem lịch sử đầy đủ]                              ║
         ╚═══════════════════════════════════════════════════════════════╝
         │
         ▼
[B2] Admin click "Rút fee" → Hiển thị form:

         ╔═══════════════════════════════════════════════════════╗
         ║  RÚT FEE PLATFORM                                             ║
         ╠═══════════════════════════════════════════════════════╣
         ║                                                               ║
         ║  Số dư có thể rút: 500,000 Trust                              ║
         ║                                                               ║
         ║  Số Trust muốn rút *:                                         ║
         ║  ┌─────────────────────────────────────────────────────────┐ ║
         ║  │  100,000                    [Rút tất cả]                 │ ║
         ║  └─────────────────────────────────────────────────────────┘ ║
         ║                                                               ║
         ║  Quy đổi: 100,000 Trust = 100,000,000 VND                     ║
         ║                                                               ║
         ║  ─────────────────────────────────────────────────────────── ║
         ║  THÔNG TIN NGÂN HÀNG                                          ║
         ║  ─────────────────────────────────────────────────────────── ║
         ║                                                               ║
         ║  Ngân hàng: Vietcombank (VCB)                                 ║
         ║  Số tài khoản: 1234567890                                     ║
         ║  Chủ tài khoản: CONG TY TNHH TAPHOAMMO                        ║
         ║                                                               ║
         ║  [Thay đổi tài khoản]                                         ║
         ║                                                               ║
         ║  ─────────────────────────────────────────────────────────── ║
         ║  Ghi chú (tùy chọn):                                          ║
         ║  ┌─────────────────────────────────────────────────────────┐ ║
         ║  │ Rút phí commission tháng 01/2025                        │ ║
         ║  └─────────────────────────────────────────────────────────┘ ║
         ║                                                               ║
         ║  [Tiếp tục]  [Hủy]                                            ║
         ╚═══════════════════════════════════════════════════════════════╝
         │
         ▼
[B3] Validate input:
         │
         ├── amount <= 0 ──► "Số Trust phải lớn hơn 0"
         ├── amount > withdrawable_commission ──► "Số dư không đủ"
         │
         ▼
[B4] Check permissions:

         Check admin has PLATFORM_WITHDRAW permission
         │
         ├── No permission ──► "Bạn không có quyền rút tiền platform"
         │
         ▼
[B5] Check threshold:

         IF amount > 500_000_000 VND (500,000 Trust):
         └── Require supervisor approval (2FA)
         │
         ▼
[B6] Hiển thị xác nhận:

         ╔═══════════════════════════════════════════════════════╗
         ║  XÁC NHẬN RÚT FEE                                             ║
         ╠═══════════════════════════════════════════════════════╣
         ║                                                               ║
         ║  Bạn sắp rút:                                                 ║
         ║                                                               ║
         ║  💰 100,000 Trust = 100,000,000 VND                           ║
         ║                                                               ║
         ║  Đến tài khoản:                                               ║
         ║  🏦 Vietcombank - 1234567890                                  ║
         ║     CONG TY TNHH TAPHOAMMO                                    ║
         ║                                                               ║
         ║  ─────────────────────────────────────────────────────────── ║
         ║  Sau khi rút:                                                 ║
         ║  • Fee có thể rút: 400,000 Trust                              ║
         ║  • Escrow đang giữ: 4,500,000 Trust (không đổi)               ║
         ║                                                               ║
         ║  ⚠️ Xác nhận bằng 2FA:                                        ║
         ║  ┌─────────────────────────────────────────────────────────┐ ║
         ║  │  [______]                                               │ ║
         ║  └─────────────────────────────────────────────────────────┘ ║
         ║                                                               ║
         ║  [Xác nhận rút tiền]  [Hủy]                                   ║
         ╚═══════════════════════════════════════════════════════════════╝
         │
         ▼
[B7] Verify 2FA:
         │
         ├── Invalid 2FA ──► "Mã xác thực không đúng"
         │
         ▼
[B8] Admin confirms → BEGIN TRANSACTION
         │
         ▼
[B9] Validate Platform Wallet again:

         GET /wallets/PLATFORM
         │
         ├── withdrawable_commission < amount ──► "Race condition detected"
         │                                          → Rollback
         │
         ▼
[B10] Tạo PlatformWithdrawal record:

         INSERT INTO platform_withdrawals {
           amount_trust: 100_000,
           amount_vnd: 100_000_000,
           bank_name: "Vietcombank",
           bank_account: "1234567890",
           account_holder: "CONG TY TNHH TAPHOAMMO",
           status: "PENDING",
           requested_by: "admin_001",
           requested_at: NOW(),
           admin_note: "Rút phí tháng 01/2025"
         }
         │
         ▼
[B11] Lock Platform Commission:

         UPDATE wallets SET
           withdrawable_commission -= 100_000,
           platform_withdrawal_locked += 100_000
         WHERE user_id = 'PLATFORM'

         -- Không trừ available_trust chưa (chờ bank confirm)
         │
         ▼
[B12] Tạo Transaction (Pending):

         INSERT INTO transactions {
           wallet_id: "PLATFORM",
           type: "PlatformWithdrawalPending",
           amount: 0,  // Chưa trừ
           withdrawal_id: "PLTW-001",
           description: "Platform withdrawal pending"
         }
         │
         ▼
[B13] COMMIT TRANSACTION
         │
         ▼
[B14] Trigger Bank Transfer:

         Call Internal Bank API:
         POST /bank/transfer {
           from: "COMPANY_ACCOUNT",
           to: {
             bank: "VCB",
             account: "1234567890",
             holder: "CONG TY TNHH TAPHOAMMO"
           },
           amount: 100_000_000,
           reference: "PLTW-001",
           note: "Platform fee withdrawal"
         }
         │
         ▼
[B15] Update status to PROCESSING:

         UPDATE platform_withdrawals SET
           status = "PROCESSING"
         WHERE id = 'PLTW-001'
         │
         ▼
[END1] Withdrawal initiated, waiting for bank confirmation

═══════════════════════════════════════════════════════════════════
[BANK WEBHOOK - COMPLETION]
═══════════════════════════════════════════════════════════════════

[BW1] Bank callback received:

         POST /webhooks/bank/platform-withdrawal
         {
           reference: "PLTW-001",
           status: "SUCCESS",
           bank_reference: "VCB-123456789",
           completed_at: "2025-01-15 14:00:00"
         }
         │
         ▼
[BW2] Validate callback (signature, etc.)
         │
         ▼
[BW3] BEGIN TRANSACTION
         │
         ▼
[BW4] Get withdrawal record:

         GET platform_withdrawals WHERE id = 'PLTW-001'
         │
         ├── status != 'PROCESSING' ──► "Invalid state transition"
         │
         ▼
[BW5] Update Platform Wallet - Complete:

         UPDATE wallets SET
           platform_withdrawal_locked -= 100_000,  // Unlock
           available_trust -= 100_000,             // Trừ actual
           total_trust -= 100_000,
           withdrawn_commission += 100_000         // Track total withdrawn
         WHERE user_id = 'PLATFORM'
         │
         ▼
[BW6] Tạo Transaction Complete:

         INSERT INTO transactions {
           wallet_id: "PLATFORM",
           type: "PlatformWithdrawalComplete",
           amount: -100_000,
           withdrawal_id: "PLTW-001",
           bank_reference: "VCB-123456789",
           description: "Platform withdrawal completed"
         }
         │
         ▼
[BW7] Update PlatformWithdrawal:

         UPDATE platform_withdrawals SET
           status = "COMPLETED",
           bank_reference = "VCB-123456789",
           completed_at = NOW()
         WHERE id = 'PLTW-001'
         │
         ▼
[BW8] Tạo AuditLog:

         INSERT INTO audit_logs {
           action: "PLATFORM_WITHDRAWAL_COMPLETED",
           admin_id: "admin_001",
           amount_trust: 100_000,
           amount_vnd: 100_000_000,
           bank_reference: "VCB-123456789",
           created_at: NOW()
         }
         │
         ▼
[BW9] COMMIT TRANSACTION
         │
         ▼
[BW10] Notify:

         📧 Email to Finance Team + CEO:
         "✅ Platform Withdrawal Completed

          Amount: 100,000 Trust (100,000,000 VND)
          Bank: Vietcombank - ****7890
          Reference: VCB-123456789

          Requested by: admin_001
          Completed at: 15/01/2025 14:00:00"
         │
         ▼
[END2] Platform withdrawal completed
```

### 10.7 Admin Operations Business Rules (UPDATED)

| # | Rule |
|---|------|
| **BR9.1** | Admin KHÔNG cần supervisor approval khi deduct |
| **BR9.2** | Nếu user available < deduct amount → Phần thiếu chuyển thành **NỢ** |
| **BR9.3** | Debt được trừ TỰ ĐỘNG khi seller nhận tiền từ escrow release |
| **BR9.4** | Debt được trừ TỰ ĐỘNG khi user nạp tiền (deposit) |
| **BR9.5** | Thứ tự ưu tiên khi nhận tiền: Trả nợ trước → Còn lại vào available |
| **BR9.6** | User có debt sẽ thấy warning trong dashboard |
| **BR9.7** | User có debt VẪN có thể bán hàng (để trả nợ) |
| **BR9.8** | User có debt KHÔNG thể withdraw cho đến khi trả hết nợ |
| **BR9.9** | Debt history được track chi tiết với từng repayment |
| **BR9.10** | Admin có thể xem debt status của user bất cứ lúc nào |
| **BR9.11** | Lock wallet: available → dispute_locked (total không đổi) |
| **BR9.12** | Unlock wallet: dispute_locked → available (total không đổi) |
| **BR9.13** | Lock toàn bộ available → Wallet status: SUSPENDED |
| **BR9.14** | Unlock hết locks → Wallet status: ACTIVE |
| **BR9.15** | User nhận email notification cho mọi admin operation |
| **BR9.16** | DisputeLock record track từng lần lock riêng biệt |
| **BR9.17** | Admin cần case reference (ticket ID, investigation ID) khi lock |
| **BR10.1** | Chỉ được rút `withdrawable_commission`, KHÔNG được rút escrow |
| **BR10.2** | Cần permission `PLATFORM_WITHDRAW` |
| **BR10.3** | Rút > 500,000 Trust (500M VND) cần supervisor approval |
| **BR10.4** | Bắt buộc 2FA cho mọi withdrawal |
| **BR10.5** | Bank transfer là async, cần webhook confirmation |
| **BR10.6** | Nếu bank fail → Rollback lock, trả lại withdrawable |
| **BR10.7** | Mọi withdrawal phải có audit trail đầy đủ |
| **BR10.8** | Email notification cho Finance Team + CEO |
| **BR10.9** | Platform Wallet balance validation trước và sau withdrawal |
| **BR10.10** | Reconciliation check: available = escrow_holding + withdrawable + locked |

---

## 11. Reconciliation - Đối soát hệ thống

### 11.1 Tổng quan

**Reconciliation đảm bảo:**
- ✅ Không có Trust bị leak hoặc tạo từ không khí
- ✅ Platform Wallet balance khớp với tổng escrows + withdrawable_commission
- ✅ Tổng Trust trong hệ thống = Tổng VND đã nạp / 1000
- ✅ Phát hiện bất thường để alert admin

**3 loại reconciliation:**
1. **Real-time Balance Check** - Mỗi transaction kiểm tra balance invariants
2. **Monthly Snapshot** - Tạo verified snapshot mỗi tháng
3. **Daily Full Reconciliation** - Reconcile toàn hệ thống mỗi ngày

### 11.2 Real-time Balance Check

```
┌─────────────────────────────────────────────────────────────────┐
│           REAL-TIME BALANCE CHECK - AFTER TRANSACTION           │
└─────────────────────────────────────────────────────────────────┘

Chạy: SAU MỌI TRANSACTION (in transaction)
Mục đích: Phát hiện ngay lập tức nếu có balance mismatch

[CHECK 1] Total == Sum of States?

        Formula:
        total_trust == available_trust + withdrawal_locked + dispute_locked + admin_debt

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

### 11.3 Monthly Snapshot Flow

```
┌─────────────────────────────────────────────────────────────────┐
│              MONTHLY SNAPSHOT - VERIFICATION                     │
└─────────────────────────────────────────────────────────────────┘

Schedule: Ngày 1 hàng tháng, 2:00 AM
Duration: ~30 phút (tùy số lượng wallets)

[START] Cron triggered: 2025-02-01 02:00:00
        │
        ▼
[B1] Xác định tháng cần snapshot:

        target_month = 2025-01 (previous month)
        start_date = 2025-01-01 00:00:00
        end_date = 2025-01-31 23:59:59
        │
        ▼
[B2] Query tất cả wallets:

        SELECT * FROM wallets
        WHERE status IN ('ACTIVE', 'SUSPENDED')
        │
        Result: 1,250 wallets
        │
        ▼
[B3] Loop qua từng wallet (batch 100):
        │
        ▼
[B4] Get wallet info:

        wallet_id = "WLT-USER-123"
        total_trust = 500000
        │
        ▼
[B5] Query tất cả transactions của wallet:

        SELECT * FROM transactions
        WHERE wallet_id = 'WLT-USER-123'
          AND created_at <= '2025-01-31 23:59:59'
        │
        Result: 245 transactions
        │
        ▼
[B6] Calculate balance from transactions:

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
[B7] Compare calculated vs actual:

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
[B8] Tạo MonthlySnapshot record:

        INSERT INTO monthly_snapshots {
          wallet_id: "WLT-USER-123",
          month: "2025-01",
          opening_balance: 400000,
          credits: 200000,
          debits: -100000,
          calculated_balance: 500000,
          actual_balance: 485000,
          discrepancy: 15000,
          status: "REQUIRE_MANUAL_REVIEW",  // or "VERIFIED"
          created_at: NOW()
        }
        │
        ▼
[B9] Handle discrepancy:

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
[B10] Continue next wallet
        │
        ├── Vẫn còn wallets ──► Quay lại [B4]
        │
        └── Hết wallets ──► [B11]
        │
        ▼
[B11] Generate monthly report:

        ╔═══════════════════════════════════════════════════════╗
        ║  MONTHLY SNAPSHOT REPORT - JANUARY 2025                ║
        ╠═══════════════════════════════════════════════════════╣
        ║                                                       ║
        ║  Total wallets: 1,250                                 ║
        ║  Verified: 1,240 (99.2%)                             ║
        ║  Discrepancy found: 10 (0.8%)                         ║
        ║                                                       ║
        ║  Discrepancy details:                                  ║
        ║  • Critical (>100): 2 wallets                        ║
        ║    - WLT-USER-123: 15,000 Trust                      ║
        ║    - WLT-USER-456: 8,500 Trust                       ║
        ║                                                       ║
        ║  • Warning (1-100): 8 wallets                        ║
        ║                                                       ║
        ║  Total discrepancy amount: 23,500 Trust               ║
        ║                                                       ║
        ║  Actions required:                                    ║
        ║  • [Investigate] WLT-USER-123                         ║
        ║  • [Investigate] WLT-USER-456                         ║
        ║                                                       ║
        ╚═══════════════════════════════════════════════════════╝
        │
        ▼
[B12] Email report to admin team:

        TO: admin-team@company.com
        SUBJECT: Monthly Wallet Snapshot - January 2025

        ... (report above) ...

        ▼
[END] Monthly snapshot completed
```

### 11.4 Daily Full Reconciliation Flow

```
┌─────────────────────────────────────────────────────────────────┐
│            DAILY FULL RECONCILIATION - 5 CHECKS                 │
└─────────────────────────────────────────────────────────────────┘

Schedule: Mỗi ngày 3:00 AM
Duration: ~15 phút

[START] Cron triggered: 2025-01-16 03:00:00
        │
        ▼
═══════════════════════════════════════════════════════════════════
[CHECK 1] System Total Trust
═══════════════════════════════════════════════════════════════════

[B1.1] Calculate total all wallets:

        total_all_wallets = Σ(wallet.total_trust)
                          = 50,000,000 Trust

[B1.2] Calculate total deposits:

        total_deposits = Σ(transactions WHERE type = 'DepositConvert')
                       = 60,000,000 Trust

[B1.3] Calculate total withdrawals:

        total_withdrawals = Σ(transactions WHERE type = 'WithdrawalComplete')
                          = 10,000,000 Trust

[B1.4] Compare:

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

═══════════════════════════════════════════════════════════════════
[CHECK 2] Platform Wallet Balance (UPDATED)
═══════════════════════════════════════════════════════════════════

[B2.1] Get Platform Wallet:

        SELECT * FROM wallets WHERE user_id = 'PLATFORM'

        available_trust = 5,000,000 Trust
        total_trust = 5,100,000 Trust
        withdrawable_commission = 500,000 Trust
        escrow_holding = 4,500,000 Trust (calculated)

[B2.2] Query active escrows:

        SELECT SUM(amount) FROM escrow_holds
        WHERE status = 'HOLDING'

        total_escrows = 4,500,000 Trust

[B2.3] Compare:

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

═══════════════════════════════════════════════════════════════════
[CHECK 3] VND ↔ Trust Reconciliation
═══════════════════════════════════════════════════════════════════

[B3.1] Sum VND deposits:

        total_vnd_deposits = Σ(DepositVND.vnd_amount)
                          = 50,000,000,000 VND

[B3.2] Sum Trust deposits:

        total_trust_deposits = Σ(DepositConvert.amount)
                             = 50,000,000 Trust

[B3.3] Compare:

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

═══════════════════════════════════════════════════════════════════
[CHECK 4] Withdrawal VND Reconciliation
═══════════════════════════════════════════════════════════════════

[B4.1] Sum withdrawal Trust:

        total_withdrawal_trust = Σ(WithdrawalComplete.amount)
                              = 8,000,000 Trust

[B4.2] Sum withdrawal VND:

        total_withdrawal_vnd = Σ(WithdrawalRequest.vnd_amount)
                            = 7,600,000,000 VND

[B4.3] Sum commission deducted:

        total_commission = Σ(CommissionDeduct.amount)
                         = 400,000 Trust

[B4.4] Calculate expected VND:

        expected_vnd = (total_withdrawal_trust - total_commission) * 1000
                     = (8,000,000 - 400,000) * 1000
                     = 7,600,000,000 VND

[B4.5] Compare:

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

═══════════════════════════════════════════════════════════════════
[CHECK 5] Money Flow Balance
═══════════════════════════════════════════════════════════════════

[B5.1] Calculate inflow:

        inflow = Σ(DepositConvert.amount)
               = 60,000,000 Trust

[B5.2] Calculate outflow:

        outflow = Σ(WithdrawalComplete.amount)
                = 10,000,000 Trust

[B5.3] Calculate remaining:

        remaining = inflow - outflow
                 = 60,000,000 - 10,000,000
                 = 50,000,000 Trust

[B5.4] Get actual total wallets:

        total_all_wallets = 50,000,000 Trust

[B5.5] Compare:

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

═══════════════════════════════════════════════════════════════════

[B6] Generate daily reconciliation report:

        ╔═══════════════════════════════════════════════════════╗
        ║  DAILY RECONCILIATION REPORT - 2025-01-16              ║
        ╠═══════════════════════════════════════════════════════╣
        ║                                                       ║
        ║  Check 1 - System Total Trust: ✅ PASSED              ║
        ║  Check 2 - Platform Wallet: ✅ PASSED                 ║
        ║  Check 3 - VND ↔ Trust: ✅ PASSED                     ║
        ║  Check 4 - Withdrawal VND: ✅ PASSED                  ║
        ║  Check 5 - Money Flow: ✅ PASSED                      ║
        ║                                                       ║
        ║  All checks passed successfully!                      ║
        ║                                                       ║
        ║  System summary:                                      ║
        ║  • Total wallets: 1,250                               ║
        ║  • Total Trust: 50,000,000                           ║
        ║  • Total VND deposited: 50,000,000,000                ║
        ║  • Platform escrows: 5,000,000                       ║
        ║                                                       ║
        ╚═══════════════════════════════════════════════════════╝

        IF any alerts:
        └──→ 📧 Send URGENT email with alert details

        ELSE:
        └──→ 📧 Send normal daily report

        ▼
[B7] Save report to database:

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

### 11.5 Reconciliation Business Rules

| # | Rule |
|---|------|
| **BR10.1** | Real-time check chạy sau MỌI transaction (in transaction) |
| **BR10.2** | Nếu real-time check fail → Consider rollback transaction |
| **BR10.3** | Monthly snapshot chạy ngày 1 hàng tháng lúc 2:00 AM |
| **BR10.4** | Snapshot discrepancy > 100 Trust → CRITICAL alert + manual review |
| **BR10.5** | Snapshot discrepancy <= 100 Trust → WARNING alert |
| **BR10.6** | Daily reconciliation chạy lúc 3:00 AM mỗi ngày |
| **BR10.7** | Daily reconciliation có 5 checks độc lập |
| **BR10.8** | Bất kỳ check nào fail → URGENT email to admin team |
| **BR10.9** | Reconciliation report lưu vào database để audit |
| **BR10.10** | Platform Wallet balance PHẢI luôn == Tổng active escrows + withdrawable_commission |

---

## Appendix A: Summary of All Flows V2

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

## Appendix B: Platform Wallet Balance Formula (UPDATED)

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

## Appendix C: Key Differences from V1

| Aspect | V1 (Seller Wallet Escrow) | V2 (Platform Wallet) |
|--------|---------------------------|----------------------|
| **Escrow Location** | Seller pending_balance | **Platform Wallet** |
| **Purchase Flow** | Buyer → Seller.pending | Buyer → **Platform** |
| **Release Flow** | Seller.pending → Seller.available | **Platform → Seller** (95%) |
| **Commission Timing** | Ghi nhận debt, tracking only | Ghi nhận debt → Trừ khi withdraw |
| **Commission Transfer** | Không move tiền | **Platform nhận khi seller withdraw** |
| **Refund** | Seller.pending → Buyer | **Platform → Buyer** |
| **Debt System** | Không có | **Admin debt với auto-repayment** |
| **Seller Cancel** | Có thể cancel | **KHÔNG - không áp dụng digital goods** |
| **Dispute** | Buyer request, admin quyết định | **Buyer request + by chứng + seller response + auto-refund 2 ngày** |
| **Platform Withdraw** | Không có | **Admin có thể rút commission** |
| **Control** | Seller có tiền (locked state) | **Platform kiểm soát hoàn toàn** |
| **Security** | Seller có thể thấy tiền (locked) | **Seller không thấy cho đến khi release** |
| **Audit** | State changes | **Real money movements** |

---

**End of Document**

Tài liệu này cung cấp đầy đủ flows cho Wallet V2 với kiến trúc Platform Wallet, bao gồm Escrow, Dispute & Refund (enhanced), Admin Operations (với Debt System), Platform Withdraw, và Reconciliation. Mọi giao dịch đều qua Platform Wallet để đảm bảo kiểm soát và an toàn.
