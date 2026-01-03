# Hệ Thống Shop V2 - Tài Liệu Flow Hoàn Chỉnh

## Tổng quan

Tài liệu này định nghĩa TẤT CẢ flows, điều kiện và biến số cho hệ thống Shop từ 3 góc nhìn: **Vendor** (Người bán), **Buyer** (Người mua), và **Admin**.

**Thay đổi chính so với V1:**
- ❌ Bỏ: Hệ thống reseller (allow_reseller, reseller_discount, quản lý reseller)
- ✅ Bắt buộc: Telegram username khi tạo shop
- ✅ Giữ: Hệ thống shop nhiều cấp độ (New → Silver → Gold → Diamond → Partner)
- ✅ Giữ: Cấu trúc phí dựa trên commission
- ✅ Mới: Dashboard đơn giản hóa với analytics 7 ngày

---

# PHẦN 1: FLOWS CỦA VENDOR

## 1. Tạo Gian Hàng (Vendor)

### 1.1 Điều kiện thực hiện

┌─────────────────────────────────────────────────────────────┐
│               ĐIỀU KIỆN TẠO GIAN HÀNG                      │
└─────────────────────────────────────────────────────────────┘

1. **Preconditions** (Trước khi flow bắt đầu)
   ├── Actor: Vendor (đã đăng ký và được duyệt)
   ├── State: Vendor chưa có gian hàng nào
   └── Data: vendor_id từ JWT token

2. **Input Requirements** (Dữ liệu đầu vào)
   ├── shop_name: String (3-50 ký tự, bắt buộc)
   ├── shop_slug: String (3-60 ký tự, unique, bắt buộc)
   ├── shop_description: String (tối đa 500 ký tự, bắt buộc)
   ├── shop_logo: File (jpg/png, max 2MB, bắt buộc)
   ├── shop_banner: File (jpg/png, max 5MB, tùy chọn)
   ├── telegram_username: String (@username format, bắt buộc)
   ├── warranty_policy: String (tùy chọn)
   ├── refund_policy: String (tùy chọn)
   └── support_hours: String (tùy chọn)

3. **Validation Rules** (Quy tắc validate)
   ├── shop_name: Required, min=3, max=50
   ├── shop_slug: Required, unique, alphanumeric + hyphen
   ├── shop_description: Required, max=500
   ├── telegram_username: Required, phải bắt đầu bằng @
   ├── logo: Required, file type valid, size <= 2MB
   └── banner: Optional, file type valid, size <= 5MB

4. **Edge Cases** (Trường hợp đặc biệt)
   ├── Slug đã tồn tại ──► Trả về lỗi, KHÔNG gợi ý (theo yêu cầu user)
   ├── File upload thất bại ──► Hiển thị lỗi upload
   ├── Telegram username sai format ──► Validate ở client
   └── Tài khoản vendor không hợp lệ ──► 403 Forbidden

---

### 1.2 Flow Tạo Gian Hàng

┌─────────────────────────────────────────────────────────────┐
│              FLOW TẠO GIAN HÀNG (4 BƯỚC)                   │
└─────────────────────────────────────────────────────────────┘

[Bước 1] User truy cập /vendor/dashboard
         │
         ├── Shop đã tồn tại ──► Redirect sang /vendor/shop/dashboard
         ├── Chưa có shop ──► [Bước 2]
         │
         ▼
[Bước 2] Hiển thị Wizard - Bước 1: Thông tin cơ bản
         │
         │ UI Fields:
         │ ├── shop_name (text input)
         │ ├── shop_slug (text input, auto-generate từ shop_name)
         │ └── shop_description (textarea)
         │
         ├── User click "Tiếp tục" ──► [Bước 3]
         │
         ▼
[Bước 3] Validate thông tin cơ bản
         │
         ├── Validate: shop_name (3-50 chars)
         ├── Validate: shop_slug (3-60 chars, alphanumeric + hyphen)
         ├── Validate: shop_description (max 500 chars)
         ├── Check slug uniqueness:
         │   ├── Slug đã tồn tại ──► Return error "Slug đã được sử dụng"
         │   └── Slug available ──► [Bước 4]
         │
         ▼
[Bước 4] Hiển thị Wizard - Bước 2: Hình ảnh (Branding)
         │
         │ UI Fields:
         │ ├── shop_logo (file upload, REQUIRED)
         │   ├── Allowed: jpg, jpeg, png
         │   ├── Max size: 2MB
         │   └── Recommended: 200x200px
         │ └── shop_banner (file upload, OPTIONAL)
         │     ├── Allowed: jpg, jpeg, png
         │     ├── Max size: 5MB
         │     └── Recommended: 1200x300px
         │
         ├── User click "Tiếp tục" ──► [Bước 5]
         │
         ▼
[Bước 5] Xử lý file upload
         │
         ├── Validate file type (MIME check)
         ├── Validate file size
         ├── Resize to standard dimensions
         ├── Optimize compression
         ├── Store in: /storage/shops/{shop_id}/
         │
         ├── Upload thành công ──► [Bước 6]
         ├── Upload thất bại ──► Return error "Upload file thất bại"
         │
         ▼
[Bước 6] Hiển thị Wizard - Bước 3: Telegram (REQUIRED)
         │
         │ UI Field:
         │ └── telegram_username (text input, REQUIRED)
         │     ├── Format: @username (MUST start with @)
         │     ├── Min length: 11 chars (including @)
         │     └── Max length: 32 chars
         │
         │ Instructions:
         │ "Bạn sẽ cần gửi /start {verification_code}
         │  cho bot @p2pmmo để xác nhận"
         │
         ├── User click "Tiếp tục" ──► [Bước 7]
         │
         ▼
[Bước 7] Validate Telegram username
         │
         ├── Validate: Starts with @
         ├── Validate: Length 11-32 chars
         ├── Validate: Pattern ^@[a-zA-Z0-9_]{5,31}$
         │
         ├── Valid ──► [Bước 8]
         ├── Invalid ──► Return error "Sai định dạng @username"
         │
         ▼
[Bước 8] Hiển thị Wizard - Bước 4: Chính sách (Optional)
         │
         │ UI Fields:
         │ ├── warranty_policy (textarea, OPTIONAL)
         │ ├── refund_policy (textarea, OPTIONAL)
         │ └── support_hours (text, OPTIONAL)
         │     Example: "8h-22h hàng ngày"
         │
         ├── User click "Tạo gian hàng" ──► [Bước 9]
         │
         ▼
[Bước 9] Final Validation & Create Shop Record
         │
         ├── Validate: ALL required fields present
         ├── Re-check: shop_slug uniqueness
         ├── Validate: Logo file uploaded
         ├── Validate: Telegram username format
         │
         ├── All valid ──► [Bước 10]
         ├── Invalid ──► Return validation errors
         │
         ▼
[Bước 10] Insert vào MongoDB
         │
         ├── Collection: shops
         ├── Document:
         │   {
         │     _id: ObjectId,
         │     vendor_id: ObjectId (from JWT),
         │     shop_name: String,
         │     shop_slug: String (indexed, unique),
         │     description: String,
         │     logo: String (file path),
         │     banner: String | null,
         │     telegram_username: String,
         │     telegram_chat_id: null,
         │     telegram_verified: false,
         │     warranty_policy: String | null,
         │     refund_policy: String | null,
         │     support_hours: String | null,
         │     status: "active",
         │     rating: 0.00,
         │     total_reviews: 0,
         │     total_sales: 0,
         │     total_products: 0,
         │     commission_rate: 10.00,
         │     level: "new",
         │     suspended_reason: null,
         │     suspended_until: null,
         │     created_at: DateTime,
         │     updated_at: DateTime
         │   }
         │
         ├── Success ──► [Bước 11]
         ├── Error ──► Return error "Không thể tạo shop"
         │
         ▼
[Bước 11] Tạo storage directory
         │
         ├── Create: /storage/shops/{shop_id}/
         ├── Subdirectories:
         │   ├── products/
         │   ├── documents/
         │   └── exports/
         │
         ▼
[Bước 12] Generate Telegram verification code
         │
         ├── Generate: UUID v4
         ├── Store in Redis:
         │   Key: telegram:verify:{shop_id}
         │   Value: {verification_code, telegram_username}
         │   TTL: 86400 seconds (24 hours)
         │
         ▼
[Bước 13] Queue welcome email
         │
         ├── To: vendor.email
         ├── Subject: "Chào mừng đến với P2PMMO"
         ├── Template: welcome_email.html
         └── Queue: email_queue
         │
         ▼
[Bước 14] Redirect sang Dashboard
         │
         ├── URL: /vendor/shop/dashboard
         ├── Flash: "Gian hàng đã tạo thành công!
         │          Vui lòng xác nhận Telegram."
         └── END

---

### 1.3 Edge Cases & Error Handling

| Case | Condition | Handling | User Message |
|------|-----------|----------|--------------|
| Shop đã tồn tại | shop_id found cho vendor_id | Redirect | - |
| Slug đã được sử dụng | shop_slug exists trong DB | Return error | "Slug này đã được sử dụng" |
| Shop name quá ngắn | length < 3 | Validate | "Tên shop phải từ 3-50 ký tự" |
| Shop name quá dài | length > 50 | Validate | "Tên shop phải từ 3-50 ký tự" |
| Slug sai format | contains special chars | Validate | "Slug chỉ được chứa chữ, số và gạch ngang" |
| Telegram sai format | không bắt đầu bằng @ | Validate | "Username phải bắt đầu bằng @" |
| Logo chưa upload | logo file is null | Validate | "Vui lòng upload logo shop" |
| Logo file quá lớn | size > 2MB | Validate | "Logo không được quá 2MB" |
| Logo sai định dạng | not jpg/png | Validate | "Logo phải là định dạng JPG hoặc PNG" |
| Banner quá lớn | size > 5MB | Validate | "Banner không được quá 5MB" |

---

## 2. Xác Thực Telegram (Vendor)

### 2.1 Điều kiện thực hiện

┌─────────────────────────────────────────────────────────────┐
│            ĐIỀU KIỆN XÁC THỰC TELEGRAM                     │
└─────────────────────────────────────────────────────────────┘

1. **Preconditions**
   ├── Actor: Vendor (đã tạo shop)
   ├── State: shop.telegram_verified = false
   └── Data: verification_code từ Redis

2. **Input Requirements**
   └── verification_code: String (UUID format)

3. **Validation Rules**
   └── verification_code: phải tồn tại trong Redis

4. **Edge Cases**
   ├── Code không tồn tại ──► Error "Mã không hợp lệ hoặc đã hết hạn"
   ├── Code đã hết hạn ──► Error "Mã đã hết hạn (24h)"
   ├── Shop đã verified ──► Info "Đã xác nhận, không cần làm lại"
   └── Telegram username không khớp ──► Warning nhưng cho phép tiếp tục

---

### 2.2 Flow Xác Thực Telegram

┌─────────────────────────────────────────────────────────────┐
│            FLOW XÁC THỰC TELEGRAM                          │
└─────────────────────────────────────────────────────────────┘

[Bước 1] User gửi /start {verification_code} cho @p2pmmo
         │
         ▼
[Bước 2] Bot nhận command qua Telegram Bot API
         │
         ├── Extract:
         │   ├── telegram_chat_id (from update)
         │   └── verification_code (from message text)
         │
         ├── Không có code ──► Reply: "Vui lòng nhập mã xác nhận"
         │
         ▼
[Bước 3] Validate verification code
         │
         ├── Query Redis:
         │   ├── Pattern: telegram:verify:*
         │   ├── Filter: value.verification_code == {code}
         │   └── Return: shop_id, telegram_username
         │
         ├── Code not found ──► [B3a] Return error
         ├── Code found ──► [Bước 4]
         │
         ▼
[Bước 3a] Bot gửi lỗi message
         │
         ├── Message: "Mã không hợp lệ hoặc đã hết hạn.
         │              Vui lòng kiểm tra lại hoặc yêu cầu mã mới."
         └── END (Flow terminate)
         │
         ▼
[Bước 4] Fetch shop details
         │
         ├── Query MongoDB:
         │   ├── Collection: shops
         │   ├── Filter: { _id: shop_id }
         │   └── Return: shop document
         │
         ├── Shop not found ──► [B4a] Return error
         ├── Shop found ──► [Bước 5]
         │
         ▼
[Bước 4a] Bot gửi error message
         │
         ├── Message: "Không tìm thấy gian hàng.
         │              Vui lòng liên hệ support."
         └── END (Flow terminate)
         │
         ▼
[Bước 5] Check verification status
         │
         ├── telegram_verified = true ──► [B5a] Info message
         ├── telegram_verified = false ──► [Bước 6]
         │
         ▼
