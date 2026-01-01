# Chức năng Gian hàng (Shop/Vendor Management)

## Tổng quan

Hệ thống gian hàng cho phép Vendor tạo và quản lý cửa hàng trực tuyến của mình trên sàn TaphoaMMO. Mỗi Vendor chỉ có một gian hàng duy nhất và có toàn quyền quản lý sản phẩm, kho hàng, và các cài đặt liên quan.

---

## 1. Tạo và Thiết lập Gian hàng

### 1.1 Tạo gian hàng mới

**Điều kiện tiên quyết:**
- User đã đăng ký và được duyệt làm Vendor
- Chưa có gian hàng

### Flow tạo gian hàng

```
┌─────────────────────────────────────────────────────────────────┐
│                   FLOW TẠO GIAN HÀNG MỚI                        │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Vendor được duyệt, lần đầu vào Vendor Dashboard
         │
         ▼
[Bước 2] Hệ thống kiểm tra đã có shop chưa
         │
         ├── Đã có ──► Redirect đến Shop Dashboard
         │
         ▼
[Bước 3] Hiển thị wizard tạo gian hàng

         Step 1: Thông tin cơ bản
         ├── shop_name: Tên gian hàng (3-50 ký tự)
         ├── shop_slug: URL gian hàng (auto từ name, có thể edit)
         ├── shop_description: Mô tả ngắn (max 500 ký tự)
         │
         ▼
         Step 2: Hình ảnh
         ├── shop_logo: Logo (required, 200x200px recommended)
         ├── shop_banner: Banner (optional, 1200x300px)
         │
         ▼
         Step 3: Thông tin liên hệ
         ├── contact_email: Email liên hệ
         ├── contact_phone: Số điện thoại
         ├── facebook_page: Fanpage Facebook
         ├── telegram_channel: Kênh Telegram
         ├── zalo_oa: Zalo OA
         │
         ▼
         Step 4: Chính sách
         ├── warranty_policy: Chính sách bảo hành
         ├── refund_policy: Chính sách hoàn tiền
         ├── support_hours: Giờ hỗ trợ
         │
         ▼
[Bước 4] Validate từng step
         │
         ├── Lỗi ──► Hiển thị và yêu cầu sửa
         │
         ▼
[Bước 5] Kiểm tra shop_slug unique
         │
         ├── Trùng ──► Suggest slug mới (thêm số)
         │
         ▼
[Bước 6] Upload và xử lý images
         - Resize về kích thước chuẩn
         - Optimize cho web
         - Lưu vào storage
         │
         ▼
[Bước 7] Tạo bản ghi Shop trong database
         - status: active
         - rating: 0 (chưa có đánh giá)
         - total_sales: 0
         - total_products: 0
         │
         ▼
[Bước 8] Liên kết Shop với User (vendor_id)
         │
         ▼
[Bước 9] Tạo thư mục storage riêng cho shop
         │
         ▼
[Bước 10] Gửi email chào mừng với hướng dẫn
          │
          ▼
[Bước 11] Redirect đến Shop Dashboard với tour hướng dẫn
```

### Dữ liệu gian hàng

| Trường | Type | Required | Mô tả |
|--------|------|----------|-------|
| shop_name | varchar(50) | Yes | Tên hiển thị |
| shop_slug | varchar(60) | Yes | URL-friendly, unique |
| description | text | Yes | Mô tả gian hàng |
| logo | varchar(255) | Yes | Path to logo |
| banner | varchar(255) | No | Path to banner |
| contact_email | varchar(255) | No | Email công khai |
| contact_phone | varchar(20) | No | SĐT công khai |
| facebook_page | varchar(255) | No | URL Facebook |
| telegram_channel | varchar(100) | No | @username |
| zalo_oa | varchar(20) | No | Số ZaloOA |
| warranty_policy | text | No | Chính sách BH |
| refund_policy | text | No | Chính sách hoàn |
| support_hours | varchar(100) | No | VD: "8h-22h hàng ngày" |
| status | enum | Yes | active/inactive/suspended |
| rating | decimal(3,2) | Yes | Điểm đánh giá TB |
| total_reviews | int | Yes | Số lượng đánh giá |
| total_sales | int | Yes | Tổng đơn đã bán |
| total_products | int | Yes | Số sản phẩm |
| allow_reseller | boolean | Yes | Cho phép resell |
| commission_rate | decimal(5,2) | No | Phí sàn (%) |

