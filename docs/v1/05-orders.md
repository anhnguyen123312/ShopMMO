# Chức năng Đơn hàng (Orders)

## Tổng quan

Hệ thống đơn hàng của TaphoaMMO được thiết kế cho sản phẩm số với đặc điểm: **giao hàng tự động ngay lập tức** sau khi thanh toán. Buyer thanh toán bằng số dư ví, hệ thống xuất sản phẩm từ kho và hiển thị ngay trên màn hình.

---

## 1. Các trạng thái đơn hàng

### 1.1 Sơ đồ trạng thái

```
┌─────────────────────────────────────────────────────────────────┐
│                   TRẠNG THÁI ĐƠN HÀNG                           │
└─────────────────────────────────────────────────────────────────┘

                    ┌─────────────┐
                    │   PENDING   │ ◄── Đơn vừa tạo, chờ thanh toán
                    └──────┬──────┘
                           │
              ┌────────────┼────────────┐
              │            │            │
              ▼            ▼            ▼
       ┌──────────┐ ┌──────────┐ ┌──────────┐
       │ CANCELLED│ │  FAILED  │ │   PAID   │
       └──────────┘ └──────────┘ └────┬─────┘
       User hủy     Lỗi thanh toán    │
                                      │ Auto delivery
                                      ▼
                               ┌──────────┐
                               │DELIVERED │ ◄── Đã giao sản phẩm
                               └────┬─────┘
                                    │
                    ┌───────────────┼───────────────┐
                    │               │               │
                    ▼               ▼               ▼
             ┌──────────┐   ┌──────────┐   ┌──────────┐
             │COMPLETED │   │ DISPUTED │   │ REFUNDED │
             └──────────┘   └────┬─────┘   └──────────┘
             Sau 3 ngày,    Buyer khiếu nại Full refund
             không dispute        │
                                  │
                    ┌─────────────┼─────────────┐
                    │             │             │
                    ▼             ▼             ▼
             ┌──────────┐ ┌──────────┐ ┌──────────┐
             │COMPLETED │ │REFUNDED  │ │ PARTIAL  │
             └──────────┘ └──────────┘ │ REFUND   │
             Dispute rejected         └──────────┘
```

### 1.2 Chi tiết trạng thái

| Status | Mô tả | Buyer có thể | Vendor có thể |
|--------|-------|--------------|---------------|
| pending | Đang chờ thanh toán | Hủy, Thanh toán | - |
| cancelled | Buyer đã hủy | - | - |
| failed | Thanh toán thất bại | Thử lại | - |
| paid | Đã thanh toán, đang xử lý | - | - |
| delivered | Đã giao sản phẩm | Khiếu nại (3 ngày) | Xem chi tiết |
| disputed | Đang khiếu nại | Theo dõi | Phản hồi |
| completed | Hoàn thành | Đánh giá | Nhận tiền |
| refunded | Đã hoàn tiền | - | - |
| partial_refund | Hoàn một phần | - | - |

---

## 2. Flow Mua hàng

### 2.1 Flow mua hàng thông thường

