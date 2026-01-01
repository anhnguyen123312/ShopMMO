# Chức năng Khiếu nại và Tranh chấp (Disputes)

## Tổng quan

Hệ thống dispute cho phép Buyer khiếu nại đơn hàng có vấn đề trong vòng 3 ngày kể từ khi mua. Quy trình giải quyết theo 3 cấp: Vendor → Admin → Final Decision, đảm bảo công bằng cho cả hai bên.

---

## 1. Điều kiện tạo Dispute

### 1.1 Khi nào có thể khiếu nại

```
┌─────────────────────────────────────────────────────────────────┐
│                ĐIỀU KIỆN TẠO DISPUTE                            │
└─────────────────────────────────────────────────────────────────┘

Buyer có thể tạo dispute khi TẤT CẢ điều kiện sau thỏa mãn:

1. Order Status:
   └── status = 'delivered' (đã giao)

2. Thời hạn:
   └── Trong vòng 3 ngày (72 giờ) kể từ delivered_at
   └── dispute_deadline chưa qua

3. Chưa có dispute:
   └── Chưa tạo dispute cho order này
   └── Hoặc dispute trước đã bị rejected và còn trong thời hạn

4. Order chưa completed:
   └── status != 'completed'
```

### 1.2 Các loại lý do khiếu nại

| Reason Code | Mô tả | Mức độ |
|-------------|-------|--------|
| wrong_item | Sản phẩm sai/không đúng mô tả | High |
| not_working | Sản phẩm không hoạt động (die) | High |
| duplicate | Sản phẩm đã từng mua/bị trùng | High |
| missing_items | Thiếu số lượng so với đặt | Medium |
| quality_issue | Chất lượng không như cam kết | Medium |
| partial_working | Một phần sản phẩm không hoạt động | Medium |
| other | Lý do khác | Low |

---

## 2. Các trạng thái Dispute

```
┌─────────────────────────────────────────────────────────────────┐
│                  TRẠNG THÁI DISPUTE                             │
└─────────────────────────────────────────────────────────────────┘

                    ┌─────────────┐
                    │   PENDING   │ ◄── Buyer vừa tạo, chờ vendor
                    └──────┬──────┘
                           │
                           ▼
                    ┌─────────────┐
                    │ VENDOR_RESP │ ◄── Vendor đã phản hồi
                    └──────┬──────┘
                           │
              ┌────────────┼────────────┐
              │            │            │
              ▼            ▼            ▼
       ┌──────────┐ ┌──────────┐ ┌──────────┐
       │ RESOLVED │ │ ESCALATED│ │ REJECTED │
       └──────────┘ └────┬─────┘ └──────────┘
       Vendor chấp        │      Buyer không đồng ý
       nhận refund        │      có thể escalate
                          ▼
                   ┌─────────────┐
                   │ADMIN_REVIEW │ ◄── Admin đang xem xét
                   └──────┬──────┘
                          │
              ┌───────────┼───────────┐
              │           │           │
              ▼           ▼           ▼
       ┌──────────┐┌──────────┐┌──────────┐
       │REFUNDED  ││ PARTIAL  ││ CLOSED   │
       └──────────┘│ REFUND   │└──────────┘
       Full refund └──────────┘ Reject dispute
```

### Chi tiết trạng thái

| Status | Mô tả | Chờ action từ |
|--------|-------|---------------|
| pending | Mới tạo | Vendor |
| vendor_responded | Vendor đã phản hồi | Buyer |
| escalated | Buyer escalate lên Admin | Admin |
| admin_review | Admin đang xem xét | Admin |
| resolved | Vendor chấp nhận refund | - |
| refunded | Đã hoàn tiền đầy đủ | - |
| partial_refund | Hoàn một phần | - |
| rejected | Dispute bị từ chối | - |
| closed | Đóng không xử lý | - |

---

## 3. Flow tạo Dispute

### 3.1 Buyer tạo dispute

