# Admin Operations - Wallet Adjustments

## Tổng quan

**Admin Operations** cho phép admin thực hiện các thao tác quản lý wallet system:

1. **Manual Deduct** - Trừ tiền từ user wallet (với Debt System)
2. **Manual Deposit** - Nạp tiền trực tiếp Trust cho user
3. **Lock Wallet** - Khóa ví user (dispute/investigation)
4. **Unlock Wallet** - Mở khóa ví user
5. **Platform Withdraw** - Rút commission từ Platform Wallet

**Actors:**
- Admin - Thực hiện operations
- Supervisor - Phê duyệt các operations lớn
- System - Auto-processing, validation

---

## 1. Manual Deduct Flow (với Debt System)

### 1.1 Conditions

┌─────────────────────────────────────────┐
│         ĐIỀU KIỆN THỰC HIỆN            │
└─────────────────────────────────────────┘

1. **Actors**: Admin, Supervisor (if large amount), User (target)
2. **Preconditions**:
   ├── Admin logged in with WALLET_DEDUCT permission
   ├── Target wallet exists
   └── Wallet có sufficient balance (hoặc cho phép tạo nợ)

3. **Input Requirements**:
   ├── target_user_id: String
   ├── trust_amount: Number
   ├── reason: String (min 20 chars, required)
   └── allow_debt: Boolean (optional)

4. **Business Rules**:
   ├── Nếu available >= amount → Trừ thẳng
   ├── Nếu available < amount → Phần thiếu chuyển thành **NỢ**
   ├── Debt được trừ TỰ ĐỘNG khi seller nhận tiền từ escrow release
   ├── Debt được trừ TỰ ĐỘNG khi user nạp tiền
   └── Amount > 10,000 Trust requires supervisor approval

### 1.2 Flow Diagram

```
┌─────────────────────────────────────────┐
│      FLOW ADMIN MANUAL DEDUCT V2        │
│         (WITH DEBT SYSTEM)             │
└─────────────────────────────────────────┘

[START] Admin vào Users > Chi tiết user > Ví > Điều chỉnh số dư
         │
         ▼
[B1] Hiển thị form điều chỉnh:

         ╔═════════════════════════════════════════╗
         ║  ĐIỀU CHỈNH SỐ DƯ                             ║
         ╠═════════════════════════════════════════╣
         ║                                               ║
         ║  User: user123 (seller_002)                   ║
         ║  Số dư hiện tại:                              ║
         ║  • Available: 200 Trust                       ║
         ║  • Withdrawal locked: 100 Trust               ║
         ║  • Total: 300 Trust                           ║
         ║  • Nợ hiện tại: 0 Trust                       ║
         ║                                               ║
         ║  ──────────────────────────────────────────   ║
         ║  Loại điều chỉnh:                             ║
         ║  ○ Cộng tiền                                  ║
         ║  ● Trừ tiền                                   ║
         ║                                               ║
         ║  Số Trust cần trừ: [___500___]                ║
         ║                                               ║
         ║  ⚠️ CẢNH BÁO: Số dư không đủ!                 ║
         ║  • Cần trừ: 500 Trust                         ║
         ║  • Available: 200 Trust                       ║
         ║  • Thiếu: 300 Trust                           ║
         ║                                               ║
         ║  ☑️ Cho phép User nợ số tiền thiếu            ║
         ║     → User sẽ nợ 300 Trust                    ║
         ║     → Khi user bán hàng, tiền sẽ tự động trừ nợ  ║
         ║                                               ║
         ║  Lý do * (tối thiểu 20 ký tự):                ║
         ║  ┌───────────────────────────────────────────┐║
         ║  │ User vi phạm chính sách gian lận, cần bồi thường │║
         ║  │   cho buyer theo case INV-2025-00123       │║
         ║  └───────────────────────────────────────────┘║
         ║                                               ║
         ║  [Xác nhận]                                   ║
         ╚═════════════════════════════════════════════════╝
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

         ╔═════════════════════════════════════════╗
         ║  XÁC NHẬN TRỪ TIỀN + TẠO NỢ                   ║
         ╠═════════════════════════════════════════╣
         ║                                               ║
         ║  Tổng số cần trừ: 500 Trust                   ║
         ║                                               ║
         ║  Chi tiết:                                    ║
         ║  ├── Trừ từ available: 200 Trust              ║
         ║  └── Chuyển thành NỢ: 300 Trust                ║
         ║                                               ║
         ║  Sau khi thực hiện:                           ║
         ║  • Available: 0 Trust                         ║
         ║  • Admin debt: 300 Trust                      ║
         ║                                               ║
         ║  ⚠️ User sẽ bị trừ nợ TỰ ĐỘNG khi:            ║
         ║  • Bán hàng và nhận tiền từ escrow release     ║
         ║  • Nạp tiền vào ví                             ║
         ║                                               ║
         ║  [Xác nhận]  [Hủy]                            ║
         ╚═════════════════════════════════════════════════╝
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

### 1.3 Auto Debt Repayment

Khi seller bán hàng và escrow release, hệ thống tự động trừ nợ:

```
═══════════════════════════════════════════════════════════════════
[AUTO DEBT REPAYMENT - INTEGRATION WITH ESCROW RELEASE]
═══════════════════════════════════════════════════════════════════

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

