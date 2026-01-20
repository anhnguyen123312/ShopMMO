# MMO API Reference Skill

> **Trigger**: Use when working with MMO API endpoints, understanding business logic, implementing new features, or debugging existing functionality.

## Overview

This document provides comprehensive API reference for the MMO marketplace/wallet system built with Rust, actix-web, MongoDB, and Redis.

---

## Module Architecture

```
src/modules/
├── auth/           # Authentication & Authorization
├── wallet/         # Wallet, Transactions, Escrow, Disputes
├── permissions/    # RBAC System
├── category/       # Product Categories
└── shop/           # Vendor Shops
```

---

# AUTHENTICATION MODULE

## Endpoints Summary

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| POST | `/api/auth/register` | Register new user | No |
| POST | `/api/auth/login` | Login | No |
| POST | `/api/auth/refresh` | Refresh token | No |
| POST | `/api/auth/logout` | Logout | Yes |
| GET | `/api/auth/me` | Get current user | Yes |
| POST | `/api/auth/change-password` | Change password | Yes |
| POST | `/api/auth/admin/assign-roles` | Assign roles | Admin |
| GET | `/api/auth/admin/users/{id}/roles` | Get user roles | Admin |

## Authentication Flow

```
┌─────────────────────────────────────────────────────────────┐
│  1. REGISTRATION                                            │
│     POST /api/auth/register                                 │
│     → Validate input                                        │
│     → Check email/username uniqueness                       │
│     → Hash password (bcrypt, cost 12)                       │
│     → Create user (BUYER role, PendingVerification)         │
│     → Create wallet (optional)                              │
│     → Generate tokens                                       │
│     → Return AuthResponse                                   │
└─────────────────────────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────────────────────┐
│  2. LOGIN                                                   │
│     POST /api/auth/login                                    │
│     → Find user by email/username                           │
│     → Verify password                                       │
│     → Check account active                                  │
│     → Update last_login_at                                  │
│     → Generate tokens                                       │
│     → Return AuthResponse                                   │
└─────────────────────────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────────────────────┐
│  3. AUTHENTICATED REQUESTS                                  │
│     Authorization: Bearer <access_token>                    │
│     → Middleware validates JWT                              │
│     → Extracts AuthUser to request extensions               │
│     → Handler accesses via AuthUser extractor               │
└─────────────────────────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────────────────────┐
│  4. TOKEN REFRESH                                           │
│     POST /api/auth/refresh                                  │
│     → Verify refresh token JWT                              │
│     → Find token in DB (not revoked)                        │
│     → Generate new token pair                               │
│     → Revoke old refresh token                              │
│     → Return AuthResponse                                   │
└─────────────────────────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────────────────────┐
│  5. LOGOUT                                                  │
│     POST /api/auth/logout                                   │
│     → Revoke refresh token in DB                            │
└─────────────────────────────────────────────────────────────┘
```

## JWT Token Structure (V2)

```rust
TokenClaims {
    sub: String,              // User ID
    wallet_id: String,        // Wallet ID
    username: String,         // Username
    email: String,            // Email
    role: String,             // Primary role (backward compat)
    roles: Vec<String>,       // All roles (V2)
    perm_version: u32,        // Permission version
    iat: i64,                 // Issued at
    exp: i64,                 // Expiration
    token_type: String,       // "access" or "refresh"
}
```

**Token Expiration:**
- Access: 15 minutes (JWT_ACCESS_TOKEN_EXPIRES_IN)
- Refresh: 7 days (JWT_REFRESH_TOKEN_EXPIRES_IN)

## Request/Response Examples

### Register
```json
// POST /api/auth/register
{
  "username": "johndoe123",
  "email": "user@example.com",
  "password": "password123",
  "name": "John Doe"
}

// Response 201
{
  "success": true,
  "data": {
    "accessToken": "eyJ...",
    "refreshToken": "eyJ...",
    "tokenType": "Bearer",
    "expiresIn": 900,
    "user": {
      "id": "...",
      "username": "johndoe123",
      "email": "user@example.com",
      "name": "John Doe",
      "role": "BUYER"
    }
  }
}
```

