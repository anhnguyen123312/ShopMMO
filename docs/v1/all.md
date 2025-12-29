

---

## Tính năng chính của hệ thống

### Hệ thống người dùng với 4 vai trò

| Vai trò | Quyền hạn chính |
|---------|-----------------|
| **Buyer** | Nạp tiền, mua sản phẩm, đánh giá, khiếu nại |
| **Vendor** | Tạo gian hàng, upload kho, quản lý đơn, rút tiền |
| **Reseller/CTV** | Bán lại sản phẩm shop khác, nhận hoa hồng |
| **Admin** | Quản lý toàn hệ thống, xử lý tranh chấp |

Hệ thống **xác thực 2FA tích hợp** là điểm đặc biệt - không chỉ bảo vệ tài khoản người dùng mà còn hỗ trợ sản phẩm bán có 2FA (Github, Facebook accounts). Website cung cấp tool `/2FA.html` để generate TOTP code từ chuỗi secret.

### Hệ thống sản phẩm số với auto-delivery

**Đặc thù kho hàng text-based**: Sản phẩm số được lưu dạng plain text với format `username|password|2fa_secret|email`. Khi đơn hàng hoàn thành, hệ thống **tự động xuất** item từ kho và gửi cho buyer.


**Hệ thống check trùng lặp** đảm bảo mỗi item chỉ bán **1 lần duy nhất** trên toàn platform - quan trọng cho tính minh bạch của marketplace.

### Hệ thống multi-vendor (gian hàng)

Mỗi vendor có dashboard riêng để quản lý:
- **Sản phẩm**: Tạo, chỉnh sửa, đặt giá, mô tả
- **Kho hàng**: Upload bulk text, check số lượng tồn kho
- **Đơn hàng**: Xem đơn đã bán, xử lý bảo hành
- **Tài chính**: Theo dõi doanh thu, rút tiền (sau 3 ngày pending)
- **Xếp hạng**: Dựa trên tỷ lệ khiếu nại, đánh giá khách hàng

**Tính năng Reseller** cho phép vendors enable reselling - các CTV có thể bán lại với chiết khấu thỏa thuận.

### Hệ thống thanh toán và tài chính

Đa dạng phương thức nạp tiền phục vụ thị trường Việt Nam:

| Phương thức | Chi tiết |
|-------------|----------|
| **Bank Transfer** | Auto-detect qua cú pháp, tối thiểu 30,000đ |
| **Momo** | Ví điện tử phổ biến nhất VN |
| **USDT TRC20** | Crypto cho khách quốc tế |
| **PayPal** | Tỷ giá 1 USD = 22,000 VND |

**Cơ chế giữ tiền 3 ngày** là escrow mechanism bảo vệ buyer - tiền chỉ release cho vendor sau 3 ngày không có tranh chấp. Đây là feature cốt lõi tạo niềm tin cho marketplace.

### Hệ thống pre-order thông minh

Khi sản phẩm hết hàng, buyer có thể đặt trước:
- Chọn thời gian chờ tối đa (1-7 ngày)
- Khi vendor restock → đơn tự động complete
- Hết thời gian → auto cancel và hoàn tiền
- Người đặt trước được ưu tiên lấy hàng

---

## Các flow hoạt động chi tiết

### Flow đăng ký và đăng nhập

```
1. User → /register
2. Nhập email, password, confirm password
3. Verify email (optional)
4. Setup profile: tên, social links, bank account
5. Enable 2FA (optional nhưng khuyến khích)

Login flow:
1. User → /login
2. Nhập email/username + password
3. Nếu enable 2FA → redirect /2fa/verify
4. Nhập TOTP code (có thể dùng /2FA.html tool)
5. Verify → issue session token → redirect dashboard
```

### Flow mua hàng hoàn chỉnh

```
1. Browse → /gian-hang/{shop} hoặc /danh-muc/{category}
2. Chọn sản phẩm → /gian-hang/{slug}_{id}
3. Check số lượng tồn kho (realtime)
4. Nhập số lượng, apply coupon (optional)
5. POST /order/create
   └─ Middleware CheckBalance verify số dư
6. Hệ thống:
   a. Lock stock items (prevent race condition)
   b. Deduct buyer wallet
   c. Create order + order_items
   d. Auto-delivery: xuất content từ kho
   e. Show delivered content cho buyer
   f. Create pending payout cho vendor (3 days hold)
7. Buyer nhận sản phẩm ngay lập tức
```

### Flow tạo và quản lý sản phẩm (Vendor)

```
1. Vendor → /vendor/products/create
2. Nhập: tên, mô tả, category, giá, enable reseller?
3. Upload kho hàng:
   - File txt bulk upload
   - Format: username|password|2fa|email (mỗi dòng 1 item)
4. Hệ thống check trùng với database toàn platform
5. Save product + product_items
6. Product live ngay (hoặc pending admin approval)

Quản lý kho:
- Dashboard hiển thị: tổng items, đã bán, còn lại
- Restock: upload thêm items bất kỳ lúc nào
- Pre-orders tự động fulfill khi restock
```

