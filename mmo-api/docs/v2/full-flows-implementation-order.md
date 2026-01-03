# Full Flows & Implementation Order - P2PMMO V2

## Tổng quan

Document này mô tả **full flows** cho tất cả actors (Buyer, Vendor, Admin) và đề xuất thứ tự implementation dựa trên dependencies.

---

# PHẦN 1: FULL FLOW DIAGRAMS

## 1.1 FLOW BUYER (Người mua)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          BUYER - FULL FLOW                                  │
└─────────────────────────────────────────────────────────────────────────────┘

                    ┌──────────────┐
                    │  ANONYMOUS   │
                    └──────┬───────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
        ▼                  ▼                  ▼
  ┌──────────┐      ┌──────────┐      ┌──────────┐
  │ REGISTER │      │  LOGIN   │      │ BROWSE   │
  └─────┬────┘      └─────┬────┘      └─────┬────┘
        │                  │                  │
        ▼                  ▼                  ▼
  ┌──────────┐      ┌──────────┐      ┌──────────────────┐
  │ Buyer    │      │ Buyer    │      │ VIEW PRODUCTS    │
  │ (pending)│      │(active)  │      │ VIEW SHOPS        │
  └─────┬────┘      └─────┬────┘      │ SEARCH           │
        │                  │                  │ FILTERS
        ▼                  │          └─────────┬────────┘
  ┌──────────┐            │                    │
  │ Verify   │            │                    ▼
  │ Email    │            │            ┌──────────────────┐
  └─────┬────┘            │            │  VIEW PRODUCT    │
        │                  │            │  - Check stock    │
        └──────────────────┘            │  - Read reviews   │
                   │                     └─────────┬────────┘
                   │                               │
                   │          ┌────────────────────┘
                   │          │
                   ▼          ▼
            ┌────────────────────────┐
            │     WALLET CENTER       │
            │  - Deposit money       │
            │  - View balance        │
            │  - View transactions   │
            └───────────┬────────────┘
                        │
        ┌───────────────┼───────────────┐
        │               │               │
        ▼               ▼               ▼
  ┌──────────┐   ┌──────────┐   ┌──────────┐
  │ BANK     │   │ MOMO     │   │ USDT     │
  │ TRANSFER │   │         │   │ PAYPAL   │
  └─────┬────┘   └─────┬────┘   └─────┬────┘
        │              │              │
        └──────────────┼──────────────┘
                       ▼
                ┌──────────────┐
                │ WALLET READY │
                └──────┬───────┘
                       │
        ┌──────────────┼──────────────┐
        │              │              │
        ▼              ▼              ▼
  ┌──────────────┐┌──────────────┐┌──────────────┐
  │ BUY PRODUCT  ││ PRE-ORDER    ││ COUPON       │
  │              ││              ││              │
  │ [Check stock]││[Out of stock]││[Apply code]  │
  │ [Select qty] ││[Set wait day]││[Get discount]│
  │ [Optional:   ││[Pay now]     ││              │
  │  2FA verify] ││              ││              │
  └──────┬───────┘└──────┬───────┘└──────┬───────┘
         │                │                │
         └────────────────┼────────────────┘
                          │
                          ▼
                   ┌──────────────┐
                   │  CHECKOUT    │
                   │              │
                   │ [Validate]   │
                   │ [Confirm]    │
                   └──────┬───────┘
                          │
                          ▼
                   ┌──────────────┐
                   │  PAYMENT     │
                   │              │
                   │ [Deduct from]│
                   │ [wallet]     │
                   └──────┬───────┘
                          │
                          ▼
                   ┌──────────────┐
                   │ AUTO DELIVER │
                   │              │
                   │ [Lock stock] │
                   │ [Create order]│
                   │ [Export items]│
                   │ [Show content]│
                   └──────┬───────┘
                          │
                          ▼
                   ┌──────────────┐
                   │ ORDER STATUS │
                   │              │
                   │ PENDING → PAID│
                   │ → DELIVERED  │
                   └──────┬───────┘
                          │
        ┌─────────────────┼─────────────────┐
        │                 │                 │
        ▼                 ▼                 ▼
  ┌──────────┐     ┌──────────┐     ┌──────────┐
  │ HAPPY    │     │ ISSUE?   │     │ WAITING  │
  │ PATH     │     │ CREATE   │     │ 3 DAYS   │
  │          │     │ DISPUTE  │     │          │
  │[Review   │     └─────┬────┘     └─────┬────┘
  │ product] │           │                 │
  │[Rate     │           ▼                 ▼
  │ shop]    │    ┌──────────┐       ┌──────────┐
  └──────────┘    │ DISPUTE  │       │ COMPLETE │
                   │ FLOW    │       │ RELEASE  │
                   │          │       │ TO VENDOR│
                   └─────┬────┘       └──────────┘
                         │
        ┌────────────────┼────────────────┐
        │                │                │
        ▼                ▼                ▼
  ┌──────────┐     ┌──────────┐     ┌──────────┐
  │ REFUND   │     │ PARTIAL  │     │ REJECTED │
  │ FULL     │     │ REFUND   │     │          │
  └──────────┘     └──────────┘     └──────────┘