### Login
```json
// POST /api/auth/login
{
  "identifier": "johndoe123",  // or email
  "password": "password123"
}
```

### Assign Roles (Admin)
```json
// POST /api/auth/admin/assign-roles
{
  "userId": "507f1f77bcf86cd799439011",
  "roles": ["BUYER", "SELLER"]
}
```

---

# CATEGORY MODULE

## Endpoints Summary

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| GET | `/api/categories/tree` | Get category tree | No |
| GET | `/api/categories/{id}` | Get category by ID | No |
| POST | `/api/admin/categories` | Create category | Admin |
| PUT | `/api/admin/categories/{id}` | Update category | Admin |
| DELETE | `/api/admin/categories/{id}` | Delete category | Admin |
| POST | `/api/admin/categories/reorder` | Reorder categories | Admin |

## Category Schema

```rust
Category {
    id: ObjectId,
    name: String,              // 3-50 chars, unique
    slug: String,              // lowercase, hyphens
    parent_id: Option<ObjectId>,
    commission_rate: f64,      // 0-100%
    icon: Option<String>,
    description: Option<String>,
    sort_order: i32,
    status: CategoryStatus,    // Active | Deleted
}
```

## Key Feature: Auto Inventory Collection

When creating a category, the system automatically creates:
- MongoDB collection: `inventory_{slug}`
- Indexes on: `product_id`, `shop_id`, `is_sold`, `content`, `created_at`

---

# SHOP MODULE

## Endpoints Summary

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| POST | `/api/vendor/shop/create` | Create shop | Vendor |
| GET | `/api/vendor/shop/dashboard` | Get dashboard | Vendor |
| GET | `/api/vendor/shop/verification` | Get verification info | Vendor |
| PUT | `/api/vendor/shop/update` | Update shop | Vendor |
| PUT | `/api/vendor/shop/policies` | Update policies | Vendor |
| POST | `/api/vendor/shop/upload/logo` | Upload logo | Vendor |
| POST | `/api/vendor/shop/upload/banner` | Upload banner | Vendor |
| GET | `/api/shops/{shop_id}` | Get shop | No |
| GET | `/api/shops/slug/{slug}` | Get by slug | No |
| GET | `/api/shops` | List shops | No |
| POST | `/api/shop/telegram/verify` | Verify telegram | Bot |
| GET | `/admin/api/shops/stats` | Get statistics | Admin |

## Shop Completion Workflow

```
1. Create Shop
   └── telegram_verified: false
   └── total_products: 0
   └── policies: null
   └── completion: 0%

2. Verify Telegram (via bot)
   └── telegram_verified: true
   └── completion: 33%

3. Add Products
   └── total_products > 0
   └── completion: 66%

4. Set Policies
   └── policies: set
   └── is_complete: true
   └── completion: 100%
```

## Shop Levels

| Level | Sales Range | Commission |
|-------|-------------|------------|
| New | 0-100 | 5% |
| Silver | 101-500 | 5% |
| Gold | 501-2000 | 5% |
| Diamond | 2001-10000 | 5% |
| Partner | 10000+ | Negotiable |

## Telegram Verification Flow

1. Create shop → Generate verification code (UUID)
2. Store in Redis: `telegram:code:{code}` → shop_id (24h TTL)
3. User sends `/start {code}` to @p2pmmo bot
4. Bot calls `POST /api/shop/telegram/verify`
5. System updates shop: `telegram_verified = true`
6. Delete verification codes from Redis

---

# INFRASTRUCTURE

## Request Flow

```
HTTP Request
    │
    ▼ RequestId Middleware (generates X-Request-ID)
    ▼ CORS Middleware
    ▼ AuthMiddleware (validates JWT, extracts AuthUser)
    ▼ GrantsMiddleware (extracts roles for actix-web-grants)
    ▼ Handler (validates input, calls service)
    ▼ Service (business logic)
    ▼ Repository (database operations)
    ▼
HTTP Response
```

