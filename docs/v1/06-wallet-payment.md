# Chức năng Ví và Thanh toán (Wallet & Payment)

## Tổng quan

Hệ thống ví điện tử nội bộ là trung tâm của mọi giao dịch trên TaphoaMMO. Buyer nạp tiền vào ví trước, sau đó dùng số dư để mua hàng. Vendor nhận tiền vào ví (với 3 ngày holding) và có thể rút về tài khoản ngân hàng.

---

## 1. Cấu trúc Ví (Wallet)

### 1.1 Các loại số dư

```
┌─────────────────────────────────────────────────────────────────┐
│                    CẤU TRÚC SỐ DƯ VÍ                            │
└─────────────────────────────────────────────────────────────────┘

BUYER WALLET:
┌─────────────────────────────────────────────────────────────────┐
│  Số dư khả dụng (Available Balance)                             │
│  └── Có thể dùng để mua hàng ngay                              │
│                                                                 │
│  Số dư: 500,000đ                                               │
└─────────────────────────────────────────────────────────────────┘

VENDOR WALLET:
┌─────────────────────────────────────────────────────────────────┐
│  Số dư khả dụng (Available Balance)                             │
│  └── Có thể rút về ngân hàng                                   │
│  └── Tiền từ đơn hàng đã qua 3 ngày                           │
│                                                                 │
│  Số dư đang giữ (Pending Balance)                              │
│  └── Tiền từ đơn hàng chưa qua 3 ngày                         │
│  └── Không thể rút                                             │
│                                                                 │
│  Available: 2,000,000đ                                         │
│  Pending:     500,000đ                                         │
│  ─────────────────────                                         │
│  Total:     2,500,000đ                                         │
└─────────────────────────────────────────────────────────────────┘
```

### 1.2 Các loại giao dịch

| Type | Mô tả | Buyer | Vendor |
|------|-------|-------|--------|
| deposit | Nạp tiền | ✅ | ✅ |
| purchase | Mua hàng | ✅ (trừ) | - |
| sale | Bán hàng | - | ✅ (cộng pending) |
| sale_released | Tiền được release | - | ✅ (pending → available) |
| refund | Hoàn tiền | ✅ (cộng) | ✅ (trừ) |
| withdraw | Rút tiền | - | ✅ (trừ) |
| commission | Hoa hồng CTV | ✅ (cộng) | - |
| adjustment | Admin điều chỉnh | ✅ | ✅ |

---

## 2. Nạp tiền (Deposit)

### 2.1 Các phương thức nạp tiền

```
┌─────────────────────────────────────────────────────────────────┐
│               PHƯƠNG THỨC NẠP TIỀN                              │
└─────────────────────────────────────────────────────────────────┘

1. CHUYỂN KHOẢN NGÂN HÀNG (Bank Transfer)
   ├── Ngân hàng hỗ trợ: Vietcombank, Techcombank, MB Bank, ...
   ├── Tự động xác nhận qua API banking
   ├── Tối thiểu: 30,000đ
   ├── Phí: 0%
   └── Thời gian: 1-5 phút (auto) hoặc 24h (manual)

2. VÍ ĐIỆN TỬ MOMO
   ├── Quét QR hoặc chuyển số điện thoại
   ├── Tự động xác nhận qua API
   ├── Tối thiểu: 10,000đ
   ├── Phí: 0%
   └── Thời gian: Tức thì

3. USDT (TRC20)
   ├── Chuyển USDT vào địa chỉ ví
   ├── Xác nhận sau 20 confirmations
   ├── Tối thiểu: 10 USDT
   ├── Tỷ giá: Cập nhật realtime
   └── Thời gian: 5-30 phút

4. PAYPAL
   ├── Thanh toán qua PayPal
   ├── Tỷ giá: 1 USD = 22,000 VND
   ├── Tối thiểu: 5 USD
   ├── Phí: 5%
   └── Thời gian: Tức thì
```

### 2.2 Flow nạp tiền Bank Transfer

