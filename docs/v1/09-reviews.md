# Chức năng Đánh giá (Reviews & Ratings)

## Tổng quan

Hệ thống đánh giá cho phép Buyer review sản phẩm sau khi mua, giúp xây dựng uy tín cho Vendor và hỗ trợ quyết định mua của các buyer khác. Chỉ buyer đã mua hàng thực sự mới có thể đánh giá.

---

## 1. Điều kiện đánh giá

### 1.1 Khi nào có thể đánh giá

```
┌─────────────────────────────────────────────────────────────────┐
│               ĐIỀU KIỆN TẠO ĐÁNH GIÁ                            │
└─────────────────────────────────────────────────────────────────┘

Buyer có thể đánh giá khi TẤT CẢ điều kiện sau thỏa mãn:

1. Order Status:
   ├── status = 'delivered' HOẶC
   └── status = 'completed'

2. Chưa đánh giá:
   └── Chưa có review cho order này

3. Không có dispute đang mở:
   └── Không có dispute pending/escalated

4. Thời hạn:
   └── Trong vòng 30 ngày kể từ delivered_at
```

### 1.2 Cấu trúc đánh giá

| Thành phần | Mô tả | Bắt buộc |
|------------|-------|----------|
| Rating | 1-5 sao | Có |
| Comment | Nhận xét text | Không |
| Tags | Tags nhanh có sẵn | Không |
| Images | Ảnh minh họa | Không |

---

## 2. Flow tạo đánh giá

### 2.1 Buyer tạo review

```
┌─────────────────────────────────────────────────────────────────┐
│                   FLOW TẠO ĐÁNH GIÁ                             │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Buyer vào đơn hàng đã delivered/completed
         │
         ▼
[Bước 2] Kiểm tra điều kiện đánh giá:
         │
         ├── Đã đánh giá ──► Hiển thị review đã tạo
         ├── Có dispute pending ──► Ẩn nút đánh giá
         ├── Quá 30 ngày ──► "Đã hết hạn đánh giá"
         │
         ▼
[Bước 3] Click "Đánh giá sản phẩm"
         │
         ▼
[Bước 4] Hiển thị form đánh giá:

         ╔═══════════════════════════════════════════════════════╗
         ║  ĐÁNH GIÁ SẢN PHẨM                                    ║
         ╠═══════════════════════════════════════════════════════╣
         ║                                                       ║
         ║  Đơn hàng: #ORD-20240115-12345                       ║
         ║  Sản phẩm: Gmail US Aged x 10                        ║
         ║  Shop: TechAccount                                    ║
         ║                                                       ║
         ║  ─────────────────────────────────────────────────── ║
         ║  ĐÁNH GIÁ CỦA BẠN *                                  ║
         ║                                                       ║
         ║  Chất lượng sản phẩm:  ☆ ☆ ☆ ☆ ☆                    ║
         ║                        (Click để chọn số sao)        ║
         ║                                                       ║
         ║  ─────────────────────────────────────────────────── ║
         ║  TAGS NHANH (chọn những gì phù hợp):                 ║
         ║  □ Hàng đúng mô tả    □ Giao hàng nhanh              ║
         ║  □ Chất lượng tốt     □ Giá hợp lý                   ║
         ║  □ Sẽ mua lại         □ Hỗ trợ nhiệt tình            ║
         ║                                                       ║
         ║  ─────────────────────────────────────────────────── ║
         ║  NHẬN XÉT (tùy chọn):                                ║
         ║  ┌───────────────────────────────────────────────────┐║
         ║  │ Chia sẻ trải nghiệm của bạn về sản phẩm...       │║
         ║  │                                                   │║
         ║  └───────────────────────────────────────────────────┘║
         ║  0/500 ký tự                                          ║
         ║                                                       ║
         ║  ─────────────────────────────────────────────────── ║
         ║  HÌNH ẢNH (tùy chọn):                                ║
         ║  [📷 Thêm ảnh] (tối đa 3 ảnh)                        ║
         ║                                                       ║
         ║  □ Đánh giá ẩn danh                                   ║
         ║                                                       ║
         ║  [Hủy]  [Gửi đánh giá]                                ║
         ╚═══════════════════════════════════════════════════════╝
         │
         ▼
[Bước 5] Buyer chọn rating và điền thông tin
         │
         ▼
[Bước 6] Validate:
         - Rating bắt buộc (1-5)
         - Comment max 500 ký tự
         - Images max 3, mỗi file max 5MB
         │
         ▼
[Bước 7] Upload images (nếu có)
         │
         ▼
[Bước 8] Tạo Review record:
         - order_id
         - product_id
         - shop_id
         - buyer_id
         - rating
         - comment
         - tags (JSON array)
         - is_anonymous
         │
         ▼
[Bước 9] Cập nhật thống kê:
         
         Product:
         └── Tính lại avg_rating và total_reviews
         
         Shop:
         └── Tính lại rating và total_reviews
         │
         ▼
[Bước 10] Gửi notification cho vendor:
          "Có đánh giá mới cho sản phẩm [name]"
          │
          ▼
[Bước 11] Hiển thị "Cảm ơn bạn đã đánh giá!"
```