```
┌─────────────────────────────────────────────────────────────────┐
│                    FLOW MUA HÀNG                                │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Buyer ở trang sản phẩm
         │
         ▼
[Bước 2] Chọn số lượng muốn mua
         │
         ├── Quantity > stock ──► Alert "Không đủ hàng"
         ├── Quantity < min_purchase ──► Alert "Tối thiểu X items"
         ├── Quantity > max_purchase ──► Alert "Tối đa X items"
         │
         ▼
[Bước 3] Nhập mã giảm giá (optional)
         │
         ├── Mã không hợp lệ ──► Alert lỗi
         ├── Mã hợp lệ ──► Hiển thị số tiền giảm
         │
         ▼
[Bước 4] Hiển thị tóm tắt đơn hàng:
         
         ╔═══════════════════════════════════════════════════════╗
         ║  TÓM TẮT ĐƠN HÀNG                                     ║
         ╠═══════════════════════════════════════════════════════╣
         ║  Sản phẩm: Gmail US Aged                              ║
         ║  Đơn giá:  15,000đ                                    ║
         ║  Số lượng: 10                                         ║
         ║  ─────────────────────────────                        ║
         ║  Tạm tính: 150,000đ                                   ║
         ║  Giảm giá SL (10%): -15,000đ                          ║
         ║  Mã giảm giá: -10,000đ                                ║
         ║  ─────────────────────────────                        ║
         ║  TỔNG THANH TOÁN: 125,000đ                            ║
         ║                                                       ║
         ║  Số dư ví: 500,000đ ✓                                 ║
         ║                                                       ║
         ║  [Hủy]  [Xác nhận mua hàng]                           ║
         ╚═══════════════════════════════════════════════════════╝
         │
         ▼
[Bước 5] Buyer click "Xác nhận mua hàng"
         │
         ▼
[Bước 6] Kiểm tra đăng nhập
         │
         ├── Chưa login ──► Redirect login với return URL
         │
         ▼
[Bước 7] Kiểm tra 2FA (nếu product yêu cầu)
         │
         ├── Cần 2FA ──► Yêu cầu nhập OTP
         │
         ▼
[Bước 8] Kiểm tra số dư ví
         │
         ├── Không đủ ──► Hiển thị nút "Nạp thêm tiền"
         │               Giữ nguyên đơn hàng pending
         │
         ▼
[Bước 9] *** BẮT ĐẦU TRANSACTION ***
         │
         ▼
[Bước 10] Lock stock items (Pessimistic Locking)
          
          SELECT * FROM product_items
          WHERE product_id = X 
            AND is_sold = false 
            AND hold_until IS NULL
          ORDER BY created_at ASC
          LIMIT {quantity}
          FOR UPDATE
          │
          ├── Không đủ items ──► Rollback, "Sản phẩm vừa hết hàng"
          │
          ▼
[Bước 11] Trừ tiền từ ví buyer
          
          UPDATE wallets SET balance = balance - {total}
          WHERE user_id = {buyer_id} AND balance >= {total}
          │
          ├── Affected rows = 0 ──► Rollback, "Không đủ số dư"
          │
          ▼
[Bước 12] Tạo Order record
          
          INSERT INTO orders (
            order_number, buyer_id, shop_id, 
            subtotal, discount, total,
            status, created_at
          )
          │
          ▼
[Bước 13] Tạo OrderItems và đánh dấu ProductItems đã bán
          
          FOR each locked item:
            - UPDATE product_items SET is_sold = true, 
                     order_id = {order_id}, sold_at = NOW()
            - INSERT INTO order_items (order_id, product_item_id, content)
          │
          ▼
[Bước 14] Cập nhật thống kê
          - Product: stock -= quantity, total_sold += quantity
          - Shop: total_sales += 1
          │
          ▼
[Bước 15] Tạo Transaction records
          - Buyer: type = purchase, amount = -total
          - Vendor: type = sale, amount = +total (pending 3 days)
          │
          ▼
[Bước 16] Xử lý coupon (nếu có)
          - Tăng usage count
          - Link coupon với order
          │
          ▼
[Bước 17] *** COMMIT TRANSACTION ***
          │
          ▼
[Bước 18] Gửi notifications
          - Email buyer: Chi tiết đơn hàng
          - Notify vendor: Có đơn hàng mới
          │
          ▼
[Bước 19] Hiển thị trang thành công với nội dung sản phẩm:

          ╔═══════════════════════════════════════════════════════╗
          ║  ✅ ĐẶT HÀNG THÀNH CÔNG!                              ║
          ╠═══════════════════════════════════════════════════════╣
          ║  Mã đơn hàng: #ORD-20240115-12345                     ║
          ║  Sản phẩm: Gmail US Aged x 10                         ║
          ║                                                       ║
          ║  NỘI DUNG SẢN PHẨM:                                   ║
          ║  ┌───────────────────────────────────────────────────┐║
          ║  │ email1@gmail.com|password1|2FA_SECRET1           │║
          ║  │ email2@gmail.com|password2|2FA_SECRET2           │║
          ║  │ email3@gmail.com|password3|2FA_SECRET3           │║
          ║  │ ... (7 items nữa)                                 │║
          ║  └───────────────────────────────────────────────────┘║
          ║                                                       ║
          ║  [📋 Copy tất cả]  [📥 Tải file TXT]                  ║
          ║                                                       ║
          ║  ⚠️ Vui lòng kiểm tra sản phẩm trong 3 ngày.         ║
          ║  Sau 3 ngày, đơn hàng sẽ tự động hoàn thành.         ║
          ║                                                       ║
          ║  [Khiếu nại]  [Xem lịch sử đơn hàng]                  ║
          ╚═══════════════════════════════════════════════════════╝
```

### 2.2 Xử lý Race Condition