---

## 2. Manual Deposit Flow

### 2.1 Conditions

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

### 2.2 Flow Diagram

```
┌─────────────────────────────────────────┐
│      ADMIN MANUAL DEPOSIT FLOW          │
└─────────────────────────────────────────┘

[A1] Admin chọn "Nạp tiền thủ công"
         │
         ▼
[A2] Hiển thị form + Validate
         │
         ├── Check permission
         ├── Check amount threshold (>100K → supervisor approval)
         │
         ▼
[A7] Admin confirms → BEGIN TRANSACTION
         │
         ▼
[A10] Tạo Transaction (ADMIN_CREDIT)
         │
         ▼
[A11] Update Target Wallet (+amount)
         │
         ▼
[A14] Send notifications
         │
         ▼
         END
```

---

## 3. Lock Wallet Flow

### 3.1 Conditions

```
┌─────────────────────────────────────────┐
│         ĐIỀU KIỆN THỰC HIỆN            │
└─────────────────────────────────────────┘

1. **Actors**: Admin
2. **Preconditions**:
   ├── Admin logged in with WALLET_LOCK permission
   └── Target wallet exists with available > 0

3. **Input Requirements**:
   ├── target_user_id: String
   ├── amount: Number (<= available_trust)
   ├── reason: String (required)
   └── case_reference: String (required, e.g., ticket ID)

4. **Business Rules**:
   ├── available → dispute_locked (total không đổi)
   ├── Lock toàn bộ available → Wallet status: SUSPENDED
   └── Case reference BẮT BUỘC (audit trail)
```

### 3.2 Flow Diagram

```
┌─────────────────────────────────────────┐
│               FLOW ADMIN LOCK WALLET V2   │
└─────────────────────────────────────────┘

[START] Admin vào Users > Chi tiết user > Ví > Khóa ví
         │
         ▼
[B1] Hiển thị wallet info + form
         │
         ├── Validate amount, reason, case_ref
         ├── Check permission
         │
         ▼
[B6] Admin confirms → BEGIN TRANSACTION
         │
         ▼
[B7] Tạo DisputeLock record
         │
         ▼
[B9] Update Wallet:
         available_trust -= amount
         dispute_locked_trust += amount
         │
         ▼
[B11] COMMIT → Invalidate cache → Notify user
         │
         ▼
[END] Wallet locked successfully
```

---

## 4. Unlock Wallet Flow

### 4.1 Flow Diagram

```
┌─────────────────────────────────────────┐
│              FLOW ADMIN UNLOCK WALLET V2  │
└─────────────────────────────────────────┘

[START] Admin vào Users > Chi tiết user > Ví > Mở khóa ví
         │
         ▼
[B1] Get danh sách DisputeLocks active
         │
         ├── Hiển thị danh sách để admin chọn
         │
         ▼
[B5] Admin confirms → BEGIN TRANSACTION
         │
         ▼
[B7] Update Wallet:
         dispute_locked_trust -= amount
         available_trust += amount
         │
         ▼
[B8] Update DisputeLock → RESOLVED
         │
         ▼
[B11] Check if còn locks active:
         ├── COUNT == 0 ──► Update wallet status = "ACTIVE"
         └── COUNT > 0 ──► Keep status = "SUSPENDED"
         │
         ▼
[END] Wallet unlocked successfully
```

---

## 5. Platform Withdraw Fee Flow

### 5.1 Conditions

```
┌─────────────────────────────────────────┐
│         ĐIỀU KIỆN THỰC HIỆN            │
└─────────────────────────────────────────┘

1. **Actors**: Admin
2. **Preconditions**:
   ├── Admin logged in with PLATFORM_WITHDRAW permission
   └── Platform withdrawable_commission > 0

3. **Input Requirements**:
   ├── amount: Number (<= withdrawable_commission)
   └── Bank info

4. **Business Rules**:
   ├── Chỉ được rút withdrawable_commission
   ├── KHÔNG được rút escrow
   ├── Rút > 500,000 Trust (500M VND) cần supervisor approval
   └── Bắt buộc 2FA cho mọi withdrawal
```

