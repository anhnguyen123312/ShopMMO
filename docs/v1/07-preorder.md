# Chức năng Đặt trước (Pre-order)

## Tổng quan

Pre-order cho phép Buyer đặt mua sản phẩm khi hết hàng và tự động nhận hàng khi Vendor restock. Tiền được trừ ngay khi đặt và hoàn lại nếu không có hàng trong thời gian cam kết.

---

## 1. Điều kiện Pre-order

### 1.1 Khi nào hiển thị Pre-order

```
┌─────────────────────────────────────────────────────────────────┐
│               ĐIỀU KIỆN HIỂN THỊ PRE-ORDER                      │
└─────────────────────────────────────────────────────────────────┘

Pre-order button hiển thị khi TẤT CẢ điều kiện sau thỏa mãn:

1. Product Settings:
   ├── allow_preorder = true
   └── status = active

2. Stock Status:
   ├── Hết hàng (stock = 0)
   └── HOẶC stock < quantity người mua muốn

3. Shop Status:
   └── shop.status = active

4. Không có quá nhiều pre-orders pending:
   └── pending_preorders < max_preorders (mặc định 100)
```

### 1.2 Các trạng thái Pre-order

| Status | Mô tả | Hành động tiếp |
|--------|-------|----------------|
| pending | Đang chờ hàng | Tự động fulfill hoặc cancel |
| fulfilled | Đã có hàng, đã giao | Chuyển thành Order |
| cancelled | Đã hủy | Hoàn tiền |
| expired | Hết hạn chờ | Auto hoàn tiền |
| refunded | Đã hoàn tiền | - |

---

## 2. Flow đặt Pre-order

### 2.1 Buyer đặt Pre-order

```
┌─────────────────────────────────────────────────────────────────┐
│                  FLOW ĐẶT PRE-ORDER                             │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Buyer ở trang sản phẩm, sản phẩm đã hết hàng
         │
         ▼
[Bước 2] Hiển thị thông tin Pre-order:

         ╔═══════════════════════════════════════════════════════╗
         ║  SẢN PHẨM TẠM HẾT HÀNG                                ║
         ╠═══════════════════════════════════════════════════════╣
         ║                                                       ║
         ║  📦 Gmail US Aged                                     ║
         ║  💰 15,000đ/item                                      ║
         ║                                                       ║
         ║  Bạn có thể đặt trước và nhận hàng khi shop          ║
         ║  có hàng mới.                                         ║
         ║                                                       ║
         ║  Số lượng đặt: [- ] 10 [ +]                          ║
         ║                                                       ║
         ║  Thời gian chờ tối đa:                                ║
         ║  ○ 1 ngày                                             ║
         ║  ○ 3 ngày (khuyến nghị)                               ║
         ║  ○ 7 ngày                                             ║
         ║                                                       ║
         ║  ─────────────────────────────────────────────────── ║
         ║  Tạm tính: 150,000đ                                   ║
         ║  Số dư ví: 500,000đ ✓                                 ║
         ║                                                       ║
         ║  ⚠️ Tiền sẽ được trừ ngay khi đặt.                    ║
         ║  Nếu không có hàng trong thời gian chờ,              ║
         ║  tiền sẽ được hoàn lại 100%.                          ║
         ║                                                       ║
         ║  [Đặt trước ngay]                                     ║
         ╚═══════════════════════════════════════════════════════╝
         │
         ▼
[Bước 3] Buyer chọn số lượng và thời gian chờ
         │
         ▼
[Bước 4] Click "Đặt trước ngay"
         │
         ▼
[Bước 5] Kiểm tra đăng nhập
         │
         ├── Chưa login ──► Redirect login
         │
         ▼
[Bước 6] Kiểm tra số dư ví
         │
         ├── Không đủ ──► "Số dư không đủ, vui lòng nạp thêm"
         │
         ▼
[Bước 7] Kiểm tra giới hạn pre-order:
         - User: max 5 pre-orders pending cùng lúc
         - Product: max 100 pre-orders pending
         │
         ├── Vượt giới hạn ──► Thông báo lỗi
         │
         ▼
[Bước 8] *** BẮT ĐẦU TRANSACTION ***
         │
         ▼
[Bước 9] Trừ tiền từ ví buyer
         │
         ▼
[Bước 10] Tạo Pre-order record:
          - buyer_id
          - product_id
          - shop_id
          - quantity
          - unit_price
          - total_amount
          - status: pending
          - expires_at: NOW() + wait_days
          - priority: timestamp (FIFO)
          │
          ▼
[Bước 11] Tạo Transaction record:
          - type: preorder_hold
          - amount: -total_amount
          │
          ▼
[Bước 12] *** COMMIT TRANSACTION ***
          │
          ▼
[Bước 13] Gửi notifications:
          - Email buyer: Xác nhận đặt trước
          - Notify vendor: Có pre-order mới
          │
          ▼
[Bước 14] Hiển thị xác nhận:

          ╔═══════════════════════════════════════════════════════╗
          ║  ✅ ĐẶT TRƯỚC THÀNH CÔNG!                             ║
          ╠═══════════════════════════════════════════════════════╣
          ║                                                       ║
          ║  Mã đặt trước: #PRE-20240115-00001                   ║
          ║  Sản phẩm: Gmail US Aged x 10                        ║
          ║  Tổng tiền: 150,000đ (đã trừ)                        ║
          ║                                                       ║
          ║  Thời hạn chờ: đến 18/01/2024 10:30                  ║
          ║                                                       ║
          ║  Bạn sẽ nhận được thông báo ngay khi có hàng.        ║
          ║  Nếu hết thời gian chờ, tiền sẽ được hoàn lại.       ║
          ║                                                       ║
          ║  [Xem đơn đặt trước]  [Tiếp tục mua sắm]             ║
          ╚═══════════════════════════════════════════════════════╝
```

