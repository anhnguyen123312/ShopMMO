# Chức năng Người dùng và Phân quyền (User & Roles)

## Tổng quan

Hệ thống quản lý người dùng của TaphoaMMO sử dụng mô hình RBAC (Role-Based Access Control) với 4 vai trò chính: Buyer, Vendor, Reseller và Admin. Mỗi vai trò có quyền hạn và giao diện riêng biệt.

---

## 1. Các vai trò trong hệ thống

### 1.1 Buyer (Người mua)
**Mô tả**: Vai trò mặc định khi đăng ký, chỉ có quyền mua hàng.

| Quyền | Mô tả |
|-------|-------|
| view_products | Xem tất cả sản phẩm |
| purchase_products | Mua sản phẩm |
| view_orders | Xem lịch sử đơn hàng của mình |
| create_dispute | Tạo khiếu nại |
| create_review | Đánh giá sản phẩm đã mua |
| manage_wallet | Nạp tiền, xem giao dịch |
| manage_profile | Cập nhật thông tin cá nhân |
| use_coupon | Sử dụng mã giảm giá |

### 1.2 Vendor (Người bán)
**Mô tả**: Chủ gian hàng, được nâng cấp từ Buyer sau khi đăng ký bán hàng.

| Quyền | Mô tả |
|-------|-------|
| *Tất cả quyền của Buyer* | |
| manage_shop | Quản lý thông tin gian hàng |
| manage_products | CRUD sản phẩm |
| manage_inventory | Upload/quản lý kho hàng |
| view_sales | Xem doanh số, thống kê |
| respond_dispute | Phản hồi khiếu nại |
| withdraw_funds | Rút tiền về tài khoản |
| create_coupon | Tạo mã giảm giá cho shop |
| enable_reseller | Bật/tắt cho phép resell |

### 1.3 Reseller (Cộng tác viên)
**Mô tả**: Bán lại sản phẩm của Vendor khác, hưởng hoa hồng.

| Quyền | Mô tả |
|-------|-------|
| *Tất cả quyền của Buyer* | |
| view_reseller_products | Xem sản phẩm được phép resell |
| resell_products | Bán lại với giá tùy chỉnh |
| view_commissions | Xem hoa hồng đã nhận |
| withdraw_commissions | Rút hoa hồng |

### 1.4 Admin (Quản trị viên)
**Mô tả**: Toàn quyền quản lý hệ thống.

| Quyền | Mô tả |
|-------|-------|
| *Tất cả quyền* | |
| manage_users | Quản lý tài khoản người dùng |
| manage_vendors | Duyệt/từ chối vendor |
| resolve_disputes | Xử lý khiếu nại cuối cùng |
| manage_categories | Quản lý danh mục |
| view_reports | Xem báo cáo tổng hợp |
| manage_settings | Cài đặt hệ thống |
| manage_transactions | Xem/xử lý giao dịch |
| ban_users | Khóa tài khoản |

---

## 2. Quản lý Profile người dùng

### 2.1 Xem và cập nhật thông tin

### Flow cập nhật Profile

```
┌─────────────────────────────────────────────────────────────────┐
│                  FLOW CẬP NHẬT PROFILE                          │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] User vào trang Profile/Settings
         │
         ▼
[Bước 2] Hệ thống load thông tin hiện tại từ DB
         │
         ▼
[Bước 3] Hiển thị form với các trường:
         - Avatar
         - Họ tên
         - Số điện thoại
         - Địa chỉ
         - Liên kết mạng xã hội (Facebook, Telegram, Zalo)
         - Thông tin ngân hàng (cho Vendor)
         │
         ▼
[Bước 4] User chỉnh sửa và submit
         │
         ▼
[Bước 5] Validate dữ liệu
         │
         ├── Lỗi validation ──► Hiển thị lỗi cụ thể
         │
         ▼
[Bước 6] Xử lý avatar (nếu có upload)
         - Validate: jpg, png, gif
         - Max size: 2MB
         - Resize về kích thước chuẩn
         - Xóa avatar cũ
         │
         ▼
[Bước 7] Cập nhật vào database
         │
         ▼
[Bước 8] Clear cache profile
         │
         ▼
[Bước 9] Hiển thị thông báo thành công
```

### Các trường thông tin Profile

