# Shop Module Context

## Status: 📋 Not Started

## Concept
Mỗi Vendor có 1 Shop (gian hàng). Shop chứa products và có rating riêng.

## Data Model
```rust
Shop {
    id: ObjectId,
    vendor_id: ObjectId,     // 1-1 with User(role=Vendor)
    name: String,
    slug: String,            // unique, URL-friendly
    description: Option<String>,
    logo_url: Option<String>,
    banner_url: Option<String>,
    
    // Stats (denormalized)
    total_products: u32,
    total_sold: u32,
    rating: f32,             // 0.0 - 5.0
    rating_count: u32,
    
    // Settings
    allow_reseller: bool,
    auto_approve_reseller: bool,
    default_reseller_discount: Option<Decimal>,
    
    // Status
    is_verified: bool,
    is_active: bool,
    
    created_at: DateTime,
    updated_at: DateTime,
}

ShopRating {
    id: ObjectId,
    shop_id: ObjectId,
    order_id: ObjectId,
    buyer_id: ObjectId,
    rating: u8,              // 1-5
    comment: Option<String>,
    created_at: DateTime,
}
```

## Endpoints (Planned)
| Method | Path | Role |
|--------|------|------|
| GET | /shops | Public |
| GET | /shops/{slug} | Public |
| GET | /vendor/shop | Vendor |
| PUT | /vendor/shop | Vendor |
| GET | /shops/{id}/products | Public |

## Refs
- Related: [context/product.md](product.md), [context/order.md](order.md)
- Reseller: [docs/v1/02-user-roles.md](../../docs/v1/02-user-roles.md)