---

## 3. Xem và quản lý Pre-order (Buyer)

### 3.1 Danh sách Pre-order

```
┌─────────────────────────────────────────────────────────────────┐
│              DANH SÁCH PRE-ORDER (BUYER)                        │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Buyer vào "Đơn đặt trước"
         │
         ▼
[Bước 2] Hiển thị danh sách:

┌─────────────────────────────────────────────────────────────────┐
│  ĐƠN ĐẶT TRƯỚC                                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │ #PRE-20240115-00001                    ⏳ Đang chờ hàng   │ │
│  │ Gmail US Aged x 10                                        │ │
│  │ 150,000đ                              Shop: TechAccount   │ │
│  │ Hết hạn: còn 2 ngày 15 giờ                               │ │
│  │                                      [Chi tiết] [Hủy]    │ │
│  └───────────────────────────────────────────────────────────┘ │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │ #PRE-20240110-00050                    ✅ Đã nhận hàng    │ │
│  │ Facebook Account x 5                                      │ │
│  │ 100,000đ                              Shop: SocialPro     │ │
│  │ Hoàn thành: 12/01/2024                                   │ │
│  │                                      [Xem đơn hàng]       │ │
│  └───────────────────────────────────────────────────────────┘ │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 Hủy Pre-order

```
┌─────────────────────────────────────────────────────────────────┐
│                  FLOW HỦY PRE-ORDER                             │
└─────────────────────────────────────────────────────────────────┘

Điều kiện hủy:
- Pre-order status = pending
- Chưa được fulfill

[Bước 1] Buyer click "Hủy" trên pre-order pending
         │
         ▼
[Bước 2] Hiển thị confirm:
         "Bạn có chắc muốn hủy đơn đặt trước này?
          Tiền sẽ được hoàn lại vào ví."
         │
         ▼
[Bước 3] Buyer xác nhận
         │
         ▼
[Bước 4] Begin Transaction
         │
         ▼
[Bước 5] Cập nhật pre-order:
         - status = cancelled
         - cancelled_at = NOW()
         - cancelled_by = buyer
         │
         ▼