---

## 2. Cập nhật thông tin Gian hàng

### Flow cập nhật

```
┌─────────────────────────────────────────────────────────────────┐
│                FLOW CẬP NHẬT THÔNG TIN SHOP                     │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Vendor vào Shop Settings
         │
         ▼
[Bước 2] Load thông tin hiện tại từ DB
         │
         ▼
[Bước 3] Hiển thị form với các tab:
         
         Tab Thông tin chung:
         ├── Tên gian hàng
         ├── Mô tả
         ├── Logo/Banner
         
         Tab Liên hệ:
         ├── Email, Phone
         ├── Social links
         
         Tab Chính sách:
         ├── Bảo hành
         ├── Hoàn tiền
         ├── Giờ hỗ trợ
         
         Tab Cài đặt nâng cao:
         ├── Cho phép Reseller
         ├── Tỷ lệ chiết khấu cho Reseller
         ├── Thông báo đơn hàng
         │
         ▼
[Bước 4] Vendor chỉnh sửa và submit
         │
         ▼
[Bước 5] Validate dữ liệu
         │
         ├── shop_slug changed ──► Kiểm tra unique
         ├── Lỗi ──► Hiển thị lỗi
         │
         ▼
[Bước 6] Xử lý thay đổi logo/banner
         │
         ├── Có upload mới ──► Xử lý file, xóa file cũ
         │
         ▼
[Bước 7] Update vào database
         │
         ▼
[Bước 8] Clear cache shop
         │
         ▼
[Bước 9] Hiển thị thông báo thành công

Lưu ý: Thay đổi shop_slug sẽ ảnh hưởng đến URL
       - Redirect 301 từ slug cũ sang slug mới
       - Giữ lại slug cũ trong 30 ngày
```

---

## 3. Dashboard Gian hàng

### 3.1 Tổng quan Dashboard

```
┌─────────────────────────────────────────────────────────────────┐
│                    SHOP DASHBOARD                               │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  THỐNG KÊ NHANH                                                 │
├─────────────────┬─────────────────┬─────────────────────────────┤
│  Doanh thu      │  Đơn hàng       │  Sản phẩm                   │
│  hôm nay        │  mới            │  tồn kho                    │
│  ₫1,250,000     │  15 đơn         │  1,234 items                │
├─────────────────┼─────────────────┼─────────────────────────────┤
│  Tổng           │  Đánh giá       │  Khiếu nại                  │
│  số dư          │  trung bình     │  đang chờ                   │
│  ₫5,430,000     │  4.8/5 ⭐       │  2 đơn                      │
└─────────────────┴─────────────────┴─────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  BIỂU ĐỒ DOANH THU 7 NGÀY GẦN NHẤT                             │
│  [=========================================]                    │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  HOẠT ĐỘNG GẦN ĐÂY                                             │
│  • 10:30 - Đơn hàng mới #12345 - 50,000đ                       │
│  • 10:25 - Khiếu nại mới từ user_abc                           │
│  • 09:15 - Sản phẩm "Gmail US" hết hàng                        │
│  • 08:00 - Tiền về ví: 500,000đ                                │
└─────────────────────────────────────────────────────────────────┘

┌──────────────────────────┬──────────────────────────────────────┐
│  SẢN PHẨM BÁN CHẠY       │  CẢNH BÁO                            │
├──────────────────────────┼──────────────────────────────────────┤
│  1. Gmail US - 234 sold  │  ⚠ 3 sản phẩm sắp hết hàng          │
│  2. Facebook - 189 sold  │  ⚠ 2 khiếu nại cần phản hồi         │
│  3. Github - 156 sold    │  ⚠ 5 pre-order đang chờ              │
└──────────────────────────┴──────────────────────────────────────┘
```

