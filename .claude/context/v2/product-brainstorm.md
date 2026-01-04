# Product Module - Brainstorm & Design

## Overview
Quản lý sản phẩm và tồn kho cho P2PMMO V2 - Cho phép vendors tạo, quản lý sản phẩm và upload inventory (dữ liệu bán hàng như tài khoản, key, license...).

## V2 Analysis (Based on Full Flows Doc)

### What Worked in V2 Design
- ✅ Module-based architecture rõ ràng (domain, dto, handler, service, repository)
- ✅ Auto-delivery ngay sau khi thanh toán
- ✅ Duplicate check 4 levels (trong product, shop, platform sold, platform unsold)
- ✅ Bulk inventory upload (paste text hoặc upload file)
- ✅ Pre-order support với auto-fulfill khi restock
- ✅ Real-time stock tracking
- ✅ Encrypted content cho product items

### V2 Key Features Required
1. **Product Management**: Create, Update, Hide, Delete products
2. **Inventory Upload**: Bulk upload với duplicate check
3. **Stock Management**: Real-time tracking, restock, view sold/unsold
4. **Pre-order**: Auto-fulfill khi restock
5. **Search & Filter**: Public search, shop products, category filters

---

## Flows & Features

### Flow 1: CREATE PRODUCT
**Description:** Vendor tạo sản phẩm mới với thông tin cơ bản

**Workflow:**
```
┌─────────────────────────────────────────────────────────────────┐
│                  VENDOR - CREATE PRODUCT                        │
└─────────────────────────────────────────────────────────────────┘

  VENDOR VIEW:
    ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
    │  CLICK   │───▶│  FILL    │───▶│ UPLOAD   │───▶│ PRODUCT  │
    │ "CREATE │    │  FORM    │    │  IMAGE   │    │ CREATED  │
    │ PRODUCT"│    │          │    │(optional)│    │(draft)   │
    └──────────┘    └────┬─────┘    └────┬─────┘    └────┬─────┘
                         │                │                │
                         ▼                ▼                ▼
                  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
                  │ - Name      │  │ Process    │  │ Redirect to │
                  │ - Category  │  │ Resize     │  │ inventory   │
                  │ - Desc      │  │ Optimize   │  │ upload      │
                  │ - Price     │  │ Save path  │  │             │
                  │ - Min/Max   │  │            │  │             │
                  │ - Settings  │  │            │  │             │
                  └─────────────┘  └─────────────┘  └─────────────┘

  ADMIN VIEW:
    (Not involved)

  BUYER VIEW:
    (Cannot see draft products)
```

**By Actor:**
- **Vendor:**
  - Fill form: name, category, description, image (optional)
  - Set price, original_price, min_purchase, max_purchase
  - Configure: allow_preorder, allow_resell, hide_stock, require_2fa
  - Upload & process image
  - Product created with status: draft, stock: 0

- **Admin:** Not involved

- **Buyer:** Cannot see draft products

**Permissions:**
- `[PRODUCT:CREATE]` - Create new product (vendor only)

**Validation:**
```rust
// Required fields
name: 5-200 chars
category: must exist
description: 50-5000 chars
price: > 0
min_purchase: >= 1, <= max_purchase
max_purchase: 0 = unlimited

// Optional
original_price: >= price (if provided)
image: max 5MB, jpg/png

// Settings
allow_preorder: bool (default false)
allow_resell: bool (default false) - V1 only, V2 removes
hide_stock: bool (default false)
require_2fa: bool (default false)
```

**Related Files:**
- domain.rs: Product, Category structs
- dto.rs: CreateProductRequest, ProductResponse
- handler.rs: create_product_handler
- service.rs: validate_product_data, create_product, process_image
- repository.rs: insert_product, find_category_by_id

---

### Flow 2: UPLOAD INVENTORY
**Description:** Upload bulk items vào product với duplicate check 4 levels

**Workflow:**
```
┌─────────────────────────────────────────────────────────────────┐
│                  VENDOR - UPLOAD INVENTORY                      │
└─────────────────────────────────────────────────────────────────┘

  VENDOR VIEW:
    ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
    │  GO TO   │───▶│  SELECT  │───▶│  PASTE   │───▶│  PARSE   │
    │INVENTORY │    │  METHOD  │    │  / UPLOAD │    │  CONTENT │
    │  UPLOAD  │    │          │    │           │    │          │
    └──────────┘    └────┬─────┘    └────┬─────┘    └────┬─────┘
                         │                │                │
                         ▼                ▼                ▼
                  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
                  │ Paste text  │  │ Upload .txt │  │ Split by    │
                  │ OR          │  │ max 10MB    │  │ newline     │
                  │ Upload file │  │            │  │ Trim        │
                  └─────────────┘  └─────────────┘  │ Remove empty│
                                                        └─────┬──────┘
                                                              │
                                                              ▼
                                                    ┌───────────────────┐
                                                    │ DUPLICATE CHECK    │
                                                    │ (4 Levels)         │
                                                    └─────┬─────────────┘
                                                          │
                    ┌─────────────────┼─────────────────┼─────────────────┐
                    │                 │                 │                 │
                    ▼                 ▼                 ▼                 ▼
            ┌───────────┐     ┌───────────┐     ┌───────────┐     ┌───────────┐
            │ Level 1:  │     │ Level 2:  │     │ Level 3:  │     │ Level 4:  │
            │ Within    │     │ Within    │     │ Platform  │     │ Platform  │
            │ product   │     │ shop      │     │ SOLD      │     │ UNSOLD    │
            │ (WARNING) │     │ (WARNING) │     │ (BLOCK)   │     │ (BLOCK)   │
            └─────┬─────┘     └─────┬─────┘     └─────┬─────┘     └─────┬─────┘
                  │                 │                 │                 │
                  └─────────────────┼─────────────────┘                 │
                                    │                                  │
                                    ▼                                  ▼
                          ┌───────────────────┐              ┌───────────────────┐
                          │ DISPLAY RESULTS   │              │ CANNOT UPLOAD     │
                          │ ───────────────── │              │ ───────────────── │
                          │ Total: 100        │              │ Show blocked      │
                          │ Valid: 85         │              │ items             │
                          │ Duplicates:       │              │                   │
                          │  - In product: 10 │              │                   │
                          │  - In shop: 5     │              │                   │
                          │ ───────────────── │              │                   │
                          │ [Confirm Upload]  │              │                   │
                          └─────────┬─────────┘              └───────────────────┘
                                    │
                                    ▼
                          ┌───────────────────┐
                          │ ENCRYPT & INSERT  │
                          │ ───────────────── │
                          │ 1. Hash content   │
                          │ 2. Encrypt        │
                          │ 3. Insert items   │
                          │ 4. Update stock   │
                          │ 5. Check pre-     │
                          │    orders         │
                          └─────────┬─────────┘
                                    │
                                    ▼
                          ┌───────────────────┐
                          │ SUCCESS           │
                          │ ───────────────── │
                          │ ✓ Added: 85 items │
                          │ Stock: 0 → 85     │
                          │ Status: draft→    │
                          │   active          │
                          │ ✓ Auto-fulfilled  │
                          │   3 pre-orders    │
                          └───────────────────┘

  AUTO-FULFILL PRE-ORDERS:
    ┌──────────┐    ┌──────────┐    ┌──────────┐
    │ NEW STOCK│───▶│ FIND PENDING│─▶│ FULFILL  │
    │ ADDED    │    │ PRE-ORDERS │  │ FIFO     │
    └──────────┘    └──────┬─────┘    └────┬─────┘
                           │                │
                           ▼                ▼
                    ┌─────────────┐  ┌─────────────┐
                    │ Order by    │  │ Lock stock  │
                    │ created_at  │  │ Create order│
                    │ (FIFO)      │  │ Mark sold   │
                    └─────────────┘  │ Notify buyer│
                                     └─────────────┘
```