### 2.2 Logic tính rating

```
┌─────────────────────────────────────────────────────────────────┐
│                 LOGIC TÍNH RATING                               │
└─────────────────────────────────────────────────────────────────┘

Product Rating:
─────────────────────────────────────────────────────────────────
avg_rating = SUM(all_reviews.rating) / COUNT(all_reviews)

Làm tròn đến 1 decimal: ROUND(avg_rating, 1)

Shop Rating (Weighted):
─────────────────────────────────────────────────────────────────
Công thức có trọng số:

shop_rating = (
  avg_review_rating * 0.7 +     -- 70% từ reviews
  completion_rate * 0.2 +        -- 20% từ tỷ lệ hoàn thành
  response_rate * 0.1            -- 10% từ tỷ lệ phản hồi
)

Trong đó:
- avg_review_rating = TB tất cả reviews (1-5)
- completion_rate = (completed_orders / total_orders) * 5
- response_rate = (responded_disputes_in_24h / total_disputes) * 5

Ví dụ:
avg_review = 4.5
completion = 98% → 4.9
response = 90% → 4.5

shop_rating = (4.5 * 0.7) + (4.9 * 0.2) + (4.5 * 0.1)
            = 3.15 + 0.98 + 0.45
            = 4.58 → 4.6
```

---

## 3. Hiển thị Reviews

### 3.1 Reviews trên trang sản phẩm

```
┌─────────────────────────────────────────────────────────────────┐
│              HIỂN THỊ REVIEWS TRÊN SẢN PHẨM                     │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  ĐÁNH GIÁ SẢN PHẨM                                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌────────────────┐  ┌─────────────────────────────────────┐   │
│  │                │  │  Filter:                            │   │
│  │   ⭐ 4.8       │  │  [Tất cả ▼] [Mới nhất ▼]           │   │
│  │   234 đánh giá │  │                                     │   │
│  │                │  │  ⭐⭐⭐⭐⭐  180 (77%)  ████████░   │   │
│  │  [Viết đánh    │  │  ⭐⭐⭐⭐☆   40 (17%)  ███░░░░░░   │   │
│  │   giá]         │  │  ⭐⭐⭐☆☆   10 (4%)   █░░░░░░░░   │   │
│  │                │  │  ⭐⭐☆☆☆    3 (1%)   ░░░░░░░░░   │   │
│  └────────────────┘  │  ⭐☆☆☆☆    1 (0%)   ░░░░░░░░░   │   │
│                      └─────────────────────────────────────┘   │
│                                                                 │
│  Tags phổ biến: #Hàng_đúng_mô_tả (120) #Chất_lượng_tốt (95)   │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │ user***23              ⭐⭐⭐⭐⭐     15/01/2024          │ │
│  │ Đã mua: Gmail US Aged x 10                                │ │
│  │                                                           │ │
│  │ Hàng chất lượng, đăng nhập được hết, sẽ ủng hộ tiếp!     │ │
│  │                                                           │ │
│  │ #Hàng_đúng_mô_tả #Chất_lượng_tốt #Sẽ_mua_lại            │ │
│  │                                                           │ │
│  │ 👍 Hữu ích (5)                                            │ │
│  └───────────────────────────────────────────────────────────┘ │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │ Người mua ẩn danh        ⭐⭐⭐⭐☆     14/01/2024          │ │
│  │ Đã mua: Gmail US Aged x 5                                 │ │
│  │                                                           │ │
│  │ Hàng ổn, có 1 tài khoản phải đổi pass nhưng shop hỗ trợ │ │
│  │ nhanh.                                                    │ │
│  │                                                           │ │
│  │ [📷 Xem 2 ảnh]                                            │ │
│  │                                                           │ │
│  │ 👍 Hữu ích (2)                                            │ │
│  └───────────────────────────────────────────────────────────┘ │
│                                                                 │
│  [Xem thêm 230 đánh giá...]                                    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 Reviews trên trang Shop

```
┌─────────────────────────────────────────────────────────────────┐
│                   REVIEWS CỦA SHOP                              │
└─────────────────────────────────────────────────────────────────┘