```
┌─────────────────────────────────────────────────────────────────┐
│                   FLOW TẠO DISPUTE                              │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Buyer vào chi tiết đơn hàng đã giao
         │
         ▼
[Bước 2] Kiểm tra điều kiện dispute:
         │
         ├── Hết hạn (>3 ngày) ──► "Đã hết thời hạn khiếu nại"
         ├── Đã có dispute pending ──► "Đang có khiếu nại chờ xử lý"
         ├── Order completed ──► Không hiển thị nút Dispute
         │
         ▼
[Bước 3] Click "Khiếu nại đơn hàng"
         │
         ▼
[Bước 4] Hiển thị form khiếu nại:

         ╔═══════════════════════════════════════════════════════╗
         ║  KHIẾU NẠI ĐƠN HÀNG #ORD-20240115-12345              ║
         ╠═══════════════════════════════════════════════════════╣
         ║                                                       ║
         ║  Sản phẩm: Gmail US Aged x 10                        ║
         ║  Tổng tiền: 125,000đ                                  ║
         ║                                                       ║
         ║  ─────────────────────────────────────────────────── ║
         ║  LÝ DO KHIẾU NẠI *                                   ║
         ║  [▼ Chọn lý do                                    ]   ║
         ║     • Sản phẩm không hoạt động (die)                 ║
         ║     • Sản phẩm sai/không đúng mô tả                  ║
         ║     • Sản phẩm bị trùng                              ║
         ║     • Thiếu số lượng                                 ║
         ║     • Chất lượng không như cam kết                   ║
         ║     • Khác                                           ║
         ║                                                       ║
         ║  SỐ LƯỢNG BỊ LỖI *                                   ║
         ║  [___5___] / 10 items                                ║
         ║                                                       ║
         ║  CHI TIẾT ITEMS BỊ LỖI *                             ║
         ║  ┌───────────────────────────────────────────────────┐║
         ║  │ Vui lòng liệt kê các items bị lỗi:               │║
         ║  │ - email1@gmail.com: wrong password               │║
         ║  │ - email2@gmail.com: account disabled             │║
         ║  │ ...                                               │║
         ║  └───────────────────────────────────────────────────┘║
         ║                                                       ║
         ║  MÔ TẢ CHI TIẾT *                                    ║
         ║  ┌───────────────────────────────────────────────────┐║
         ║  │ Mô tả cụ thể vấn đề bạn gặp phải...              │║
         ║  │                                                   │║
         ║  └───────────────────────────────────────────────────┘║
         ║                                                       ║
         ║  BẰNG CHỨNG                                          ║
         ║  [📷 Upload ảnh/video] (tối đa 5 files, 10MB/file)   ║
         ║                                                       ║
         ║  YÊU CẦU XỬ LÝ *                                     ║
         ║  ○ Hoàn tiền toàn bộ (125,000đ)                      ║
         ║  ○ Hoàn tiền items lỗi (62,500đ cho 5 items)         ║
         ║  ○ Đổi items mới                                      ║
         ║                                                       ║
         ║  [Hủy]  [Gửi khiếu nại]                               ║
         ╚═══════════════════════════════════════════════════════╝
         │
         ▼
[Bước 5] Buyer điền đầy đủ và submit
         │
         ▼
[Bước 6] Validate:
         - Lý do bắt buộc
         - Số lượng lỗi <= số lượng mua
         - Mô tả tối thiểu 50 ký tự
         │
         ▼
[Bước 7] Upload và validate files (nếu có)
         │
         ▼
[Bước 8] Tạo Dispute record:
         - status: pending
         - buyer_id, order_id, shop_id
         - reason, affected_quantity, description
         - requested_action, requested_amount
         │
         ▼
[Bước 9] Cập nhật Order:
         - status = 'disputed'
         │
         ▼
[Bước 10] Gửi notifications:
          - Email vendor: "Có khiếu nại mới cần xử lý"
          - Push notification
          │
          ▼
[Bước 11] Hiển thị xác nhận:
          "Khiếu nại đã được gửi. Vendor có 48 giờ để phản hồi."
```

---

## 4. Vendor phản hồi Dispute

### 4.1 Flow vendor response

