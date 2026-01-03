# Deposit Flows - Buyer & Vendor

## Tổng quan

**Deposit Flow** cho phép Buyer và Vendor nạp tiền vào wallet thông qua 3rd party payment gateway (VNPay, MoMo, Bank Transfer, etc.).

**Actors:**
- Buyer - Nạp tiền để mua hàng
- Vendor - Nạp tiền để trả commission hoặc các khoản khác
- 3rd Party Payment Gateway - Xử lý thanh toán
- System - Xử lý webhook và credit Trust

---

## 1. Trust Currency & Conversion

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

## 2. Deposit Flow (via 3rd Party)

### 2.1 Conditions/Requirements

┌─────────────────────────────────────────┐
│         ĐIỀU KIỆN THỰC HIỆN            │
└─────────────────────────────────────────┘

1. **Actors**: Buyer/Vendor, 3rd Party Payment Gateway, System
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

### 2.2 Flow Diagram

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

### 2.3 Edge Cases & Error Handling

| Case | Condition | Handling | User Message |
|------|-----------|----------|--------------|
| Invalid amount | < 10,000 or > 50,000,000 or not divisible by 1,000 | Reject with 400 | "Số tiền không hợp lệ. Min: 10,000, Max: 50,000,000 VND" |
| Payment expired | No webhook after 15 min | Mark as EXPIRED | "Giao dịch đã hết hạn. Vui lòng thử lại." |
| Payment cancelled | User cancels at gateway | Mark as CANCELLED | "Bạn đã hủy giao dịch." |
| Webhook duplicate | Same ref received twice | Ignore (idempotent) | - |
| Invalid signature | HMAC mismatch | Reject webhook, log | - (System alert) |
| Invariant failed | Balance doesn't match | ROLLBACK, alert admin | - (Admin intervention) |

---

## 3. Transaction Types

### 3.1 Deposit Transaction Types

| Type | Direction | Description |
|------|-----------|-------------|
| **DEPOSIT_PENDING** | - | Waiting for payment |
| **DEPOSIT_VND_RECEIVED** | CREDIT | VND received from gateway |
| **DEPOSIT_TRUST_CREDITED** | CREDIT | Trust added to wallet |
| **DEPOSIT_MANUAL** | CREDIT | Admin manual deposit |

### 3.2 Transaction Data Model

```javascript
{
  _id: ObjectId,
  tx_id: String,                  // "TXN-{ULID}"
  wallet_id: String,
  user_id: String,

  // Type & Direction
  tx_type: String,                // DEPOSIT_PENDING, DEPOSIT_TRUST_CREDITED, etc.
  direction: String,              // "CREDIT" (+) or "DEBIT" (-)

  // Amounts
  amount: Number,                 // Trust amount (float, rounded to 0.001)
  vnd_amount: Number,             // VND equivalent (nullable)

  // Balance Tracking
  balance_before: Number,
  balance_after: Number,
  balance_type: String,           // "AVAILABLE" | "WITHDRAWAL_LOCKED" | "DISPUTE_LOCKED"

  // Status
  status: String,                 // "PENDING" | "PROCESSING" | "COMPLETED" | "FAILED" | "EXPIRED" | "CANCELLED"

  // References
  reference_type: String,         // "deposit" | "withdrawal" | "order"
  reference_id: String,
  external_ref: String,           // Bank ref, payment gateway ref

  // Metadata
  initiated_by: String,           // user_id or "SYSTEM" or admin_id
  admin_note: String,

  created_at: DateTime,
  updated_at: DateTime,
  completed_at: DateTime
}
```

---

## 4. Admin Manual Deposit

### 4.1 Conditions/Requirements

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

### 4.2 Flow Diagram

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
[A3] Admin enters target user, amount, reason
         │
         ├── Validate:
         │   ├── User exists?
         │   ├── Amount valid (1-1,000,000)?
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
         ├── ├── Yes ──► Require supervisor approval (2FA)
         ├── ├── No ──► Continue to [A7]
         │
         ▼
[A6] Supervisor approves via 2FA
         │
         ├── Reject ──► END, notify admin
         ├── Approve ──► Continue
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
         ├── Type: ADMIN_CREDIT (DEPOSIT_MANUAL)
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

### 4.3 Edge Cases & Error Handling

| Case | Condition | Handling | Admin Message |
|------|-----------|----------|---------------|
| User not found | user_id doesn't exist | Return 404 | "User không tồn tại." |
| Wallet not exist | No wallet for user | Auto-create wallet | "Đã tạo wallet mới cho user." |
| Permission denied | No WALLET_DEPOSIT | Return 403 | "Bạn không có quyền thực hiện." |
| Supervisor rejected | Supervisor 2FA reject | Don't process | "Supervisor đã từ chối." |
| Invariant failed | Balance mismatch | Rollback, alert | - (System alert) |

---

## 5. API Endpoints

| Method | Endpoint | Description | Access |
|--------|----------|-------------|--------|
| POST | /api/v3/wallet/deposit/initiate | Initiate deposit via 3rd party | Buyer, Vendor |
| POST | /api/v3/wallet/deposit/webhook | Payment gateway webhook | Public (with signature) |
| GET | /api/v3/wallet/deposit/status/:tx_id | Check deposit status | Buyer, Vendor |
| POST | /api/v3/admin/wallets/deposit | Manual deposit by admin | Admin |
| GET | /api/v3/admin/wallets/deposits/history | List all deposits | Admin |

---

## 6. Business Rules Summary

| # | Rule |
|---|------|
| **BR_DEPOSIT_1** | 1000 VND = 1 Trust (cố định), làm tròn đến 0.001 Trust |
| **BR_DEPOSIT_2** | Min deposit: 10,000 VND (10 Trust) |
| **BR_DEPOSIT_3** | Max deposit: 50,000,000 VND (50,000 Trust) |
| **BR_DEPOSIT_4** | Payment expires: 15 minutes |
| **BR_DEPOSIT_5** | Webhook must be validated (HMAC signature) |
| **BR_DEPOSIT_6** | Duplicate webhook: Idempotent (ignore) |
| **BR_DEPOSIT_7** | Admin manual deposit requires reason (min 10 chars) |
| **BR_DEPOSIT_8** | Admin deposit > 100,000 Trust requires supervisor approval |
| **BR_DEPOSIT_9** | Every transaction must validate balance invariant |
| **BR_DEPOSIT_10** | All admin operations must have audit log |

---

## Related Documents

- [Wallet Overview](wallet-overview.md) - Tổng quan hệ thống
- [Withdrawal Flows](withdrawal.md) - Rút tiền cho Vendor
- [Purchase Flows](purchase.md) - Mua hàng (Buyer)
- [Admin Operations](adjustment.md) - Admin operations khác