| Trường | Loại | Bắt buộc | Validation |
|--------|------|----------|------------|
| avatar | File | Không | jpg/png/gif, max 2MB |
| full_name | String | Không | Max 100 ký tự |
| phone | String | Không | Số điện thoại VN hợp lệ |
| address | Text | Không | Max 500 ký tự |
| facebook_url | URL | Không | URL Facebook hợp lệ |
| telegram_username | String | Không | @ + 5-32 ký tự |
| zalo_phone | String | Không | Số điện thoại |
| bio | Text | Không | Max 1000 ký tự |

---

## 3. Đăng ký trở thành Vendor

### Điều kiện
- Đã có tài khoản Buyer active
- Chưa là Vendor
- Có đầy đủ thông tin cá nhân

### Flow đăng ký Vendor

```
┌─────────────────────────────────────────────────────────────────┐
│                FLOW ĐĂNG KÝ TRỞ THÀNH VENDOR                    │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Buyer vào mục "Đăng ký bán hàng"
         │
         ▼
[Bước 2] Hệ thống kiểm tra điều kiện
         │
         ├── Đã là Vendor ──► Redirect đến Vendor Dashboard
         ├── Thiếu thông tin ──► Yêu cầu bổ sung Profile
         │
         ▼
[Bước 3] Hiển thị form đăng ký gian hàng:
         
         Thông tin gian hàng:
         ├── shop_name: Tên gian hàng (unique)
         ├── shop_slug: URL gian hàng (auto generate từ name)
         ├── shop_description: Mô tả gian hàng
         ├── shop_logo: Logo
         ├── shop_banner: Banner (optional)
         
         Thông tin thanh toán:
         ├── bank_name: Tên ngân hàng
         ├── bank_account_number: Số tài khoản
         ├── bank_account_name: Tên chủ tài khoản
         
         Cam kết:
         ├── agree_vendor_terms: Đồng ý điều khoản vendor
         ├── agree_commission: Đồng ý mức phí hoa hồng
         │
         ▼
[Bước 4] User điền và submit
         │
         ▼
[Bước 5] Validate dữ liệu
         │
         ├── shop_name đã tồn tại ──► "Tên gian hàng đã được sử dụng"
         ├── Thông tin bank không hợp lệ ──► Báo lỗi cụ thể
         │
         ▼
[Bước 6] Tạo bản ghi Shop với status: pending_approval
         │
         ▼
[Bước 7] Liên kết Shop với User
         │
         ▼
[Bước 8] Gửi thông báo cho Admin để duyệt
         │
         ▼
[Bước 9] Gửi email xác nhận cho User
         │
         ▼
[Bước 10] Hiển thị thông báo "Đang chờ duyệt"

─────────────────────────────────────────────────────────────────

[Admin duyệt đơn đăng ký]
         │
         ▼
[A1] Admin vào danh sách đơn đăng ký vendor
     │
     ▼
[A2] Xem chi tiết đơn đăng ký
     - Thông tin user
     - Thông tin gian hàng
     - Lịch sử giao dịch (nếu có)
     │
     ▼
[A3] Admin quyết định
     │
     ├── Approve ──► [Flow Approve]
     ├── Reject ──► [Flow Reject]
     │
     ▼
[Flow Approve]
     │
     ├── Cập nhật Shop status: active
     ├── Thêm role 'vendor' cho User
     ├── Gửi email thông báo được duyệt
     ├── Tạo notification trong hệ thống
     │
     ▼
[Flow Reject]
     │
     ├── Cập nhật Shop status: rejected
     ├── Lưu lý do từ chối
     ├── Gửi email thông báo bị từ chối kèm lý do
     ├── Cho phép đăng ký lại sau 7 ngày
```

---

## 4. Đăng ký trở thành Reseller

### Điều kiện
- Đã có tài khoản Buyer active
- Chưa là Reseller

### Flow đăng ký Reseller

```
┌─────────────────────────────────────────────────────────────────┐
│               FLOW ĐĂNG KÝ TRỞ THÀNH RESELLER                   │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Buyer vào mục "Đăng ký CTV"
         │
         ▼
[Bước 2] Hiển thị thông tin về chương trình Reseller
         - Mức hoa hồng
         - Điều khoản
         - Cách thức hoạt động
         │
         ▼
[Bước 3] Hiển thị form đăng ký:
         ├── marketing_channel: Kênh marketing (FB, Tele, Web...)
         ├── target_audience: Đối tượng khách hàng
         ├── agree_terms: Đồng ý điều khoản
         │
         ▼
[Bước 4] User submit
         │
         ▼
[Bước 5] Tạo bản ghi Reseller liên kết với User
         │
         ▼
[Bước 6] Thêm role 'reseller' cho User
         │
         ▼
[Bước 7] Generate mã CTV unique (VD: CTV_12345)
         │
         ▼
[Bước 8] Gửi email chào mừng với hướng dẫn
         │
         ▼
[Bước 9] Redirect đến Reseller Dashboard

Lưu ý: Reseller được tự động approve (không cần admin duyệt)
```

