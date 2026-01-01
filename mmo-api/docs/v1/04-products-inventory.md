# Chức năng Sản phẩm và Kho hàng (Products & Inventory)

## Tổng quan

TaphoaMMO là sàn thương mại điện tử chuyên về **sản phẩm số** (digital products). Sản phẩm được lưu trữ dưới dạng text (account, license, code...) và được giao tự động ngay sau khi thanh toán. Hệ thống quản lý kho hàng được thiết kế đặc biệt để đảm bảo tính unique và không trùng lặp.

---

## 1. Cấu trúc Sản phẩm số

### 1.1 Mô hình dữ liệu

```
┌─────────────────────────────────────────────────────────────────┐
│                    MÔ HÌNH SẢN PHẨM SỐ                          │
└─────────────────────────────────────────────────────────────────┘

Product (Sản phẩm)
│
├── Thông tin hiển thị:
│   ├── Tên sản phẩm
│   ├── Mô tả
│   ├── Hình ảnh
│   ├── Giá bán
│   └── Danh mục
│
├── Cài đặt:
│   ├── Số lượng tối thiểu/tối đa mua
│   ├── Cho phép pre-order
│   ├── Cho phép resell
│   └── Giảm giá số lượng
│
└── Kho hàng (ProductItems):
    ├── Item 1: "email@gmail.com|password123|2FA_SECRET"
    ├── Item 2: "email2@gmail.com|password456|2FA_SECRET"
    ├── Item 3: "email3@gmail.com|password789|2FA_SECRET"
    └── ...

Mỗi ProductItem là một đơn vị sản phẩm có thể bán độc lập.
Khi buyer mua, system sẽ xuất và đánh dấu item đã bán.
```

### 1.2 Format kho hàng phổ biến

| Loại sản phẩm | Format | Ví dụ |
|---------------|--------|-------|
| Account đơn giản | email\|password | abc@gmail.com\|pass123 |
| Account + 2FA | email\|password\|2fa_secret | abc@gmail.com\|pass123\|JBSWY3DPEHPK3PXP |
| Account + backup | email\|password\|backup_email | abc@gmail.com\|pass123\|backup@gmail.com |
| License key | key | XXXX-XXXX-XXXX-XXXX |
| Code/Voucher | code | DISCOUNT50OFF |
| Multi-field | field1\|field2\|field3\|... | Tùy biến |

---

## 2. Tạo Sản phẩm mới

### Flow tạo sản phẩm

```
┌─────────────────────────────────────────────────────────────────┐
│                   FLOW TẠO SẢN PHẨM MỚI                         │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Vendor vào Products > Tạo sản phẩm
         │
         ▼
[Bước 2] Hiển thị form tạo sản phẩm:

         ╔═══════════════════════════════════════════════════════╗
         ║  THÔNG TIN CƠ BẢN                                     ║
         ╠═══════════════════════════════════════════════════════╣
         ║  Tên sản phẩm *: [_______________________________]    ║
         ║  Danh mục *:     [▼ Chọn danh mục________________]    ║
         ║  Mô tả *:        [_______________________________]    ║
         ║                  [_______________________________]    ║
         ║  Hình ảnh:       [📷 Upload] [Preview]               ║
         ╚═══════════════════════════════════════════════════════╝

         ╔═══════════════════════════════════════════════════════╗
         ║  GIÁ VÀ SỐ LƯỢNG                                      ║
         ╠═══════════════════════════════════════════════════════╣
         ║  Giá bán *:      [________] VNĐ                       ║
         ║  Giá gốc:        [________] VNĐ (hiển thị gạch ngang) ║
         ║  Min mua:        [__1_____]                           ║
         ║  Max mua:        [__100___] (0 = không giới hạn)      ║
         ╚═══════════════════════════════════════════════════════╝

         ╔═══════════════════════════════════════════════════════╗
         ║  CÀI ĐẶT NÂNG CAO                                     ║
         ╠═══════════════════════════════════════════════════════╣
         ║  □ Cho phép Pre-order khi hết hàng                    ║
         ║  □ Cho phép Reseller bán lại                          ║
         ║    └── Giá cho Reseller: [________] VNĐ               ║
         ║  □ Ẩn số lượng tồn kho                                ║
         ║  □ Yêu cầu xác nhận 2FA khi mua                       ║
         ╚═══════════════════════════════════════════════════════╝
         │
         ▼
[Bước 3] Vendor điền thông tin và submit
         │
         ▼
[Bước 4] Validate dữ liệu:
         │
         ├── Tên trống ──► "Vui lòng nhập tên sản phẩm"
         ├── Giá <= 0 ──► "Giá phải lớn hơn 0"
         ├── Min > Max ──► "Min không được lớn hơn Max"
         ├── Hình ảnh > 5MB ──► "Hình ảnh quá lớn"
         │
         ▼
[Bước 5] Xử lý hình ảnh (nếu có):
         - Validate format (jpg, png, gif, webp)
         - Resize: Max 800x800px
         - Optimize quality
         - Save vào storage
         │
         ▼
[Bước 6] Tạo bản ghi Product:
         - shop_id: Shop của vendor
         - status: draft (chưa có kho) hoặc active (nếu có kho)
         - stock: 0
         │
         ▼
[Bước 7] Redirect đến trang Upload kho hàng
         │
         ▼
[Bước 8] Hiển thị thông báo "Tạo sản phẩm thành công, hãy upload kho hàng"
```

