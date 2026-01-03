# Escrow & Dispute System - Platform Wallet V2

## Tổng quan

**Escrow & Dispute System V2:**

Hệ thống escrow với Platform Wallet giữ tiền trong 3 ngày để bảo vệ buyer. Nếu có tranh chấp, hệ thống dispute cho phép khiếu nại và hoàn tiền.

**Key Differences V1 vs V2:**
| Aspect | V1 | V2 |
|--------|----|----|
| Escrow location | Seller pending_balance | **Platform Wallet** |
| Refund source | Seller.pending → Buyer | **Platform → Buyer** |
| Seller response time | 48h | **2 ngày (48h)** |
| Auto-refund | Không | **Có (nếu seller không phản hồi)** |
| Admin extend | Không | **Có (gia hạn 1-7 ngày)** |

---

## 1. Escrow Hold Data Model

```
EscrowHold {
    id: "ESC-{ULID}"

    // Order info
    order_id: "ORD-{ULID}"
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

## 2. Escrow State Transitions

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

### 2.1 Bảng Trạng thái

| State | Mô tả | Platform Money | Seller Money | Buyer Money |
|-------|-------|----------------|--------------|-------------|
| CREATED | Mới tạo escrow | +amount | - | -amount |
| HOLDING | Đang giữ 3 ngày | Hold | - | - |
| RELEASED | Đã release cho seller | -amount | +(amount - commission - debt) | - |
| DISPUTED | Đang tranh chấp | Hold (locked) | - | - |
| REFUNDED | Đã hoàn tiền | -amount | - | +amount |

---

## 3. Auto-Release Escrow Flow

### 3.1 Cron Job Schedule

```
┌─────────────────────────────────────────────────────────────────┐
│            CRON JOB: AUTO-RELEASE ESCROW                         │
└─────────────────────────────────────────────────────────────────┘

Schedule: Mỗi giờ (0 * * * *)
Duration: Chạy tối đa 50 phút (timeout trước cron tiếp theo)
Batch size: 100 escrows/batch
```

### 3.2 Flow Chi Tiết

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

## 4. Early Release Flow (Buyer Confirms)

### 4.1 Kịch bản

**Buyer nhận hàng sớm → Muốn release tiền cho seller ngay**

```
┌─────────────────────────────────────────────────────────────────┐
│              FLOW EARLY RELEASE (BUYER CONFIRMS)                │
└─────────────────────────────────────────────────────────────────┘

Trigger: Buyer click "Đã nhận hàng" trên order detail
Condition: escrow_status == HOLDING
Timeline: Bất kỳ lúc nào trong 3 ngày escrow
```

### 4.2 Flow Chi Tiết

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
         │  "Cảm ơn bạn đã xác nhận!"
         │   "Đánh giá seller để cải thiện chất lượng dịch vụ."

         ├── Notify Seller:
         │  "Buyer đã xác nhận nhận hàng sớm!"
         │   "Bạn nhận 95 Trust từ ORD-123"

         ▼
[END] Early release completed
```

---

## 5. Dispute & Refund Flow

### 5.1 Điều kiện tạo Dispute

```
┌─────────────────────────────────────────────────────────────────┐
│                ĐIỀU KIỆN TẠO DISPUTE V2                          │
└─────────────────────────────────────────────────────────────────┘

Buyer có thể tạo dispute khi TẤT CẢ điều kiện sau thỏa mãn:

1. Order Status:
   └── escrow_status = 'HOLDING' (đang giữ escrow)

2. Thời hạn:
   └── Trong vòng 3 ngày (72 giờ) kể từ created_at
   └── Chưa quá release_at

3. Chưa có dispute:
   └── Chưa tạo dispute cho order này
   └── Hoặc dispute trước đã được resolve

4. Lý do hợp lệ:
   └── Có lý do và bằng chứng (ảnh, video, etc.)
```

### 5.2 Các loại lý do khiếu nại

| Reason Code | Mô tả | Mức độ |
|-------------|-------|--------|
| wrong_item | Sản phẩm sai/không đúng mô tả | High |
| not_working | Sản phẩm không hoạt động (die) | High |
| duplicate | Sản phẩm đã từng mua/bị trùng | High |
| missing_items | Thiếu số lượng so với đặt | Medium |
| quality_issue | Chất lượng không như cam kết | Medium |
| partial_working | Một phần sản phẩm không hoạt động | Medium |
| other | Lý do khác | Low |

### 5.3 Buyer Request Refund Flow

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
[B5] Hiển thị form yêu cầu hoàn tiền:

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

          ⚠️ Nếu không phản hồi, tiền sẽ TỰ ĐỘNG hoàn cho buyer"

         Notify Admin:
         "Có dispute mới DSP-001 cần xử lý"
         │
         ▼
[END1] Dispute created, waiting for seller response
```

### 5.4 Dispute State Transitions

```
┌─────────────────────────────────────────────────────────────────┐
│              DISPUTE STATE TRANSITION V2                        │
└─────────────────────────────────────────────────────────────────┘

                    ┌─────────────┐
                    │   PENDING   │ ◄── Buyer vừa tạo, chờ seller
                    └──────┬──────┘
                           │
                           ▼
                    ┌─────────────┐
                    │SELLER_RESP  │ ◄── Seller đã phản hồi
                    └──────┬──────┘
                           │
              ┌────────────┼────────────┐
              │            │            │
              ▼            ▼            ▼
         ┌──────────┐ ┌──────────┐ ┌──────────┐
         │RESOLVED  │ │BUYER_RESP │ │ESCALATED │
         └──────────┘ └────┬─────┘ └────┬─────┘
         Seller chấp      │      Buyer/
         nhận refund      │      Seller escalate
                           │
              ┌────────────┼────────────┐
              │            │            │
              ▼            ▼            ▼
         ┌──────────┐ ┌──────────┐ ┌──────────┐
         │RESOLVED  │ │BUYER_RESP │ │ADMIN_REV │
         │          │ └────┬─────┘ └────┬─────┘
         │          │      │  Trao đổi  │
         │          │      │  tiếp      │
         │          │      ▼            ▼
         │          │   [Max 3 lượt]  │
         │          │      │            │
         │          │      └─────┬──────┘
         │          │            │
         │          ▼            ▼
         │      ┌─────────────────────┐
         │      │     ADMIN_REVIEW    │
         │      └──────────┬──────────┘
         │                 │
         ▼                 ▼
    ┌──────────┐    ┌──────────┐┌──────────┐
    │REFUNDED  │    │ PARTIAL  ││ CLOSED   │
    └──────────┘    │ REFUND   │└──────────┘
                     └──────────┘

State Transitions:
1. PENDING → SELLER_RESP: Seller phản hồi trong 2 ngày
2. SELLER_RESP → RESOLVED: Seller chấp nhận refund
3. SELLER_RESP → BUYER_RESP: Buyer phản hồi lại (24h)
4. BUYER_RESP → BUYER_RESP: Trao đổi tiếp (tối đa 3 lượt)
5. BUYER_RESP → ESCALATED: Buyer/seller escalate lên admin
6. BUYER_RESP → ADMIN_REVIEW: Đã 3 lượt, bắt buộc admin review
7. PENDING → ESCALATED: Seller không phản hồi trong 48h (auto-escalate)
8. ADMIN_REVIEW → REFUNDED/PARTIAL_REFUND/CLOSED: Admin quyết định
```

### Chi tiết trạng thái

| Status | Mô tả | Chờ action từ | Deadline |
|--------|-------|---------------|----------|
| pending | Mới tạo | Seller | 2 ngày (48h) |
| seller_responded | Seller đã phản hồi | Buyer | 24h |
| buyer_responded | Buyer phản hồi lại | Seller | 24h |
| escalated | Buyer/seller escalate | Admin | - |
| admin_review | Admin đang xem xét | Admin | - |
| resolved | Thỏa thuận refund | - | - |
| refunded | Đã hoàn tiền đầy đủ | - | - |
| partial_refund | Hoàn một phần | - | - |
| rejected | Dispute bị từ chối | - | - |
| closed | Đóng không xử lý | - | - |

### 5.5 Timeline Trao Đổi Dispute

```
┌─────────────────────────────────────────────────────────────────┐
│                 TIMELINE XỬ LÝ DISPUTE V2                      │
└─────────────────────────────────────────────────────────────────┘