```
┌─────────────────────────────────────────────────────────────────┐
│              XỬ LÝ RACE CONDITION KHI MUA HÀNG                  │
└─────────────────────────────────────────────────────────────────┘

Scenario: 2 buyers cùng mua 1 sản phẩm chỉ còn 1 item

Timeline:
─────────────────────────────────────────────────────────────────
T1: Buyer A click mua ──► Bắt đầu transaction A
T2: Buyer B click mua ──► Bắt đầu transaction B
T3: Transaction A: SELECT ... FOR UPDATE ──► Lock item #1
T4: Transaction B: SELECT ... FOR UPDATE ──► WAIT (bị block)
T5: Transaction A: Update, Commit ──► Release lock
T6: Transaction B: Retry SELECT ──► Không còn item available
T7: Transaction B: Rollback ──► Thông báo "Hết hàng"
─────────────────────────────────────────────────────────────────

Implementation:
- Sử dụng SELECT ... FOR UPDATE để lock rows
- Transaction isolation level: READ COMMITTED
- Timeout cho lock: 10 giây
- Nếu timeout ──► Rollback và yêu cầu thử lại
```

---

## 3. Xem Lịch sử Đơn hàng

### 3.1 Danh sách đơn hàng (Buyer)

```
┌─────────────────────────────────────────────────────────────────┐
│              FLOW XEM LỊCH SỬ ĐƠN HÀNG (BUYER)                  │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Buyer vào "Đơn hàng của tôi"
         │
         ▼
[Bước 2] Query orders với filters:
         
         SELECT * FROM orders
         WHERE buyer_id = {user_id}
         ORDER BY created_at DESC
         │
         ▼
[Bước 3] Hiển thị danh sách:

┌─────────────────────────────────────────────────────────────────┐
│  LỊCH SỬ ĐƠN HÀNG                                              │
├─────────────────────────────────────────────────────────────────┤
│  Filter: [Tất cả ▼] [Tháng này ▼]  Search: [___________]       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │ #ORD-20240115-12345              15/01/2024 10:30         │ │
│  │ Gmail US Aged x 10               Shop: TechAccount         │ │
│  │ 125,000đ                         🟢 Đã giao                │ │
│  │                     [Xem chi tiết] [Khiếu nại] [Đánh giá]  │ │
│  └───────────────────────────────────────────────────────────┘ │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │ #ORD-20240114-12340              14/01/2024 15:45         │ │
│  │ Facebook Account x 5             Shop: SocialPro           │ │
│  │ 100,000đ                         ✅ Hoàn thành             │ │
│  │                                  [Xem chi tiết] [Mua lại]  │ │
│  └───────────────────────────────────────────────────────────┘ │
│                                                                 │
│  [< Prev] [1] [2] [3] ... [Next >]                             │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 Chi tiết đơn hàng

```
┌─────────────────────────────────────────────────────────────────┐
│                 FLOW XEM CHI TIẾT ĐƠN HÀNG                      │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Buyer click "Xem chi tiết"
         │
         ▼
[Bước 2] Query order với order_items:
         
         SELECT o.*, oi.*, pi.content
         FROM orders o
         JOIN order_items oi ON o.id = oi.order_id
         JOIN product_items pi ON oi.product_item_id = pi.id
         WHERE o.id = {order_id} AND o.buyer_id = {user_id}
         │
         ▼
[Bước 3] Hiển thị chi tiết:

╔═══════════════════════════════════════════════════════════════════╗
║  CHI TIẾT ĐƠN HÀNG #ORD-20240115-12345                           ║
╠═══════════════════════════════════════════════════════════════════╣
║                                                                   ║
║  Trạng thái: 🟢 Đã giao                                          ║
║  Thời gian đặt: 15/01/2024 10:30:45                              ║
║  Thời gian giao: 15/01/2024 10:30:46 (tự động)                   ║
║                                                                   ║
║  ─────────────────────────────────────────────────────────────── ║
║  THÔNG TIN SẢN PHẨM                                              ║
║  ─────────────────────────────────────────────────────────────── ║
║  Sản phẩm: Gmail US Aged                                         ║
║  Shop: TechAccount                                                ║
║  Đơn giá: 15,000đ                                                ║
║  Số lượng: 10                                                    ║
║                                                                   ║
║  ─────────────────────────────────────────────────────────────── ║
║  THANH TOÁN                                                      ║
║  ─────────────────────────────────────────────────────────────── ║
║  Tạm tính:           150,000đ                                    ║
║  Giảm giá số lượng:  -15,000đ                                    ║
║  Mã giảm giá (SAVE10): -10,000đ                                  ║
║  ─────────────────────────────────                               ║
║  Tổng thanh toán:    125,000đ                                    ║
║                                                                   ║
║  ─────────────────────────────────────────────────────────────── ║
║  NỘI DUNG SẢN PHẨM                                               ║
║  ─────────────────────────────────────────────────────────────── ║
║  ┌───────────────────────────────────────────────────────────┐   ║
║  │ email1@gmail.com|password1|JBSWY3DPEHPK3PXP              │   ║
║  │ email2@gmail.com|password2|KRSXG5CTMVRXEZLU              │   ║
║  │ email3@gmail.com|password3|GEZDGNBVGY3TQOJQ              │   ║
║  │ ... (xem thêm 7 items)                                    │   ║
║  └───────────────────────────────────────────────────────────┘   ║
║                                                                   ║
║  [📋 Copy all]  [📥 Download TXT]  [🔄 Tạo mã 2FA]               ║
║                                                                   ║
║  ─────────────────────────────────────────────────────────────── ║
║  ⏰ Thời hạn khiếu nại: còn 2 ngày 15 giờ                        ║
║                                                                   ║
║  [Khiếu nại đơn hàng]                [Quay lại danh sách]        ║
╚═══════════════════════════════════════════════════════════════════╝
```

---

## 4. Quản lý Đơn hàng (Vendor)

### 4.1 Dashboard đơn hàng

```
┌─────────────────────────────────────────────────────────────────┐
│              FLOW QUẢN LÝ ĐƠN HÀNG (VENDOR)                     │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Vendor vào "Quản lý đơn hàng"
         │
         ▼
[Bước 2] Hiển thị thống kê nhanh:
         
         ╔═══════════════════════════════════════════════════════╗
         ║  Hôm nay: 25 đơn | 2,500,000đ                         ║
         ║  Đang chờ xử lý: 0 | Khiếu nại: 2                     ║
         ╚═══════════════════════════════════════════════════════╝
         │
         ▼
[Bước 3] Danh sách đơn hàng với filters:
         - Trạng thái
         - Khoảng thời gian
         - Sản phẩm
         - Buyer
         │
         ▼
[Bước 4] Hiển thị bảng:

┌─────────────────────────────────────────────────────────────────┐
│ Mã đơn    │ Buyer   │ Sản phẩm      │ Tổng    │ TT   │ Thời gian│
├───────────┼─────────┼───────────────┼─────────┼──────┼──────────┤
│ #12345    │ user123 │ Gmail x 10    │ 125,000 │ 🟢   │ 10:30    │
│ #12344    │ user456 │ FB x 5        │ 100,000 │ 🔴   │ 10:15    │
│ #12343    │ user789 │ Github x 2    │ 80,000  │ ✅   │ 09:45    │
└─────────────────────────────────────────────────────────────────┘

Chú thích:
🟢 Đã giao (trong thời gian khiếu nại)
🔴 Đang khiếu nại
✅ Hoàn thành
```

### 4.2 Chi tiết đơn hàng (Vendor view)

```
┌─────────────────────────────────────────────────────────────────┐
│           CHI TIẾT ĐƠN HÀNG (VENDOR VIEW)                       │
└─────────────────────────────────────────────────────────────────┘

Vendor xem chi tiết đơn hàng:

╔═══════════════════════════════════════════════════════════════════╗
║  ĐƠN HÀNG #ORD-20240115-12345                                    ║
╠═══════════════════════════════════════════════════════════════════╣
║                                                                   ║
║  THÔNG TIN BUYER                                                 ║
║  ─────────────────                                               ║
║  Username: user123                                                ║
║  Tham gia: 01/2023                                               ║
║  Tổng đơn: 45 đơn | Khiếu nại: 2 (4.4%)                         ║
║                                                                   ║
║  THÔNG TIN ĐƠN HÀNG                                              ║
║  ─────────────────                                               ║
║  Sản phẩm: Gmail US Aged x 10                                    ║
║  Tổng: 125,000đ                                                  ║
║  Phí sàn (5%): 6,250đ                                            ║
║  Thực nhận: 118,750đ                                             ║
║                                                                   ║
║  TRẠNG THÁI THANH TOÁN                                           ║
║  ─────────────────────                                           ║
║  ⏳ Pending: Tiền sẽ về ví sau 2 ngày 15 giờ                     ║
║                                                                   ║
║  NỘI DUNG ĐÃ GIAO (đã mask một phần)                            ║
║  ─────────────────────────────────────                           ║
║  email1@gm***.com|pass***|JBSW***                                ║
║  email2@gm***.com|pass***|KRSX***                                ║
║  ... (8 items khác)                                              ║
║                                                                   ║
╚═══════════════════════════════════════════════════════════════════╝