### Dữ liệu sản phẩm

| Trường | Type | Required | Validation |
|--------|------|----------|------------|
| name | varchar(200) | Yes | 5-200 ký tự |
| slug | varchar(220) | Auto | Unique per shop |
| category_id | bigint | Yes | Phải tồn tại |
| description | text | Yes | 50-5000 ký tự |
| short_description | varchar(500) | No | Max 500 |
| image | varchar(255) | No | jpg/png/gif/webp, max 5MB |
| price | decimal(12,0) | Yes | > 0 |
| original_price | decimal(12,0) | No | >= price hoặc null |
| min_purchase | int | Yes | >= 1 |
| max_purchase | int | Yes | 0 = unlimited |
| allow_preorder | boolean | No | Default false |
| allow_resell | boolean | No | Default false |
| reseller_price | decimal(12,0) | No | < price |
| hide_stock | boolean | No | Default false |
| require_2fa | boolean | No | Default false |
| status | enum | Yes | draft/active/hidden/deleted |

---

## 3. Upload Kho hàng

### 3.1 Flow upload kho hàng

```
┌─────────────────────────────────────────────────────────────────┐
│                    FLOW UPLOAD KHO HÀNG                         │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Vendor vào Product > Inventory / Kho hàng
         │
         ▼
[Bước 2] Hiển thị thông tin hiện tại:
         - Tổng đã upload: 1,500
         - Đã bán: 1,234
         - Còn lại: 266
         │
         ▼
[Bước 3] Các option upload:
         
         Option A - Paste text:
         ╔═══════════════════════════════════════════════════════╗
         ║  Paste nội dung kho (mỗi dòng 1 item):               ║
         ║  ┌───────────────────────────────────────────────────┐║
         ║  │ email1@gmail.com|password1|2FA_SECRET1           │║
         ║  │ email2@gmail.com|password2|2FA_SECRET2           │║
         ║  │ email3@gmail.com|password3|2FA_SECRET3           │║
         ║  │ ...                                               │║
         ║  └───────────────────────────────────────────────────┘║
         ╚═══════════════════════════════════════════════════════╝
         
         Option B - Upload file:
         ╔═══════════════════════════════════════════════════════╗
         ║  [📁 Chọn file TXT]                                   ║
         ║  Format: .txt, mỗi dòng 1 item                        ║
         ║  Max size: 10MB                                       ║
         ╚═══════════════════════════════════════════════════════╝
         │
         ▼
[Bước 4] Vendor nhập/chọn file và click "Upload"
         │
         ▼
[Bước 5] Parse content:
         - Split by newline
         - Trim whitespace
         - Remove empty lines
         - Count total items
         │
         ▼
[Bước 6] Validate format (optional):
         - Kiểm tra có đủ fields theo config
         - VD: email|pass phải có đúng 2 parts
         │
         ├── Format sai ──► Warning nhưng vẫn cho upload
         │
         ▼
[Bước 7] *** KIỂM TRA TRÙNG LẶP ***
         │
         ├── Check trong product này ──► [Chi tiết bên dưới]
         ├── Check trong shop này ──► [Chi tiết bên dưới]
         ├── Check toàn platform ──► [Chi tiết bên dưới]
         │
         ▼
[Bước 8] Hiển thị kết quả kiểm tra:
         
         ╔═══════════════════════════════════════════════════════╗
         ║  KẾT QUẢ KIỂM TRA                                     ║
         ╠═══════════════════════════════════════════════════════╣
         ║  Tổng items upload: 500                               ║
         ║  ✅ Items hợp lệ: 485                                 ║
         ║  ⚠️ Trùng trong sản phẩm: 5                           ║
         ║  ⚠️ Trùng trong shop: 3                               ║
         ║  🚫 Trùng trên sàn (bị loại): 7                       ║
         ║                                                       ║
         ║  [Xem chi tiết items trùng]                           ║
         ║                                                       ║
         ║  [Hủy bỏ]  [Upload 485 items hợp lệ]                  ║
         ╚═══════════════════════════════════════════════════════╝
         │
         ▼
[Bước 9] Vendor xác nhận upload
         │
         ▼
[Bước 10] Insert ProductItems vào database:
          - content: nội dung item (có thể encrypt)
          - content_hash: hash để check trùng nhanh
          - is_sold: false
          - created_at: now
          │
          ▼
[Bước 11] Cập nhật Product:
          - stock += số items upload
          - status = active (nếu đang draft)
          │
          ▼
[Bước 12] Kiểm tra Pre-orders:
          - Có pre-order đang chờ?
          - Nếu có ──► [Flow Auto-fulfill Pre-order]
          │
          ▼
[Bước 13] Hiển thị thông báo thành công
```