Trang shop hiển thị:
- Tổng số reviews của tất cả sản phẩm
- Rating trung bình của shop
- Reviews gần nhất
- Filter theo sản phẩm

Tab "Đánh giá" trên trang shop:
┌─────────────────────────────────────────────────────────────────┐
│  Filter: [Tất cả sản phẩm ▼] [Rating ▼] [Mới nhất ▼]          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Reviews được group theo sản phẩm hoặc hiển thị timeline       │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 4. Vendor phản hồi Review

### Flow vendor reply

```
┌─────────────────────────────────────────────────────────────────┐
│               FLOW VENDOR PHẢN HỒI REVIEW                       │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] Vendor vào Reviews của shop
         │
         ▼
[Bước 2] Xem danh sách reviews:
         - Chưa phản hồi
         - Đã phản hồi
         - Rating thấp (1-3 sao)
         │
         ▼
[Bước 3] Click "Phản hồi" trên review
         │
         ▼
[Bước 4] Hiển thị form phản hồi:
         
         ╔═══════════════════════════════════════════════════════╗
         ║  PHẢN HỒI ĐÁNH GIÁ                                    ║
         ╠═══════════════════════════════════════════════════════╣
         ║                                                       ║
         ║  Review của user***23:                                ║
         ║  ⭐⭐⭐⭐☆ "Hàng ổn, có 1 tài khoản..."              ║
         ║                                                       ║
         ║  Phản hồi của bạn:                                    ║
         ║  ┌───────────────────────────────────────────────────┐║
         ║  │ Cảm ơn bạn đã ủng hộ shop! Rất vui vì đã hỗ trợ │║
         ║  │ bạn kịp thời. Hẹn gặp lại! ❤️                     │║
         ║  └───────────────────────────────────────────────────┘║
         ║  0/300 ký tự                                          ║
         ║                                                       ║
         ║  [Hủy]  [Gửi phản hồi]                                ║
         ╚═══════════════════════════════════════════════════════╝
         │
         ▼
[Bước 5] Validate:
         - Max 300 ký tự
         - Không chứa link, số điện thoại (tùy chọn)
         │
         ▼
[Bước 6] Lưu vendor reply
         │
         ▼
[Bước 7] Gửi notification cho buyer:
         "Shop đã phản hồi đánh giá của bạn"

Lưu ý:
- Vendor chỉ có thể phản hồi 1 lần per review
- Không thể edit/delete phản hồi
- Admin có thể ẩn phản hồi nếu vi phạm
```

---

## 5. Chỉnh sửa/Xóa Review

### 5.1 Buyer chỉnh sửa

