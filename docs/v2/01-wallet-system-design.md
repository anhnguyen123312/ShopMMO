# Wallet System V2 - Design Document

## Overview

This document outlines the complete design for the Wallet V2 system using MongoDB. The wallet system manages AP (Account Points) currency, handles deposits/withdrawals, implements escrow for seller protection, and tracks all money flows.

**Last Updated**: 2025-01-29

---

## 1. Requirements Summary

### Key Requirements
- **AP Currency**: Internal currency where 1 AP = 1,000 VND (fixed rate)
- **Balance States**:
  - AP Current (available for spending/withdrawal)
  - AP Pending Cashout (escrow, waiting for release)
- **Variable Hold Periods**: 3-5 days based on order type
- **P2P Transfers**: Users can send AP to each other
- **Admin Manual Operations**: Manual deposits/withdrawals/adjustments
- **Money Flow Tracking**: Clear visibility of seller, admin, and user money flows
- **Single Wallet**: One wallet per user (serves both buyer and seller roles)
- **Currency**: VND only
- **Database**: MongoDB

### Comparison with V1
- **V1**: Uses MySQL with separate buyer/vendor wallet structures
- **V2**: MongoDB with unified wallet, enhanced tracking, configurable escrow

---

## 2. AP Currency Specification

### Definition
- **Name**: AP (Account Points)
- **Exchange Rate**: 1 AP = 1,000 VND (fixed)
- **Precision**: Integer values only (no decimals)
- **Direction**: Two-way conversion
  - Deposit: VND → AP
  - Withdrawal: AP → VND

### Examples
```
User deposits 500,000 VND
→ Receives: 500 AP

User withdraws 1,000 AP
→ Receives: 1,000,000 VND
```

---

## 3. MongoDB Collections Schema

### 3.1 Collection: `wallets`

Stores user wallet balances and lifetime statistics.

```javascript
{
  _id: ObjectId("..."),
  userId: ObjectId("..."),  // Reference to users collection (unique)

  // Balance States
  balances: {
    apCurrent: 5000,          // Available AP (can spend/withdraw)
    apPendingCashout: 2000,   // AP in escrow (waiting for release)
    apTotal: 7000             // apCurrent + apPendingCashout (calculated)
  },

  // Lifetime Statistics
  lifetime: {
    totalDeposited: 50000,    // Total AP deposited all-time
    totalWithdrawn: 20000,    // Total AP withdrawn all-time
    totalEarned: 30000,       // Total AP earned from sales
    totalSpent: 25000,        // Total AP spent on purchases
    totalSent: 5000,          // Total AP sent via P2P transfers
    totalReceived: 3000       // Total AP received via P2P transfers
  },

  // Metadata
  currency: "VND",            // Base currency
  status: "active",           // active, frozen, suspended
  frozenAmount: 0,            // AP locked due to disputes/violations

  // Timestamps
  createdAt: ISODate("2025-01-29T00:00:00Z"),
  updatedAt: ISODate("2025-01-29T10:30:00Z")
}
```

**Indexes:**
```javascript
db.wallets.createIndex({ userId: 1 }, { unique: true })
db.wallets.createIndex({ "balances.apCurrent": 1 })
db.wallets.createIndex({ status: 1 })
db.wallets.createIndex({ updatedAt: -1 })
```

---

### 3.2 Collection: `wallet_transactions`

Complete ledger of all money movements. **Source of truth** for all balance changes.

```javascript
{
  _id: ObjectId("..."),
  transactionNumber: "TXN-20250129-00001",  // Unique, human-readable ID

  // User Info
  userId: ObjectId("..."),
  userType: "buyer",          // buyer, seller, both

  // Transaction Type & Category
  type: "purchase",           // See Transaction Types below
  category: "payment",        // deposit, withdrawal, payment, transfer, adjustment, commission

  // Amount Details
  amount: {
    ap: 100,                  // Transaction amount in AP
    vnd: 100000,              // Equivalent in VND (ap × 1000)
    direction: "debit"        // debit (minus), credit (plus)
  },

  // Balance Snapshots (Audit Trail)
  balanceSnapshot: {
    before: {
      apCurrent: 5100,
      apPendingCashout: 2000,
      apTotal: 7100
    },
    after: {
      apCurrent: 5000,
      apPendingCashout: 2000,
      apTotal: 7000
    }
  },

  // Source/Method (for deposits/withdrawals)
  source: {
    type: "manual",           // manual, bank, momo, usdt, paypal, system
    gateway: null,            // Gateway name if applicable
    reference: "ADMIN-123",   // External reference (bank txn, gateway id)
    metadata: {               // Flexible field for gateway-specific data
      adminId: ObjectId("..."),
      reason: "Manual deposit by admin",
      proofUrl: "https://storage.example.com/proof.jpg"
    }
  },

  // Related Entities
  relatedTo: {
    orderId: ObjectId("..."),          // If related to order
    sellerId: ObjectId("..."),         // If purchase/sale
    buyerId: ObjectId("..."),          // If purchase/sale
    transferToUserId: ObjectId("..."), // If P2P transfer
    escrowId: ObjectId("..."),         // If escrow-related
    withdrawalId: ObjectId("...")      // If withdrawal
  },

  // Status & Processing
  status: "completed",        // pending, processing, completed, failed, cancelled

  // Description
  description: "Purchase order #ORD-001",
  notes: "Admin note if manual adjustment",

  // Admin Action (if manual)
  adminAction: {
    adminId: ObjectId("..."),
    adminName: "admin@example.com",
    actionType: "manual_deposit",
    approvedBy: ObjectId("..."),     // For maker-checker workflow
    approvedAt: ISODate("...")
  },

  // Timestamps
  createdAt: ISODate("2025-01-29T10:00:00Z"),
  completedAt: ISODate("2025-01-29T10:00:01Z"),

  // Soft delete
  deletedAt: null
}
```

**Transaction Types:**
- `deposit` - User deposits money
- `withdraw` - User withdraws money
- `purchase` - Buyer purchases product
- `sale` - Seller receives payment (to pending)
- `release` - Escrow released (pending → current)
- `refund` - Order refunded
- `transfer_send` - P2P transfer sent
- `transfer_receive` - P2P transfer received
- `adjustment` - Admin manual adjustment
- `commission` - Platform commission

**Indexes:**
```javascript
db.wallet_transactions.createIndex({ transactionNumber: 1 }, { unique: true })
db.wallet_transactions.createIndex({ userId: 1, createdAt: -1 })
db.wallet_transactions.createIndex({ type: 1, status: 1 })
db.wallet_transactions.createIndex({ "relatedTo.orderId": 1 })
db.wallet_transactions.createIndex({ status: 1, createdAt: -1 })
db.wallet_transactions.createIndex({ createdAt: -1 })
db.wallet_transactions.createIndex({ "adminAction.adminId": 1 })
```

---

### 3.3 Collection: `escrow_holds`

Tracks all pending cashouts with configurable hold periods.

```javascript
{
  _id: ObjectId("..."),
  escrowNumber: "ESC-20250129-00001",

  // Order & Participants
  orderId: ObjectId("..."),
  sellerId: ObjectId("..."),
  buyerId: ObjectId("..."),

  // Amount Details
  amount: {
    ap: 95,                   // AP amount held (after commission)
    originalAp: 100,          // Original order amount
    commissionAp: 5,          // Platform commission taken
    vnd: 95000               // VND equivalent
  },

  // Hold Period Configuration
  holdConfig: {
    orderType: "digital_goods",       // Order type determines hold period
    holdDays: 3,                      // Days to hold (from order type config)
    releaseAt: ISODate("2025-02-01T10:00:00Z"),  // Auto-release timestamp
    releaseCondition: "auto",         // auto, manual, dispute_resolved

    // Early release conditions
    allowEarlyRelease: false,         // If buyer can trigger early release
    earlyReleaseRequested: false,
    earlyReleaseRequestedAt: null
  },

  // Status
  status: "holding",        // holding, released, refunded, partial_refund, disputed

  // Release/Refund Info
  resolution: {
    resolvedAt: null,
    resolvedBy: null,       // ObjectId of admin who resolved
    resolutionType: null,   // auto_release, manual_release, full_refund, partial_refund
    refundedAp: 0,
    releasedAp: 0,

    // Transaction references
    releaseTransactionId: null,
    refundTransactionId: null
  },

  // Dispute (if any)
  dispute: {
    hasDispute: false,
    disputeId: null,
    disputeStatus: null     // pending, investigating, resolved
  },

  // Timestamps
  createdAt: ISODate("2025-01-29T10:00:00Z"),
  updatedAt: ISODate("2025-01-29T10:00:00Z"),
  releasedAt: null
}
```

**Indexes:**
```javascript
db.escrow_holds.createIndex({ escrowNumber: 1 }, { unique: true })
db.escrow_holds.createIndex({ orderId: 1 }, { unique: true })
db.escrow_holds.createIndex({ sellerId: 1, status: 1 })
db.escrow_holds.createIndex({ status: 1, "holdConfig.releaseAt": 1 })  // For cron job
db.escrow_holds.createIndex({ "dispute.hasDispute": 1, status: 1 })
db.escrow_holds.createIndex({ createdAt: -1 })
```

---

### 3.4 Collection: `withdrawal_requests`

Manages user withdrawal requests with admin approval workflow.

```javascript
{
  _id: ObjectId("..."),
  withdrawalNumber: "WTD-20250129-00001",

  // User Info
  userId: ObjectId("..."),
  userEmail: "seller@example.com",

  // Amount Details
  amount: {
    requestedAp: 1000,       // AP user wants to withdraw
    feeAp: 0,                // Withdrawal fee in AP
    netAp: 1000,             // Net AP to withdraw
    vnd: 1000000,            // VND to send (1000 AP × 1000)
    exchangeRate: 1000       // AP to VND rate at request time
  },

  // Withdrawal Method & Destination
  method: "bank",            // bank, momo, crypto (future)
  destination: {
    type: "bank_account",
    bankName: "Vietcombank",
    accountNumber: "1234567890",
    accountName: "NGUYEN VAN A",
    bankBranch: "Ho Chi Minh City",

    // Snapshot at request time (in case user changes bank info later)
    snapshotAt: ISODate("2025-01-29T10:00:00Z")
  },

  // Status & Processing
  status: "pending",         // pending, processing, completed, rejected, cancelled

  // Admin Processing
  processing: {
    assignedTo: null,        // Admin processing this
    startedAt: null,

    // Verification
    verified: false,
    verifiedBy: null,
    verifiedAt: null,

    // Completion
    completedBy: null,
    completedAt: null,
    gatewayReference: null,  // Bank transaction reference
    proofUrl: null,          // Screenshot of bank transfer

    // Rejection
    rejectedBy: null,
    rejectedAt: null,
    rejectReason: null
  },

  // Wallet Transaction References
  deductTransactionId: ObjectId("..."),  // Transaction that deducted from wallet
  refundTransactionId: null,             // If rejected, refund transaction

  // Timestamps
  createdAt: ISODate("2025-01-29T10:00:00Z"),
  updatedAt: ISODate("2025-01-29T10:00:00Z"),

  // Notes
  userNote: "Urgent withdrawal needed",
  adminNotes: []            // Array of { adminId, note, createdAt }
}
```

**Indexes:**
```javascript
db.withdrawal_requests.createIndex({ withdrawalNumber: 1 }, { unique: true })
db.withdrawal_requests.createIndex({ userId: 1, createdAt: -1 })
db.withdrawal_requests.createIndex({ status: 1, createdAt: -1 })
db.withdrawal_requests.createIndex({ "processing.assignedTo": 1, status: 1 })
db.withdrawal_requests.createIndex({ createdAt: -1 })
```

---

### 3.5 Collection: `deposit_requests`

Tracks deposit requests (manual verification for now, auto-detect integration later).

```javascript
{
  _id: ObjectId("..."),
  depositNumber: "DEP-20250129-00001",

  // User Info
  userId: ObjectId("..."),
  userEmail: "buyer@example.com",

  // Amount Details
  amount: {
    vnd: 500000,             // VND deposited
    ap: 500,                 // AP to credit (500,000 ÷ 1,000)
    exchangeRate: 1000,      // VND per AP
    feeVnd: 0,               // Deposit fee in VND
    feeAp: 0                 // Deposit fee in AP
  },

  // Deposit Method & Source
  method: "manual",          // manual, bank, momo, usdt, paypal
  source: {
    type: "bank_transfer",
    bankName: "Vietcombank",

    // For manual deposits
    referenceNumber: null,   // User-provided reference
    proofUrl: "https://storage.example.com/proof.jpg",

    // For auto-detected (future)
    gatewayReference: null,
    gatewayTransactionId: null,
    detectedAt: null,

    // Metadata
    metadata: {
      userProvidedInfo: "Transferred at 10:30 AM on 2025-01-29"
    }
  },

  // Status & Verification
  status: "pending",         // pending, verified, completed, rejected

  // Admin Verification
  verification: {
    verifiedBy: null,        // Admin who verified
    verifiedAt: null,
    verificationNote: null,

    rejectedBy: null,
    rejectedAt: null,
    rejectReason: null
  },

  // Wallet Transaction Reference
  creditTransactionId: null,  // Transaction that credited wallet

  // Timestamps
  createdAt: ISODate("2025-01-29T10:00:00Z"),
  updatedAt: ISODate("2025-01-29T10:00:00Z"),
  completedAt: null,

  // User Note
  userNote: "Deposit for purchasing products"
}
```

**Indexes:**
```javascript
db.deposit_requests.createIndex({ depositNumber: 1 }, { unique: true })
db.deposit_requests.createIndex({ userId: 1, createdAt: -1 })
db.deposit_requests.createIndex({ status: 1, createdAt: -1 })
db.deposit_requests.createIndex({ method: 1, status: 1 })
db.deposit_requests.createIndex({ createdAt: -1 })
```

---

### 3.6 Collection: `order_type_configs`

Configurable hold periods and commission rates per order type.

```javascript
{
  _id: ObjectId("..."),
  orderType: "digital_goods",         // Unique identifier
  displayName: "Digital Goods",
  description: "Digital products like accounts, keys, etc.",

  // Escrow Configuration
  escrow: {
    holdDays: 3,                      // Default hold period in days
    allowEarlyRelease: true,          // Buyer can release early
    autoReleaseEnabled: true,         // Auto-release after hold period

    // Conditional hold periods based on criteria
    conditionalHolds: [
      {
        condition: "orderAmount",     // Field to check
        operator: ">",                // Comparison operator
        value: 1000,                  // If order > 1000 AP
        holdDays: 5                   // Hold for 5 days instead
      },
      {
        condition: "sellerReputation",
        operator: "<",
        value: 4.0,                   // If seller rating < 4.0
        holdDays: 7                   // Hold for 7 days
      },
      {
        condition: "sellerNewAccount",
        operator: "=",
        value: true,                  // If seller account < 30 days old
        holdDays: 10                  // Hold for 10 days
      }
    ]
  },

  // Commission Configuration
  commission: {
    type: "percentage",               // percentage or fixed
    value: 5,                         // 5% commission
    minAp: 1,                         // Minimum 1 AP commission
    maxAp: null                       // No maximum cap
  },

  // Status
  active: true,

  // Timestamps
  createdAt: ISODate("2025-01-29T00:00:00Z"),
  updatedAt: ISODate("2025-01-29T00:00:00Z")
}
```

**Example Order Types:**
- `digital_goods` - 3 days hold, 5% commission
- `physical_goods` - 7 days hold, 3% commission
- `services` - 5 days hold, 10% commission
- `high_value` - 10 days hold, 3% commission

**Indexes:**
```javascript
db.order_type_configs.createIndex({ orderType: 1 }, { unique: true })
db.order_type_configs.createIndex({ active: 1 })
```

---

### 3.7 Collection: `money_flow_summary`

Daily/hourly aggregated summaries for admin dashboard and reconciliation.