**By Actor:**
- **Vendor:**
  - Select upload method: Paste text OR Upload file
  - Paste format: email|password|2fa_secret|email_backup (1 line = 1 item)
  - Or upload .txt file (max 10MB)
  - View duplicate check results (4 levels)
  - Confirm upload
  - See items encrypted & inserted
  - Stock updated
  - Pre-orders auto-fulfilled

- **Admin:** Not involved

- **Buyer:** Not involved

**Permissions:**
- `[PRODUCT:UPLOAD]` - Upload inventory (vendor only)
- `[PRODUCT:VIEW_DUPLICATES]` - View duplicate check results

**Duplicate Check Levels:**
```rust
Level 1: Within current product (WARNING)
  → Check: content_hash exists in product_id?
  → Action: Warn, allow upload

Level 2: Within current shop (WARNING)
  → Check: content_hash exists in shop_id?
  → Action: Warn, allow upload

Level 3: Platform SOLD items (BLOCK)
  → Check: content_hash exists where is_sold = true?
  → Action: BLOCK, show which product sold it

Level 4: Platform UNSOLD items (BLOCK)
  → Check: content_hash exists where is_sold = false?
  → Action: BLOCK, show which product has it
```

**Data Format:**
```text
// Expected format (1 line = 1 item)
email|password|2fa_secret|email_backup
OR
account|password|backup_code

// Parse rules:
- Split by newline
- Trim whitespace
- Remove empty lines
- Encrypt each item
- Hash for duplicate check
```

**Encryption:**
```rust
// For each item:
1. content = user_input
2. content_hash = SHA256(content)
3. encrypted_content = encrypt(content, encryption_key)
4. Store: product_id, encrypted_content, content_hash, is_sold: false
```

**Related Files:**
- domain.rs: ProductItem, Product
- dto.rs: UploadInventoryRequest, DuplicateCheckResponse
- handler.rs: upload_inventory_handler, check_duplicates_handler
- service.rs: parse_inventory, check_duplicates_4_levels, encrypt_items, insert_items, auto_fulfill_preorders
- repository.rs: find_duplicate_items, insert_product_items, update_product_stock, find_pending_preorders

---

### Flow 3: VIEW PRODUCTS (PUBLIC)
**Description:** Buyer/viewer browse & search products

**Workflow:**
```
┌─────────────────────────────────────────────────────────────────┐
│                  BUYER/GUEST - VIEW PRODUCTS                    │
└─────────────────────────────────────────────────────────────────┘

  GUEST/BUYER VIEW:
    ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
    │  BROWSE  │───▶│  FILTER  │───▶│  SORT    │───▶│  VIEW    │
    │ /products│    │  & SEARCH│    │          │    │  LIST    │
    └──────────┘    └────┬─────┘    └────┬─────┘    └────┬─────┘
                         │                │                │
                         ▼                ▼                ▼
                  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
                  │ Search by   │  │ Price: Low  │  │ Product     │
                  │ keyword     │  │ to High     │  │ cards with: │
                  │ Category    │  │ High to Low │  │ - Image     │
                  │ Price range │  │ Newest      │  │ - Name      │
                  │ Stock       │  │ Best Selling │  │ - Price     │
                  │ Rating      │  │ Top Rated   │  │ - Stock     │
                  └─────────────┘  └─────────────┘  │ - Rating    │
                                                      │ - Shop name │
                                                      └─────┬───────┘
                                                            │
                                                            ▼
                                                  ┌───────────────────┐
                                                  │ CLICK PRODUCT      │
                                                  └─────┬─────────────┘
                                                        │
                                                        ▼
                                                  ┌───────────────────┐
                                                  │ PRODUCT DETAIL     │
                                                  │ ─────────────────  │
                                                  │ • Full description │
                                                  │ • All images       │
                                                  │ • Price            │
                                                  │ • Stock count      │
                                                  │ • (if hide_stock:  │
                                                  │    "Còn hàng")     │
                                                  │ • Min/max purchase │
                                                  │ • Reviews          │
                                                  │ • Shop info        │
                                                  │ • [Buy Now]        │
                                                  │ • [Pre-order]      │
                                                  │   (if stock=0 &    │
                                                  │    allow_preorder) │
                                                  └───────────────────┘

  VENDOR VIEW:
    (Cannot see own products in public browse)
    (Uses /vendor/products instead)

  ADMIN VIEW:
    ┌──────────┐    ┌──────────┐    ┌──────────┐
    │  VIEW    │───▶│  FILTER  │───▶│  ACTIONS │
    │ /admin/  │    │  BY      │    │          │
    │ products │    │ SHOP/    │    │ [Delete] │
    │          │    │ CATEGORY │    │ [Hide]   │
    └──────────┘    └──────────┘    │ [Edit]   │
                                   └──────────┘
```

