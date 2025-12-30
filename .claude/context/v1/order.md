# Order Module Context

## Status: 📋 Not Started

## Planned Files
```
src/modules/order/
├── mod.rs
├── domain.rs      # Order, OrderItem, Dispute
├── dto.rs         # CreateOrderReq, OrderRes
├── handler.rs     
├── service.rs     # OrderService, DisputeService
├── repository.rs  
└── routes.rs
```

## Order Flow (V1 Reference)
```
┌──────────────────────────────────────────────────────────┐
│  Buyer Browse → Add Cart → Checkout                      │
│       ↓                                                  │
│  Check Balance ──No──→ Redirect /deposit                 │
│       ↓ Yes                                              │
│  Lock Items (pessimistic) → Create Order → Deduct Wallet │
│       ↓                                                  │
│  Mark Items Sold → Deliver Content → Order Complete      │
│       ↓                                                  │
│  Vendor: +pending balance → After 3 days: release        │
└──────────────────────────────────────────────────────────┘
```

## Data Models
```rust
Order {
    id: ObjectId,
    order_number: String,     // TH-{timestamp}-{random}
    buyer_id: ObjectId,
    shop_id: ObjectId,
    vendor_id: ObjectId,
    
    items: Vec<OrderItem>,
    subtotal: Decimal,
    discount: Decimal,        // from coupon
    total: Decimal,
    
    status: OrderStatus,      // Pending | Completed | Disputed | Refunded
    payment_status: PaymentStatus,
    
    coupon_code: Option<String>,
    note: Option<String>,
    
    completed_at: Option<DateTime>,
    created_at: DateTime,
}

OrderItem {
    product_id: ObjectId,
    product_name: String,     // snapshot
    unit_price: Decimal,      // snapshot
    quantity: u32,
    content: Vec<String>,     // delivered items content
}

Dispute {
    id: ObjectId,
    order_id: ObjectId,
    buyer_id: ObjectId,
    vendor_id: ObjectId,
    
    reason: String,
    evidence: Vec<String>,    // image URLs
    status: DisputeStatus,    // Open | VendorResponded | Resolved | Rejected
    
    vendor_response: Option<String>,
    admin_decision: Option<String>,
    refund_amount: Option<Decimal>,
    
    resolved_at: Option<DateTime>,
    created_at: DateTime,
}
```

## Pre-order Flow
```
Stock = 0 & allow_preorder = true
  ↓
Buyer đặt trước (chọn wait time 1-7 days)
  ↓
Create Order status = PreOrder
Hold balance (not deduct)
  ↓
Vendor restock → Auto-match pending orders → Complete
  ↓
Timeout → Auto-cancel → Refund hold
```

## Dispute Window
- Buyer có 3 ngày để dispute (trước khi tiền release cho vendor)
- Vendor respond → Admin quyết định nếu không resolve

## Endpoints (Planned)
| Method | Path | Role |
|--------|------|------|
| POST | /orders | Buyer |
| GET | /orders | Auth |
| GET | /orders/{id} | Auth (owner) |
| POST | /orders/{id}/dispute | Buyer |
| POST | /vendor/orders/{id}/respond | Vendor |
| POST | /admin/disputes/{id}/resolve | Admin |

## Refs
- V1 Orders: [docs/v1/all.md](../../docs/v1/all.md)
- Related: [context/wallet.md](wallet.md), [context/product.md](product.md)