### 3.2 Flow hiển thị Dashboard

```
┌─────────────────────────────────────────────────────────────────┐
│                 FLOW LOAD DASHBOARD DATA                        │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Vendor truy cập Dashboard
         │
         ▼
[Bước 2] Kiểm tra cache có data không
         │
         ├── Có cache valid ──► Trả về từ cache
         │
         ▼
[Bước 3] Query các metrics:
         
         Revenue Today:
         ├── SUM(orders.total) WHERE shop_id = X AND DATE(created_at) = today
         
         Orders Today:
         ├── COUNT(orders) WHERE shop_id = X AND DATE(created_at) = today
         
         Stock Count:
         ├── SUM(product_items.quantity) WHERE product.shop_id = X AND is_sold = false
         
         Available Balance:
         ├── vendor_wallet.available_balance (đã qua 3 ngày)
         
         Pending Balance:
         ├── vendor_wallet.pending_balance (chưa qua 3 ngày)
         
         Average Rating:
         ├── shops.rating
         
         Pending Disputes:
         ├── COUNT(disputes) WHERE shop_id = X AND status = pending
         │
         ▼
[Bước 4] Query chart data (7 ngày)
         │
         ▼
[Bước 5] Query recent activities
         ├── Orders gần nhất
         ├── Disputes gần nhất
         ├── Stock alerts
         │
         ▼
[Bước 6] Cache kết quả (TTL: 5 phút)
         │
         ▼
[Bước 7] Render dashboard với data
```

---

## 4. Quản lý Cài đặt Reseller

### 4.1 Bật/Tắt cho phép Resell

```
┌─────────────────────────────────────────────────────────────────┐
│              FLOW CÀI ĐẶT RESELLER CHO SHOP                     │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Vendor vào Settings > Reseller
         │
         ▼
[Bước 2] Hiển thị thông tin:
         - Trạng thái: Đang bật/tắt
         - Số Reseller đang hoạt động
         - Doanh thu từ Reseller
         │
         ▼
[Bước 3] Các tùy chọn:
         
         Toggle "Cho phép Resell":
         ├── ON: Reseller có thể bán sản phẩm của shop
         ├── OFF: Ẩn khỏi danh sách reseller
         
         Tỷ lệ chiết khấu mặc định:
         ├── % lợi nhuận Reseller nhận được
         ├── VD: 10% = Reseller bán 100k, nhận 10k
         
         Cho phép Reseller đặt giá:
         ├── ON: Reseller tự đặt giá bán
         ├── OFF: Giá cố định theo shop
         
         Giá sàn (nếu cho đặt giá):
         ├── Giá tối thiểu Reseller được bán
         │
         ▼
[Bước 4] Vendor thay đổi và save
         │
         ▼
[Bước 5] Cập nhật vào database
         │
         ▼
[Bước 6] Nếu tắt Reseller:
         - Thông báo cho các Reseller đang hoạt động
         - Không ảnh hưởng đơn hàng đang xử lý
         │
         ▼
[Bước 7] Hiển thị thông báo thành công
```

### 4.2 Quản lý Reseller của Shop

```
┌─────────────────────────────────────────────────────────────────┐
│              FLOW XEM DANH SÁCH RESELLER                        │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Vendor vào Resellers
         │
         ▼
[Bước 2] Query resellers đã bán sản phẩm của shop
         │
         ▼
[Bước 3] Hiển thị danh sách:
         
         Cho mỗi Reseller:
         ├── Username
         ├── Tổng đơn đã bán
         ├── Tổng doanh thu cho shop
         ├── Tổng hoa hồng đã trả
         ├── Trạng thái: Active/Blocked
         │
         ▼
[Bước 4] Vendor có thể:
         
         Block Reseller:
         ├── Ngăn reseller bán sản phẩm của shop
         ├── Không ảnh hưởng reseller ở shop khác
         
         Xem chi tiết:
         ├── Lịch sử đơn hàng
         ├── Các sản phẩm đã bán
```