### 3.2 Logic kiểm tra trùng lặp

```
┌─────────────────────────────────────────────────────────────────┐
│              LOGIC KIỂM TRA TRÙNG LẶP                           │
└─────────────────────────────────────────────────────────────────┘

Mỗi item được hash để check nhanh:
content_hash = SHA256(normalize(content))

normalize(content):
1. Lowercase toàn bộ
2. Trim whitespace đầu cuối
3. Loại bỏ ký tự đặc biệt thừa
4. Chuẩn hóa delimiter (| thành |)

─────────────────────────────────────────────────────────────────

CHECK LEVEL 1: Trong product hiện tại
├── Query: SELECT content_hash FROM product_items 
│          WHERE product_id = X AND is_sold = false
├── Kết quả: Warning, cho upload (có thể vendor cố tình)

CHECK LEVEL 2: Trong shop hiện tại (các product khác)
├── Query: SELECT content_hash FROM product_items pi
│          JOIN products p ON pi.product_id = p.id
│          WHERE p.shop_id = X AND p.id != Y
├── Kết quả: Warning, cho upload

CHECK LEVEL 3: Toàn platform (đã bán)
├── Query: SELECT content_hash FROM product_items
│          WHERE is_sold = true
├── Kết quả: BLOCK - Không cho upload items này
├── Lý do: Item đã bán cho buyer khác, không được bán lại

CHECK LEVEL 4: Toàn platform (chưa bán, shop khác)
├── Query: SELECT content_hash FROM product_items pi
│          JOIN products p ON pi.product_id = p.id
│          WHERE p.shop_id != X AND is_sold = false
├── Kết quả: BLOCK - Đã có shop khác đang bán

─────────────────────────────────────────────────────────────────

CẢNH BÁO GIAN LẬN:
Nếu phát hiện vendor cố upload items đã bán:
├── Log warning
├── Nếu lặp lại >= 3 lần ──► Cảnh báo admin
├── Nếu lặp lại >= 10 lần ──► Auto suspend shop
```

---

## 4. Quản lý Sản phẩm

### 4.1 Danh sách sản phẩm

