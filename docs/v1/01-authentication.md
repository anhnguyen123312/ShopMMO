# Chức năng Xác thực (Authentication)

## Tổng quan

Hệ thống xác thực của TaphoaMMO bao gồm các chức năng: đăng ký tài khoản, đăng nhập, quên mật khẩu, đổi mật khẩu, và xác thực hai yếu tố (2FA). Hệ thống được thiết kế để đảm bảo an toàn cao nhất cho người dùng trong môi trường giao dịch sản phẩm số.

---

## 1. Đăng ký tài khoản (Registration)

### Mục đích
Cho phép người dùng mới tạo tài khoản để tham gia mua bán trên sàn.

### Điều kiện tiên quyết
- Chưa có tài khoản trên hệ thống
- Có email hợp lệ và chưa được sử dụng
- Không bị block IP hoặc fingerprint

### Dữ liệu đầu vào
| Trường | Bắt buộc | Mô tả | Validation |
|--------|----------|-------|------------|
| username | Có | Tên đăng nhập | 4-30 ký tự, chỉ chữ và số, không trùng |
| email | Có | Địa chỉ email | Email hợp lệ, không trùng |
| password | Có | Mật khẩu | Tối thiểu 8 ký tự, có chữ hoa, chữ thường, số |
| password_confirmation | Có | Xác nhận mật khẩu | Phải khớp với password |
| captcha | Có | Mã captcha | Phải đúng với captcha hiển thị |
| agree_terms | Có | Đồng ý điều khoản | Phải check |

### Flow chi tiết

```
┌─────────────────────────────────────────────────────────────────┐
│                    FLOW ĐĂNG KÝ TÀI KHOẢN                       │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Người dùng truy cập trang đăng ký
         │
         ▼
[Bước 2] Hệ thống hiển thị form đăng ký + Captcha
         │
         ▼
[Bước 3] Người dùng điền thông tin và submit
         │
         ▼
[Bước 4] Hệ thống validate dữ liệu
         │
         ├── Lỗi validation ──► Hiển thị lỗi, quay lại form
         │
         ▼
[Bước 5] Kiểm tra Captcha
         │
         ├── Sai Captcha ──► Hiển thị lỗi, generate captcha mới
         │
         ▼
[Bước 6] Kiểm tra username/email đã tồn tại
         │
         ├── Đã tồn tại ──► Thông báo lỗi tương ứng
         │
         ▼
[Bước 7] Kiểm tra IP/fingerprint có bị block
         │
         ├── Bị block ──► Từ chối đăng ký
         │
         ▼
[Bước 8] Hash password với bcrypt
         │
         ▼
[Bước 9] Tạo bản ghi User mới trong database
         - status: pending_verification
         - role: buyer (mặc định)
         - wallet_balance: 0
         │
         ▼
[Bước 10] Tạo Wallet liên kết với User
          │
          ▼
[Bước 11] Generate verification token
          │
          ▼
[Bước 12] Gửi email xác thực (nếu bật tính năng)
          │
          ▼
[Bước 13] Redirect đến trang thông báo đăng ký thành công
          │
          ▼
[Bước 14] Người dùng click link trong email
          │
          ▼
[Bước 15] Hệ thống verify token
          │
          ├── Token không hợp lệ/hết hạn ──► Thông báo lỗi
          │
          ▼
[Bước 16] Cập nhật status: active
          │
          ▼
[Bước 17] Auto login và redirect dashboard
```

### Xử lý lỗi
| Lỗi | Xử lý |
|-----|-------|
| Username đã tồn tại | Hiển thị "Tên đăng nhập đã được sử dụng" |
| Email đã tồn tại | Hiển thị "Email đã được đăng ký" |
| Captcha sai | Refresh captcha, hiển thị lỗi |
| IP bị block | Hiển thị "Không thể đăng ký từ địa chỉ này" |
| Gửi email thất bại | Log lỗi, cho phép gửi lại sau |

### Ghi chú bảo mật
- Rate limit: Tối đa 5 lần đăng ký/IP/giờ
- Không cho phép bypass captcha bằng addon/tool
- Log tất cả attempt đăng ký để phát hiện abuse

---

## 2. Đăng nhập (Login)

### Mục đích
Cho phép người dùng đã có tài khoản truy cập vào hệ thống.

### Điều kiện tiên quyết
- Có tài khoản hợp lệ
- Tài khoản đang active (không bị banned/suspended)

### Dữ liệu đầu vào
| Trường | Bắt buộc | Mô tả |
|--------|----------|-------|
| login | Có | Username hoặc Email |
| password | Có | Mật khẩu |
| remember | Không | Ghi nhớ đăng nhập |
| captcha | Có (sau 3 lần sai) | Mã captcha |