T+0:      Buyer tạo dispute
          └── Seller có 48h (2 ngày) để phản hồi

T+48h:    Nếu seller không phản hồi
          └── AUTO-ESCALATE lên Admin (không phải auto-refund)
          └── Admin sẽ review và quyết định

T+0-48h:  Seller phản hồi
          └── Buyer có 24h để phản hồi lại

T+48h+24h: Nếu buyer không phản hồi
          └── Nếu seller accept → auto complete refund
          └── Nếu seller reject → auto close dispute

Trao đổi liên tiếp:
          └── Mỗi bên có 24h để phản hồi
          └── Tối đa 3 lượt trao đổi (6 messages)
          └── Sau 3 lượt → Bắt buộc escalate lên Admin

Lưu ý quan trọng:
- Mỗi bên có thể update thêm bằng chứng (max 3 ảnh/lần)
- Admin có thể can thiệp bất cứ lúc nào
- Dispute có thể escalate bất cứ lúc nào (không đợi 3 lượt)
```

### 5.6 Seller Response Flow (4 Lựa Chọn)

```
═══════════════════════════════════════════════════════════════════
[SELLER RESPONSE FLOW - 4 OPTIONS]
═══════════════════════════════════════════════════════════════════

[SR1] Seller vào Disputes > DSP-001 (or click notification)
        │
        ▼
[SR2] Check deadline:
        │
        ├── NOW > seller_deadline ──► "Đã hết hạn phản hồi"
        │                                (Dispute đã auto-escalate)
        │
        ▼
[SR3] Hiển thị form phản hồi seller với 4 lựa chọn:

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
        ║  QUYẾT ĐỊNH CỦA BẠN: *                                       ║
        ║                                                               ║
        ║  ⦿ CHẤP NHẬN - Đồng ý hoàn tiền như buyer yêu cầu            ║
        ║    └── Tiền sẽ được hoàn cho buyer ngay                     ║
        ║                                                               ║
        ║  ⦿ CHẤP NHẬN MỘT PHẦN - Đề xuất hoàn một phần               ║
        ║    └── Số tiền đề xuất: [___50___] Trust                     ║
        ║    └── Lý do: [____________________________]               ║
        ║                                                               ║
        ║  ⦿ TỪ CHỐI - Không đồng ý khiếu nại                          ║
        ║    └── Lý do từ chối *:                                     ║
        ║        [____________________________________________]       ║
        ║    └── Bằng chứng: [📷 Upload ảnh/video]                     ║
        ║                                                               ║
        ║  ⦿ ĐỔI HÀNG - Gửi items thay thế                             ║
        ║    └── Upload items mới: [📤 Upload file txt/csv]            ║
        ║    └── Số lượng đổi: [___5___] items                        ║
        ║                                                               ║
        ║  ─────────────────────────────────────────────────────────── ║
        ║  PHẢN HỒI THÊM * (tối thiểu 20 ký tự)                       ║
        ║  ┌───────────────────────────────────────────────────────────┐║
        ║  │ Giải thích chi tiết về quyết định của bạn...            │║
        ║  └───────────────────────────────────────────────────────────┘║
        ║                                                               ║
        ║  HÌNH ẢNH BẰNG CHỨNG (tối đa 5 ảnh, < 5MB)                  ║
        ║  ┌─────────────────────────────────────────────────────────┐ ║
        ║  │  📷 [Chọn ảnh]                                          │ ║
        ║  │  ┌────────┐  ┌────────┐  ┌────────┐                    │ ║
        ║  │  │ proof1 │  │ proof2 │  │  + Add │                    │ ║
        ║  │  └────────┘  └────────┘  └────────┘                    │ ║
        ║  └─────────────────────────────────────────────────────────┘ ║
        ║                                                               ║
        ║  💡 Gợi ý bằng chứng:                                         ║
        ║  • Screenshot đăng nhập thành công                            ║
        ║  • Screenshot thông tin tài khoản                             ║
        ║  • Video màn hình (upload link)                               ║
        ║  • Log file, history                                          ║
        ║                                                               ║
        ║  [Gửi phản hồi]  [Hủy]                                        ║
        ╚═══════════════════════════════════════════════════════════════╝
        │
        ▼
[SR4] Validate input:
        │
        ├── Chưa chọn action ──► "Vui lòng chọn quyết định của bạn"
        ├── reason.length < 20 ──► "Lý do phải ít nhất 20 ký tự"
        ├── Partial accept: amount <= 0 ──► "Số tiền phải lớn hơn 0"
        ├── Partial accept: amount > order.total ──► "Số tiền không vượt quá tổng đơn"
        ├── Replacement: items file empty ──► "Vui lòng upload items mới"
        ├── images.count > 5 ──► "Tối đa 5 ảnh"
        ├── any image.size > 5MB ──► "Ảnh không được quá 5MB"
        │
        ▼
[SR5] Upload images & items to storage:
        │
        FOR each image:
          upload_path = "disputes/DSP-{id}/seller_{index}.{ext}"
          upload_to_s3(image, upload_path)
        │
        IF action == replacement:
          upload items file
        │
        ▼
[SR6] Seller submits → BEGIN TRANSACTION
        │
        ▼
[SR7] Xử lý theo action:

        ├── CHẤP NHẬN FULL → [SR8]
        ├── CHẤP NHẬN MỘT PHẦN → [SR12]
        ├── TỪ CHỐI → [SR16]
        └── ĐỔI HÀNG → [SR20]

═══════════════════════════════════════════════════════════════════
[PATH 1] CHẤP NHẬN FULL
═══════════════════════════════════════════════════════════════════

[SR8] Update DisputeCase:

        UPDATE dispute_cases SET
          status = "RESOLVED",
          seller_action = "ACCEPT",
          seller_response = "...",
          seller_evidence_images: [...],
          seller_responded_at = NOW(),
          resolved_at = NOW(),
          resolved_by = "SELLER",
          resolution = "SELLER_ACCEPTED"
        WHERE id = 'DSP-001'
        │
        ▼
[SR9] COMMIT TRANSACTION
        │
        ▼
[SR10] Process Refund:

        -- Platform Wallet - Escrow Amount
        -- Buyer Wallet + Escrow Amount
        -- Update escrow status = REFUNDED
        │
        (Xem Refund Flow chi tiết ở section 6)
        │
        ▼
[SR11] Notify:

        ├── Buyer: "Seller đã chấp nhận hoàn tiền. Số tiền: 100 Trust"
        └── Seller: "Đã hoàn tiền cho buyer. Số tiền: 100 Trust"

        ▼
[END_SR1] Dispute resolved - refunded

═══════════════════════════════════════════════════════════════════
[PATH 2] CHẤP NHẬN MỘT PHẦN
═══════════════════════════════════════════════════════════════════

[SR12] Update DisputeCase:

        UPDATE dispute_cases SET
          status = "BUYER_RESPONDED",
          seller_action = "PARTIAL_ACCEPT",
          seller_offer_amount = 50,
          seller_response = "...",
          seller_evidence_images: [...],
          seller_responded_at = NOW(),
          buyer_deadline = NOW() + 24 HOURS
        WHERE id = 'DSP-001'
        │
        ▼
[SR13] COMMIT TRANSACTION
        │
        ▼