```
┌─────────────────────────────────────────────────────────────────┐
│                FLOW VENDOR PHẢN HỒI DISPUTE                     │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Vendor nhận notification về dispute mới
         │
         ▼
[Bước 2] Vào Disputes > Chi tiết dispute
         │
         ▼
[Bước 3] Xem thông tin dispute:
         - Lý do khiếu nại
         - Items bị report lỗi
         - Bằng chứng từ buyer
         - Yêu cầu của buyer
         │
         ▼
[Bước 4] Vendor chọn action:

         ╔═══════════════════════════════════════════════════════╗
         ║  PHẢN HỒI KHIẾU NẠI                                   ║
         ╠═══════════════════════════════════════════════════════╣
         ║                                                       ║
         ║  QUYẾT ĐỊNH CỦA BẠN:                                  ║
         ║                                                       ║
         ║  ○ CHẤP NHẬN - Đồng ý hoàn tiền như buyer yêu cầu    ║
         ║    └── Tiền sẽ được hoàn cho buyer ngay              ║
         ║                                                       ║
         ║  ○ CHẤP NHẬN MỘT PHẦN - Đề xuất hoàn một phần       ║
         ║    └── Số tiền đề xuất: [________] VNĐ               ║
         ║    └── Lý do: [_________________________]            ║
         ║                                                       ║
         ║  ○ TỪ CHỐI - Không đồng ý khiếu nại                  ║
         ║    └── Lý do từ chối *:                              ║
         ║        [_________________________________]           ║
         ║    └── Bằng chứng: [📷 Upload]                       ║
         ║                                                       ║
         ║  ○ ĐỔI HÀNG - Gửi items thay thế                     ║
         ║    └── Số lượng đổi: [___]                           ║
         ║    └── Upload items mới: [📤 Upload file]            ║
         ║                                                       ║
         ║  [Gửi phản hồi]                                       ║
         ╚═══════════════════════════════════════════════════════╝
         │
         ▼
[Bước 5] Vendor submit phản hồi
         │
         ▼
[Bước 6] Xử lý theo quyết định:

         ├── CHẤP NHẬN FULL:
         │   ├── status = resolved
         │   ├── Hoàn tiền buyer (xem Flow Refund)
         │   ├── Trừ từ pending balance vendor
         │   └── Kết thúc dispute
         │
         ├── CHẤP NHẬN MỘT PHẦN:
         │   ├── status = vendor_responded
         │   ├── Chờ buyer confirm
         │   └── Buyer có thể accept hoặc escalate
         │
         ├── TỪ CHỐI:
         │   ├── status = vendor_responded
         │   ├── Chờ buyer xem xét
         │   └── Buyer có thể accept hoặc escalate
         │
         └── ĐỔI HÀNG:
             ├── Tạo replacement items
             ├── status = vendor_responded
             └── Chờ buyer confirm nhận đủ
         │
         ▼
[Bước 7] Gửi notification cho buyer
         │
         ▼
[Bước 8] Update dispute với response details
```

### 4.2 Timeline phản hồi

```
┌─────────────────────────────────────────────────────────────────┐
│                 TIMELINE XỬ LÝ DISPUTE                          │
└─────────────────────────────────────────────────────────────────┘

T+0:      Buyer tạo dispute
          └── Vendor có 48h để phản hồi

T+48h:    Nếu vendor không phản hồi
          └── Auto escalate lên Admin
          └── Tăng mức độ nghiêm trọng

T+0-48h:  Vendor phản hồi
          └── Buyer có 24h để phản hồi lại

T+48h+24h: Nếu buyer không phản hồi
          └── Nếu vendor accept → auto complete refund
          └── Nếu vendor reject → auto close dispute

Lưu ý:
- Mỗi bên có 3 lượt trao đổi trước khi bắt buộc escalate
- Admin can thiệp bất cứ lúc nào nếu cần
```

---

## 5. Escalate lên Admin

### 5.1 Khi nào escalate

```
┌─────────────────────────────────────────────────────────────────┐
│                  ĐIỀU KIỆN ESCALATE                             │
└─────────────────────────────────────────────────────────────────┘

Tự động escalate:
1. Vendor không phản hồi trong 48h
2. Đã trao đổi 3 lượt mà không đạt thỏa thuận
3. Dispute liên quan đến số tiền lớn (> 1,000,000đ)

Buyer yêu cầu escalate:
1. Không đồng ý với phản hồi của vendor
2. Cho rằng vendor không thiện chí

Vendor yêu cầu escalate:
1. Cho rằng buyer lạm dụng dispute
2. Cần admin xác minh bằng chứng
```