[Bước 5a] Bot gửi info message
         │
         ├── Message: "Gian hàng này đã được xác nhận."
         └── END (Flow terminate)
         │
         ▼
[Bước 6] Verify Telegram username (soft validation)
         │
         ├── Get user profile from Telegram API
         ├── Compare: user.username == shop.telegram_username
         │
         ├── Match ──► [Bước 7]
         ├── No match ──► [B6a] Warning nhưng continue
         │
         ▼
[Bước 6a] Bot gửi warning
         │
         ├── Message: "⚠️ Username không khớp với shop settings.
         │              Tuy nhiên, bạn vẫn có thể tiếp tục."
         └── Continue sang [Bước 7]
         │
         ▼
[Bước 7] Update shop với telegram_chat_id
         │
         ├── Update MongoDB:
         │   ├── Collection: shops
         │   ├── Filter: { _id: shop_id }
         │   ├── Set:
         │   │   telegram_chat_id: {chat_id}
         │   │   telegram_verified: true
         │   │   telegram_verified_at: DateTime
         │   │   updated_at: DateTime
         │
         ├── Success ──► [Bước 8]
         ├── Error ──► [B7a] Return error
         │
         ▼
[Bước 7a] Bot gửi error message
         │
         ├── Message: "Có lỗi xảy ra. Vui lòng thử lại sau."
         └── END (Flow terminate)
         │
         ▼
[Bước 8] Delete verification code từ Redis
         │
         ├── DEL telegram:verify:{shop_id}
         │
         ▼
[Bước 9] Bot gửi success message
         │
         ├── Message:
         │   """
         │   ✅ Xác nhận thành công!
         │
         │   Gian hàng "{shop_name}" đã liên kết với Telegram.
         │
         │   Bạn sẽ nhận được thông báo:
         │   • 🆕 Khi có đơn hàng mới
         │   • ✅ Khi đơn hàng được thanh toán
         │   • ⚠️ Khi có khiếu nại mới
         │   • 📉 Khi sản phẩm sắp hết hàng
         │   • 🎉 Khi shop lên level
         │
         │   Chúc bạn bán hàng多多!
         │   """
         │
         ▼
[Bước 10] Send test notification (direct call)
         │
         ├── Gọi telegram_service.send_message()
         ├── Message: "🔔 Đây là thông báo kiểm tra.
         │               Bạn đã thiết lập thành công!"
         │
         ├── Delivery success ──► [Bước 11]
         ├── Delivery fail ──► Log warning, continue
         │
         ▼
[Bước 11] Update dashboard banner (real-time)
         │
         ├── Via WebSocket (if connected)
         ├── Remove: "Telegram verification pending" banner
         ├── Show: "✅ Telegram đã xác nhận"
         │
         └── END

---

### 2.3 Edge Cases & Error Handling

| Case | Condition | Handling | User Message |
|------|-----------|----------|--------------|
| Code không tồn tại | Redis lookup returns null | Return error | "Mã không hợp lệ hoặc đã hết hạn" |
| Code đã hết hạn | Redis TTL expired | Return error | "Mã đã hết hạn (24h)" |
| Shop không tồn tại | DB query returns null | Return error | "Không tìm thấy gian hàng" |
| Đã verified trước đó | telegram_verified = true | Info message | "Gian hàng đã được xác nhận" |
| Username không khớp | telegram username khác | Warning + Continue | "Username không khớp nhưng có thể tiếp tục" |
| DB update fail | MongoDB error | Return error | "Có lỗi xảy ra, vui lòng thử lại" |
| Bot API error | HTTP error | Log + Retry | - |

---

## 3. Xem Dashboard (Vendor)

### 3.1 Điều kiện thực hiện

┌─────────────────────────────────────────────────────────────┐
│               ĐIỀU KIỆN XEM DASHBOARD                      │
└─────────────────────────────────────────────────────────────┘

1. **Preconditions**
   ├── Actor: Vendor (đã đăng nhập)
   ├── State: Có shop đã tạo
   └── Data: vendor_id từ JWT token

2. **Input Requirements**
   └── shop_id: ObjectId (từ vendor_id lookup)

3. **Validation Rules**
   ├── User phải logged in
   ├── User role = vendor
   └── Vendor có shop

4. **Edge Cases**
   ├── Chưa đăng nhập ──► Redirect /login
   ├── Không phải vendor ──► 403 Forbidden
   ├── Chưa có shop ──► Redirect /vendor/shop/create
   └── Shop bị đình chỉ ──► Show suspension notice

---

### 3.2 Flow Xem Dashboard

┌─────────────────────────────────────────────────────────────┐
│              FLOW XEM DASHBOARD                             │
└─────────────────────────────────────────────────────────────┘

[Bước 1] User truy cập /vendor/shop/dashboard
         │
         ├── Check auth: NOT logged in ──► Redirect /login
         ├── Check auth: logged in ──► [Bước 2]
         │
         ▼
[Bước 2] Check user role
         │
         ├── role != "vendor" ──► 403 Forbidden
         ├── role = "vendor" ──► [Bước 3]
         │
         ▼
[Bước 3] Load vendor's shop
         │
         ├── Query MongoDB:
         │   ├── Collection: shops
         │   ├── Filter: { vendor_id: vendor_id }
         │   └── Return: shop document
         │
         ├── No shop found ──► Redirect /vendor/shop/create
         ├── shop.status = "suspended" ──► [B3a] Show notice
         ├── shop.status = "active" ──► [Bước 4]
         │
         ▼
[Bước 3a] Show suspension notice
         │
         ├── UI: Warning banner
         ├── Message: "Gian hàng của bạn đang bị đình chỉ.
         │              Lý do: {suspended_reason}"
         └── Continue sang [Bước 4]
         │
         ▼
[Bước 4] Check Redis cache
         │
         ├── Query Redis:
         │   ├── Key: dashboard:{shop_id}
         │   └── TTL: 300 seconds (5 minutes)
         │
         ├── Cache hit ──► [Bước 16] (Skip calculation)
         ├── Cache miss ──► [Bước 5] (Calculate fresh)
         │
         ▼
[Bước 5] Calculate Revenue Today
         │
         ├── Query MongoDB:
         │   ├── Collection: orders
         │   ├── Aggregate:
         │   │   [
         │   │     { $match: {
         │   │         shop_id: shop_id,
         │   │         status: { $in: ["paid", "delivered", "completed"] },
         │   │         DATE(created_at) = TODAY
         │   │     }},
         │   │     { $group: {
         │   │         _id: null,
         │   │         revenue: { $sum: "$total" }
         │   │     }}
         │   │   ]
         │   └── Return: revenue_today (Decimal)
         │
         ▼
[Bước 6] Calculate Orders Today
         │
         ├── Query MongoDB:
         │   ├── Collection: orders
         │   ├── Count:
         │   │   shop_id: shop_id,
         │   │   DATE(created_at) = TODAY
         │   └── Return: orders_today (Integer)
         │
         ▼
[Bước 7] Calculate Products in Stock
         │
         ├── Query MongoDB:
         │   ├── Collection: product_items
         │   ├── Aggregate:
         │   │   [
         │   │     { $match: {
         │   │         shop_id: shop_id,
         │   │         is_sold: false,
         │   │         status: "available"
         │   │     }},
         │   │     { $group: {
         │   │         _id: null,
         │   │         total: { $sum: "$quantity" }
         │   │     }}
         │   │   ]
         │   └── Return: products_in_stock (Integer)
         │
         ▼
[Bước 8] Calculate Available Balance
         │
         ├── Query MongoDB:
         │   ├── Collection: vendor_wallets
         │   ├── Find:
         │   │   vendor_id: vendor_id
         │   └── Return: available_balance (Decimal)
         │
         ├── Note: Số dư đã cleared (> 3 ngày)
         │
         ▼
[Bước 9] Get Shop Rating & Level
         │
         ├── From shop object (already loaded):
         │   ├── rating (Decimal)
         │   ├── total_reviews (Integer)
         │   └── level (String)
         │
         ├── Calculate badge:
         │   ├── new → 🆕
         │   ├── silver → 🥈
         │   ├── gold → 🥇
         │   ├── diamond → 💎
         │   └── partner → ✅
         │
         ▼
[Bước 10] Calculate Pending Disputes
         │
         ├── Query MongoDB:
         │   ├── Collection: disputes
         │   ├── Count:
         │   │   shop_id: shop_id,
         │   │   status: "pending"
         │   └── Return: pending_disputes (Integer)
         │
         ▼
[Bước 11] Get Revenue Chart (7 Days)
         │
         ├── Query MongoDB:
         │   ├── Collection: orders
         │   ├── Aggregate:
         │   │   [
         │   │     { $match: {
         │   │         shop_id: shop_id,
         │   │         status: { $in: ["paid", "delivered", "completed"] },
         │   │         created_at: { $gte: 7 days ago }
         │   │     }},
         │   │     { $group: {
         │   │         _id: { $dateToString: { format: "%Y-%m-%d", date: "$created_at" }},
         │   │         revenue: { $sum: "$total" },
         │   │         orders: { $sum: 1 }
         │   │     }},
         │   │     { $sort: { _id: 1 }}
         │   │     ]
         │   └── Fill missing dates with 0
         │
         ├── Return: revenue_chart (Array of 7 data points)
         │
         ▼
[Bước 12] Get Top 5 Products
         │
         ├── Query MongoDB:
         │   ├── Collection: product_items
         │   ├── Aggregate:
         │   │   [
         │   │     { $match: {
         │   │         shop_id: shop_id,
         │   │         is_sold: true
         │   │     }},
         │   │     { $lookup: {
         │   │         from: "products",
         │   │         localField: "product_id",
         │   │         foreignField: "_id",
         │   │         as: "product"
         │   │     }},
         │   │     { $unwind: "$product" },
         │   │     { $group: {
         │   │         _id: "$product_id",
         │   │         product_name: { $first: "$product.name" },
         │   │         sold_count: { $sum: 1 },
         │   │         revenue: { $sum: "$price" }
         │   │     }},
         │   │     { $sort: { sold_count: -1 }},
         │   │     { $limit: 5 }
         │   │   ]
         │   └── Return: top_products (Array)
         │
         ▼
[Bước 13] Get Alerts
         │
         ├── Query 1 - Low Stock:
         │   ├── MongoDB product_items
         │   ├── Match: quantity < 10 AND quantity > 0
         │   └── Group by product_id
         │
         ├── Query 2 - Sold Out:
         │   ├── MongoDB products
         │   ├── Match: no available items
         │   └── status = "active"
         │
         ├── Query 3 - Pending Disputes:
         │   ├── Already counted in [Bước 10]
         │   └── If count > 0, add alert
         │
         └── Return: alerts (Array)
         │
         ▼
[Bước 14] Get Recent Activity (Last 10)
         │
         ├── Query Union MongoDB:
         │   ├── Collection: orders
         │   │   Match: shop_id
         │   │   Sort: created_at DESC
         │   │   Limit: 5
         │   │   Project: type="order", title="Đơn #code"
         │   │
         │   ├── Collection: disputes
         │   │   Match: shop_id
         │   │   Sort: created_at DESC
         │   │   Limit: 5
         │   │   Project: type="dispute", title="Khiếu nại #id"
         │   │
         │   └── Collection: wallet_transactions
         │       Match: vendor_id, type="commission_received"
         │       Sort: created_at DESC
         │       Limit: 5
         │       Project: type="wallet", title="Tiền về: +amount"
         │
         └── Return: recent_activity (Array, sorted, limit 10)
         │
         ▼
[Bước 15] Cache dashboard data
         │
         ├── SET Redis:
         │   ├── Key: dashboard:{shop_id}
         │   ├── Value: JSON of all data
         │   └── TTL: 300 seconds
         │
         ▼
[Bước 16] Render Dashboard UI
         │
         ┌─────────────────────────────────────────────────┐
         │  SHOP DASHBOARD                                 │
         ├─────────────────────────────────────────────────┤
         │  Header:                                        │
         │  ├── Shop name                                  │
         │  ├── Level badge                                │
         │  └── Rating: ⭐ 4.5/5 (123 đánh giá)             │
         │                                                 │
         │  Quick Stats (Today):                           │
         │  ┌────────┬────────┬────────┬────────────┐      │
         │  │Revenue │ Orders │ Stock  │ Balance    │      │
         │  │1.250K  │ 15     │ 1,234  │ 5.430K     │      │
         │  └────────┴────────┴────────┴────────────┘      │
         │                                                 │
         │  Revenue Chart (7 Days):                        │
         │  [Line/Bar chart visualization]                 │
         │                                                 │
         │  Top 5 Products:                                │
         │  1. Gmail US - 234 sold                         │
         │  2. Facebook - 189 sold                         │
         │  ...                                            │
         │                                                 │
         │  Alerts:                                        │
         │  ⚠️ 3 products low stock                        │
         │  ⚠️ 2 disputes pending                          │
         │                                                 │
         │  Recent Activity:                               │
         │  • 10:30 - Order #12345 - ₫50,000               │
         │  • 10:25 - Dispute from user_abc                │
         │  ...                                            │
         └─────────────────────────────────────────────────┘
         │
         └── END

