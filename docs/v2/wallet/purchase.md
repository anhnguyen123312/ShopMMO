# Purchase Flows - Buyer

## Tổng quan

**Purchase Flow** cho phép Buyer mua hàng và thanh toán bằng Trust. Tiền sẽ được chuyển từ Buyer Wallet → Platform Wallet (để giữ escrow).

**Actors:**
- Buyer - Mua hàng
- Seller - Nhận đơn hàng
- System - Xử lý giao dịch, tạo escrow
- Platform Wallet - Giữ escrow trong 3 ngày

**Key Features:**
- Buyer pays full price to Platform
- Platform holds in escrow (3 days)
- Seller receives after escrow release
- Commission deducted from seller's portion (không phải buyer)

---

## 1. Purchase Flow Overview

```
┌─────────────────────────────────────────────────────────────────┐
│              PURCHASE FLOW - BUYER TO PLATFORM                 │
└─────────────────────────────────────────────────────────────────┘

BUYER WALLET     PLATFORM WALLET       SELLER WALLET
    │                  │                    │
    │  Pay             │                    │
    ├──────────────────→│                    │
    │  -100 Trust      │ +100 Trust          │
    │                  │ (Hold escrow)       │
    │                  │                    │
    │         ─────────┴──────────────────────┐
    │                  │                      │
    ▼                  ▼                      ▼
  [Wait]      [Hold 3 days]          [Processing]
    │              │                      │
    │              │                      │
    │         ─────┴───────────────────────┐
    │                                      │
    │              After 3 days            │
    │         (Auto/Early release)         │
    │                                      │
    │              │                   [Receive]
    │              │                  -5 commission
    │              │                  +95 available
    │              │                      │
    ▼              ▼                      ▼
 Done         Released               Completed
```

---

## 2. User Purchase Flow

### 2.1 Conditions/Requirements

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

### 2.2 Flow Diagram

┌─────────────────────────────────────────┐
│        USER PURCHASE FLOW               │
└─────────────────────────────────────────┘

[B1] User chọn sản phẩm, nhấn "Mua ngay"
         │
         ▼
[B2] Hệ thống validate:
         │
         ├── Check stock: quantity >= requested?
         │   ├── No ──► "Sản phẩm đã hết hàng"
         │   └── Yes ──► Continue
         │
         └── Check balance: available >= total_price?
             ├── No ──► "Số dư không đủ. Vui lòng nạp thêm tiền."
             └── Yes ──► Continue
         │
         ▼
[B3] BEGIN DATABASE TRANSACTION
         │
         ▼
[B4] Tạo Order:
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
[B5] Deduct Buyer Wallet:
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
[B6] Credit Platform Wallet (Escrow):
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
[B7] Tạo EscrowHold:
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
[B8] Update Order:
         │
         ├── payment_status: PENDING → PAID
         └── order_status: PENDING → CONFIRMED
         │
         ▼
[B9] Decrease product stock:
         │
         └── product.stock -= quantity
         │
         ▼
[B10] Validate Invariants:
         │
         ├── Buyer invariant passed?
         ├── Platform invariant passed?
         └── Escrow amount matches?
         │
         ├── Any failed ──► ROLLBACK, alert admin
         ├── All passed ──► COMMIT
         │
         ▼
[B11] Send notifications:
         │
         ├── To Buyer: "Đặt hàng thành công. Mã đơn: ORD-xxx"
         └── To Seller: "Bạn có đơn hàng mới. ORD-xxx"
         │
         ▼
         END

### 2.3 Edge Cases & Error Handling

| Case | Condition | Handling | User Message |
|------|-----------|----------|--------------|
| Out of stock | stock < quantity | Reject with 400 | "Sản phẩm đã hết hàng." |
| Insufficient balance | available < total_price | Reject with 400 | "Số dư không đủ. Vui lòng nạp thêm." |
| Race condition | Concurrent purchase | Use optimistic lock with version | "Đã có người khác mua ngay trước đó. Vui lòng thử lại." |
| Transaction failed | DB error | Rollback all changes | "Giao dịch thất bại. Vui lòng thử lại." |
| Invariant failed | Balance mismatch | Rollback, alert admin CRITICAL | - (System alert) |

---