### Flow thanh toán và rút tiền

```
Nạp tiền (Bank Transfer):
1. User → /deposit
2. Chọn Bank Transfer, nhập số tiền
3. Hệ thống generate cú pháp: NAPTIEN_{user_id}
4. User chuyển khoản với cú pháp đúng
5. Webhook từ banking API detect giao dịch
6. Auto credit vào wallet

Rút tiền (Vendor):
1. Vendor → /vendor/payouts
2. Check "Số dư khả dụng" (sau 3 ngày hold)
3. Nhập số tiền rút
4. Submit → Admin approve → Bank transfer
```

### Flow xử lý khiếu nại

```
1. Buyer phát hiện lỗi (sản phẩm die, sai thông tin)
2. Tạo dispute trong 3 ngày (trước khi tiền release)
3. Vendor phản hồi: accept hoặc reject
4. Nếu không resolve → Admin can thiệp
5. Kết quả:
   - Refund buyer (full/partial)
   - Hoặc reject dispute → release tiền vendor
```

---


### Stack công nghệ phân tích

| Component | Technology |
|-----------|------------|
| **Backend** | Laravel 10/11, PHP 8.1+ |
| **Database** | MySQL 8.0 (hoặc MariaDB) |
| **Cache/Queue** | Redis |
| **Frontend** | Blade templates, vanilla JS hoặc Vue.js |
| **CSS** | Bootstrap 5 hoặc Tailwind CSS |
| **Build tool** | Vite (Laravel 10+) |
| **API Auth** | Laravel Sanctum |
| **2FA** | pragmarx/google2fa-laravel |
| **Roles/Permissions** | spatie/laravel-permission |
| **Payment** | Custom integration (VN Banks, Momo) |

### Các package quan trọng

- **spatie/laravel-permission**: Quản lý 4 roles (buyer, vendor, reseller, admin) với granular permissions
- **pragmarx/google2fa**: Generate và verify TOTP codes cho 2FA
- **laravel/sanctum**: API authentication cho vendor integration
- **intervention/image**: Resize/optimize product images
- **maatwebsite/excel**: Export reports, import bulk inventory

---

## Kiến trúc bảo mật

### Các layer bảo mật

1. **Authentication Layer**
   - Session-based auth với CSRF protection
   - Optional 2FA với TOTP
   - Rate limiting login attempts

2. **Authorization Layer**
   - Role-based access control (RBAC)
   - Middleware stack: `auth → verified → role:vendor`

3. **Transaction Security**
   - Pessimistic locking khi xuất kho (prevent overselling)
   - 3-day escrow hold
   - Duplicate check toàn platform

4. **API Security**
   - Sanctum token với expiration
   - IP whitelisting cho vendor API (optional)
   - Rate limiting per token

5. **Anti-Bot/Scraping**
   - 403 response cho requests thiếu proper headers
   - Captcha trên sensitive forms
   - Session fingerprinting

---

## So sánh với các Laravel marketplace khác

| Feature | TaphoaMMO | Bagisto | S-Cart |
|---------|-----------|---------|--------|
| **Multi-vendor** | ✅ | ✅ | ✅ |
| **Digital products** | ✅ (core) | ✅ | ❌ |
| **Auto-delivery** | ✅ | ❌ | ❌ |
| **Escrow system** | ✅ (3 days) | ❌ | ❌ |
| **Pre-order** | ✅ | ✅ | ❌ |
| **Reseller system** | ✅ | ❌ | ❌ |
| **2FA for products** | ✅ | ❌ | ❌ |
| **VN Payment** | ✅ | ❌ | ✅ |

TaphoaMMO có kiến trúc **chuyên biệt cho digital goods** với các tính năng escrow, auto-delivery, và duplicate checking không có trong các e-commerce platform thông thường.

---

## Kết luận

TaphoaMMO (taphoammo.net) là một **custom-built Laravel digital marketplace** với kiến trúc phức tạp phục vụ đặc thù thị trường MMO Việt Nam. Dù source code không public, phân tích reverse-engineer cho thấy hệ thống được xây dựng với:

- **Kiến trúc multi-tenant** cho vendor independence
- **Event-driven design** với queues xử lý auto-delivery
- **Strong transactional integrity** với pessimistic locking và escrow
- **Scalable API layer** cho vendor integrations
- **Robust security** với 2FA, rate limiting, và anti-bot measures

Với ~2.8M monthly visits và 90+ phút average session, đây là case study thành công cho Laravel trong xây dựng marketplace quy mô lớn tại Việt Nam. Developers muốn build tương tự nên tham khảo **Bagisto** (multi-vendor) kết hợp custom digital delivery module.