---

### 3.3 Edge Cases & Error Handling

| Case | Condition | Handling | User Message |
|------|-----------|----------|--------------|
| Chưa đăng nhập | JWT token missing | Redirect | - |
| Không phải vendor | role != "vendor" | 403 | "Bạn không có quyền truy cập" |
| Chưa có shop | shop not found | Redirect | - |
| Shop bị đình chỉ | status = "suspended" | Show notice | "Gian hàng đang bị đình chỉ" |
| Cache hit | Redis key exists | Return cached | - |
| Wallet không tồn tại | wallet not found | Show 0 | "Số dư: ₫0" |
| Query timeout | MongoDB slow query | Log + Retry | - |

---

## 4. Cập Nhật Thông Tin Shop (Vendor)

### 4.1 Điều kiện thực hiện

┌─────────────────────────────────────────────────────────────┐
│            ĐIỀU KIỆN CẬP NHẬT SHOP                        │
└─────────────────────────────────────────────────────────────┘

1. **Preconditions**
   ├── Actor: Vendor (đã đăng nhập)
   ├── State: Có shop đã tạo
   └── Data: vendor_id từ JWT

2. **Input Requirements** (Tất cả optional)
   ├── shop_name: String (3-50 chars)
   ├── shop_slug: String (3-60 chars, unique)
   ├── description: String (max 500)
   ├── logo: File (jpg/png, max 2MB)
   ├── banner: File (jpg/png, max 5MB)
   ├── telegram_username: String (@username format)
   ├── warranty_policy: String
   ├── refund_policy: String
   └── support_hours: String

3. **Validation Rules**
   ├── Nếu shop_slug thay đổi → phải unique
   ├── Nếu telegram_username thay đổi → reset verification
   └── Logo cũ bị xóa khi upload new

4. **Edge Cases**
   ├── Slug đã tồn tại ──► Error "Slug đã được sử dụng"
   ├── Telegram thay đổi ──► Reset verified = false
   └── File upload fail ──► Error message

---

### 4.2 Flow Cập Nhật Shop

┌─────────────────────────────────────────────────────────────┐
│            FLOW CẬP NHẬT THÔNG TIN SHOP                    │
└─────────────────────────────────────────────────────────────┘

[Bước 1] User truy cập /vendor/shop/settings
         │
         ├── Check auth: NOT logged in ──► Redirect /login
         ├── Check auth: logged in ──► [Bước 2]
         │
         ▼
[Bước 2] Load current shop data
         │
         ├── Query MongoDB:
         │   ├── Collection: shops
         │   ├── Filter: { vendor_id: vendor_id }
         │   └── Return: shop document
         │
         ├── Not found ──► Redirect /vendor/shop/create
         ├── Found ──► [Bước 3]
         │
         ▼
[Bước 3] Display pre-filled form
         │
         ┌─────────────────────────────────────────────────┐
         │  SHOP SETTINGS                                  │
         ├─────────────────────────────────────────────────┤
         │  Shop Name:     [Taphoa MMO Shop        ]      │
         │  Shop Slug:     [taphoa-mmo-shop        ]      │
         │  Description:   [Premium digital...    ]      │
         │                                                 │
         │  Logo:          [📁 Browse]  Current: [View]   │
         │  Banner:        [📁 Browse]  Current: [View]   │
         │                                                 │
         │  Telegram:      [@taphoammo             ]      │
         │                                                 │
         │  Warranty:      [_________________]            │
         │  Refund:        [_________________]            │
         │  Support Hours: [8h-22h hàng ngày      ]      │
         │                                                 │
         │                      [Cancel] [Save Changes]   │
         └─────────────────────────────────────────────────┘
         │
         ├── User edits fields ──► [Bước 4]
         │
         ▼
[Bước 4] User clicks "Save Changes"
         │
         ├── Collect changed fields only
         ├── Validate changes:
         │   ├── shop_name: min 3, max 50 (if changed)
         │   ├── shop_slug: min 3, max 60, unique (if changed)
         │   ├── description: max 500 (if changed)
         │   ├── telegram: starts with @ (if changed)
         │   └── logo/banner: file validation (if uploaded)
         │
         ├── Invalid ──► Return validation errors
         ├── Valid ──► [Bước 5]
         │
         ▼
[Bước 5] Handle shop_slug change (if applicable)
         │
         ├── shop_slug CHANGED ──► [B5a]
         ├── shop_slug unchanged ──► [Bước 7]
         │
         ▼
[Bước 5a] Check uniqueness
         │
         ├── Query MongoDB:
         │   ├── Collection: shops
         │   ├── Filter: {
         │   │     shop_slug: new_slug,
         │   │     _id: { $ne: current_shop_id }
         │   │   }
         │
         ├── Slug exists ──► Return error "Slug đã được sử dụng"
         ├── Slug available ──► [B5b]
         │
         ▼
[Bước 5b] Store slug history
         │
         ├── Insert MongoDB:
         │   ├── Collection: shop_slugs_history
         │   ├── Document:
         │   │   {
         │   │     shop_id: ObjectId,
         │   │     old_slug: String,
         │   │     created_at: DateTime,
         │   │     expires_at: DateTime (NOW + 30 days)
         │   │     }
         │
         └── Continue sang [Bước 6]
         │
         ▼
[Bước 6] Setup redirect
         │
         ├── Note: 301 redirect from old_slug to new_slug
         ├── Expires: 30 days
         │
         └── Continue sang [Bước 7]
         │
         ▼
[Bước 7] Process file uploads (if applicable)
         │
         ├── Logo uploaded ──► [B7a]
         ├── Banner uploaded ──► [B7b]
         ├── No file upload ──► [Bước 8]
         │
         ▼
[Bước 7a] Process logo
         │
         ├── Validate file type (MIME)
         ├── Validate file size (<= 2MB)
         ├── Resize to 200x200px
         ├── Optimize compression
         ├── Store: /storage/shops/{shop_id}/logo_{timestamp}.jpg
         ├── Delete old logo file
         │
         └── Continue sang [Bước 8]
         │
         ▼
[Bước 7b] Process banner
         │
         ├── Validate file type (MIME)
         ├── Validate file size (<= 5MB)
         ├── Resize to 1200x300px
         ├── Optimize compression
         ├── Store: /storage/shops/{shop_id}/banner_{timestamp}.jpg
         ├── Delete old banner (if exists)
         │
         └── Continue sang [Bước 8]
         │
         ▼
[Bước 8] Update MongoDB
         │
         ├── Collection: shops
         ├── Filter: { _id: shop_id }
         ├── Set:
         │   ├── Only changed fields
         │   ├── updated_at: DateTime
         │
         ├── Success ──► [Bước 9]
         ├── Error ──► Return error "Cập nhật thất bại"
         │
         ▼
[Bước 9] Invalidate cache
         │
         ├── DEL Redis: dashboard:{shop_id}
         ├── DEL Redis: shop:{shop_id}
         ├── DEL Redis: shop:slug:{old_slug} (if changed)
         │
         └── Continue sang [Bước 10]
         │
         ▼
[Bước 10] Handle telegram_username change (if applicable)
         │
         ├── telegram_username CHANGED ──► [B10a]
         ├── telegram_username unchanged ──► [Bước 11]
         │
         ▼
[Bước 10a] Reset telegram verification
         │
         ├── Update MongoDB:
         │   ├── Collection: shops
         │   ├── Set:
         │   │   telegram_chat_id: null
         │   │   telegram_verified: false
         │   │   telegram_verified_at: null
         │
         ├── Generate new verification code
         ├── Store in Redis: telegram:verify:{shop_id}
         │
         ├── Gọi telegram_service.send_message() (direct):
         │   ├── To: old telegram_chat_id (if exists)
         │   ├── Message: "Telegram username đã thay đổi.
         │                 Vui lòng xác nhận lại."
         │
         ├── Send email to vendor:
         │   ├── Queue: email_queue
         │   ├── Subject: "Vui lòng xác nhận lại Telegram"
         │   ├── Content: New verification code
         │
         └── Continue sang [Bước 11]
         │
         ▼
[Bước 11] Return success response
         │
         ├── Flash: "Cập nhật thành công"
         ├── If telegram changed:
         │   └── Flash: "Vui lòng xác nhận lại Telegram"
         │
         └── END

---

### 4.3 Edge Cases & Error Handling

| Case | Condition | Handling | User Message |
|------|-----------|----------|--------------|
| Slug đã tồn tại | shop_slug exists (exclude self) | Return error | "Slug đã được sử dụng" |
| File quá lớn | size > limit | Validate | "File quá lớn" |
| Sai định dạng | invalid MIME type | Validate | "Định dạng không hợp lệ" |
| Upload fail | storage error | Return error | "Upload thất bại" |
| DB update fail | MongoDB error | Return error | "Cập nhật thất bại" |
| Telegram thay đổi | username changed | Reset verified | "Vui lòng xác nhận lại" |

---

## 5. Tự Động Lên Level (System)

### 5.1 Điều kiện thực hiện

┌─────────────────────────────────────────────────────────────┐
│          ĐIỀU KIỆN TỰ ĐỘNG LÊN LEVEL                      │
└─────────────────────────────────────────────────────────────┘

1. **Preconditions**
   ├── Actor: System (Cron job OR Event-based)
   ├── State: Shop status = "active"
   └── Trigger: Every hour OR after every 10 orders

2. **Input Requirements**
   └── None (automated)

3. **Business Rules**
   ├── Partner level: Manual only (không auto)
   ├── Diamond: 1000+ orders, rating >= 4.8
   ├── Gold: 200+ orders, rating >= 4.5
   ├── Silver: 50+ orders, rating >= 4.0
   └── New: Default

4. **Edge Cases**
   ├── Rating drop ──► (Optional) Downgrade
   └── Partner ──► Never downgrade

---

### 5.2 Flow Tự Động Lên Level

┌─────────────────────────────────────────────────────────────┐
│            FLOW TỰ ĐỘNG LÊN LEVEL                          │
└─────────────────────────────────────────────────────────────┘

[Bước 1] Trigger: Cron OR Event
         │
         ├── Cron: Every hour at :00
         ├── Event: After every 10 orders (total_sales % 10 = 0)
         │
         ▼
[Bước 2] Query shops cần check
         │
         ├── Query MongoDB:
         │   ├── Collection: shops
         │   ├── Filter:
         │   │   status: "active"
         │   │   level: { $ne: "partner" }
         │   │   $or: [
         │   │     { updated_at: { $lt: 1 hour ago }},
         │   │     { total_sales: { $mod: [10, 0] }}
         │   │   ]
         │
         └── Return: shops_to_check (Array)
         │
         ▼
[Bước 3] For each shop, calculate new level
         │
         ├── Loop: for shop in shops_to_check
         │   │
         │   ├── Get metrics:
         │   │   ├── total_sales = shop.total_sales
         │   │   └── rating = shop.rating
         │   │
         │   ├── Calculate new_level:
         │   │   ├── IF total_sales >= 1000 AND rating >= 4.8
         │   │   │   THEN new_level = "diamond"
         │   │   ├── ELSE IF total_sales >= 200 AND rating >= 4.5
         │   │   │   THEN new_level = "gold"
         │   │   ├── ELSE IF total_sales >= 50 AND rating >= 4.0
         │   │   │   THEN new_level = "silver"
         │   │   ├── ELSE
         │   │   │   THEN new_level = "new"
         │   │   │
         │   │   └── Compare: new_level vs shop.level
         │   │
         │   ├── new_level != shop.level ──► Add to upgrade_list
         │   └── new_level == shop.level ──► Skip
         │
         └── upgrade_list (Array)
         │
         ▼
[Bước 4] Process level upgrades
         │
         ├── Loop: for shop in upgrade_list
         │   │
         │   ├── Get commission rate:
         │   │   ├── diamond → 5%
         │   │   ├── gold → 6%
         │   │   ├── silver → 8%
         │   │   └── new → 10%
         │   │
         │   ├── Update MongoDB:
         │   │   ├── Collection: shops
         │   │   ├── Filter: { _id: shop._id }
         │   │   ├── Set:
         │   │   │   level: new_level
         │   │   │   commission_rate: new_rate
         │   │   │   updated_at: DateTime
         │   │   │
         │   │   └── Get badge emoji
         │   │       ├── new → 🆕
         │   │       ├── silver → 🥈
         │   │       ├── gold → 🥇
         │   │       └── diamond → 💎
         │   │
         │   └── Continue sang [Bước 5]
         │
         └── Continue
         │
         ▼