## Error Handling Chain

```
DbError → ServiceError → ApiError → HTTP Response
```

| Error Type | HTTP Status |
|------------|-------------|
| NotFound | 404 |
| BadRequest | 400 |
| Unauthorized | 401 |
| Forbidden | 403 |
| Conflict | 409 |
| InternalError | 500 |
| DatabaseError | 500 |

## Standard Response Format

```json
// Success
{
  "success": true,
  "message": null,
  "data": { /* payload */ },
  "error": null
}

// Error
{
  "success": false,
  "message": "Error description",
  "data": null,
  "error": {
    "error": "Detailed message",
    "status_code": 400
  }
}
```

## Database Connections

### MongoDB
- Connection pooling: min 10, max 100
- Transaction support via sessions
- Collections in `database::mongodb::collections`

### Redis
- Key patterns in `database::redis::keys`
- Used for: sessions, tokens, rate limits, OTP, telegram verification, permission cache

## Utility Functions

### ID Generation
```rust
generate_transaction_number()  // TXN-20250130-00001
generate_escrow_number()       // ESC-20250130-00001
generate_withdrawal_number()   // WTD-20250130-00001
generate_deposit_number()      // DEP-20250130-00001
generate_order_number()        // ORD-20250130-00001
generate_request_id()          // UUID
```

### Password Hashing
```rust
hash_password(password, cost)      // bcrypt hash
verify_password(password, hash)    // bcrypt verify
```

### JWT
```rust
generate_access_token_v2(...)      // 15min access token
generate_refresh_token_v2(...)     // 7d refresh token
verify_token(token, secret)        // Validate and decode
parse_duration("15m")              // Parse duration string
```

---

# CONFIGURATION

## Environment Variables

```env
# Server
HOST=127.0.0.1
PORT=8080
ENVIRONMENT=development

# MongoDB
MONGODB_URI=mongodb://localhost:27017
MONGODB_DATABASE=mmo_db
MONGODB_MAX_POOL_SIZE=100
MONGODB_MIN_POOL_SIZE=10

# Redis
REDIS_URI=redis://localhost:6379

# JWT
JWT_SECRET=your-super-secret-key
JWT_ACCESS_TOKEN_EXPIRES_IN=15m
JWT_REFRESH_TOKEN_EXPIRES_IN=7d

# Security
BCRYPT_COST=12

# CORS
CORS_ALLOWED_ORIGINS=http://localhost:3000

# Telegram
TELEGRAM_BOT_API_KEY=your-bot-api-key
```

---

# TESTING

## Run Commands

```bash
# Unit tests
cargo test --lib

# Integration tests (requires MongoDB)
cargo test -- --ignored

# Specific module tests
cargo test --lib wallet::domain::tests
cargo test --lib auth::domain::tests

# With output
cargo test -- --nocapture
```

## Test Patterns

### Unit Tests
- Location: End of source files in `#[cfg(test)] mod tests { }`
- Use `#[test]` attribute
- Mock dependencies where needed

### Integration Tests
- Location: `tests/` directory
- Use `#[tokio::test]` and `#[ignore]`
- Connect to test database
- Cleanup after tests

---

# CHECKLIST: Adding New Feature

- [ ] Create domain model in `domain.rs`
- [ ] Create DTOs in `dto.rs`
- [ ] Implement repository in `repository.rs`
- [ ] Implement service in `service.rs`
- [ ] Create handlers in `handler.rs`
- [ ] Configure routes in `routes.rs`
- [ ] Add to module exports in `mod.rs`
- [ ] Register routes in `main.rs`
- [ ] Add OpenAPI docs with utoipa
- [ ] Write unit tests
- [ ] Write integration tests
- [ ] Update this documentation

---

# WALLET MODULE

## Endpoints Summary