[SR14] Notify Buyer:

        📧 Email + Push:
        "Seller đã phản hồi dispute DSP-001

         Seller đề xuất hoàn một phần: 50 Trust
         Lý do: ...

         ⏰ Bạn có 24h để:
         • Chấp nhận đề xuất → Hoàn 50 Trust
         • Từ chối → Escalate lên Admin
         • Không phản hồi → Tự động escalate"
        │
        ▼
[END_SR2] Waiting for buyer response (24h)

═══════════════════════════════════════════════════════════════════
[PATH 3] TỪ CHỐI
═══════════════════════════════════════════════════════════════════

[SR16] Update DisputeCase:

        UPDATE dispute_cases SET
          status = "BUYER_RESPONDED",
          seller_action = "REJECT",
          seller_response = "...",
          seller_evidence_images: [...],
          seller_responded_at = NOW(),
          buyer_deadline = NOW() + 24 HOURS
        WHERE id = 'DSP-001'
        │
        ▼
[SR17] COMMIT TRANSACTION
        │
        ▼
[SR18] Notify Buyer:

        📧 Email + Push:
        "Seller đã từ chối dispute DSP-001

         Lý do: Sản phẩm hoàn toàn đúng mô tả...

         ⏰ Bạn có 24h để:
         • Chấp nhận từ chối → Release escrow cho seller
         • Escalate lên Admin
         • Không phản hồi → Tự động escalate"
        │
        ▼
[END_SR3] Waiting for buyer response (24h)

═══════════════════════════════════════════════════════════════════
[PATH 4] ĐỔI HÀNG
═══════════════════════════════════════════════════════════════════

[SR20] Update DisputeCase:

        UPDATE dispute_cases SET
          status = "BUYER_RESPONDED",
          seller_action = "REPLACEMENT",
          seller_response = "...",
          seller_evidence_images: [...],
          seller_replacement_items = "items.txt",
          seller_responded_at = NOW(),
          buyer_deadline = NOW() + 24 HOURS
        WHERE id = 'DSP-001'
        │
        ▼
[SR21] Lưu replacement items:

        Process replacement items file
        Validate items format
        Store for buyer to claim
        │
        ▼
[SR22] COMMIT TRANSACTION
        │
        ▼
[SR23] Notify Buyer:

        📧 Email + Push:
        "Seller đã gửi items thay thế

         Số lượng: 5 items
         File: items.txt

         ⏰ Bạn có 24h để:
         • Confirm nhận items → Dispute closed
         • Escalate lên Admin nếu items vẫn lỗi
         • Không phản hồi → Tự động escalate"
        │
        ▼
[END_SR4] Waiting for buyer response (24h)
```

### 5.7 Buyer Response Flow

```
═══════════════════════════════════════════════════════════════════
[BUYER RESPONSE FLOW]
═══════════════════════════════════════════════════════════════════

[BR1] Buyer vào Disputes > DSP-001 (or click notification)
        │
        ▼
[BR2] Check deadline:
        │
        ├── NOW > buyer_deadline ──► "Đã hết hạn phản hồi"
        │                               → Auto-escalate
        │
        ▼
[BR3] Hiển thị form buyer response (tuỳ theo seller action):

        ╔═══════════════════════════════════════════════════════════════╗
        ║  PHẢN HỒI DISPUTE - DSP-001                                ║
        ╠═══════════════════════════════════════════════════════════════╣
        ║                                                               ║
        ║  Đơn hàng: ORD-123                                            ║
        ║  Seller: seller_002                                           ║
        ║  Số tiền: 100 Trust                                           ║
        ║                                                               ║
        ║  ─────────────────────────────────────────────────────────── ║
        ║  📜 LỊCH SỬ TRAO ĐỔI                                       ║
        ║  ─────────────────────────────────────────────────────────── ║
        ║  [15/01 10:00] 🛒 Buyer: Yêu cầu hoàn tiền                   ║
        ║                  "Sản phẩm không đúng mô tả"                 ║
        ║  [16/01 12:00] 🏪 Seller: Chấp nhận một phần                ║
        ║                  "Đề xuất hoàn 50 Trust"                     ║
        ║                                                               ║
        ║  ─────────────────────────────────────────────────────────── ║
        ║  PHẢN HỒI CỦA SELLER:                                      ║
        ║  "Đề xuất hoàn 50 Trust vì 5/10 items còn tốt"               ║
        ║                                                               ║
        ║  📷 BẰNG CHỨNG CỦA SELLER:                                   ║
        ║  ┌────────┐  ┌────────┐                                       ║
        ║  │ proof1 │  │ proof2 │  (click để xem)                      ║
        ║  └────────┘  └────────┘                                       ║
        ║                                                               ║
        ║  ─────────────────────────────────────────────────────────── ║
        ║  ⏰ DEADLINE: 17/01/2025 12:00 (còn 23h)                     ║
        ║  ─────────────────────────────────────────────────────────── ║
        ║                                                               ║
        ║  QUYẾT ĐỊNH CỦA BẠN: *                                       ║
        ║                                                               ║
        ║  ⦿ Chấp nhận đề xuất của seller                             ║
        ║    └── Hoàn 50 Trust, đóng dispute                         ║
        ║                                                               ║
        ║  ⦿ Từ chối - Yêu cầu Admin xử lý                           ║
        ║    └── Escalate lên Admin để review                         ║
        ║                                                               ║
        ║  THÊM THÔNG TIN (tối thiểu 20 ký tự):                        ║
        ║  ┌───────────────────────────────────────────────────────────┐║
        ║  │ Tôi không đồng ý. 5 items seller nói tốt               │║
        ║  │ thực ra cũng bị lỗi. Tôi muốn hoàn đủ 100 Trust.       │║
        ║  └───────────────────────────────────────────────────────────┘║
        ║                                                               ║
        ║  THÊM BẰNG CHỨNG (tối đa 3 ảnh, < 5MB):                      ║
        ║  ┌─────────────────────────────────────────────────────────┐ ║
        ║  │  📷 [Chọn thêm ảnh]                                     │ ║
        ║  │  ┌────────┐  ┌────────┐  ┌────────┐                    │ ║
        ║  │  │ img3   │  │ img4   │  │  + Add │                    │ ║
        ║  │  └────────┘  └────────┘  └────────┘                    │ ║
        ║  └─────────────────────────────────────────────────────────┘ ║
        ║                                                               ║
        ║  [Gửi phản hồi]  [Hủy]                                        ║
        ╚═══════════════════════════════════════════════════════════════╝
        │
        ▼
[BR4] Validate input:
        │
        ├── Chưa chọn action ──► "Vui lòng chọn quyết định"
        ├── reason.length < 20 ──► "Thông tin phải ít nhất 20 ký tự"
        ├── new images.count > 3 ──► "Tối đa 3 ảnh thêm"
        │
        ▼
[BR5] Upload new images (if any)
        │
        ▼
[BR6] Buyer submits → BEGIN TRANSACTION
        │
        ▼
[BR7] Check current exchange count:
        │
        exchange_count = COUNT(all updates)
                       = buyer_updates + seller_updates

        IF exchange_count >= 6:  // 3 lượt mỗi bên
        └──→ Force escalate to admin (max exchanges reached)
        │
        ▼
[BR8] Xử lý theo action:

        ├── Chấp nhận → [BR9] Resolve dispute
        ├── Từ chối → [BR13] Escalate
        └── Force escalate → [BR13]

═══════════════════════════════════════════════════════════════════
[PATH 1] CHẤP NHẬN
═══════════════════════════════════════════════════════════════════

[BR9] Update DisputeCase:

        UPDATE dispute_cases SET
          status = "RESOLVED",
          buyer_action = "ACCEPT_OFFER",
          buyer_updates = buyer_updates || [{
            message: "...",
            images: [...],
            at: NOW()
          }],
          buyer_responded_at = NOW(),
          resolved_at = NOW(),
          resolved_by = "BUYER_ACCEPTED",
          resolution = "PARTIAL_REFUND",
          refund_amount = {seller_offer_amount}
        WHERE id = 'DSP-001'
        │
        ▼