[Bước 5] Send notifications (direct telegram calls)
         │
         ├── Loop: for upgraded_shop in upgrade_list
         │   │
         │   ├── Gọi telegram_service.notify_level_up() (direct):
         │   │   ├── To: shop.telegram_chat_id
         │   │   ├── Message:
         │   │   │   """
         │   │   │   🎉 Chúc mừng!
         │   │   │   │
         │   │   │   Gian hàng của bạn đã lên level:
         │   │   │   {old_badge} → {new_badge}
         │   │   │   │
         │   │   │   Phí sàn: {old_rate}% → {new_rate}%
         │   │   │   """
         │   │
         │   ├── Queue Email notification:
         │   │   ├── Queue: email_queue
         │   │   ├── To: vendor.email
         │   │   ├── Subject: "Chúc mừng shop lên level {new_level}!"
         │   │   └── Template: level_up_email.html
         │   │
         │   └── Create in-app notification
         │       ├── Collection: notifications
         │       ├── Document: {
         │       │     user_id: vendor_id,
         │       │     type: "level_up",
         │       │     title: "Lên level mới!",
         │       │     message: "Shop đã lên {new_level}",
         │       │     read: false,
         │       │     created_at: DateTime
         │       │     }
         │
         └── Continue
         │
         ▼
[Bước 6] Log level changes
         │
         ├── Loop: for upgraded_shop in upgrade_list
         │   │
         │   ├── Insert MongoDB:
         │   │   ├── Collection: shop_level_history
         │   │   ├── Document:
         │   │   │   {
         │   │   │     shop_id: ObjectId,
         │   │   │     old_level: String,
         │   │   │     new_level: String,
         │   │   │     old_commission: Decimal,
         │   │   │     new_commission: Decimal,
         │   │   │     triggered_by: "system_auto",
         │   │   │     created_at: DateTime
         │   │   │   }
         │
         └── Continue
         │
         ▼
[Bước 7] Invalidate cache
         │
         ├── Loop: for shop in upgrade_list
         │   │
         │   ├── DEL Redis: dashboard:{shop._id}
         │   └── DEL Redis: shop:{shop._id}
         │
         └── END

---

### 5.3 Level Calculation Logic

```javascript
// Pseudo-code cho level calculation
function calculateShopLevel(totalSales, rating, currentLevel) {
  // Partner never downgrade
  if (currentLevel === 'partner') return 'partner';

  // Check from highest to lowest
  if (totalSales >= 1000 && rating >= 4.8) return 'diamond';
  if (totalSales >= 200 && rating >= 4.5) return 'gold';
  if (totalSales >= 50 && rating >= 4.0) return 'silver';

  // Default
  return 'new';
}

function getCommissionRate(level) {
  const rates = {
    new: 10,
    silver: 8,
    gold: 6,
    diamond: 5,
    partner: 0  // Custom
  };
  return rates[level] || 10;
}
```

---

### 5.4 Edge Cases & Error Handling

| Case | Condition | Handling | User Message |
|------|-----------|----------|--------------|
| Partner level | level = "partner" | Skip check | - |
| Rating drop | metrics below threshold | (Optional) Downgrade | - |
| No shops to check | empty array | Return early | - |
| DB update fail | MongoDB error | Log + Retry | - |
| Notification fail | Telegram/Email error | Log only | - |

---

# PHẦN 2: FLOWS CỦA BUYER

## 6. Xem Trang Shop (Buyer)

### 6.1 Điều kiện thực hiện

┌─────────────────────────────────────────────────────────────┐
│             ĐIỀU KIỆN XEM TRANG SHOP                       │
└─────────────────────────────────────────────────────────────┘

1. **Preconditions**
   ├── Actor: Buyer (người mua, có thể guest)
   ├── State: Không cần đăng nhập
   └── Data: shop_id và shop_slug từ URL

2. **Input Requirements**
   ├── shop_slug: String (từ URL)
   └── shop_id: ObjectId (từ URL)

3. **Validation Rules**
   ├── ObjectId format valid
   └── Shop status = "active"

4. **Edge Cases**
   ├── Shop không tồn tại ──► 404 Not Found
   ├── Shop không active ──► 404 "Shop không khả dụng"
   ├── Slug thay đổi ──► 301 redirect
   └── Shop bị đình chỉ ──► Show suspension message

---

### 6.2 Flow Xem Trang Shop

┌─────────────────────────────────────────────────────────────┐
│              FLOW XEM TRANG SHOP                            │
└─────────────────────────────────────────────────────────────┘

[Bước 1] User truy cập /gian-hang/{shop_slug}_{shop_id}
         │
         ├── Validate ObjectId format
         ├── Invalid format ──► 400 Bad Request
         ├── Valid format ──► [Bước 2]
         │
         ▼
[Bước 2] Parse URL parameters
         │
         ├── Extract: shop_slug (string)
         ├── Extract: shop_id (ObjectId)
         │
         ▼
[Bước 3] Query shop details
         │
         ├── Query MongoDB:
         │   ├── Collection: shops
         │   ├── Filter: {
         │   │     _id: shop_id,
         │   │     deleted_at: null
         │   │   }
         │   └── Return: shop document
         │
         ├── Not found ──► [B3a] Return 404
         ├── Found ──► [Bước 4]
         │
         ▼
[Bước 3a] Return 404 Not Found
         │
         ├── Response: {
         │   error: "Không tìm thấy gian hàng"
         │ }
         └── END
         │
         ▼
[Bước 4] Check shop status
         │
         ├── shop.status != "active" ──► [B4a] Return 404
         ├── shop.status = "active" ──► [Bước 5]
         │
         ▼
[Bước 4a] Return 404 with message
         │
         ├── Response: {
         │   error: "Gian hàng không khả dụng"
         │ }
         └── END
         │
         ▼
[Bước 5] Check slug redirect (if slug changed)
         │
         ├── shop_slug (from DB) != shop_slug (from URL)
         │   ──► [B5a] Check redirect history
         ├── shop_slug matches ──► [Bước 6]
         │
         ▼
[Bước 5a] Query slug history
         │
         ├── Query MongoDB:
         │   ├── Collection: shop_slugs_history
         │   ├── Filter: {
         │   │     shop_id: shop_id,
         │   │     old_slug: slug_from_url,
         │   │     expires_at: { $gt: NOW() }
         │   │   }
         │
         ├── Found ──► [B5b] Return 301 redirect
         ├── Not found ──► [Bước 6]
         │
         ▼
[Bước 5b] Return 301 Redirect
         │
         ├── Location: /gian-hang/{current_slug}_{shop_id}
         └── END
         │
         ▼
[Bước 6] Increment view count (async)
         │
         ├── Update MongoDB:
         │   ├── Collection: shops
         │   ├── Filter: { _id: shop_id }
         │   ├── $inc: { view_count: 1 }
         │
         ├── Note: Fire and forget, don't wait
         │
         ▼
[Bước 7] Parse query parameters
         │
         ├── page: Integer (default: 0)
         ├── tab: Enum (products|reviews|info, default: products)
         ├── sort_by: Enum (newest|best_selling|price_asc|price_desc)
         └── category: String (optional)
         │
         ▼
[Bước 8] Query shop products
         │
         ├── Query MongoDB:
         │   ├── Collection: products
         │   ├── Aggregate:
         │   │   [
         │   │     { $match: {
         │   │         shop_id: shop_id,
         │   │         status: "active",
         │   │         deleted_at: null
         │   │     }},
         │   │     { $lookup: {
         │   │         from: "product_items",
         │   │         localField: "_id",
         │   │         foreignField: "product_id",
         │   │         as: "items"
         │   │     }},
         │   │     { $addFields: {
         │   │         available_count: {
         │   │             $size: {
         │   │                 $filter: {
         │   │                     input: "$items",
         │   │                     cond: {
         │   │                         $and: [
         │   │                             { $eq: ["$$this.is_sold", false] },
         │   │                             { $eq: ["$$this.status", "available"] }
         │   │                         ]
         │   │                     }
         │   │                 }
         │   │             }
         │   │         },
         │   │         min_price: { $min: "$items.price" }
         │   │     }},
         │   │     { $sort: { created_at: -1 }},
         │   │     { $skip: page * 20 },
         │   │     { $limit: 20 }
         │   │     ]
         │
         └── Return: products (Array)
         │
         ▼
[Bước 9] Query shop reviews (if tab = reviews)
         │
         ├── tab != "reviews" ──► Skip to [Bước 10]
         ├── tab = "reviews" ──► Execute query
         │
         ├── Query MongoDB:
         │   ├── Collection: reviews
         │   ├── Aggregate:
         │   │   [
         │   │     { $match: {
         │   │         shop_id: shop_id,
         │   │         status: "approved",
         │   │         verified: true
         │   │     }},
         │   │     { $lookup: {
         │   │         from: "users",
         │   │         localField: "user_id",
         │   │         foreignField: "_id",
         │   │         as: "user"
         │   │     }},
         │   │     { $unwind: "$user" },
         │   │     { $project: {
         │   │         rating: 1,
         │   │         comment: 1,
         │   │         anonymous: 1,
         │   │         created_at: 1,
         │   │         username: {
         │   │             $cond: [
         │   │                 { $eq: ["$anonymous", true] },
         │   │                 "Người dùng ẩn danh",
         │   │                 "$user.username"
         │   │             ]
         │   │         },
         │   │         avatar: "$user.avatar"
         │   │     }},
         │   │     { $sort: { created_at: -1 }},
         │   │     { $skip: page * 10 },
         │   │     { $limit: 10 }
         │   │     ]
         │
         └── Return: reviews (Array)
         │
         ▼
[Bước 10] Calculate shop stats
         │
         ├── From shop object:
         │   ├── rating (Decimal)
         │   ├── total_reviews (Integer)
         │   ├── total_sales (Integer)
         │   ├── total_products (Integer)
         │   ├── level (String)
         │   └── created_at (DateTime)
         │
         ├── Calculate completion_rate:
         │   ├── Query MongoDB orders collection
         │   ├── Match: shop_id, status = "completed"
         │   ├── Formula: (completed / total) * 100
         │
         └── Return: stats (Object)
         │
         ▼
[Bước 11] Render shop page UI
         │
         ┌─────────────────────────────────────────────────┐
         │  SHOP PAGE                                      │
         ├─────────────────────────────────────────────────┤
         │  Header Section:                                │
         │  ┌─────────────────────────────────────────┐   │
         │  │ [Banner 1200x300]                      │   │
         │  ├─────────────────────────────────────────┤   │
         │  │ [Logo] Shop Name   🥇 Gold              │   │
         │  │ ⭐ 4.8/5 (1,234 đánh giá)               │   │
         │  │ 5,678 đã bán | Tham gia: 01/2024        │   │
         │  │                                         │   │
         │  │ [Liên hệ qua Telegram @username]        │   │
         │  └─────────────────────────────────────────┘   │
         │                                                 │
         │  Description Section:                           │
         │  Shop description text here...                 │
         │                                                 │
         │  Tabs Navigation:                              │
         │  [Sản phẩm] [Đánh giá] [Thông tin]            │
         │                                                 │
         │  Products Tab:                                 │
         │  ┌─────────────────────────────────────────┐   │
         │  │ Filter: [Category ▼] [Sort: Mới nhất▼]│   │
         │  ├─────────────────────────────────────────┤   │
         │  │ ┌──────────┐ ┌──────────┐ ┌──────────┐ │   │
         │  │ │ Product  │ │ Product  │ │ Product  │ │   │
         │  │ │  Image   │ │  Image   │ │  Image   │ │   │
         │  │ │ ₫50,000  │ │ ₫75,000  │ │ ₫30,000  │ │   │
         │  │ │ 25 stock │ │ 12 stock │ │ 0 stock  │ │   │
         │  │ └──────────┘ └──────────┘ └──────────┘ │   │
         │  └─────────────────────────────────────────┘   │
         │                                                 │
         │  Pagination: [< 1 2 3 4 5 >]                  │
         └─────────────────────────────────────────────────┘
         │
         └── END

---

### 6.3 Edge Cases & Error Handling

| Case | Condition | Handling | User Message |
|------|-----------|----------|--------------|
| Shop không tồn tại | DB query returns null | 404 | "Không tìm thấy gian hàng" |
| Shop không active | status != "active" | 404 | "Gian hàng không khả dụng" |
| Slug thay đổi | old_slug in URL | 301 redirect | - |
| View count fail | MongoDB error | Log only | - |
| No products | empty array | Show message | "Chưa có sản phẩm nào" |
| No reviews | empty array | Show message | "Chưa có đánh giá nào" |