### User Wallet APIs (`/api/wallet`)

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| GET | `/api/wallet/balance` | Get wallet balance | User |
| GET | `/api/wallet/transactions` | Get transaction history | User |
| POST | `/api/wallet/deposit/initiate` | Initiate deposit | User |
| GET | `/api/wallet/deposit/status/{tx_id}` | Get deposit status | User |
| GET | `/api/wallet/deposits/history` | Get deposit history | User |
| POST | `/api/wallet/withdrawal` | Create withdrawal request | Vendor |
| POST | `/api/wallet/purchase` | Create purchase (escrow) | Buyer |
| POST | `/api/wallet/escrow/{id}/early-release` | Early release escrow | Buyer |
| POST | `/api/wallet/escrow/{id}/dispute` | Create dispute | User |
| GET | `/api/wallet/disputes` | List disputes | User |
| GET | `/api/wallet/disputes/{id}` | Get dispute detail | User |
| POST | `/api/wallet/disputes/{id}/seller/respond` | Seller respond | Seller |
| POST | `/api/wallet/disputes/{id}/buyer/respond` | Buyer respond | Buyer |

### Admin Wallet APIs (`/admin/api/wallet`)

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| POST | `/admin/api/wallet/freeze` | Freeze wallet | Admin |
| POST | `/admin/api/wallet/unfreeze` | Unfreeze wallet | Admin |
| POST | `/admin/api/wallet/debit` | Manual debit | Admin |
| POST | `/admin/api/wallet/commission` | Set shop commission | Admin |
| POST | `/admin/api/wallet/deposit` | Admin manual deposit | Admin |
| POST | `/admin/api/wallet/deposit/manual` | Manual deposit | Admin |
| POST | `/admin/api/wallet/deposit/auto` | Auto deposit | Admin |
| POST | `/admin/api/wallet/deposit/webhook` | Deposit webhook | Internal |
| GET | `/admin/api/wallet/deposits/history` | Admin deposit history | Admin |
| GET | `/admin/api/wallet/withdrawal/{id}/validate` | Validate withdrawal | Admin |
| POST | `/admin/api/wallet/withdrawal/{id}/approve` | Approve withdrawal | Admin |
| POST | `/admin/api/wallet/withdrawal/{id}/reject` | Reject withdrawal | Admin |
| POST | `/admin/api/wallet/withdrawal/{id}/complete` | Complete transfer | Admin |
| POST | `/admin/api/wallet/escrow/{id}/resolve/refund` | Resolve - refund | Admin |
| POST | `/admin/api/wallet/escrow/{id}/resolve/release` | Resolve - release | Admin |
| POST | `/admin/api/wallet/escrow/jobs/auto-release` | Auto-release job | Admin |
| POST | `/admin/api/wallet/disputes/{id}/extend` | Extend deadline | Admin |
| POST | `/admin/api/wallet/disputes/partial-refund` | Partial refund | Admin |
| POST | `/admin/api/wallet/disputes/jobs/auto-escalate` | Auto-escalate job | Admin |
| GET | `/admin/api/wallet/logs` | Get admin logs | Admin |
| GET | `/admin/api/wallet/dashboard` | Dashboard stats | Admin |
| POST | `/admin/api/wallet/reconcile` | Trigger reconciliation | Admin |
| POST | `/admin/api/wallet/cron/start` | Start cron jobs | Admin |
| POST | `/admin/api/wallet/cron/stop` | Stop cron jobs | Admin |

## Wallet Domain Models

### Wallet
```rust
Wallet {
    wallet_id: String,           // WLT-{user_id}
    user_id: ObjectId,
    wallet_type: WalletType,     // User | Seller | Platform
    balance: i64,                // Current balance (Trust currency)
    pending_in: i64,             // Pending incoming
    pending_out: i64,            // Pending outgoing
    locked: i64,                 // Locked in escrow
    total_deposited: i64,
    total_withdrawn: i64,
    admin_debt: i64,             // Debt from admin operations
    is_frozen: bool,
    is_active: bool,
    commission_rate: Option<f64>, // Shop-specific commission
}
```

