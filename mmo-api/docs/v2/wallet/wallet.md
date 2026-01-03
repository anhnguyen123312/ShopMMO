# Wallet System V2 - Platform Wallet Architecture

## Tài liệu hướng dẫn

Tài liệu này đã được tách thành các file nhỏ theo chức năng để dễ tham khảo:

---

## 1. [Tổng quan hệ thống](wallet-overview.md)

**Nội dung:**
- Trust Currency (Float, làm tròn đến 0.001)
- Platform Wallet Architecture
- Wallet Data Models
- Transaction Data Models
- Key Differences V1 vs V2

**Dành cho:** Tất cả actors - Đọc trước để hiểu tổng quan

---

## 2. [Deposit Flows - Nạp tiền](deposit.md)

**Actors:** Buyer, Vendor, Admin

**Nội dung:**
- User Deposit Flow (qua 3rd party: VNPay, MoMo, Bank Transfer)
- Admin Manual Deposit Flow
- Transaction Types
- Business Rules

**Flows:**
- Buyer/Vendor nạp tiền qua payment gateway
- Webhook handling từ 3rd party
- Admin nạp tiền thủ công cho user

---

## 3. [Withdrawal Flows - Rút tiền](withdrawal.md)

**Actors:** Vendor (Seller), System, Admin

**Nội dung:**
- Seller Withdrawal Flow (với Commission Deduction)
- Validation Engine (4 checks: Balance, Flow, Fraud, Limits)
- Admin Withdrawal Review
- Transaction Types

**Flows:**
- Seller yêu cầu rút tiền
- System tự động validate (auto-approve nếu risk_score < 0.3)
- Commission được trừ khi seller rút tiền
- Admin review các case cần manual approval

---

## 4. [Purchase Flows - Mua hàng](purchase.md)

**Actors:** Buyer, Seller, System

**Nội dung:**
- User Purchase Flow
- Order Status Flow
- Transaction Types
- Business Rules

**Flows:**
- Buyer mua hàng → Trừ tiền Buyer Wallet
- Tiền chuyển vào Platform Wallet (Escrow)
- Tạo EscrowHold record

---

## 5. [Escrow System - Giữ tiền & Tranh chấp](escrow.md)

**Actors:** System, Buyer, Seller, Admin

**Nội dung:**
- Auto-Release Escrow Flow (cron job)
- Early Release Flow (buyer confirms)
- Dispute & Refund Flow (buyer request, seller response, admin resolution)
- Auto-Refund Cron Job
- Business Rules

**Flows:**
- Sau 3 ngày → Auto-release escrow cho seller
- Buyer có thể confirm sớm → Release ngay
- Buyer có thể request refund (với bằng chứng ảnh)
- Seller có 2 ngày phản hồi, nếu không → Auto-refund
- Admin có thể resolve dispute hoặc extend deadline

---

## 6. [Admin Operations - Thao tác quản trị](adjustment.md)

**Actors:** Admin, Supervisor

**Nội dung:**
- Manual Deduct Flow (với Debt System)
- Auto Debt Repayment (tích hợp Escrow Release)
- Manual Deposit Flow
- Lock Wallet Flow
- Unlock Wallet Flow
- Platform Withdraw Fee Flow
- Business Rules

**Flows:**
- Admin trừ tiền user (có thể tạo nợ nếu không đủ balance)
- Debt được tự động trừ khi seller bán hàng hoặc nạp tiền
- Admin khóa/mở khóa ví user (dispute/investigation)
- Admin rút commission từ Platform Wallet về tài khoản công ty

---

## 7. [Reconciliation - Đối soát hệ thống](reconciliation.md)

**Actors:** System, Admin

**Nội dung:**
- Real-time Balance Check (sau mỗi transaction)
- Monthly Snapshot Flow (ngày 1 hàng tháng)
- Daily Full Reconciliation Flow (5 checks)
- Business Rules