[Bước 6] Hoàn tiền vào ví buyer:
         UPDATE wallets SET balance = balance + {amount}
         │
         ▼
[Bước 7] Tạo transaction:
         - type: preorder_refund
         - amount: +total_amount
         │
         ▼
[Bước 8] Commit Transaction
         │
         ▼
[Bước 9] Gửi notification
         │
         ▼
[Bước 10] Hiển thị "Đã hủy và hoàn tiền thành công"
```

---

## 4. Auto-fulfill Pre-order (Khi Vendor restock)

### 4.1 Flow tự động fulfill

```
┌─────────────────────────────────────────────────────────────────┐
│             FLOW AUTO-FULFILL KHI RESTOCK                       │
└─────────────────────────────────────────────────────────────────┘

[Trigger] Vendor upload kho hàng mới cho product
         │
         ▼
[Bước 1] Sau khi upload thành công, kiểm tra pre-orders:
         
         SELECT * FROM preorders
         WHERE product_id = {product_id}
           AND status = 'pending'
           AND expires_at > NOW()
         ORDER BY created_at ASC  -- FIFO: ai đặt trước được ưu tiên
         │
         ├── Không có pre-order ──► Kết thúc
         │
         ▼
[Bước 2] Lấy số lượng stock mới:
         new_stock = số items vừa upload
         │
         ▼
[Bước 3] Với mỗi pre-order (theo thứ tự FIFO):
         │
         ▼
[Bước 4] Kiểm tra đủ hàng không:
         │
         ├── new_stock < preorder.quantity
         │   └── Skip, chờ restock tiếp
         │
         ▼
[Bước 5] *** BẮT ĐẦU TRANSACTION ***
         │
         ▼
[Bước 6] Lock stock items:
         SELECT * FROM product_items
         WHERE product_id = {product_id}
           AND is_sold = false
           AND hold_until IS NULL
         ORDER BY created_at ASC
         LIMIT {preorder.quantity}
         FOR UPDATE
         │
         ▼
[Bước 7] Tạo Order từ Pre-order:
         INSERT INTO orders (
           buyer_id, shop_id, product_id,
           quantity, total, status = 'delivered',
           preorder_id
         )
         │
         ▼
[Bước 8] Đánh dấu items đã bán:
         UPDATE product_items SET
           is_sold = true,
           order_id = {new_order_id},
           sold_at = NOW()
         │
         ▼
[Bước 9] Tạo Order Items (copy content)
         │
         ▼
[Bước 10] Cập nhật Pre-order:
          - status = fulfilled
          - fulfilled_at = NOW()
          - order_id = {new_order_id}
          │
          ▼
[Bước 11] Tạo Payout cho vendor (pending 3 days)
          │
          ▼
[Bước 12] Cập nhật thống kê:
          - Product: stock, total_sold
          - Shop: total_sales
          │
          ▼
[Bước 13] *** COMMIT TRANSACTION ***
          │
          ▼
[Bước 14] Gửi notifications:
          - Email buyer: Đơn đặt trước đã có hàng!
          - Push notification
          - Notify vendor: Pre-order đã được fulfill
          │
          ▼
[Bước 15] Cập nhật new_stock và tiếp tục với pre-order tiếp theo
          new_stock -= preorder.quantity
          │
          ├── new_stock <= 0 ──► Dừng
          │
          ▼
[Bước 16] Lặp lại từ Bước 4 cho pre-order tiếp theo
```

### 4.2 Ví dụ minh họa

```
┌─────────────────────────────────────────────────────────────────┐
│                    VÍ DỤ FULFILL                                │
└─────────────────────────────────────────────────────────────────┘

Tình huống:
- Có 3 pre-orders pending cho sản phẩm Gmail:
  1. PRE-001: User A đặt 10 items (đặt lúc 10:00)
  2. PRE-002: User B đặt 5 items (đặt lúc 11:00)
  3. PRE-003: User C đặt 20 items (đặt lúc 12:00)