**By Actor:**
- **Buyer/Guest:**
  - Browse all products (status: active)
  - Search by keyword
  - Filter by category, price, stock, rating
  - Sort by price, newest, best selling, top rated
  - View product detail page
  - See shop info, reviews
  - Click "Buy Now" or "Pre-order"

- **Vendor:**
  - Not used (uses /vendor/products)

- **Admin:**
  - View all products across all shops
  - Filter by shop, category, status
  - Actions: Delete, Hide, Edit

**Permissions:**
- `[PRODUCT:VIEW]` - View products (public)
- `[PRODUCT:VIEW_DETAIL]` - View product detail (public)
- `[PRODUCT:VIEW_ALL]` - View all products including inactive (admin)
- `[PRODUCT:DELETE]` - Delete product (admin)
- `[PRODUCT:HIDE]` - Hide product (admin)

**Filters:**
```rust
// Query parameters
category: Option<ObjectId>
shop_id: Option<ObjectId>
keyword: Option<String>  // search in name, description
price_min: Option<Decimal>
price_max: Option<Decimal>
in_stock: bool  // only show products with stock > 0
rating_min: Option<u8>  // 1-5 stars
sort_by: "price_asc" | "price_desc" | "newest" | "best_selling" | "top_rated"
page: usize (default 1)
per_page: usize (default 20, max 100)

// Product status
- Only show status: active (for public)
- Admin can see all statuses
```

**Hide Stock Behavior:**
```rust
if product.hide_stock == true {
  display: "Còn hàng"  // Don't show exact number
} else {
  display: "123 còn lại"  // Show exact count
}
```

**Related Files:**
- domain.rs: Product, Shop, Category
- dto.rs: ProductListRequest, ProductDetailResponse, ProductCard
- handler.rs: list_products_handler, view_product_handler
- service.rs: filter_products, sort_products, paginate
- repository.rs: find_products, find_product_by_slug, aggregate_stats

---

### Flow 4: UPDATE PRODUCT
**Description:** Vendor chỉnh sửa thông tin sản phẩm

**Workflow:**
```
┌─────────────────────────────────────────────────────────────────┐
│                  VENDOR - UPDATE PRODUCT                        │
└─────────────────────────────────────────────────────────────────┘

  VENDOR VIEW:
    ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
    │  GO TO   │───▶│  CLICK   │───▶│  EDIT    │───▶│  SAVE    │
    │/vendor/  │    │ "Edit"   │    │  FIELDS  │    │ CHANGES  │
    │products  │    │          │    │          │    │          │
    └──────────┘    └────┬─────┘    └────┬─────┘    └────┬─────┘
                         │                │                │
                         ▼                ▼                ▼
                  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
                  │ Cannot edit │  │ Editable:   │  │ Validate    │
                  │ - slug      │  │ - name      │  │ - Same rules │
                  │ - shop_id   │  │ - category  │  │   as create │
                  │ - created_at│  │ - desc      │  │ - Update    │
                  │             │  │ - price     │  │   updated_at│
                  │             │  │ - settings  │  │ - Version   │
                  │             │  │ - image     │  │   check     │
                  └─────────────┘  └─────────────┘  └─────┬──────┘
                                                         │
                                                         ▼
                                              ┌───────────────────┐
                                              │ RESTRICTIONS      │
                                              │ ───────────────── │
                                              │ If product has:   │
                                              │ • Sold items → Can │
                                              │   still edit info │
                                              │ • Active orders → │
                                              │   Cannot change   │
                                              │   price (warn)    │
                                              └───────────────────┘
```

**By Actor:**
- **Vendor:**
  - Edit product info (name, category, description, image)
  - Change price (with warning if active orders)
  - Update settings (min/max, pre-order, hide_stock, 2fa)
  - Cannot edit: slug, shop_id, created_at
  - Changes logged in version history

- **Admin:** Can edit any product

- **Buyer:** Not involved

**Permissions:**
- `[PRODUCT:UPDATE]` - Update product (vendor/admin)
- `[PRODUCT:UPDATE_PRICE]` - Update product price (vendor/admin)

**Restrictions:**
```rust
// Version check (optimistic locking)
if product.version != request.version {
  return Err("Product was modified by another session");
}

// Price change warning
if has_active_orders(product_id) && price_changed {
  show_warning: "This product has active orders. Changing price may affect disputes.";
}
```

**Editable Fields:**
```rust
// CAN edit:
- name
- category_id
- description
- image
- price
- original_price
- min_purchase
- max_purchase
- allow_preorder
- hide_stock
- require_2fa

// CANNOT edit:
- slug (immutable after creation)
- shop_id (immutable)
- created_at (immutable)
```

**Related Files:**
- domain.rs: Product, ProductVersion
- dto.rs: UpdateProductRequest
- handler.rs: update_product_handler
- service.rs: validate_update, check_active_orders, update_product
- repository.rs: update_product, create_version_history

---

### Flow 5: DELETE PRODUCT (SOFT DELETE)
**Description:** Vendor hoặc admin xóa sản phẩm (soft delete)

**Workflow:**
```
┌─────────────────────────────────────────────────────────────────┐
│                  VENDOR/ADMIN - DELETE PRODUCT                   │
└─────────────────────────────────────────────────────────────────┘

  VENDOR VIEW:
    ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
    │  CLICK   │───▶│  CONFIRM │───▶│  CHECK   │───▶│  SOFT    │
    │ "Delete" │    │  ACTION  │    │  ORDERS  │    │  DELETE  │
    └──────────┘    └────┬─────┘    └────┬─────┘    └────┬─────┘
                         │                │                │
                         ▼                ▼                ▼
                  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
                  │ Show        │  │ If has      │  │ status =    │
                  │ warning     │  │ active      │  │ deleted     │
                  │ with        │  │ orders →    │  │ deleted_at  │
                  │ product     │  │ BLOCK with  │  │ = NOW()     │
                  │ info        │  │ reason      │  │ Keep data   │
                  └─────────────┘  └─────────────┘  │ 30 days     │
                                                        └─────┬──────┘
                                                              │
                                                              ▼
                                                  ┌───────────────────┐
                                                  │ CLEANUP (CRON)    │
                                                  │ ───────────────── │
                                                  │ After 30 days:    │
                                                  │ • Hard delete     │
                                                  │ • Delete items    │
                                                  │ • Delete from     │
                                                  │   search index    │
                                                  └───────────────────┘

  ADMIN VIEW:
    ┌──────────┐    ┌──────────┐    ┌──────────┐
    │  CAN     │───▶│  FORCE   │───▶│  DELETE │
    │  DELETE  │    │  DELETE  │    │  (ignore │
    │  ANY     │    │  (skip   │    │  orders) │
    │  PRODUCT │    │  order   │    │          │
    └──────────┘    │  check)  │    └──────────┘
                   └──────────┘
```