### Flow chi tiết

```
┌─────────────────────────────────────────────────────────────────┐
│                      FLOW ĐĂNG NHẬP                             │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Người dùng truy cập trang đăng nhập
         │
         ▼
[Bước 2] Hệ thống kiểm tra session hiện tại
         │
         ├── Đã login ──► Redirect dashboard
         │
         ▼
[Bước 3] Hiển thị form đăng nhập
         - Kiểm tra số lần login fail của IP
         - Nếu >= 3 lần: hiển thị Captcha
         │
         ▼
[Bước 4] Người dùng nhập thông tin và submit
         │
         ▼
[Bước 5] Validate captcha (nếu có)
         │
         ├── Sai Captcha ──► Tăng fail count, hiển thị lỗi
         │
         ▼
[Bước 6] Tìm user theo username hoặc email
         │
         ├── Không tìm thấy ──► Tăng fail count
         │                      Thông báo "Sai thông tin đăng nhập"
         │
         ▼
[Bước 7] Kiểm tra trạng thái tài khoản
         │
         ├── Banned ──► "Tài khoản đã bị khóa vĩnh viễn"
         ├── Suspended ──► "Tài khoản tạm khóa đến [date]"
         ├── Pending ──► "Vui lòng xác thực email"
         │
         ▼
[Bước 8] Verify password với hash trong DB
         │
         ├── Sai password ──► Tăng fail count
         │                    Thông báo "Sai thông tin đăng nhập"
         │
         ▼
[Bước 9] Kiểm tra 2FA có được bật không
         │
         ├── Có 2FA ──► Chuyển đến [Flow 2FA Verification]
         │
         ▼
[Bước 10] Reset fail count của IP
          │
          ▼
[Bước 11] Tạo session mới
          - Lưu user_id, role, permissions
          - Set remember token nếu check "Ghi nhớ"
          │
          ▼
[Bước 12] Ghi log đăng nhập
          - IP address
          - User agent
          - Timestamp
          - Geolocation (nếu có)
          │
          ▼
[Bước 13] Kiểm tra có intended URL không
          │
          ├── Có ──► Redirect đến intended URL
          │
          ▼
[Bước 14] Redirect theo role
          - Admin: /admin/dashboard
          - Vendor: /vendor/dashboard  
          - Buyer: /dashboard hoặc /
```

### Cơ chế chống brute-force
```
┌─────────────────────────────────────────────────────────────────┐
│               RATE LIMITING ĐĂNG NHẬP                           │
└─────────────────────────────────────────────────────────────────┘

Fail count theo IP:
├── 1-2 lần: Cho phép thử tiếp
├── 3-5 lần: Yêu cầu Captcha
├── 6-10 lần: Captcha + delay 30 giây giữa các lần
├── 11-20 lần: Block 15 phút
├── 21+ lần: Block 1 giờ, thông báo admin

Fail count theo Account:
├── 1-4 lần: Cho phép thử tiếp
├── 5-9 lần: Yêu cầu Captcha
├── 10+ lần: Lock account 30 phút
                Gửi email cảnh báo cho user
```

---

## 3. Xác thực hai yếu tố (2FA)

### Mục đích
Tăng cường bảo mật bằng cách yêu cầu mã OTP từ ứng dụng authenticator.

### Các trạng thái 2FA
| Trạng thái | Mô tả |
|------------|-------|
| disabled | Chưa bật 2FA |
| pending_setup | Đang trong quá trình thiết lập |
| enabled | Đã bật và đang hoạt động |

### Flow Bật 2FA

```
┌─────────────────────────────────────────────────────────────────┐
│                      FLOW BẬT 2FA                               │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] User vào Settings > Bảo mật > Bật 2FA
         │
         ▼
[Bước 2] Hệ thống yêu cầu nhập mật khẩu xác nhận
         │
         ├── Sai mật khẩu ──► Thông báo lỗi
         │
         ▼
[Bước 3] Generate secret key (32 ký tự base32)
         │
         ▼
[Bước 4] Tạo provisioning URI cho QR code
         Format: otpauth://totp/TaphoaMMO:{username}?secret={secret}&issuer=TaphoaMMO
         │
         ▼
[Bước 5] Hiển thị:
         - QR Code để scan
         - Secret key dạng text (cho nhập thủ công)
         - Hướng dẫn sử dụng Google Authenticator/Authy
         │
         ▼
[Bước 6] User scan QR hoặc nhập secret vào app
         │
         ▼
[Bước 7] User nhập mã OTP từ app để xác nhận
         │
         ▼
[Bước 8] Hệ thống verify OTP
         │
         ├── Sai OTP ──► Cho thử lại (tối đa 3 lần)
         │
         ▼
[Bước 9] Generate 10 backup codes
         │
         ▼
[Bước 10] Lưu vào database:
          - two_factor_secret: encrypted secret
          - two_factor_enabled: true
          - two_factor_confirmed_at: timestamp
          - backup_codes: encrypted JSON array
          │
          ▼
[Bước 11] Hiển thị backup codes
          - Yêu cầu user lưu lại
          - Mỗi code chỉ dùng được 1 lần
          │
          ▼
[Bước 12] Gửi email thông báo đã bật 2FA
```