[BR10] Add buyer update to history
        │
        ▼
[BR11] COMMIT TRANSACTION
        │
        ▼
[BR12] Process Partial Refund:

        -- Refund seller_offer_amount to buyer
        -- Release remaining to seller

        → Notify both parties
        → END

═══════════════════════════════════════════════════════════════════
[PATH 2] TỪ CHỐI / ESCALATE
═══════════════════════════════════════════════════════════════════

[BR13] Update DisputeCase:

        UPDATE dispute_cases SET
          status = "ESCALATED",
          buyer_action = "ESCALATE",
          buyer_updates = buyer_updates || [{
            message: "...",
            images: [...],
            at: NOW()
          }],
          buyer_responded_at = NOW(),
          escalated_at = NOW(),
          escalated_by = "BUYER",
          escalate_reason = "Buyer không đồng ý với seller response"
        WHERE id = 'DSP-001'
        │
        ▼
[BR14] Add buyer update to history
        │
        ▼
[BR15] COMMIT TRANSACTION
        │
        ▼
[BR16] Notify:

        ├── Seller: "Buyer đã escalate dispute lên Admin"
        └── Admin: "Dispute DSP-001 cần review"

        ▼
[END_BR] Waiting for admin resolution
```

### 5.8 Auto-Escalate Cron Job

```
═══════════════════════════════════════════════════════════════════
[AUTO-ESCALATE CRON JOB]
═══════════════════════════════════════════════════════════════════

Schedule: Mỗi 30 phút (0,30 * * * *)

Cron job này xử lý 2 trường hợp auto-escalate:

1. Seller không phản hồi trong 48h
2. Buyer không phản hồi trong 24h sau seller response

═══════════════════════════════════════════════════════════════════
[CASE 1] SELLER NO RESPONSE (48h)
═══════════════════════════════════════════════════════════════════

[AE1] Query disputes cần auto-escalate:

        SELECT * FROM dispute_cases
        WHERE status = 'PENDING'           -- Seller CHƯA phản hồi
          AND seller_response IS NULL
          AND NOW() > seller_deadline      -- Quá 2 ngày (48h)
        │
        ├── Không có ──► Continue Case 2
        │
        ▼
[AE2] Loop qua từng dispute:
        │
        ▼
[AE3] Update DisputeCase:

        UPDATE dispute_cases SET
          status = "ESCALATED",
          escalated_at = NOW(),
          escalated_by = "SYSTEM_AUTO",
          escalate_reason = "Seller không phản hồi trong 48 giờ"
        WHERE id = dispute.id
        │
        ▼
[AE4] Notify:

        ├── Buyer:
        "⏰ Seller chưa phản hồi dispute DSP-001
         Đã quá 48 giờ. Dispute đã được chuyển lên Admin."

        ├── Seller:
        "⚠️ Bạn chưa phản hồi dispute DSP-001
         Đã quá 48 giờ. Dispute đã được escalate lên Admin."

        ├── Admin:
        "🚨 Dispute DSP-001 auto-escalated
         Seller không phản hồi trong 48h. Cần admin review."

        ▼
[AE5] Continue next dispute

═══════════════════════════════════════════════════════════════════
[CASE 2] BUYER NO RESPONSE (24h)
═══════════════════════════════════════════════════════════════════

[AE10] Query disputes buyer không phản hồi:

        SELECT * FROM dispute_cases
        WHERE status IN ('SELLER_RESPONDED', 'BUYER_RESPONDED')
          AND NOW() > buyer_deadline      -- Quá 24h
        │
        ├── Không có ──► END
        │
        ▼
[AE11] Loop qua từng dispute:
        │
        ▼
[AE12] Check seller action:

        IF seller_action = 'ACCEPT' OR seller_action = 'PARTIAL_ACCEPT':
        └──→ Auto-resolve in favor of seller

        IF seller_action = 'REJECT' OR seller_action = 'REPLACEMENT':
        └──→ Auto-escalate to admin
        │
        ▼
[AE13] Xử lý theo seller action:

        ├── ACCEPT/PARTIAL_ACCEPT → [AE14] Auto-resolve
        └── REJECT/REPLACEMENT → [AE18] Auto-escalate

═══════════════════════════════════════════════════════════════════
[SUBCASE 2A] AUTO-RESOLVE (SELLER ACCEPTED)
═══════════════════════════════════════════════════════════════════

[AE14] Buyer không phản hồi + Seller chấp nhận
     └──→ Auto-resolve theo seller đề xuất

        UPDATE dispute_cases SET
          status = "RESOLVED",
          resolved_at = NOW(),
          resolved_by = "SYSTEM_AUTO",
          resolution = "BUYER_NO_RESPONSE_SELLER_ACCEPTED"
        WHERE id = dispute.id
        │
        ▼
[AE15] Process resolution:

        IF seller_action = 'ACCEPT':
        └──→ Full refund

        IF seller_action = 'PARTIAL_ACCEPT':
        └──→ Refund seller_offer_amount
        │
        ▼
[AE16] Notify:

        ├── Buyer: "Bạn chưa phản hồi trong 24h. Dispute đã được
                     giải quyết theo đề xuất của seller."

        ├── Seller: "Buyer chưa phản hồi. Dispute đã được giải quyết
                     theo đề xuất của bạn."

        ▼
[AE17] Continue next dispute

═══════════════════════════════════════════════════════════════════
[SUBCASE 2B] AUTO-ESCALATE (SELLER REJECT)
═══════════════════════════════════════════════════════════════════

[AE18] Buyer không phản hồi + Seller từ chối/đổi hàng
     └──→ Auto-escalate để admin quyết định

        UPDATE dispute_cases SET
          status = "ESCALATED",
          escalated_at = NOW(),
          escalated_by = "SYSTEM_AUTO",
          escalate_reason = "Buyer không phản hồi trong 24 giờ. Seller yêu cầu Admin review."
        WHERE id = dispute.id
        │
        ▼
[AE19] Notify:

        ├── Buyer: "Bạn chưa phản hồi trong 24h. Dispute đã
                     được escalate lên Admin để review."

        ├── Seller: "Buyer chưa phản hồi. Dispute đã được
                     escalate lên Admin."

        ├── Admin: "Dispute DSP-001 auto-escalated. Buyer không
                     phản hồi trong 24h. Cần admin review."

        ▼
[AE20] Continue next dispute

═══════════════════════════════════════════════════════════════════

[END] Auto-escalate cron job completed
```

---

## 6. Admin Resolution Flow

### 6.1 Admin Review Dashboard

```
═══════════════════════════════════════════════════════════════════
[ADMIN DISPUTE DASHBOARD V2]
═══════════════════════════════════════════════════════════════════

[A1] Admin vào Disputes > DSP-001
        │
        ▼