### 5.2 Flow escalate

```
┌─────────────────────────────────────────────────────────────────┐
│                   FLOW ESCALATE                                 │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Buyer/Vendor click "Yêu cầu Admin xử lý"
         │
         ▼
[Bước 2] Nhập lý do escalate:
         ┌───────────────────────────────────────────────────────┐
         │ Vui lòng cho biết lý do bạn muốn Admin can thiệp:    │
         │ [____________________________________________]        │
         │ [____________________________________________]        │
         │                                                       │
         │ Thêm bằng chứng bổ sung: [📷 Upload]                 │
         └───────────────────────────────────────────────────────┘
         │
         ▼
[Bước 3] Cập nhật dispute:
         - status = escalated
         - escalated_at = NOW()
         - escalated_by = buyer/vendor
         - escalate_reason
         │
         ▼
[Bước 4] Gửi notification cho Admin
         │
         ▼
[Bước 5] Thông báo cho cả 2 bên:
         "Dispute đã được chuyển lên Admin. 
          Thời gian xử lý: 24-48 giờ."
```

---

## 6. Admin xử lý Dispute

### 6.1 Dashboard Disputes (Admin)

```
┌─────────────────────────────────────────────────────────────────┐
│              ADMIN DISPUTES DASHBOARD                           │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  DISPUTES CẦN XỬ LÝ                                            │
├─────────────────────────────────────────────────────────────────┤
│  🔴 Khẩn cấp (>48h): 3                                         │
│  🟡 Escalated: 8                                                │
│  🟢 Pending vendor: 15                                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Filter: [Escalated ▼] [Tất cả shop ▼] [7 ngày ▼]             │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │ #DIS-001  │ Gmail x10 │ 125,000đ │ 🟡 Escalated │ 2h ago │ │
│  │ Buyer: user123 │ Shop: TechAccount │ Reason: not_working  │ │
│  │                                              [Xử lý ngay] │ │
│  └───────────────────────────────────────────────────────────┘ │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 6.2 Flow Admin review

```
┌─────────────────────────────────────────────────────────────────┐
│                FLOW ADMIN XỬ LÝ DISPUTE                         │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Admin mở chi tiết dispute
         │
         ▼
[Bước 2] Xem toàn bộ thông tin:

         ╔═══════════════════════════════════════════════════════╗
         ║  DISPUTE #DIS-20240115-001                            ║
         ╠═══════════════════════════════════════════════════════╣
         ║                                                       ║
         ║  ┌─────────────────┐    ┌─────────────────┐          ║
         ║  │     BUYER       │    │     VENDOR      │          ║
         ║  │ user123         │    │ TechAccount     │          ║
         ║  │ Đơn: 45         │    │ Đơn: 1,234      │          ║
         ║  │ Dispute: 2 (4%) │    │ Dispute: 15 (1%)│          ║
         ║  │ Trust: ⭐⭐⭐⭐  │    │ Trust: ⭐⭐⭐⭐⭐ │          ║
         ║  └─────────────────┘    └─────────────────┘          ║
         ║                                                       ║
         ║  ─────────────────────────────────────────────────── ║
         ║  THÔNG TIN ĐƠN HÀNG                                  ║
         ║  Order: #ORD-20240115-12345                          ║
         ║  Sản phẩm: Gmail US Aged x 10                        ║
         ║  Tổng: 125,000đ | Đã giao: 15/01 10:30              ║
         ║                                                       ║
         ║  ─────────────────────────────────────────────────── ║
         ║  NỘI DUNG DISPUTE                                    ║
         ║  Lý do: Sản phẩm không hoạt động                     ║
         ║  Items lỗi: 5/10                                     ║
         ║  Yêu cầu: Hoàn tiền 62,500đ                          ║
         ║                                                       ║
         ║  ─────────────────────────────────────────────────── ║
         ║  TIMELINE TRAO ĐỔI                                   ║
         ║  [15/01 11:00] Buyer: Tạo dispute                    ║
         ║  [15/01 15:00] Vendor: Từ chối, claim hàng tốt       ║
         ║  [15/01 16:00] Buyer: Không đồng ý, escalate         ║
         ║                                                       ║
         ║  ─────────────────────────────────────────────────── ║
         ║  BẰNG CHỨNG                                          ║
         ║  Buyer: [📷 Screenshot lỗi đăng nhập]                ║
         ║  Vendor: [📷 Screenshot test account OK]             ║
         ║                                                       ║
         ╚═══════════════════════════════════════════════════════╝
         │
         ▼