**By Actor:**
- **Vendor:**
  - Request delete (with confirmation)
  - Blocked if has active orders
  - Soft delete: status = deleted
  - Data kept 30 days before hard delete

- **Admin:**
  - Can delete any product
  - Option to force delete (skip order check)
  - Same soft delete process

- **Buyer:** Not involved

**Permissions:**
- `[PRODUCT:DELETE]` - Delete own product (vendor)
- `[PRODUCT:DELETE_ANY]` - Delete any product (admin)
- `[PRODUCT:FORCE_DELETE]` - Force delete ignoring active orders (admin)

**Soft Delete Behavior:**
```rust
// Soft delete
product.status = "deleted"
product.deleted_at = Some(Utc::now())
product.deleted_by = Some(user_id)

// Keep data for 30 days
// After 30 days → Hard delete:
// - Delete product_items
// - Delete product
// - Remove from search index
// - Keep in order_items (for history)

// Active order check
if has_active_orders(product_id) {
  return Err("Cannot delete: Product has active orders");
}
```

**Admin Force Delete:**
```rust
if admin && force_delete {
  // Override order check
  // Soft delete anyway
  // Log admin action
}
```

**Related Files:**
- domain.rs: Product
- dto.rs: DeleteProductRequest
- handler.rs: delete_product_handler
- service.rs: check_active_orders, soft_delete_product
- repository.rs: update_product_status, find_active_orders

---

### Flow 6: VIEW INVENTORY DETAIL
**Description:** Vendor xem chi tiết inventory (unsold, sold, on hold)

**Workflow:**
```
┌─────────────────────────────────────────────────────────────────┐
│                  VENDOR - VIEW INVENTORY                        │
└─────────────────────────────────────────────────────────────────┘

  VENDOR VIEW:
    ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
    │  GO TO   │───▶│  VIEW    │───▶│  SELECT  │───▶│  VIEW    │
    │ PRODUCT  │    │  STATS   │    │  TAB     │    │  ITEMS   │
    │ DETAIL   │    │          │    │          │    │          │
    └──────────┘    └────┬─────┘    └────┬─────┘    └────┬─────┘
                         │                │                │
                         ▼                ▼                ▼
                  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
                  │ Total: 500  │  │ TABS:       │  │ LIST ITEMS  │
                  │ Sold: 350   │  │ • Unsold    │  │ ─────────── │
                  │ Remaining:  │  │ • Sold      │  │ • Content   │
                  │   150       │  │ • On Hold   │  │ • Status    │
                  │ On Hold: 0  │  │ (pre-orders)│  │ • Order #   │
                  └─────────────┘  └─────────────┘  │ • Sold date │
                                                      │ • Actions   │
                                                      └─────┬──────┘
                                                            │
                    ┌───────────────────────┼───────────────┘
                    │                       │
                    ▼                       ▼
            ┌───────────────┐       ┌───────────────┐
            │ UNSOLD TAB    │       │ SOLD TAB      │
            │ ───────────── │       │ ───────────── │
            │ Show FULL     │       │ Show MASKED   │
            │ content       │       │ email***|***  │
            │ Can delete    │       │ Show order #  │
            │ Can export    │       │ Show buyer    │
            └───────────────┘       └───────────────┘
```

**By Actor:**
- **Vendor:**
  - View inventory stats
  - Switch tabs: Unsold, Sold, On Hold
  - **Unsold tab:** See full content, can delete items, can export
  - **Sold tab:** See masked content, order number, buyer info
  - **On Hold tab:** See pre-order items, hold until date

- **Admin:** Not involved

- **Buyer:** Not involved

**Permissions:**
- `[PRODUCT:VIEW_INVENTORY]` - View inventory (vendor only)
- `[PRODUCT:DELETE_ITEMS]` - Delete unsold items (vendor only)
- `[PRODUCT:EXPORT_INVENTORY]` - Export inventory (vendor only)

**Masking for Sold Items:**
```rust
// Sold items - Mask content
fn mask_content(content: &str) -> String {
  let parts: Vec<&str> = content.split('|').collect();
  parts.iter()
    .enumerate()
    .map(|(i, part)| {
      if i == 0 {
        // First part (email): show first 2 chars, mask rest
        format!("{}***", &part[..2.min(part.len())])
      } else {
        // Other parts: show first 3 chars, mask rest
        format!("{}***", &part[..3.min(part.len())])
      }
    })
    .collect::<Vec<_>>()
    .join("|")
}

// Example:
// Original: email@gmail.com|password123|2facode|backup
// Masked:   em***@gm***.com|pas***|2fa***|bac***
```

**Tabs:**
```rust
Tab 1: Unsold (is_sold = false, hold_until = None)
  → Show full content
  → Can delete items
  → Can export to .txt

Tab 2: Sold (is_sold = true)
  → Show masked content
  → Show order_number
  → Show buyer username
  → Show sold_at date

Tab 3: On Hold (hold_until IS NOT NULL AND is_sold = false)
  → Show full content (vendor owns it)
  → Show hold_until date
  → Show pre_order_number
```

**Related Files:**
- domain.rs: ProductItem, Order
- dto.rs: InventoryStatsResponse, InventoryItemResponse
- handler.rs: view_inventory_handler, delete_items_handler, export_inventory_handler
- service.rs: get_inventory_stats, mask_content, delete_items
- repository.rs: find_items_by_product, aggregate_inventory_stats

---

### Flow 7: RESTOCK
**Description:** Vendor thêm inventory khi stock thấp