---

## 7. Liên Hệ Shop Qua Telegram (Buyer)

### 7.1 Điều kiện thực hiện

┌─────────────────────────────────────────────────────────────┐
│          ĐIỀU KIỆN LIÊN HỆ TELEGRAM                        │
└─────────────────────────────────────────────────────────────┘

1. **Preconditions**
   ├── Actor: Buyer (không cần đăng nhập)
   ├── State: Đang xem trang shop
   └── Data: shop.telegram_username

2. **Input Requirements**
   └── None (click action)

3. **Validation Rules**
   └── telegram_username phải tồn tại

4. **Edge Cases**
   ├── Telegram username không có ──► Hide button
   └── Invalid username ──► Log error

---

### 7.2 Flow Liên Hệ Telegram

┌─────────────────────────────────────────────────────────────┐
│            FLOW LIÊN HỆ TELEGRAM                            │
└─────────────────────────────────────────────────────────────┘

[Bước 1] User clicks "Liên hệ qua Telegram"
         │
         ├── Get shop.telegram_username
         ├── username not exists ──► Do nothing
         ├── username exists ──► [Bước 2]
         │
         ▼
[Bước 2] Generate Telegram link
         │
         ├── Format: https://t.me/{username_without_@}
         ├── Example: https://t.me/taphoammo
         │
         ▼
[Bước 3] Open Telegram
         │
         ├── Action: window.open(telegram_link, '_blank')
         └── END

---

### 7.3 Edge Cases & Error Handling

| Case | Condition | Handling | User Message |
|------|-----------|----------|--------------|
| Username không tồn tại | telegram_username = null | Hide button | - |
| Invalid username | invalid format | Log error | - |

---

## 8. Để Lại Đánh Giá Shop (Buyer)

### 8.1 Điều kiện thực hiện

┌─────────────────────────────────────────────────────────────┐
│            ĐIỀU KIỆN ĐÁNH GIÁ SHOP                        │
└─────────────────────────────────────────────────────────────┘

1. **Preconditions**
   ├── Actor: Buyer (đã đăng nhập)
   ├── State: Đơn hàng đã completed
   └── Data: order_id, shop_id, user_id

2. **Input Requirements**
   ├── order_id: ObjectId
   ├── rating: Integer (1-5, required)
   ├── comment: String (max 1000, required)
   └── anonymous: Boolean (optional)

3. **Validation Rules**
   ├── Order phải thuộc user hiện tại
   ├── Order status = "completed"
   ├── Chưa review HOẶC updating existing review
   └── rating trong range 1-5

4. **Edge Cases**
   ├── Order not found ──► 404
   ├── Order not completed ──► Error "Chưa thể đánh giá"
   ├── Already reviewed ──► Load existing for update
   └── Invalid rating ──► Validate error

---

### 8.2 Flow Đánh Giá Shop

┌─────────────────────────────────────────────────────────────┐
│              FLOW ĐÁNH GIÁ SHOP                            │
└─────────────────────────────────────────────────────────────┘

[Bước 1] User clicks "Đánh giá" from order page
         │
         ├── Check auth: NOT logged in ──► Redirect /login
         ├── Logged in ──► [Bước 2]
         │
         ▼
[Bước 2] Check order eligibility
         │
         ├── Query MongoDB:
         │   ├── Collection: orders
         │   ├── Filter: {
         │   │     _id: order_id,
         │   │     buyer_id: current_user_id,
         │   │     shop_id: shop_id,
         │   │     status: "completed",
         │   │     deleted_at: null
         │   │   }
         │   └── Return: order document
         │
         ├── Not found ──► [B2a] Return error
         ├── Not completed ──► [B2b] Return error
         ├── Found ──► [Bước 3]
         │
         ▼
[Bước 2a] Return error
         │
         ├── Response: {
         │   error: "Không tìm thấy đơn hàng"
         │ }
         └── END
         │
         ▼
[Bước 2b] Return error
         │
         ├── Response: {
         │   error: "Chưa thể đánh giá. Đơn hàng chưa hoàn thành."
         │ }
         └── END
         │
         ▼
[Bước 3] Check existing review
         │
         ├── Query MongoDB:
         │   ├── Collection: reviews
         │   ├── Filter: {
         │   │     order_id: order_id,
         │   │     user_id: current_user_id,
         │   │     shop_id: shop_id
         │   │   }
         │   └── Return: existing_review or null
         │
         ├── Exists ──► [Bước 4] (Update mode)
         ├── Not exists ──► [Bước 4] (Create mode)
         │
         ▼
[Bước 4] Display review form
         │
         ┌─────────────────────────────────────────────────┐
         │  REVIEW FORM                                    │
         ├─────────────────────────────────────────────────┤
         │  Đánh giá đơn hàng: #12345                      │
         │                                                 │
         │  Rating: ⭐⭐⭐⭐⭐ (1-5 stars, clickable)      │
         │                                                 │
         │  Comment:                                      │
         │  ┌─────────────────────────────────────────┐   │
         │  │ Chất lượng tốt, giao hàng nhanh.        │   │
         │  │ Cảm ơn shop!                            │   │
         │  └─────────────────────────────────────────┘   │
         │  (tối đa 1000 ký tự)                          │
         │                                                 │
         │  ☐ Ẩn danh (không hiển thị tên)              │
         │                                                 │
         │        [Cancel]            [Gửi đánh giá]      │
         │                                                 │
         │  (hoặc [Cập nhật đánh giá] nếu edit)           │
         └─────────────────────────────────────────────────┘
         │
         ├── User fills form ──► [Bước 5]
         │
         ▼
[Bước 5] Submit review
         │
         ├── Input: rating, comment, anonymous
         ├── Validate:
         │   ├── rating: required, 1-5
         │   └── comment: required, max 1000
         │
         ├── Invalid ──► Return validation errors
         ├── Valid ──► [Bước 6]
         │
         ▼
[Bước 6] Save or update review
         │
         ├── existing_review exists ──► [B6a] Update
         ├── existing_review null ──► [B6b] Insert
         │
         ▼
[Bước 6a] Update existing review
         │
         ├── Update MongoDB:
         │   ├── Collection: reviews
         │   ├── Filter: { _id: existing_review._id }
         │   ├── Set:
         │   │   rating: rating
         │   │   comment: comment
         │   │   anonymous: anonymous
         │   │   updated_at: DateTime
         │
         └── Continue sang [Bước 7]
         │
         ▼
[Bước 6b] Insert new review
         │
         ├── Insert MongoDB:
         │   ├── Collection: reviews
         │   ├── Document:
         │   │   {
         │   │     _id: ObjectId,
         │   │     order_id: order_id,
         │   │     shop_id: shop_id,
         │   │     user_id: current_user_id,
         │   │     rating: rating,
         │   │     comment: comment,
         │   │     anonymous: anonymous,
         │   │     verified: true,
         │   │     status: "approved",
         │   │     created_at: DateTime,
         │   │     updated_at: DateTime
         │   │   }
         │
         └── Continue sang [Bước 7]
         │
         ▼
[Bước 7] Recalculate shop rating
         │
         ├── Query MongoDB:
         │   ├── Collection: reviews
         │   ├── Aggregate:
         │   │   [
         │   │     { $match: {
         │   │         shop_id: shop_id,
         │   │         status: "approved",
         │   │         verified: true
         │   │     }},
         │   │     { $group: {
         │   │         _id: null,
         │   │         avg_rating: { $avg: "$rating" },
         │   │         total_reviews: { $sum: 1 }
         │   │     }}
         │   │   ]
         │
         └── Return: new_rating, new_review_count
         │
         ▼
[Bước 8] Update shop rating
         │
         ├── Update MongoDB:
         │   ├── Collection: shops
         │   ├── Filter: { _id: shop_id }
         │   ├── Set:
         │   │   rating: new_rating
         │   │   total_reviews: new_review_count
         │   │   updated_at: DateTime
         │
         ├── Success ──► [Bước 9]
         ├── Error ──► Log only, continue
         │
         ▼
[Bước 9] Trigger level check (async)
         │
         ├── Queue: level_check_job
         ├── Shop level may change based on new rating
         │
         └── Fire and forget
         │
         ▼
[Bước 10] Send notification to vendor (direct call)
         │
         ├── Gọi telegram_service.send_message() (direct):
         │   ├── To: shop.telegram_chat_id
         │   ├── Message:
         │   │   """
         │   │   ⭐ Đánh giá mới từ khách hàng!
         │   │
         │   │   Đơn hàng: #{order_code}
         │   │   Số sao: {rating}⭐
         │   │   Nội dung: {comment}
         │   │   """
         │
         └── Continue
         │
         ▼
[Bước 11] Invalidate shop cache
         │
         ├── DEL Redis: dashboard:{shop_id}
         ├── DEL Redis: shop:{shop_id}
         │
         ▼
[Bước 12] Return success response
         │
         ├── Response: {
         │   message: "Đánh giá thành công",
         │   review: { ... }
         │ }
         │
         └── END

---

### 8.3 Edge Cases & Error Handling

| Case | Condition | Handling | User Message |
|------|-----------|----------|--------------|
| Order not found | DB query returns null | 404 | "Không tìm thấy đơn hàng" |
| Not completed | status != "completed" | Error | "Chưa thể đánh giá" |
| Invalid rating | rating not 1-5 | Validate | "Rating phải từ 1-5 sao" |
| Comment too long | length > 1000 | Validate | "Bình luận tối đa 1000 ký tự" |
| DB insert fail | MongoDB error | Return error | "Không thể lưu đánh giá" |
| Rating update fail | MongoDB error | Log only | - |

---

# PHẦN 3: FLOWS CỦA ADMIN

## 9. Xem Tất Cả Shops (Admin)

### 9.1 Điều kiện thực hiện

┌─────────────────────────────────────────────────────────────┐
│            ĐIỀU KIỆN XEM TẤT CẢ SHOPS                     │
└─────────────────────────────────────────────────────────────┘

1. **Preconditions**
   ├── Actor: Admin (đã đăng nhập)
   ├── State: Admin có quyền xem shops
   └── Data: admin_id từ JWT token

2. **Input Requirements** (Query parameters - all optional)
   ├── status: Enum (active|inactive|suspended, default: all)
   ├── level: Enum (new|silver|gold|diamond|partner, default: all)
   ├── search: String (shop_name or telegram_username)
   ├── sort_by: Enum (created_at|rating|total_sales, default: created_at)
   ├── sort_order: Enum (asc|desc, default: desc)
   ├── page: Integer (default: 1)
   └── per_page: Integer (default: 20, max: 100)

3. **Validation Rules**
   ├── User role = "admin"
   ├── per_page <= 100
   └── status/level values valid nếu có

4. **Edge Cases**
   ├── Không phải admin ──► 403 Forbidden
   ├── Page > total pages ──► Return empty array
   └── Invalid filter ──► Ignore filter

---

### 9.2 Flow Xem Tất Cả Shops

┌─────────────────────────────────────────────────────────────┐
│              FLOW XEM TẤT CẢ SHOPS (ADMIN)                  │
└─────────────────────────────────────────────────────────────┘

[A1] Admin truy cập /admin/shops
         │
         ├── Check auth: NOT logged in ──► Redirect /login
         ├── Logged in ──► [A2]
         │
         ▼
[A2] Check admin role
         │
         ├── role != "admin" ──► 403 Forbidden
         ├── role = "admin" ──► [A3]
         │
         ▼
[A3] Parse query parameters
         │
         ├── status: "all" (default) hoặc specific value
         ├── level: "all" (default) hoặc specific value
         ├── search: null hoặc search string
         ├── sort_by: "created_at" (default)
         ├── sort_order: "desc" (default)
         ├── page: 1 (default)
         └── per_page: 20 (default, max 100)
         │
         ▼
[A4] Build MongoDB query
         │
         ├── Base filter: { deleted_at: null }
         │
         ├── Apply status filter:
         │   ├── status != "all"
         │   └── Add: status: {status_value}
         │
         ├── Apply level filter:
         │   ├── level != "all"
         │   └── Add: level: {level_value}
         │
         ├── Apply search filter:
         │   ├── search not empty
         │   └── Add: $or: [
         │         { shop_name: { $regex: search, $options: "i" }},
         │         { telegram_username: { $regex: search, $options: "i" }}
         │       ]
         │
         └── Query object ready
         │
         ▼
[A5] Execute count query
         │
         ├── Query MongoDB:
         │   ├── Collection: shops
         │   ├── Count: { query from A4 }
         │   └── Return: total_shops
         │
         ▼