```javascript
{
  _id: ObjectId("..."),

  // Time Period
  periodType: "daily",       // hourly, daily, monthly
  periodDate: ISODate("2025-01-29T00:00:00Z"),

  // Inflows (AP coming into system)
  inflows: {
    deposits: {
      totalAp: 10000,
      totalVnd: 10000000,
      count: 50,
      byMethod: {
        manual: { ap: 5000, vnd: 5000000, count: 20 },
        bank: { ap: 3000, vnd: 3000000, count: 20 },
        momo: { ap: 2000, vnd: 2000000, count: 10 }
      }
    },
    transfers: {              // P2P transfers (internal movement)
      totalAp: 500,
      count: 10
    }
  },

  // Outflows (AP leaving system)
  outflows: {
    withdrawals: {
      totalAp: 5000,
      totalVnd: 5000000,
      count: 25,
      byMethod: {
        bank: { ap: 4000, vnd: 4000000, count: 20 },
        momo: { ap: 1000, vnd: 1000000, count: 5 }
      }
    },
    transfers: {              // P2P transfers (internal movement)
      totalAp: 500,
      count: 10
    }
  },

  // Internal Movements (AP moving between users)
  internal: {
    purchases: {
      totalAp: 8000,
      count: 120,
      averageAp: 66.67
    },
    refunds: {
      totalAp: 200,
      count: 5
    },
    escrowReleased: {
      totalAp: 7500,
      count: 100
    }
  },

  // System Balances Snapshot
  systemSnapshot: {
    totalApInWallets: 150000,         // Sum of all user wallets (apTotal)
    totalApCurrent: 100000,           // Sum of all apCurrent
    totalApPending: 50000,            // Sum of all apPendingCashout
    totalVndEquivalent: 150000000,    // Total AP × 1000

    // Real money tracking (actual VND in gateways)
    realMoneyBalance: {
      bank: 80000000,                 // Actual VND in bank account
      momo: 20000000,                 // Actual VND in MoMo wallet
      total: 100000000
    },

    // Reconciliation
    difference: -50000000,            // realMoneyBalance - totalVndEquivalent
    reconciled: false,
    lastReconciledAt: null
  },

  // Commission Earned by Platform
  commission: {
    totalAp: 400,
    totalVnd: 400000,
    count: 120,
    averageAp: 3.33
  },

  // Timestamps
  createdAt: ISODate("2025-01-29T23:59:59Z"),
  updatedAt: ISODate("2025-01-29T23:59:59Z")
}
```

**Indexes:**
```javascript
db.money_flow_summary.createIndex({ periodType: 1, periodDate: -1 })
db.money_flow_summary.createIndex({ periodDate: -1 })
```

---

## 4. Complete System Flowcharts

This section provides comprehensive flowcharts for all wallet system functions, including main flows, sub-flows, and their interconnections.

### 4.1 System Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        WALLET SYSTEM V2 - HIGH LEVEL                        │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────┐         ┌──────────────┐         ┌─────────────┐
│   BUYER     │         │    ADMIN     │         │   SELLER    │
│             │         │              │         │             │
│ - Deposit   │         │ - Manual Ops │         │ - Withdraw  │
│ - Purchase  │         │ - Approve    │         │ - Earnings  │
│ - Transfer  │         │ - Adjust     │         │ - Transfer  │
└──────┬──────┘         └──────┬───────┘         └──────┬──────┘
       │                       │                        │
       │                       │                        │
       └───────────────────────┼────────────────────────┘
                               │
                               ▼
                    ┌──────────────────────┐
                    │   WALLET SERVICE     │
                    │                      │
                    │ ┌──────────────────┐ │
                    │ │ Balance Manager  │ │
                    │ └──────────────────┘ │
                    │ ┌──────────────────┐ │
                    │ │Transaction Ledger│ │
                    │ └──────────────────┘ │
                    │ ┌──────────────────┐ │
                    │ │ Escrow Manager   │ │
                    │ └──────────────────┘ │
                    └──────────┬───────────┘
                               │
                ┌──────────────┼──────────────┐
                │              │              │
                ▼              ▼              ▼
        ┌──────────────┐ ┌──────────┐ ┌─────────────┐
        │   MONGODB    │ │  CRON    │ │ NOTIFICATION│
        │              │ │  JOBS    │ │  SERVICE    │
        │ - wallets    │ │          │ │             │
        │ - txns       │ │ - Release│ │ - Email     │
        │ - escrows    │ │ - Summary│ │ - Push      │
        │ - requests   │ │ - Reports│ │ - SMS       │
        └──────────────┘ └──────────┘ └─────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                            DATA FLOW TYPES                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│ AP INFLOW:   External money → System (Deposits)                            │
│ AP OUTFLOW:  System → External money (Withdrawals)                         │
│ AP INTERNAL: User ↔ User (Purchases, Transfers)                            │
│ AP ESCROW:   Current → Pending → Current (Time-based release)              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### 4.2 Master Flow: Wallet Balance State Machine