**Workflow:**
```
┌─────────────────────────────────────────────────────────────────┐
│                  VENDOR - RESTOCK                                │
└─────────────────────────────────────────────────────────────────┘

  VENDOR VIEW:
    ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
    │  VIEW    │───▶│  CLICK   │───▶│  UPLOAD  │───▶│  STOCK   │
    │  PRODUCT │    │ "Restock"│    │  ITEMS   │    │ UPDATED  │
    │  DETAIL  │    │          │    │          │    │          │
    └──────────┘    └────┬─────┘    └────┬─────┘    └────┬─────┘
                         │                │                │
                         ▼                ▼                ▼
                  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
                  │ Show current│  │ Same as     │  │ Duplicate   │
                  │ stock: 15   │  │ upload      │  │ check       │
                  │ Low stock   │  │ flow        │  │ Insert      │
                  │ warning     │  │             │  │ Update stock│
                  └─────────────┘  └─────────────┘  └─────┬──────┘
                                                         │
                                                         ▼
                                              ┌───────────────────┐
                                              │ AUTO-FULFILL      │
                                              │ PRE-ORDERS        │
                                              │ ───────────────── │
                                              │ Find pending      │
                                              │ pre-orders for    │
                                              │ this product      │
                                              │ (FIFO by          │
                                              │  created_at)      │
                                              │ ───────────────── │
                                              │ For each:         │
                                              │ 1. Lock stock     │
                                              │ 2. Create order   │
                                              │ 3. Mark sold      │
                                              │ 4. Update pre-    │
                                              │    order status   │
                                              │ 5. Notify buyer   │
                                              │ 6. Continue until │
                                              │    stock runs out │
                                              └───────────────────┘
```

**By Actor:**
- **Vendor:**
  - View product detail (see low stock warning)
  - Click "Restock"
  - Upload more items (same flow as initial upload)
  - Duplicate check runs
  - Items added to inventory
  - Stock updated
  - **Pre-orders auto-fulfilled** (FIFO)

- **Admin:** Not involved

- **Buyer:** Not involved (but notified if pre-order fulfilled)

**Permissions:**
- `[PRODUCT:RESTOCK]` - Restock product (vendor only)
- `[PRODUCT:AUTO_FULFILL]` - Auto-fulfill pre-orders (system)

**Auto-fulfill Logic:**
```rust
// When vendor restocks
async fn on_restock(product_id: ObjectId, new_stock_count: i32) {
  // 1. Get pending pre-orders for this product
  let pre_orders = find_pending_pre_orders(product_id)
    .order_by("created_at")
    .await;

  let mut remaining_stock = new_stock_count;

  for pre_order in pre_orders {
    if remaining_stock <= 0 {
      break; // Out of stock, stop fulfilling
    }

    // 2. Check if we can fulfill this pre-order
    if pre_order.quantity <= remaining_stock {
      // 3. Lock stock items
      let items = lock_unsold_items(product_id, pre_order.quantity).await?;

      // 4. Create order
      let order = create_order_from_preorder(&pre_order, &items).await?;

      // 5. Mark items sold
      mark_items_sold(&items, order.id).await?;

      // 6. Update pre-order status
      update_preorder_status(pre_order.id, "fulfilled").await?;

      // 7. Notify buyer
      send_notification(
        pre_order.buyer_id,
        "Your pre-order is ready!",
        format!("Order #{} has been created. View your items now!", order.order_number)
      ).await;

      remaining_stock -= pre_order.quantity;
    }
  }
}
```

**Low Stock Warning:**
```rust
// Show warning when stock is low
if product.stock < 10 {
  show_warning: "Low stock! Consider restocking soon.";
}

if product.stock == 0 {
  if product.allow_preorder {
    show_button: "Pre-order" (instead of "Buy now")
  } else {
    show_message: "Out of stock"
  }
}
```

**Related Files:**
- domain.rs: Product, ProductItem, PreOrder, Order
- dto.rs: RestockRequest
- handler.rs: restock_handler
- service.rs: process_restock, auto_fulfill_preorders
- repository.rs: find_pending_preorders, lock_unsold_items

---

### Flow 8: SEARCH PRODUCTS (ADMIN)
**Description:** Admin tìm kiếm & quản lý tất cả sản phẩm

**Workflow:**
```
┌─────────────────────────────────────────────────────────────────┐
│                  ADMIN - SEARCH PRODUCTS                        │
└─────────────────────────────────────────────────────────────────┘

  ADMIN VIEW:
    ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
    │  GO TO   │───▶│  APPLY   │───▶│  VIEW    │───▶│  TAKE    │
    │/admin/   │    │  FILTERS │    │  RESULTS │    │  ACTION  │
    │products  │    │          │    │          │    │          │
    └──────────┘    └────┬─────┘    └────┬─────┘    └────┬─────┘
                         │                │                │
                         ▼                ▼                ▼
                  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
                  │ Filters:    │  │ Table with: │  │ Actions:    │
                  │ • Shop      │  │ - Product   │  │ • Edit      │
                  │ • Category  │  │ - Shop      │  │ • Hide      │
                  │ • Status    │  │ - Category  │  │ • Delete    │
                  │ • Price     │  │ - Price     │  │ • View items│
                  │ • Stock     │  │ - Stock     │  │ • Suspend  │
                  │ • Rating    │  │ - Sold      │  │ • View shop │
                  │ • Date      │  │ - Revenue   │  └─────────────┘
                  │ Search by   │  │ - Status    │
                  │ keyword/    │  │ - Actions   │
                  │ slug        │  └─────────────┘
                  └─────────────┘

  VENDOR VIEW:
    (Not involved)

  BUYER VIEW:
    (Not involved)
```

**By Actor:**
- **Admin:**
  - View all products (all statuses)
  - Filter by shop, category, status, price, stock, rating, date
  - Search by keyword or slug
  - View product details, shop info, revenue
  - Actions: Edit, Hide, Delete, Suspend shop

- **Vendor:** Not involved

- **Buyer:** Not involved

**Permissions:**
- `[PRODUCT:VIEW_ALL]` - View all products including inactive (admin)
- `[PRODUCT:EDIT_ANY]` - Edit any product (admin)
- `[PRODUCT:HIDE_ANY]` - Hide any product (admin)
- `[PRODUCT:DELETE_ANY]` - Delete any product (admin)
- `[PRODUCT:SUSPEND_SHOP]` - Suspend shop (removes all products) (admin)

**Admin Filters:**
```rust
// Extended filters for admin
shop_id: Option<ObjectId>
vendor_id: Option<ObjectId>
category_id: Option<ObjectId>
status: Option<String>  // draft, active, hidden, deleted, suspended
price_min: Option<Decimal>
price_max: Option<Decimal>
stock_min: Option<i32>
stock_max: Option<i32>
rating_min: Option<Decimal>
created_after: Option<DateTime>
created_before: Option<DateTime>
keyword: Option<String>
slug: Option<String>
has_violations: bool
sort_by: "created_at" | "stock" | "sold" | "revenue" | "dispute_rate"
page: usize
per_page: usize
```