Lưu ý: Vendor không thấy full content sau khi bán
       để tránh lạm dụng sản phẩm đã bán
```

---

## 5. Tự động hoàn thành đơn hàng

### Flow auto-complete

```
┌─────────────────────────────────────────────────────────────────┐
│              FLOW TỰ ĐỘNG HOÀN THÀNH ĐƠN HÀNG                   │
└─────────────────────────────────────────────────────────────────┘

Cron Job chạy mỗi giờ:

[Bước 1] Query đơn hàng cần auto-complete:
         
         SELECT * FROM orders
         WHERE status = 'delivered'
           AND dispute_deadline < NOW()
           AND NOT EXISTS (
             SELECT 1 FROM disputes 
             WHERE order_id = orders.id 
               AND status IN ('pending', 'processing')
           )
         │
         ▼
[Bước 2] Với mỗi đơn hàng:
         │
         ▼
[Bước 3] Cập nhật trạng thái:
         UPDATE orders SET status = 'completed', completed_at = NOW()
         │
         ▼
[Bước 4] Release tiền cho vendor:
         
         - Tìm pending payout record
         - Cập nhật status = released
         - Cộng vào available_balance của vendor
         │
         ▼
[Bước 5] Tạo transaction record cho vendor:
         type = sale_completed
         │
         ▼
[Bước 6] Gửi notification cho vendor:
         "Đơn hàng #XXX đã hoàn thành. +118,750đ vào ví."
         │
         ▼
[Bước 7] Log action

─────────────────────────────────────────────────────────────────

Timeline của đơn hàng:
─────────────────────────────────────────────────────────────────
T+0:        Buyer mua, tiền trừ ngay
T+0:        Sản phẩm giao ngay
T+0 → T+3:  Thời gian khiếu nại (3 ngày = 72 giờ)
T+3:        Không có dispute → Auto complete
T+3:        Tiền release cho vendor
─────────────────────────────────────────────────────────────────
```

---

## 6. Hủy đơn hàng

### Flow hủy đơn (chỉ áp dụng khi pending)

```
┌─────────────────────────────────────────────────────────────────┐
│                    FLOW HỦY ĐƠN HÀNG                            │
└─────────────────────────────────────────────────────────────────┘

Điều kiện hủy:
- Đơn hàng status = pending (chưa thanh toán)
- Chỉ buyer mới có quyền hủy

[Bước 1] Buyer vào chi tiết đơn pending
         │
         ▼
[Bước 2] Click "Hủy đơn hàng"
         │
         ▼
[Bước 3] Confirm: "Bạn có chắc muốn hủy đơn hàng này?"
         │
         ▼
[Bước 4] Cập nhật order:
         - status = cancelled
         - cancelled_at = NOW()
         - cancelled_by = buyer
         │
         ▼
[Bước 5] Release hold trên stock (nếu có):
         UPDATE product_items 
         SET hold_until = NULL, preorder_id = NULL
         WHERE preorder_id = {order_id}
         │
         ▼
[Bước 6] Hoàn coupon (nếu đã dùng):
         - Giảm usage count
         - Unlink từ order
         │
         ▼
[Bước 7] Hiển thị "Đã hủy đơn hàng"

Lưu ý: Đơn đã paid/delivered KHÔNG thể hủy
       Phải đi qua quy trình Khiếu nại
```

---

## 7. Mua lại (Reorder)

### Flow mua lại

```
┌─────────────────────────────────────────────────────────────────┐
│                     FLOW MUA LẠI                                │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Buyer ở đơn hàng đã completed
         │
         ▼
[Bước 2] Click "Mua lại"
         │
         ▼
[Bước 3] Kiểm tra sản phẩm còn tồn tại và active
         │
         ├── Không còn ──► "Sản phẩm không còn bán"
         │
         ▼
[Bước 4] Kiểm tra tồn kho
         │
         ├── Hết hàng ──► Redirect trang SP, hiển thị "Hết hàng"
         │
         ▼
[Bước 5] Pre-fill form mua hàng:
         - Số lượng = số lượng đơn cũ (hoặc max available)
         - Giá hiện tại (có thể khác đơn cũ)
         │
         ▼