```
┌─────────────────────────────────────────────────────────────────┐
│              FLOW NẠP TIỀN CHUYỂN KHOẢN                         │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] User vào Ví > Nạp tiền
         │
         ▼
[Bước 2] Chọn phương thức "Chuyển khoản ngân hàng"
         │
         ▼
[Bước 3] Hệ thống hiển thị thông tin chuyển khoản:

         ╔═══════════════════════════════════════════════════════╗
         ║  THÔNG TIN CHUYỂN KHOẢN                               ║
         ╠═══════════════════════════════════════════════════════╣
         ║                                                       ║
         ║  Ngân hàng: VIETCOMBANK                               ║
         ║  Số tài khoản: 1234567890                             ║
         ║  Chủ tài khoản: CONG TY TNHH ABC                      ║
         ║                                                       ║
         ║  ─────────────────────────────────────────────────── ║
         ║  NỘI DUNG CHUYỂN KHOẢN (BẮT BUỘC):                   ║
         ║  ┌───────────────────────────────────────────────────┐║
         ║  │  NAP 12345                                        │║
         ║  └───────────────────────────────────────────────────┘║
         ║  [📋 Copy]                                            ║
         ║                                                       ║
         ║  ⚠️ Lưu ý:                                            ║
         ║  • Nội dung chuyển khoản phải CHÍNH XÁC              ║
         ║  • Số tiền tối thiểu: 30,000đ                        ║
         ║  • Tiền sẽ vào ví trong 1-5 phút                     ║
         ║                                                       ║
         ║  [Tôi đã chuyển khoản]                                ║
         ╚═══════════════════════════════════════════════════════╝
         │
         ▼
[Bước 4] User chuyển khoản với nội dung đúng
         │
         ▼
[Bước 5] Webhook từ Banking API nhận giao dịch:
         {
           "bank": "VCB",
           "amount": 100000,
           "content": "NAP 12345",
           "time": "2024-01-15 10:30:00"
         }
         │
         ▼
[Bước 6] Parse nội dung chuyển khoản:
         - Regex: /NAP\s*(\d+)/i
         - Extract user_id: 12345
         │
         ├── Không parse được ──► Log để xử lý manual
         │
         ▼
[Bước 7] Verify user tồn tại và active
         │
         ├── User không hợp lệ ──► Log để xử lý manual
         │
         ▼
[Bước 8] Kiểm tra duplicate:
         - Cùng amount + content + time trong 5 phút
         │
         ├── Duplicate ──► Ignore
         │
         ▼
[Bước 9] Tạo Transaction record:
         - user_id: 12345
         - type: deposit
         - amount: 100,000
         - method: bank_transfer
         - status: completed
         │
         ▼
[Bước 10] Cộng tiền vào ví:
          UPDATE wallets 
          SET balance = balance + 100000
          WHERE user_id = 12345
          │
          ▼
[Bước 11] Gửi notification:
          - Push notification
          - Email (optional)
          │
          ▼
[Bước 12] User thấy số dư được cập nhật
```

### 2.3 Flow nạp tiền USDT

```
┌─────────────────────────────────────────────────────────────────┐
│                  FLOW NẠP TIỀN USDT                             │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] User chọn phương thức USDT
         │
         ▼
[Bước 2] Hệ thống generate/hiển thị địa chỉ ví USDT:

         ╔═══════════════════════════════════════════════════════╗
         ║  NẠP TIỀN BẰNG USDT (TRC20)                           ║
         ╠═══════════════════════════════════════════════════════╣
         ║                                                       ║
         ║  Địa chỉ ví:                                          ║
         ║  TJYs7CzVBvfv5krWjpQCfVKS3iLh5DVxxx                   ║
         ║  [📋 Copy] [QR Code]                                  ║
         ║                                                       ║
         ║  Network: TRC20 (Tron)                                ║
         ║  ⚠️ Chỉ gửi USDT qua mạng TRC20                       ║
         ║                                                       ║
         ║  Tỷ giá hiện tại: 1 USDT = 24,500 VND                ║
         ║  Tối thiểu: 10 USDT                                   ║
         ║  Phí mạng: ~1 USDT                                    ║
         ║                                                       ║
         ║  Trạng thái: ⏳ Đang chờ giao dịch...                 ║
         ╚═══════════════════════════════════════════════════════╝
         │
         ▼
[Bước 3] User gửi USDT đến địa chỉ
         │
         ▼
[Bước 4] Backend monitor blockchain (mỗi 30 giây):
         - Scan incoming transactions
         - Filter by address
         │
         ▼
[Bước 5] Phát hiện giao dịch mới:
         - Amount: 50 USDT
         - TxHash: xxx
         - Confirmations: 0
         │
         ▼
[Bước 6] Tạo pending deposit record
         │
         ▼
[Bước 7] Đợi confirmations (thường 20 blocks)
         │
         ├── < 20 confirmations ──► Hiển thị "Đang xác nhận X/20"
         │
         ▼
[Bước 8] Đủ confirmations:
         - Lấy tỷ giá tại thời điểm
         - VND amount = 50 * 24,500 = 1,225,000
         │
         ▼
[Bước 9] Cộng tiền và thông báo
```