---

## 5. Thống kê và Báo cáo

### 5.1 Báo cáo doanh thu

```
┌─────────────────────────────────────────────────────────────────┐
│                  FLOW XEM BÁO CÁO DOANH THU                     │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Vendor vào Reports > Revenue
         │
         ▼
[Bước 2] Chọn khoảng thời gian:
         - Hôm nay
         - 7 ngày
         - 30 ngày
         - Tháng này
         - Tháng trước
         - Custom range
         │
         ▼
[Bước 3] Query data:
         
         Summary:
         ├── Tổng doanh thu
         ├── Tổng đơn hàng
         ├── Giá trị TB/đơn
         ├── Tổng phí sàn
         ├── Doanh thu ròng
         
         By Product:
         ├── Sản phẩm - Số lượng - Doanh thu
         
         By Day:
         ├── Ngày - Đơn hàng - Doanh thu
         │
         ▼
[Bước 4] Hiển thị:
         - Biểu đồ đường (trend)
         - Bảng chi tiết
         - Pie chart (theo sản phẩm)
         │
         ▼
[Bước 5] Export options:
         - Excel
         - PDF
         - CSV
```

### 5.2 Báo cáo sản phẩm

```
┌─────────────────────────────────────────────────────────────────┐
│                FLOW XEM BÁO CÁO SẢN PHẨM                        │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Vendor vào Reports > Products
         │
         ▼
[Bước 2] Hiển thị overview:
         
         Tổng quan:
         ├── Tổng sản phẩm: 50
         ├── Đang active: 45
         ├── Hết hàng: 3
         ├── Bị ẩn: 2
         
         Cảnh báo:
         ├── Sản phẩm sắp hết (< 10 items): [list]
         ├── Sản phẩm không bán được 7 ngày: [list]
         │
         ▼
[Bước 3] Bảng chi tiết:
         
         | Sản phẩm | Tồn kho | Đã bán | Doanh thu | Tỷ lệ khiếu nại |
         |----------|---------|--------|-----------|-----------------|
         | Gmail US | 150     | 1,234  | 12,340,000| 0.5%           |
         │
         ▼
[Bước 4] Sort và filter:
         - Sort by: Sales, Stock, Revenue, Dispute rate
         - Filter by: Category, Status
```

---

## 6. Cài đặt Thông báo

### Flow cài đặt notification

```
┌─────────────────────────────────────────────────────────────────┐
│                FLOW CÀI ĐẶT THÔNG BÁO                           │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Vendor vào Settings > Notifications
         │
         ▼
[Bước 2] Hiển thị các loại thông báo:
         
         Email Notifications:
         ├── □ Đơn hàng mới
         ├── □ Khiếu nại mới
         ├── □ Đánh giá mới
         ├── □ Sản phẩm hết hàng
         ├── □ Báo cáo hàng ngày
         ├── □ Báo cáo hàng tuần
         
         Telegram Notifications:
         ├── Bot token (nếu có)
         ├── □ Đơn hàng mới
         ├── □ Khiếu nại mới
         
         Browser Notifications:
         ├── □ Push notifications
         │
         ▼
[Bước 3] Vendor check/uncheck và save
         │
         ▼
[Bước 4] Lưu preferences vào database
         │
         ▼
[Bước 5] Test notification (optional)
         - Gửi test email
         - Gửi test Telegram
```

---

## 7. Xem Gian hàng (Public View)

### Flow xem gian hàng từ buyer

