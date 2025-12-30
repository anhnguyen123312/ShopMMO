# Product Module Context

## Status: 📋 Not Started

## Planned Files
```
src/modules/product/
├── mod.rs
├── domain.rs      # Product, ProductItem, Category
├── dto.rs         # CreateProductReq, ProductRes, ItemRes
├── handler.rs     
├── service.rs     # ProductService, InventoryService
├── repository.rs  
└── routes.rs
```

## Core Concept
**Digital Products** với auto-delivery. Kho hàng dạng text (account, license, key).

## Data Models (from V1)
```rust
Product {
    id: ObjectId,
    shop_id: ObjectId,
    category_id: ObjectId,
    name: String,
    slug: String,            // unique per shop
    description: String,
    image_url: Option<String>,
    price: Decimal,
    original_price: Option<Decimal>,
    min_purchase: u32,       // default 1
    max_purchase: u32,       // 0 = unlimited
    allow_preorder: bool,
    allow_resell: bool,
    reseller_price: Option<Decimal>,
    hide_stock: bool,
    require_2fa: bool,
    status: ProductStatus,   // Draft | Active | Hidden | Deleted
    stock_count: u32,        // denormalized
    sold_count: u32,
    created_at: DateTime,
}

ProductItem {
    id: ObjectId,
    product_id: ObjectId,
    content: String,         // encrypted: "email|pass|2fa"
    is_sold: bool,
    sold_at: Option<DateTime>,
    order_id: Option<ObjectId>,
    hold_until: Option<DateTime>,  // for pre-order
    added_at: DateTime,
}

Category {
    id: ObjectId,
    parent_id: Option<ObjectId>,
    name: String,
    slug: String,
    icon: Option<String>,
    sort_order: i32,
    is_active: bool,
}
```

## Key Flows

### Upload Inventory
```
Vendor paste text → Parse by line → Check duplicates (global) → Store items
```

### Purchase (Auto-delivery)
```
Buyer checkout → Lock items → Create order → Deduct balance
              → Mark items sold → Return content to buyer
```

### Duplicate Check
Mỗi item chỉ bán 1 lần trên toàn platform. Hash và check trước khi add.

## Endpoints (Planned)
| Method | Path | Role |
|--------|------|------|
| GET | /products | Public |
| GET | /products/{id} | Public |
| POST | /vendor/products | Vendor |
| PUT | /vendor/products/{id} | Vendor |
| POST | /vendor/products/{id}/items | Vendor |
| GET | /categories | Public |

## Refs
- V1 Products: [docs/v1/04-products-inventory.md](../../docs/v1/04-products-inventory.md)
- Related: [context/order.md](order.md), [context/shop.md](shop.md)

## Notes
- Realtime stock check (polling 30s)
- Item content should be encrypted at rest
- Bulk upload cần batch processing