```

### Buyer Flows Chi Tiết

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     BUYER - FLOWS CHI TIẾT                                │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│ 1. FLOW ĐĂNG KÝ & ĐĂNG NHẬP                                                  │
└─────────────────────────────────────────────────────────────────────────────┘

REGISTER FLOW:
  1. /register → Enter: username, email, password
  2. Solve captcha
  3. Check duplicate (username, email)
  4. Hash password (bcrypt)
  5. Create User (status: pending_verification, role: buyer)
  6. Create Wallet (balance: 0)
  7. Send verification email (optional)
  8. Display: "Check email to verify"

LOGIN FLOW:
  1. /login → Enter: username/email + password
  2. Check credentials
  3. If 2FA enabled → Require OTP
  4. Verify TOTP code
  5. Create session
  6. Redirect by role:
     - Admin → /admin/dashboard
     - Vendor → /vendor/dashboard
     - Buyer → /dashboard

2FA SETUP (Optional):
  1. Settings > Security > Enable 2FA
  2. Enter password to confirm
  3. Generate secret key
  4. Display QR code
  5. User scans with authenticator app
  6. Enter OTP to verify
  7. Generate backup codes
  8. Store: two_factor_secret, backup_codes


┌─────────────────────────────────────────────────────────────────────────────┐
│ 2. FLOW WALLET - NẠP TIỀN                                                   │
└─────────────────────────────────────────────────────────────────────────────┘

BANK TRANSFER:
  1. /wallet/deposit → Select "Bank Transfer"
  2. Display bank info + content: NAP {user_id}
  3. User transfers with correct content
  4. Webhook receives transaction
  5. Parse: regex /NAP\s*(\d+)/i
  6. Validate user exists
  7. Check duplicate (same amount + content + time in 5 min)
  8. Create transaction (type: deposit)
  9. Credit wallet: balance += amount
  10. Notify user

MOMO:
  1. Select "Momo"
  2. Display QR code / phone number
  3. User transfers
  4. Webhook from Momo API
  5. Validate and credit

USDT:
  1. Select "USDT (TRC20)"
  2. Display wallet address
  3. User sends USDT
  4. Backend monitors blockchain
  5. Wait for 20 confirmations
  6. Get exchange rate
  7. Credit VND amount

MANUAL DEPOSIT (Admin):
  1. User contacts support with proof
  2. Admin verifies
  3. Admin creates manual deposit
  4. Credit wallet


┌─────────────────────────────────────────────────────────────────────────────┐
│ 3. FLOW MUA SẢN PHẨM                                                       │
└─────────────────────────────────────────────────────────────────────────────┘

BUY PRODUCT FLOW:
  1. Browse → /products/{slug} or /shops/{shop_slug}
  2. View product details
  3. Check real-time stock
  4. Select quantity (min_purchase to max_purchase)
  5. Apply coupon (optional)
  6. Click "Buy now"
  7. Check login → Redirect if not
  8. Check 2FA (if product requires)
  9. Check wallet balance
     → Not enough: Show "Top up" button
  10. Display order summary:
      - Subtotal
      - Bulk discount (if any)
      - Coupon discount
      - Total
  11. Confirm purchase
  12. *** TRANSACTION START ***
  13. Lock stock items (SELECT ... FOR UPDATE)
      → Not enough: Rollback, "Out of stock"
  14. Deduct from buyer wallet
      → Fail: Rollback
  15. Create order (status: paid)
  16. Mark items sold (is_sold: true)
  17. Create order_items (copy content)
  18. Update stats (product.stock, shop.total_sales)
  19. Create transaction records:
      - Buyer: type=purchase, amount=-total
      - Vendor: type=sale, amount=+total (pending 3 days)
  20. Handle coupon (increase usage)
  21. *** COMMIT ***
  22. Send notifications
  23. Display success page WITH PRODUCT CONTENT
  24. Auto-delivery: Show items immediately


┌─────────────────────────────────────────────────────────────────────────────┐
│ 4. FLOW PRE-ORDER (KHI HẾT HÀNG)                                           │
└─────────────────────────────────────────────────────────────────────────────┘

PRE-ORDER FLOW:
  1. Product out of stock (stock = 0)
  2. Display "Pre-order" button (if allow_preorder = true)
  3. Click "Pre-order"
  4. Select quantity
  5. Select wait time (1-7 days)
  6. Show total + warning: "Money deducted now, refunded if no stock"
  7. Confirm
  8. Check limits:
      - Max 5 pre-orders per user
      - Max 100 pre-orders per product
  9. *** TRANSACTION ***
  10. Deduct from wallet
  11. Create pre-order (status: pending)
  12. Set expires_at = NOW() + wait_days
  13. Create transaction (type: preorder_hold)
  14. *** COMMIT ***
  15. Notify user + vendor

AUTO-FULFILL (Trigger: Vendor restocks):
  1. Vendor uploads new stock
  2. Query pending pre-orders (FIFO by created_at)
  3. For each pre-order:
     a. Lock stock items
     b. Create order (status: delivered)
     c. Mark items sold
     d. Update pre-order (status: fulfilled)
     e. Create payout for vendor
     f. Notify buyer: "Your pre-order is ready!"
     g. Continue until stock runs out

AUTO-EXPIRE (Cron every 15 min):
  1. Query pre-orders where expires_at < NOW() AND status = pending
  2. For each:
     a. Update status = expired
     b. Refund wallet
     c. Create transaction (type: preorder_refund)
     d. Notify buyer


┌─────────────────────────────────────────────────────────────────────────────┐
│ 5. FLOW ĐƠN HÀNG & KHIẾU NẠI                                               │
└─────────────────────────────────────────────────────────────────────────────┘

ORDER STATUS FLOW:
  PENDING → PAID → DELIVERED → [DISPUTED / COMPLETED] → [REFUNDED / RELEASED]

  T+0:      Order created, paid, delivered immediately
  T+0~T+3:  Buyer can create dispute
  T+3:      If no dispute → Auto complete, release money to vendor

VIEW ORDERS:
  1. /orders → List all orders
  2. Filters: status, date range, search
  3. Click order → View details:
     - Order info
     - Product items (FULL content visible)
     - Payment info
     - Actions: [Dispute] [Review] [Reorder]

CREATE DISPUTE (Within 3 days):
  1. Click "Dispute" on delivered order
  2. Select reason: wrong_item, not_working, duplicate, etc.
  3. Enter affected quantity
  4. List problematic items
  5. Describe issue (min 50 chars)
  6. Upload proof (images, max 5 files)
  7. Select requested action:
      - Full refund
      - Partial refund
      - Replace items
  8. Submit
  9. Create dispute (status: pending)
  10. Update order (status: disputed)
  11. Notify vendor

DISPUTE RESOLUTION:
  Buyer → Vendor Response (48h) → [Resolve / Escalate] → Admin Decision

  Possible outcomes:
  - RESOLVED: Vendor accepts, refund processed
  - REFUNDED: Admin approves full refund
  - PARTIAL_REFUND: Admin approves partial
  - CLOSED: Dispute rejected, money released to vendor


┌─────────────────────────────────────────────────────────────────────────────┐
│ 6. FLOW ĐÁNH GIÁ                                                           │
└─────────────────────────────────────────────────────────────────────────────┘

CREATE REVIEW:
  1. Order status = delivered OR completed
  2. Within 30 days of delivery
  3. No pending dispute
  4. Click "Review product"
  5. Select rating: 1-5 stars
  6. Select quick tags (optional)
  7. Write comment (optional, max 500 chars)
  8. Upload images (optional, max 3)
  9. Check "Anonymous" (optional)
  10. Submit
  11. Create review
  12. Update product avg_rating
  13. Update shop rating
  14. Notify vendor

VENDOR REPLY:
  1. Vendor sees review
  2. Click "Reply"
  3. Write response (max 500 chars)
  4. Submit
  5. Add reply to review
```

---