### Transaction
```rust
Transaction {
    tx_id: String,               // TXN-YYYYMMDD-NNNNN
    wallet_id: String,
    tx_type: TransactionType,    // Deposit | Withdrawal | EscrowHold | EscrowRelease | Commission | AdminDebit | AdminCredit
    direction: Direction,        // In | Out
    amount: i64,
    balance_before: i64,
    balance_after: i64,
    status: String,              // pending | completed | failed
    reference_id: Option<String>, // Related escrow/order ID
    description: Option<String>,
}
```

### EscrowHold
```rust
EscrowHold {
    escrow_id: String,           // ESC-YYYYMMDD-NNNNN
    order_id: String,
    buyer_wallet_id: String,
    seller_wallet_id: String,
    amount: i64,
    platform_fee: i64,           // Commission amount
    status: EscrowStatus,        // Held | Released | Refunded | Disputed
    release_at: DateTime,        // Auto-release time (72h)
    released_at: Option<DateTime>,
}
```

### DisputeCase
```rust
DisputeCase {
    dispute_id: String,
    escrow_id: String,
    order_id: String,
    buyer_id: String,
    seller_id: String,
    reason: DisputeReason,       // NotAsDescribed | NotDelivered | QualityIssue | Other
    buyer_evidence: Vec<String>,
    seller_evidence: Vec<String>,
    status: DisputeStatus,       // Open | SellerResponded | BuyerResponded | Escalated | Resolved
    seller_action: Option<SellerAction>, // Accept | PartialAccept | Reject | Replacement
    refund_amount: Option<i64>,
    seller_deadline: DateTime,
    buyer_deadline: Option<DateTime>,
    exchange_count: i32,         // Max 3 exchanges
    resolved_by: Option<String>,
    resolution: Option<String>,
}
```

## Key Workflows

### Purchase & Escrow Flow
```
Buyer initiates purchase
    │
    ▼
POST /api/wallet/purchase
    ├── Validate buyer balance
    ├── Lock funds in escrow
    ├── Create EscrowHold (status: Held)
    ├── Create transaction (EscrowHold, OUT)
    ├── Set auto-release time (72h)
    └── Return escrow details
    │
    ▼
[Wait 72h or buyer confirms]
    │
    ├── Auto-release OR POST /escrow/{id}/early-release
    │
    ▼
Release Escrow
    ├── Calculate commission (5% default)
    ├── Transfer to seller (amount - commission)
    ├── Transfer commission to platform
    ├── Update EscrowHold (status: Released)
    └── Create transactions for all parties
```

### Dispute Flow
```
Buyer opens dispute
    │
    ▼
POST /api/wallet/escrow/{id}/dispute
    ├── Change escrow status to Disputed
    ├── Create DisputeCase
    ├── Set seller_deadline (48h)
    └── Notify seller
    │
    ▼
Seller responds (within 48h)
    │
    ├── ACCEPT → Full refund to buyer
    ├── PARTIAL_ACCEPT → Propose partial refund
    ├── REJECT → Provide evidence
    └── REPLACEMENT → Offer replacement
    │
    ▼
[If PARTIAL_ACCEPT or REJECT]
    │
    ▼
Buyer responds (48h)
    │
    ├── ACCEPT_OFFER → Process partial refund
    └── ESCALATE → Admin reviews
    │
    ▼
[Max 3 exchanges, then auto-escalate]
    │
    ▼
Admin Resolution
    ├── Full refund
    ├── Partial refund
    └── Release to seller
```

### Withdrawal Flow
```
Vendor requests withdrawal
    │
    ▼
POST /api/wallet/withdrawal
    ├── Validate balance (balance - locked - pending_out >= amount)
    ├── Check daily limits
    ├── Check fraud patterns
    ├── Create WithdrawalRequest (status: pending)
    ├── Lock funds (pending_out += amount)
    └── Return request details
    │
    ▼
Admin reviews
    │
    ├── GET /admin/.../validate → Run validation checks
    │
    ├── POST /admin/.../approve → Approve for processing
    │
    └── POST /admin/.../reject → Reject with reason
    │
    ▼
[If approved]
    │
    ▼
POST /admin/.../complete
    ├── Deduct from balance
    ├── Create transaction (Withdrawal, OUT)
    ├── Update total_withdrawn
    └── Update request (status: completed)
```