```
┌─────────────────────────────────────────────────────────────────┐
│                 DANH SÁCH SẢN PHẨM (VENDOR)                     │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ [+ Tạo mới]  Search: [___________]  Filter: [▼ Trạng thái]     │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────┬────────────────────┬────────┬───────┬───────┬──────┐ │
│  │ Hình │ Tên sản phẩm       │ Giá    │ Kho   │ Bán   │ TT   │ │
│  ├──────┼────────────────────┼────────┼───────┼───────┼──────┤ │
│  │ [📷] │ Gmail US Aged      │ 15,000 │ 150   │ 1,234 │ 🟢   │ │
│  │ [📷] │ Facebook Account   │ 25,000 │ 0     │ 890   │ 🔴   │ │
│  │ [📷] │ Github Pro         │ 50,000 │ 45    │ 567   │ 🟢   │ │
│  │ [📷] │ Discord Nitro      │ 35,000 │ 12    │ 234   │ 🟡   │ │
│  └──────┴────────────────────┴────────┴───────┴───────┴──────┘ │
│                                                                 │
│  🟢 Active   🟡 Sắp hết (<20)   🔴 Hết hàng   ⚫ Ẩn            │
│                                                                 │
│  [Pagination: < 1 2 3 4 5 ... 10 >]                            │
└─────────────────────────────────────────────────────────────────┘
```

### 4.2 Chỉnh sửa sản phẩm

```
┌─────────────────────────────────────────────────────────────────┐
│                 FLOW CHỈNH SỬA SẢN PHẨM                         │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Vendor click Edit trên sản phẩm
         │
         ▼
[Bước 2] Load thông tin hiện tại
         │
         ▼
[Bước 3] Hiển thị form edit (tương tự form tạo)
         
         Các trường có thể edit:
         ✅ Tên sản phẩm
         ✅ Mô tả
         ✅ Hình ảnh
         ✅ Giá bán (có cảnh báo nếu đang có pre-order)
         ✅ Min/Max mua
         ✅ Cài đặt pre-order
         ✅ Cài đặt reseller
         ✅ Trạng thái (active/hidden)
         
         Không thể edit:
         ❌ Danh mục (phải tạo sản phẩm mới)
         ❌ Kho hàng (quản lý riêng)
         │
         ▼
[Bước 4] Vendor edit và submit
         │
         ▼
[Bước 5] Validate changes
         │
         ▼
[Bước 6] Nếu thay đổi giá:
         │
         ├── Có pre-order pending?
         │   ├── Yes ──► Hiển thị warning
         │   │          "Có X pre-order với giá cũ. Tiếp tục?"
         │   │          - Giữ nguyên giá cũ cho pre-order
         │   │          - Hoặc hủy pre-order và hoàn tiền
         │   │
         │   ▼
         │
         ▼
[Bước 7] Update database
         │
         ▼
[Bước 8] Clear cache product
         │
         ▼
[Bước 9] Hiển thị thông báo thành công
```

### 4.3 Ẩn/Xóa sản phẩm

```
┌─────────────────────────────────────────────────────────────────┐
│                    FLOW ẨN SẢN PHẨM                             │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Vendor click "Ẩn" trên sản phẩm
         │
         ▼
[Bước 2] Hiển thị confirm:
         "Ẩn sản phẩm này? Sản phẩm sẽ không hiển thị cho buyer."
         │
         ▼
[Bước 3] Vendor xác nhận
         │
         ▼
[Bước 4] Update status = hidden
         │
         ▼
[Bước 5] Xử lý pre-order (nếu có):
         - Hủy tất cả pre-order pending
         - Hoàn tiền cho buyer
         │
         ▼
[Bước 6] Sản phẩm vẫn hiển thị trong Vendor dashboard
         Có thể "Hiện lại" bất cứ lúc nào

─────────────────────────────────────────────────────────────────

┌─────────────────────────────────────────────────────────────────┐
│                    FLOW XÓA SẢN PHẨM                            │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Vendor click "Xóa" trên sản phẩm
         │
         ▼
[Bước 2] Kiểm tra điều kiện xóa:
         │
         ├── Còn kho chưa bán?
         │   └── Warning: "Còn X items chưa bán sẽ bị xóa"
         │
         ├── Có pre-order pending?
         │   └── Block: "Không thể xóa, hãy hủy pre-order trước"
         │
         ├── Có dispute chưa giải quyết?
         │   └── Block: "Không thể xóa, có khiếu nại pending"
         │
         ▼
[Bước 3] Hiển thị confirm với password:
         "Xác nhận xóa vĩnh viễn sản phẩm này?"
         │
         ▼
[Bước 4] Soft delete:
         - status = deleted
         - deleted_at = now
         │
         ▼
[Bước 5] Giữ lại data cho:
         - Lịch sử đơn hàng
         - Báo cáo thống kê
         │
         ▼
[Bước 6] Sau 30 ngày: Hard delete (optional, by admin)
```