---

## 5. Quản lý Sessions và Thiết bị

### Xem danh sách phiên đăng nhập

```
┌─────────────────────────────────────────────────────────────────┐
│              FLOW XEM VÀ QUẢN LÝ SESSIONS                       │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] User vào Settings > Bảo mật > Phiên đăng nhập
         │
         ▼
[Bước 2] Query tất cả sessions của user từ bảng sessions
         │
         ▼
[Bước 3] Hiển thị danh sách với thông tin:
         
         Cho mỗi session:
         ├── IP Address
         ├── User Agent (parse thành device + browser)
         ├── Location (từ IP)
         ├── Last Activity
         ├── Current session marker (nếu là session đang dùng)
         ├── Nút "Đăng xuất" (cho session khác)
         │
         ▼
[Bước 4] User click "Đăng xuất" trên một session
         │
         ▼
[Bước 5] Xóa session đó khỏi database
         │
         ▼
[Bước 6] Refresh danh sách
```

### Đăng xuất tất cả thiết bị khác

```
[Bước 1] User click "Đăng xuất tất cả thiết bị khác"
         │
         ▼
[Bước 2] Yêu cầu nhập mật khẩu xác nhận
         │
         ├── Sai mật khẩu ──► Báo lỗi
         │
         ▼
[Bước 3] Xóa tất cả sessions trừ session hiện tại
         │
         ▼
[Bước 4] Gửi email thông báo
         │
         ▼
[Bước 5] Hiển thị thông báo thành công
```

---

## 6. Quản lý User (Admin)

### 6.1 Danh sách Users

```
┌─────────────────────────────────────────────────────────────────┐
│                 FLOW QUẢN LÝ USERS (ADMIN)                      │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Admin vào Admin Panel > Users
         │
         ▼
[Bước 2] Hiển thị danh sách với các cột:
         - ID
         - Username
         - Email
         - Role(s)
         - Status
         - Balance
         - Ngày đăng ký
         - Lần login cuối
         │
         ▼
[Bước 3] Các tính năng filter/search:
         - Search by username/email
         - Filter by role
         - Filter by status
         - Filter by date range
         - Sort by column
```

### 6.2 Chi tiết và chỉnh sửa User

```
[Bước 1] Admin click vào user cần xem
         │
         ▼
[Bước 2] Hiển thị chi tiết:
         
         Tab Thông tin chung:
         ├── Profile đầy đủ
         ├── Roles hiện tại
         ├── Số dư ví
         
         Tab Hoạt động:
         ├── Lịch sử đăng nhập
         ├── Lịch sử giao dịch
         ├── Đơn hàng gần đây
         
         Tab Shop (nếu là Vendor):
         ├── Thông tin gian hàng
         ├── Sản phẩm
         ├── Doanh số
         
         Tab Khiếu nại:
         ├── Các dispute liên quan
         │
         ▼
[Bước 3] Admin có thể:
         - Chỉnh sửa thông tin
         - Thay đổi role
         - Reset mật khẩu
         - Điều chỉnh số dư (với lý do)
         - Suspend/Ban account
```

### 6.3 Ban/Suspend User

```
┌─────────────────────────────────────────────────────────────────┐
│                    FLOW BAN/SUSPEND USER                        │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Admin chọn action Ban hoặc Suspend
         │
         ▼
[Bước 2] Hiển thị form:
         
         Suspend:
         ├── Thời gian suspend (1 ngày - 1 năm)
         ├── Lý do
         
         Ban:
         ├── Lý do
         ├── Có khóa IP không
         │
         ▼
[Bước 3] Admin điền và xác nhận
         │
         ▼
[Bước 4] Cập nhật user status
         │
         ▼
[Bước 5] Invalidate tất cả sessions của user
         │
         ▼
[Bước 6] Nếu là Vendor:
         - Đánh dấu shop inactive
         - Ẩn tất cả sản phẩm
         - Giữ nguyên đơn hàng pending (chờ xử lý riêng)
         │
         ▼
[Bước 7] Gửi email thông báo cho user
         │
         ▼
[Bước 8] Ghi log action của admin

─────────────────────────────────────────────────────────────────

[Xử lý khi user bị suspend login]
         │
         ▼
[S1] User cố login
     │
     ▼
[S2] Kiểm tra status = suspended
     │
     ▼
[S3] Kiểm tra suspended_until
     │
     ├── Đã hết hạn ──► Auto unban, cho login
     │
     ▼
[S4] Hiển thị thông báo:
     "Tài khoản tạm khóa đến [date]. Lý do: [reason]"
```