**Suspend Shop:**
```rust
// When admin suspends a shop
async fn suspend_shop(shop_id: ObjectId, reason: &str) {
  // 1. Update shop status
  update_shop_status(shop_id, "suspended", reason).await;

  // 2. Hide all products
  update_all_products_status(shop_id, "hidden").await;

  // 3. Notify vendor
  send_notification(shop.vendor_id, "Shop suspended", reason).await;

  // 4. Log admin action
  log_admin_action("suspend_shop", shop_id, reason).await;
}
```

**Related Files:**
- domain.rs: Product, Shop
- dto.rs: AdminProductSearchRequest, AdminProductListResponse
- handler.rs: admin_search_products_handler, suspend_shop_handler
- service.rs: filter_products_admin, suspend_shop_products
- repository.rs: find_products_admin, aggregate_product_stats

---

## Permission Matrix

| Action | Permission Code | Buyer | Vendor | Admin |
|--------|----------------|-------|--------|-------|
| View products (public) | [PRODUCT:VIEW] | ✅ | ✅ | ✅ |
| View product detail | [PRODUCT:VIEW_DETAIL] | ✅ | ✅ | ✅ |
| View all products | [PRODUCT:VIEW_ALL] | ❌ | ❌ | ✅ |
| Create product | [PRODUCT:CREATE] | ❌ | ✅ | ❌ |
| Update own product | [PRODUCT:UPDATE] | ❌ | ✅ (own) | ✅ |
| Update product price | [PRODUCT:UPDATE_PRICE] | ❌ | ✅ (own) | ✅ |
| Upload inventory | [PRODUCT:UPLOAD] | ❌ | ✅ (own) | ❌ |
| View duplicates | [PRODUCT:VIEW_DUPLICATES] | ❌ | ✅ (own) | ❌ |
| View inventory | [PRODUCT:VIEW_INVENTORY] | ❌ | ✅ (own) | ❌ |
| Delete inventory items | [PRODUCT:DELETE_ITEMS] | ❌ | ✅ (own) | ❌ |
| Export inventory | [PRODUCT:EXPORT_INVENTORY] | ❌ | ✅ (own) | ❌ |
| Restock product | [PRODUCT:RESTOCK] | ❌ | ✅ (own) | ❌ |
| Hide own product | [PRODUCT:HIDE] | ❌ | ✅ (own) | ✅ |
| Delete own product | [PRODUCT:DELETE] | ❌ | ✅ (own) | ✅ |
| Edit any product | [PRODUCT:EDIT_ANY] | ❌ | ❌ | ✅ |
| Hide any product | [PRODUCT:HIDE_ANY] | ❌ | ❌ | ✅ |
| Delete any product | [PRODUCT:DELETE_ANY] | ❌ | ❌ | ✅ |
| Force delete | [PRODUCT:FORCE_DELETE] | ❌ | ❌ | ✅ |
| Suspend shop products | [PRODUCT:SUSPEND_SHOP] | ❌ | ❌ | ✅ |
| Auto-fulfill pre-orders | [PRODUCT:AUTO_FULFILL] | ❌ | ❌ | ✅ (system) |

---

## API Endpoints

| Method | Endpoint | Permission | Description |
|--------|----------|------------|-------------|
| **Public** | | | |
| GET | /api/products | [PRODUCT:VIEW] | List products (public browse) |
| GET | /api/products/{slug} | [PRODUCT:VIEW_DETAIL] | View product detail |
| **Vendor** | | | |
| POST | /api/vendor/products | [PRODUCT:CREATE] | Create new product |
| PUT | /api/vendor/products/{id} | [PRODUCT:UPDATE] | Update product |
| PATCH | /api/vendor/products/{id}/price | [PRODUCT:UPDATE_PRICE] | Update price only |
| POST | /api/vendor/products/{id}/inventory | [PRODUCT:UPLOAD] | Upload inventory |
| POST | /api/vendor/products/{id}/inventory/check | [PRODUCT:VIEW_DUPLICATES] | Check duplicates before upload |
| GET | /api/vendor/products/{id}/inventory | [PRODUCT:VIEW_INVENTORY] | View inventory detail |
| DELETE | /api/vendor/products/{id}/inventory | [PRODUCT:DELETE_ITEMS] | Delete inventory items |
| GET | /api/vendor/products/{id}/inventory/export | [PRODUCT:EXPORT_INVENTORY] | Export inventory to .txt |
| POST | /api/vendor/products/{id}/restock | [PRODUCT:RESTOCK] | Restock product |
| DELETE | /api/vendor/products/{id} | [PRODUCT:DELETE] | Delete own product |
| GET | /api/vendor/products | [PRODUCT:VIEW] | List own products |
| **Admin** | | | |
| GET | /api/admin/products | [PRODUCT:VIEW_ALL] | List all products with filters |
| GET | /api/admin/products/{id} | [PRODUCT:VIEW_ALL] | View any product detail |
| PUT | /api/admin/products/{id} | [PRODUCT:EDIT_ANY] | Edit any product |
| DELETE | /api/admin/products/{id} | [PRODUCT:DELETE_ANY] | Delete any product |
| POST | /api/admin/products/{id}/force-delete | [PRODUCT:FORCE_DELETE] | Force delete (ignore orders) |
| POST | /api/admin/shops/{id}/suspend | [PRODUCT:SUSPEND_SHOP] | Suspend shop (hide all products) |

---

## V2 Improvements

### Analysis of V2 Design vs Potential Enhancements

Based on the V2 Full Flows document, here are potential improvements to consider:

---

### 🔄 Pending Review

#### 1. **Real-time Stock Updates via WebSocket**
**V1:** Page refresh to see stock changes
**V2 Proposal:** WebSocket push when stock changes
**Benefit:** Better UX for popular products, prevent overselling
**Complexity:** Medium
```
📌 SUGGESTION: Real-time stock updates
   V1: Manual page refresh
   V2: WebSocket push stock updates to all viewers
   Benefit: Prevent overselling, better UX
   Complexity: Medium

   Add this feature? (yes/no/skip for now)
```

#### 2. **Bulk Edit Products**
**V1:** Edit products one by one
**V2 Proposal:** Select multiple products → Bulk update price, category, settings
**Benefit:** Save time for vendors with many products
**Complexity:** Low
```
📌 SUGGESTION: Bulk edit products
   V1: Edit one by one
   V2: Select multiple → Bulk update price/category/settings
   Benefit: Save time for vendors
   Complexity: Low

   Add this feature? (yes/no/skip for now)
```