```
┌─────────────────────────────────────────────────────────────────────────────┐
│              WALLET BALANCE STATE MACHINE - LIFECYCLE                       │
└─────────────────────────────────────────────────────────────────────────────┘

                         ┌──────────────────────┐
                         │  EXTERNAL MONEY      │
                         │  (VND in Bank/MoMo)  │
                         └──────────┬───────────┘
                                    │
                                    │ DEPOSIT (Manual/Auto)
                                    │ VND → AP (÷ 1000)
                                    ▼
                         ┌──────────────────────┐
                    ┌────│   AP CURRENT         │────┐
                    │    │  (Available Balance) │    │
                    │    └──────────────────────┘    │
                    │              │                  │
    WITHDRAW        │              │                  │ P2P TRANSFER SEND
    (Admin Approve) │              │ PURCHASE         │ (Immediate)
                    │              │ (Buyer pays)     │
                    ▼              ▼                  ▼
         ┌──────────────┐  ┌──────────────────┐  ┌──────────────┐
         │  WITHDRAWAL  │  │  AP PENDING      │  │ ANOTHER USER │
         │  REQUEST     │  │  CASHOUT         │  │ AP CURRENT   │
         │  (Pending)   │  │  (Escrow Hold)   │  │              │
         └──────┬───────┘  └────────┬─────────┘  └──────────────┘
                │                   │
                │ Admin              │ AUTO-RELEASE
                │ Processes          │ (3-5 days)
                ▼                   │ OR Manual Release
         ┌──────────────┐           │
         │  EXTERNAL    │◄──────────┘
         │  MONEY       │
         │  (VND out)   │   ┌────────────────────┐
         └──────────────┘   │  ESCROW RELEASED   │
                            │  Pending → Current │
                            └────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│ STATE TRANSITIONS & RULES                                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│ 1. DEPOSIT:     External VND → AP Current                                  │
│    - Requires: Admin verification (manual) OR Gateway callback (auto)      │
│    - Updates:  lifetime.totalDeposited                                      │
│                                                                             │
│ 2. PURCHASE:    Buyer AP Current → Seller AP Pending                       │
│    - Requires: Sufficient buyer balance                                     │
│    - Deducts:  Platform commission before adding to seller pending         │
│    - Creates:  Escrow hold record with release date                        │
│                                                                             │
│ 3. RELEASE:     Seller AP Pending → Seller AP Current                      │
│    - Trigger:  Auto (cron after holdDays) OR Manual (admin/buyer)          │
│    - Updates:  lifetime.totalEarned (already counted at purchase)           │
│                                                                             │
│ 4. WITHDRAW:    AP Current → Withdrawal Request → External VND             │
│    - Immediate: Deduct from AP Current                                      │
│    - Pending:   Admin must process and transfer real money                 │
│    - If reject: Refund to AP Current                                        │
│    - Updates:   lifetime.totalWithdrawn                                     │
│                                                                             │
│ 5. TRANSFER:    User A AP Current → User B AP Current                      │
│    - Immediate: Atomic transaction (deduct A, credit B)                    │
│    - Updates:   A.totalSent, B.totalReceived                               │
│                                                                             │
│ 6. REFUND:      Seller AP Pending → Buyer AP Current                       │
│    - Trigger:   Dispute resolved in favor of buyer                          │
│    - Reverses:  Purchase transaction                                        │
│    - Updates:   Escrow status to "refunded"                                │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### 4.3 Flow Type 1: DEPOSIT (Money Inflow)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    DEPOSIT FLOW - MANUAL BY ADMIN                           │
│ Type: INFLOW | Category: DEPOSIT | Actor: Admin + User                     │
└─────────────────────────────────────────────────────────────────────────────┘

START: User transfers VND to company bank account
│
├─── USER ACTION ───────────────────────────────────────────────────────────┐
│                                                                            │
│  [1] User transfers VND via bank/MoMo                                     │
│  [2] User takes screenshot/proof                                           │
│  [3] User contacts support OR submits deposit request                     │
│      - Provides: Amount, Screenshot, Transaction reference                │
│                                                                            │
└────────────────────────────────────────────────┬───────────────────────────┘
                                                 │
                                                 ▼
┌─── ADMIN RECEIVES REQUEST ──────────────────────────────────────────────────┐
│                                                                             │
│  [4] Admin reviews deposit request                                         │
│      ┌─────────────────────────────────────────┐                          │
│      │ CHECK:                                   │                          │
│      │ ✓ Screenshot authentic?                  │                          │
│      │ ✓ Amount matches bank statement?         │                          │
│      │ ✓ Not duplicate (already processed)?     │                          │
│      │ ✓ User account valid & active?           │                          │
│      └─────────────────────────────────────────┘                          │
│           │                                                                 │
│           ├─── REJECT ──────────────────┐                                  │
│           │                              │                                  │
│           │                              ▼                                  │
│           │                    [Admin marks rejected]                       │
│           │                    [Notifies user with reason]                  │
│           │                    END: No wallet change                        │
│           │                                                                 │
│           └─── APPROVE ─────────────────┐                                  │
│                                          │                                  │
└──────────────────────────────────────────┼──────────────────────────────────┘
                                           │
                                           ▼
┌─── SYSTEM PROCESSING (MongoDB Transaction) ─────────────────────────────────┐
│                                                                             │
│  [5] BEGIN TRANSACTION                                                     │
│                                                                             │
│  [6] Convert VND → AP                                                      │
│      Formula: AP = VND ÷ 1000                                              │
│      Example: 500,000 VND → 500 AP                                         │
│                                                                             │
│  [7] Get current wallet state                                              │
│      Query: db.wallets.findOne({ userId })                                 │
│      Capture: balances.apCurrent (for snapshot)                            │
│                                                                             │
│  [8] Create wallet_transaction record                                      │
│      ┌──────────────────────────────────────────┐                         │
│      │ transactionNumber: DEP-YYYYMMDD-XXXXX    │                         │
│      │ userId: [User ObjectId]                   │                         │
│      │ type: "deposit"                           │                         │
│      │ category: "deposit"                       │                         │
│      │ amount.ap: +500                           │                         │
│      │ amount.vnd: +500000                       │                         │
│      │ amount.direction: "credit"                │                         │
│      │ balanceSnapshot.before: {...}             │                         │
│      │ balanceSnapshot.after: {...}              │                         │
│      │ source.type: "manual"                     │                         │
│      │ source.metadata.adminId: [Admin ID]       │                         │
│      │ status: "completed"                       │                         │
│      └──────────────────────────────────────────┘                         │
│                                                                             │
│  [9] Update wallet balance                                                 │
│      UPDATE: db.wallets.updateOne(                                         │
│        { userId },                                                          │
│        {                                                                    │
│          $inc: {                                                            │
│            "balances.apCurrent": +500,                                     │
│            "lifetime.totalDeposited": +500                                 │
│          }                                                                  │
│        }                                                                    │
│      )                                                                      │
│                                                                             │
│  [10] Update deposit_request status                                        │
│       UPDATE: status = "completed"                                         │
│       SET: creditTransactionId, completedAt                                │
│                                                                             │
│  [11] COMMIT TRANSACTION                                                   │
│                                                                             │
└─────────────────────────────────────────────┬───────────────────────────────┘
                                              │
                                              ▼
┌─── POST-PROCESSING ──────────────────────────────────────────────────────────┐
│                                                                              │
│  [12] Update money_flow_summary (async)                                    │
│       - Increment daily inflows.deposits.manual                             │
│       - Update system balance snapshot                                      │
│                                                                              │
│  [13] Send notification to user                                            │
│       Type: "deposit_completed"                                             │
│       Message: "500 AP added to your wallet"                                │
│       Channel: Email + Push notification                                    │
│                                                                              │
└──────────────────────────────────────────────┬───────────────────────────────┘
                                               │
                                               ▼
                                             END

┌─────────────────────────────────────────────────────────────────────────────┐
│ SUB-FLOW CONNECTIONS                                                        │
├─────────────────────────────────────────────────────────────────────────────┤
│ → Called by: User request, Admin manual input                              │
│ → Calls: Transaction Service, Notification Service                         │
│ → Updates: wallets, wallet_transactions, deposit_requests                  │
│ → Triggers: money_flow_summary aggregation (async)                         │
│                                                                             │
│ VALIDATION POINTS:                                                          │
│ • Step 4: Admin verification (manual gate)                                 │
│ • Step 5: Transaction atomicity (MongoDB session)                          │
│ • Step 7: Wallet exists check                                              │
│ • Step 11: Commit or rollback all changes                                  │
│                                                                             │
│ ERROR HANDLING:                                                             │
│ • If transaction fails → Rollback all changes                              │
│ • If notification fails → Log error (don't block flow)                     │
│ • If duplicate detected → Reject with clear message                        │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### 4.4 Flow Type 2: PURCHASE (Internal Money Movement)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    PURCHASE FLOW - BUYER TO SELLER                          │
│ Type: INTERNAL | Category: PAYMENT | Actor: Buyer + System                 │
└─────────────────────────────────────────────────────────────────────────────┘

START: Buyer confirms order purchase
│
├─── PRE-VALIDATION ──────────────────────────────────────────────────────────┐
│                                                                             │
│  [1] Get order details                                                     │
│      - buyerId, sellerId, totalAp, orderType                               │
│                                                                             │
│  [2] Load order type configuration                                         │
│      Query: db.order_type_configs.findOne({ orderType })                   │
│      Extract:                                                               │
│      ┌───────────────────────────────────────┐                            │
│      │ - commission.type: "percentage"       │                            │
│      │ - commission.value: 5 (%)             │                            │
│      │ - escrow.holdDays: 3                  │                            │
│      │ - escrow.conditionalHolds: [...]      │                            │
│      └───────────────────────────────────────┘                            │
│                                                                             │
│  [3] Calculate commission                                                  │
│      commissionAp = ceil(totalAp × 5 / 100)                                │
│      sellerReceivesAp = totalAp - commissionAp                             │
│      Example: 100 AP → 5 AP commission, 95 AP to seller                    │
│                                                                             │
│  [4] Determine hold period                                                 │
│      ┌──────────────────────────────────────────┐                         │
│      │ FOR EACH conditional in conditionalHolds  │                         │
│      │   IF condition matches (amount, seller)   │                         │
│      │     holdDays = conditional.holdDays       │                         │
│      │     BREAK                                 │                         │
│      └──────────────────────────────────────────┘                         │
│      Default: 3 days, Can extend to 5-10 days                              │
│                                                                             │
│  [5] Check buyer balance                                                   │
│      buyerWallet = db.wallets.findOne({ userId: buyerId })                 │
│      ┌─────────────────────────────────┐                                   │
│      │ IF buyerWallet.balances.apCurrent < totalAp                         │
│      │   REJECT: "Insufficient balance"                                    │
│      │   END                                                                │
│      └─────────────────────────────────┘                                   │
│                                                                             │
└─────────────────────────────────────────────┬───────────────────────────────┘
                                              │
                                              ▼
┌─── SYSTEM PROCESSING (Atomic MongoDB Transaction) ──────────────────────────┐
│                                                                             │
│  [6] BEGIN TRANSACTION (session)                                           │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐  │
│  │ STEP 6A: DEDUCT FROM BUYER                                          │  │
│  ├─────────────────────────────────────────────────────────────────────┤  │
│  │                                                                      │  │
│  │  [6A.1] Create buyer transaction record                             │  │
│  │         type: "purchase"                                             │  │
│  │         amount.ap: -100 (negative = deduction)                       │  │
│  │         relatedTo.orderId: [Order ID]                                │  │
│  │         relatedTo.sellerId: [Seller ID]                              │  │
│  │         status: "completed"                                          │  │
│  │                                                                      │  │
│  │  [6A.2] Update buyer wallet                                          │  │
│  │         $inc: { "balances.apCurrent": -100 }                         │  │
│  │         $inc: { "lifetime.totalSpent": +100 }                        │  │
│  │                                                                      │  │
│  └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐  │
│  │ STEP 6B: ADD TO SELLER PENDING                                      │  │
│  ├─────────────────────────────────────────────────────────────────────┤  │
│  │                                                                      │  │
│  │  [6B.1] Create seller transaction record                            │  │
│  │         type: "sale"                                                 │  │
│  │         amount.ap: +95 (after commission)                            │  │
│  │         relatedTo.orderId: [Order ID]                                │  │
│  │         relatedTo.buyerId: [Buyer ID]                                │  │
│  │         description: "Sale pending 3-day release"                    │  │
│  │         status: "completed"                                          │  │
│  │                                                                      │  │
│  │  [6B.2] Update seller wallet                                         │  │
│  │         $inc: { "balances.apPendingCashout": +95 }                   │  │
│  │         $inc: { "lifetime.totalEarned": +95 }                        │  │
│  │                                                                      │  │
│  └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐  │
│  │ STEP 6C: CREATE ESCROW HOLD                                         │  │
│  ├─────────────────────────────────────────────────────────────────────┤  │
│  │                                                                      │  │
│  │  [6C.1] Calculate release timestamp                                  │  │
│  │         releaseAt = NOW() + holdDays                                 │  │
│  │         Example: 2025-01-29 + 3 days = 2025-02-01                    │  │
│  │                                                                      │  │
│  │  [6C.2] Create escrow_holds record                                   │  │
│  │         ┌─────────────────────────────────────┐                     │  │
│  │         │ escrowNumber: ESC-YYYYMMDD-XXXXX    │                     │  │
│  │         │ orderId: [Order ID]                  │                     │  │
│  │         │ sellerId: [Seller ID]                │                     │  │
│  │         │ buyerId: [Buyer ID]                  │                     │  │
│  │         │ amount.ap: 95                        │                     │  │
│  │         │ amount.originalAp: 100               │                     │  │
│  │         │ amount.commissionAp: 5               │                     │  │
│  │         │ holdConfig.holdDays: 3               │                     │  │
│  │         │ holdConfig.releaseAt: [Timestamp]    │                     │  │
│  │         │ status: "holding"                    │                     │  │
│  │         └─────────────────────────────────────┘                     │  │
│  │                                                                      │  │
│  └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐  │
│  │ STEP 6D: RECORD PLATFORM COMMISSION                                 │  │
│  ├─────────────────────────────────────────────────────────────────────┤  │
│  │                                                                      │  │
│  │  [6D.1] Create commission transaction                               │  │
│  │         type: "commission"                                           │  │
│  │         userId: null (system transaction)                            │  │
│  │         amount.ap: +5                                                │  │
│  │         relatedTo.orderId: [Order ID]                                │  │
│  │         description: "Platform 5% commission"                        │  │
│  │                                                                      │  │
│  └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  [7] Update order status                                                   │
│      db.orders.updateOne(                                                  │
│        { _id: orderId },                                                    │
│        {                                                                    │
│          status: "escrow",                                                 │
│          escrowId: [Escrow ID],                                            │
│          escrowReleaseAt: [Release Timestamp]                              │
│        }                                                                    │
│      )                                                                      │
│                                                                             │
│  [8] COMMIT TRANSACTION                                                    │
│      ✓ All or nothing - if any step fails, entire transaction rolls back  │
│                                                                             │
└─────────────────────────────────────────────┬───────────────────────────────┘
                                              │
                                              ▼
┌─── POST-PROCESSING ──────────────────────────────────────────────────────────┐
│                                                                              │
│  [9] Send notifications                                                     │
│      → Buyer: "Order purchased for 100 AP"                                  │
│      → Seller: "You earned 95 AP (pending 3-day release)"                   │
│                                                                              │
│  [10] Update daily summary (async)                                          │
│       money_flow_summary.internal.purchases += 100 AP                       │
│       money_flow_summary.commission += 5 AP                                 │
│                                                                              │
└──────────────────────────────────────────────┬───────────────────────────────┘
                                               │
                                               ▼
                                          ┌─────────┐
                                          │  DONE   │
                                          │         │
                                          │ Escrow  │
                                          │ Created │
                                          └────┬────┘
                                               │
                    ┌──────────────────────────┴───────────────────────────┐
                    │                                                      │
                    │ ESCROW WILL AUTO-RELEASE AFTER holdDays              │
                    │ See Flow 4.5: Auto-Release Escrow (Cron Job)         │
                    │                                                      │
                    └──────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│ SUB-FLOW CONNECTIONS                                                        │
├─────────────────────────────────────────────────────────────────────────────┤
│ → Called by: Order Service when buyer confirms purchase                    │
│ → Calls: Transaction Service, Escrow Service, Notification Service         │
│ → Updates: wallets (2), wallet_transactions (3), escrow_holds (1),         │
│            orders (1)                                                       │
│ → Triggers: Auto-release escrow flow (scheduled 3-5 days later)            │
│                                                                             │
│ CRITICAL POINTS:                                                            │
│ • Step 5: Balance check BEFORE transaction                                 │
│ • Step 6: ALL operations in single transaction                             │
│ • Step 6C: Escrow created immediately (not delayed)                         │
│ • Step 8: Atomic commit (all or nothing)                                   │
│                                                                             │
│ MONEY ACCOUNTING:                                                           │
│ • Buyer balance:    -100 AP Current                                         │
│ • Seller balance:   +95 AP Pending (not Current yet)                        │
│ • Platform:         +5 AP Commission                                        │
│ • Total:            -100 + 95 + 5 = 0 (balanced)                            │
│                                                                             │
│ ESCROW LIFECYCLE:                                                           │
│ 1. Created here (status: "holding")                                         │
│ 2. Auto-released by cron (Flow 4.5)                                         │
│ 3. OR manually released by admin                                            │
│ 4. OR refunded if disputed (Flow 4.7)                                       │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### 4.5 Flow Type 3: AUTO-RELEASE ESCROW (Scheduled)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│              AUTO-RELEASE ESCROW FLOW - CRON JOB                            │
│ Type: INTERNAL | Category: ESCROW | Actor: System (Automated)              │
│ Schedule: Every hour (0 * * * *)                                           │
└─────────────────────────────────────────────────────────────────────────────┘

START: Cron job triggered (hourly)
│
├─── DISCOVERY PHASE ──────────────────────────────────────────────────────────┐
│                                                                             │
│  [1] Query expired escrows                                                 │
│      ┌────────────────────────────────────────────────┐                    │
│      │ db.escrow_holds.find({                         │                    │
│      │   status: "holding",                           │                    │
│      │   "holdConfig.releaseAt": { $lte: NOW() },     │                    │
│      │   "dispute.hasDispute": false                  │                    │
│      │ })                                              │                    │
│      └────────────────────────────────────────────────┘                    │
│                                                                             │
│      Result: Array of escrow_holds ready for release                       │
│                                                                             │
│  [2] Log discovery                                                         │
│      Log: "Found {count} escrows ready for release"                        │
│                                                                             │
│  [3] Check if any found                                                    │
│      ┌────────────────────────────┐                                        │
│      │ IF count = 0               │                                        │
│      │   Log: "No escrows to release"                                      │
│      │   END                       │                                        │
│      └────────────────────────────┘                                        │
│                                                                             │
└─────────────────────────────────────────────┬───────────────────────────────┘
                                              │
                                              ▼
┌─── PROCESSING LOOP ──────────────────────────────────────────────────────────┐
│                                                                             │
│  FOR EACH escrow in expiredEscrows:                                        │
│  │                                                                          │
│  ├─── PROCESS SINGLE ESCROW ───────────────────────────────────────────┐  │
│  │                                                                      │  │
│  │  [4] Extract escrow details                                         │  │
│  │      - escrowId, orderId, sellerId, buyerId                          │  │
│  │      - amount.ap (to release)                                        │  │
│  │      - holdDays, releaseAt                                           │  │
│  │                                                                      │  │
│  │  [5] BEGIN TRANSACTION (per escrow)                                  │  │
│  │                                                                      │  │
│  │  ┌──────────────────────────────────────────────────────────────┐  │  │
│  │  │ STEP 5A: GET SELLER WALLET STATE                             │  │  │
│  │  ├──────────────────────────────────────────────────────────────┤  │  │
│  │  │                                                               │  │  │
│  │  │  sellerWallet = db.wallets.findOne(                           │  │  │
│  │  │    { userId: sellerId },                                      │  │  │
│  │  │    { session }                                                │  │  │
│  │  │  )                                                             │  │  │
│  │  │                                                               │  │  │
│  │  │  Capture current state:                                       │  │  │
│  │  │  - balances.apCurrent                                         │  │  │
│  │  │  - balances.apPendingCashout                                  │  │  │
│  │  │                                                               │  │  │
│  │  └──────────────────────────────────────────────────────────────┘  │  │
│  │                                                                      │  │
│  │  ┌──────────────────────────────────────────────────────────────┐  │  │
│  │  │ STEP 5B: CREATE RELEASE TRANSACTION                          │  │  │
│  │  ├──────────────────────────────────────────────────────────────┤  │  │
│  │  │                                                               │  │  │
│  │  │  Create wallet_transaction:                                   │  │  │
│  │  │  ┌────────────────────────────────────────┐                  │  │  │
│  │  │  │ transactionNumber: TXN-YYYYMMDD-XXXXX  │                  │  │  │
│  │  │  │ userId: sellerId                        │                  │  │  │
│  │  │  │ type: "release"                         │                  │  │  │
│  │  │  │ category: "payment"                     │                  │  │  │
│  │  │  │ amount.ap: 95                           │                  │  │  │
│  │  │  │ amount.direction: "internal"            │                  │  │  │
│  │  │  │ balanceSnapshot.before: {               │                  │  │  │
│  │  │  │   apCurrent: 2000,                      │                  │  │  │
│  │  │  │   apPendingCashout: 95                  │                  │  │  │
│  │  │  │ }                                        │                  │  │  │
│  │  │  │ balanceSnapshot.after: {                │                  │  │  │
│  │  │  │   apCurrent: 2095,                      │                  │  │  │
│  │  │  │   apPendingCashout: 0                   │                  │  │  │
│  │  │  │ }                                        │                  │  │  │
│  │  │  │ relatedTo.orderId: [Order ID]           │                  │  │  │
│  │  │  │ relatedTo.escrowId: [Escrow ID]         │                  │  │  │
│  │  │  │ status: "completed"                     │                  │  │  │
│  │  │  │ description: "Auto-release escrow"      │                  │  │  │
│  │  │  └────────────────────────────────────────┘                  │  │  │
│  │  │                                                               │  │  │
│  │  └──────────────────────────────────────────────────────────────┘  │  │
│  │                                                                      │  │
│  │  ┌──────────────────────────────────────────────────────────────┐  │  │
│  │  │ STEP 5C: MOVE MONEY IN SELLER WALLET                         │  │  │
│  │  ├──────────────────────────────────────────────────────────────┤  │  │
│  │  │                                                               │  │  │
│  │  │  db.wallets.updateOne(                                        │  │  │
│  │  │    { userId: sellerId },                                      │  │  │
│  │  │    {                                                           │  │  │
│  │  │      $inc: {                                                  │  │  │
│  │  │        "balances.apPendingCashout": -95,  ← Subtract          │  │  │
│  │  │        "balances.apCurrent": +95          ← Add               │  │  │
│  │  │      },                                                        │  │  │
│  │  │      $set: { updatedAt: NOW() }                               │  │  │
│  │  │    },                                                          │  │  │
│  │  │    { session }                                                │  │  │
│  │  │  )                                                             │  │  │
│  │  │                                                               │  │  │
│  │  │  Note: apTotal unchanged (internal move)                      │  │  │
│  │  │        lifetime stats unchanged (already counted at sale)     │  │  │
│  │  │                                                               │  │  │
│  │  └──────────────────────────────────────────────────────────────┘  │  │
│  │                                                                      │  │
│  │  ┌──────────────────────────────────────────────────────────────┐  │  │
│  │  │ STEP 5D: UPDATE ESCROW STATUS                                │  │  │
│  │  ├──────────────────────────────────────────────────────────────┤  │  │
│  │  │                                                               │  │  │
│  │  │  db.escrow_holds.updateOne(                                   │  │  │
│  │  │    { _id: escrowId },                                         │  │  │
│  │  │    {                                                           │  │  │
│  │  │      $set: {                                                  │  │  │
│  │  │        status: "released",                                    │  │  │
│  │  │        "resolution.resolvedAt": NOW(),                        │  │  │
│  │  │        "resolution.resolutionType": "auto_release",           │  │  │
│  │  │        "resolution.releasedAp": 95,                           │  │  │
│  │  │        "resolution.releaseTransactionId": [Txn ID],           │  │  │
│  │  │        releasedAt: NOW(),                                     │  │  │
│  │  │        updatedAt: NOW()                                       │  │  │
│  │  │      }                                                         │  │  │
│  │  │    },                                                          │  │  │
│  │  │    { session }                                                │  │  │
│  │  │  )                                                             │  │  │
│  │  │                                                               │  │  │
│  │  └──────────────────────────────────────────────────────────────┘  │  │
│  │                                                                      │  │
│  │  ┌──────────────────────────────────────────────────────────────┐  │  │
│  │  │ STEP 5E: UPDATE ORDER STATUS                                 │  │  │
│  │  ├──────────────────────────────────────────────────────────────┤  │  │
│  │  │                                                               │  │  │
│  │  │  db.orders.updateOne(                                         │  │  │
│  │  │    { _id: orderId },                                          │  │  │
│  │  │    {                                                           │  │  │
│  │  │      $set: {                                                  │  │  │
│  │  │        status: "completed",                                   │  │  │
│  │  │        completedAt: NOW(),                                    │  │  │
│  │  │        updatedAt: NOW()                                       │  │  │
│  │  │      }                                                         │  │  │
│  │  │    },                                                          │  │  │
│  │  │    { session }                                                │  │  │
│  │  │  )                                                             │  │  │
│  │  │                                                               │  │  │
│  │  └──────────────────────────────────────────────────────────────┘  │  │
│  │                                                                      │  │
│  │  [6] COMMIT TRANSACTION                                             │  │
│  │                                                                      │  │
│  │  [7] Log success                                                    │  │
│  │      Log: "Released escrow {escrowNumber}: {ap} AP to {sellerId}"  │  │
│  │                                                                      │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│  │                                                                          │
│  ├─── ERROR HANDLING ──────────────────────────────────────────────────┐  │
│  │                                                                      │  │
│  │  CATCH (error):                                                      │  │
│  │    - Log error with escrow details                                   │  │
│  │    - Rollback transaction (automatic)                                │  │
│  │    - Continue to next escrow (don't stop entire job)                 │  │
│  │    - Alert admin if critical                                         │  │
│  │                                                                      │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│  │                                                                          │
│  ├─── POST-PROCESSING (Outside Transaction) ───────────────────────────┐  │
│  │                                                                      │  │
│  │  [8] Send notification to seller                                     │  │
│  │      Type: "escrow_released"                                         │  │
│  │      Title: "Payment Released"                                       │  │
│  │      Message: "95 AP released to your available balance"             │  │
│  │      Channel: Email + Push                                           │  │
│  │                                                                      │  │
│  │      Note: If notification fails, log but don't rollback             │  │
│  │                                                                      │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  END FOR EACH                                                              │
│                                                                             │
└─────────────────────────────────────────────┬───────────────────────────────┘
                                              │
                                              ▼
┌─── COMPLETION ───────────────────────────────────────────────────────────────┐
│                                                                              │
│  [9] Update daily summary                                                   │
│      money_flow_summary.internal.escrowReleased += total released AP        │
│                                                                              │
│  [10] Log job completion                                                    │
│       Log: "Escrow release job completed: {successCount}/{totalCount}"      │
│                                                                              │
└──────────────────────────────────────────────┬───────────────────────────────┘
                                               │
                                               ▼
                                              END

┌─────────────────────────────────────────────────────────────────────────────┐
│ SUB-FLOW CONNECTIONS                                                        │
├─────────────────────────────────────────────────────────────────────────────┤
│ → Triggered by: Cron scheduler (every hour)                                │
│ → Called by: System scheduler (not user-initiated)                         │
│ → Calls: Transaction Service, Notification Service                         │
│ → Updates: wallets, wallet_transactions, escrow_holds, orders              │
│ → Connects to: Purchase flow (created the escrow)                          │
│               Withdrawal flow (seller can now withdraw)                    │
│                                                                             │
│ CRITICAL DESIGN DECISIONS:                                                  │
│ • One transaction per escrow (isolated failures)                           │
│ • Process all found escrows (don't stop on single failure)                 │
│ • Notification failures don't block release                                │
│ • Only release if NO active dispute                                        │
│                                                                             │
│ TIMING & SCHEDULE:                                                          │
│ • Runs: Every hour (0 * * * *)                                             │
│ • Query window: releaseAt <= NOW()                                         │
│ • Max delay: Up to 59 minutes after release time                           │
│ • Example: Release time 10:15 → Runs at 11:00                              │
│                                                                             │
│ BALANCE IMPACT:                                                             │
│ • Seller apPendingCashout: -95 AP                                           │
│ • Seller apCurrent: +95 AP                                                  │
│ • Seller apTotal: Unchanged (internal move)                                 │
│ • System total: Unchanged (no money in/out)                                 │
│                                                                             │
│ FOLLOW-ON FLOWS:                                                            │
│ • Seller can now withdraw this AP (Flow 4.6)                               │
│ • Order marked completed (visible in order history)                        │
│ • Statistics updated (seller earnings report)                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### 4.6 Flow Type 4: WITHDRAWAL (Money Outflow)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                  WITHDRAWAL FLOW - SELLER TO BANK                           │
│ Type: OUTFLOW | Category: WITHDRAWAL | Actor: Seller + Admin               │
└─────────────────────────────────────────────────────────────────────────────┘

START: Seller initiates withdrawal request
│
├─── USER REQUEST PHASE ───────────────────────────────────────────────────────┐
│                                                                             │
│  [1] Seller navigates to withdraw page                                     │
│      Displays:                                                              │
│      - AP Current (available): 2,000 AP                                     │
│      - AP Pending (escrow): 500 AP (not withdrawable)                       │
│      - Bank account on file                                                 │
│                                                                             │
│  [2] Seller enters withdrawal details                                      │
│      Input:                                                                 │
│      - Withdrawal amount: 1,000 AP                                          │
│      - Confirms bank account                                                │
│      - Optional note                                                        │
│                                                                             │
│  [3] PRE-VALIDATION                                                         │
│      ┌─────────────────────────────────────────────────┐                   │
│      │ CHECK 1: Amount > 0                             │                   │
│      │ CHECK 2: Amount <= apCurrent (not pending)      │                   │
│      │ CHECK 3: Bank account verified                  │                   │
│      │ CHECK 4: Within daily limit                     │                   │
│      │ CHECK 5: Not exceeded withdrawal count/day      │                   │
│      │ CHECK 6: Account not frozen                     │                   │
│      └─────────────────────────────────────────────────┘                   │
│           │                                                                 │
│           ├─── FAIL ──────────┐                                            │
│           │                    ▼                                            │
│           │          Return error message                                   │
│           │          END                                                    │
│           │                                                                 │
│           └─── PASS ──────────┐                                            │
│                                │                                            │
└────────────────────────────────┼────────────────────────────────────────────┘
                                 │
                                 ▼
┌─── IMMEDIATE WALLET DEDUCTION (Atomic Transaction) ─────────────────────────┐
│                                                                             │
│  [4] Calculate fees                                                        │
│      ┌────────────────────────────────────┐                                │
│      │ IF amount >= 500 AP                │                                │
│      │   feeAp = 0 (free withdrawal)      │                                │
│      │ ELSE                                │                                │
│      │   feeAp = 10 (fee for small amt)   │                                │
│      └────────────────────────────────────┘                                │
│      netAp = requestedAp - feeAp                                            │
│      vnd = netAp × 1000                                                     │
│                                                                             │
│  [5] BEGIN TRANSACTION                                                     │
│                                                                             │
│  [6] Get bank account snapshot                                             │
│      - Save current bank details to withdrawal record                      │
│      - Protects against user changing bank info after request              │
│                                                                             │
│  [7] Create withdrawal_requests record                                     │
│      ┌──────────────────────────────────────────┐                         │
│      │ withdrawalNumber: WTD-YYYYMMDD-XXXXX     │                         │
│      │ userId: sellerId                          │                         │
│      │ amount.requestedAp: 1000                  │                         │
│      │ amount.feeAp: 0                           │                         │
│      │ amount.netAp: 1000                        │                         │
│      │ amount.vnd: 1,000,000                     │                         │
│      │ method: "bank"                            │                         │
│      │ destination: {                            │                         │
│      │   bankName: "Vietcombank",                │                         │
│      │   accountNumber: "1234567890",            │                         │
│      │   accountName: "NGUYEN VAN A"             │                         │
│      │ }                                          │                         │
│      │ status: "pending"                         │                         │
│      └──────────────────────────────────────────┘                         │
│                                                                             │
│  [8] Create deduction transaction                                          │
│      ┌──────────────────────────────────────────┐                         │
│      │ type: "withdraw"                          │                         │
│      │ amount.ap: -1000                          │                         │
│      │ amount.direction: "debit"                 │                         │
│      │ balanceSnapshot.before: {                 │                         │
│      │   apCurrent: 2000                         │                         │
│      │ }                                          │                         │
│      │ balanceSnapshot.after: {                  │                         │
│      │   apCurrent: 1000                         │                         │
│      │ }                                          │                         │
│      │ relatedTo.withdrawalId: [WTD ID]          │                         │
│      │ status: "completed"                       │                         │
│      │ description: "Withdrawal request WTD-..." │                         │
│      └──────────────────────────────────────────┘                         │
│                                                                             │
│  [9] Deduct from seller wallet                                             │
│      db.wallets.updateOne(                                                 │
│        { userId: sellerId },                                                │
│        {                                                                    │
│          $inc: {                                                            │
│            "balances.apCurrent": -1000,                                    │
│            "lifetime.totalWithdrawn": +1000                                │
│          }                                                                  │
│        },                                                                   │
│        { session }                                                          │
│      )                                                                      │
│                                                                             │
│      CRITICAL: Money deducted IMMEDIATELY, not after approval               │
│      Reason: Prevents double-spending while admin processes                 │
│                                                                             │
│  [10] Link transaction to withdrawal                                       │
│       withdrawal_requests.deductTransactionId = [Transaction ID]           │
│                                                                             │
│  [11] COMMIT TRANSACTION                                                   │
│                                                                             │
└─────────────────────────────────────────────┬───────────────────────────────┘
                                              │
                                              ▼
┌─── POST-REQUEST ─────────────────────────────────────────────────────────────┐
│                                                                              │
│  [12] Notify admin team                                                     │
│       Type: "new_withdrawal_request"                                        │
│       Message: "New withdrawal: 1,000 AP (1,000,000 VND)"                   │
│       Channel: Admin dashboard + Email                                      │
│                                                                              │
│  [13] Notify user                                                           │
│       Message: "Withdrawal request submitted. Processing in 1-24 hours."    │
│                                                                              │
└─────────────────────────────────────────────┬────────────────────────────────┘
                                              │
                            ┌─────────────────┴─────────────────┐
                            │                                   │
                            │  WAITING FOR ADMIN PROCESSING...  │
                            │  Can take 1-24 hours              │
                            │                                   │
                            └─────────────────┬─────────────────┘
                                              │
                    ┌─────────────────────────┴──────────────────────────┐
                    │                                                    │
                    ▼                                                    ▼
      ┌─────────────────────────┐                         ┌──────────────────────┐
      │   ADMIN APPROVES        │                         │   ADMIN REJECTS      │
      │   (See Branch A)        │                         │   (See Branch B)     │
      └─────────────────────────┘                         └──────────────────────┘


┌─── BRANCH A: ADMIN APPROVAL ─────────────────────────────────────────────────┐
│                                                                              │
│  [A1] Admin views pending withdrawals queue                                 │
│       Dashboard shows:                                                       │
│       - Withdrawal details                                                   │
│       - User history                                                         │
│       - Seller reputation                                                    │
│       - Flagged risks                                                        │
│                                                                              │
│  [A2] Admin reviews and verifies                                            │
│       ✓ Bank account name matches user                                      │
│       ✓ No suspicious activity                                              │
│       ✓ User account in good standing                                       │
│                                                                              │
│  [A3] Admin transfers money (MANUAL STEP)                                   │
│       - Admin uses internet banking                                          │
│       - Transfers 1,000,000 VND to seller's bank                             │
│       - Takes screenshot of transfer confirmation                            │
│                                                                              │
│  [A4] Admin marks as completed                                              │
│       Input:                                                                 │
│       - Gateway reference (bank transaction ID)                              │
│       - Upload proof screenshot                                              │
│       - Click "Approve & Complete"                                           │
│                                                                              │
│  [A5] Update withdrawal status                                              │
│       db.withdrawal_requests.updateOne(                                     │
│         { _id: withdrawalId },                                               │
│         {                                                                    │
│           $set: {                                                            │
│             status: "completed",                                             │
│             "processing.completedBy": adminId,                               │
│             "processing.completedAt": NOW(),                                 │
│             "processing.gatewayReference": "VCB-TXN-123456",                 │
│             "processing.proofUrl": "https://storage/proof.jpg"               │
│           }                                                                  │
│         }                                                                    │
│       )                                                                      │
│                                                                              │
│  [A6] Notify seller                                                         │
│       Type: "withdrawal_completed"                                           │
│       Message: "1,000,000 VND transferred to your Vietcombank account"       │
│       Include: Bank reference number                                         │
│                                                                              │
│  [A7] Update daily summary                                                  │
│       money_flow_summary.outflows.withdrawals += 1000 AP                    │
│                                                                              │
│  END: Withdrawal complete, money sent                                       │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘


┌─── BRANCH B: ADMIN REJECTION ────────────────────────────────────────────────┐
│                                                                              │
│  [B1] Admin reviews and decides to reject                                   │
│       Reasons:                                                               │
│       - Bank account mismatch                                                │
│       - Suspicious activity detected                                         │
│       - Insufficient documentation                                           │
│       - Compliance issues                                                    │
│                                                                              │
│  [B2] BEGIN REFUND TRANSACTION                                              │
│                                                                              │
│  [B3] Get current wallet state                                              │
│       sellerWallet = db.wallets.findOne({ userId: sellerId })               │
│                                                                              │
│  [B4] Create refund transaction                                             │
│       ┌──────────────────────────────────────────┐                         │
│       │ type: "refund"                            │                         │
│       │ category: "withdrawal"                    │                         │
│       │ amount.ap: +1000 (returning)              │                         │
│       │ amount.direction: "credit"                │                         │
│       │ relatedTo.withdrawalId: [WTD ID]          │                         │
│       │ status: "completed"                       │                         │
│       │ description: "Refund rejected withdrawal" │                         │
│       │ adminAction.adminId: [Admin ID]           │                         │
│       └──────────────────────────────────────────┘                         │
│                                                                              │
│  [B5] Refund to wallet                                                      │
│       db.wallets.updateOne(                                                 │
│         { userId: sellerId },                                                │
│         {                                                                    │
│           $inc: {                                                            │
│             "balances.apCurrent": +1000,                                    │
│             "lifetime.totalWithdrawn": -1000  ← Reverse                     │
│           }                                                                  │
│         },                                                                   │
│         { session }                                                          │
│       )                                                                      │
│                                                                              │
│  [B6] Update withdrawal status                                              │
│       db.withdrawal_requests.updateOne(                                     │
│         { _id: withdrawalId },                                               │
│         {                                                                    │
│           $set: {                                                            │
│             status: "rejected",                                              │
│             "processing.rejectedBy": adminId,                                │
│             "processing.rejectedAt": NOW(),                                  │
│             "processing.rejectReason": "Bank account mismatch",              │
│             refundTransactionId: [Refund Transaction ID]                    │
│           }                                                                  │
│         }                                                                    │
│       )                                                                      │
│                                                                              │
│  [B7] COMMIT REFUND TRANSACTION                                             │
│                                                                              │
│  [B8] Notify seller                                                         │
│       Type: "withdrawal_rejected"                                            │
│       Message: "Withdrawal rejected: Bank account mismatch.                  │
│                 1,000 AP refunded to your wallet."                           │
│       Include: Reject reason, next steps                                     │
│                                                                              │
│  END: Withdrawal rejected, money refunded                                   │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│ SUB-FLOW CONNECTIONS                                                        │
├─────────────────────────────────────────────────────────────────────────────┤
│ → Called by: Seller user action                                            │
│ → Calls: Transaction Service, Notification Service, Admin Service          │
│ → Updates: wallets, wallet_transactions, withdrawal_requests               │
│ → Prerequisites: Escrow released (Flow 4.5) to have AP Current             │
│                                                                             │
│ KEY DESIGN DECISION: IMMEDIATE DEDUCTION                                    │
│ • Money deducted from wallet at request time (Step 9)                      │
│ • NOT after admin approval                                                  │
│ • Prevents: User spending same AP while request pending                    │
│ • Trade-off: If rejected, must refund (extra transaction)                  │
│ • Alternative: Hold money in wallet but mark "reserved" (more complex)     │
│                                                                             │
│ TWO OUTCOMES:                                                               │
│ 1. APPROVED:                                                                │
│    - Real money sent to bank                                                │
│    - AP already deducted (no further wallet change)                         │
│    - Status: completed                                                      │
│                                                                             │
│ 2. REJECTED:                                                                │
│    - Real money NOT sent                                                    │
│    - AP refunded to wallet (reverse deduction)                              │
│    - Status: rejected                                                       │
│    - User can try again with corrected info                                 │
│                                                                             │
│ MONEY ACCOUNTING (Approved):                                                │
│ • Seller wallet: -1,000 AP Current (at request)                             │
│ • System outflow: -1,000,000 VND (real money out)                           │
│ • Platform balance: -1,000,000 VND (real money)                             │
│                                                                             │
│ MONEY ACCOUNTING (Rejected):                                                │
│ • Seller wallet: -1,000 AP (at request), +1,000 AP (at refund) = 0         │
│ • System outflow: 0 (no real money moved)                                   │
│                                                                             │
│ SECURITY & COMPLIANCE:                                                      │
│ • Bank account name must match user KYC                                     │
│ • Large amounts may require additional verification                         │
│ • Daily limits prevent rapid drain                                          │
│ • Admin approval creates audit trail                                        │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### 4.7 Flow Type 5: P2P TRANSFER (User to User)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    P2P TRANSFER FLOW - USER TO USER                         │
│ Type: INTERNAL | Category: TRANSFER | Actor: User A + User B               │
└─────────────────────────────────────────────────────────────────────────────┘

START: User A wants to send AP to User B
│
├─── USER ACTION & VALIDATION ─────────────────────────────────────────────────┐
│                                                                             │
│  [1] User A initiates transfer                                             │
│      Input:                                                                 │
│      - Recipient identifier (email, username, ID)                           │
│      - Amount: 50 AP                                                        │
│      - Optional message: "Payment for service"                              │
│                                                                             │
│  [2] PRE-VALIDATION                                                         │
│      ┌─────────────────────────────────────────────────┐                   │
│      │ CHECK 1: Amount > 0                             │                   │
│      │ CHECK 2: Amount >= minimum (e.g., 1 AP)         │                   │
│      │ CHECK 3: User A has sufficient balance          │                   │
│      │ CHECK 4: Within daily transfer limit            │                   │
│      │ CHECK 5: Recipient exists                       │                   │
│      │ CHECK 6: Recipient != Self                      │                   │
│      │ CHECK 7: Recipient account active               │                   │
│      │ CHECK 8: Sender account not frozen              │                   │
│      └─────────────────────────────────────────────────┘                   │
│           │                                                                 │
│           ├─── FAIL ──────────┐                                            │
│           │                    ▼                                            │
│           │          Return specific error                                  │
│           │          END                                                    │
│           │                                                                 │
│           └─── PASS ──────────┐                                            │
│                                │                                            │
└────────────────────────────────┼────────────────────────────────────────────┘
                                 │
                                 ▼
┌─── ATOMIC TRANSFER (Single MongoDB Transaction) ────────────────────────────┐
│                                                                             │
│  CRITICAL: Both sender deduction and receiver credit must be atomic        │
│            Either both succeed or both fail                                 │
│                                                                             │
│  [3] BEGIN TRANSACTION                                                     │
│                                                                             │
│  [4] Get both wallet states                                                │
│      senderWallet = db.wallets.findOne(                                    │
│        { userId: userAId },                                                 │
│        { session }                                                          │
│      )                                                                      │
│                                                                             │
│      receiverWallet = db.wallets.findOne(                                  │
│        { userId: userBId },                                                 │
│        { session }                                                          │
│      )                                                                      │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐ │
│  │ STEP 4A: DEDUCT FROM SENDER (User A)                                 │ │
│  ├──────────────────────────────────────────────────────────────────────┤ │
│  │                                                                       │ │
│  │  [4A.1] Create sender transaction                                    │ │
│  │         ┌────────────────────────────────────────┐                  │ │
│  │         │ transactionNumber: TXN-YYYYMMDD-XXXXX  │                  │ │
│  │         │ userId: userAId                         │                  │ │
│  │         │ type: "transfer_send"                   │                  │ │
│  │         │ category: "transfer"                    │                  │ │
│  │         │ amount.ap: -50                          │                  │ │
│  │         │ amount.direction: "debit"               │                  │ │
│  │         │ balanceSnapshot.before: {               │                  │ │
│  │         │   apCurrent: 1000                       │                  │ │
│  │         │ }                                        │                  │ │
│  │         │ balanceSnapshot.after: {                │                  │ │
│  │         │   apCurrent: 950                        │                  │ │
│  │         │ }                                        │                  │ │
│  │         │ relatedTo.transferToUserId: userBId     │                  │ │
│  │         │ description: "Transfer 50 AP to User B" │                  │ │
│  │         │ status: "completed"                     │                  │ │
│  │         └────────────────────────────────────────┘                  │ │
│  │                                                                       │ │
│  │  [4A.2] Update sender wallet                                         │ │
│  │         db.wallets.updateOne(                                        │ │
│  │           { userId: userAId },                                       │ │
│  │           {                                                           │ │
│  │             $inc: {                                                  │ │
│  │               "balances.apCurrent": -50,                            │ │
│  │               "lifetime.totalSent": +50                             │ │
│  │             },                                                        │ │
│  │             $set: { updatedAt: NOW() }                               │ │
│  │           },                                                          │ │
│  │           { session }                                                │ │
│  │         )                                                             │ │
│  │                                                                       │ │
│  └──────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐ │
│  │ STEP 4B: CREDIT TO RECEIVER (User B)                                 │ │
│  ├──────────────────────────────────────────────────────────────────────┤ │
│  │                                                                       │ │
│  │  [4B.1] Create receiver transaction                                  │ │
│  │         ┌────────────────────────────────────────┐                  │ │
│  │         │ transactionNumber: TXN-YYYYMMDD-XXXXX  │  ← Different #   │ │
│  │         │ userId: userBId                         │                  │ │
│  │         │ type: "transfer_receive"                │                  │ │
│  │         │ category: "transfer"                    │                  │ │
│  │         │ amount.ap: +50                          │                  │ │
│  │         │ amount.direction: "credit"              │                  │ │
│  │         │ balanceSnapshot.before: {               │                  │ │
│  │         │   apCurrent: 500                        │                  │ │
│  │         │ }                                        │                  │ │
│  │         │ balanceSnapshot.after: {                │                  │ │
│  │         │   apCurrent: 550                        │                  │ │
│  │         │ }                                        │                  │ │
│  │         │ relatedTo.transferFromUserId: userAId   │                  │ │
│  │         │ description: "Received 50 AP from A"    │                  │ │
│  │         │ status: "completed"                     │                  │ │
│  │         └────────────────────────────────────────┘                  │ │
│  │                                                                       │ │
│  │  [4B.2] Update receiver wallet                                       │ │
│  │         db.wallets.updateOne(                                        │ │
│  │           { userId: userBId },                                       │ │
│  │           {                                                           │ │
│  │             $inc: {                                                  │ │
│  │               "balances.apCurrent": +50,                            │ │
│  │               "lifetime.totalReceived": +50                         │ │
│  │             },                                                        │ │
│  │             $set: { updatedAt: NOW() }                               │ │
│  │           },                                                          │ │
│  │           { session }                                                │ │
│  │         )                                                             │ │
│  │                                                                       │ │
│  └──────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
│  [5] COMMIT TRANSACTION                                                    │
│      ✓ Both wallets updated atomically                                     │
│      ✓ Two transaction records created                                     │
│      ✓ If any step fails, entire operation rolls back                      │
│                                                                             │
└─────────────────────────────────────────────┬───────────────────────────────┘
                                              │
                                              ▼
┌─── POST-PROCESSING ──────────────────────────────────────────────────────────┐
│                                                                              │
│  [6] Send notifications                                                     │
│      → User A (Sender):                                                     │
│        Type: "transfer_sent"                                                │
│        Message: "You sent 50 AP to User B"                                  │
│                                                                              │
│      → User B (Receiver):                                                   │
│        Type: "transfer_received"                                             │
│        Message: "You received 50 AP from User A"                            │
│        Include: Sender's message if provided                                │
│                                                                              │
│  [7] Update daily summary (async)                                           │
│      money_flow_summary.inflows.transfers += 50 AP                          │
│      money_flow_summary.outflows.transfers += 50 AP                         │
│      (Note: Internal transfer, so inflow = outflow)                         │
│                                                                              │
│  [8] Log transfer for analytics                                             │
│      - P2P transfer volume                                                  │
│      - User behavior patterns                                               │
│      - Fraud detection                                                      │
│                                                                              │
└──────────────────────────────────────────────┬───────────────────────────────┘
                                               │
                                               ▼
                                              END

┌─────────────────────────────────────────────────────────────────────────────┐
│ SUB-FLOW CONNECTIONS                                                        │
├─────────────────────────────────────────────────────────────────────────────┤
│ → Called by: User action (transfer button/API)                             │
│ → Calls: Transaction Service, Notification Service                         │
│ → Updates: wallets (2), wallet_transactions (2)                            │
│ → No external systems involved (purely internal)                           │
│                                                                             │
│ CRITICAL ATOMICITY REQUIREMENTS:                                            │
│ • Must use MongoDB transaction (session)                                   │
│ • Sender deduction and receiver credit are ONE operation                   │
│ • Cannot have "money lost" (deducted but not credited)                     │
│ • Cannot have "money created" (credited but not deducted)                  │
│ • If transaction fails, both wallets unchanged                             │
│                                                                             │
│ MONEY ACCOUNTING:                                                           │
│ • User A: -50 AP Current                                                    │
│ • User B: +50 AP Current                                                    │
│ • System total: 0 change (conservation of AP)                               │
│ • No commission/fee taken (direct transfer)                                 │
│                                                                             │
│ TWO TRANSACTION RECORDS:                                                    │
│ • Why two? Each user sees transfer in their own transaction history        │
│ • Linked via relatedTo.transferToUserId and transferFromUserId             │
│ • Both have same timestamp                                                  │
│ • Different transaction numbers (independent audit trail)                  │
│                                                                             │
│ USE CASES:                                                                  │
│ • Paying another user for services                                         │
│ • Gifting AP to friend/family                                              │
│ • Splitting costs among users                                              │
│ • Reimbursement scenarios                                                  │
│                                                                             │
│ ANTI-FRAUD MEASURES:                                                        │
│ • Daily transfer limits per user                                           │
│ • Cannot transfer to self                                                  │
│ • Recipient must be verified/active                                        │
│ • Large transfers may trigger review                                       │
│ • Pattern detection for suspicious chains                                  │
│                                                                             │
│ EXTENSIBILITY:                                                              │
│ • Future: Add transfer fees (% or fixed)                                   │
│ • Future: Transfer messages/notes                                          │
│ • Future: Recurring transfers                                              │
│ • Future: Escrow transfers (hold until condition)                          │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### 4.8 Flow Type 6: REFUND (Dispute Resolution)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                REFUND FLOW - ESCROW TO BUYER (DISPUTE)                      │
│ Type: INTERNAL | Category: REFUND | Actor: Admin (via Dispute System)      │
└─────────────────────────────────────────────────────────────────────────────┘

START: Admin resolves dispute in favor of buyer
│
├─── DISPUTE CONTEXT ──────────────────────────────────────────────────────────┐
│                                                                             │
│  Background:                                                                │
│  [1] Buyer purchased product → Payment in escrow (Flow 4.4)                │
│  [2] Buyer received product with issues                                     │
│  [3] Buyer opened dispute within escrow period                              │
│  [4] Admin investigated and decided: REFUND BUYER                           │
│                                                                             │
│  Current state:                                                             │
│  - Escrow holds 95 AP (seller's pending amount)                             │
│  - Escrow status: "disputed"                                                │
│  - Dispute status: "resolved" → refund                                      │
│                                                                             │
└─────────────────────────────────────────────┬───────────────────────────────┘
                                              │
                                              ▼
┌─── REFUND DECISION TYPES ────────────────────────────────────────────────────┐
│                                                                             │
│  Admin chooses refund type:                                                 │
│                                                                             │
│  ┌─────────────────────┐         ┌─────────────────────┐                   │
│  │  FULL REFUND        │         │  PARTIAL REFUND     │                   │
│  │  100% to buyer      │         │  Split resolution   │                   │
│  └─────────┬───────────┘         └─────────┬───────────┘                   │
│            │                               │                               │
│            │                               │                               │
│            └───────────┬───────────────────┘                               │
│                        │                                                    │
└────────────────────────┼────────────────────────────────────────────────────┘
                         │
                         ▼
┌─── FULL REFUND PROCESSING (Atomic Transaction) ─────────────────────────────┐
│                                                                             │
│  Scenario: Buyer gets 100% refund (95 AP from escrow + 5 AP commission)    │
│                                                                             │
│  [1] Get current states                                                    │
│      escrow = db.escrow_holds.findOne({ orderId })                         │
│      - escrow.amount.ap = 95 (in seller pending)                            │
│      - escrow.amount.commissionAp = 5 (platform took)                       │
│      - escrow.amount.originalAp = 100 (buyer originally paid)               │
│                                                                             │
│  [2] BEGIN TRANSACTION                                                     │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐ │
│  │ STEP 2A: DEDUCT FROM SELLER PENDING                                  │ │
│  ├──────────────────────────────────────────────────────────────────────┤ │
│  │                                                                       │ │
│  │  [2A.1] Create seller deduction transaction                          │ │
│  │         type: "refund"                                               │ │
│  │         amount.ap: -95                                               │ │
│  │         amount.direction: "debit"                                    │ │
│  │         relatedTo.orderId, buyerId, escrowId                         │ │
│  │         description: "Refund to buyer (dispute resolved)"            │ │
│  │                                                                       │ │
│  │  [2A.2] Update seller wallet                                         │ │
│  │         db.wallets.updateOne(                                        │ │
│  │           { userId: sellerId },                                      │ │
│  │           {                                                           │ │
│  │             $inc: {                                                  │ │
│  │               "balances.apPendingCashout": -95,                     │ │
│  │               "lifetime.totalEarned": -95  ← Reverse earning        │ │
│  │             }                                                         │ │
│  │           },                                                          │ │
│  │           { session }                                                │ │
│  │         )                                                             │ │
│  │                                                                       │ │
│  └──────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐ │
│  │ STEP 2B: REFUND TO BUYER (Full amount including commission)          │ │
│  ├──────────────────────────────────────────────────────────────────────┤ │
│  │                                                                       │ │
│  │  Decision: Should buyer get commission back?                         │ │
│  │  → YES: Refund full 100 AP (95 from escrow + 5 from platform)        │ │
│  │  → NO:  Refund only 95 AP (platform keeps commission)                │ │
│  │                                                                       │ │
│  │  Assuming FULL REFUND policy:                                        │ │
│  │                                                                       │ │
│  │  [2B.1] Create buyer credit transaction                              │ │
│  │         type: "refund"                                               │ │
│  │         amount.ap: +100  ← Full original payment                     │ │
│  │         amount.direction: "credit"                                   │ │
│  │         relatedTo.orderId, sellerId, escrowId                        │ │
│  │         description: "Refund for order (dispute resolved)"           │ │
│  │                                                                       │ │
│  │  [2B.2] Update buyer wallet                                          │ │
│  │         db.wallets.updateOne(                                        │ │
│  │           { userId: buyerId },                                       │ │
│  │           {                                                           │ │
│  │             $inc: {                                                  │ │
│  │               "balances.apCurrent": +100,                           │ │
│  │               "lifetime.totalSpent": -100  ← Reverse spending       │ │
│  │             }                                                         │ │
│  │           },                                                          │ │
│  │           { session }                                                │ │
│  │         )                                                             │ │
│  │                                                                       │ │
│  └──────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐ │
│  │ STEP 2C: REVERSE PLATFORM COMMISSION                                 │ │
│  ├──────────────────────────────────────────────────────────────────────┤ │
│  │                                                                       │ │
│  │  [2C.1] Create commission reversal transaction                       │ │
│  │         type: "commission_reversal"                                  │ │
│  │         userId: null (system)                                        │ │
│  │         amount.ap: -5  ← Negative (reversing)                        │ │
│  │         relatedTo.orderId                                            │ │
│  │         description: "Reverse commission (refund)"                   │ │
│  │                                                                       │ │
│  └──────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐ │
│  │ STEP 2D: UPDATE ESCROW STATUS                                        │ │
│  ├──────────────────────────────────────────────────────────────────────┤ │
│  │                                                                       │ │
│  │  db.escrow_holds.updateOne(                                          │ │
│  │    { _id: escrowId },                                                │ │
│  │    {                                                                  │ │
│  │      $set: {                                                         │ │
│  │        status: "refunded",                                           │ │
│  │        "resolution.resolvedAt": NOW(),                               │ │
│  │        "resolution.resolvedBy": adminId,                             │ │
│  │        "resolution.resolutionType": "full_refund",                   │ │
│  │        "resolution.refundedAp": 100,                                 │ │
│  │        "resolution.refundTransactionId": [Txn ID],                   │ │
│  │        "dispute.disputeStatus": "resolved",                          │ │
│  │        updatedAt: NOW()                                              │ │
│  │      }                                                                │ │
│  │    },                                                                 │ │
│  │    { session }                                                       │ │
│  │  )                                                                    │ │
│  │                                                                       │ │
│  └──────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐ │
│  │ STEP 2E: UPDATE ORDER STATUS                                         │ │
│  ├──────────────────────────────────────────────────────────────────────┤ │
│  │                                                                       │ │
│  │  db.orders.updateOne(                                                │ │
│  │    { _id: orderId },                                                 │ │
│  │    {                                                                  │ │
│  │      $set: {                                                         │ │
│  │        status: "refunded",                                           │ │
│  │        refundedAt: NOW(),                                            │ │
│  │        refundReason: "Dispute resolved in favor of buyer"            │ │
│  │      }                                                                │ │
│  │    },                                                                 │ │
│  │    { session }                                                       │ │
│  │  )                                                                    │ │
│  │                                                                       │ │
│  └──────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
│  [3] COMMIT TRANSACTION                                                    │
│                                                                             │
└─────────────────────────────────────────────┬───────────────────────────────┘
                                              │
                                              ▼
┌─── POST-PROCESSING ──────────────────────────────────────────────────────────┐
│                                                                              │
│  [4] Send notifications                                                     │
│      → Buyer:                                                               │
│        Type: "refund_completed"                                              │
│        Message: "100 AP refunded to your wallet for order #XXX"              │
│                                                                              │
│      → Seller:                                                               │
│        Type: "sale_refunded"                                                 │
│        Message: "Order #XXX refunded due to dispute resolution"              │
│        Impact: -95 AP from pending balance                                   │
│                                                                              │
│  [5] Update dispute record                                                  │
│      db.disputes.updateOne(                                                 │
│        { orderId },                                                          │
│        {                                                                     │
│          status: "resolved",                                                 │
│          resolution: "buyer_refund",                                         │
│          resolvedBy: adminId,                                                │
│          resolvedAt: NOW()                                                   │
│        }                                                                     │
│      )                                                                       │
│                                                                              │
│  [6] Update daily summary                                                   │
│      money_flow_summary.internal.refunds += 100 AP                          │
│                                                                              │
└──────────────────────────────────────────────┬───────────────────────────────┘
                                               │
                                               ▼
                                              END

┌─────────────────────────────────────────────────────────────────────────────┐
│ SUB-FLOW CONNECTIONS                                                        │
├─────────────────────────────────────────────────────────────────────────────┤
│ → Called by: Dispute resolution system                                     │
│ → Prerequisites: Escrow created (Flow 4.4), Dispute opened                 │
│ → Calls: Transaction Service, Notification Service, Dispute Service        │
│ → Updates: wallets (2), wallet_transactions (3), escrow_holds (1),         │
│            orders (1), disputes (1)                                         │
│                                                                             │
│ MONEY ACCOUNTING (Full Refund):                                             │
│ • Seller pending: -95 AP                                                    │
│ • Buyer current: +100 AP                                                    │
│ • Platform commission: -5 AP (reversed)                                     │
│ • Total: -95 + 100 - 5 = 0 (balanced)                                       │
│                                                                             │
│ PARTIAL REFUND VARIANT:                                                     │
│ • If admin decides 50/50 split:                                             │
│   - Buyer gets: +50 AP                                                      │
│   - Seller keeps: 45 AP (released to current)                               │
│   - Platform: keeps 5 AP commission                                         │
│ • Implementation: Adjust amounts in steps 2A and 2B                         │
│                                                                             │
│ CRITICAL TIMING:                                                            │
│ • Must happen BEFORE escrow auto-release                                    │
│ • Auto-release cron checks dispute.hasDispute flag                          │
│ • Disputed escrows are skipped by auto-release                              │
│ • Only processed via manual dispute resolution                              │
│                                                                             │
│ COMMISSION REFUND POLICY:                                                   │
│ • Decision point: Refund commission to buyer?                              │
│ • Option A: Full refund (100 AP) - buyer friendly                           │
│ • Option B: Keep commission (95 AP) - platform keeps commission             │
│ • Recommended: Option A (better customer experience)                        │
│                                                                             │
│ FOLLOW-ON EFFECTS:                                                          │
│ • Seller reputation may be affected                                         │
│ • Order marked as "refunded" in history                                     │
│ • Buyer can place new orders                                                │
│ • Dispute closed and archived                                               │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### 4.9 Flow Type 7: ADMIN MANUAL ADJUSTMENT

```
┌─────────────────────────────────────────────────────────────────────────────┐
│             ADMIN MANUAL ADJUSTMENT FLOW - BALANCE CORRECTION               │
│ Type: ADMIN | Category: ADJUSTMENT | Actor: Admin                          │
└─────────────────────────────────────────────────────────────────────────────┘