### Flow Xác thực 2FA khi đăng nhập

```
┌─────────────────────────────────────────────────────────────────┐
│                  FLOW XÁC THỰC 2FA                              │
└─────────────────────────────────────────────────────────────────┘

[Từ Login Flow Bước 9 - User có 2FA enabled]
         │
         ▼
[Bước 1] Tạo temporary session với flag: requires_2fa
         │
         ▼
[Bước 2] Redirect đến trang nhập mã 2FA
         │
         ▼
[Bước 3] Hiển thị form nhập OTP
         - Input 6 số
         - Link "Dùng backup code"
         - Link đến tool /2FA.html (nếu cần generate)
         │
         ▼
[Bước 4] User nhập mã OTP
         │
         ▼
[Bước 5] Kiểm tra loại mã
         │
         ├── Backup code (8 ký tự) ──► [Flow Backup Code]
         │
         ▼
[Bước 6] Verify TOTP
         - Lấy secret từ DB
         - Decrypt secret
         - Validate với time window ±30 giây
         │
         ├── Sai OTP ──► Tăng fail count
         │               Còn < 5 lần: cho thử tiếp
         │               >= 5 lần: khóa 15 phút
         │
         ▼
[Bước 7] Xóa temporary session
         │
         ▼
[Bước 8] Tạo full session
         │
         ▼
[Bước 9] Redirect đến dashboard/intended URL

─────────────────────────────────────────────────────────────────

[Flow Backup Code]
         │
         ▼
[B1] Kiểm tra code trong danh sách backup_codes
     │
     ├── Không tìm thấy ──► "Backup code không hợp lệ"
     │
     ▼
[B2] Kiểm tra code đã được sử dụng chưa
     │
     ├── Đã dùng ──► "Backup code đã được sử dụng"
     │
     ▼
[B3] Đánh dấu code đã sử dụng
     │
     ▼
[B4] Kiểm tra số code còn lại
     │
     ├── Còn <= 2 codes ──► Cảnh báo user nên tạo codes mới
     │
     ▼
[B5] Tiếp tục từ Bước 7 của Flow chính
```

---

## 4. Quên mật khẩu (Forgot Password)

### Flow chi tiết

```
┌─────────────────────────────────────────────────────────────────┐
│                  FLOW QUÊN MẬT KHẨU                             │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] User click "Quên mật khẩu" tại trang login
         │
         ▼
[Bước 2] Hiển thị form nhập email
         │
         ▼
[Bước 3] User nhập email và submit
         │
         ▼
[Bước 4] Validate email format
         │
         ├── Email không hợp lệ ──► Hiển thị lỗi
         │
         ▼
[Bước 5] Kiểm tra rate limit
         - Tối đa 3 request/email/giờ
         - Tối đa 10 request/IP/giờ
         │
         ├── Vượt limit ──► "Vui lòng thử lại sau"
         │
         ▼
[Bước 6] Tìm user theo email
         │
         ├── Không tìm thấy ──► Vẫn hiển thị "Đã gửi email"
         │                      (Để tránh leak thông tin)
         │
         ▼
[Bước 7] Generate reset token
         - Token ngẫu nhiên 64 ký tự
         - Thời hạn: 1 giờ
         │
         ▼
[Bước 8] Lưu token vào bảng password_resets
         - email
         - token (hashed)
         - created_at
         │
         ▼
[Bước 9] Gửi email chứa link reset
         Link format: /password/reset/{token}?email={email}
         │
         ▼
[Bước 10] Hiển thị thông báo "Đã gửi email hướng dẫn"

─────────────────────────────────────────────────────────────────

[User click link trong email]
         │
         ▼
[Bước 11] Verify token
          - Kiểm tra token tồn tại
          - Kiểm tra chưa hết hạn
          - Kiểm tra khớp với email
          │
          ├── Token không hợp lệ ──► "Link đã hết hạn hoặc không hợp lệ"
          │
          ▼
[Bước 12] Hiển thị form đặt mật khẩu mới
          - password
          - password_confirmation
          │
          ▼
[Bước 13] User nhập mật khẩu mới và submit
          │
          ▼
[Bước 14] Validate password
          │
          ├── Không đạt yêu cầu ──► Hiển thị lỗi
          │
          ▼
[Bước 15] Hash password mới
          │
          ▼
[Bước 16] Cập nhật password trong bảng users
          │
          ▼
[Bước 17] Xóa token khỏi bảng password_resets
          │
          ▼
[Bước 18] Invalidate tất cả session hiện tại của user
          │
          ▼
[Bước 19] Gửi email thông báo đã đổi mật khẩu
          │
          ▼
[Bước 20] Redirect đến trang login với thông báo thành công
```