## 1.2 FLOW VENDOR (Người bán)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          VENDOR - FULL FLOW                                 │
└─────────────────────────────────────────────────────────────────────────────┘

                    ┌──────────────┐
                    │  BUYER       │
                    └──────┬───────┘
                           │
                           ▼
                    ┌──────────────┐
                    │ REGISTER AS  │
                    │  VENDOR      │
                    └──────┬───────┘
                           │
                           ▼
                    ┌──────────────┐
                    │ CREATE SHOP  │
                    │ (Wizard 4    │
                    │  steps)      │
                    └──────┬───────┘
                           │
                           ▼
                    ┌──────────────┐
                    │ SHOP CREATED │
                    │ (pending     │
                    │  telegram)   │
                    └──────┬───────┘
                           │
                           ▼
                    ┌──────────────┐
                    │ VERIFY       │
                    │ TELEGRAM     │
                    └──────┬───────┘
                           │
                           ▼
                    ┌──────────────┐
                    │ SHOP ACTIVE  │
                    └──────┬───────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
        ▼                  ▼                  ▼
  ┌──────────┐      ┌──────────┐      ┌──────────┐
  │ PRODUCTS │      │ INVENTORY │      │ ORDERS   │
  │ MANAGE   │      │ UPLOAD    │      │ VIEW     │
  └─────┬────┘      └─────┬────┘      └─────┬────┘
        │                  │                  │
        ▼                  ▼                  ▼
  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐
  │ CREATE       │  │ BULK UPLOAD  │  │ VIEW ORDER   │
  │ PRODUCT      │  │ PASTE TEXT   │  │ DETAILS      │
  │              │  │ UPLOAD TXT   │  │              │
  │ [Name]       │  │              │  │ [Buyer info] │
  │ [Category]   │  │ CHECK        │  │ [Items sold] │
  │ [Price]      │  │ DUPLICATES   │  │ [Revenue]    │
  │ [Min/Max]    │  │              │  │              │
  │ [Pre-order?] │  │ AUTO-FULFILL │  │              │
  └──────┬───────┘  │ PRE-ORDERS   │  └──────────────┘
         │          └──────────────┘           │
         │                                     │
         └─────────────────┬───────────────────┘
                           │
                           ▼
                    ┌──────────────┐
                    │ WALLET       │
                    │              │
                    │ AVAILABLE    │
                    │ + PENDING    │
                    └──────┬───────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
        ▼                  ▼                  ▼
  ┌──────────┐      ┌──────────┐      ┌──────────┐
  │ PAYOUTS  │      │ REPORTS  │      │ DISPUTES │
  │          │      │          │      │          │
  │ [Request │      │ [Revenue │      │ [Respond │
  │  withdraw]│      │  by day] │      │  in 48h] │
  │ [History]│      │ [By      │      │ [Accept/ │
  └─────┬────┘      │  product]│      │  Reject] │
        │          └──────────┘      └─────┬────┘
        │                                     │
        ▼                                     ▼
  ┌──────────────┐                   ┌──────────────┐
  │ ADMIN        │                   │ ESCALATE TO  │
  │ APPROVES     │                   │ ADMIN        │
  │ → MONEY      │                   └──────────────┘
  │ TRANSFERRED  │
  └──────────────┘