```
┌─────────────────────────────────────────────────────────────────┐
│                FLOW CHỈNH SỬA REVIEW                            │
└─────────────────────────────────────────────────────────────────┘

Điều kiện edit:
- Trong vòng 7 ngày kể từ khi tạo
- Vendor chưa reply
- Tối đa 2 lần edit

[Bước 1] Buyer vào "Đánh giá của tôi"
         │
         ▼
[Bước 2] Click "Chỉnh sửa" trên review
         │
         ▼
[Bước 3] Kiểm tra điều kiện:
         │
         ├── Quá 7 ngày ──► "Không thể chỉnh sửa sau 7 ngày"
         ├── Đã có reply ──► "Không thể chỉnh sửa khi shop đã phản hồi"
         ├── Đã edit 2 lần ──► "Đã hết lượt chỉnh sửa"
         │
         ▼
[Bước 4] Hiển thị form edit
         │
         ▼
[Bước 5] Buyer sửa và submit
         │
         ▼
[Bước 6] Update review:
         - Lưu version cũ vào history
         - Cập nhật nội dung mới
         - edit_count += 1
         - updated_at = NOW()
         │
         ▼
[Bước 7] Hiển thị badge "Đã chỉnh sửa" trên review
```

### 5.2 Buyer xóa review

```
┌─────────────────────────────────────────────────────────────────┐
│                  FLOW XÓA REVIEW                                │
└─────────────────────────────────────────────────────────────────┘

Điều kiện xóa:
- Trong vòng 24 giờ kể từ khi tạo
- Vendor chưa reply

[Bước 1] Buyer click "Xóa" trên review
         │
         ▼
[Bước 2] Kiểm tra điều kiện
         │
         ├── Quá 24h ──► "Không thể xóa sau 24 giờ"
         ├── Đã có reply ──► "Không thể xóa khi shop đã phản hồi"
         │
         ▼
[Bước 3] Confirm: "Bạn có chắc muốn xóa đánh giá này?"
         │
         ▼
[Bước 4] Soft delete review:
         - is_deleted = true
         - deleted_at = NOW()
         │
         ▼
[Bước 5] Recalculate ratings
         │
         ▼
[Bước 6] Hiển thị "Đã xóa đánh giá"
```

---

## 6. Report Review (Báo cáo vi phạm)

### Flow report

```
┌─────────────────────────────────────────────────────────────────┐
│                  FLOW REPORT REVIEW                             │
└─────────────────────────────────────────────────────────────────┘

[Bước 1] User click "Báo cáo" trên review
         │
         ▼
[Bước 2] Hiển thị form report:
         
         Lý do báo cáo:
         ○ Spam/Quảng cáo
         ○ Ngôn ngữ thô tục
         ○ Thông tin sai sự thật
         ○ Review fake (chưa mua hàng)
         ○ Khác: [___________]
         │
         ▼
[Bước 3] Submit report
         │
         ▼
[Bước 4] Tạo Report record
         │
         ▼
[Bước 5] Nếu review nhận >= 3 reports:
         └── Auto hide và notify admin
         │
         ▼
[Bước 6] Admin review và quyết định:
         - Giữ nguyên review
         - Ẩn review vĩnh viễn
         - Cảnh cáo/ban user tạo review
```

---

## 7. Admin quản lý Reviews

### 7.1 Dashboard Reviews

```
┌─────────────────────────────────────────────────────────────────┐
│              ADMIN REVIEWS DASHBOARD                            │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  THỐNG KÊ REVIEWS                                              │
├─────────────────────────────────────────────────────────────────┤
│  Hôm nay: 156 reviews | TB: ⭐ 4.6                              │
│  Cần xử lý: 5 reports | Hidden: 2                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Filter: [Reported ▼] [1 sao ▼] [Hôm nay ▼]                   │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │ Review #123 | user123 | Shop: ABC | ⭐ (1 sao)            │ │
│  │ "Lừa đảo, hàng die hết..."                                │ │
│  │ Reports: 2 | Created: 2h ago                              │ │
│  │                         [Xem chi tiết] [Hide] [Delete]    │ │
│  └───────────────────────────────────────────────────────────┘ │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 7.2 Admin actions

```
Admin có thể:
1. Hide review - Ẩn khỏi public, giữ trong DB
2. Delete review - Xóa vĩnh viễn
3. Edit review - Sửa nội dung vi phạm
4. Verify review - Đánh dấu đã kiểm tra
5. Ban reviewer - Khóa quyền review của user