---

## 5. Xem kho hàng chi tiết

### Flow xem và quản lý items

```
┌─────────────────────────────────────────────────────────────────┐
│                  FLOW XEM KHO HÀNG CHI TIẾT                     │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Vendor vào Product > Xem kho hàng
         │
         ▼
[Bước 2] Hiển thị thống kê:
         
         ╔═══════════════════════════════════════════════════════╗
         ║  THỐNG KÊ KHO HÀNG                                    ║
         ╠═══════════════════════════════════════════════════════╣
         ║  Tổng đã upload:     1,500 items                      ║
         ║  Đã bán:             1,234 items (82.3%)              ║
         ║  Còn lại:            266 items                        ║
         ║  Đang hold (pre-order): 10 items                      ║
         ║  Khả dụng:           256 items                        ║
         ╚═══════════════════════════════════════════════════════╝
         │
         ▼
[Bước 3] Tabs hiển thị:
         
         [Chưa bán (266)] [Đã bán (1,234)] [Đang hold (10)]
         │
         ▼
[Bước 4] Tab "Chưa bán":
         
         ┌─────────────────────────────────────────────────────┐
         │ □ | Nội dung                    | Ngày upload       │
         ├─────────────────────────────────────────────────────┤
         │ □ | abc@gmail.com|pass***       | 2024-01-15 10:30  │
         │ □ | def@gmail.com|pass***       | 2024-01-15 10:30  │
         │ □ | ghi@gmail.com|pass***       | 2024-01-14 09:20  │
         └─────────────────────────────────────────────────────┘
         
         [Xóa đã chọn]  [Export]
         │
         ▼
[Bước 5] Tab "Đã bán":
         
         ┌─────────────────────────────────────────────────────┐
         │ Nội dung           | Đơn hàng | Buyer   | Ngày bán │
         ├─────────────────────────────────────────────────────┤
         │ xyz@gmail.com|***  | #12345   | user123 | 15/01/24 │
         │ uvw@gmail.com|***  | #12340   | user456 | 14/01/24 │
         └─────────────────────────────────────────────────────┘
         
         Lưu ý: Content bị mask một phần vì đã bán
```

### Xóa items khỏi kho

```
┌─────────────────────────────────────────────────────────────────┐
│                   FLOW XÓA ITEMS KHỎ KHO                        │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Vendor chọn items cần xóa (checkbox)
         │
         ▼
[Bước 2] Click "Xóa đã chọn"
         │
         ▼
[Bước 3] Kiểm tra items:
         │
         ├── Item đang hold (pre-order) ──► Không cho xóa
         ├── Item đã bán ──► Không hiển thị checkbox
         │
         ▼
[Bước 4] Confirm: "Xóa X items đã chọn?"
         │
         ▼
[Bước 5] Hard delete từ database
         │
         ▼
[Bước 6] Update product stock
         │
         ▼
[Bước 7] Log action để audit
```

---

## 6. Giảm giá theo số lượng

### Cấu hình bulk discount

```
┌─────────────────────────────────────────────────────────────────┐
│              FLOW CÀI ĐẶT GIẢM GIÁ SỐ LƯỢNG                     │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Vendor vào Product > Pricing > Giảm giá số lượng
         │
         ▼
[Bước 2] Hiển thị form cài đặt:
         
         Giá gốc: 15,000 VNĐ/item
         
         ╔═══════════════════════════════════════════════════════╗
         ║  BẢNG GIÁ THEO SỐ LƯỢNG                               ║
         ╠═══════════════════════════════════════════════════════╣
         ║  Từ [10] đến [49] items:  Giảm [5]%  = 14,250đ/item   ║
         ║  Từ [50] đến [99] items:  Giảm [10]% = 13,500đ/item   ║
         ║  Từ [100] trở lên:        Giảm [15]% = 12,750đ/item   ║
         ║                                                       ║
         ║  [+ Thêm mức giá]                                      ║
         ╚═══════════════════════════════════════════════════════╝
         │
         ▼
[Bước 3] Vendor cấu hình và save
         │
         ▼
[Bước 4] Validate:
         - Các mức không overlap
         - % giảm hợp lý (không âm, không > 100%)
         │
         ▼
[Bước 5] Lưu vào bảng product_discounts
         │
         ▼
[Bước 6] Khi buyer mua, áp dụng discount tương ứng
```