- Vendor upload 15 items mới

Kết quả (FIFO):
─────────────────────────────────────────────────────────────────
Stock ban đầu: 15 items

1. PRE-001 (User A, 10 items):
   - Đủ hàng (15 >= 10) ✓
   - Fulfill → User A nhận 10 items
   - Stock còn: 15 - 10 = 5 items

2. PRE-002 (User B, 5 items):
   - Đủ hàng (5 >= 5) ✓
   - Fulfill → User B nhận 5 items
   - Stock còn: 5 - 5 = 0 items

3. PRE-003 (User C, 20 items):
   - Không đủ hàng (0 < 20) ✗
   - Vẫn pending, chờ restock tiếp
─────────────────────────────────────────────────────────────────
```

---

## 5. Auto-expire Pre-order

### Flow xử lý hết hạn

```
┌─────────────────────────────────────────────────────────────────┐
│              FLOW AUTO-EXPIRE PRE-ORDER                         │
└─────────────────────────────────────────────────────────────────┘

Cron Job: Chạy mỗi 15 phút

[Bước 1] Query pre-orders hết hạn:
         
         SELECT * FROM preorders
         WHERE status = 'pending'
           AND expires_at <= NOW()
         │
         ▼
[Bước 2] Với mỗi pre-order hết hạn:
         │
         ▼
[Bước 3] Begin Transaction
         │
         ▼
[Bước 4] Cập nhật status:
         UPDATE preorders SET
           status = 'expired',
           expired_at = NOW()
         │
         ▼
[Bước 5] Hoàn tiền vào ví buyer:
         UPDATE wallets SET balance = balance + {amount}
         │
         ▼
[Bước 6] Tạo transaction:
         type = preorder_expired_refund
         │
         ▼
[Bước 7] Commit Transaction
         │
         ▼
[Bước 8] Gửi notifications:
         - Email: "Đơn đặt trước đã hết hạn, tiền đã hoàn"
         - Push notification
         │
         ▼
[Bước 9] Log và tiếp tục với pre-order tiếp theo
```

---

## 6. Quản lý Pre-order (Vendor)

### 6.1 Dashboard Pre-order

```
┌─────────────────────────────────────────────────────────────────┐
│              QUẢN LÝ PRE-ORDER (VENDOR)                         │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Vendor vào Orders > Pre-orders
         │
         ▼
[Bước 2] Hiển thị thống kê:

         ╔═══════════════════════════════════════════════════════╗
         ║  PRE-ORDERS                                           ║
         ╠═══════════════════════════════════════════════════════╣
         ║  Đang chờ: 15 đơn (3,500,000đ)                        ║
         ║  Sắp hết hạn (24h): 3 đơn                             ║
         ║  Đã fulfill tháng này: 45 đơn                         ║
         ╚═══════════════════════════════════════════════════════╝
         │
         ▼
[Bước 3] Danh sách chi tiết theo sản phẩm:

┌─────────────────────────────────────────────────────────────────┐
│ Sản phẩm          │ Pre-orders │ Tổng SL  │ Tổng tiền │ Action │
├───────────────────┼────────────┼──────────┼───────────┼────────┤
│ Gmail US Aged     │ 8 đơn      │ 150 items│ 2,250,000 │[Upload]│
│ Facebook Account  │ 5 đơn      │ 50 items │ 1,000,000 │[Upload]│
│ Github Pro        │ 2 đơn      │ 10 items │   500,000 │[Upload]│
└─────────────────────────────────────────────────────────────────┘
```

### 6.2 Vendor notification

```
Vendor nhận thông báo khi:
1. Có pre-order mới
2. Pre-order sắp hết hạn (24h trước)
3. Pre-order đã được fulfill tự động
4. Pre-order bị buyer hủy

Thông báo giúp vendor:
- Biết nhu cầu của buyer
- Ưu tiên restock sản phẩm có nhiều pre-order
- Không bỏ lỡ doanh thu
```

---

## 7. Chính sách và Giới hạn

### 7.1 Giới hạn Pre-order

```
┌─────────────────────────────────────────────────────────────────┐
│                 GIỚI HẠN PRE-ORDER                              │
└─────────────────────────────────────────────────────────────────┘

