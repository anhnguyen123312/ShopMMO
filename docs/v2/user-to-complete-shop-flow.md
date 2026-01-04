# Flow Từ Tạo User Đến Shop Hoàn Thiện - P2PMMO V2

## Tổng quan

Document này mô tả flow hoàn chỉnh từ khi user đăng ký tài khoản đến khi shop hoàn thiện và sẵn sàng bán hàng.

**Flow chính:** `REGISTER → BUYER → CREATE SHOP → AUTO VENDOR → VERIFY TELEGRAM → COMPLETE`

---

## PHẦN 1: ĐĂNG KÝ TÀI KHOẢN

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         PHẦN 1: ĐĂNG KÝ TÀI KHOẢN                           │
└─────────────────────────────────────────────────────────────────────────────┘

                        ┌──────────────────┐
                        │  Truy cập /register  │
                        └────────┬─────────┘
                                 │
                                 ▼
                        ┌──────────────────┐
                        │ Hiển thị form      │
                        │ • username         │
                        │ • email            │
                        │ • password         │
                        │ • captcha          │
                        └────────┬─────────┘
                                 │
                                 ▼
                        ┌──────────────────┐
                        │ Validate input    │
                        └────────┬─────────┘
                                 │
                    ┌────────────┴────────────┐
                    │ Lỗi?                     │
                    ├────────────┬────────────┤
                    │ Có        │ Không      │
                    ▼           ▼            ▼
              Show error    ┌──────────────────┐
                            │ Check captcha    │
                            └────────┬─────────┘
                                     │
                        ┌────────────┴────────────┐
                        │ Sai?                     │
                        ├────────────┬────────────┤
                        │ Có        │ Không      │
                        ▼           ▼            ▼
                  Refresh    ┌──────────────────┐
                            │ Check trùng       │
                            │ username/email    │
                            └────────┬─────────┘
                                     │
                        ┌────────────┴────────────┐
                        │ Đã tồn tại?             │
                        ├────────────┬────────────┤
                        │ Có        │ Không      │
                        ▼           ▼            ▼
              Show error    ┌──────────────────┐
                                │ Hash password    │
                                │ (bcrypt)         │
                                └────────┬─────────┘
                                         │
                                         ▼
                                ┌──────────────────┐
                                │ CREATE USER      │
                                │ ┌──────────────┐ │
                                │ │ status:      │ │
                                │ │   pending_verification│
                                │ │ role: buyer  │ │
                                │ └──────────────┘ │
                                └────────┬─────────┘
                                         │
                              ┌──────────┴──────────┐
                              │                     │
                              ▼                     ▼
                    ┌──────────────────┐   ┌──────────────────┐
                    │ CREATE WALLET    │   │ Generate         │
                    │ (tự động)        │   │ verification     │
                    │ balance: 0       │   │ token            │
                    └──────────────────┘   └────────┬─────────┘
                                                       │
                              ┌────────────────────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │ Gửi email verify  │
                    │ (nếu bật)        │
                    └────────┬─────────┘
                             │
                             ▼
                    ┌──────────────────┐
                    │ Hiển thị:        │
                    │ "Đăng ký thành   │
                    │  công, kiểm tra  │
                    │  email"          │
                    └──────────────────┘

                          ┌────────────────────┐
                          │ User click link    │
                          │ trong email        │
                          └────────┬───────────┘
                                   │
                                   ▼
                          ┌──────────────────┐
                          │ Verify token     │
                          └────────┬─────────┘
                                   │
                      ┌────────────┴────────────┐
                      │ Valid?                   │
                      ├────────────┬────────────┤
                      │ Không     │ Có         │
                      ▼           ▼            ▼
                Show error    ┌──────────────────┐
                              │ UPDATE USER      │
                              │ status: active   │
                              └────────┬─────────┘
                                       │
                                       ▼
                              ┌──────────────────┐
                              │ Auto login       │
                              │ Redirect:        │
                              │ /dashboard       │
                              └──────────────────┘

                                    ┌─────────────────────┐
                                    │     END PHẦN 1      │
                                    │   User = Buyer      │
                                    │  Wallet = Ready     │
                                    └─────────────────────┘