START: Admin needs to manually adjust user balance
│
├─── USE CASES ────────────────────────────────────────────────────────────────┐
│                                                                             │
│  When admin manual adjustment is needed:                                   │
│  • Correction: System error caused incorrect balance                       │
│  • Compensation: User experienced service issues                           │
│  • Promotional credit: Marketing campaign                                  │
│  • Penalty deduction: Terms violation                                      │
│  • Migration adjustment: V1 to V2 balance sync                             │
│  • Dispute resolution: Manual correction after investigation               │
│                                                                             │
└─────────────────────────────────────────────┬───────────────────────────────┘
                                              │
                                              ▼
┌─── ADMIN ACTION & VALIDATION ────────────────────────────────────────────────┐
│                                                                             │
│  [1] Admin navigates to user wallet management                             │
│      Path: Admin Panel → Users → [Select User] → Wallet                    │
│                                                                             │
│  [2] View current wallet state                                             │
│      Display:                                                               │
│      - AP Current: 1,000                                                    │
│      - AP Pending: 500                                                      │
│      - Recent transactions                                                  │
│      - Lifetime stats                                                       │
│                                                                             │
│  [3] Click "Adjust Balance"                                                │
│                                                                             │
│  [4] Fill adjustment form                                                  │
│      ┌──────────────────────────────────────────────┐                      │
│      │ Adjustment Type:                             │                      │
│      │ ○ Add AP                                     │                      │
│      │ ● Deduct AP                                  │                      │
│      │                                              │                      │
│      │ Amount: [____100____] AP                     │                      │
│      │                                              │                      │
│      │ Reason (required):                           │                      │
│      │ [System error correction - duplicate        │                      │
│      │  withdrawal processed on 2025-01-28]        │                      │
│      │                                              │                      │
│      │ Notify user: ☑ Yes ☐ No                     │                      │
│      │                                              │                      │
│      │ [Cancel] [Submit Adjustment]                 │                      │
│      └──────────────────────────────────────────────┘                      │
│                                                                             │
│  [5] PRE-VALIDATION                                                         │
│      ┌─────────────────────────────────────────────────┐                   │
│      │ CHECK 1: Amount > 0                             │                   │
│      │ CHECK 2: Reason provided (min 10 characters)    │                   │
│      │ CHECK 3: If deduction, amount <= apCurrent      │                   │
│      │ CHECK 4: Admin has permission                   │                   │
│      │ CHECK 5: User account exists & active           │                   │
│      └─────────────────────────────────────────────────┘                   │
│           │                                                                 │
│           ├─── FAIL ──────────┐                                            │
│           │                    ▼                                            │
│           │          Show validation error                                  │
│           │          END                                                    │
│           │                                                                 │
│           └─── PASS ──────────┐                                            │
│                                │                                            │
└────────────────────────────────┼────────────────────────────────────────────┘
                                 │
                                 ▼