[A2] Xem chi tiết dispute với multi-exchange:

        ╔═══════════════════════════════════════════════════════╗
        ║  DISPUTE DETAILS - DSP-001                            ║
        ╠═══════════════════════════════════════════════════════╣
        ║                                                       ║
        ║  📋 THÔNG TIN CHUNG                                   ║
        ║  ─────────────────────────────────────────────────── ║
        ║  Order: ORD-123         Escrow: ESC-001              ║
        ║  Buyer: buyer_001       Seller: seller_002            ║
        ║  Amount: 100 Trust      Created: 15/01/2025 10:00     ║
        ║                                                       ║
        ║  ⚡ TRẠNG THÁI: BUYER_RESPONDED                       ║
        ║     Đang chờ: Buyer phản hồi                         ║
        ║     Deadline: 17/01/2025 14:00 (còn 3h)              ║
        ║                                                       ║
        ║  🔄 LƯỢT TRAO ĐỔI: 2/3                               ║
        ║     Buyer đã gửi: 2 messages                         ║
        ║     Seller đã gửi: 2 messages                        ║
        ║     Còn lại: 1 lượt mỗi bên                          ║
        ║                                                       ║
        ║  ─────────────────────────────────────────────────── ║
        ║  🏪 HÀNH ĐỘNG GẦN ĐÂY CỦA SELLER                     ║
        ║  ─────────────────────────────────────────────────── ║
        ║                                                       ║
        ║  [16/01 12:30] PARTIAL_ACCEPT                         ║
        ║  "Chúng tôi đồng ý hoàn 30% do lỗi giao hàng"         ║
        ║  💰 Offer amount: 30 Trust                           ║
        ║                                                       ║
        ║  4 lựa chọn seller có thể dùng:                       ║
        ║  ✅ ACCEPT      - Chấp nhận full refund               ║
        ║  💵 PARTIAL     - Chấp nhận một phần                  ║
        ║  ❌ REJECT      - Từ chối dispute                     ║
        ║  🔄 REPLACEMENT - Đổi hàng                            ║
        ║                                                       ║
        ║  ─────────────────────────────────────────────────── ║
        ║  📜 TIMELINE TRAO ĐỔI ĐẦY ĐỦ                         ║
        ║  ─────────────────────────────────────────────────── ║
        ║                                                       ║
        ║  【Lượt 1 - Buyer】                                   ║
        ║  ┌─ [15/01 10:00] 🛒 BUYER REQUEST ─────────────┐    ║
        ║  │ Type: REFUND_REQUEST                          │    ║
        ║  │ "Sản phẩm không đúng như mô tả, không thể   │    ║
        ║  │  đăng nhập được. Xin hoàn tiền 100%"         │    ║
        ║  │ 📷 [img_broken.png] [img_error_login.png]    │    ║
        ║  └──────────────────────────────────────────────┘    ║
        ║                                                       ║
        ║  【Lượt 1 - Seller】                                   ║
        ║  ┌─ [16/01 09:00] 🏪 SELLER RESPONSE ────────────┐    ║
        ║  │ Type: REJECT                                   │    ║
        ║  │ "Sản phẩm hoàn toàn đúng mô tả. Tài khoản    │    ║
        ║  │  vẫn hoạt động bình thường. Đã test OK"       │    ║
        ║  │ 📷 [proof_working.png] [video_demo.mp4]       │    ║
        ║  └──────────────────────────────────────────────┘    ║
        ║                                                       ║
        ║  【Lượt 2 - Buyer】                                   ║
        ║  ┌─ [16/01 11:00] 🛒 BUYER UPDATE ───────────────┐    ║
        ║  │ Type: EVIDENCE_UPDATE                         │    ║
        ║  │ "Đã thử lại nhiều lần vẫn không được. Đây là  │    ║
        ║  │  video minh họa lỗi khi đăng nhập"            │    ║
        ║  │ 📷 [video_error_login.mp4]                    │    ║
        ║  └──────────────────────────────────────────────┘    ║
        ║                                                       ║
        ║  【Lượt 2 - Seller】                                   ║
        ║  ┌─ [16/01 12:30] 🏪 SELLER RESPONSE ────────────┐    ║
        ║  │ Type: PARTIAL_ACCEPT ✅                        │    ║
        ║  │ "Xin lỗi vì sự bất tiện. Sau khi kiểm tra,   │    ║
        ║  │  chúng tôi nhận ra có thể lỗi từ batch.      │    ║
        ║  │  Đồng ý hoàn 30% (30 Trust) để khắc phục"     │    ║
        ║  │ 💰 Offer: 30 Trust                            │    ║
        ║  └──────────────────────────────────────────────┘    ║
        ║                                                       ║
        ║  ⏳ Đang chờ Buyer phản hồi (deadline: 17/01 14:00)   ║
        ║                                                       ║
        ╚═══════════════════════════════════════════════════════╝
        │
        ▼
[A3] Admin xem options dựa trên status:

        ╔═══════════════════════════════════════════════════════╗
        ║  HÀNH ĐỘNG ADMIN                                     ║
        ╠═══════════════════════════════════════════════════════╣
        ║                                                       ║
        ║  📊 OPTIONS CHO DISPUTE NÀY:                          ║
        ║                                                       ║
        ║  ┌─────────────────────────────────────────────────┐ ║
        ║  │ 1️⃣  CHỜ ĐỐI THOẠI TIẾP (KHÔNG LÀM GÌ)         │ ║
        ║  │    • Status: BUYER_RESPONDED                    │ ║
        ║  │    • Buyer vẫn còn 1 lượt trao đổi             │ ║
        ║  │    • Deadline: 17/01/2025 14:00                │ ║
        ║  │    → Hệ thống sẽ tự xử lý khi deadline          │ ║
        ║  └─────────────────────────────────────────────────┘ ║
        ║                                                       ║
        ║  ┌─────────────────────────────────────────────────┐ ║
        ║  │ 2️⃣  GIA HẠN THỜI GIAN (1-7 ngày)                │ ║
        ║  │    • Cho 2 bên thêm thời gian giải quyết        │ ║
        ║  │    • Cần nhập lý do                             │ ║
        ║  └─────────────────────────────────────────────────┘ ║
        ║                                                       ║
        ║  ┌─────────────────────────────────────────────────┐ ║
        ║  │ 3️⃣  ESCALATE LÊN ADMIN REVIEW NGAY             │ ║
        ║  │    • Admin sẽ quyết định ngay                   │ ║
        ║  │    • Bỏ qua các trao đổi còn lại               │ ║
        ║  └─────────────────────────────────────────────────┘ ║
        ║                                                       ║
        ║  ┌─────────────────────────────────────────────────┐ ║
        ║  │ 4️⃣  FORCE RELEASE CHO SELLER                   │ ║
        ║  │    • Từ chối dispute, release escrow           │ ║
        ║  │    • Seller nhận 95%, Platform nhận 5%         │ ║
        ║  │    ⚠️  Cần lý do bắt buộc                        │ ║
        ║  └─────────────────────────────────────────────────┘ ║
        ║                                                       ║
        ║  ┌─────────────────────────────────────────────────┐ ║
        ║  │ 5️⃣  FORCE HOÀN TIỀN CHO BUYER                  │ ║
        ║  │    • Chấp nhận dispute, refund 100%            │ ║
        ║  │    • Buyer nhận lại 100 Trust                  │ ║
        ║  │    • Escrow được giải phóng                    │ ║
        ║  │    ⚠️  Cần lý do bắt buộc                        │ ║
        ║  └─────────────────────────────────────────────────┘ ║
        ║                                                       ║
        ║  ┌─────────────────────────────────────────────────┐ ║
        ║  │ 6️⃣  HOÀN MỘT PHẦN (PARTIAL REFUND)             │ ║
        ║  │    • Hoàn X% cho buyer, Y% cho seller          │ ║
        ║  │    • Nhập %: Buyer [___] Seller [___]          │ ║
        ║  │    ⚠️  Tổng phải = 100%                         │ ║
        ║  └─────────────────────────────────────────────────┘ ║
        ║                                                       ║
        ║  Ghi chú admin *:                                     ║
        ║  ┌───────────────────────────────────────────────────┐║
        ║  │ Seller đã đề nghị hoàn 30%. Buyer có thể chấp   │ ║
        ║  │ nhận hoặc tiếp tục trao đổi.                   │ ║
        ║  └───────────────────────────────────────────────────┘║
        ║                                                       ║
        ║  [Xác nhận]                                           ║
        ╚═══════════════════════════════════════════════════════╝
        │
        ▼