---

## 7. Nâng cấp/Thay đổi Role

### Flow thay đổi Role (Admin)

```
┌─────────────────────────────────────────────────────────────────┐
│                   FLOW THAY ĐỔI ROLE                            │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Admin vào chi tiết user
         │
         ▼
[Bước 2] Click "Manage Roles"
         │
         ▼
[Bước 3] Hiển thị danh sách roles:
         - Checkbox cho mỗi role
         - Roles hiện tại đã check
         │
         ▼
[Bước 4] Admin check/uncheck và submit
         │
         ▼
[Bước 5] Validate:
         - Không thể xóa role cuối cùng
         - Admin role cần xác nhận đặc biệt
         │
         ▼
[Bước 6] Sync roles trong database
         │
         ▼
[Bước 7] Clear permission cache
         │
         ▼
[Bước 8] Ghi log
         │
         ▼
[Bước 9] Gửi email thông báo cho user (nếu role thay đổi đáng kể)
```

---

## 8. Xác thực và Authorization

### Middleware Chain cho các routes

```
┌─────────────────────────────────────────────────────────────────┐
│                    MIDDLEWARE CHAIN                             │
└─────────────────────────────────────────────────────────────────┘

Public Routes (không cần auth):
├── Trang chủ
├── Danh mục sản phẩm
├── Chi tiết sản phẩm
├── Login/Register
└── Forgot Password

Auth Routes (cần đăng nhập):
├── auth ─► Dashboard
├── auth ─► Profile
├── auth ─► Orders
├── auth ─► Wallet
└── auth ─► Cart/Checkout

Vendor Routes:
├── auth ─► vendor ─► Shop Settings
├── auth ─► vendor ─► Products
├── auth ─► vendor ─► Inventory
├── auth ─► vendor ─► Sales
└── auth ─► vendor ─► Payouts

Admin Routes:
├── auth ─► admin ─► Dashboard
├── auth ─► admin ─► Users
├── auth ─► admin ─► Vendors
├── auth ─► admin ─► Disputes
├── auth ─► admin ─► Reports
└── auth ─► admin ─► Settings
```

### Logic kiểm tra quyền

```
Khi user truy cập route được bảo vệ:
         │
         ▼
[1] Check Authentication
    │
    ├── Chưa login ──► Redirect /login với intended URL
    │
    ▼
[2] Check 2FA (nếu enabled)
    │
    ├── Chưa verify 2FA ──► Redirect /2fa/verify
    │
    ▼
[3] Check Account Status
    │
    ├── Banned ──► Logout, hiển thị thông báo
    ├── Suspended ──► Hiển thị thông báo với thời hạn
    │
    ▼
[4] Check Role
    │
    ├── Không có role phù hợp ──► 403 Forbidden
    │
    ▼
[5] Check Permission (nếu có)
    │
    ├── Không có permission ──► 403 Forbidden
    │
    ▼
[6] Allow Request
```

---

## Database Schema

### Bảng users
| Column | Type | Mô tả |
|--------|------|-------|
| id | bigint | PK |
| username | varchar(30) | Unique |
| email | varchar(255) | Unique |
| password | varchar(255) | |
| full_name | varchar(100) | |
| phone | varchar(20) | |
| avatar | varchar(255) | Path to avatar |
| status | enum | active/pending/suspended/banned |
| suspended_until | timestamp | Nullable |
| suspend_reason | text | Nullable |
| email_verified_at | timestamp | |
| created_at | timestamp | |
| updated_at | timestamp | |

### Bảng roles
| Column | Type | Mô tả |
|--------|------|-------|
| id | bigint | PK |
| name | varchar(50) | buyer/vendor/reseller/admin |
| display_name | varchar(100) | |
| description | text | |

### Bảng user_roles (pivot)
| Column | Type | Mô tả |
|--------|------|-------|
| user_id | bigint | FK |
| role_id | bigint | FK |

### Bảng permissions
| Column | Type | Mô tả |
|--------|------|-------|
| id | bigint | PK |
| name | varchar(100) | view_products, create_dispute... |
| description | text | |

### Bảng role_permissions (pivot)
| Column | Type | Mô tả |
|--------|------|-------|
| role_id | bigint | FK |
| permission_id | bigint | FK |