```

### Điều kiện đăng ký

| Trường | Bắt buộc | Validation |
|-------|----------|------------|
| username | Có | 4-30 ký tự, chỉ chữ và số, không trùng |
| email | Có | Email hợp lệ, không trùng |
| password | Có | Tối thiểu 8 ký tự, có chữ hoa, chữ thường, số |
| password_confirmation | Có | Phải khớp với password |
| captcha | Có | Phải đúng với captcha hiển thị |
| agree_terms | Có | Phải check |

### Rate limiting

- Tối đa 5 lần đăng ký/IP/giờ
- Sau 3 lần fail: yêu cầu captcha
- Sau 10 lần fail: block 15 phút

---

## PHẦN 2: TẠO SHOP VENDOR

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         PHẦN 2: TẠO SHOP VENDOR                             │
└─────────────────────────────────────────────────────────────────────────────┘

                        ┌──────────────────┐
                        │ Buyer truy cập:  │
                        │ /vendor/dashboard│
                        └────────┬─────────┘
                                 │
                                 ▼
                        ┌──────────────────┐
                        │ Check shop exists │
                        └────────┬─────────┘
                                 │
                    ┌────────────┴────────────┐
                    │ Đã có shop?             │
                    ├────────────┬────────────┤
                    │ Có        │ Không      │
                    ▼           ▼            ▼
          Redirect to    ┌──────────────────┐
          /vendor/       │ Hiển thị Wizard  │
          shop/dashboard │ TẠO GIAN HÀNG    │
                        └────────┬─────────┘
                                 │
                    ┌────────────┴─────────────────────────────────────┐
                    │              BƯỚC 1: Thông tin cơ bản           │
                    │ • shop_name (3-50 chars)                         │
                    │ • shop_slug (auto-generate, unique)              │
                    │ • shop_description (max 500 chars)               │
                    └──────────────────────────────────────────────────┘
                                 │
                                 ▼
                        ┌──────────────────┐
                        │ Validate input    │
                        │ Validate slug     │
                        └────────┬─────────┘
                                 │
                    ┌────────────┴────────────┐
                    │ Slug đã tồn tại?        │
                    ├────────────┬────────────┤
                    │ Có        │ Không      │
                    ▼           ▼            ▼
              Show error    ┌──────────────────┐
                            │              BƯỚC 2: Hình ảnh          │
                            │ • shop_logo (REQUIRED, max 2MB)        │
                            │ • shop_banner (OPTIONAL, max 5MB)      │
                            └────────┬─────────┘
                                     │
                                     ▼
                            ┌──────────────────┐
                            │ Upload files     │
                            │ • Validate type  │
                            │ • Resize         │
                            │ • Optimize       │
                            └────────┬─────────┘
                                     │
                    ┌────────────┴────────────┐
                    │ Upload thành công?      │
                    ├────────────┬────────────┤
                    │ Không     │ Có         │
                    ▼           ▼            ▼
              Show error    ┌──────────────────┐
                            │              BƯỚC 3: Telegram (REQUIRED) │
                            │ • telegram_username (@ format)            │
                            │ "Bạn sẽ cần gửi /start {code}"            │
                            └────────┬─────────┘
                                     │
                                     ▼
                            ┌──────────────────┐
                            │ Validate @ format │
                            └────────┬─────────┘
                                     │
                    ┌────────────┴────────────┐
                    │ Valid?                  │
                    ├────────────┬────────────┤
                    │ Không     │ Có         │
                    ▼           ▼            ▼
              Show error    ┌──────────────────┐
                            │              BƯỚC 4: Chính sách (OPTIONAL)│
                            │ • warranty_policy                       │
                            │ • refund_policy                         │
                            │ • support_hours                         │
                            └────────┬─────────┘
                                     │
                                     ▼
                            ┌──────────────────┐
                            │ Click:            │
                            │ "Tạo gian hàng"   │
                            └────────┬─────────┘
                                     │
                                     ▼
                            ┌──────────────────┐
                            │ Final validation  │
                            └────────┬─────────┘
                                     │
                    ┌────────────┴────────────┐
                    │ All valid?              │
                    ├────────────┬────────────┤
                    │ Không     │ Có         │
                    ▼           ▼            ▼
              Show error    ┌──────────────────┐
                            │ CREATE SHOP       │
                            │ ┌──────────────┐ │
                            │ │ vendor_id    │ │
                            │ │ shop_name    │ │
                            │ │ shop_slug    │ │
                            │ │ telegram_... │ │
                            │ │ telegram_... │ │
                            │ │   verified:  │ │
                            │ │   false      │ │
                            │ │ status:      │ │
                            │ │   active     │ │
                            │ │ level: new   │ │
                            │ └──────────────┘ │
                            └────────┬─────────┘
                                     │
                              ┌──────┴──────┐
                              │             │
                              ▼             ▼
                    ┌──────────────┐  ┌──────────────────┐
                    │ CREATE       │  │ Gen verification  │
                    │ storage dir  │  │ code (UUID)       │
                    │ /shops/{id}/ │  │ Store in Redis    │
                    └──────────────┘  │ TTL: 24h          │
                                      └────────┬─────────┘
                                               │
                                               ▼
                                      ┌──────────────────┐
                                      │ UPDATE USER      │
                                      │ ADD role: vendor │
                                      └────────┬─────────┘
                                               │
                                               ▼
                                      ┌──────────────────┐
                                      │ Queue welcome    │
                                      │ email            │
                                      └────────┬─────────┘
                                               │
                                               ▼
                                      ┌──────────────────┐
                                      │ Redirect:         │
                                      │ /vendor/shop/     │
                                      │ dashboard         │
                                      │ Flash: "Vui lòng  │
                                      │ xác nhận Telegram"│
                                      └──────────────────┘

                                    ┌─────────────────────┐
                                    │  END PHẦN 2         │
                                    │ User = Buyer+Vendor │
                                    │ Shop = Created      │
                                    │ Telegram = Pending  │
                                    └─────────────────────┘
```