┌─── SECURITY CONFIRMATION ────────────────────────────────────────────────────┐
│                                                                             │
│  [6] Require admin password confirmation                                   │
│      ┌──────────────────────────────────────────────┐                      │
│      │ CONFIRM BALANCE ADJUSTMENT                   │                      │
│      │                                              │                      │
│      │ ⚠️  You are about to DEDUCT 100 AP           │                      │
│      │     from user: buyer@example.com             │                      │
│      │                                              │                      │
│      │ Current balance: 1,000 AP                    │                      │
│      │ After adjustment: 900 AP                     │                      │
│      │                                              │                      │
│      │ This action cannot be undone.                │                      │
│      │                                              │                      │
│      │ Enter your admin password to confirm:        │                      │
│      │ [________________________]                   │                      │
│      │                                              │                      │
│      │ [Cancel] [Confirm Adjustment]                │                      │
│      └──────────────────────────────────────────────┘                      │
│                                                                             │
│  [7] Verify admin password                                                 │
│      ┌────────────────────────────┐                                        │
│      │ IF password incorrect       │                                        │
│      │   Reject: "Invalid password"                                        │
│      │   Log failed attempt                                                │
│      │   END                        │                                        │
│      └────────────────────────────┘                                        │
│                                                                             │
└─────────────────────────────────────────────┬───────────────────────────────┘
                                              │
                                              ▼