### 2.4 Nạp tiền thủ công (Manual)

```
┌─────────────────────────────────────────────────────────────────┐
│              FLOW NẠP TIỀN THỦ CÔNG (ADMIN)                     │
└─────────────────────────────────────────────────────────────────┘

Áp dụng khi:
- Auto detect thất bại
- User quên ghi nội dung
- Phương thức không hỗ trợ auto

[Bước 1] User liên hệ support kèm:
         - Ảnh chụp bill chuyển khoản
         - Username
         - Số tiền
         │
         ▼
[Bước 2] Admin vào Admin Panel > Deposits > Manual
         │
         ▼
[Bước 3] Kiểm tra:
         - Bill hợp lệ
         - Chưa được xử lý trước đó
         │
         ▼
[Bước 4] Tạo manual deposit:
         - Chọn user
         - Nhập số tiền
         - Upload ảnh bill
         - Ghi chú lý do
         │
         ▼
[Bước 5] Submit và xác nhận
         │
         ▼
[Bước 6] Hệ thống:
         - Tạo transaction với type = deposit_manual
         - Cộng tiền vào ví user
         - Gửi notification
         │
         ▼
[Bước 7] Log admin action
```

---

## 3. Xem lịch sử giao dịch

### Flow xem transactions

```
┌─────────────────────────────────────────────────────────────────┐
│              FLOW XEM LỊCH SỬ GIAO DỊCH                         │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] User vào Ví > Lịch sử giao dịch
         │
         ▼
[Bước 2] Query transactions:
         
         SELECT * FROM transactions
         WHERE user_id = {user_id}
         ORDER BY created_at DESC
         │
         ▼
[Bước 3] Hiển thị với filters:

┌─────────────────────────────────────────────────────────────────┐
│  LỊCH SỬ GIAO DỊCH                                             │
├─────────────────────────────────────────────────────────────────┤
│  Số dư hiện tại: 500,000đ                                      │
│                                                                 │
│  Filter: [Tất cả ▼] [Tháng này ▼]                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  15/01/2024 10:30                                              │
│  🔴 Mua hàng - Gmail US x 10                    -125,000đ      │
│  Số dư sau: 500,000đ                                           │
│                                                                 │
│  15/01/2024 09:00                                              │
│  🟢 Nạp tiền - Chuyển khoản VCB               +200,000đ        │
│  Số dư sau: 625,000đ                                           │
│                                                                 │
│  14/01/2024 15:20                                              │
│  🔵 Hoàn tiền - Đơn #12340                    +50,000đ         │
│  Số dư sau: 425,000đ                                           │
│                                                                 │
│  [Load more...]                                                │
└─────────────────────────────────────────────────────────────────┘

Icon:
🟢 Cộng tiền (deposit, refund, commission)
🔴 Trừ tiền (purchase, withdraw)
🔵 Hoàn tiền
🟡 Đang xử lý
```

---

## 4. Rút tiền (Withdraw) - Vendor

### 4.1 Điều kiện rút tiền

```
┌─────────────────────────────────────────────────────────────────┐
│                  ĐIỀU KIỆN RÚT TIỀN                             │
└─────────────────────────────────────────────────────────────────┘

1. Chỉ Vendor mới có thể rút tiền

2. Số tiền rút:
   - Tối thiểu: 100,000đ
   - Tối đa: Số dư khả dụng
   - Không thể rút số dư pending

3. Thông tin ngân hàng:
   - Phải cập nhật đầy đủ
   - Tên chủ TK phải khớp với tên đăng ký

4. Giới hạn:
   - Tối đa 3 lệnh rút/ngày
   - Tối đa 50,000,000đ/ngày

5. Phí rút:
   - Miễn phí nếu >= 500,000đ
   - 10,000đ nếu < 500,000đ
```