---

## 5. Đổi mật khẩu (Change Password)

### Flow chi tiết

```
┌─────────────────────────────────────────────────────────────────┐
│                   FLOW ĐỔI MẬT KHẨU                             │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] User đã đăng nhập, vào Settings > Đổi mật khẩu
         │
         ▼
[Bước 2] Hiển thị form:
         - current_password
         - new_password
         - new_password_confirmation
         │
         ▼
[Bước 3] User nhập thông tin và submit
         │
         ▼
[Bước 4] Verify current_password
         │
         ├── Sai mật khẩu ──► "Mật khẩu hiện tại không đúng"
         │
         ▼
[Bước 5] Validate new_password
         - Đủ độ mạnh
         - Không trùng với mật khẩu cũ
         │
         ├── Không đạt ──► Hiển thị lỗi cụ thể
         │
         ▼
[Bước 6] Kiểm tra 2FA (nếu đã bật)
         │
         ├── Có 2FA ──► Yêu cầu nhập OTP
         │
         ▼
[Bước 7] Hash new_password
         │
         ▼
[Bước 8] Cập nhật vào database
         │
         ▼
[Bước 9] Invalidate các session khác (giữ session hiện tại)
         │
         ▼
[Bước 10] Gửi email thông báo
          │
          ▼
[Bước 11] Hiển thị thông báo thành công
```

---

## 6. Đăng xuất (Logout)

### Flow chi tiết

```
┌─────────────────────────────────────────────────────────────────┐
│                     FLOW ĐĂNG XUẤT                              │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] User click "Đăng xuất"
         │
         ▼
[Bước 2] Xóa session hiện tại
         │
         ▼
[Bước 3] Xóa remember token (nếu có)
         │
         ▼
[Bước 4] Ghi log đăng xuất
         │
         ▼
[Bước 5] Redirect đến trang chủ hoặc login
```

### Đăng xuất tất cả thiết bị

```
[Bước 1] User chọn "Đăng xuất tất cả thiết bị"
         │
         ▼
[Bước 2] Yêu cầu nhập mật khẩu xác nhận
         │
         ▼
[Bước 3] Xóa tất cả sessions của user
         │
         ▼
[Bước 4] Regenerate remember token
         │
         ▼
[Bước 5] Gửi email thông báo
         │
         ▼
[Bước 6] Redirect đến login
```

---

## Database Schema liên quan

### Bảng users (các trường authentication)
| Column | Type | Mô tả |
|--------|------|-------|
| id | bigint | Primary key |
| username | varchar(30) | Unique |
| email | varchar(255) | Unique |
| password | varchar(255) | Bcrypt hash |
| email_verified_at | timestamp | Null nếu chưa verify |
| remember_token | varchar(100) | Cho "Remember me" |
| two_factor_secret | text | Encrypted TOTP secret |
| two_factor_enabled | boolean | Default false |
| two_factor_confirmed_at | timestamp | Khi hoàn tất setup |
| backup_codes | text | Encrypted JSON array |
| status | enum | active, pending, suspended, banned |
| last_login_at | timestamp | |
| last_login_ip | varchar(45) | |

### Bảng password_resets
| Column | Type | Mô tả |
|--------|------|-------|
| email | varchar(255) | Index |
| token | varchar(255) | Hashed |
| created_at | timestamp | |

### Bảng login_attempts
| Column | Type | Mô tả |
|--------|------|-------|
| id | bigint | Primary key |
| ip_address | varchar(45) | |
| user_id | bigint | Nullable |
| success | boolean | |
| user_agent | text | |
| created_at | timestamp | |

---

## Cấu hình bảo mật

### Password Requirements
- Minimum length: 8 ký tự
- Phải có: chữ hoa, chữ thường, số
- Không được trùng với 5 mật khẩu gần nhất (optional)

### Session Configuration
- Lifetime: 120 phút (không activity)
- Driver: database hoặc redis
- Secure cookie: true (production)
- Same-site: lax

### Rate Limits
| Action | Limit |
|--------|-------|
| Login attempts | 5/phút/IP |
| Password reset | 3/giờ/email |
| Registration | 5/giờ/IP |
| 2FA verification | 5/15 phút |