┌─── SYSTEM PROCESSING (MongoDB Transaction) ──────────────────────────────────┐
│                                                                             │
│  [8] BEGIN TRANSACTION                                                     │
│                                                                             │
│  [9] Get current wallet state                                              │
│      wallet = db.wallets.findOne({ userId }, { session })                  │
│      Capture: balances.apCurrent (for snapshot)                             │
│                                                                             │
│  [10] Create wallet_transaction record                                     │
│       ┌────────────────────────────────────────────┐                       │
│       │ transactionNumber: TXN-YYYYMMDD-XXXXX      │                       │
│       │ userId: [User ObjectId]                     │                       │
│       │ type: "adjustment"                          │                       │
│       │ category: "adjustment"                      │                       │
│       │ amount.ap: -100  ← Negative for deduction   │                       │
│       │ amount.vnd: -100000                         │                       │
│       │ amount.direction: "debit"                   │                       │
│       │ balanceSnapshot.before: {                   │                       │
│       │   apCurrent: 1000                           │                       │
│       │ }                                            │                       │
│       │ balanceSnapshot.after: {                    │                       │
│       │   apCurrent: 900                            │                       │
│       │ }                                            │                       │
│       │ source.type: "admin_manual"                 │                       │
│       │ status: "completed"                         │                       │
│       │ description: "Admin adjustment: -100 AP"    │                       │
│       │ notes: "System error correction..."         │                       │
│       │ adminAction: {                              │                       │
│       │   adminId: [Admin ObjectId],                │                       │
│       │   adminName: "admin@example.com",           │                       │
│       │   actionType: "balance_deduction",          │                       │
│       │   reason: "System error correction...",     │                       │
│       │   confirmedWithPassword: true,              │                       │
│       │   approvedBy: [Admin ObjectId],             │                       │
│       │   approvedAt: NOW()                         │                       │
│       │ }                                            │                       │
│       │ createdAt: NOW()                            │                       │
│       │ completedAt: NOW()                          │                       │
│       └────────────────────────────────────────────┘                       │
│                                                                             │
│  [11] Update wallet balance                                                │
│       db.wallets.updateOne(                                                │
│         { userId },                                                         │
│         {                                                                   │
│           $inc: {                                                           │
│             "balances.apCurrent": -100                                     │
│             // Note: NOT updating lifetime stats for adjustments           │
│             // Adjustments are corrections, not user actions               │
│           },                                                                │
│           $set: { updatedAt: NOW() }                                        │
│         },                                                                  │
│         { session }                                                         │
│       )                                                                     │
│                                                                             │
│  [12] COMMIT TRANSACTION                                                   │
│                                                                             │
└─────────────────────────────────────────────┬───────────────────────────────┘
                                              │
                                              ▼
┌─── POST-PROCESSING & AUDIT ──────────────────────────────────────────────────┐
│                                                                              │
│  [13] Create detailed audit log entry                                       │
│       db.admin_action_logs.insertOne({                                      │
│         actionType: "wallet_adjustment",                                    │
│         adminId: [Admin ObjectId],                                          │
│         adminEmail: "admin@example.com",                                    │
│         adminIp: "192.168.1.1",                                             │
│         targetUserId: [User ObjectId],                                      │
│         targetUserEmail: "buyer@example.com",                               │
│         changes: {                                                           │
│           field: "balances.apCurrent",                                      │
│           oldValue: 1000,                                                   │
│           newValue: 900,                                                    │
│           delta: -100                                                       │
│         },                                                                   │
│         reason: "System error correction - duplicate withdrawal...",        │
│         transactionId: [Transaction ObjectId],                              │
│         approvalRequired: true,                                             │
│         approvedBy: [Admin ObjectId],                                       │
│         approvedAt: NOW(),                                                  │
│         timestamp: NOW()                                                    │
│       })                                                                     │
│                                                                              │
│  [14] Alert senior admin (if large amount)                                  │
│       IF abs(amount) > 1000 AP:                                             │
│         Send alert to senior admins                                         │
│         Message: "Large wallet adjustment: -100 AP by admin@example.com"    │
│                                                                              │
│  [15] Send notification to user (if checkbox selected)                      │
│       Type: "wallet_adjusted"                                               │
│       Message: "Your wallet balance was adjusted: -100 AP                   │
│                 Reason: System error correction...                          │
│                 Contact support if you have questions."                     │
│                                                                              │
│  [16] Update admin dashboard statistics                                     │
│       - Total adjustments today                                             │
│       - Total AP adjusted (positive/negative)                               │
│                                                                              │
└──────────────────────────────────────────────┬───────────────────────────────┘
                                               │
                                               ▼
                                              END

┌─────────────────────────────────────────────────────────────────────────────┐
│ SUB-FLOW CONNECTIONS                                                        │
├─────────────────────────────────────────────────────────────────────────────┤
│ → Called by: Admin user action                                             │
│ → Calls: Transaction Service, Audit Service, Notification Service          │
│ → Updates: wallets, wallet_transactions, admin_action_logs                 │
│ → Triggers: Alerts for large adjustments                                   │
│                                                                             │
│ SECURITY MEASURES:                                                          │
│ • Password confirmation required                                           │
│ • Reason is mandatory (min 10 characters)                                  │
│ • Full audit trail with admin details                                      │
│ • IP address logging                                                        │
│ • Alerts for large amounts                                                  │
│ • Failed attempt logging                                                    │
│                                                                             │
│ TWO-DIRECTION OPERATIONS:                                                   │
│ • ADD AP: amount.ap = positive (+100)                                       │
│   - Use case: Compensation, promotion                                       │
│   - $inc: { "balances.apCurrent": +100 }                                   │
│                                                                             │
│ • DEDUCT AP: amount.ap = negative (-100)                                    │
│   - Use case: Correction, penalty                                           │
│   - $inc: { "balances.apCurrent": -100 }                                   │
│   - Must check: current balance >= deduction amount                        │
│                                                                             │
│ MAKER-CHECKER EXTENSION (Future):                                           │
│ • Step 6: Create adjustment request (status: pending)                      │
│ • Separate admin approves request                                          │
│ • Only then execute Steps 8-12                                              │
│ • For amounts > threshold (e.g., 10,000 AP)                                 │
│                                                                             │
│ IMPORTANT POLICY DECISION:                                                  │
│ • Adjustments do NOT affect lifetime stats                                  │
│ • Reason: They are corrections, not user activity                          │
│ • Alternative: Could update stats if considered "real" transactions        │
│ • Recommended: Keep separate for clarity                                    │
│                                                                             │
│ MONEY ACCOUNTING:                                                           │
│ • User wallet: -100 AP Current                                              │
│ • System balance: Depends on adjustment reason                              │
│   - If correction: No net system change (was wrong before)                 │
│   - If penalty: System gains +100 AP (user loses)                           │
│   - If compensation: System loses -100 AP (user gains)                      │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 5. Transaction Flows

### 4.1 Manual Deposit by Admin

**Scenario**: User transfers money to bank but auto-detect fails. Admin manually credits wallet.

```javascript
// STEP 1: Admin creates deposit request
const depositRequest = {
  depositNumber: "DEP-20250129-00001",
  userId: ObjectId("user123"),
  userEmail: "buyer@example.com",
  amount: {
    vnd: 500000,
    ap: 500,
    exchangeRate: 1000,
    feeVnd: 0,
    feeAp: 0
  },
  method: "manual",
  source: {
    type: "bank_transfer",
    bankName: "Vietcombank",
    proofUrl: "https://storage.example.com/proof-user123.jpg",
    metadata: {
      userProvidedInfo: "Transferred via Vietcombank app at 10:30 AM"
    }
  },
  status: "verified",
  verification: {
    verifiedBy: ObjectId("admin456"),
    verifiedAt: new Date(),
    verificationNote: "Bank statement verified, credit approved"
  },
  createdAt: new Date(),
  updatedAt: new Date()
};

await db.deposit_requests.insertOne(depositRequest);

// STEP 2: Get current wallet balance
const wallet = await db.wallets.findOne({ userId: ObjectId("user123") });
const balanceBefore = {
  apCurrent: wallet.balances.apCurrent,
  apPendingCashout: wallet.balances.apPendingCashout,
  apTotal: wallet.balances.apTotal
};

// STEP 3: Create wallet transaction
const transaction = {
  transactionNumber: "TXN-20250129-00001",
  userId: ObjectId("user123"),
  userType: "buyer",
  type: "deposit",
  category: "deposit",
  amount: {
    ap: 500,
    vnd: 500000,
    direction: "credit"
  },
  balanceSnapshot: {
    before: balanceBefore,
    after: {
      apCurrent: balanceBefore.apCurrent + 500,
      apPendingCashout: balanceBefore.apPendingCashout,
      apTotal: balanceBefore.apTotal + 500
    }
  },
  source: {
    type: "manual",
    reference: depositRequest.depositNumber,
    metadata: {
      depositRequestId: depositRequest._id,
      adminId: ObjectId("admin456"),
      reason: "Manual deposit - bank transfer verified"
    }
  },
  status: "completed",
  description: "Manual deposit via Vietcombank",
  adminAction: {
    adminId: ObjectId("admin456"),
    adminName: "admin@example.com",
    actionType: "manual_deposit",
    approvedBy: ObjectId("admin456"),
    approvedAt: new Date()
  },
  createdAt: new Date(),
  completedAt: new Date()
};

// STEP 4: Execute in transaction
const session = client.startSession();
try {
  await session.withTransaction(async () => {
    // Insert transaction record
    await db.wallet_transactions.insertOne(transaction, { session });

    // Update wallet
    await db.wallets.updateOne(
      { userId: ObjectId("user123") },
      {
        $inc: {
          "balances.apCurrent": 500,
          "lifetime.totalDeposited": 500
        },
        $set: { updatedAt: new Date() }
      },
      { session }
    );

    // Mark deposit request as completed
    await db.deposit_requests.updateOne(
      { _id: depositRequest._id },
      {
        $set: {
          status: "completed",
          creditTransactionId: transaction._id,
          completedAt: new Date()
        }
      },
      { session }
    );
  });
} finally {
  await session.endSession();
}

// STEP 5: Send notification to user
await sendNotification(ObjectId("user123"), {
  type: "deposit_completed",
  title: "Deposit Successful",
  message: "500 AP has been added to your wallet",
  amount: 500
});
```