---

# PERMISSIONS MODULE

## Endpoints Summary

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| GET | `/permissions` | List all permissions | Admin |
| POST | `/permissions/roles` | Create role | Admin |
| GET | `/permissions/roles` | List roles | Admin |
| DELETE | `/permissions/roles/{name}` | Delete role | Admin |
| PUT | `/permissions/roles/{name}/permissions` | Update role permissions | Admin |
| POST | `/permissions/roles/assign` | Assign role to user | Admin |

## Permission Format (V2)

Permissions follow a **resource:action:scope** format similar to AWS IAM:

```
resource:action:scope
         │      │
         │      └── Scope (optional):
         │          - own   → User acts on their own resources
         │          - all   → User acts on any resource (admin)
         │          - admin → Admin-only operations
         │          - (none)→ Universal action
         │
         └── Action: create, read, update, delete, etc.
```

**Examples:**
- `product:create:own` - Seller creates their own product
- `wallet:read:all` - Admin reads any wallet
- `dispute:resolve:refund` - Admin resolves dispute with refund
- `order:cancel` - Cancel order (universal)

## Permission Constants (113 total)

### Product Permissions (14)
| Permission | Description |
|------------|-------------|
| `product:create:own` | Seller creates own product |
| `product:read:own` | Read own products |
| `product:read:all` | Read all products (admin/public) |
| `product:update:own` | Update own product |
| `product:update:all` | Update any product (admin) |
| `product:delete:own` | Delete own product |
| `product:delete:all` | Delete any product (admin) |
| `product:list:own` | List own products |
| `product:list:all` | List all products |
| `product:publish` | Publish product (make visible) |
| `product:unpublish` | Unpublish product |
| `product:bulk_update` | Bulk update products |
| `product:import` | Import products from file |
| `product:export` | Export products to file |

### Order Permissions (12)
| Permission | Description |
|------------|-------------|
| `order:create:own` | Buyer creates order |
| `order:read:own` | Read own orders (buyer/seller) |
| `order:read:all` | Read all orders (admin) |
| `order:update:own` | Update own order |
| `order:update:all` | Update any order (admin) |
| `order:list:own` | List own orders |
| `order:list:all` | List all orders (admin) |
| `order:cancel` | Cancel order |
| `order:confirm` | Seller confirms order |
| `order:ship` | Mark order as shipped |
| `order:complete` | Mark order as complete |
| `order:refund` | Process refund (admin) |

### Wallet Permissions (25)
| Permission | Description |
|------------|-------------|
| **Basic** | |
| `wallet:read:own` | Read own wallet balance |
| `wallet:read:all` | Read all wallets (admin) |
| `wallet:list:all` | List all wallets (admin) |
| **Deposit** | |
| `wallet:deposit:own` | User initiates deposit |
| `wallet:deposit:manual` | Admin manual deposit |
| `wallet:deposit:auto` | System auto deposit |
| `wallet:deposit:webhook` | Process payment webhook |
| **Withdrawal** | |
| `wallet:withdraw:own` | User requests withdrawal |
| `wallet:withdraw:validate` | Admin validates withdrawal |
| `wallet:withdraw:approve` | Admin approves withdrawal |
| `wallet:withdraw:reject` | Admin rejects withdrawal |
| `wallet:withdraw:complete` | Admin completes transfer |
| **Escrow** | |
| `wallet:escrow:create` | Create escrow (purchase) |
| `wallet:escrow:release` | Release escrow to seller |
| `wallet:escrow:refund` | Refund escrow to buyer |
| `wallet:escrow:early_release` | Buyer early release |
| `wallet:escrow:auto_release` | System auto-release job |
| **Admin Operations** | |
| `wallet:freeze` | Freeze wallet (admin) |
| `wallet:unfreeze` | Unfreeze wallet (admin) |
| `wallet:debit:admin` | Admin manual debit |
| `wallet:credit:admin` | Admin manual credit |
| `wallet:set_commission` | Set shop commission rate |
| `wallet:reconcile` | Trigger reconciliation |
| `wallet:dashboard` | View admin dashboard |
| `wallet:logs` | View admin operation logs |
| `wallet:cron:manage` | Start/stop cron jobs |