```

### Vendor Flows Chi Tiết

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     VENDOR - FLOWS CHI TIẾT                                │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│ 1. FLOW TẠO SHOP & TELEGRAM VERIFY                                          │
└─────────────────────────────────────────────────────────────────────────────┘

REGISTER AS VENDOR:
  1. /vendor/dashboard (first time)
  2. Redirect to shop creation wizard

CREATE SHOP WIZARD:
  Step 1 - Basic Info:
    - shop_name (3-50 chars)
    - shop_slug (auto-generate, unique)
    - shop_description (max 500 chars)

  Step 2 - Branding:
    - shop_logo (REQUIRED, jpg/png, max 2MB)
    - shop_banner (optional, max 5MB)

  Step 3 - Telegram (REQUIRED in V2):
    - telegram_username (@ format, 11-32 chars)
    - Generate verification code (UUID)
    - Store in Redis: telegram:verify:{shop_id}, TTL 24h
    - Instruction: "Send /start {code} to @p2pmmo bot"

  Step 4 - Policies (Optional):
    - warranty_policy
    - refund_policy
    - support_hours

  5. Create shop (status: active, telegram_verified: false)
  6. Add role 'vendor' to user
  7. Create storage: /storage/shops/{shop_id}/
  8. Redirect to /vendor/shop/dashboard
  9. Flash: "Please verify Telegram"

TELEGRAM VERIFICATION:
  1. User opens Telegram
  2. Sends /start {verification_code} to @p2pmmo bot
  3. Bot receives command via Bot API
  4. Extract: chat_id, verification_code
  5. Query Redis to find shop_id by code
  6. Fetch shop details
  7. Check telegram_verified
     - Already verified: Info message
  8. Soft compare username (warning if mismatch)
  9. Update shop:
     - telegram_chat_id
     - telegram_verified: true
     - telegram_verified_at
  10. Delete Redis key
  11. Bot: "✅ Verified! You will receive notifications..."
  12. Send test notification
  13. Update dashboard (via WebSocket)


┌─────────────────────────────────────────────────────────────────────────────┐
│ 2. FLOW SẢN PHẨM & KHO HÀNG                                                 │
└─────────────────────────────────────────────────────────────────────────────┘

CREATE PRODUCT:
  1. /vendor/products/create
  2. Enter product info:
     - name (required, 5-200 chars)
     - category (required)
     - description (required, 50-5000 chars)
     - image (optional, max 5MB)
     - price (required, > 0)
     - original_price (optional)
     - min_purchase (default 1)
     - max_purchase (0 = unlimited)
     - allow_preorder (default false)
     - allow_resell (default false)
     - hide_stock (default false)
     - require_2fa (default false)
  3. Validate
  4. Process image (resize, optimize)
  5. Create product (status: draft, stock: 0)
  6. Redirect to inventory upload

UPLOAD INVENTORY:
  1. /vendor/products/{id}/inventory
  2. Show current stats: total, sold, remaining

  Option A - Paste Text:
    - Paste content (each line = 1 item)
    - Format: email|password|2fa_secret|email_backup

  Option B - Upload File:
    - Upload .txt file (max 10MB)

  3. Parse content:
     - Split by newline
     - Trim whitespace
     - Remove empty lines
     - Count items

  4. *** DUPLICATE CHECK ***
     - Level 1: Within current product (warning)
     - Level 2: Within current shop (warning)
     - Level 3: Platform-wide SOLD items (BLOCK)
     - Level 4: Platform-wide unsold items (BLOCK)

  5. Display check results:
     - Total uploaded
     - Valid items
     - Duplicates in product
     - Duplicates in shop
     - Platform duplicates (blocked)

  6. Confirm upload
  7. Insert product_items (encrypt content)
  8. Update product (stock += count, status = active)
  9. Check pre-orders → Auto-fulfill

MANAGE PRODUCTS:
  - List view with filters (status, category, stock level)
  - Edit: Update info, change price, toggle settings
  - Hide: status = hidden (preserves data)
  - Delete: Soft delete (status = deleted, keep 30 days)

VIEW INVENTORY DETAIL:
  - Tab: Unsold | Sold | On hold (pre-orders)
  - Unsold: Show full content, allow delete
  - Sold: Mask content, show order + buyer
  - Delete selected unsold items

RESTOCK:
  - Upload more items anytime
  - Auto-fulfill pending pre-orders (FIFO)


┌─────────────────────────────────────────────────────────────────────────────┐
│ 3. FLOW ĐƠN HÀNG (VENDOR VIEW)                                              │
└─────────────────────────────────────────────────────────────────────────────┘

VIEW ORDERS:
  1. /vendor/orders
  2. Dashboard stats:
     - Today: orders count, revenue
     - Pending: 0 (auto-delivery)
     - Disputes: count
  3. List with filters:
     - Status
     - Date range
     - Product
     - Buyer

VIEW ORDER DETAILS:
  1. Click order
  2. Show:
     - Buyer info (username, join date, total orders, dispute rate)
     - Order info (product, quantity, total)
     - Commission fee (5-10%)
     - Net amount
     - Payout status:
       - ⏳ Pending: Release in X days
       - ✅ Available: Can withdraw

  3. Show sold items (MASKED for security):
     - email@gm***.com|pass***|JBSW***

  Note: Vendor cannot see full content after sale


┌─────────────────────────────────────────────────────────────────────────────┐
│ 4. FLOW VÍ & RÚT TIỀN                                                       │
└─────────────────────────────────────────────────────────────────────────────┘

WALLET STRUCTURE:
  - Available Balance: Can withdraw (orders > 3 days)
  - Pending Balance: Holding (orders < 3 days)

PAYOUT FLOW:
  1. /vendor/payouts
  2. Show wallet summary:
     - Available: 2,000,000đ
     - Pending: 500,000đ (release in 2 days)

  3. Enter withdrawal amount:
     - Min: 100,000đ
     - Max: Available balance
     - Fee: 0đ if >= 500,000đ, else 10,000đ

  4. Validate:
     - Bank info configured
     - Within daily limits (3 requests, 50M max)

  5. Require 2FA (if enabled)

  6. Create withdrawal request (status: pending)
  7. Deduct from available_balance (hold)

  8. Notify admin

  9. Admin processes:
     a. Approve:
        - Transfer manually
        - status = completed
        - Email vendor
     b. Reject:
        - status = rejected
        - Refund to available_balance
        - Email vendor with reason


┌─────────────────────────────────────────────────────────────────────────────┐
│ 5. FLOW KHIẾU NẠI                                                          │
└─────────────────────────────────────────────────────────────────────────────┘

VIEW DISPUTES:
  1. /vendor/disputes
  2. List with urgency indicators:
     - 🔴 Urgent (>48h): Respond now!
     - 🟡 New: < 48h
     - 🟢 Responded: Waiting buyer

RESPOND TO DISPUTE (Must respond within 48h):
  1. Click dispute
  2. View buyer's claim:
     - Reason
     - Affected items
     - Description
     - Proof images
     - Requested action

  3. Choose action:

     a) ACCEPT FULL:
        - Agree to refund as requested
        - Money deducted from pending_balance
        - status = resolved
        - Notify buyer

     b) ACCEPT PARTIAL:
        - Propose partial refund amount
        - Enter reason
        - status = vendor_responded
        - Buyer can accept or escalate

     c) REJECT:
        - Enter rejection reason
        - Upload counter-proof
        - status = vendor_responded
        - Buyer can accept or escalate

     d) REPLACE ITEMS:
        - Upload replacement items
        - status = vendor_responded
        - Buyer confirms receipt

TIMELINE:
  T+0:    Buyer creates dispute
  T+48h:  Vendor must respond (else auto-escalate)
  T+72h:  Buyer must respond to vendor's response
  T+96h:  Auto-escalate if no resolution

ESCALATION:
  - Admin reviews evidence
  - Makes final decision:
    * REFUNDED: Full refund to buyer
    * PARTIAL_REFUND: Split difference
    * CLOSED: Reject dispute, release money


┌─────────────────────────────────────────────────────────────────────────────┐
│ 6. FLOW BÁO CÁO & THỐNG KÊ                                                  │
└─────────────────────────────────────────────────────────────────────────────┘

DASHBOARD:
  - Revenue today, this week, this month
  - Orders count
  - Average rating
  - Pending disputes
  - Low stock alerts

REVENUE REPORTS:
  - By time range
  - By product
  - By day
  - Export: Excel, PDF, CSV

PRODUCT REPORTS:
  - Total products
  - Active, out of stock, hidden
  - Low stock warning (< 10 items)
  - No sales in 7 days
  - Sort by: sales, revenue, dispute rate

SHOP ANALYTICS:
  - View count
  - Conversion rate
  - Traffic sources
  - Popular products
```

---

## 1.3 FLOW ADMIN (Quản trị viên)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          ADMIN - FULL FLOW                                  │
└─────────────────────────────────────────────────────────────────────────────┘

                    ┌──────────────┐
                    │  ADMIN       │
                    │  LOGIN       │
                    └──────┬───────┘
                           │
                           ▼
                    ┌──────────────┐
                    │ DASHBOARD    │
                    └──────┬───────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
        ▼                  ▼                  ▼
  ┌──────────┐      ┌──────────┐      ┌──────────┐
  │ USERS    │      │ VENDORS  │      │ SHOPS    │
  │ MANAGE   │      │ APPROVE  │      │ MANAGE   │
  └─────┬────┘      └─────┬────┘      └─────┬────┘
        │                  │                  │
        ▼                  ▼                  ▼
  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐
  │ [View users] │  │ [Pending     │  │ [View all    │
  │ [Edit user]  │  │  requests]   │  │  shops]      │
  │ [Ban user]   │  │ [Approve/    │  │ [Suspend     │
  │ [Roles]      │  │  Reject]     │  │  shop]       │
  └──────────────┘  └──────────────┘  └──────────────┘

                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
        ▼                  ▼                  ▼
  ┌──────────┐      ┌──────────┐      ┌──────────┐
  │ ORDERS   │      │ DEPOSITS │      │ PAYOUTS  │
  │ VIEW     │      │ APPROVE  │      │ PROCESS  │
  └─────┬────┘      └─────┬────┘      └─────┬────┘
        │                  │                  │
        ▼                  ▼                  ▼
  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐
  │ [All orders] │  │ [Manual      │  │ [Pending     │
  │ [By shop]    │  │  deposits]   │  │  requests]   │
  │ [By buyer]   │  │ [Verify      │  │ [Approve →   │
  │ [By status]  │  │  proof]      │  │  Transfer]   │
  └──────────────┘  │ [Credit      │  │ [Reject →    │
                    │  wallet]     │  │  Refund]     │
                    └──────────────┘  └──────────────┘

                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
        ▼                  ▼                  ▼
  ┌──────────┐      ┌──────────┐      ┌──────────┐
  │ DISPUTES │      │ PRODUCTS │      │ REPORTS  │
  │ RESOLVE  │      │ MANAGE   │      │ VIEW     │
  └─────┬────┘      └─────┬────┘      └─────┬────┘
        │                  │                  │
        ▼                  ▼                  ▼
  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐
  │ [Escalated]  │  │ [Categories] │  │ [Revenue]    │
  │ [Pending     │  │ [All         │  │ [Sales]      │
  │  > 48h]      │  │  products]   │  │ [Disputes]   │
  │ [Admin       │  │ [Delete      │  │ [Users]      │
  │  decision]   │  │ 违规 items]  │  │ [Payouts]    │
  └──────────────┘  └──────────────┘  └──────────────┘

                           │
                           ▼
                    ┌──────────────┐
                    │ SETTINGS     │
                    │              │
                    │ [Site config]│
                    │ [Payment]    │
                    │ [Commission] │
                    │ [Email]      │
                    └──────────────┘