[A6] Execute data query
         │
         ├── Query MongoDB:
         │   ├── Collection: shops
         │   ├── Find: { query from A4 }
         │   ├── Sort: { sort_by: sort_order }
         │   ├── Skip: (page - 1) * per_page
         │   └── Limit: per_page
         │
         └── Return: shops (Array)
         │
         ▼
[A7] Render admin shop list
         │
         ┌─────────────────────────────────────────────────┐
         │  ADMIN SHOP LIST                                │
         ├─────────────────────────────────────────────────┤
         │  Filters:                                       │
         │  Status: [All ▼] Level: [All ▼]                │
         │  Search: [_______________] [Search]            │
         │                                                 │
         │  ┌──────┬──────┬─────────┬────────┬────┬────┐ │
         │  │ Logo │ Name │Telegram │ Level  │Status│Act │ │
         │  ├──────┼──────┼─────────┼────────┼────┼────┤ │
         │  │[img] │ShopA │@teleA   │ 🥇Gold │●Act │... │ │
         │  │[img] │ShopB │@teleB   │ 🆕New  │⚠Sus │... │ │
         │  │[img] │ShopC │@teleC   │ 💎Diam │●Act │... │ │
         │  └──────┴──────┴─────────┴────────┴────┴────┘ │
         │                                                 │
         │  Actions per shop:                              │
         │  [View] [Edit] [Suspend/Activate] [Delete]    │
         │                                                 │
         │  Pagination: [< 1 2 3 ... 10 >]                │
         │  Total: 234 shops                              │
         └─────────────────────────────────────────────────┘
         │
         └── END

---

### 9.3 Edge Cases & Error Handling

| Case | Condition | Handling | User Message |
|------|-----------|----------|--------------|
| Không phải admin | role != "admin" | 403 | "Bạn không có quyền truy cập" |
| per_page quá lớn | > 100 | Cap at 100 | - |
| Invalid status | not in enum | Ignore filter | - |
| Invalid level | not in enum | Ignore filter | - |
| Page > total | exceed max page | Return empty | - |
| DB query timeout | slow query | Log + Retry | - |

---

## 10. Đình Chỉ Shop (Admin)

### 10.1 Điều kiện thực hiện

┌─────────────────────────────────────────────────────────────┐
│              ĐIỀU KIỆN ĐÌNH CHỈ SHOP                       │
└─────────────────────────────────────────────────────────────┘

1. **Preconditions**
   ├── Actor: Admin (đã đăng nhập)
   ├── State: Admin có quyền suspend shop
   └── Data: shop_id, admin_id

2. **Input Requirements** (Từ form)
   ├── reason: String (required, max 1000)
   ├── duration: Enum (temporary|permanent, required)
   ├── duration_days: Integer (required if temporary, 1-365)
   ├── notify_vendor: Boolean (default: true)
   ├── freeze_balance: Boolean (default: true)
   └── handle_pending_orders: Enum (keep|cancel|admin_manual, required)

3. **Validation Rules**
   ├── reason required
   ├── duration_days required if temporary
   └── handle_pending_orders required

4. **Edge Cases**
   ├── Shop không tồn tại ──► 404
   ├── Shop đã suspended ──► Update existing
   └── Không có pending orders ──► Skip handling

---

### 10.2 Flow Đình Chỉ Shop

┌─────────────────────────────────────────────────────────────┐
│              FLOW ĐÌNH CHỈ SHOP (ADMIN)                     │
└─────────────────────────────────────────────────────────────┘

[A1] Admin clicks "Đình chỉ" on shop detail
         │
         ├── Check auth: NOT logged in ──► Redirect /login
         ├── Logged in ──► [A2]
         │
         ▼
[A2] Load shop detail
         │
         ├── Query MongoDB:
         │   ├── Collection: shops
         │   ├── Filter: { _id: shop_id }
         │   └── Return: shop document
         │
         ├── Not found ──► Return 404
         ├── Found ──► [A3]
         │
         ▼
[A3] Display suspension confirmation modal
         │
         ┌─────────────────────────────────────────────────┐
         │  ĐÌNH CHỈ GIAN HÀNG                            │
         ├─────────────────────────────────────────────────┤
         │  Shop: Taphoa MMO Shop                          │
         │  Vendor: @taphoammo                            │
         │                                                 │
         │  Lý do đình chỉ: * (required)                  │
         │  ┌─────────────────────────────────────────┐   │
         │  │                                         │   │
         │  └─────────────────────────────────────────┘   │
         │  (tối đa 1000 ký tự)                          │
         │                                                 │
         │  Thời hạn:                                    │
         │  ○ Tạm thời                                   │
         │     Số ngày: [7] (1-365)                      │
         │  ○ Vĩnh viễn                                   │
         │                                                 │
         │  Tùy chọn:                                    │
         │  ☑ Gửi thông báo cho vendor                    │
         │  ☑ Đóng băng số dư ví                         │
         │                                                 │
         │  Xử lý đơn hàng đang chờ:                     │
         │  ○ Giữ nguyên                                  │
         │  ● Hủy và hoàn tiền (theo yêu cầu user)        │
         │  ○ Admin xử lý thủ công                        │
         │                                                 │
         │        [Hủy]              [Xác nhận]           │
         └─────────────────────────────────────────────────┘
         │
         ├── Admin fills form ──► [A4]
         │
         ▼
[A4] Validate input
         │
         ├── Validate: reason (required, max 1000)
         ├── Validate: duration (required)
         ├── Validate: duration_days (required if temporary)
         ├── Validate: handle_pending_orders (required)
         │
         ├── Invalid ──► Return validation errors
         ├── Valid ──► [A5]
         │
         ▼
[A5] Calculate suspension dates
         │
         ├── duration = "permanent"
         │   └── suspended_until = null
         │
         ├── duration = "temporary"
         │   └── suspended_until = NOW() + duration_days days
         │
         ▼
[A6] Update shop status
         │
         ├── Update MongoDB:
         │   ├── Collection: shops
         │   ├── Filter: { _id: shop_id }
         │   ├── Set:
         │   │   status: "suspended"
         │   │   suspended_reason: reason
         │   │   suspended_until: suspended_until (or null)
         │   │   updated_at: DateTime
         │
         ├── Success ──► [A7]
         ├── Error ──► Return error
         │
         ▼
[A7] Hide all shop products
         │
         ├── Update MongoDB:
         │   ├── Collection: products
         │   ├── Filter: {
         │   │     shop_id: shop_id,
         │   │     status: "active"
         │   │     }
         │   ├── Set:
         │   │   status: "hidden"
         │   │   updated_at: DateTime
         │
         └── Continue
         │
         ▼
[A8] Handle pending orders
         │
         ├── handle_pending_orders = "cancel" ──► [A8a]
         ├── handle_pending_orders = "keep" ──► [A8b]
         ├── handle_pending_orders = "admin_manual" ──► [A8c]
         │
         ▼
[A8a] Cancel and refund orders
         │
         ├── Query MongoDB:
         │   ├── Collection: orders
         │   ├── Find: {
         │   │     shop_id: shop_id,
         │   │     status: { $in: ["pending", "paid"] }
         │   │   }
         │   └── Return: orders (Array)
         │
         ├── For each order:
         │   ├── Update order status = "cancelled"
         │   ├── Refund to buyer wallet:
         │   │   ├── Collection: buyer_wallets
         │   │   ├── $inc: { available_balance: order.total }
         │   │   ├── Create transaction record
         │   │
         │   ├── Gọi telegram_service.send_message() (direct):
         │   │   ├── To: buyer.telegram_chat_id
         │   │   ├── Message: "Đơn hàng đã bị hủy vì shop bị đình chỉ"
         │   │
         │   └── Gọi telegram_service.send_message() (direct):
         │       ├── To: shop.telegram_chat_id
         │       ├── Message: "Đơn hàng đã bị hủy và hoàn tiền cho buyer"
         │
         └── Continue sang [A9]
         │
         ▼
[A8b] Keep orders as-is
         │
         ├── Do nothing
         ├── Vendor can still process pending orders
         │
         └── Continue sang [A9]
         │
         ▼
[A8c] Create admin task for manual handling
         │
         ├── Insert MongoDB:
         │   ├── Collection: admin_tasks
         │   ├── Document:
         │   │   {
         │   │     shop_id: shop_id,
         │   │     type: "handle_suspended_shop_orders",
         │   │     priority: "high",
         │   │     status: "pending",
         │   │     created_at: DateTime
         │   │     }
         │
         ├── Notify admins:
         │   ├── Create notification for all admins
         │   └── Message: "Shop bị đình chỉ, cần xử lý orders thủ công"
         │
         └── Continue sang [A9]
         │
         ▼
[A9] Freeze wallet (if selected)
         │
         ├── freeze_balance = true ──► Execute
         ├── freeze_balance = false ──► Skip to [A10]
         │
         ├── Query MongoDB:
         │   ├── Collection: vendor_wallets
         │   ├── Find: { vendor_id: shop.vendor_id }
         │
         ├── Update MongoDB:
         │   ├── Set:
         │   │   is_frozen: true
         │   │   frozen_at: DateTime
         │   │   frozen_reason: reason
         │   │   updated_at: DateTime
         │
         ├── Effect:
         │   ├── Cannot withdraw
         │   ├── Cannot transfer
         │   └── Balance still shows
         │
         └── Continue sang [A10]
         │
         ▼
[A10] Notify vendor (if selected)
         │
         ├── notify_vendor = true ──► Execute
         ├── notify_vendor = false ──► Skip to [A11]
         │
         ├── Queue Email:
         │   ├── To: vendor.email
         │   ├── Subject: "Gian hàng của bạn đã bị đình chỉ"
         │   ├── Content:
         │   │   ├── Reason
         │   │   ├── Duration
         │   │   ├── What happens next
         │   │   └── Appeal process
         │
         ├── Gọi telegram_service.send_message() (direct):
         │   ├── To: shop.telegram_chat_id
         │   ├── Message:
         │   │   """
         │   │   ⚠️ THÔNG BÁO QUAN TRỌNG
         │   │
         │   │   Gian hàng của bạn đã bị đình chỉ.
         │   │
         │   │   Lý do: {reason}
         │   │   Thời hạn: {duration}
         │   │
         │   │   Vui lòng kiểm tra email để biết chi tiết.
         │   │   """
         │
         └── Continue sang [A11]
         │
         ▼
[A11] Log admin action
         │
         ├── Insert MongoDB:
         │   ├── Collection: admin_action_log
         │   ├── Document:
         │   │   {
         │   │     admin_id: admin_id,
         │   │     action: "suspend_shop",
         │   │     target_type: "shop",
         │   │     target_id: shop_id,
         │   │     details: {
         │   │       reason: reason,
         │   │       duration: duration,
         │   │       duration_days: duration_days,
         │   │       froze_balance: freeze_balance,
         │   │       handled_orders: handle_pending_orders
         │   │     },
         │   │     ip_address: client_ip,
         │   │     created_at: DateTime
         │   │     }
         │
         └── Continue sang [A12]
         │
         ▼
[A12] Invalidate caches
         │
         ├── DEL Redis: dashboard:{shop_id}
         ├── DEL Redis: shop:{shop_id}
         ├── DEL Redis: shop:slug:{shop_slug}
         ├── DEL Redis: shops:admin:list
         │
         ▼
[A13] Return success response
         │
         ├── Response: {
         │   message: "Đã đình chỉ gian hàng thành công"
         │ }
         │
         └── END

---

### 10.3 Edge Cases & Error Handling

| Case | Condition | Handling | User Message |
|------|-----------|----------|--------------|
| Shop not found | DB query returns null | 404 | "Không tìm thấy gian hàng" |
| Reason empty | reason = null/empty | Validate | "Vui lòng nhập lý do" |
| Invalid duration | not temporary\|permanent | Validate | "Thời hạn không hợp lệ" |
| duration_days missing | temporary but no days | Validate | "Vui lòng nhập số ngày" |
| No pending orders | empty orders array | Skip | - |
| Wallet not found | vendor_wallet not found | Log only | - |

---

## 11. Nâng Cấp Shop Lên Partner (Admin)

### 11.1 Điều kiện thực hiện

┌─────────────────────────────────────────────────────────────┐
│         ĐIỀU KIỆN NÂNG CẤP LÊN PARTNER                     │
└─────────────────────────────────────────────────────────────┘

1. **Preconditions**
   ├── Actor: Admin (đã đăng nhập)
   ├── State: Admin có quyền upgrade shop
   └── Data: shop_id, admin_id