**Flows:**
- Real-time: Kiểm tra balance invariants sau mỗi transaction
- Monthly: Tạo snapshot và verify balance từ transactions
- Daily: Reconcile toàn hệ thống với 5 checks độc lập

---

## Quick Reference

### Trust Currency

```
┌─────────────────────────────────────────┐
│            TRUST CURRENCY                │
├─────────────────────────────────────────┤
│  1000 VND = 1 Trust (Cố định)           │
│                                          │
│  • Type: Float (không phải integer)     │
│  • Rounding: 3 chữ số thập phân (0.001) │
│  • 0.001 Trust = 1 VND                  │
└─────────────────────────────────────────┘
```

### Platform Wallet Balance Formula

```
Platform_Available_Trust = Σ(Active Escrows) + Withdrawable_Commission + Withdrawal_Locked

Trong đó:
- Active Escrows = Σ(EscrowHold WHERE status = HOLDING)
- Withdrawable_Commission = Tổng commission đã collect từ seller withdrawals
- Withdrawal_Locked = Số commission đang trong quá trình rút
```

### Key Transaction Types

| Type | Direction | Description |
|------|-----------|-------------|
| **DEPOSIT_TRUST_CREDITED** | CREDIT | Trust added to wallet |
| **PURCHASE_DEBIT** | DEBIT | Buyer pays for order |
| **ESCROW_HOLD** | CREDIT | Platform receives escrow |
| **ESCROW_RELEASE_SELLER** | CREDIT | Seller receives payment |
| **COMMISSION_RELEASED** | CREDIT | Platform receives commission |
| **WITHDRAWAL_COMPLETED** | DEBIT | Finalize withdrawal |
| **AdminDeduct** | DEBIT | Admin deduct from user |
| **AdminDebtCreated** | - | Record debt (no balance change) |
| **AdminDebtRepayment** | DEBIT | Auto debt repayment |

---

## Summary of All Flows V2

| Flow | Luồng tiền | Platform Wallet Role |
|------|-----------|---------------------|
| **Deposit** | Bank → User Wallet | Không tham gia |
| **Purchase** | Buyer → **Platform** | **Nhận tiền**, giữ escrow |
| **Escrow Release** | **Platform** → Seller (95%) | **Trả tiền**, giữ 5% commission |
| **Withdrawal** | Seller → Bank<br>Commission → **Platform** | **Nhận commission thực** |
| **Refund** | **Platform** → Buyer | **Trả lại tiền** escrow |
| **Admin Deduct** | User → Void (or create debt) | Không tham gia |
| **Platform Withdraw** | **Platform** → Company Bank | **Rút commission** |

---

## Key Differences from V1

| Aspect | V1 (Seller Wallet Escrow) | V2 (Platform Wallet) |
|--------|---------------------------|----------------------|
| **Escrow Location** | Seller pending_balance | **Platform Wallet** |
| **Purchase Flow** | Buyer → Seller.pending | Buyer → **Platform** |
| **Release Flow** | Seller.pending → Seller.available | **Platform → Seller** (95%) |
| **Commission Timing** | Ghi nhận debt, tracking only | Ghi nhận debt → Trừ khi withdraw |
| **Commission Transfer** | Không move tiền | **Platform nhận khi seller withdraw** |
| **Refund** | Seller.pending → Buyer | **Platform → Buyer** |
| **Debt System** | Không có | **Admin debt với auto-repayment** |
| **Dispute** | Buyer request, admin quyết định | **Buyer request + bằng chứng + seller response + auto-refund 2 ngày** |
| **Platform Withdraw** | Không có | **Admin có thể rút commission** |
| **Control** | Seller có tiền (locked state) | **Platform kiểm soát hoàn toàn** |
| **Security** | Seller có thể thấy tiền (locked) | **Seller không thấy cho đến khi release** |

---

**End of Index**

Để đọc chi tiết từng phần, vui lòng xem các file tương ứng ở trên.