### 4.2 Flow rút tiền

```
┌─────────────────────────────────────────────────────────────────┐
│                    FLOW RÚT TIỀN                                │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Vendor vào Ví > Rút tiền
         │
         ▼
[Bước 2] Hiển thị thông tin ví:

         ╔═══════════════════════════════════════════════════════╗
         ║  RÚT TIỀN VỀ NGÂN HÀNG                                ║
         ╠═══════════════════════════════════════════════════════╣
         ║                                                       ║
         ║  Số dư khả dụng: 2,000,000đ                          ║
         ║  Số dư đang giữ: 500,000đ (release sau 2 ngày)       ║
         ║                                                       ║
         ║  ─────────────────────────────────────────────────── ║
         ║  THÔNG TIN NGÂN HÀNG                                 ║
         ║  Ngân hàng: Vietcombank                               ║
         ║  Số TK: ****7890                                      ║
         ║  Chủ TK: NGUYEN VAN A                                 ║
         ║  [Thay đổi]                                           ║
         ║                                                       ║
         ║  ─────────────────────────────────────────────────── ║
         ║  SỐ TIỀN RÚT                                         ║
         ║  [________________] VNĐ                               ║
         ║  [Rút tất cả: 2,000,000đ]                            ║
         ║                                                       ║
         ║  Phí rút: 0đ                                          ║
         ║  Thực nhận: 2,000,000đ                                ║
         ║                                                       ║
         ║  [Yêu cầu rút tiền]                                   ║
         ╚═══════════════════════════════════════════════════════╝
         │
         ▼
[Bước 3] Vendor nhập số tiền và submit
         │
         ▼
[Bước 4] Validate:
         │
         ├── < 100,000 ──► "Tối thiểu 100,000đ"
         ├── > available ──► "Vượt quá số dư khả dụng"
         ├── Vượt limit ngày ──► "Đã đạt giới hạn rút tiền hôm nay"
         │
         ▼
[Bước 5] Yêu cầu xác thực 2FA (nếu có)
         │
         ▼
[Bước 6] Tính phí:
         - amount >= 500,000 ──► fee = 0
         - amount < 500,000 ──► fee = 10,000
         │
         ▼
[Bước 7] Tạo Withdrawal request:
         - status: pending
         - amount: số tiền yêu cầu
         - fee: phí
         - net_amount: thực nhận
         - bank_info: thông tin ngân hàng
         │
         ▼
[Bước 8] Trừ tiền từ ví (hold):
         UPDATE wallets 
         SET available_balance = available_balance - {amount}
         WHERE user_id = {vendor_id}
         │
         ▼
[Bước 9] Gửi notification cho Admin
         │
         ▼
[Bước 10] Hiển thị:
          "Yêu cầu rút tiền đã được ghi nhận. 
           Thời gian xử lý: 1-24 giờ trong giờ hành chính."

─────────────────────────────────────────────────────────────────

[Admin xử lý yêu cầu rút tiền]
         │
         ▼
[A1] Admin vào Withdrawals > Pending
     │
     ▼
[A2] Xem chi tiết yêu cầu:
     - Thông tin vendor
     - Số tiền
     - Thông tin ngân hàng
     - Lịch sử rút tiền
     │
     ▼
[A3] Admin chuyển khoản thủ công
     │
     ▼
[A4] Cập nhật trạng thái:
     │
     ├── Approve:
     │   - status = completed
     │   - Nhập transaction reference
     │   - Gửi email thông báo vendor
     │
     ├── Reject:
     │   - status = rejected
     │   - Nhập lý do
     │   - Hoàn tiền vào ví vendor
     │   - Gửi email thông báo
     │
     ▼
[A5] Log admin action
```

### 4.3 Trạng thái Withdrawal

