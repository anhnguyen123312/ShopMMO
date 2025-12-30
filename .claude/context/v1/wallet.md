# Wallet Module Context

## Status: 🚧 Skeleton Created - Needs V2 Implementation

## Files
```
src/modules/wallet/
├── mod.rs
├── domain.rs      # Wallet, WalletTransaction (TODO)
├── dto.rs         # BalanceRes, DepositReq (TODO)
├── handler.rs     # get_balance (TODO others)
├── service.rs     # WalletService (TODO)
├── repository.rs  # WalletRepo (TODO)
└── routes.rs
```

## V1 Wallet Structure (Reference)
```
BUYER: Available Balance only
VENDOR: Available + Pending (3-day hold)
```

## Transaction Types
| Type | Description | Buyer | Vendor |
|------|-------------|-------|--------|
| deposit | Nạp tiền | ✅ | ✅ |
| purchase | Mua hàng | ✅(-) | - |
| sale | Bán hàng | - | ✅(+pending) |
| sale_released | Release sau 3 ngày | - | ✅(pending→avail) |
| refund | Hoàn tiền | ✅(+) | ✅(-) |
| withdraw | Rút tiền | - | ✅(-) |
| commission | Hoa hồng CTV | ✅(+) | - |
| adjustment | Admin điều chỉnh | ✅ | ✅ |

## V2 Design Models
```rust
// See: docs/v2/01-wallet-system-design.md

WalletTransaction {
    id, wallet_id, type, amount,
    balance_before, balance_after,   // Snapshot
    order_id, escrow_hold_id,
    description, created_at
}

EscrowHold {
    id, wallet_id, order_id,
    amount, hold_until,
    status: Holding | Released | Refunded,
    released_at
}

WithdrawalRequest {
    id, wallet_id, amount,
    bank_info, status, admin_note,
    created_at, processed_at
}
```

## Endpoints (Planned)
| Method | Path | Handler | Auth |
|--------|------|---------|------|
| GET | /wallet/balance | get_balance | Bearer |
| GET | /wallet/transactions | list_txs | Bearer |
| POST | /wallet/deposit | request_deposit | Bearer |
| POST | /wallet/withdraw | request_withdraw | Vendor |
| POST | /wallet/transfer | p2p_transfer | Bearer |

## Refs
- V1 Wallet: [docs/v1/06-wallet-payment.md](../../docs/v1/06-wallet-payment.md)
- V2 Design: [docs/v2/01-wallet-system-design.md](../../docs/v2/01-wallet-system-design.md)
- Related: [context/order.md](order.md), [context/escrow.md](escrow.md)

## Implementation Phases
1. Core Models + Balance Query
2. Transaction System + Snapshots
3. Deposit System (Manual + Auto)
4. Withdrawal System
5. Escrow + Auto-release
6. P2P Transfer
7. Reports + Reconciliation