### Logic tính giá

```
┌─────────────────────────────────────────────────────────────────┐
│                  LOGIC TÍNH GIÁ KHI MUA                         │
└─────────────────────────────────────────────────────────────────┘

Input: product_id, quantity

[Step 1] Lấy giá gốc: base_price = product.price

[Step 2] Tìm discount tier phù hợp:
         SELECT discount_percent 
         FROM product_discounts
         WHERE product_id = X
           AND min_quantity <= quantity
           AND (max_quantity >= quantity OR max_quantity IS NULL)
         ORDER BY min_quantity DESC
         LIMIT 1

[Step 3] Tính giá sau discount:
         unit_price = base_price * (1 - discount_percent/100)

[Step 4] Tính tổng:
         subtotal = unit_price * quantity

[Step 5] Áp dụng coupon (nếu có):
         final_total = apply_coupon(subtotal, coupon)

Output: unit_price, subtotal, discount_amount, final_total
```

---

## 7. Hiển thị Sản phẩm (Buyer View)

### Flow xem chi tiết sản phẩm

```
┌─────────────────────────────────────────────────────────────────┐
│              FLOW XEM CHI TIẾT SẢN PHẨM (BUYER)                 │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Buyer truy cập /gian-hang/{shop_slug}_{shop_id}
         hoặc click vào sản phẩm từ danh sách
         │
         ▼
[Bước 2] Query product với điều kiện:
         - product.id = X
         - product.status = active
         - product.shop.status = active
         │
         ├── Không tìm thấy ──► 404 Page
         │
         ▼
[Bước 3] Load related data:
         - Shop info
         - Category info
         - Stock count
         - Reviews
         - Discount tiers
         │
         ▼
[Bước 4] Render trang sản phẩm:

┌─────────────────────────────────────────────────────────────────┐
│  [Breadcrumb: Home > Danh mục > Sản phẩm]                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐  Gmail US Aged 2020                           │
│  │             │  ────────────────────                          │
│  │   [IMAGE]   │  Shop: TechAccount ⭐ 4.8                      │
│  │             │                                                │
│  └─────────────┘  💰 15,000 VNĐ  ̶2̶0̶,̶0̶0̶0̶ ̶V̶N̶Đ̶               │
│                                                                 │
│                   📦 Còn lại: 150 items                         │
│                                                                 │
│                   Số lượng: [- ] 1 [ +]                         │
│                                                                 │
│                   Tổng: 15,000 VNĐ                              │
│                                                                 │
│                   [🛒 Mua ngay]                                 │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│  [Tab: Mô tả] [Tab: Bảng giá] [Tab: Đánh giá (234)]            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Mô tả chi tiết sản phẩm...                                    │
│  - Gmail US tạo năm 2020                                       │
│  - Đã verify phone                                              │
│  - Format: email|password|2FA                                   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Realtime stock check

```
┌─────────────────────────────────────────────────────────────────┐
│               FLOW KIỂM TRA TỒN KHO REALTIME                    │
└─────────────────────────────────────────────────────────────────┘

Khi buyer ở trang sản phẩm:

[Mỗi 30 giây]
     │
     ▼
[AJAX call] GET /api/products/{id}/stock
     │
     ▼
[Server] Query current stock:
         SELECT COUNT(*) FROM product_items
         WHERE product_id = X AND is_sold = false AND hold_until IS NULL
     │
     ▼
[Response] { "stock": 150, "status": "available" }
     │
     ▼
[Client] Update UI:
         - Cập nhật số lượng hiển thị
         - Nếu stock = 0 và allow_preorder:
           └── Hiển thị nút "Đặt trước"
         - Nếu stock = 0 và không allow_preorder:
           └── Hiển thị "Hết hàng"
         - Nếu stock < quantity đã chọn:
           └── Alert và giảm quantity