| Status | Mô tả | Vendor action |
|--------|-------|---------------|
| pending | Chờ admin xử lý | Có thể hủy |
| processing | Admin đang xử lý | Không thể hủy |
| completed | Đã chuyển tiền | - |
| rejected | Bị từ chối | Xem lý do, thử lại |
| cancelled | Vendor đã hủy | - |

---

## 5. Escrow System (Giữ tiền 3 ngày)

### 5.1 Logic Escrow

```
┌─────────────────────────────────────────────────────────────────┐
│                    HỆ THỐNG ESCROW                              │
└─────────────────────────────────────────────────────────────────┘

Mục đích:
- Bảo vệ buyer trong 3 ngày đầu
- Cho phép khiếu nại nếu sản phẩm có vấn đề
- Tạo niềm tin cho marketplace

Timeline:
─────────────────────────────────────────────────────────────────
T+0h:   Buyer mua hàng
        └── Tiền trừ từ ví buyer
        └── Tiền vào pending_balance của vendor

T+0h đến T+72h: Thời gian bảo vệ buyer
        └── Buyer có thể khiếu nại
        └── Vendor không thể rút số tiền này

T+72h:  Không có dispute
        └── Auto release: pending → available
        └── Vendor có thể rút tiền

Nếu có Dispute:
        └── Tiền tiếp tục bị hold
        └── Chờ kết quả giải quyết
        └── Refund buyer HOẶC release vendor
─────────────────────────────────────────────────────────────────
```

### 5.2 Flow Release tiền

```
┌─────────────────────────────────────────────────────────────────┐
│                FLOW RELEASE TIỀN TỰ ĐỘNG                        │
└─────────────────────────────────────────────────────────────────┘

Cron Job: Chạy mỗi giờ

[Bước 1] Query payouts cần release:
         
         SELECT * FROM payouts
         WHERE status = 'pending'
           AND release_at <= NOW()
           AND order_id NOT IN (
             SELECT order_id FROM disputes
             WHERE status IN ('pending', 'processing')
           )
         │
         ▼
[Bước 2] Với mỗi payout:
         │
         ▼
[Bước 3] Begin Transaction
         │
         ▼
[Bước 4] Cập nhật payout:
         UPDATE payouts 
         SET status = 'released', released_at = NOW()
         WHERE id = {payout_id}
         │
         ▼
[Bước 5] Chuyển tiền trong ví vendor:
         UPDATE vendor_wallets SET
           pending_balance = pending_balance - {amount},
           available_balance = available_balance + {amount}
         WHERE vendor_id = {vendor_id}
         │
         ▼
[Bước 6] Tạo transaction record:
         type = sale_released
         │
         ▼
[Bước 7] Cập nhật order status = completed
         │
         ▼
[Bước 8] Commit Transaction
         │
         ▼
[Bước 9] Gửi notification cho vendor
```

---

## 6. Admin: Điều chỉnh số dư

### Flow điều chỉnh (Admin)

```
┌─────────────────────────────────────────────────────────────────┐
│              FLOW ADMIN ĐIỀU CHỈNH SỐ DƯ                        │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Admin vào Users > Chi tiết user > Ví
         │
         ▼
[Bước 2] Hiển thị số dư hiện tại
         │
         ▼
[Bước 3] Click "Điều chỉnh số dư"
         │
         ▼
[Bước 4] Nhập thông tin:
         
         ╔═══════════════════════════════════════════════════════╗
         ║  ĐIỀU CHỈNH SỐ DƯ                                     ║
         ╠═══════════════════════════════════════════════════════╣
         ║  User: user123                                        ║
         ║  Số dư hiện tại: 500,000đ                            ║
         ║                                                       ║
         ║  Loại điều chỉnh:                                     ║
         ║  ○ Cộng tiền                                          ║
         ║  ○ Trừ tiền                                           ║
         ║                                                       ║
         ║  Số tiền: [___________]                               ║
         ║                                                       ║
         ║  Lý do *:                                             ║
         ║  [_______________________________________]            ║
         ║  [_______________________________________]            ║
         ║                                                       ║
         ║  [Xác nhận]                                           ║
         ╚═══════════════════════════════════════════════════════╝
         │
         ▼
[Bước 5] Validate:
         - Lý do bắt buộc
         - Trừ tiền không được > số dư
         │
         ▼
[Bước 6] Xác nhận bằng password admin
         │
         ▼
[Bước 7] Thực hiện điều chỉnh:
         - Tạo transaction type = adjustment
         - Cập nhật balance
         │
         ▼
[Bước 8] Gửi notification cho user
         │
         ▼
[Bước 9] Log admin action với full details
```