**Result:**
- User wallet: +500 AP Current
- Transaction record created
- Deposit request marked completed
- User receives notification

---

### 4.2 User Purchases Product (With Escrow)

**Scenario**: Buyer purchases 100 AP product from Seller. Platform takes 5% commission. Seller receives 95 AP to pending balance with 3-day hold.

```javascript
const order = {
  _id: ObjectId("order789"),
  buyerId: ObjectId("buyer123"),
  sellerId: ObjectId("seller456"),
  totalAp: 100,
  orderType: "digital_goods"
};

// Get order type config
const orderConfig = await db.order_type_configs.findOne({
  orderType: "digital_goods"
});

// Calculate commission
const commissionAp = Math.max(
  Math.ceil(order.totalAp * orderConfig.commission.value / 100),
  orderConfig.commission.minAp
);
const sellerReceivesAp = order.totalAp - commissionAp;

// Calculate hold period
let holdDays = orderConfig.escrow.holdDays;
// Check conditional holds (e.g., if order > 1000 AP, hold longer)
for (const condition of orderConfig.escrow.conditionalHolds) {
  if (condition.condition === "orderAmount" &&
      condition.operator === ">" &&
      order.totalAp > condition.value) {
    holdDays = condition.holdDays;
    break;
  }
}

const session = client.startSession();
try {
  await session.withTransaction(async () => {

    // STEP 1: Deduct from buyer wallet
    const buyerWallet = await db.wallets.findOne(
      { userId: order.buyerId },
      { session }
    );

    const buyerTransaction = {
      transactionNumber: generateTxnNumber(),
      userId: order.buyerId,
      userType: "buyer",
      type: "purchase",
      category: "payment",
      amount: {
        ap: -100,
        vnd: -100000,
        direction: "debit"
      },
      balanceSnapshot: {
        before: {
          apCurrent: buyerWallet.balances.apCurrent,
          apPendingCashout: buyerWallet.balances.apPendingCashout,
          apTotal: buyerWallet.balances.apTotal
        },
        after: {
          apCurrent: buyerWallet.balances.apCurrent - 100,
          apPendingCashout: buyerWallet.balances.apPendingCashout,
          apTotal: buyerWallet.balances.apTotal - 100
        }
      },
      relatedTo: {
        orderId: order._id,
        sellerId: order.sellerId
      },
      status: "completed",
      description: `Purchase order #${order._id}`,
      createdAt: new Date(),
      completedAt: new Date()
    };

    await db.wallet_transactions.insertOne(buyerTransaction, { session });
    await db.wallets.updateOne(
      { userId: order.buyerId },
      {
        $inc: {
          "balances.apCurrent": -100,
          "lifetime.totalSpent": 100
        },
        $set: { updatedAt: new Date() }
      },
      { session }
    );

    // STEP 2: Add to seller pending balance (after commission)
    const sellerWallet = await db.wallets.findOne(
      { userId: order.sellerId },
      { session }
    );

    const sellerTransaction = {
      transactionNumber: generateTxnNumber(),
      userId: order.sellerId,
      userType: "seller",
      type: "sale",
      category: "payment",
      amount: {
        ap: sellerReceivesAp,
        vnd: sellerReceivesAp * 1000,
        direction: "credit"
      },
      balanceSnapshot: {
        before: {
          apCurrent: sellerWallet.balances.apCurrent,
          apPendingCashout: sellerWallet.balances.apPendingCashout,
          apTotal: sellerWallet.balances.apTotal
        },
        after: {
          apCurrent: sellerWallet.balances.apCurrent,
          apPendingCashout: sellerWallet.balances.apPendingCashout + sellerReceivesAp,
          apTotal: sellerWallet.balances.apTotal + sellerReceivesAp
        }
      },
      relatedTo: {
        orderId: order._id,
        buyerId: order.buyerId
      },
      status: "completed",
      description: `Sale of order #${order._id} (pending ${holdDays}-day release)`,
      createdAt: new Date(),
      completedAt: new Date()
    };

    await db.wallet_transactions.insertOne(sellerTransaction, { session });
    await db.wallets.updateOne(
      { userId: order.sellerId },
      {
        $inc: {
          "balances.apPendingCashout": sellerReceivesAp,
          "lifetime.totalEarned": sellerReceivesAp
        },
        $set: { updatedAt: new Date() }
      },
      { session }
    );

    // STEP 3: Create escrow hold
    const releaseAt = new Date();
    releaseAt.setDate(releaseAt.getDate() + holdDays);

    const escrowHold = {
      escrowNumber: generateEscrowNumber(),
      orderId: order._id,
      sellerId: order.sellerId,
      buyerId: order.buyerId,
      amount: {
        ap: sellerReceivesAp,
        originalAp: order.totalAp,
        commissionAp: commissionAp,
        vnd: sellerReceivesAp * 1000
      },
      holdConfig: {
        orderType: order.orderType,
        holdDays: holdDays,
        releaseAt: releaseAt,
        releaseCondition: "auto",
        allowEarlyRelease: orderConfig.escrow.allowEarlyRelease,
        earlyReleaseRequested: false
      },
      status: "holding",
      resolution: {
        resolvedAt: null,
        resolvedBy: null,
        resolutionType: null,
        refundedAp: 0,
        releasedAp: 0,
        releaseTransactionId: null,
        refundTransactionId: null
      },
      dispute: {
        hasDispute: false,
        disputeId: null,
        disputeStatus: null
      },
      createdAt: new Date(),
      updatedAt: new Date()
    };

    await db.escrow_holds.insertOne(escrowHold, { session });

    // STEP 4: Record platform commission
    const commissionTransaction = {
      transactionNumber: generateTxnNumber(),
      userId: null,  // System transaction
      userType: null,
      type: "commission",
      category: "commission",
      amount: {
        ap: commissionAp,
        vnd: commissionAp * 1000,
        direction: "credit"
      },
      relatedTo: {
        orderId: order._id,
        sellerId: order.sellerId,
        buyerId: order.buyerId
      },
      status: "completed",
      description: `Platform commission ${orderConfig.commission.value}% from order #${order._id}`,
      createdAt: new Date(),
      completedAt: new Date()
    };

    await db.wallet_transactions.insertOne(commissionTransaction, { session });

    // STEP 5: Update order status
    await db.orders.updateOne(
      { _id: order._id },
      {
        $set: {
          status: "escrow",
          escrowId: escrowHold._id,
          escrowReleaseAt: releaseAt,
          updatedAt: new Date()
        }
      },
      { session }
    );
  });
} finally {
  await session.endSession();
}

// Send notifications
await sendNotification(order.buyerId, {
  type: "purchase_completed",
  message: `Order #${order._id} purchased for 100 AP`
});

await sendNotification(order.sellerId, {
  type: "sale_pending",
  message: `You earned ${sellerReceivesAp} AP (pending ${holdDays}-day verification)`
});
```

**Result:**
- Buyer: -100 AP Current
- Seller: +95 AP Pending Cashout
- Platform: +5 AP Commission
- Escrow created with 3-day hold
- Order status: escrow

---

### 4.3 Auto-Release Escrow (Cron Job)

**Scenario**: Cron job runs every hour to release expired escrows.

```javascript
// Cron job: runs every hour
async function autoReleaseEscrows() {
  const now = new Date();

  // Find all escrows ready for release
  const expiredEscrows = await db.escrow_holds.find({
    status: "holding",
    "holdConfig.releaseAt": { $lte: now },
    "dispute.hasDispute": false
  }).toArray();

  console.log(`Found ${expiredEscrows.length} escrows to release`);

  for (const escrow of expiredEscrows) {
    const session = client.startSession();
    try {
      await session.withTransaction(async () => {

        // STEP 1: Get seller wallet
        const sellerWallet = await db.wallets.findOne(
          { userId: escrow.sellerId },
          { session }
        );

        // STEP 2: Create release transaction
        const releaseTransaction = {
          transactionNumber: generateTxnNumber(),
          userId: escrow.sellerId,
          userType: "seller",
          type: "release",
          category: "payment",
          amount: {
            ap: escrow.amount.ap,
            vnd: escrow.amount.vnd,
            direction: "internal"  // Moving within same wallet
          },
          balanceSnapshot: {
            before: {
              apCurrent: sellerWallet.balances.apCurrent,
              apPendingCashout: sellerWallet.balances.apPendingCashout,
              apTotal: sellerWallet.balances.apTotal
            },
            after: {
              apCurrent: sellerWallet.balances.apCurrent + escrow.amount.ap,
              apPendingCashout: sellerWallet.balances.apPendingCashout - escrow.amount.ap,
              apTotal: sellerWallet.balances.apTotal  // Total unchanged
            }
          },
          relatedTo: {
            orderId: escrow.orderId,
            escrowId: escrow._id,
            buyerId: escrow.buyerId
          },
          status: "completed",
          description: `Auto-release escrow #${escrow.escrowNumber}`,
          createdAt: now,
          completedAt: now
        };

        await db.wallet_transactions.insertOne(releaseTransaction, { session });

        // STEP 3: Move from pending to current in seller wallet
        await db.wallets.updateOne(
          { userId: escrow.sellerId },
          {
            $inc: {
              "balances.apPendingCashout": -escrow.amount.ap,
              "balances.apCurrent": escrow.amount.ap
            },
            $set: { updatedAt: now }
          },
          { session }
        );

        // STEP 4: Update escrow status
        await db.escrow_holds.updateOne(
          { _id: escrow._id },
          {
            $set: {
              status: "released",
              "resolution.resolvedAt": now,
              "resolution.resolutionType": "auto_release",
              "resolution.releasedAp": escrow.amount.ap,
              "resolution.releaseTransactionId": releaseTransaction._id,
              releasedAt: now,
              updatedAt: now
            }
          },
          { session }
        );

        // STEP 5: Update order status
        await db.orders.updateOne(
          { _id: escrow.orderId },
          {
            $set: {
              status: "completed",
              completedAt: now,
              updatedAt: now
            }
          },
          { session }
        );

        console.log(`Released escrow ${escrow.escrowNumber}: ${escrow.amount.ap} AP to seller ${escrow.sellerId}`);
      });

      // STEP 6: Send notification (outside transaction)
      await sendNotification(escrow.sellerId, {
        type: "escrow_released",
        title: "Payment Released",
        message: `${escrow.amount.ap} AP has been released to your available balance`,
        amount: escrow.amount.ap,
        orderId: escrow.orderId
      });

    } catch (error) {
      console.error(`Failed to release escrow ${escrow.escrowNumber}:`, error);
      // Log error for admin review
    } finally {
      await session.endSession();
    }
  }
}

// Schedule to run every hour
cron.schedule('0 * * * *', autoReleaseEscrows);
```

**Result:**
- Seller: AP moves from Pending → Current
- Escrow status: released
- Order status: completed
- Seller receives notification

---

### 4.4 P2P Transfer (User to User)

**Scenario**: User A sends 50 AP to User B.

```javascript
async function transferAP(fromUserId, toUserId, amount, message) {
  // Validate
  if (amount <= 0) throw new Error("Amount must be positive");

  const fromWallet = await db.wallets.findOne({ userId: fromUserId });
  if (fromWallet.balances.apCurrent < amount) {
    throw new Error("Insufficient balance");
  }

  const toWallet = await db.wallets.findOne({ userId: toUserId });
  if (!toWallet) throw new Error("Recipient wallet not found");

  const session = client.startSession();
  try {
    await session.withTransaction(async () => {

      // STEP 1: Deduct from sender
      const senderTransaction = {
        transactionNumber: generateTxnNumber(),
        userId: fromUserId,
        type: "transfer_send",
        category: "transfer",
        amount: {
          ap: -amount,
          vnd: -amount * 1000,
          direction: "debit"
        },
        balanceSnapshot: {
          before: {
            apCurrent: fromWallet.balances.apCurrent,
            apPendingCashout: fromWallet.balances.apPendingCashout,
            apTotal: fromWallet.balances.apTotal
          },
          after: {
            apCurrent: fromWallet.balances.apCurrent - amount,
            apPendingCashout: fromWallet.balances.apPendingCashout,
            apTotal: fromWallet.balances.apTotal - amount
          }
        },
        relatedTo: {
          transferToUserId: toUserId
        },
        status: "completed",
        description: message || `Transfer ${amount} AP to user`,
        createdAt: new Date(),
        completedAt: new Date()
      };

      await db.wallet_transactions.insertOne(senderTransaction, { session });
      await db.wallets.updateOne(
        { userId: fromUserId },
        {
          $inc: {
            "balances.apCurrent": -amount,
            "lifetime.totalSent": amount
          },
          $set: { updatedAt: new Date() }
        },
        { session }
      );

      // STEP 2: Credit to receiver
      const receiverTransaction = {
        transactionNumber: generateTxnNumber(),
        userId: toUserId,
        type: "transfer_receive",
        category: "transfer",
        amount: {
          ap: amount,
          vnd: amount * 1000,
          direction: "credit"
        },
        balanceSnapshot: {
          before: {
            apCurrent: toWallet.balances.apCurrent,
            apPendingCashout: toWallet.balances.apPendingCashout,
            apTotal: toWallet.balances.apTotal
          },
          after: {
            apCurrent: toWallet.balances.apCurrent + amount,
            apPendingCashout: toWallet.balances.apPendingCashout,
            apTotal: toWallet.balances.apTotal + amount
          }
        },
        relatedTo: {
          transferFromUserId: fromUserId
        },
        status: "completed",
        description: message || `Received ${amount} AP from user`,
        createdAt: new Date(),
        completedAt: new Date()
      };

      await db.wallet_transactions.insertOne(receiverTransaction, { session });
      await db.wallets.updateOne(
        { userId: toUserId },
        {
          $inc: {
            "balances.apCurrent": amount,
            "lifetime.totalReceived": amount
          },
          $set: { updatedAt: new Date() }
        },
        { session }
      );
    });
  } finally {
    await session.endSession();
  }

  // Send notifications
  await sendNotification(toUserId, {
    type: "transfer_received",
    message: `You received ${amount} AP`,
    amount: amount
  });
}
```

**Result:**
- Sender: -50 AP Current
- Receiver: +50 AP Current
- Two linked transactions created
- Both users receive notifications

---

### 4.5 Withdrawal Request & Processing

**Scenario**: Seller requests to withdraw 1,000 AP to bank account.

```javascript
// STEP 1: User creates withdrawal request
async function createWithdrawal(userId, requestedAp, destination) {
  const wallet = await db.wallets.findOne({ userId });

  // Validate
  if (wallet.balances.apCurrent < requestedAp) {
    throw new Error("Insufficient available balance");
  }

  // Calculate fee (example: free if >= 500 AP, else 10 AP fee)
  const feeAp = requestedAp >= 500 ? 0 : 10;
  const netAp = requestedAp - feeAp;

  const session = client.startSession();
  let withdrawalRequest;

  try {
    await session.withTransaction(async () => {

      // Create withdrawal request
      withdrawalRequest = {
        withdrawalNumber: generateWithdrawalNumber(),
        userId: userId,
        userEmail: wallet.userEmail,
        amount: {
          requestedAp: requestedAp,
          feeAp: feeAp,
          netAp: netAp,
          vnd: netAp * 1000,
          exchangeRate: 1000
        },
        method: destination.method,
        destination: {
          ...destination,
          snapshotAt: new Date()
        },
        status: "pending",
        processing: {
          assignedTo: null,
          verified: false
        },
        createdAt: new Date(),
        updatedAt: new Date()
      };

      const insertResult = await db.withdrawal_requests.insertOne(
        withdrawalRequest,
        { session }
      );
      withdrawalRequest._id = insertResult.insertedId;

      // Immediately deduct from wallet (hold)
      const deductTransaction = {
        transactionNumber: generateTxnNumber(),
        userId: userId,
        type: "withdraw",
        category: "withdrawal",
        amount: {
          ap: -requestedAp,
          vnd: -requestedAp * 1000,
          direction: "debit"
        },
        balanceSnapshot: {
          before: {
            apCurrent: wallet.balances.apCurrent,
            apPendingCashout: wallet.balances.apPendingCashout,
            apTotal: wallet.balances.apTotal
          },
          after: {
            apCurrent: wallet.balances.apCurrent - requestedAp,
            apPendingCashout: wallet.balances.apPendingCashout,
            apTotal: wallet.balances.apTotal - requestedAp
          }
        },
        relatedTo: {
          withdrawalId: withdrawalRequest._id
        },
        status: "completed",  // Deduction is immediate
        description: `Withdrawal request #${withdrawalRequest.withdrawalNumber}`,
        createdAt: new Date(),
        completedAt: new Date()
      };

      await db.wallet_transactions.insertOne(deductTransaction, { session });
      await db.wallets.updateOne(
        { userId: userId },
        {
          $inc: {
            "balances.apCurrent": -requestedAp,
            "lifetime.totalWithdrawn": requestedAp
          },
          $set: { updatedAt: new Date() }
        },
        { session }
      );

      // Link transaction to withdrawal
      await db.withdrawal_requests.updateOne(
        { _id: withdrawalRequest._id },
        { $set: { deductTransactionId: deductTransaction._id } },
        { session }
      );
    });
  } finally {
    await session.endSession();
  }

  // Notify admin
  await notifyAdmin({
    type: "new_withdrawal_request",
    withdrawalId: withdrawalRequest._id,
    amount: requestedAp
  });

  return withdrawalRequest;
}