```

---

## 8. Danh mục sản phẩm

### Cấu trúc danh mục

```
┌─────────────────────────────────────────────────────────────────┐
│                   CẤU TRÚC DANH MỤC                             │
└─────────────────────────────────────────────────────────────────┘

Categories (quản lý bởi Admin):

├── Email
│   ├── Gmail
│   ├── Outlook
│   ├── Yahoo
│   └── Email khác
│
├── Mạng xã hội
│   ├── Facebook
│   ├── Instagram
│   ├── Twitter/X
│   ├── TikTok
│   └── MXH khác
│
├── Developer
│   ├── Github
│   ├── Gitlab
│   ├── Bitbucket
│   └── Cloud Services
│
├── Gaming
│   ├── Steam
│   ├── Epic Games
│   ├── Origin
│   └── Game khác
│
├── Streaming
│   ├── Netflix
│   ├── Spotify
│   ├── Youtube Premium
│   └── Streaming khác
│
└── Khác
    ├── VPN
    ├── License
    └── Misc
```

### Database Schema

```sql
-- Bảng categories
CREATE TABLE categories (
    id BIGINT PRIMARY KEY,
    parent_id BIGINT NULL,          -- Cho nested categories
    name VARCHAR(100),
    slug VARCHAR(120) UNIQUE,
    description TEXT,
    icon VARCHAR(50),               -- Icon class hoặc emoji
    sort_order INT DEFAULT 0,
    status ENUM('active', 'hidden'),
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    FOREIGN KEY (parent_id) REFERENCES categories(id)
);

-- Bảng products
CREATE TABLE products (
    id BIGINT PRIMARY KEY,
    shop_id BIGINT NOT NULL,
    category_id BIGINT NOT NULL,
    name VARCHAR(200),
    slug VARCHAR(220),
    description TEXT,
    short_description VARCHAR(500),
    image VARCHAR(255),
    price DECIMAL(12,0),
    original_price DECIMAL(12,0),
    min_purchase INT DEFAULT 1,
    max_purchase INT DEFAULT 0,
    stock INT DEFAULT 0,
    total_sold INT DEFAULT 0,
    allow_preorder BOOLEAN DEFAULT FALSE,
    allow_resell BOOLEAN DEFAULT FALSE,
    reseller_price DECIMAL(12,0),
    hide_stock BOOLEAN DEFAULT FALSE,
    require_2fa BOOLEAN DEFAULT FALSE,
    status ENUM('draft', 'active', 'hidden', 'deleted'),
    deleted_at TIMESTAMP NULL,
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    FOREIGN KEY (shop_id) REFERENCES shops(id),
    FOREIGN KEY (category_id) REFERENCES categories(id),
    UNIQUE KEY (shop_id, slug)
);

-- Bảng product_items (kho hàng)
CREATE TABLE product_items (
    id BIGINT PRIMARY KEY,
    product_id BIGINT NOT NULL,
    content TEXT NOT NULL,           -- Nội dung item (có thể encrypt)
    content_hash VARCHAR(64) NOT NULL, -- SHA256 để check trùng
    is_sold BOOLEAN DEFAULT FALSE,
    sold_at TIMESTAMP NULL,
    order_id BIGINT NULL,
    hold_until TIMESTAMP NULL,       -- Cho pre-order
    preorder_id BIGINT NULL,
    created_at TIMESTAMP,
    FOREIGN KEY (product_id) REFERENCES products(id),
    FOREIGN KEY (order_id) REFERENCES orders(id),
    INDEX idx_content_hash (content_hash),
    INDEX idx_product_sold (product_id, is_sold)
);

-- Bảng product_discounts (giảm giá số lượng)
CREATE TABLE product_discounts (
    id BIGINT PRIMARY KEY,
    product_id BIGINT NOT NULL,
    min_quantity INT NOT NULL,
    max_quantity INT NULL,           -- NULL = unlimited
    discount_percent DECIMAL(5,2),
    created_at TIMESTAMP,
    FOREIGN KEY (product_id) REFERENCES products(id)
);
```