Per User:
├── Max pending pre-orders: 5 đơn cùng lúc
├── Max quantity per pre-order: Theo product max_purchase
└── Max total value pending: 5,000,000đ

Per Product:
├── Max pending pre-orders: 100 đơn
└── Max total quantity pending: 1,000 items

Thời gian chờ:
├── Minimum: 1 ngày
├── Maximum: 7 ngày
└── Default: 3 ngày
```

### 7.2 Chính sách hoàn tiền

```
┌─────────────────────────────────────────────────────────────────┐
│              CHÍNH SÁCH HOÀN TIỀN PRE-ORDER                     │
└─────────────────────────────────────────────────────────────────┘

1. Buyer tự hủy:
   └── Hoàn 100% ngay lập tức

2. Hết hạn chờ:
   └── Hoàn 100% tự động

3. Vendor ngưng bán sản phẩm:
   └── Hoàn 100% cho tất cả pre-orders của sản phẩm đó

4. Vendor bị suspend:
   └── Admin xử lý case-by-case
   └── Thường hoàn 100%

5. Sau khi fulfill (đã giao hàng):
   └── Áp dụng chính sách dispute như đơn hàng thường
```

---

## Database Schema

### Bảng preorders

```sql
CREATE TABLE preorders (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    preorder_number VARCHAR(30) UNIQUE,        -- PRE-YYYYMMDD-XXXXX
    
    buyer_id BIGINT NOT NULL,
    shop_id BIGINT NOT NULL,
    product_id BIGINT NOT NULL,
    
    quantity INT NOT NULL,
    unit_price DECIMAL(12,0) NOT NULL,
    total_amount DECIMAL(12,0) NOT NULL,
    
    status ENUM('pending', 'fulfilled', 'cancelled', 'expired', 'refunded'),
    
    -- Timing
    wait_days INT NOT NULL,                    -- 1, 3, hoặc 7
    expires_at TIMESTAMP NOT NULL,
    
    -- Fulfillment
    order_id BIGINT NULL,                      -- Link đến order sau khi fulfill
    fulfilled_at TIMESTAMP NULL,
    
    -- Cancellation
    cancelled_at TIMESTAMP NULL,
    cancelled_by ENUM('buyer', 'vendor', 'system', 'admin') NULL,
    cancel_reason TEXT NULL,
    
    -- Expiry
    expired_at TIMESTAMP NULL,
    
    -- Refund
    refunded_at TIMESTAMP NULL,
    refund_transaction_id BIGINT NULL,
    
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    FOREIGN KEY (buyer_id) REFERENCES users(id),
    FOREIGN KEY (shop_id) REFERENCES shops(id),
    FOREIGN KEY (product_id) REFERENCES products(id),
    FOREIGN KEY (order_id) REFERENCES orders(id),
    
    INDEX idx_product_status (product_id, status),
    INDEX idx_expires (status, expires_at),
    INDEX idx_buyer (buyer_id)
);
```

### Queries thường dùng

```sql
-- Pre-orders pending cho 1 product (để fulfill)
SELECT * FROM preorders
WHERE product_id = ? AND status = 'pending' AND expires_at > NOW()
ORDER BY created_at ASC;

-- Pre-orders sắp hết hạn (để notify vendor)
SELECT * FROM preorders
WHERE shop_id = ? 
  AND status = 'pending'
  AND expires_at BETWEEN NOW() AND DATE_ADD(NOW(), INTERVAL 24 HOUR);

-- Tổng pre-orders pending của user
SELECT COUNT(*), SUM(total_amount) FROM preorders
WHERE buyer_id = ? AND status = 'pending';

-- Pre-orders hết hạn cần xử lý
SELECT * FROM preorders
WHERE status = 'pending' AND expires_at <= NOW();
```