// STEP 2: Admin approves withdrawal
async function approveWithdrawal(withdrawalId, adminId, gatewayReference, proofUrl) {
  await db.withdrawal_requests.updateOne(
    { _id: withdrawalId },
    {
      $set: {
        status: "completed",
        "processing.completedBy": adminId,
        "processing.completedAt": new Date(),
        "processing.gatewayReference": gatewayReference,
        "processing.proofUrl": proofUrl,
        updatedAt: new Date()
      }
    }
  );

  const withdrawal = await db.withdrawal_requests.findOne({ _id: withdrawalId });

  // Send notification
  await sendNotification(withdrawal.userId, {
    type: "withdrawal_completed",
    title: "Withdrawal Completed",
    message: `${withdrawal.amount.vnd.toLocaleString()} VND has been transferred to your bank account`,
    amount: withdrawal.amount.requestedAp
  });
}

// STEP 3: Admin rejects withdrawal (refund to wallet)
async function rejectWithdrawal(withdrawalId, adminId, rejectReason) {
  const withdrawal = await db.withdrawal_requests.findOne({ _id: withdrawalId });

  const session = client.startSession();
  try {
    await session.withTransaction(async () => {

      // Create refund transaction
      const wallet = await db.wallets.findOne({ userId: withdrawal.userId }, { session });

      const refundTransaction = {
        transactionNumber: generateTxnNumber(),
        userId: withdrawal.userId,
        type: "refund",
        category: "withdrawal",
        amount: {
          ap: withdrawal.amount.requestedAp,
          vnd: withdrawal.amount.requestedAp * 1000,
          direction: "credit"
        },
        balanceSnapshot: {
          before: {
            apCurrent: wallet.balances.apCurrent,
            apPendingCashout: wallet.balances.apPendingCashout,
            apTotal: wallet.balances.apTotal
          },
          after: {
            apCurrent: wallet.balances.apCurrent + withdrawal.amount.requestedAp,
            apPendingCashout: wallet.balances.apPendingCashout,
            apTotal: wallet.balances.apTotal + withdrawal.amount.requestedAp
          }
        },
        relatedTo: {
          withdrawalId: withdrawal._id
        },
        status: "completed",
        description: `Refund rejected withdrawal #${withdrawal.withdrawalNumber}`,
        adminAction: {
          adminId: adminId,
          actionType: "withdrawal_rejection_refund"
        },
        createdAt: new Date(),
        completedAt: new Date()
      };

      await db.wallet_transactions.insertOne(refundTransaction, { session });

      // Refund to wallet
      await db.wallets.updateOne(
        { userId: withdrawal.userId },
        {
          $inc: {
            "balances.apCurrent": withdrawal.amount.requestedAp,
            "lifetime.totalWithdrawn": -withdrawal.amount.requestedAp  // Reverse
          },
          $set: { updatedAt: new Date() }
        },
        { session }
      );

      // Update withdrawal status
      await db.withdrawal_requests.updateOne(
        { _id: withdrawalId },
        {
          $set: {
            status: "rejected",
            "processing.rejectedBy": adminId,
            "processing.rejectedAt": new Date(),
            "processing.rejectReason": rejectReason,
            refundTransactionId: refundTransaction._id,
            updatedAt: new Date()
          }
        },
        { session }
      );
    });
  } finally {
    await session.endSession();
  }

  // Notify user
  await sendNotification(withdrawal.userId, {
    type: "withdrawal_rejected",
    title: "Withdrawal Rejected",
    message: `Your withdrawal request has been rejected: ${rejectReason}. Amount refunded to wallet.`,
    amount: withdrawal.amount.requestedAp
  });
}
```

**Result (Approved):**
- User wallet: Already deducted when requested
- Real money sent to user's bank
- Withdrawal status: completed

**Result (Rejected):**
- User wallet: Refunded
- Withdrawal status: rejected
- User notified with reason

---

## 5. Implementation Plan

### Phase 1: Core Wallet Infrastructure (Week 1-2)
- [ ] Create MongoDB collections with indexes
- [ ] Implement Wallet Service (CRUD operations)
- [ ] Implement Transaction Service (create, query, generate numbers)
- [ ] Build basic API endpoints (balance, transaction history)
- [ ] Write unit tests

### Phase 2: Deposit System (Week 3)
- [ ] Admin manual deposit functionality
- [ ] Deposit request creation and verification
- [ ] Upload proof/receipt system
- [ ] Deposit approval workflow
- [ ] Deposit reports and admin queue

### Phase 3: Purchase & Escrow System (Week 4-5)
- [ ] Order type configuration management
- [ ] Purchase flow integration
- [ ] Escrow creation with variable hold periods
- [ ] Conditional hold logic implementation
- [ ] Auto-release cron job
- [ ] Manual release by admin
- [ ] Refund system (full/partial)

### Phase 4: Withdrawal System (Week 6)
- [ ] Withdrawal request creation
- [ ] Immediate wallet deduction
- [ ] Admin processing queue
- [ ] Approval with proof upload
- [ ] Rejection with auto-refund
- [ ] Withdrawal limits and fees

### Phase 5: P2P Transfer (Week 7)
- [ ] Transfer API endpoint
- [ ] Balance validation
- [ ] Paired transaction creation
- [ ] Transfer limits and fees
- [ ] Transfer history

### Phase 6: Money Flow Tracking & Reports (Week 8)
- [ ] Daily summary aggregation cron
- [ ] Admin dashboard metrics
- [ ] Seller earnings report
- [ ] System reconciliation tools
- [ ] Export functionality (CSV, Excel)

### Phase 7: Admin Tools & Security (Week 9)
- [ ] Manual balance adjustment
- [ ] Wallet freezing/unfreezing
- [ ] 2FA for large operations
- [ ] Audit log system
- [ ] Suspicious activity detection
- [ ] Admin action logging

### Phase 8: Future Enhancements (Later)
- [ ] Auto-deposit detection (bank API, MoMo API)
- [ ] Additional payment gateways (USDT, PayPal)
- [ ] Scheduled withdrawals
- [ ] Wallet vouchers/cards
- [ ] Referral rewards

---

## 6. Technical Considerations

### 6.1 MongoDB Transactions
Always use transactions for wallet operations to ensure atomicity:

```javascript
const session = client.startSession();
try {
  await session.withTransaction(async () => {
    // All wallet operations here
    await db.wallets.updateOne(..., { session });
    await db.wallet_transactions.insertOne(..., { session });
    await db.orders.updateOne(..., { session });
  });
} finally {
  await session.endSession();
}
```

### 6.2 Idempotency
- Use `transactionNumber` as idempotency key
- Check for duplicate transactions before processing
- Store request IDs to handle retries safely

### 6.3 Cron Jobs
Required scheduled tasks:

1. **Escrow Auto-Release**: Every hour
   ```javascript
   cron.schedule('0 * * * *', autoReleaseEscrows);
   ```

2. **Daily Summary Aggregation**: Every midnight
   ```javascript
   cron.schedule('0 0 * * *', generateDailySummary);
   ```

3. **Pending Deposit Reminders**: Every 6 hours
   ```javascript
   cron.schedule('0 */6 * * *', remindPendingDeposits);
   ```

4. **Withdrawal Processing Alerts**: Every hour
   ```javascript
   cron.schedule('0 * * * *', alertPendingWithdrawals);
   ```

### 6.4 Notifications
Events that trigger notifications:

- Deposit completed
- Withdrawal approved/rejected
- Escrow released
- Transfer received
- Low balance warning (< 10 AP)
- Purchase completed
- Sale pending release

### 6.5 Number Generation
Consistent format for all identifiers:

```javascript
function generateTxnNumber() {
  const date = new Date().toISOString().slice(0, 10).replace(/-/g, '');
  const sequence = getNextSequence('txn', date);  // Daily counter
  return `TXN-${date}-${sequence.toString().padStart(5, '0')}`;
}
// Example: TXN-20250129-00001

function generateEscrowNumber() {
  const date = new Date().toISOString().slice(0, 10).replace(/-/g, '');
  const sequence = getNextSequence('escrow', date);
  return `ESC-${date}-${sequence.toString().padStart(5, '0')}`;
}
// Example: ESC-20250129-00001

function generateWithdrawalNumber() {
  const date = new Date().toISOString().slice(0, 10).replace(/-/g, '');
  const sequence = getNextSequence('withdrawal', date);
  return `WTD-${date}-${sequence.toString().padStart(5, '0')}`;
}
// Example: WTD-20250129-00001

function generateDepositNumber() {
  const date = new Date().toISOString().slice(0, 10).replace(/-/g, '');
  const sequence = getNextSequence('deposit', date);
  return `DEP-${date}-${sequence.toString().padStart(5, '0')}`;
}
// Example: DEP-20250129-00001
```

### 6.6 Performance Optimization
- Use MongoDB indexes effectively (listed in each collection)
- Implement pagination for transaction history
- Cache wallet balances (Redis) with TTL
- Use aggregation pipelines for reports
- Implement read replicas for reporting queries

---

## 7. Open Questions & Decisions Needed

Please provide answers to finalize the implementation:

### 7.1 Commission Rates
- [ ] Are commission rates different per order type or global?
- [ ] Current assumption: Configurable per order type (5% for digital goods)

### 7.2 Withdrawal Fees
- [ ] Should we charge withdrawal fees? If yes, how much?
- [ ] Current assumption: Free if >= 500 AP, else 10 AP fee

### 7.3 P2P Transfer Fees
- [ ] Do P2P transfers have fees?
- [ ] Current assumption: No fees for transfers

### 7.4 Minimum Amounts
- [ ] Minimum deposit amount?
- [ ] Minimum withdrawal amount?
- [ ] Minimum transfer amount?
- [ ] Current assumption: 10 AP minimum for all

### 7.5 Admin Approval Thresholds
- [ ] Does every deposit need admin approval?
- [ ] Does every withdrawal need admin approval?
- [ ] Or only above certain thresholds?
- [ ] Current assumption: All manual deposits/withdrawals need approval

### 7.6 Refund Window
- [ ] Can buyers request refunds anytime during escrow?
- [ ] Or only with valid reasons (disputes)?
- [ ] Current assumption: Only through dispute system

### 7.7 Maker-Checker Workflow
- [ ] Should high-value operations require dual approval?
- [ ] What's the threshold?
- [ ] Current assumption: Not implemented initially

### 7.8 Daily Limits
- [ ] Maximum withdrawals per user per day?
- [ ] Maximum withdrawal amount per day?
- [ ] Maximum transfer amount per day?
- [ ] Current assumption: No limits initially

---

## 8. Migration from V1 to V2

### 8.1 Data Migration Strategy

**Option 1: One-time migration**
- Export all V1 wallet data
- Transform to V2 schema
- Import into MongoDB
- Freeze V1 system
- Switch to V2

**Option 2: Gradual migration**
- Run V1 and V2 in parallel
- New transactions go to V2
- Migrate users gradually
- Sync balances until complete

**Recommended**: Option 1 for clean cutover

### 8.2 Migration Steps

1. **Export V1 Data**
   ```sql
   SELECT * FROM wallets;
   SELECT * FROM transactions;
   SELECT * FROM payouts;
   SELECT * FROM withdrawals;
   ```

2. **Transform to V2 Schema**
   - Map V1 `balance` → V2 `balances.apCurrent`
   - Map V1 `pending_balance` → V2 `balances.apPendingCashout`
   - Convert VND amounts to AP (divide by 1000)
   - Generate transaction numbers for existing transactions
   - Preserve all transaction history

3. **Validation**
   - Verify total AP matches total VND/1000
   - Check all balances sum correctly
   - Validate all escrows transferred
   - Test withdrawal/deposit flows

4. **Cutover**
   - Freeze V1 system (read-only)
   - Import V2 data
   - Verify balances
   - Enable V2 system
   - Monitor for 24-48 hours

---

## 9. API Endpoints (Summary)

### User APIs
```
GET    /api/wallet/balance              - Get wallet balance
GET    /api/wallet/transactions         - Get transaction history
GET    /api/wallet/transactions/:id     - Get transaction details
POST   /api/wallet/transfer             - Send AP to another user
POST   /api/wallet/withdraw             - Request withdrawal
GET    /api/wallet/withdrawals          - Get withdrawal history
```

### Admin APIs
```
POST   /api/admin/deposits/manual       - Create manual deposit
GET    /api/admin/deposits/pending      - Get pending deposits
POST   /api/admin/deposits/:id/approve  - Approve deposit
POST   /api/admin/deposits/:id/reject   - Reject deposit

GET    /api/admin/withdrawals/pending   - Get pending withdrawals
POST   /api/admin/withdrawals/:id/approve - Process withdrawal
POST   /api/admin/withdrawals/:id/reject  - Reject withdrawal

GET    /api/admin/escrows               - Get escrow list
POST   /api/admin/escrows/:id/release   - Manual release escrow
POST   /api/admin/escrows/:id/refund    - Refund escrow

POST   /api/admin/wallet/adjust         - Manual balance adjustment
POST   /api/admin/wallet/freeze         - Freeze wallet
POST   /api/admin/wallet/unfreeze       - Unfreeze wallet

GET    /api/admin/reports/daily         - Daily summary
GET    /api/admin/reports/reconciliation - Reconciliation report
GET    /api/admin/reports/seller-earnings - Seller earnings
```

---

## 10. References

### Related Documents
- [V1 Wallet System](../v1/06-wallet-payment.md) - Previous implementation
- Order System Design (to be created)
- Dispute System Design (to be created)

### External Resources
- [MongoDB Transactions](https://www.mongodb.com/docs/manual/core/transactions/)
- [Exness Wallet System](https://www.exness.com/) - Inspiration

---

**Document Version**: 1.0
**Created**: 2025-01-29
**Last Updated**: 2025-01-29
**Status**: Design Phase - Awaiting decisions on open questions