```

### Admin Flows Chi Tiết

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     ADMIN - FLOWS CHI TIẾT                                │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│ 1. FLOW QUẢN LÝ NGƯỜI DÙNG                                                │
└─────────────────────────────────────────────────────────────────────────────┘

USERS MANAGEMENT:
  1. /admin/users
  2. List with filters:
     - Search by username/email
     - Filter by role
     - Filter by status
     - Filter by date range
  3. Actions:

  VIEW USER DETAIL:
    - Profile info
    - Roles
    - Wallet balance
    - Login history
    - Transaction history
    - Orders (if buyer)
    - Shop (if vendor)
    - Disputes

  EDIT USER:
    - Update profile
    - Add/remove roles
    - Adjust wallet (with reason)

  BAN USER:
    - Select reason
    - Choose duration (temporary/permanent)
    - Notify user
    - Log action


┌─────────────────────────────────────────────────────────────────────────────┐
│ 2. FLOW DUYỆT VENDOR                                                        │
└─────────────────────────────────────────────────────────────────────────────┘

VENDOR REGISTRATIONS (V1 only - V2 auto approves):
  [V2 FLOW: Auto approve when shop created]

VIEW VENDORS:
  1. /admin/vendors
  2. List all vendors
  3. Show stats:
     - Total vendors
     - Active today
     - New this week
     - Pending disputes

  VENDOR DETAIL:
    - User info
    - Shop info
    - Total sales
    - Rating
    - Dispute rate
    - Payout history
    - Login history


┌─────────────────────────────────────────────────────────────────────────────┐
│ 3. FLOW QUẢN LÝ SHOPS                                                       │
└─────────────────────────────────────────────────────────────────────────────┘

SHOPS MANAGEMENT:
  1. /admin/shops
  2. List all shops
  3. Filters: status, level, date range

  SHOP DETAIL:
    - Shop info
    - Vendor info
    - Products count
    - Total sales
    - Rating
    - Commission rate
    - Level (New → Silver → Gold → Diamond → Partner)

  SUSPEND SHOP:
    - Select reason
    - Set duration
    - Notify vendor
    - Hide all products

  DELETE SHOP:
    - Only if no active orders
    - Soft delete
    - Keep data for 30 days


┌─────────────────────────────────────────────────────────────────────────────┐
│ 4. FLOW NẠP TIỀN (MANUAL)                                                  │
└─────────────────────────────────────────────────────────────────────────────┘

MANUAL DEPOSITS:
  1. /admin/deposits/manual
  2. List pending manual requests
  3. Or create new:

  CREATE MANUAL DEPOSIT:
    - Select user (search)
    - Enter amount
    - Upload proof (bank transfer screenshot)
    - Enter reason
    - Submit

  4. System:
     - Create transaction (type: deposit_manual)
     - Credit wallet
     - Notify user
     - Log admin action


┌─────────────────────────────────────────────────────────────────────────────┐
│ 5. FLOW RÚT TIỀN                                                           │
└─────────────────────────────────────────────────────────────────────────────┘

PAYOUT REQUESTS:
  1. /admin/payouts
  2. List pending withdrawals

  PROCESS WITHDRAWAL:
    - View vendor info
    - View bank details
    - View withdrawal history
    - Check:

    APPROVE:
      1. Transfer money via bank
      2. Enter transaction reference
      3. Update status = completed
      4. Email vendor: "Payment sent"
      5. Log action

    REJECT:
      1. Enter reason
      2. Update status = rejected
      3. Refund to vendor's available_balance
      4. Email vendor with reason
      5. Log action


┌─────────────────────────────────────────────────────────────────────────────┐
│ 6. FLOW XỬ LÝ DISPUTE                                                       │
└─────────────────────────────────────────────────────────────────────────────┘

DISPUTES DASHBOARD:
  1. /admin/disputes
  2. Priority indicators:
     - 🔴 Critical: > 48h without vendor response
     - 🟡 Escalated: Needs admin attention
     - 🟢 New: Recently created

  VIEW DISPUTE:
    - Buyer info
    - Vendor info
    - Order details
    - Dispute reason
    - Items affected
    - Evidence from both sides
    - Communication history

  ADMIN DECISION:
    Options:

    a) REFUND BUYER (Full):
       - Deduct from vendor pending_balance
       - Credit buyer wallet
       - Update order status = refunded
       - Update dispute status = refunded
       - Notify both parties

    b) PARTIAL REFUND:
       - Enter refund amount
       - Deduct from vendor
       - Credit buyer (partial)
       - Update order status = partial_refund
       - Update dispute status = partial_refund
       - Notify both parties

    c) REJECT DISPUTE:
       - Enter reason
       - Update dispute status = closed
       - Release money to vendor
       - Update order status = completed
       - Notify both parties

    d) REQUEST MORE INFO:
       - Ask buyer/vendor for more evidence
       - Pause escalation timer


┌─────────────────────────────────────────────────────────────────────────────┐
│ 7. FLOW SẢN PHẨM                                                           │
└─────────────────────────────────────────────────────────────────────────────┘

PRODUCTS MANAGEMENT:
  1. /admin/products
  2. List all products across all shops
  3. Filters:
     - Shop
     - Category
     - Status
     - Price range
     - Stock level

  DELETE VIOLATING ITEMS:
    - Select product_items
    - Bulk delete
    - Notify vendor
    - Update product stock

  CATEGORIES:
    - Create/Edit/Delete categories
    - Set commission rates per category


┌─────────────────────────────────────────────────────────────────────────────┐
│ 8. FLOW BÁO CÁO                                                            │
└─────────────────────────────────────────────────────────────────────────────┘

REPORTS DASHBOARD:
  1. /admin/reports

  REVENUE REPORT:
    - Platform revenue (commission)
    - By date range
    - By shop
    - By category
    - Export options

  SALES REPORT:
    - Total orders
    - Total sales volume
    - Average order value
    - By product
    - By shop

  DISPUTE REPORT:
    - Total disputes
    - Resolution rate
    - By shop (dispute rate)
    - By reason
    - Average resolution time

  USER REPORT:
    - New users
    - Active users
    - By role
    - Growth trend

  PAYOUT REPORT:
    - Total payouts
    - Pending payouts
    - By vendor
    - By date range
```