```
┌─────────────────────────────────────────────────────────────────┐
│               FLOW XEM GIAN HÀNG (BUYER VIEW)                   │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] User truy cập /gian-hang/{shop_slug}_{shop_id}
         │
         ▼
[Bước 2] Parse slug và ID từ URL
         │
         ▼
[Bước 3] Query shop từ database
         │
         ├── Không tìm thấy ──► 404 Not Found
         ├── Shop inactive/suspended ──► 404 hoặc thông báo
         │
         ▼
[Bước 4] Kiểm tra redirect (nếu slug đã đổi)
         │
         ├── Slug cũ ──► 301 Redirect đến slug mới
         │
         ▼
[Bước 5] Load shop data:
         
         Thông tin shop:
         ├── Logo, banner, tên, mô tả
         ├── Rating, số đánh giá
         ├── Ngày tham gia
         ├── Tổng sản phẩm
         ├── Liên kết liên hệ
         │
         ▼
[Bước 6] Load products của shop:
         - Chỉ products có status = active
         - Chỉ products có stock > 0 (optional)
         - Pagination: 20 items/page
         │
         ▼
[Bước 7] Load reviews gần nhất
         │
         ▼
[Bước 8] Hiển thị trang shop:
         
         ┌─────────────────────────────────────────────────────┐
         │  [Banner]                                           │
         │  [Logo] Tên Shop                    [Contact Btns]  │
         │  ⭐ 4.8 (1,234 đánh giá) | 5,678 đã bán            │
         │  Mô tả ngắn về shop...                              │
         ├─────────────────────────────────────────────────────┤
         │  [Tab: Sản phẩm] [Tab: Đánh giá] [Tab: Thông tin]  │
         ├─────────────────────────────────────────────────────┤
         │  Filter: [Danh mục ▼] [Sắp xếp ▼]                  │
         │                                                     │
         │  [Product] [Product] [Product] [Product]           │
         │  [Product] [Product] [Product] [Product]           │
         │                                                     │
         │  [Pagination]                                       │
         └─────────────────────────────────────────────────────┘
```

---

## 8. Đánh giá và Xếp hạng Shop

### Cách tính điểm đánh giá

```
┌─────────────────────────────────────────────────────────────────┐
│                  LOGIC TÍNH ĐIỂM SHOP                           │
└─────────────────────────────────────────────────────────────────┘

Rating của shop được tính từ:

1. Điểm đánh giá trung bình (70%):
   - Trung bình cộng tất cả reviews
   - Chỉ tính reviews verified (đã mua hàng)

2. Tỷ lệ hoàn thành đơn (20%):
   - (Đơn thành công / Tổng đơn) * 5
   - Đơn thành công = delivered + không có dispute

3. Tỷ lệ phản hồi (10%):
   - Tỷ lệ trả lời khiếu nại trong 24h
   - (Phản hồi đúng hạn / Tổng khiếu nại) * 5

Công thức:
shop_rating = (avg_review * 0.7) + (completion_rate * 0.2) + (response_rate * 0.1)

Cập nhật: Mỗi khi có review mới hoặc dispute kết thúc
```

### Hệ thống badge/cấp độ

```
┌─────────────────────────────────────────────────────────────────┐
│                   CẤP ĐỘ GIAN HÀNG                              │
└─────────────────────────────────────────────────────────────────┘

Level 1 - Mới (New Seller):
├── Điều kiện: Mới tạo shop
├── Badge: 🆕
├── Phí sàn: 10%

Level 2 - Bạc (Silver):
├── Điều kiện: 50+ đơn, rating >= 4.0
├── Badge: 🥈
├── Phí sàn: 8%

Level 3 - Vàng (Gold):
├── Điều kiện: 200+ đơn, rating >= 4.5
├── Badge: 🥇
├── Phí sàn: 6%

Level 4 - Kim cương (Diamond):
├── Điều kiện: 1000+ đơn, rating >= 4.8
├── Badge: 💎
├── Phí sàn: 5%
├── Ưu tiên hiển thị

Level 5 - Đối tác (Partner):
├── Điều kiện: Được admin nâng cấp thủ công
├── Badge: ✅ Verified
├── Phí sàn: Thỏa thuận
├── Hỗ trợ ưu tiên
```