### Dispute Permissions (13)
| Permission | Description |
|------------|-------------|
| `dispute:create:own` | Buyer creates dispute |
| `dispute:read:own` | Read own disputes |
| `dispute:read:all` | Read all disputes (admin) |
| `dispute:list:own` | List own disputes |
| `dispute:list:all` | List all disputes (admin) |
| `dispute:respond:seller` | Seller responds to dispute |
| `dispute:respond:buyer` | Buyer responds to dispute |
| `dispute:escalate` | Escalate to admin |
| `dispute:resolve:refund` | Admin resolves with refund |
| `dispute:resolve:release` | Admin resolves with release |
| `dispute:partial_refund` | Admin partial refund |
| `dispute:extend_deadline` | Admin extends deadline |
| `dispute:auto_escalate` | System auto-escalate job |

### User Permissions (14)
| Permission | Description |
|------------|-------------|
| `user:create` | Create user (registration) |
| `user:read:own` | Read own profile |
| `user:read:all` | Read any user (admin) |
| `user:update:own` | Update own profile |
| `user:update:all` | Update any user (admin) |
| `user:delete:own` | Delete own account |
| `user:delete:all` | Delete any user (admin) |
| `user:list:all` | List all users (admin) |
| `user:suspend` | Suspend user (admin) |
| `user:activate` | Activate user (admin) |
| `user:verify_email` | Verify email |
| `user:reset_password` | Reset password |
| `user:assign_roles` | Assign roles (admin) |
| `user:view_roles` | View user roles (admin) |

### Role Permissions (6)
| Permission | Description |
|------------|-------------|
| `role:create` | Create role (admin) |
| `role:read` | Read roles |
| `role:update` | Update role (admin) |
| `role:delete` | Delete role (admin) |
| `role:list` | List all roles |
| `role:assign_permissions` | Assign permissions to role |

### Shop Permissions (16)
| Permission | Description |
|------------|-------------|
| `shop:create:own` | Vendor creates shop |
| `shop:read:own` | Read own shop |
| `shop:read:all` | Read any shop |
| `shop:update:own` | Update own shop |
| `shop:update:all` | Update any shop (admin) |
| `shop:delete:own` | Delete own shop |
| `shop:delete:all` | Delete any shop (admin) |
| `shop:list:all` | List all shops |
| `shop:verify:telegram` | Verify telegram |
| `shop:suspend` | Suspend shop (admin) |
| `shop:activate` | Activate shop (admin) |
| `shop:set_commission` | Set commission (admin) |
| `shop:upload:logo` | Upload shop logo |
| `shop:upload:banner` | Upload shop banner |
| `shop:update:policies` | Update shop policies |
| `shop:view:stats` | View shop statistics (admin) |

### Category Permissions (7)
| Permission | Description |
|------------|-------------|
| `category:create` | Create category (admin) |
| `category:read` | Read categories (public) |
| `category:update` | Update category (admin) |
| `category:delete` | Delete category (admin) |
| `category:list` | List categories (public) |
| `category:reorder` | Reorder categories (admin) |
| `category:tree` | Get category tree (public) |

### Admin Permissions (5)
| Permission | Description |
|------------|-------------|
| `admin:full` | Full admin access (grants all) |
| `admin:read` | Read admin resources |
| `admin:write` | Write admin resources |
| `admin:system:config` | System configuration |
| `admin:audit:logs` | View audit logs |

## Role Schema

```rust
Role {
    name: String,                    // BUYER, SELLER, ADMIN, etc.
    display_name: String,
    level: i32,                      // Priority level
    parent_role_id: Option<ObjectId>,
    inherits_from: Vec<String>,      // Parent role names
    direct_permissions: Vec<ObjectId>,
    flattened_permissions: Vec<String>, // All permissions (inherited + direct)
    is_system: bool,                 // System roles can't be deleted
    is_active: bool,
    version: i32,
}
```