[A4] Nếu Admin chọn "Escalate ngay":

        ╔═══════════════════════════════════════════════════════╗
        ║  ADMIN RESOLUTION - QUYẾT ĐỊNH DISPUTE               ║
        ╠═══════════════════════════════════════════════════════╣
        ║                                                       ║
        ║  Review timeline trao đổi (2/3 lượt):                 ║
        ║  ─────────────────────────────────────────────────── ║
        ║                                                       ║
        ║  📊 SUMMARY:                                          ║
        ║  • Buyer yêu cầu: 100% refund                        ║
        ║  • Seller từ chối → Đề nghị: 30% refund              ║
        ║  • Buyer chưa phản hồi offer                         ║
        ║                                                       ║
        ║  📋 BY CHỨNG CẢ 2 BÊN:                               ║
        ║  Buyer: 3 images, 2 videos                           ║
        ║  Seller: 2 images, 1 video                           ║
        ║                                                       ║
        ║  ─────────────────────────────────────────────────── ║
        ║  QUYẾT ĐỊNH CỦA ADMIN                                ║
        ║  ─────────────────────────────────────────────────── ║
        ║                                                       ║
        ║  ● Chấp nhận đầy đủ (Refund 100%)                     ║
        ║  ○ Chấp nhận một phần (Partial refund)               ║
        ║  ○ Từ chối, release cho seller                       ║
        ║                                                       ║
        ║  Nếu Partial, nhập phần trăm:                         ║
        ║  Buyer nhận: [___60___]% (60 Trust)                  ║
        ║  Seller nhận: [___40___]% (40 Trust)                 ║
        ║                                                       ║
        ║  Lý do quyết định *:                                  ║
        ║  ┌───────────────────────────────────────────────────┐║
        ║  │ Seller đã đề nghị 30%. Xét thấy by chứng của    │ ║
        ║  │ buyer thuyết phục hơn. Quyết định refund 60%   │ ║
        ║  │ để chia sẻ rủi ro cho 2 bên.                   │ ║
        ║  └───────────────────────────────────────────────────┘║
        ║                                                       ║
        ║  [Xác nhận quyết định]  [Hủy]                        ║
        ╚═══════════════════════════════════════════════════════╝
```

**Dashboard hiển thị đầy đủ:**
- 4 seller response types (ACCEPT, PARTIAL_ACCEPT, REJECT, REPLACEMENT)
- Exchange counter (Lượt 2/3)
- Complete timeline với tất cả buyer_updates và seller_updates
- Seller action type prominent (PARTIAL_ACCEPT ✅)
- Deadlines và next action clearly displayed
- 6 admin options tùy theo dispute status

### 6.2 Admin Extend Deadline Flow

```
─────────────────────────────────────────────────────────────────
[ADMIN EXTEND DEADLINE PATH]
─────────────────────────────────────────────────────────────────

[EX1] Admin chọn "Gia hạn thời gian":

        ╔═══════════════════════════════════════════════════════╗
        ║  GIA HẠN THỜI GIAN DISPUTE                              ║
        ╠═══════════════════════════════════════════════════════╣
        ║                                                       ║
        ║  Dispute: DSP-001                                     ║
        ║  Deadline hiện tại: 17/01/2025 10:00                  ║
        ║                                                       ║
        ║  Số ngày gia hạn *:                                  ║
        ║  ┌─────────┐                                          ║
        ║  │    3    │ ngày (tối đa 7 ngày)                     ║
        ║  └─────────┘                                          ║
        ║                                                       ║
        ║  Deadline mới: 20/01/2025 10:00                       ║
        ║                                                       ║
        ║  Lý do gia hạn *:                                    ║
        ║  ┌───────────────────────────────────────────────────┐║
        ║  │ Cần thêm thời gian để xác minh thông tin sản phẩm. ║║
        ║  │   Yêu cầu seller cung cấp thêm bằng chứng video.   ║║
        ║  └───────────────────────────────────────────────────┘║
        ║                                                       ║
        ║  [Xác nhận gia hạn]  [Hủy]                            ║
        ╚═══════════════════════════════════════════════════════╝
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

### 6.3 Refund Flow - Full & Partial với Fee Calculation