---

## 9. Suspend/Deactivate Shop

### Flow từ Vendor (tự ngưng hoạt động)

```
┌─────────────────────────────────────────────────────────────────┐
│               FLOW NGƯNG HOẠT ĐỘNG SHOP                         │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Vendor vào Settings > Trạng thái shop
         │
         ▼
[Bước 2] Click "Tạm ngưng gian hàng"
         │
         ▼
[Bước 3] Hiển thị cảnh báo:
         - Sản phẩm sẽ bị ẩn
         - Không nhận đơn mới
         - Đơn đang xử lý vẫn tiếp tục
         - Pre-order sẽ bị hủy và hoàn tiền
         │
         ▼
[Bước 4] Yêu cầu xác nhận mật khẩu
         │
         ▼
[Bước 5] Cập nhật shop status: inactive
         │
         ▼
[Bước 6] Cập nhật products status: hidden
         │
         ▼
[Bước 7] Xử lý pre-orders:
         - Cancel tất cả pre-order pending
         - Hoàn tiền cho buyers
         │
         ▼
[Bước 8] Hiển thị thông báo thành công
         │
         ▼
[Lưu ý] Vendor có thể kích hoạt lại bất cứ lúc nào
```

### Flow từ Admin (đình chỉ shop)

```
┌─────────────────────────────────────────────────────────────────┐
│                FLOW ADMIN ĐÌNH CHỈ SHOP                         │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Admin vào Vendors > Chi tiết shop
         │
         ▼
[Bước 2] Click "Đình chỉ gian hàng"
         │
         ▼
[Bước 3] Nhập thông tin:
         - Lý do đình chỉ
         - Thời hạn (tạm thời/vĩnh viễn)
         - Gửi email thông báo?
         │
         ▼
[Bước 4] Cập nhật shop status: suspended
         │
         ▼
[Bước 5] Cập nhật products status: hidden
         │
         ▼
[Bước 6] Xử lý pending orders:
         - Giữ nguyên đơn đã paid
         - Admin sẽ xử lý thủ công hoặc auto refund
         │
         ▼
[Bước 7] Freeze balance:
         - Vendor không thể rút tiền
         │
         ▼
[Bước 8] Gửi email thông báo cho vendor
         │
         ▼
[Bước 9] Ghi log admin action
```

---

## Database Schema

### Bảng shops
| Column | Type | Mô tả |
|--------|------|-------|
| id | bigint | PK |
| vendor_id | bigint | FK -> users |
| shop_name | varchar(50) | |
| shop_slug | varchar(60) | Unique |
| description | text | |
| logo | varchar(255) | |
| banner | varchar(255) | |
| contact_email | varchar(255) | |
| contact_phone | varchar(20) | |
| facebook_page | varchar(255) | |
| telegram_channel | varchar(100) | |
| zalo_oa | varchar(20) | |
| warranty_policy | text | |
| refund_policy | text | |
| support_hours | varchar(100) | |
| status | enum | active/inactive/suspended |
| rating | decimal(3,2) | |
| total_reviews | int | |
| total_sales | int | |
| total_products | int | |
| allow_reseller | boolean | |
| reseller_discount | decimal(5,2) | |
| commission_rate | decimal(5,2) | |
| level | enum | new/silver/gold/diamond/partner |
| suspended_reason | text | |
| suspended_until | timestamp | |
| created_at | timestamp | |
| updated_at | timestamp | |

### Bảng shop_slugs_history
| Column | Type | Mô tả |
|--------|------|-------|
| id | bigint | PK |
| shop_id | bigint | FK |
| old_slug | varchar(60) | |
| created_at | timestamp | |
| expires_at | timestamp | Redirect hết hạn sau 30 ngày |
