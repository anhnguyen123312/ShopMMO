# Escrow Module Context

## Status: 📋 Part of Wallet V2

## Concept
Escrow = Giữ tiền 3 ngày trước khi release cho vendor. Bảo vệ buyer trong dispute window.

## Flow
```
Order Complete
     ↓
Create EscrowHold {
    wallet_id: vendor_wallet,
    order_id,
    amount,
    hold_until: now + 3 days,
    status: Holding
}
     ↓
Vendor pending_balance += amount
     ↓
┌─────────────────────────────┐
│ Wait 3 days                 │
│                             │
│ [No Dispute]    [Dispute]   │
│      ↓              ↓       │
│  Auto-release   Admin Review │
│      ↓              ↓       │
│  Available++    Refund/Release│
└─────────────────────────────┘
```

## Data Model
```rust
EscrowHold {
    id: ObjectId,
    wallet_id: ObjectId,
    order_id: ObjectId,
    amount: Decimal,
    hold_until: DateTime,
    status: EscrowStatus,     // Holding | Released | Refunded
    released_at: Option<DateTime>,
    released_by: Option<ObjectId>,  // null = auto, Some = manual
    created_at: DateTime,
}

enum EscrowStatus {
    Holding,    // Within 3-day window
    Released,   // Money moved to available
    Refunded,   // Money returned to buyer
}
```

## Configurable Hold Periods (V2)
```rust
OrderTypeConfig {
    order_type: String,      // "digital", "preorder", "reseller"
    hold_days: u32,          // 3, 7, etc.
    auto_release: bool,
}
```

## Cron Job: Auto-release
```
Every hour:
  Find EscrowHold where:
    - status = Holding
    - hold_until <= now
    - No active dispute on order
  
  For each:
    - Update status = Released
    - Vendor: pending -= amount, available += amount
    - Create WalletTransaction type = sale_released
```

## Refs
- Part of: [context/wallet.md](wallet.md)
- Related: [context/order.md](order.md)
- V2 Design: [docs/v2/01-wallet-system-design.md](../../docs/v2/01-wallet-system-design.md)