```
═══════════════════════════════════════════════════════════════════
[REFUND FLOW - TRẢ TIỀN CHO BUYER]
═══════════════════════════════════════════════════════════════════

Flow này xử lý 3 trường hợp refund:
1. FULL REFUND - Hoàn 100% cho buyer
2. PARTIAL REFUND - Hoàn X% cho buyer, (100-X)% cho seller
3. ADMIN DECISION - Admin quyết định phần trăm

═══════════════════════════════════════════════════════════════════
[CASE 1] FULL REFUND (100%)
═══════════════════════════════════════════════════════════════════

[R1] Dispute được resolve với refund_amount = 100 Trust

        Escrow info:
        - order_id: ORD-123
        - escrow_id: ESC-001
        - escrow_amount: 100 Trust (original purchase price)
        - buyer_id: buyer_001
        - seller_id: seller_002
        │
        ▼
[R2] BEGIN TRANSACTION
        │
        ▼
[R3] Calculate amounts:

        escrow_amount = 100 Trust

        Full refund:
        refund_to_buyer = 100 Trust
        to_seller = 0 Trust
        platform_fee = 0 Trust  (Không thu fee khi refund)

        ⚠️ LƯU Ý: Khi refund 100%, Platform KHÔNG thu fee
        vì không có giao dịch thành công.
        │
        ▼
[R4] Tạo Transaction Platform (ESCROW_REFUND):

        INSERT INTO transactions {
          wallet_id: "WLT-PLATFORM",
          type: "ESCROW_REFUND",
          direction: "DEBIT",
          amount: -100,
          balance_type: "AVAILABLE",
          reference_id: "ESC-001",
          order_id: "ORD-123",
          dispute_id: "DSP-001",
          description: "Full refund to buyer (dispute resolved)",
          balance_before: 5000000,
          balance_after: 4999900
        }
        │
        ▼
[R5] Update Platform Wallet:

        UPDATE wallets SET
          available_trust = available_trust - 100,
          total_trust = total_trust - 100,
          escrow_holding = escrow_holding - 100  -- calculated field
        WHERE user_id = 'PLATFORM'
        │
        ▼
[R6] Tạo Transaction Buyer (REFUND_CREDITED):

        INSERT INTO transactions {
          wallet_id: "WLT-BUYER-001",
          type: "REFUND_CREDITED",
          direction: "CREDIT",
          amount: +100,
          balance_type: "AVAILABLE",
          reference_id: "ESC-001",
          order_id: "ORD-123",
          dispute_id: "DSP-001",
          description: "Full refund from dispute DSP-001",
          balance_before: 500,
          balance_after: 600
        }
        │
        ▼
[R7] Update Buyer Wallet:

        UPDATE wallets SET
          available_trust = available_trust + 100,
          total_trust = total_trust + 100
        WHERE user_id = 'buyer_001'
        │
        ▼
[R8] Update EscrowHold:

        UPDATE escrow_holds SET
          status = "REFUNDED",
          released_at = NOW(),
          refund_amount = 100,
          released_to = "buyer_001",
          refund_reason = "Dispute resolved - full refund"
        WHERE id = 'ESC-001'
        │
        ▼
[R9] Update DisputeCase:

        UPDATE dispute_cases SET
          status = "REFUNDED",
          refund_amount = 100,
          refund_processed_at = NOW()
        WHERE id = 'DSP-001'
        │
        ▼
[R10] Update Order:

        UPDATE orders SET
          order_status = "REFUNDED",
          refunded_at = NOW(),
          refund_amount = 100
        WHERE id = 'ORD-123'
        │
        ▼
[R11] Validate Invariants:

        ├── Platform invariant: available_trust == total_escrows + withdrawable_commission
        ├── Buyer invariant: total_trust == available + withdrawal_locked + dispute_locked
        └── Escrow amount == refund amount
        │
        ├── Any failed ──► ROLLBACK, alert admin
        ├── All passed ──► COMMIT
        │
        ▼
[R12] Notify:

        ├── Buyer:
        "✅ Đã hoàn tiền cho dispute DSP-001

         Số tiền: 100 Trust
         Đơn hàng: ORD-123

         Số dư ví của bạn: 600 Trust"

        ├── Seller:
        "⚠️ Dispute DSP-001 đã được resolve

         Quyết định: Full refund cho buyer
         Bạn nhận: 0 Trust

         Lý do: [admin reason hoặc seller accepted]"

        └── Admin (nếu auto-resolve):
        "Dispute DSP-001 auto-resolved: Full refund"
        │
        ▼
[END_R1] Full refund completed

═══════════════════════════════════════════════════════════════════
[CASE 2] PARTIAL REFUND (X% cho buyer, Y% cho seller)
═══════════════════════════════════════════════════════════════════

[PR1] Dispute được resolve với partial refund:

        Ví dụ: Buyer chấp nhận offer 30 Trust (30%)
        Hoặc: Admin quyết định refund 60 Trust (60%)

        Escrow info:
        - escrow_amount: 100 Trust
        - refund_amount: 60 Trust (60%)
        - seller_receive: 40 Trust (40%)
        │
        ▼
[PR2] Calculate commission:

        commission_rate = 5% (default)
        seller_portion = 40 Trust
        platform_commission = seller_portion * commission_rate
                           = 40 * 0.05
                           = 2 Trust

        seller_net = seller_portion - platform_commission
                   = 40 - 2
                   = 38 Trust

        💰 SUMMARY:
        ┌─────────────────────────────────────────┐
        │ Buyer receives:  60 Trust              │
        │ Seller receives: 38 Trust (after fee)  │
        │ Platform fee:     2 Trust              │
        │ Total:           100 Trust ✅          │
        └─────────────────────────────────────────┘
        │
        ▼
[PR3] BEGIN TRANSACTION
        │
        ▼
[PR4] Tạo Transaction Platform (ESCROW_PARTIAL_REFUND):

        INSERT INTO transactions {
          wallet_id: "WLT-PLATFORM",
          type: "ESCROW_PARTIAL_REFUND",
          direction: "DEBIT",
          amount: -100,  -- Total escrow
          balance_type: "AVAILABLE",
          reference_id: "ESC-001",
          order_id: "ORD-123",
          dispute_id: "DSP-001",
          description: "Partial refund: 60 to buyer, 38 to seller, 2 fee",
          breakdown: {
            refund_to_buyer: 60,
            to_seller: 38,
            platform_fee: 2
          },
          balance_before: 5000000,
          balance_after: 4999900
        }
        │
        ▼
[PR5] Update Platform Wallet:

        UPDATE wallets SET
          available_trust = available_trust - 100,
          total_trust = total_trust - 100,
          withdrawable_commission = withdrawable_commission + 2
        WHERE user_id = 'PLATFORM'
        │
        ▼
[PR6] Tạo Transaction Buyer (REFUND_CREDITED):

        INSERT INTO transactions {
          wallet_id: "WLT-BUYER-001",
          type: "REFUND_CREDITED",
          direction: "CREDIT",
          amount: +60,
          balance_type: "AVAILABLE",
          reference_id: "ESC-001",
          order_id: "ORD-123",
          dispute_id: "DSP-001",
          description: "Partial refund: 60 Trust (60%) from dispute",
          balance_before: 500,
          balance_after: 560
        }
        │
        ▼
[PR7] Update Buyer Wallet:

        UPDATE wallets SET
          available_trust = available_trust + 60,
          total_trust = total_trust + 60
        WHERE user_id = 'buyer_001'
        │
        ▼
[PR8] Tạo Transaction Seller (ESCROW_RELEASE_SELLER):

        INSERT INTO transactions {
          wallet_id: "WLT-SELLER-002",
          type: "ESCROW_RELEASE_SELLER",
          direction: "CREDIT",
          amount: +38,  -- After commission
          balance_type: "AVAILABLE",
          reference_id: "ESC-001",
          order_id: "ORD-123",
          dispute_id: "DSP-001",
          description: "Escrow release: 38 Trust (after 5% fee)",
          balance_before: 1000,
          balance_after: 1038,
          breakdown: {
            gross: 40,
            commission: -2,
            net: 38
          }
        }
        │
        ▼
[PR9] Update Seller Wallet:

        UPDATE wallets SET
          available_trust = available_trust + 38,
          total_trust = total_trust + 38,
          lifetime_earned = lifetime_earned + 40  -- Gross
        WHERE user_id = 'seller_002'
        │
        ▼
[PR10] Tạo Transaction Platform (COMMISSION):

        INSERT INTO transactions {
          wallet_id: "WLT-PLATFORM",
          type: "COMMISSION_RELEASED",
          direction: "CREDIT",
          amount: +2,
          balance_type: "WITHDRAWABLE_COMMISSION",
          reference_id: "ESC-001",
          order_id: "ORD-123",
          description: "Commission from partial refund (5% of 40)",
          seller_id: "seller_002",
          breakdown: {
            seller_portion: 40,
            commission_rate: 0.05,
            commission_amount: 2
          }
        }
        │
        ▼
[PR11] Update EscrowHold:

        UPDATE escrow_holds SET
          status = "PARTIAL_REFUND",
          released_at = NOW(),
          refund_amount = 60,
          seller_amount = 38,
          commission_amount = 2,
          released_to_buyer = "buyer_001",
          released_to_seller = "seller_002",
          refund_reason = "Partial refund from dispute"
        WHERE id = 'ESC-001'
        │
        ▼
[PR12] Update DisputeCase:

        UPDATE dispute_cases SET
          status = "PARTIAL_REFUND",
          refund_amount = 60,
          seller_amount = 38,
          commission_amount = 2,
          refund_processed_at = NOW()
        WHERE id = 'DSP-001'
        │
        ▼
[PR13] Update Order:

        UPDATE orders SET
          order_status = "PARTIALLY_REFUNDED",
          refunded_at = NOW(),
          refund_amount = 60,
          seller_amount = 38
        WHERE id = 'ORD-123'
        │
        ▼
[PR14] Validate Invariants:

        CHECK 1: Total money out = 100 (escrow amount)
                60 (buyer) + 38 (seller) + 2 (fee) = 100 ✅

        CHECK 2: Platform.available decreased by 100
                Platform.withdrawable_commission increased by 2 ✅

        CHECK 3: Buyer + Seller balances updated correctly ✅

        ├── Any failed ──► ROLLBACK, alert admin
        ├── All passed ──► COMMIT
        │
        ▼
[PR15] Notify:

        ├── Buyer:
        "✅ Đã hoàn tiền một phần

         Dispute: DSP-001
         Đơn hàng: ORD-123

         Bạn nhận: 60 Trust
         Số dư ví: 560 Trust"

        ├── Seller:
        "⚠️ Dispute DSP-001 đã được resolve

         Buyer refund: 60 Trust (60%)
         Bạn nhận: 38 Trust (40% sau khi trừ fee 5%)
         Commission: 2 Trust

         Số dư ví: 1,038 Trust"

        └── Admin (nếu applicable):
        "Partial refund processed"
        │
        ▼
[END_PR2] Partial refund completed

═══════════════════════════════════════════════════════════════════
[CASE 3] ADMIN DECISION - CUSTOM PERCENTAGE
═══════════════════════════════════════════════════════════════════

[AR1] Admin quyết định custom percentage:

        ╔═══════════════════════════════════════════╗
        ║  ADMIN RESOLUTION                         ║
        ╠═══════════════════════════════════════════╣
        ║                                           ║
        ║  Escrow amount: 100 Trust                 ║
        ║                                           ║
        ║  Buyer nhận: [___75___]%  (75 Trust)      ║
        ║  Seller nhận: [___25___]% (25 Trust)      ║
        ║                                           ║
        ║  ✅ Validate: Tổng = 100%                ║
        ║                                           ║
        ║  Commission (5% của seller portion):      ║
        ║  25 * 0.05 = 1.25 Trust                   ║
        ║  Seller nhận: 25 - 1.25 = 23.75 Trust     ║
        ║                                           ║
        ║  💰 FINAL:                                ║
        ║  Buyer: 75.00 Trust                       ║
        ║  Seller: 23.75 Trust                      ║
        ║  Platform: 1.25 Trust                     ║
        ║  Total: 100.00 Trust ✅                   ║
        ║                                           ║
        ║  Lý do: [_________________________]       ║
        ║                                           ║
        ║  [Xác nhận]  [Hủy]                        ║
        ╚═══════════════════════════════════════════╝
        │
        ▼
[AR2] Admin confirms → Follow [PR3] to [PR15] above
     with custom amounts
        │
        ▼
[END_AR3] Admin decision refund completed

═══════════════════════════════════════════════════════════════════
[FEE CALCULATION SUMMARY]
═══════════════════════════════════════════════════════════════════

┌─────────────────────────────────────────────────────────────────┐
│              FEE CALCULATION REFERENCE TABLE                    │
├─────────────────────────────────────────────────────────────────┤
│ Escrow │ Buyer  │ Seller │ Commission │ Seller │ Platform │    │
│ Amount │ Refund │ Portion│ (5%)       │ Net    │ Fee      │    │
├────────┼────────┼────────┼────────────┼────────┼──────────┤    │
│ 100    │ 100%   │ 0      │ 0          │ 0      │ 0        │    │
│        │ (100)  │ (0)    │            │        │          │    │
├────────┼────────┼────────┼────────────┼────────┼──────────┤    │
│ 100    │ 80%    │ 20%    │ 20*0.05=1  │ 19     │ 1        │    │
│        │ (80)   │ (20)   │            │        │          │    │
├────────┼────────┼────────┼────────────┼────────┼──────────┤    │
│ 100    │ 60%    │ 40%    │ 40*0.05=2  │ 38     │ 2        │    │
│        │ (60)   │ (40)   │            │        │          │    │
├────────┼────────┼────────┼────────────┼────────┼──────────┤    │
│ 100    │ 50%    │ 50%    │ 50*0.05=2.5│ 47.5   │ 2.5      │    │
│        │ (50)   │ (50)   │            │        │          │    │
├────────┼────────┼────────┼────────────┼────────┼──────────┤    │
│ 100    │ 30%    │ 70%    │ 70*0.05=3.5│ 66.5   │ 3.5      │    │
│        │ (30)   │ (70)   │            │        │          │    │
├────────┼────────┼────────┼────────────┼────────┼──────────┤    │
│ 100    │ 20%    │ 80%    │ 80*0.05=4  │ 76     │ 4        │    │
│        │ (20)   │ (80)   │            │        │          │    │
├────────┼────────┼────────┼────────────┼────────┼──────────┤    │
│ 100    │ 0%     │ 100%   │ 100*0.05=5 │ 95     │ 5        │    │
│        │ (0)    │ (100)  │            │        │          │    │
└────────┴────────┴────────┴────────────┴────────┴──────────┘    │

Formula:
─────────
buyer_refund       = escrow_amount * buyer_percent
seller_gross       = escrow_amount * seller_percent
platform_commission = seller_gross * commission_rate (default 5%)
seller_net         = seller_gross - platform_commission

Invariant: buyer_refund + seller_net + platform_commission == escrow_amount
```