### Input requirements

| Trường | Bắt buộc | Validation |
|-------|----------|------------|
| shop_name | Có | 3-50 ký tự, unique |
| shop_slug | Có | 3-60 ký tự, alphanumeric + hyphen, unique |
| shop_description | Có | Max 500 ký tự |
| shop_logo | Có | jpg/png, max 2MB |
| shop_banner | Không | jpg/png, max 5MB |
| telegram_username | Có | @username format, 11-32 chars |
| warranty_policy | Không | Text |
| refund_policy | Không | Text |
| support_hours | Không | Text |

### Shop levels

```
New → Silver → Gold → Diamond → Partner
```

---

## PHẦN 3: XÁC THỰC TELEGRAM

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      PHẦN 3: XÁC THỰC TELEGRAM                              │
└─────────────────────────────────────────────────────────────────────────────┘

                        ┌──────────────────┐
                        │ User mở Telegram  │
                        │ Gửi cho @p2pmmo: │
                        │ /start {code}    │
                        └────────┬─────────┘
                                 │
                                 ▼
                        ┌──────────────────┐
                        │ Bot nhận command  │
                        │ Extract:          │
                        │ • chat_id         │
                        │ • verification_   │
                        │   code            │
                        └────────┬─────────┘
                                 │
                                 ▼
                        ┌──────────────────┐
                        │ Query Redis       │
                        │ Find shop by code │
                        └────────┬─────────┘
                                 │
                    ┌────────────┴────────────┐
                    │ Code tồn tại?           │
                    ├────────────┬────────────┤
                    │ Không     │ Có         │
                    ▼           ▼            ▼
              Bot: Error    ┌──────────────────┐
                            │ Fetch shop        │
                            └────────┬─────────┘
                                     │
                        ┌────────────┴────────────┐
                        │ Shop found?             │
                        ├────────────┬────────────┤
                        │ Không     │ Có         │
                        ▼           ▼            ▼
                  Bot: Error    ┌──────────────────┐
                                │ Check:            │
                                │ telegram_verified │
                                └────────┬─────────┘
                                         │
                            ┌────────────┴────────────┐
                            │ Đã verified?            │
                            ├────────────┬────────────┤
                            │ Có        │ Không      │
                            ▼           ▼            ▼
                      Bot: Info    ┌──────────────────┐
                                    │ Compare username │
                                    │ (soft check)     │
                                    └────────┬─────────┘
                                             │
                                ┌────────────┴────────────┐
                                │ Match?                  │
                                ├────────────┬────────────┤
                                │ Không     │ Có         │
                                ▼           ▼            ▼
                          Bot: Warning  ┌──────────────────┐
                               (continue)│ UPDATE SHOP      │
                                          │ • chat_id        │
                                          │ • verified: true │
                                          │ • verified_at    │
                                          └────────┬─────────┘
                                                   │
                                                   ▼
                                          ┌──────────────────┐
                                          │ DEL Redis key    │
                                          └────────┬─────────┘
                                                   │
                                                   ▼
                                          ┌──────────────────┐
                                          │ Bot: Success      │
                                          │ "Đã liên kết!     │
                                          │  Bạn sẽ nhận      │
                                          │  thông báo..."    │
                                          └────────┬─────────┘
                                                   │
                                                   ▼
                                          ┌──────────────────┐
                                          │ Send test        │
                                          │ notification     │
                                          └────────┬─────────┘
                                                   │
                                                   ▼
                                          ┌──────────────────┐
                                          │ Update dashboard  │
                                          │ (via WebSocket)  │
                                          └──────────────────┘

                                    ┌─────────────────────┐
                                    │  END PHẦN 3         │
                                    │ Telegram = Verified │
                                    └─────────────────────┘