---

## Database Schema

### Bảng wallets

```sql
CREATE TABLE wallets (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    user_id BIGINT UNIQUE NOT NULL,
    balance DECIMAL(15,0) DEFAULT 0,           -- Buyer: tổng số dư
    available_balance DECIMAL(15,0) DEFAULT 0, -- Vendor: có thể rút
    pending_balance DECIMAL(15,0) DEFAULT 0,   -- Vendor: đang giữ
    total_deposited DECIMAL(15,0) DEFAULT 0,
    total_withdrawn DECIMAL(15,0) DEFAULT 0,
    total_spent DECIMAL(15,0) DEFAULT 0,       -- Buyer
    total_earned DECIMAL(15,0) DEFAULT 0,      -- Vendor
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    FOREIGN KEY (user_id) REFERENCES users(id),
    
    CHECK (balance >= 0),
    CHECK (available_balance >= 0),
    CHECK (pending_balance >= 0)
);
```

### Bảng transactions

```sql
CREATE TABLE transactions (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    transaction_number VARCHAR(30) UNIQUE,     -- TXN-YYYYMMDD-XXXXX
    user_id BIGINT NOT NULL,
    type ENUM('deposit', 'deposit_manual', 'purchase', 'sale', 
              'sale_released', 'refund', 'withdraw', 
              'commission', 'adjustment'),
    amount DECIMAL(15,0) NOT NULL,             -- Positive hoặc negative
    balance_before DECIMAL(15,0),
    balance_after DECIMAL(15,0),
    
    -- Reference
    order_id BIGINT NULL,
    withdrawal_id BIGINT NULL,
    
    -- Deposit specific
    deposit_method ENUM('bank_transfer', 'momo', 'usdt', 'paypal') NULL,
    deposit_reference VARCHAR(100) NULL,       -- Bank transaction ref
    
    -- Meta
    description TEXT,
    admin_id BIGINT NULL,                      -- Nếu là adjustment
    admin_note TEXT NULL,
    
    status ENUM('pending', 'completed', 'failed', 'cancelled'),
    
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP NULL,
    
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (order_id) REFERENCES orders(id),
    FOREIGN KEY (admin_id) REFERENCES users(id),
    
    INDEX idx_user (user_id),
    INDEX idx_type (type),
    INDEX idx_created (created_at)
);
```

### Bảng withdrawals

```sql
CREATE TABLE withdrawals (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    withdrawal_number VARCHAR(30) UNIQUE,
    user_id BIGINT NOT NULL,
    amount DECIMAL(15,0) NOT NULL,
    fee DECIMAL(15,0) DEFAULT 0,
    net_amount DECIMAL(15,0) NOT NULL,
    
    -- Bank info (snapshot)
    bank_name VARCHAR(100),
    bank_account_number VARCHAR(30),
    bank_account_name VARCHAR(100),
    
    status ENUM('pending', 'processing', 'completed', 'rejected', 'cancelled'),
    
    -- Admin processing
    processed_by BIGINT NULL,
    processed_at TIMESTAMP NULL,
    bank_reference VARCHAR(100) NULL,
    reject_reason TEXT NULL,
    
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (processed_by) REFERENCES users(id)
);
```

### Bảng payouts (Escrow tracking)

```sql
CREATE TABLE payouts (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    order_id BIGINT UNIQUE NOT NULL,
    vendor_id BIGINT NOT NULL,
    amount DECIMAL(15,0) NOT NULL,             -- Sau khi trừ commission
    
    status ENUM('pending', 'released', 'refunded', 'partial_refund'),
    release_at TIMESTAMP NOT NULL,             -- created_at + 3 days
    released_at TIMESTAMP NULL,
    
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (order_id) REFERENCES orders(id),
    FOREIGN KEY (vendor_id) REFERENCES users(id),
    
    INDEX idx_release (status, release_at)
);
```