## Default Roles Configuration

### BUYER (Level 0)
```
product:read:all, product:list:all
order:create:own, order:read:own, order:list:own, order:cancel
wallet:read:own, wallet:deposit:own, wallet:escrow:create, wallet:escrow:early_release
dispute:create:own, dispute:read:own, dispute:list:own, dispute:respond:buyer, dispute:escalate
user:read:own, user:update:own, user:delete:own, user:verify_email, user:reset_password
shop:read:all
category:read, category:list, category:tree
```

### SELLER (Level 1, inherits BUYER)
```
+ product:create:own, product:update:own, product:delete:own, product:list:own, product:publish, product:unpublish
+ order:confirm, order:ship, order:complete
+ wallet:withdraw:own
+ dispute:respond:seller
+ shop:create:own, shop:read:own, shop:update:own, shop:delete:own, shop:verify:telegram
+ shop:upload:logo, shop:upload:banner, shop:update:policies
```

### MODERATOR (Level 2)
```
product:read:all, product:list:all
order:read:all, order:list:all
dispute:read:all, dispute:list:all, dispute:extend_deadline
user:read:all, user:list:all
shop:read:all, shop:list:all
```

### ADMIN (Level 3, inherits MODERATOR)
```
+ product:update:all, product:delete:all, product:bulk_update, product:import, product:export
+ order:update:all, order:refund
+ wallet:read:all, wallet:list:all, wallet:deposit:manual, wallet:deposit:auto
+ wallet:withdraw:validate, wallet:withdraw:approve, wallet:withdraw:reject, wallet:withdraw:complete
+ wallet:escrow:release, wallet:escrow:refund, wallet:freeze, wallet:unfreeze
+ wallet:debit:admin, wallet:credit:admin, wallet:set_commission, wallet:reconcile
+ wallet:dashboard, wallet:logs, wallet:cron:manage
+ dispute:resolve:refund, dispute:resolve:release, dispute:partial_refund
+ user:update:all, user:delete:all, user:suspend, user:activate, user:assign_roles, user:view_roles
+ role:create, role:read, role:update, role:delete, role:list, role:assign_permissions
+ shop:update:all, shop:delete:all, shop:suspend, shop:activate, shop:set_commission, shop:view:stats
+ category:create, category:update, category:delete, category:reorder
+ admin:read, admin:write, admin:audit:logs
```

### SUPER_ADMIN (Level 4, inherits ADMIN)
```
+ admin:full, admin:system:config
+ wallet:deposit:webhook, wallet:escrow:auto_release
+ dispute:auto_escalate
```

## RBAC Flow

```
Request with JWT
    │
    ▼
AuthMiddleware
    ├── Extract roles from JWT claims
    └── Create AuthUser with roles array
    │
    ▼
GrantsMiddleware
    ├── Call extract_permissions(auth_user)
    ├── Check perm_version against cache
    ├── If stale: fetch from DB, update cache
    └── Return permissions HashSet
    │
    ▼
Handler with #[protect("resource:action:scope")]
    └── actix-web-grants checks if permission exists
    │
    ▼
[Allow or 403 Forbidden]
```

## Permission Cache

- Key: `permissions:{user_id}`
- Value: JSON with permissions array and version
- TTL: Until perm_version changes
- Invalidation: On role assignment, increment user's perm_version

## API Usage Examples

```rust
// Handler requiring wallet:read:own
#[protect("wallet:read:own")]
async fn get_balance(auth: AuthUser) -> Result<impl Responder, ApiError> {
    // User can only read their own wallet
}

// Handler requiring admin wallet read
#[protect("wallet:read:all")]
async fn admin_list_wallets() -> Result<impl Responder, ApiError> {
    // Admin can read any wallet
}

// Handler with multiple permissions (ANY)
#[protect(any("dispute:resolve:refund", "dispute:resolve:release"))]
async fn resolve_dispute() -> Result<impl Responder, ApiError> {
    // Admin can resolve with either action
}
```