```

### Verification flow

1. Code được generate khi tạo shop (UUID v4)
2. Lưu trong Redis: `telegram:verify:{shop_id}` với TTL 24h
3. User gửi `/start {code}` cho bot @p2pmmo
4. Bot validates và cập nhật `telegram_chat_id`, `telegram_verified: true`

---

## PHẦN 4: HOÀN THIỆN SHOP

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    PHẦN 4: HOÀN THIỆN SHOP                                  │
└─────────────────────────────────────────────────────────────────────────────┘

                    Shop hiện tại:
                    ✅ Created
                    ✅ Telegram verified
                    ❌ No products
                    ❌ No policies

                        ┌──────────────────┐
                        │ Vendor dashboard  │
                        │ Banner: "Hoàn     │
                        │ thiện shop để     │
                        │ bắt đầu bán"      │
                        └────────┬─────────┘
                                 │
                ┌────────────────┴────────────────┐
                │                                  │
                ▼                                  ▼
    ┌───────────────────┐              ┌──────────────────┐
    │ THÊM SẢN PHẨM     │              │ CẤU HÌNH CHÍNH   │
    │                   │              │ SÁCH             │
    │ • Tạo sản phẩm    │              │                   │
    │ • Upload kho      │              │ • warranty_policy│
    │ • Set giá         │              │ • refund_policy  │
    │ • Set stock       │              │ • support_hours  │
    └────────┬──────────┘              └────────┬─────────┘
             │                                   │
             │                                   ▼
             │                          ┌──────────────────┐
             │                          │ Validate & Save  │
             │                          └────────┬─────────┘
             │                                   │
             └───────────────────┬───────────────┘
                                 │
                                 ▼
                        ┌──────────────────┐
                        │ Check completion │
                        └────────┬─────────┘
                                 │
                ┌────────────────┴────────────────┐
                │                                  │
                ▼                                  ▼
    ┌───────────────────┐              ┌──────────────────┐
    │ total_products>0? │              │ policies set?    │
    └────────┬──────────┘              └────────┬─────────┘
             │                                   │
             └───────────────────┬───────────────┘
                                 │
                    ┌────────────┴────────────┐
                    │ Cả 2 điều kiện OK?      │
                    ├────────────┬────────────┤
                    │ Không     │ Có         │
                    ▼           ▼            ▼
              Continue    ┌──────────────────┐
                          │ SHOP HOÀN THIỆN! │
                          │ ┌──────────────┐ │
                          │ │ ✅ Created   │ │
                          │ │ ✅ Telegram  │ │
                          │ │ ✅ Products  │ │
                          │ │ ✅ Policies  │ │
                          │ └──────────────┘ │
                          └────────┬─────────┘
                                   │
                                   ▼
                          ┌──────────────────┐
                          │ Update dashboard  │
                          │ "Shop đã sẵn sàng │
                          │  bán hàng!"       │
                          └────────┬─────────┘
                                   │
                                   ▼
                          ┌──────────────────┐
                          │ Start analytics   │
                          │ Start tracking    │
                          │ sales/ratings     │
                          └────────┬─────────┘
                                   │
                                   ▼
                          ┌──────────────────┐
                          │ Notify via       │
                          │ Telegram:         │
                          │ "🎉 Shop của bạn  │
                          │  đã sẵn sàng!"    │
                          └──────────────────┘

                                    ┌─────────────────────┐
                                    │   ✅ END PHẦN 4     │
                                    │   SHOP HOÀN THIỆN   │
                                    │   Ready to sell!    │
                                    └─────────────────────┘
```