[Bước 3] Admin xem xét và quyết định:

         ╔═══════════════════════════════════════════════════════╗
         ║  QUYẾT ĐỊNH CỦA ADMIN                                 ║
         ╠═══════════════════════════════════════════════════════╣
         ║                                                       ║
         ║  ○ HOÀN TIỀN TOÀN BỘ CHO BUYER                       ║
         ║    └── Hoàn 125,000đ                                 ║
         ║                                                       ║
         ║  ○ HOÀN TIỀN MỘT PHẦN                                ║
         ║    └── Số tiền: [_______] VNĐ                        ║
         ║                                                       ║
         ║  ○ TỪ CHỐI DISPUTE - BUYER SAI                       ║
         ║    └── Không hoàn tiền                               ║
         ║    └── Cảnh cáo buyer (nếu cần)                      ║
         ║                                                       ║
         ║  ○ TỪ CHỐI DISPUTE - KHÔNG ĐỦ BẰNG CHỨNG            ║
         ║    └── Không hoàn tiền                               ║
         ║                                                       ║
         ║  LÝ DO QUYẾT ĐỊNH *:                                 ║
         ║  ┌───────────────────────────────────────────────────┐║
         ║  │ [Chi tiết lý do admin đưa ra quyết định]         │║
         ║  └───────────────────────────────────────────────────┘║
         ║                                                       ║
         ║  HÀNH ĐỘNG BỔ SUNG:                                  ║
         ║  □ Cảnh cáo buyer về hành vi abuse                   ║
         ║  □ Cảnh cáo vendor về chất lượng                     ║
         ║  □ Tăng mức giám sát shop                            ║
         ║                                                       ║
         ║  [Xác nhận quyết định]                                ║
         ╚═══════════════════════════════════════════════════════╝
         │
         ▼
[Bước 4] Admin confirm quyết định
         │
         ▼
[Bước 5] Thực hiện action:
         │
         ├── Refund Full:
         │   └── Flow Refund với amount = order.total
         │
         ├── Refund Partial:
         │   └── Flow Refund với amount = admin input
         │
         └── Reject:
             └── Close dispute, release tiền cho vendor
         │
         ▼
[Bước 6] Cập nhật dispute:
         - status = refunded/partial_refund/rejected
         - resolved_at = NOW()
         - resolved_by = admin_id
         - resolution_note
         │
         ▼
[Bước 7] Cập nhật order status
         │
         ▼
[Bước 8] Gửi email cho cả 2 bên với quyết định
         │
         ▼
[Bước 9] Log admin action
```

---

## 7. Flow Refund

### 7.1 Quy trình hoàn tiền

```
┌─────────────────────────────────────────────────────────────────┐
│                     FLOW REFUND                                 │
└─────────────────────────────────────────────────────────────────┘

[Input] dispute_id, refund_amount, resolved_by

[Bước 1] Begin Transaction
         │
         ▼
[Bước 2] Validate:
         - refund_amount <= order.total
         - dispute status is valid for refund
         │
         ▼
[Bước 3] Xác định nguồn tiền refund:
         │
         ├── Tiền còn trong pending (chưa release)?
         │   └── Trừ từ vendor.pending_balance
         │
         └── Tiền đã release?
             └── Trừ từ vendor.available_balance
             └── Nếu không đủ → Log và xử lý manual
         │
         ▼
[Bước 4] Trừ tiền từ vendor wallet:
         UPDATE vendor_wallets SET
           pending_balance = pending_balance - {amount}
           -- hoặc available_balance
         WHERE vendor_id = {vendor_id}
         │
         ▼
[Bước 5] Cộng tiền vào buyer wallet:
         UPDATE wallets SET
           balance = balance + {refund_amount}
         WHERE user_id = {buyer_id}
         │
         ▼