[Bước 6] Tiếp tục Flow mua hàng bình thường
```

---

## 8. Export đơn hàng

### Flow export (Vendor)

```
┌─────────────────────────────────────────────────────────────────┐
│                  FLOW EXPORT ĐƠN HÀNG                           │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Vendor vào Đơn hàng > Export
         │
         ▼
[Bước 2] Chọn options:
         
         ╔═══════════════════════════════════════════════════════╗
         ║  EXPORT ĐƠN HÀNG                                      ║
         ╠═══════════════════════════════════════════════════════╣
         ║  Khoảng thời gian:                                    ║
         ║  [01/01/2024] đến [15/01/2024]                        ║
         ║                                                       ║
         ║  Trạng thái:                                          ║
         ║  ☑ Hoàn thành  ☐ Đang xử lý  ☐ Khiếu nại            ║
         ║                                                       ║
         ║  Sản phẩm:                                            ║
         ║  [▼ Tất cả sản phẩm                              ]    ║
         ║                                                       ║
         ║  Format:                                              ║
         ║  ○ Excel (.xlsx)                                      ║
         ║  ○ CSV                                                ║
         ║                                                       ║
         ║  Nội dung:                                            ║
         ║  ☑ Thông tin đơn  ☑ Thống kê  ☐ Chi tiết items      ║
         ║                                                       ║
         ║  [Export]                                             ║
         ╚═══════════════════════════════════════════════════════╝
         │
         ▼
[Bước 3] Generate file
         │
         ▼
[Bước 4] Download file

Lưu ý: Không export full content của items đã bán
       Chỉ export thống kê và thông tin đơn hàng
```

---

## Database Schema

### Bảng orders

```sql
CREATE TABLE orders (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    order_number VARCHAR(30) UNIQUE,      -- ORD-YYYYMMDD-XXXXX
    buyer_id BIGINT NOT NULL,
    shop_id BIGINT NOT NULL,
    product_id BIGINT NOT NULL,
    
    -- Pricing
    quantity INT NOT NULL,
    unit_price DECIMAL(12,0) NOT NULL,
    subtotal DECIMAL(12,0) NOT NULL,
    discount_amount DECIMAL(12,0) DEFAULT 0,
    coupon_id BIGINT NULL,
    total DECIMAL(12,0) NOT NULL,
    
    -- Commission
    commission_rate DECIMAL(5,2),          -- % phí sàn
    commission_amount DECIMAL(12,0),
    vendor_amount DECIMAL(12,0),           -- Số tiền vendor nhận
    
    -- Status
    status ENUM('pending', 'cancelled', 'failed', 'paid', 
                'delivered', 'disputed', 'completed', 
                'refunded', 'partial_refund'),
    
    -- Timestamps
    paid_at TIMESTAMP NULL,
    delivered_at TIMESTAMP NULL,
    dispute_deadline TIMESTAMP NULL,       -- paid_at + 3 days
    completed_at TIMESTAMP NULL,
    cancelled_at TIMESTAMP NULL,
    cancelled_by ENUM('buyer', 'system', 'admin') NULL,
    
    -- Refund info
    refund_amount DECIMAL(12,0) NULL,
    refund_reason TEXT NULL,
    refunded_at TIMESTAMP NULL,
    
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    FOREIGN KEY (buyer_id) REFERENCES users(id),
    FOREIGN KEY (shop_id) REFERENCES shops(id),
    FOREIGN KEY (product_id) REFERENCES products(id),
    FOREIGN KEY (coupon_id) REFERENCES coupons(id),
    
    INDEX idx_buyer (buyer_id),
    INDEX idx_shop (shop_id),
    INDEX idx_status (status),
    INDEX idx_created (created_at)
);
```

### Bảng order_items

```sql
CREATE TABLE order_items (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    order_id BIGINT NOT NULL,
    product_item_id BIGINT NOT NULL,
    content TEXT NOT NULL,                 -- Copy content tại thời điểm mua
    
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (order_id) REFERENCES orders(id),
    FOREIGN KEY (product_item_id) REFERENCES product_items(id),
    
    INDEX idx_order (order_id)
);
```

### Generate Order Number

```
Format: ORD-YYYYMMDD-XXXXX

Trong đó:
- ORD: Prefix
- YYYYMMDD: Ngày tạo
- XXXXX: Số tự tăng trong ngày (reset mỗi ngày)

Ví dụ: ORD-20240115-00001, ORD-20240115-00002, ...

Implementation:
- Dùng Redis INCR với key = "order_counter:{date}"
- TTL = 48 hours
- Hoặc dùng sequence table trong DB
```