### Điều kiện shop hoàn thiện

Shop được coi là hoàn thiện khi TẤT CẢ:

1. ✅ Shop đã tạo (`status: active`)
2. ✅ Telegram đã xác thực (`telegram_verified: true`)
3. ✅ Có ít nhất 1 sản phẩm (`total_products > 0`)
4. ✅ Đã cấu hình chính sách (`warranty_policy`, `refund_policy`, `support_hours`)

---

## TỔNG KẾT

### So sánh V1 vs V2

| Tính năng | V1 | V2 |
|-----------|----|----|
| Đăng ký Vendor | Form riêng, chờ Admin duyệt | Tự động khi tạo shop |
| Approval | Admin phải approve | Không cần approve |
| Telegram | Không bắt buộc | **BẮT BUỘC** |
| Reseller | Có (system reseller) | **ĐÃ BỎ** |
| Shop completion | Tạo shop xong = xong | Cần Telegram + Products + Policies |

### Flow state transitions

```
┌──────────────┐
│  Anonymous   │
└──────┬───────┘
       │ REGISTER
       ▼
┌──────────────┐     VERIFY EMAIL
│ Buyer        │ ─────────────────────► ┌──────────────┐
│ (pending)    │                         │ Buyer        │
└──────┬───────┘                         │ (active)     │
       │                                 └──────┬───────┘
       │                                         │
       │ LOGIN                                   │ CREATE SHOP
       │                                         ▼
       │                                  ┌──────────────┐
       │                                  │ Vendor       │
       │                                  │ (pending TG) │
       │                                  └──────┬───────┘
       │                                         │
       │                                         │ VERIFY TELEGRAM
       │                                         ▼
       │                                  ┌──────────────┐
       │                                  │ Vendor       │
       └─────────────────────────────────►│ (active)     │
                                          │ Shop: Created│
                                          └──────┬───────┘
                                                 │
                                                 │ ADD PRODUCTS
                                                 │ + POLICIES
                                                 ▼
                                          ┌──────────────┐
                                          │ Vendor       │
                                          │ Shop: READY  │
                                          └──────────────┘
```

### Checklist implementation

- [ ] PHẦN 1: Register & Login
  - [ ] Form đăng ký với validation
  - [ ] Captcha integration
  - [ ] Rate limiting
  - [ ] Email verification
  - [ ] Auto wallet creation
  - [ ] Login flow with 2FA support

- [ ] PHẦN 2: Tạo Shop
  - [ ] 4-step Wizard (Basic → Images → Telegram → Policies)
  - [ ] Slug generation & validation
  - [ ] File upload (logo, banner)
  - [ ] Telegram username validation
  - [ ] Auto add vendor role
  - [ ] Verification code generation

- [ ] PHẦN 3: Telegram Verification
  - [ ] Bot @p2pmmo command handling
  - [ ] Redis verification code storage
  - [ ] Username soft validation
  - [ ] Shop update with chat_id
  - [ ] Test notification

- [ ] PHẦN 4: Shop Completion
  - [ ] Dashboard completion banner
  - [ ] Product creation flow
  - [ ] Policy configuration
  - [ ] Completion check logic
  - [ ] Analytics tracking

---

## Refs

- [Authentication V1](../v1/01-authentication.md)
- [User Roles V1](../v1/02-user-roles.md)
- [Shop Management V1](../v1/03-shop-management.md)
- [Wallet System V2](../v2/01-wallet-system-design.md)
- [Complete Flows V2](./shop/01-complete-flows.md)