---

# PHẦN 2: DEPENDENCIES GIỮA CÁC MODULES

## 2.1 Dependency Graph

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      DEPENDENCY GRAPH                                      │
└─────────────────────────────────────────────────────────────────────────────┘

                    ┌──────────────┐
                    │   CORE BASE  │
                    │              │
                    │ • User Model │
                    │ • Auth (JWT) │
                    │ • DB Setup   │
                    │ • Redis      │
                    └──────┬───────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
        ▼                  ▼                  ▼
  ┌──────────┐      ┌──────────┐      ┌──────────┐
  │ WALLET   │      │ 2FA      │      │ ROLES    │
  │ MODULE   │      │ MODULE   │      │ MODULE   │
  └─────┬────┘      └─────┬────┘      └─────┬────┘
        │                  │                  │
        └──────────────────┼──────────────────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
        ▼                  ▼                  ▼
  ┌──────────┐      ┌──────────┐      ┌──────────┐
  │ AUTH     │      │ EMAIL    │      │ NOTIF    │
  │ FLOW     │      │ SERVICE  │      │ SERVICE  │
  └─────┬────┘      └─────┬────┘      └─────┬────┘
        │                  │                  │
        └──────────────────┼──────────────────┘
                           │
                           ▼
                    ┌──────────────┐
                    │   SHOP       │
                    │   MODULE     │
                    └──────┬───────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
        ▼                  ▼                  ▼
  ┌──────────┐      ┌──────────┐      ┌──────────┐
  │ TELEGRAM │      │ PRODUCT  │      │ CATEGORY │
  │ BOT      │      │ MODULE   │      │ MODULE   │
  └─────┬────┘      └─────┬────┘      └──────────┘
        │                  │
        └──────────────────┼──────────────────┐
                           │                  │
                           ▼                  ▼
                    ┌──────────────┐   ┌──────────────┐
                    │ INVENTORY   │   │ ORDER        │
                    │ MODULE      │   │ MODULE       │
                    └──────┬───────┘   └──────┬───────┘
                           │                  │
                           └──────────────────┼──────────────────┐
                                              │                  │
                                              ▼                  ▼
                                       ┌──────────────┐   ┌──────────────┐
                                       │ DISPUTE     │   │ REVIEW       │
                                       │ MODULE      │   │ MODULE       │
                                       └──────┬───────┘   └──────┬───────┘
                                              │                  │
                                              └──────────────────┼──────────────────┐
                                                                 │                  │
                                                                 ▼                  ▼
                                                          ┌──────────────┐   ┌──────────────┐
                                                          │ PRE-ORDER    │   │ REPORTS      │
                                                          │ MODULE       │   │ ANALYTICS    │
                                                          └──────────────┘   └──────────────┘
```

## 2.2 Module Dependencies Table

| Module | Requires | Required By |
|--------|----------|-------------|
| **Base** | - | All modules |
| **User** | Base | Wallet, Roles, Auth |
| **Wallet** | User | Order, Dispute, Payout |
| **Roles** | Base | Auth, Shop, All |
| **2FA** | User, Wallet | Auth (optional) |
| **Auth** | User, Roles (Optional: 2FA) | All protected routes |
| **Email** | Base | Auth, Order, Dispute |
| **Notification** | Base, Telegram | Order, Dispute, Shop |
| **Shop** | Auth (vendor), User | Product, Order, Report |
| **Telegram** | Shop | Shop (verification) |
| **Category** | Auth (admin) | Product |
| **Product** | Auth (vendor), Shop, Category | Inventory, Order, Review |
| **Inventory** | Auth (vendor), Product | Order, Pre-order |
| **Order** | Auth, Wallet, Inventory | Dispute, Review, Payout |
| **Dispute** | Auth, Order | - |
| **Review** | Auth, Order | Shop (rating) |
| **Pre-order** | Auth, Order, Inventory | - |
| **Payout** | Auth (vendor), Wallet | - |
| **Report** | Auth (vendor/admin), Order, Product | - |

---

# PHẦN 3: THỨ TỰ IMPLEMENTATION

## 3.1 Đề xuất thứ tự (Recommended)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│              IMPLEMENTATION PHASES (V2 FOR RUST)                            │
└─────────────────────────────────────────────────────────────────────────────┘

PHASE 0: FOUNDATION (Week 1-2)
├─ 1. Project Setup
│  ├─ Init Rust project (cargo new)
│  ├─ Add dependencies (actix-web, mongo, redis, jwt)
│  ├─ Config structure (dotenv)
│  ├─ DB connection pool
│  └─ Redis connection
│
├─ 2. Base Models
│  ├─ User model (MongoDB schema)
│  ├─ Wallet model
│  ├─ Base DTOs
│  └─ Error types
│
└─ 3. Core Middleware
   ├─ JWT authentication
   ├─ Role-based authorization
   └─ Error handlers


PHASE 1: AUTH & USER (Week 3-4)
├─ 1. User Module
│  ├─ Register handler
│  ├─ Login handler
│  ├─ Password hashing (bcrypt)
│  ├─ Email verification
│  └─ Password reset
│
├─ 2. 2FA Module (Optional but important)
│  ├─ Generate secret
│  ├─ Create QR code
│  ├─ Verify TOTP
│  └─ Backup codes
│
├─ 3. Wallet Module (Basic)
│  ├─ Create wallet with user
│  ├─ View balance
│  └─ Transaction history
│
└─ 4. Profile Management
   ├─ Update profile
   ├─ Change password
   └─ Session management


PHASE 2: SHOP & PRODUCT (Week 5-7)
├─ 1. Shop Module
│  ├─ Create shop (4-step wizard)
│  ├─ Update shop
│  ├─ View shop (public)
│  └─ Shop dashboard
│
├─ 2. Telegram Integration
│  ├─ Generate verification code
│  ├─ Bot command handler
│  ├─ Verify telegram
│  └─ Send notifications
│
├─ 3. Category Module
│  ├─ CRUD categories
│  └─ Set commission rates
│
├─ 4. Product Module
│  ├─ Create product
│  ├─ Update product
│  ├─ List products
│  ├─ View product (public)
│  └─ Product search/filter
│
└─ 5. Inventory Module
   ├─ Bulk upload (paste/file)
   ├─ Duplicate check (4 levels)
   ├─ View inventory
   ├─ Delete items
   └─ Auto-fulfill pre-orders


PHASE 3: ORDER & PAYMENT (Week 8-10)
├─ 1. Deposit (Wallet)
│  ├─ Bank transfer webhook
│  ├─ Momo webhook
│  ├─ USDT monitoring
│  └─ Manual deposit (admin)
│
├─ 2. Order Module
│  ├─ Create order (with transaction)
│  ├─ Lock stock (pessimistic)
│  ├─ Auto-delivery
│  ├─ View orders (buyer/vendor)
│  ├─ Order status flow
│  └─ Auto-complete (cron)
│
├─ 3. Coupon Module (Optional)
│  ├─ Create coupon
│  ├─ Validate coupon
│  └─ Apply discount
│
└─ 4. Pre-order Module
   ├─ Create pre-order
   ├─ Auto-fulfill on restock
   ├─ Auto-expire (cron)
   └─ Cancel pre-order


PHASE 4: DISPUTE & REVIEW (Week 11-12)
├─ 1. Dispute Module
│  ├─ Create dispute
│  ├─ Vendor response
│  ├─ Escalate to admin
│  ├─ Admin decision
│  ├─ Refund processing
│  └─ Auto-escalate (cron)
│
├─ 2. Review Module
│  ├─ Create review
│  ├─ View reviews (product/shop)
│  ├─ Vendor reply
│  ├─ Calculate ratings
│  └─ Filter/sort
│
└─ 3. Payout Module
   ├─ Request withdrawal
   ├─ Admin approve/reject
   ├─ Bank transfer
   └─ Withdrawal history


PHASE 5: ADMIN & REPORTS (Week 13-14)
├─ 1. Admin Dashboard
│  ├─ Stats overview
│  ├─ User management
│  ├─ Vendor management
│  └─ Shop management
│
├─ 2. Admin Actions
│  ├─ Manual deposit
│  ├─ Process payout
│  ├─ Suspend shop/ban user
│  └─ Adjust wallet
│
├─ 3. Reports (Vendor)
│  ├─ Revenue report
│  ├─ Product report
│  ├─ Export (Excel/PDF)
│  └─ Dashboard charts
│
└─ 4. Reports (Admin)
   ├─ Platform revenue
   ├─ Sales report
   ├─ Dispute report
   ├─ User report
   └─ Payout report


PHASE 6: OPTIONAL FEATURES (Week 15+)
├─ 1. Reseller Module (V1 only, removed in V2)
├─ 2. Advanced Analytics
├─ 3. Recommendation System
├─ 4. Chat/Messaging
└─ 5. Mobile App API
```