Mỗi action đều được log với lý do
```

---

## 8. Anti-Gaming Measures

### 8.1 Chống fake review

```
┌─────────────────────────────────────────────────────────────────┐
│              CHỐNG FAKE REVIEW                                  │
└─────────────────────────────────────────────────────────────────┘

1. Verified Purchase:
   - Chỉ cho review khi có order thực
   - Hiển thị badge "Đã mua hàng"

2. Rate Limiting:
   - Max 10 reviews/ngày/user
   - Min 1 phút giữa các review

3. Duplicate Detection:
   - Check nội dung tương tự
   - Flag nếu copy-paste

4. Behavior Analysis:
   - Detect pattern: cùng IP, timing
   - Flag batch reviews cho 1 shop

5. Review Karma:
   - Users có nhiều "Hữu ích" được tin tưởng hơn
   - Users bị report nhiều bị giảm weight

6. New Account Restriction:
   - Account < 7 ngày không thể review
   - Hoặc cần verify phone
```

---

## Database Schema

### Bảng reviews

```sql
CREATE TABLE reviews (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    
    order_id BIGINT NOT NULL,
    product_id BIGINT NOT NULL,
    shop_id BIGINT NOT NULL,
    buyer_id BIGINT NOT NULL,
    
    rating TINYINT NOT NULL,              -- 1-5
    comment TEXT,
    tags JSON,                            -- ["tag1", "tag2"]
    
    is_anonymous BOOLEAN DEFAULT FALSE,
    
    -- Vendor reply
    vendor_reply TEXT,
    vendor_replied_at TIMESTAMP NULL,
    
    -- Edit tracking
    edit_count TINYINT DEFAULT 0,
    is_edited BOOLEAN DEFAULT FALSE,
    
    -- Moderation
    is_hidden BOOLEAN DEFAULT FALSE,
    hidden_reason TEXT,
    hidden_by BIGINT NULL,
    
    is_deleted BOOLEAN DEFAULT FALSE,
    deleted_at TIMESTAMP NULL,
    
    -- Stats
    helpful_count INT DEFAULT 0,
    report_count INT DEFAULT 0,
    
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    FOREIGN KEY (order_id) REFERENCES orders(id),
    FOREIGN KEY (product_id) REFERENCES products(id),
    FOREIGN KEY (shop_id) REFERENCES shops(id),
    FOREIGN KEY (buyer_id) REFERENCES users(id),
    
    UNIQUE KEY unique_order_review (order_id),
    INDEX idx_product (product_id, is_hidden, is_deleted),
    INDEX idx_shop (shop_id, is_hidden, is_deleted),
    INDEX idx_rating (rating)
);
```

### Bảng review_images

```sql
CREATE TABLE review_images (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    review_id BIGINT NOT NULL,
    image_path VARCHAR(255) NOT NULL,
    sort_order TINYINT DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (review_id) REFERENCES reviews(id) ON DELETE CASCADE
);
```

### Bảng review_helpful

```sql
CREATE TABLE review_helpful (
    review_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    PRIMARY KEY (review_id, user_id),
    FOREIGN KEY (review_id) REFERENCES reviews(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id)
);
```

### Bảng review_reports

```sql
CREATE TABLE review_reports (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    review_id BIGINT NOT NULL,
    reporter_id BIGINT NOT NULL,
    reason ENUM('spam', 'offensive', 'false_info', 'fake', 'other'),
    description TEXT,
    status ENUM('pending', 'reviewed', 'actioned', 'dismissed'),
    reviewed_by BIGINT NULL,
    reviewed_at TIMESTAMP NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (review_id) REFERENCES reviews(id),
    FOREIGN KEY (reporter_id) REFERENCES users(id),
    FOREIGN KEY (reviewed_by) REFERENCES users(id)
);
```