#### 3. **Product Variants**
**V1:** Single variant per product
**V2 Proposal:** Support variants (e.g., "1 month", "3 months", "1 year")
**Benefit:** More flexibility for vendors
**Complexity:** High
```
📌 SUGGESTION: Product variants
   V1: One product = one price
   V2: Support variants (duration, tier, etc.)
   Benefit: More flexible pricing
   Complexity: High

   Add this feature? (yes/no/skip for now)
```

#### 4. **Schedule Product Publication**
**V1:** Product visible immediately after creation
**V2 Proposal:** Schedule auto-publish at specific time
**Benefit:** Prepare products in advance, auto-publish
**Complexity:** Low
```
📌 SUGGESTION: Scheduled publishing
   V1: Publish immediately
   V2: Set publish_date → Auto-publish at scheduled time
   Benefit: Prepare in advance
   Complexity: Low

   Add this feature? (yes/no/skip for now)
```

#### 5. **Advanced Duplicate Check**
**V1:** Exact match only (content_hash)
**V2 Proposal:** Fuzzy matching for similar items (e.g., same email, different password)
**Benefit:** Catch more duplicates, improve quality
**Complexity:** High
```
📌 SUGGESTION: Fuzzy duplicate detection
   V1: Exact match only
   V2: Fuzzy match (same email, different password = warn)
   Benefit: Better duplicate detection
   Complexity: High

   Add this feature? (yes/no/skip for now)
```

#### 6. **Inventory Import from URL**
**V1:** Paste or upload file
**V2 Proposal:** Import from URL (pastebin, direct URL)
**Benefit:** Convenient for remote sources
**Complexity:** Low
```
📌 SUGGESTION: Import from URL
   V1: Paste text or upload file
   V2: Provide URL → Fetch content
   Benefit: Import from remote sources
   Complexity: Low

   Add this feature? (yes/no/skip for now)
```

#### 7. **Product Analytics Dashboard**
**V1:** Basic stats only
**V2 Proposal:** Detailed analytics: views, conversion rate, traffic sources
**Benefit:** Help vendors optimize products
**Complexity:** Medium
```
📌 SUGGESTION: Product analytics
   V1: Basic sales count
   V2: Views, conversion rate, traffic sources, popular times
   Benefit: Data-driven decisions
   Complexity: Medium

   Add this feature? (yes/no/skip for now)
```

#### 8. **Auto-hide Low Quality Products**
**V1:** Manual review only
**V2 Proposal:** Auto-hide if high dispute rate or many complaints
**Benefit:** Protect buyers, improve platform quality
**Complexity:** Medium
```
📌 SUGGESTION: Auto-hide low quality
   V1: Manual review
   V2: Auto-hide if dispute_rate > 20% or complaints > 10
   Benefit: Protect buyers
   Complexity: Medium

   Add this feature? (yes/no/skip for now)
```

---

### ✅ V2 Base Features (Already in Design)

These are already planned in V2:
- ✅ Module-based architecture
- ✅ Encrypted product items
- ✅ 4-level duplicate check
- ✅ Bulk upload (paste/file)
- ✅ Auto-fulfill pre-orders
- ✅ Soft delete with 30-day retention
- ✅ Real-time stock tracking
- ✅ Search & filter products
- ✅ Inventory management (unsold/sold/on hold tabs)
- ✅ Masked content for sold items

---

## Dependencies

### Required By
- **Order Module**: Needs product data (price, stock, info) to create orders
- **Inventory Module**: Manages product items
- **Pre-order Module**: Checks product.allow_preorder
- **Review Module**: Links reviews to products
- **Search Module**: Indexes products for search

### Depends On
- **User Module**: User authentication (vendor/admin)
- **Shop Module**: Products belong to shops
- **Category Module**: Products have categories
- **Auth Module**: Permission checks
- **Storage Module**: Store product images

### External Services
- **Image Storage**: Store product images (local/S3)
- **Search Index**: Elasticsearch/Meilisearch for product search (optional)
- **WebSocket**: Real-time stock updates (if implemented)

---

## Open Questions

1. **Product Variants**: Do we need variants in V2, or keep it simple for MVP?
2. **Real-time Updates**: Is WebSocket necessary for stock updates, or is polling sufficient?
3. **Duplicate Detection**: Is fuzzy matching needed, or is exact match enough?
4. **Analytics**: Should we include detailed analytics in V2, or defer to V2.1?
5. **Search Engine**: Use MongoDB text search or integrate Elasticsearch/Meilisearch?
6. **Image Processing**: Resize/optimize images on upload? What sizes?
7. **Export Format**: Export inventory as .txt only, or also support .csv, .json?

---

## Implementation Phases

### Phase 1: Foundation (Domain Models + Repository)
**Priority:** HIGH - Must be done first

**Tasks:**
1. [ ] Create `domain.rs` with: Product, ProductItem, ProductVersion structs
2. [ ] Create `repository.rs` with: CRUD operations, find by filters
3. [ ] Write unit tests for repository
4. [ ] Test MongoDB queries

**Dependencies:** None (but Category module should exist first)

**Estimated Files:**
- `src/modules/product/domain.rs`
- `src/modules/product/repository.rs`

---

### Phase 2: Data Transfer Objects (DTOs)
**Priority:** HIGH

**Tasks:**
1. [ ] Create request DTOs: CreateProductRequest, UpdateProductRequest, UploadInventoryRequest
2. [ ] Create response DTOs: ProductResponse, ProductCard, InventoryStatsResponse
3. [ ] Add validation rules
4. [ ] Write tests for DTO validation

**Dependencies:** Phase 1

**Estimated Files:**
- `src/modules/product/dto.rs`

---

### Phase 3: Business Logic (Service Layer)
**Priority:** HIGH

**Tasks:**
1. [ ] Implement ProductService: create, update, delete, search
2. [ ] Implement InventoryService: upload, check duplicates, restock
3. [ ] Implement duplicate check (4 levels)
4. [ ] Implement encryption for product items
5. [ ] Implement auto-fulfill pre-orders
6. [ ] Write unit tests

**Dependencies:** Phase 1, Phase 2

**Estimated Files:**
- `src/modules/product/service.rs`

---