## 3.2 Critical Path Analysis

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    CRITICAL PATH FOR MVP                                   │
└─────────────────────────────────────────────────────────────────────────────┘

Must-have for minimum viable marketplace:

1. USER + AUTH
   ↓
2. WALLET + DEPOSIT (Bank only)
   ↓
3. SHOP + TELEGRAM (V2 requirement)
   ↓
4. PRODUCT + INVENTORY
   ↓
5. ORDER + AUTO-DELIVERY
   ↓
6. DISPUTE (basic)
   ↓
7. PAYOUT
   ↓
8. ADMIN (basic)

Nice-to-have (can be deferred):

- 2FA
- Momo/USDT deposit
- Pre-order
- Reviews
- Reports
- Coupons
```

## 3.3 Parallel Development Opportunities

```
┌─────────────────────────────────────────────────────────────────────────────┐
│              PARALLEL DEVELOPMENT TEAMS                                    │
└─────────────────────────────────────────────────────────────────────────────┘

TEAM A: Core (Backend)
├─ User, Auth, Wallet
├─ Shop, Product, Inventory
└─ Order, Payment

TEAM B: Integration (External)
├─ Telegram Bot
├─ Banking APIs
├─ Email service
└─ Payment gateways

TEAM C: Resolution (User-facing)
├─ Dispute
├─ Review
└─ Payout

TEAM D: Admin (Internal)
├─ User management
├─ Shop management
├─ Deposit approval
├─ Dispute resolution
└─ Reports

TEAM E: Frontend (Can work in parallel with backend)
├─ User interface
├─ Vendor dashboard
├─ Admin panel
└─ Mobile responsive
```

---

# PHẦN 4: MÔ HÌNH DỮ LIỆU TÓM TẮT

## 4.1 Collections MongoDB

```javascript
// Users
{
  _id: ObjectId,
  username: String (unique),
  email: String (unique),
  password_hash: String,
  role: ["buyer", "vendor", "admin"],
  status: "active" | "suspended" | "pending_verification",
  profile: {
    full_name: String,
    phone: String,
    avatar: String
  },
  two_factor_enabled: Boolean,
  two_factor_secret: String (encrypted),
  backup_codes: [String] (encrypted),
  created_at: DateTime,
  updated_at: DateTime
}

// Wallets
{
  _id: ObjectId,
  user_id: ObjectId (ref: users),
  balance: Decimal128,
  available_balance: Decimal128, // vendor only
  pending_balance: Decimal128, // vendor only
  created_at: DateTime,
  updated_at: DateTime
}

// Transactions
{
  _id: ObjectId,
  user_id: ObjectId (ref: users),
  type: "deposit" | "purchase" | "sale" | "refund" | "withdraw" | "preorder_hold",
  amount: Decimal128,
  balance_after: Decimal128,
  method: String, // for deposits
  status: "pending" | "completed" | "failed",
  reference_id: ObjectId, // order_id, dispute_id, etc.
  created_at: DateTime
}

// Shops
{
  _id: ObjectId,
  vendor_id: ObjectId (ref: users),
  shop_name: String,
  shop_slug: String (unique),
  description: String,
  logo: String,
  banner: String,
  telegram_username: String,
  telegram_chat_id: String,
  telegram_verified: Boolean,
  status: "active" | "suspended" | "inactive",
  rating: Decimal128,
  total_reviews: Number,
  total_sales: Number,
  total_products: Number,
  commission_rate: Decimal128,
  level: "new" | "silver" | "gold" | "diamond" | "partner",
  created_at: DateTime,
  updated_at: DateTime
}

// Products
{
  _id: ObjectId,
  shop_id: ObjectId (ref: shops),
  category_id: ObjectId,
  name: String,
  slug: String,
  description: String,
  image: String,
  price: Decimal128,
  original_price: Decimal128,
  min_purchase: Number,
  max_purchase: Number,
  stock: Number,
  allow_preorder: Boolean,
  allow_resell: Boolean, // V1 only
  hide_stock: Boolean,
  require_2fa: Boolean,
  status: "draft" | "active" | "hidden" | "deleted",
  avg_rating: Decimal128,
  total_reviews: Number,
  total_sold: Number,
  created_at: DateTime,
  updated_at: DateTime
}