**Key Points:**

1. **Full Refund (100%)**: Platform KHÔNG thu fee vì giao dịch không thành công
2. **Partial Refund**: Platform thu 5% commission trên phần seller nhận
3. **Seller Net = Seller Gross - Commission**
4. **Commission được cộng vào `withdrawable_commission` của Platform**
5. **Tất cả refunds phải pass invariant checks trước khi commit**

---

## 7. Business Rules

| # | Rule |
|---|------|
| **BR_ESCROW_1** | Escrow period: 3 ngày (72 giờ) |
| **BR_ESCROW_2** | Auto-release cron chạy mỗi giờ |
| **BR_ESCROW_3** | Buyer có thể confirm sớm để release ngay |
| **BR_ESCROW_4** | Platform.available PHẢI >= total escrow amounts |
| **BR_ESCROW_5** | Commission: 5% (default), trừ khi seller withdraw |
| **BR_ESCROW_6** | Admin debt: Trừ tự động khi seller nhận escrow |
| **BR_DISPUTE_1** | Buyer phải cung cấp lý do ít nhất 20 ký tự |
| **BR_DISPUTE_2** | Buyer có thể upload tối đa 5 ảnh làm bằng chứng (< 5MB) |
| **BR_DISPUTE_3** | Seller có **48h (2 ngày)** để phản hồi từ khi dispute tạo |
| **BR_DISPUTE_4** | Nếu seller không phản hồi trong 48h → **Auto-escalate lên Admin** (không phải auto-refund) |
| **BR_DISPUTE_5** | Admin có thể gia hạn thêm 1-7 ngày nếu cần xác minh |
| **BR_DISPUTE_6** | Cả buyer và seller có thể cập nhật thêm thông tin (multi-exchange) |
| **BR_DISPUTE_7** | Mỗi update có thể đính kèm tối đa 3 ảnh (initial: 5 ảnh) |
| **BR_DISPUTE_8** | Ảnh bằng chứng lưu trong S3/MinIO với path `disputes/{dispute_id}/` |
| **BR_DISPUTE_9** | Seller có 4 lựa chọn: Accept, Partial Accept, Reject, Replacement |
| **BR_DISPUTE_10** | Seller có 48h (2 ngày) để phản hồi, nếu không → Auto-escalate (không phải auto-refund) |
| **BR_DISPUTE_11** | Buyer có 24h để phản hồi lại sau seller response |
| **BR_DISPUTE_12** | Buyer không phản hồi trong 24h + Seller accept → Auto-resolve theo seller |
| **BR_DISPUTE_13** | Buyer không phản hồi trong 24h + Seller reject → Auto-escalate |
| **BR_DISPUTE_14** | Tối đa 3 lượt trao đổi (6 messages) trước khi bắt buộc escalate |
| **BR_DISPUTE_15** | Đã 3 lượt → Force escalate lên Admin (không thể trao đổi thêm) |
| **BR_DISPUTE_16** | Auto-escalate cron job chạy mỗi 30 phút (xử lý cả 2 trường hợp) |
| **BR_DISPUTE_17** | Buyer/Seller có thể escalate bất cứ lúc nào (không đợi 3 lượt) |
| **BR_DISPUTE_18** | Admin có thể can thiệp bất cứ lúc nào |
| **BR_DISPUTE_19** | Seller KHỞI BỎ flow cancel order - không áp dụng digital goods |
| **BR_REFUND_1** | Full refund (100%) - Platform KHÔNG thu fee vì giao dịch không thành công |
| **BR_REFUND_2** | Partial refund - Platform thu 5% commission trên phần seller nhận |
| **BR_REFUND_3** | Seller Net = Seller Gross - Platform Commission |
| **BR_REFUND_4** | Commission được cộng vào `withdrawable_commission` của Platform Wallet |
| **BR_REFUND_5** | Invariant: buyer_refund + seller_net + platform_commission == escrow_amount |
| **BR_REFUND_6** | Mọi refund phải validate invariants trước khi commit transaction |
| **BR_REFUND_7** | Refund transaction type: `ESCROW_REFUND` (full) hoặc `ESCROW_PARTIAL_REFUND` (partial) |
| **BR_REFUND_8** | Buyer credit transaction type: `REFUND_CREDITED` |
| **BR_REFUND_9** | Seller nhận transaction type: `ESCROW_RELEASE_SELLER` với breakdown {gross, commission, net} |
| **BR_REFUND_10** | Platform commission transaction type: `COMMISSION_RELEASED` |

---

## Related Documents

- [Wallet Overview](wallet-overview.md) - Tổng quan hệ thống
- [Purchase Flows](purchase.md) - Mua hàng tạo escrow
- [Admin Operations](adjustment.md) - Admin operations
- [Withdrawal Flows](withdrawal.md) - Rút tiền với commission