### Phase 4: HTTP Handlers
**Priority:** HIGH

**Tasks:**
1. [ ] Create public handlers: list_products, view_product
2. [ ] Create vendor handlers: create_product, update_product, upload_inventory, restock
3. [ ] Create admin handlers: search_products, suspend_shop
4. [ ] Add permission checks
5. [ ] Add input validation
6. [ ] Write integration tests

**Dependencies:** Phase 2, Phase 3

**Estimated Files:**
- `src/modules/product/handler.rs`

---

### Phase 5: Routes Configuration
**Priority:** MEDIUM

**Tasks:**
1. [ ] Configure public routes (/api/products)
2. [ ] Configure vendor routes (/api/vendor/products)
3. [ ] Configure admin routes (/api/admin/products)
4. [ ] Add middleware (auth, permissions)
5. [ ] Test routing

**Dependencies:** Phase 4

**Estimated Files:**
- `src/modules/product/routes.rs`
- `src/modules/product/mod.rs`

---

### Phase 6: Integration & Testing
**Priority:** MEDIUM

**Tasks:**
1. [ ] Integration tests (full flow: create → upload → view → buy)
2. [ ] Test duplicate check (all 4 levels)
3. [ ] Test auto-fulfill pre-orders
4. [ ] Test inventory masking
5. [ ] Load testing (bulk upload)

**Dependencies:** Phase 5

---

### Phase 7: Optional Features
**Priority:** LOW

**Tasks:**
1. [ ] Bulk edit products (if approved)
2. [ ] Scheduled publishing (if approved)
3. [ ] Product analytics (if approved)
4. [ ] WebSocket real-time updates (if approved)

**Dependencies:** Phase 6

---

### Phase 8: Documentation
**Priority:** LOW

**Tasks:**
1. [ ] Add rustdoc comments
2. [ ] Update OpenAPI/Swagger docs
3. [ ] Write usage examples

**Dependencies:** Phase 7

---

## Implementation Order (Within Each Phase)

**For Product Module:**

```
Phase 1 - Foundation:
  ├─ Create Product struct (domain.rs)
  ├─ Create ProductItem struct (domain.rs)
  ├─ Create ProductVersion struct (domain.rs)
  ├─ Implement ProductRepository (repository.rs)
  └─ Test: Insert product, find by slug, update, delete

Phase 2 - DTOs:
  ├─ Create CreateProductRequest (dto.rs)
  ├─ Create UpdateProductRequest (dto.rs)
  ├─ Create UploadInventoryRequest (dto.rs)
  ├─ Create ProductResponse, ProductCard (dto.rs)
  ├─ Add validation rules
  └─ Test: Validation

Phase 3 - Service:
  ├─ Implement ProductService::create (service.rs)
  ├─ Implement ProductService::update (service.rs)
  ├─ Implement ProductService::search (service.rs)
  ├─ Implement InventoryService::upload (service.rs)
  ├─ Implement check_duplicates_4_levels (service.rs)
  ├─ Implement encrypt_items (service.rs)
  ├─ Implement auto_fulfill_preorders (service.rs)
  └─ Test: All service functions

Phase 4 - Handler:
  ├─ Implement list_products_handler (public)
  ├─ Implement view_product_handler (public)
  ├─ Implement create_product_handler (vendor)
  ├─ Implement update_product_handler (vendor)
  ├─ Implement upload_inventory_handler (vendor)
  ├─ Implement restock_handler (vendor)
  ├─ Implement admin_search_products_handler (admin)
  └─ Add permission checks

Phase 5 - Routes:
  ├─ Add public routes (/api/products)
  ├─ Add vendor routes (/api/vendor/products)
  ├─ Add admin routes (/api/admin/products)
  └─ Add middleware

Phase 6 - Integration:
  ├─ Test: Create product → Upload inventory → View
  ├─ Test: Duplicate check (all 4 levels)
  ├─ Test: Restock → Auto-fulfill pre-orders
  ├─ Test: Buy → Item marked sold → Masked in inventory
  └─ Load test: Bulk upload 10,000 items

Phase 7 - Optional:
  ├─ Implement bulk edit (if approved)
  ├─ Implement scheduled publishing (if approved)
  └─ Implement analytics (if approved)

Phase 8 - Docs:
  ├─ Add rustdoc
  ├─ Update OpenAPI
  └─ Write examples
```

---

## Dependencies Map

```
┌─────────────────────────────────────────────────────────┐
│              PRODUCT MODULE DEPENDENCIES                │
└─────────────────────────────────────────────────────────┘

                    ┌──────────────┐
                    │ USER MODULE  │
                    │ (Auth)       │
                    └──────┬───────┘
                           │
                    ┌──────▼───────┐
                    │ SHOP MODULE  │◀───────┐
                    └──────┬───────┘        │
                           │                │
                    ┌──────▼───────┐        │
                    │ CATEGORY     │        │
                    │ MODULE       │        │
                    └──────┬───────┘        │
                           │                │
        ┌──────────────────┼────────────────┘
        │                  │
        ▼                  ▼
  ┌──────────┐      ┌──────────┐
  │ PRODUCT  │      │ STORAGE  │
  │ MODULE   │─────▶│ MODULE   │
  │          │      │ (Images) │
  └─────┬────┘      └──────────┘
        │
        ├──────────────────┬──────────────────┐
        │                  │                  │
        ▼                  ▼                  ▼
  ┌──────────┐      ┌──────────┐      ┌──────────┐
  │ ORDER    │      │ PRE-     │      │ REVIEW   │
  │ MODULE   │      │ ORDER    │      │ MODULE   │
  └──────────┘      └──────────┘      └──────────┘
```

---

## Next Steps

1. [ ] **Review Pending Suggestions** - Decide which V2 improvements to implement
2. [ ] **Answer Open Questions** - Resolve design decisions
3. [ ] **Approve Design** - Sign off on this brainstorm
4. [ ] **Create Implementation Plan** - Run `/write-plan product` to create detailed plan
5. [ ] **Set Up Git Worktree** - Create isolated branch for development
6. [ ] **Start Phase 1** - Begin with domain models and repository

---

## Document Info

**Created:** 2025-01-04
**Module:** Product Module
**Status:** Draft - Pending Review
**Related Documents:**
- [V2 Full Flows](../../full-flows.md)
- [Wallet System Design](./01-wallet-system-design.md)
- [Shop Flows](./shop/01-complete-flows.md)