2. **Input Requirements** (Từ form)
   ├── custom_commission_rate: Decimal (optional, 0-10)
   ├── notes: String (optional, max 1000)
   ├── notify_vendor: Boolean (default: true)
   └── effective_immediately: Boolean (default: true)

3. **Validation Rules**
   ├── custom_commission_rate trong range 0-10
   └── notes max 1000 characters

4. **Edge Cases**
   ├── Shop không tồn tại ──► 404
   ├── Đã là Partner ──► Update commission only
   └── Admin password required ──► Confirm action

---

### 11.2 Flow Nâng Cấp Lên Partner

┌─────────────────────────────────────────────────────────────┐
│         FLOW NÂNG CẤP LÊN PARTNER (ADMIN)                   │
└─────────────────────────────────────────────────────────────┘

[A1] Admin clicks "Nâng cấp Partner" on shop detail
         │
         ├── Check auth: NOT logged in ──► Redirect /login
         ├── Logged in ──► [A2]
         │
         ▼
[A2] Load shop detail
         │
         ├── Query MongoDB:
         │   ├── Collection: shops
         │   ├── Filter: { _id: shop_id }
         │   └── Return: shop document
         │
         ├── Display: Current level, stats, metrics
         │
         ├── Not found ──► Return 404
         ├── Found ──► [A3]
         │
         ▼
[A3] Display upgrade confirmation modal
         │
         ┌─────────────────────────────────────────────────┐
         │  NÂNG CẤP LÊN PARTNER                          │
         ├─────────────────────────────────────────────────┤
         │  ⚠️ CẢNH BÁO                                   │
         │  Thao tác này sẽ đưa shop lên cấp độ cao nhất.  │
         │  Không thể tự động downgrade, cần thủ công.    │
         │                                                 │
         │  Shop: Taphoa MMO Shop                          │
         │  Level hiện tại: 🥇 Gold (6% fee)              │
         │  Level mới: ✅ Partner                          │
         │                                                 │
         │  Phí sàn custom:                                │
         │  [____] % (để trống nếu chưa chốt, 0-10)       │
         │                                                 │
         │  Ghi chú:                                      │
         │  ┌─────────────────────────────────────────┐   │
         │  │ Thỏa thuận via email                   │   │
         │  └─────────────────────────────────────────┘   │
         │  (tối đa 1000 ký tự)                          │
         │                                                 │
         │  Tùy chọn:                                    │
         │  ☑ Gửi thông báo cho vendor                    │
         │  ☑ Áp dụng ngay lập tức                        │
         │                                                 │
         │  Xác nhận admin password:                      │
         │  [________________]                            │
         │                                                 │
         │        [Hủy]              [Xác nhận]           │
         └─────────────────────────────────────────────────┘
         │
         ├── Admin fills form ──► [A4]
         │
         ▼
[A4] Validate admin password
         │
         ├── Verify admin password
         ├── Invalid ──► Return error "Sai mật khẩu"
         ├── Valid ──► [A5]
         │
         ▼
[A5] Validate input
         │
         ├── Validate: custom_commission_rate (0-10)
         ├── Validate: notes (max 1000)
         │
         ├── Invalid ──► Return validation errors
         ├── Valid ──► [A6]
         │
         ▼
[A6] Update shop level
         │
         ├── Update MongoDB:
         │   ├── Collection: shops
         │   ├── Filter: { _id: shop_id }
         │   ├── Set:
         │   │   level: "partner"
         │   │   commission_rate: custom_rate OR existing
         │   │   updated_at: DateTime
         │
         ├── Success ──► [A7]
         ├── Error ──► Return error
         │
         ▼
[A7] Log level change
         │
         ├── Insert MongoDB:
         │   ├── Collection: shop_level_history
         │   ├── Document:
         │   │   {
         │   │     shop_id: shop_id,
         │   │     old_level: shop.level,
         │   │     new_level: "partner",
         │   │     old_commission: shop.commission_rate,
         │   │     new_commission: new_rate,
         │   │     triggered_by: "admin_manual",
         │   │     admin_id: admin_id,
         │   │     notes: notes,
         │   │     created_at: DateTime
         │   │     }
         │
         └── Continue sang [A8]
         │
         ▼
[A8] Notify vendor (if selected)
         │
         ├── notify_vendor = true ──► Execute
         ├── notify_vendor = false ──► Skip to [A9]
         │
         ├── Queue Email:
         │   ├── To: vendor.email
         │   ├── Subject: "Chúc mừng! Gian hàng của bạn đã trở thành Partner"
         │   ├── Content:
         │   │   ├── Congratulations message
         │   │   ├── Partner benefits
         │   │   ├── Custom commission (if set)
         │   │   └── Contact info for support
         │
         ├── Gọi telegram_service.notify_level_up() (direct):
         │   ├── To: shop.telegram_chat_id
         │   ├── Message:
         │   │   """
         │   │   🎊 CHÚC MỪNG!
         │   │
         │   │   Gian hàng của bạn đã trở thành
         │   │   ✅ PARTNER
         │   │
         │   │   Cảm ơn sự đồng hành cùng P2PMMO!
         │   │   """
         │
         └── Continue sang [A9]
         │
         ▼
[A9] Log admin action
         │
         ├── Insert MongoDB:
         │   ├── Collection: admin_action_log
         │   ├── Document:
         │   │   {
         │   │     admin_id: admin_id,
         │   │     action: "upgrade_shop_to_partner",
         │   │     target_type: "shop",
         │   │     target_id: shop_id,
         │   │     details: {
         │   │       notes: notes,
         │   │       custom_commission: custom_rate
         │   │     },
         │   │     ip_address: client_ip,
         │   │     created_at: DateTime
         │   │     }
         │
         └── Continue sang [A10]
         │
         ▼
[A10] Invalidate caches
         │
         ├── DEL Redis: dashboard:{shop_id}
         ├── DEL Redis: shop:{shop_id}
         │
         ▼
[A11] Return success response
         │
         ├── Response: {
         │   message: "Đã nâng cấp lên Partner thành công"
         │ }
         │
         └── END

---

### 11.3 Edge Cases & Error Handling

| Case | Condition | Handling | User Message |
|------|-----------|----------|--------------|
| Shop not found | DB query returns null | 404 | "Không tìm thấy gian hàng" |
| Invalid password | admin password wrong | Error | "Sai mật khẩu admin" |
| Invalid commission | rate not 0-10 | Validate | "Phí phải từ 0-10%" |
| Notes too long | length > 1000 | Validate | "Ghi chú tối đa 1000 ký tự" |
| Already Partner | level = "partner" | Update only | - |

---

# PHẦN 4: TELEGRAM BOT INTEGRATION

## 12. Telegram Bot Service Design

### 12.1 Kiến trúc Telegram Bot

**Thay đổi kiến trúc:**
- ❌ KHÔNG sử dụng Queue Worker (Redis queue + background worker)
- ✅ Sử dụng trực tiếp Telegram Bot Library cho Rust
- ✅ Gọi `send_message()` trong các function như gọi `logger::info()`

### 12.2 Telegram Bot Library cho Rust

**Recommended Crate:** `teloxide` hoặc `frankenstein`

```toml
# Cargo.toml
[dependencies]
teloxide = { version = "0.12", features = ["full"] }
log = "0.4"
```

### 12.3 Telegram Service Structure

```rust
// src/modules/shop/telegram_service.rs

use teloxide::{prelude::*, types::ParseMode};
use anyhow::Result;

pub struct TelegramService {
    bot: Bot,
}

impl TelegramService {
    pub fn new() -> Self {
        let bot_token = std::env::var("TELEGRAM_BOT_TOKEN")
            .expect("TELEGRAM_BOT_TOKEN must be set");
        Self {
            bot: Bot::new(bot_token),
        }
    }

    /// Gửi text message đơn giản
    pub async fn send_message(
        &self,
        chat_id: i64,
        text: String,
    ) -> Result<()> {
        self.bot
            .send_message(ChatId(chat_id), text)
            .parse_mode(ParseMode::Html)
            .disable_web_page_preview(true)
            .send()
            .await?;

        Ok(())
    }

    /// Gửi notification khi có đơn hàng mới
    pub async fn notify_new_order(
        &self,
        chat_id: i64,
        order_code: &str,
        product_name: &str,
        total: i64,
    ) -> Result<()> {
        let message = format!(
            "🆕 ĐƠN HÀNG MỚI\n\n\
             Mã đơn: #{}\n\
             Sản phẩm: {}\n\
             Tổng tiền: ₫{}\n\n\
             Vui lòng xử lý đơn hàng.",
            order_code, product_name, total
        );

        self.send_message(chat_id, message).await
    }

    /// Gửi notification khi đơn hàng đã thanh toán
    pub async fn notify_order_paid(
        &self,
        chat_id: i64,
        order_code: &str,
        amount: i64,
        net_amount: i64,
        commission_rate: f64,
    ) -> Result<()> {
        let message = format!(
            "✅ ĐÃ THANH TOÁN\n\n\
             Mã đơn: #{}\n\
             Số tiền: ₫{}\n\n\
             Ví của bạn:\n\
             +₫{} (sau phí sàn {}%)\n\n\
             Số tiền sẽ có sẵn sau 3 ngày.",
            order_code, amount, net_amount, commission_rate
        );

        self.send_message(chat_id, message).await
    }

    /// Gửi notification khi có khiếu nại
    pub async fn notify_dispute(
        &self,
        chat_id: i64,
        dispute_id: &str,
        order_code: &str,
        reason: &str,
    ) -> Result<()> {
        let message = format!(
            "⚠️ KHIẾU NẠI MỚI\n\n\
             Mã khiếu nại: #{}\n\
             Đơn hàng: #{}\n\
             Lý do: {}\n\n\
             Vui lòng phản hồi trong 24h.",
            dispute_id, order_code, reason
        );

        self.send_message(chat_id, message).await
    }

    /// Gửi notification khi sắp hết hàng
    pub async fn notify_low_stock(
        &self,
        chat_id: i64,
        product_name: &str,
        quantity: i32,
    ) -> Result<()> {
        let message = format!(
            "📉 SẮP HẾT HÀNG\n\n\
             Sản phẩm: {}\n\
             Số lượng còn: {}\n\n\
             Vui lòng nhập thêm hàng.",
            product_name, quantity
        );

        self.send_message(chat_id, message).await
    }

    /// Gửi notification khi lên level
    pub async fn notify_level_up(
        &self,
        chat_id: i64,
        old_level: &str,
        new_level: &str,
        old_rate: f64,
        new_rate: f64,
    ) -> Result<()> {
        let badges = map_level_to_badge();
        let message = format!(
            "🎉 LÊN LEVEL MỚI!\n\n\
             Chúc mừng gian hàng lên cấp độ:\n\
             {} → {}\n\n\
             Phí sàn: {}% → {}%\n\n\
             Cảm ơn bạn đã đồng hành cùng P2PMMO!",
            badges.get(old_level).unwrap_or(&"".to_string()),
            badges.get(new_level).unwrap_or(&"".to_string()),
            old_rate, new_rate
        );

        self.send_message(chat_id, message).await
    }

    /// Gửi báo cáo hàng ngày
    pub async fn notify_daily_summary(
        &self,
        chat_id: i64,
        date: &str,
        revenue: i64,
        orders: i32,
        items_sold: i32,
    ) -> Result<()> {
        let message = format!(
            "📊 BÁO CÁO NGÀY {}\n\n\
             Doanh thu: ₫{}\n\
             Đơn hàng: {} đơn\n\
             Sản phẩm bán: {}\n\n\
             Cảm ơn bạn đã nỗ lực!",
            date, revenue, orders, items_sold
        );

        self.send_message(chat_id, message).await
    }
}

// Helper function
fn map_level_to_badge() -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    map.insert("new".to_string(), "🆕".to_string());
    map.insert("silver".to_string(), "🥈".to_string());
    map.insert("gold".to_string(), "🥇".to_string());
    map.insert("diamond".to_string(), "💎".to_string());
    map.insert("partner".to_string(), "✅".to_string());
    map
}
```

### 12.4 Cách Sử dụng Trong Code

**Ví dụ: Trong Order Service**

```rust
// src/modules/order/service.rs

use crate::modules::shop::telegram_service::TelegramService;
use log::info;

pub struct OrderService {
    telegram: TelegramService,
    // ... other fields
}

impl OrderService {
    pub async fn create_order(
        &self,
        dto: CreateOrderDto,
    ) -> Result<Order, ServiceError> {
        // ... business logic ...

        // Gửi Telegram notification (fire and forget)
        if let Some(chat_id) = shop.telegram_chat_id {
            let telegram = self.telegram.clone();
            tokio::spawn(async move {
                if let Err(e) = telegram.notify_new_order(
                    chat_id,
                    &order.order_code,
                    &product.name,
                    order.total,
                ).await {
                    error!("Failed to send Telegram notification: {}", e);
                }
            });
        }

        // Log như bình thường
        info!("Order {} created successfully", order.order_code);

        Ok(order)
    }
}
```