// Product Items (Inventory)
{
  _id: ObjectId,
  product_id: ObjectId (ref: products),
  content: String (encrypted),
  content_hash: String (SHA256),
  is_sold: Boolean,
  hold_until: DateTime (for pre-orders),
  order_id: ObjectId (ref: orders),
  sold_at: DateTime,
  created_at: DateTime
}

// Orders
{
  _id: ObjectId,
  order_number: String (unique),
  buyer_id: ObjectId (ref: users),
  shop_id: ObjectId (ref: shops),
  product_id: ObjectId (ref: products),
  quantity: Number,
  unit_price: Decimal128,
  subtotal: Decimal128,
  discount: Decimal128,
  total: Decimal128,
  status: "pending" | "paid" | "delivered" | "disputed" | "completed" | "refunded" | "cancelled",
  dispute_deadline: DateTime,
  delivered_at: DateTime,
  completed_at: DateTime,
  preorder_id: ObjectId,
  created_at: DateTime,
  updated_at: DateTime
}

// Order Items
{
  _id: ObjectId,
  order_id: ObjectId (ref: orders),
  product_item_id: ObjectId (ref: product_items),
  content: String (copy from product_item),
  created_at: DateTime
}

// Pre-orders
{
  _id: ObjectId,
  pre_order_number: String (unique),
  buyer_id: ObjectId (ref: users),
  shop_id: ObjectId (ref: shops),
  product_id: ObjectId (ref: products),
  quantity: Number,
  unit_price: Decimal128,
  total_amount: Decimal128,
  status: "pending" | "fulfilled" | "cancelled" | "expired" | "refunded",
  wait_days: Number,
  expires_at: DateTime,
  fulfilled_at: DateTime,
  order_id: ObjectId (ref: orders),
  created_at: DateTime
}

// Disputes
{
  _id: ObjectId,
  dispute_number: String (unique),
  buyer_id: ObjectId (ref: users),
  vendor_id: ObjectId (ref: users),
  order_id: ObjectId (ref: orders),
  shop_id: ObjectId (ref: shops),
  reason: String,
  affected_quantity: Number,
  description: String,
  evidence: [String], // image URLs
  requested_action: String,
  requested_amount: Decimal128,
  status: "pending" | "vendor_responded" | "escalated" | "admin_review" | "resolved" | "refunded" | "partial_refund" | "rejected" | "closed",
  escalated_at: DateTime,
  escalated_by: String,
  admin_decision: String,
  admin_amount: Decimal128,
  created_at: DateTime,
  updated_at: DateTime
}

// Dispute Messages
{
  _id: ObjectId,
  dispute_id: ObjectId (ref: disputes),
  user_id: ObjectId (ref: users),
  role: "buyer" | "vendor" | "admin",
  message: String,
  attachments: [String],
  created_at: DateTime
}

// Reviews
{
  _id: ObjectId,
  order_id: ObjectId (ref: orders),
  product_id: ObjectId (ref: products),
  shop_id: ObjectId (ref: shops),
  buyer_id: ObjectId (ref: users),
  rating: Number (1-5),
  comment: String,
  tags: [String],
  images: [String],
  is_anonymous: Boolean,
  vendor_reply: String,
  helpful_count: Number,
  created_at: DateTime
}

// Withdrawals
{
  _id: ObjectId,
  vendor_id: ObjectId (ref: users),
  amount: Decimal128,
  fee: Decimal128,
  net_amount: Decimal128,
  bank_name: String,
  bank_account_number: String,
  bank_account_name: String,
  status: "pending" | "processing" | "completed" | "rejected" | "cancelled",
  transaction_ref: String,
  rejection_reason: String,
  processed_by: ObjectId (ref: users), // admin
  processed_at: DateTime,
  created_at: DateTime
}

// Categories
{
  _id: ObjectId,
  name: String,
  slug: String,
  parent_id: ObjectId,
  commission_rate: Decimal128,
  icon: String,
  description: String,
  status: "active" | "inactive",
  sort_order: Number,
  created_at: DateTime
}

// Coupons
{
  _id: ObjectId,
  code: String (unique),
  shop_id: ObjectId (ref: shops), // null for platform coupons
  type: "fixed" | "percentage",
  value: Decimal128,
  min_purchase: Decimal128,
  max_discount: Decimal128,
  usage_limit: Number,
  used_count: Number,
  valid_from: DateTime,
  valid_until: DateTime,
  status: "active" | "inactive",
  created_at: DateTime
}

// Settings (Platform)
{
  _id: ObjectId,
  key: String (unique),
  value: String / JSON,
  description: String,
  updated_at: DateTime
}
```

---

# PHẦN 5: KEY DIFFERENCES V1 vs V2

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         V1 vs V2 CHANGES                                   │
└─────────────────────────────────────────────────────────────────────────────┘

┌──────────────────┬─────────────────────────┬─────────────────────────┐
│     FEATURE      │          V1             │          V2             │
├──────────────────┼─────────────────────────┼─────────────────────────┤
│ Vendor Sign-up   │ Form + Admin approval   │ Auto on shop creation    │
├──────────────────┼─────────────────────────┼─────────────────────────┤
│ Reseller System  │ Full system             │ ❌ REMOVED              │
├──────────────────┼─────────────────────────┼─────────────────────────┤
│ Telegram         │ Optional                │ ✅ REQUIRED              │
├──────────────────┼─────────────────────────┼─────────────────────────┤
│ Shop Completion  │ Created = done          │ Requires:               │
│                  │                         │ - Telegram verified    │
│                  │                         │ - Has products          │
│                  │                         │ - Policies set          │
├──────────────────┼─────────────────────────┼─────────────────────────┤
│ Tech Stack       │ Laravel + MySQL         │ Rust + MongoDB          │
├──────────────────┼─────────────────────────┼─────────────────────────┤
│ Architecture    │ Monolith                │ Module-based            │
└──────────────────┴─────────────────────────┴─────────────────────────┘
```

---

# REFS

- [Authentication V1](../v1/01-authentication.md)
- [User Roles V1](../v1/02-user-roles.md)
- [Shop Management V1](../v1/03-shop-management.md)
- [Products & Inventory V1](../v1/04-products-inventory.md)
- [Orders V1](../v1/05-orders.md)
- [Wallet & Payment V1](../v1/06-wallet-payment.md)
- [Pre-order V1](../v1/07-preorder.md)
- [Disputes V1](../v1/08-disputes.md)
- [Reviews V1](../v1/09-reviews.md)
- [Shop Flows V2](./shop/01-complete-flows.md)
- [Wallet System V2](./01-wallet-system-design.md)
- [User to Complete Shop V2](./user-to-complete-shop-flow.md)