[Bước 6] Tạo transactions:
         - Buyer: type = refund, amount = +refund_amount
         - Vendor: type = refund, amount = -refund_amount
         │
         ▼
[Bước 7] Cập nhật dispute:
         - status = refunded/partial_refund
         - refund_amount
         - resolved_at
         │
         ▼
[Bước 8] Cập nhật order:
         - status = refunded/partial_refund
         - refund_amount
         │
         ▼
[Bước 9] Cập nhật payout (nếu còn pending):
         - status = refunded
         │
         ▼
[Bước 10] Commit Transaction
          │
          ▼
[Bước 11] Gửi notifications và emails
```

---

## 8. Thống kê Dispute

### 8.1 Metrics quan trọng

```
┌─────────────────────────────────────────────────────────────────┐
│                  DISPUTE METRICS                                │
└─────────────────────────────────────────────────────────────────┘

Per Shop:
├── dispute_rate = (disputes / total_orders) * 100%
├── resolve_time_avg = Thời gian TB giải quyết
├── refund_rate = (refunded_disputes / total_disputes) * 100%
└── escalation_rate = (escalated / total_disputes) * 100%

Platform:
├── total_disputes_pending
├── total_disputes_today
├── avg_resolution_time
├── refund_amount_today
└── top_dispute_reasons

Warning thresholds:
├── Shop dispute_rate > 5% → Cảnh báo
├── Shop dispute_rate > 10% → Giám sát
├── Shop dispute_rate > 20% → Xem xét suspend
```

---

## Database Schema

### Bảng disputes

```sql
CREATE TABLE disputes (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    dispute_number VARCHAR(30) UNIQUE,
    
    order_id BIGINT NOT NULL,
    buyer_id BIGINT NOT NULL,
    shop_id BIGINT NOT NULL,
    
    -- Dispute info
    reason ENUM('wrong_item', 'not_working', 'duplicate', 
                'missing_items', 'quality_issue', 'partial_working', 'other'),
    affected_quantity INT,
    affected_items TEXT,                       -- JSON list of affected items
    description TEXT NOT NULL,
    
    -- Request
    requested_action ENUM('full_refund', 'partial_refund', 'replacement'),
    requested_amount DECIMAL(12,0),
    
    -- Status
    status ENUM('pending', 'vendor_responded', 'escalated', 
                'admin_review', 'resolved', 'refunded', 
                'partial_refund', 'rejected', 'closed'),
    
    -- Vendor response
    vendor_response TEXT,
    vendor_action ENUM('accept', 'partial_accept', 'reject', 'replacement'),
    vendor_offer_amount DECIMAL(12,0),
    vendor_responded_at TIMESTAMP NULL,
    
    -- Escalation
    escalated_at TIMESTAMP NULL,
    escalated_by ENUM('buyer', 'vendor', 'system'),
    escalate_reason TEXT,
    
    -- Resolution
    resolved_by BIGINT NULL,                   -- Admin user_id
    resolved_at TIMESTAMP NULL,
    resolution_note TEXT,
    final_action ENUM('full_refund', 'partial_refund', 'rejected'),
    refund_amount DECIMAL(12,0),
    
    -- Timing
    deadline TIMESTAMP,                        -- Vendor response deadline
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    FOREIGN KEY (order_id) REFERENCES orders(id),
    FOREIGN KEY (buyer_id) REFERENCES users(id),
    FOREIGN KEY (shop_id) REFERENCES shops(id),
    FOREIGN KEY (resolved_by) REFERENCES users(id),
    
    INDEX idx_status (status),
    INDEX idx_shop (shop_id),
    INDEX idx_deadline (status, deadline)
);
```

### Bảng dispute_messages

```sql
CREATE TABLE dispute_messages (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    dispute_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    user_type ENUM('buyer', 'vendor', 'admin'),
    message TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (dispute_id) REFERENCES disputes(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);
```

### Bảng dispute_attachments

```sql
CREATE TABLE dispute_attachments (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    dispute_id BIGINT NOT NULL,
    uploaded_by BIGINT NOT NULL,
    file_path VARCHAR(255) NOT NULL,
    file_type VARCHAR(50),
    file_size INT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (dispute_id) REFERENCES disputes(id),
    FOREIGN KEY (uploaded_by) REFERENCES users(id)
);
```