### 12.5 Error Handling

```rust
// Wrap Telegram call với error handling
pub async fn safe_send_telegram(
    telegram: &TelegramService,
    chat_id: Option<i64>,
    message_fn: impl FnOnce(&TelegramService) -> impl Future<Output = Result<()>>,
) {
    if let Some(chat_id) = chat_id {
        if let Err(e) = message_fn(telegram).await {
            // Log error nhưng không ảnh hưởng business logic
            error!("Telegram notification failed: {}", e);
            // Optional: Mark shop có vấn đề
            // shop.telegram_has_issue = true;
        }
    }
}
```

### 12.6 Edge Cases & Error Handling

| Case | Condition | Handling | User Message |
|------|-----------|----------|--------------|
| chat_id null | telegram_chat_id = null | Skip notification | - |
| Bot blocked | API return 403/Forbidden | Log error, mark issue | - |
| Network error | Connection timeout | Log error only | - |
| Invalid chat_id | API return 400 | Log error, mark issue | - |
| Rate limit | Too many requests | Wait and retry | - |

---

# PHẦN 5: FLOWS HỆ THỐNG (Background Jobs)

## 13. Báo Cáo Hàng Ngày (Cron Job)

### 13.1 Điều kiện thực hiện

┌─────────────────────────────────────────────────────────────┐
│            ĐIỀU KIỆN BÁO CÁO HÀNG NGÀY                     │
└─────────────────────────────────────────────────────────────┘

1. **Preconditions**
   ├── Actor: System (Cron Job)
   ├── State: Cron scheduled at 22:00 daily
   └── Data: Tất cả active shops

2. **Input Requirements**
   └── None (automated)

3. **Business Rules**
   ├── Chỉ gửi cho shops có hoạt động trong ngày
   ├── Shop phải telegram_verified = true
   └── Skip nếu không có orders/items_sold

4. **Edge Cases**
   ├── No active shops ──► Return early
   ├── No activity today ──► Skip shop
   └── Telegram fail ──► Log only

---

### 13.2 Flow Báo Cáo Hàng Ngày

┌─────────────────────────────────────────────────────────────┐
│            FLOW BÁO CÁO HÀNG NGÀY (CRON)                   │
└─────────────────────────────────────────────────────────────┘

[B1] Cron triggers at 22:00 daily
         │
         ├── Timezone: UTC+7 (Vietnam)
         └── Continue sang [B2]
         │
         ▼
[B2] Query active shops with Telegram verified
         │
         ├── Query MongoDB:
         │   ├── Collection: shops
         │   ├── Filter:
         │   │   status: "active"
         │   │   telegram_verified: true
         │   │   telegram_chat_id: { $ne: null }
         │   │   deleted_at: null
         │   └── Return: active_shops (Array)
         │
         ├── Empty array ──► [B2a] Return early
         ├── Has shops ──► [B3]
         │
         ▼
[B2a] Log and return
         │
         ├── Log: "No active shops with Telegram found"
         └── END
         │
         ▼
[B3] For each shop, calculate daily stats
         │
         ├── Loop: for shop in active_shops
         │   │
         │   ├── Query orders today:
         │   │   ├── Collection: orders
         │   │   ├── Aggregate:
         │   │   │   [
         │   │   │     { $match: {
         │   │   │         shop_id: shop._id,
         │   │   │         DATE(created_at) = TODAY,
         │   │   │         status: { $in: ["paid", "completed", "delivered"] }
         │   │   │     }},
         │   │   │     { $group: {
         │   │   │         _id: null,
         │   │   │         orders: { $sum: 1 },
         │   │   │         revenue: { $sum: "$total" }
         │   │   │     }}
         │   │   │   ]
         │   │   └── Return: { orders, revenue }
         │   │
         │   ├── Query items sold today:
         │   │   ├── Collection: product_items
         │   │   ├── Count: {
         │   │   │     shop_id: shop._id,
         │   │   │     DATE(updated_at) = TODAY,
         │   │   │     is_sold: true
         │   │   │   }
         │   │   └── Return: items_sold
         │   │
         │   ├── Query new reviews today:
         │   │   ├── Collection: reviews
         │   │   ├── Count: {
         │   │   │     shop_id: shop._id,
         │   │   │     DATE(created_at) = TODAY,
         │   │   │     status: "approved"
         │   │   │   }
         │   │   └── Return: new_reviews
         │   │
         │   ├── Query disputes today:
         │   │   ├── Collection: disputes
         │   │   ├── Aggregate:
         │   │   │   [
         │   │   │     { $match: {
         │   │   │         shop_id: shop._id,
         │   │   │         DATE(created_at) = TODAY
         │   │   │     }},
         │   │   │     { $group: {
         │   │   │         _id: null,
         │   │   │         disputes: { $sum: 1 },
         │   │   │         pending: {
         │   │   │           $sum: { $cond: [{ $eq: ["$status", "pending"] }, 1, 0] }
         │   │   │         }
         │   │   │     }}
         │   │   │   ]
         │   │   └── Return: { disputes, pending }
         │   │
         │   ├── Query top 3 products today:
         │   │   ├── Collection: product_items
         │   │   ├── Aggregate:
         │   │   │   [
         │   │   │     { $match: {
         │   │   │         shop_id: shop._id,
         │   │   │         DATE(updated_at) = TODAY,
         │   │   │         is_sold: true
         │   │   │     }},
         │   │   │     { $lookup: {
         │   │   │         from: "products",
         │   │   │         localField: "product_id",
         │   │   │         foreignField: "_id",
         │   │   │         as: "product"
         │   │   │     }},
         │   │   │     { $unwind: "$product" },
         │   │   │     { $group: {
         │   │   │         _id: "$product_id",
         │   │   │         product_name: { $first: "$product.name" },
         │   │   │         sold: { $sum: 1 }
         │   │   │     }},
         │   │   │     { $sort: { sold: -1 }},
         │   │   │     { $limit: 3 }
         │   │   │   ]
         │   │   └── Return: top_products (Array)
         │   │
         │   └── Compile summary_data object
         │
         └── summaries (Array)
         │
         ▼
[B4] Filter shops with activity
         │
         ├── Loop: for summary in summaries
         │   │
         │   ├── summary.orders > 0 OR summary.items_sold > 0
         │   │   ──► Add to send_list
         │   │
         │   └── Skip shop if no activity
         │
         └── send_list (Array)
         │
         ▼
[B5] Send daily summary notifications (direct calls)
         │
         ├── Loop: for summary in send_list
         │   │
         │   ├── Gọi telegram_service.notify_daily_summary() (direct):
         │   │   ├── chat_id: shop.telegram_chat_id
         │   │   ├── date: TODAY
         │   │   ├── revenue: summary.revenue
         │   │   ├── orders: summary.orders
         │   │   ├── items_sold: summary.items_sold
         │   │   │
         │   ├── On success: Continue
         │   ├── On error: Log error, continue next
         │   │
         │   └── Continue
         │
         └── Continue sang [B6]
         │
         ▼
[B6] Log job execution
         │
         ├── Insert MongoDB:
         │   ├── Collection: cron_job_log
         │   ├── Document:
         │   │   {
         │   │     job_name: "daily_summary_report",
         │   │     shops_processed: active_shops.length,
         │   │     notifications_sent: send_list.length,
         │   │     started_at: start_time,
         │   │     completed_at: DateTime,
         │   │     status: "success",
         │   │     duration_ms: duration
         │   │     }
         │
         ├── Log: "Daily summary completed: {send_list.length} notifications sent"
         │
         └── END

---

### 13.3 Edge Cases & Error Handling

| Case | Condition | Handling | User Message |
|------|-----------|----------|--------------|
| No active shops | empty array | Return early | - |
| No activity | orders = 0 AND items_sold = 0 | Skip shop | - |
| Query timeout | MongoDB slow query | Log + Retry | - |
| Telegram fail | API error | Log only | - |
| Log fail | MongoDB error | Log to console | - |

---

# PHẦN 6: TỔNG HỢP TẤT CẢ BIẾN

## Biến Thực Thể Shop

| Biến | Kiểu | Bắt buộc | Mặc định | Mô tả |
|-----|------|----------|----------|-------|
| id | ObjectId | Auto | - | Primary key |
| vendor_id | ObjectId | Yes | - | FK → users |
| shop_name | String | Yes | - | Tên hiển thị (3-50) |
| shop_slug | String | Yes | - | URL slug (unique, 3-60) |
| description | String | Yes | - | Mô tả shop (max 500) |
| logo | String | Yes | - | Đường dẫn file logo |
| banner | String | No | null | Đường dẫn file banner |
| telegram_username | String | Yes | - | @username format |
| telegram_chat_id | i64 | No | null | Telegram chat ID |
| telegram_verified | Boolean | Yes | false | Đã xác nhận Telegram |
| warranty_policy | String | No | null | Chính sách bảo hành |
| refund_policy | String | No | null | Chính sách hoàn tiền |
| support_hours | String | No | null | Giờ hỗ trợ |
| status | Enum | Yes | active | active/inactive/suspended |
| rating | Decimal | Yes | 0.00 | Đánh giá trung bình (0-5) |
| total_reviews | Integer | Yes | 0 | Số lượng đánh giá |
| total_sales | Integer | Yes | 0 | Số lượng đơn hàng |
| total_products | Integer | Yes | 0 | Số lượng sản phẩm |
| commission_rate | Decimal | Yes | 10.00 | Phí sàn (%) |
| level | Enum | Yes | new | new/silver/gold/diamond/partner |
| suspended_reason | String | No | null | Lý do đình chỉ |
| suspended_until | DateTime | No | null | Hết hạn đình chỉ |
| view_count | Integer | Yes | 0 | Lượt xem trang |
| created_at | DateTime | Auto | NOW() | Thời gian tạo |
| updated_at | DateTime | Auto | NOW() | Cập nhật lần cuối |

---

## Tất Cả Điều Kiện Validation

| Điều kiện | Kiểm tra | Hành động | Message |
|----------|----------|------------|---------|
| shop_exists | shop_id found trong DB | Redirect | - |
| shop_slug_unique | Không match trong shops table | Return error | "Slug đã được sử dụng" |
| shop_name_valid | 3-50 chars | Validate | "Tên phải 3-50 ký tự" |
| telegram_format | Bắt đầu bằng @ | Validate | "Phải bắt đầu bằng @" |
| telegram_verified | chat_id not null | Validate | "Chưa xác nhận Telegram" |
| logo_uploaded | File exists và valid | Validate | "Vui lòng upload logo" |
| shop_active | status = "active" | Validate | "Shop không hoạt động" |
| vendor_authorized | user.role = "vendor" | 403 | "Không có quyền truy cập" |
| admin_authorized | user.role = "admin" | 403 | "Không có quyền truy cập" |

---

## Tất Cả Business Rules

| Điều kiện | Kiểm tra | Hành động |
|----------|----------|------------|
| level_upgrade | metrics meet threshold | Auto upgrade |
| telegram_notify | chat_id exists AND verified | Send notification |
| cache_hit | Redis key exists | Return cached |
| low_stock | quantity < 10 | Alert vendor |
| sold_out | quantity = 0 | Alert vendor |
| pending_disputes | count > 0 | Show alert |
| slug_changed | old_slug != new_slug | Create redirect 301 |
| partner_manual | Admin action only | Manual upgrade |
| shop_suspended | status = "suspended" | Hide products |

---

# KẾT THÚC TÀI LIỆU

Tài liệu này bao gồm:
- **13 flows hoàn chỉnh** cho Vendor, Buyer, Admin, và System
- **Tất cả biến số** với kiểu dữ liệu, nguồn, và mô tả
- **Tất cả điều kiện** với validation rules
- **Decision trees chi tiết** với mọi nhánh
- **Edge cases tables** cho mỗi flow
- **UI mockups** trong ASCII art format
- **Database queries** cho từng thao tác
- **Error handling** và fallback logic

**Telegram Integration:**
- Sử dụng Rust library (teloxide/frankenstein)
- Gọi trực tiếp `send_message()` trong code
- Không cần Queue Worker
- Xử lý lỗi như log (không ảnh hưởng business logic)

**Tài liệu tham khảo:**
- [V1 Shop Management](../v1/03-shop-management.md)
- [V2 Shop System Design](./00-shop-system-design.md)
- [V2 Wallet System](./wallet/00-wallet-design.md)
- [Write Docs Skill](../../../../.claude/skills/write-docs.md)