## 3. Transaction Types

### 3.1 Purchase Transaction Types

| Type | Direction | Description |
|------|-----------|-------------|
| **PURCHASE_DEBIT** | DEBIT | Buyer pays for order |
| **ESCROW_HOLD** | CREDIT | Platform receives escrow |
| **ESCROW_RELEASE_PLATFORM** | DEBIT | Platform releases escrow |
| **ESCROW_RELEASE_SELLER** | CREDIT | Seller receives payment |
| **COMMISSION_ACCRUE** | - | Record commission debt |
| **COMMISSION_RELEASED** | CREDIT | Platform receives commission |

### 3.2 EscrowHold Data Model

```javascript
{
  _id: ObjectId,
  id: String,                      // "ESC-{ULID}"

  // Order info
  order_id: String,                // "ORD-{ULID}"
  buyer_id: String,
  seller_id: String,

  // Amount
  amount: Number,                  // Số tiền đang giữ (float)

  // Status
  status: String,                  // "HOLDING" | "RELEASED" | "REFUNDED" | "DISPUTED"

  // Timeline
  created_at: DateTime,
  release_at: DateTime,            // 3 days later
  released_at: DateTime,

  // Commission
  commission_amount: Number,       // Sẽ tính khi release
  commission_rate: Number,         // 5%

  // Early release
  early_release: Boolean,
  early_release_by: String,

  // Dispute
  dispute_id: String,
  locked_at: DateTime
}
```

---

## 4. Order Status Flow

```
┌─────────────────────────────────────────────────────────────────┐
│              ORDER STATUS TRANSITION                            │
└─────────────────────────────────────────────────────────────────┘

[PENDING]
    │
    │ Buyer pays
    ▼
[CONFIRMED] ────────► [CANCELLED] (Seller cancels before ship)
    │
    │ Seller ships / Digital goods delivered
    ▼
[SHIPPED / DELIVERED]
    │
    │ 3 days escrow passes OR Buyer confirms
    ▼
[COMPLETED]

[DISPUTED] ────────► [REFUNDED] (Buyer wins)
              │
              └───────► [COMPLETED] (Seller wins)
```

---

## 5. Integration with Escrow System

Sau khi purchase hoàn tất:

1. **Escrow tự động release sau 3 ngày** → Xem [Escrow System](escrow.md)
2. **Buyer có thể confirm sớm** → Release ngay lập tức
3. **Buyer có thể khiếu nại (Dispute)** → Hold escrow cho đến khi resolve

---

## 6. API Endpoints

| Method | Endpoint | Description | Access |
|--------|----------|-------------|--------|
| POST | /api/v3/orders/create | Create order | Buyer |
| GET | /api/v3/orders/:id | Get order details | Buyer, Seller |
| GET | /api/v3/orders/my | List my orders | Buyer |
| POST | /api/v3/orders/:id/confirm | Confirm received (early release) | Buyer |
| POST | /api/v3/orders/:id/dispute | Open dispute | Buyer |
| GET | /api/v3/seller/orders/pending | List pending orders | Seller |

---

## 7. Business Rules Summary

| # | Rule |
|---|------|
| **BR_PURCHASE_1** | Buyer must have sufficient balance before purchase |
| **BR_PURCHASE_2** | Stock checked and locked before payment |
| **BR_PURCHASE_3** | Buyer pays full amount to Platform (escrow) |
| **BR_PURCHASE_4** | Platform holds escrow for 3 days |
| **BR_PURCHASE_5** | Seller receives after escrow release |
| **BR_PURCHASE_6** | Commission deducted from seller (not buyer) |
| **BR_PURCHASE_7** | Optimistic locking for race condition |
| **BR_PURCHASE_8** | All invariants validated before commit |
| **BR_PURCHASE_9** | Notifications sent to both parties |
| **BR_PURCHASE_10** | Digital goods delivered immediately |

---

## Related Documents

- [Wallet Overview](wallet-overview.md) - Tổng quan hệ thống
- [Escrow System](escrow.md) - Escrow auto-release flow
- [Dispute & Refund](wallet.md#dispute) - Khiếu nại và hoàn tiền
- [Deposit Flows](deposit.md) - Nạp tiền để mua hàng