### 5.2 Flow Diagram

```
┌─────────────────────────────────────────┐
│         FLOW ADMIN WITHDRAW PLATFORM FEE │
└─────────────────────────────────────────┘

[START] Admin vào Finance > Platform Wallet > Rút fee
         │
         ▼
[B1] Hiển thị Platform Wallet Overview
         │
         ├── Available: X Trust
         ├── Escrow holding: Y Trust (không thể rút)
         └── Fee có thể rút: Z Trust
         │
         ▼
[B2] Admin click "Rút fee" → Validate → Show confirmation
         │
         ├── Verify 2FA
         │
         ▼
[B8] Admin confirms → BEGIN TRANSACTION
         │
         ▼
[B10] Tạo PlatformWithdrawal record
         │
         ▼
[B11] Lock Platform Commission:
         withdrawable_commission -= amount
         platform_withdrawal_locked += amount
         │
         ▼
[B13] COMMIT → Trigger Bank Transfer
         │
         ▼
[B14] Call Bank API → Transfer VND
         │
         ├── Success ──► Go to completion flow
         ├── Failed ──► Retry 3 times
         │
         ▼
[Bank Webhook - Completion]
         │
         ▼
[BW4] Complete: Update Platform Wallet
         │
         ├── platform_withdrawal_locked -= amount
         ├── available_trust -= amount
         ├── total_trust -= amount
         ├── withdrawn_commission += amount
         │
         ▼
[BW6] Notify Finance Team + CEO
         │
         ▼
[END] Platform withdrawal completed
```

---

## 6. Business Rules Summary

| # | Rule |
|---|------|
| **BR_ADJUST_1** | Admin KHÔNG cần supervisor approval khi deduct |
| **BR_ADJUST_2** | Nếu user available < deduct amount → Phần thiếu chuyển thành **NỢ** |
| **BR_ADJUST_3** | Debt được trừ TỰ ĐỘNG khi seller nhận tiền từ escrow release |
| **BR_ADJUST_4** | Debt được trừ TỰ ĐỘNG khi user nạp tiền (deposit) |
| **BR_ADJUST_5** | Thứ tự ưu tiên khi nhận tiền: Trả nợ trước → Còn lại vào available |
| **BR_ADJUST_6** | User có debt sẽ thấy warning trong dashboard |
| **BR_ADJUST_7** | User có debt VẪN có thể bán hàng (để trả nợ) |
| **BR_ADJUST_8** | User có debt KHÔNG thể withdraw cho đến khi trả hết nợ |
| **BR_ADJUST_9** | Debt history được track chi tiết với từng repayment |
| **BR_ADJUST_10** | Lock wallet: available → dispute_locked (total không đổi) |
| **BR_ADJUST_11** | Unlock wallet: dispute_locked → available (total không đổi) |
| **BR_ADJUST_12** | Lock toàn bộ available → Wallet status: SUSPENDED |
| **BR_ADJUST_13** | Unlock hết locks → Wallet status: ACTIVE |
| **BR_ADJUST_14** | Mọi admin operation phải có audit log |
| **BR_ADJUST_15** | DisputeLock record track từng lần lock riêng biệt |
| **BR_ADJUST_16** | Admin cần case reference (ticket ID, investigation ID) khi lock |
| **BR_ADJUST_17** | Chỉ được rút `withdrawable_commission`, KHÔNG được rút escrow |
| **BR_ADJUST_18** | Cần permission `PLATFORM_WITHDRAW` |
| **BR_ADJUST_19** | Rút > 500,000 Trust cần supervisor approval |
| **BR_ADJUST_20** | Bắt buộc 2FA cho mọi platform withdrawal |

---

## 7. API Endpoints

| Method | Endpoint | Description | Access |
|--------|----------|-------------|--------|
| POST | /api/v3/admin/wallets/deposit | Manual deposit | Admin |
| POST | /api/v3/admin/wallets/deduct | Manual deduct | Admin |
| POST | /api/v3/admin/wallets/lock | Lock wallet | Admin |
| POST | /api/v3/admin/wallets/unlock | Unlock wallet | Admin |
| POST | /api/v3/admin/platform/withdraw | Withdraw platform fee | Admin |
| GET | /api/v3/admin/wallets/:id/debt | Get debt history | Admin |
| GET | /api/v3/admin/wallets/:id/locks | Get lock history | Admin |

---

## Related Documents

- [Wallet Overview](wallet-overview.md) - Tổng quan hệ thống
- [Escrow System](escrow.md) - Escrow auto-release với debt repayment
- [Withdrawal Flows](withdrawal.md) - Rút tiền với commission deduction
