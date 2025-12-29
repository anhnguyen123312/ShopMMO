1. AP Currency Definition

  First, let's clarify what AP represents:

  Questions for you:
  - What does "AP" stand for? (e.g., "Account Points", "Application Points"?)
  - AP is name currency in shop, 1 AP = 1.000VND 

  My initial understanding:
  - AP is your internal currency (like Exness uses USD as base currency)
  - Users deposit real money → converted to AP
  - All transactions happen in AP
  - Withdrawals convert AP back to real money

  ---
  2. Balance States Design

  Based on your requirements, I see you need:

  USER WALLET STRUCTURE (V2):
  ┌─────────────────────────────────────────────────────────────┐
  │  AP CURRENT (Available Balance)                             │
  │  └── Can be used for purchases (Buyer)                     │
  │  └── Can be withdrawn (Seller)                             │
  │                                                             │
  │  AP PENDING CASHOUT (Escrow/Hold Balance)                  │
  │  └── Seller earnings waiting 3-5 days verification         │
  │  └── Cannot be withdrawn yet                                │
  │  └── Automatically releases to AP CURRENT after hold period │
  │                                                             │
  │  AP TOTAL = AP CURRENT + AP PENDING                         │
  └─────────────────────────────────────────────────────────────┘

  Questions:
  - Should we have separate "pending deposit" for deposits being verified?
  - Do you want a "frozen/locked" balance for disputes or violations?
  - Should the hold period be configurable per order? (3-5 days varies by what?)

  ---
  3. Money Flow Tracking System

  You mentioned tracking like Exness. Here's my proposed structure:

  3.1 Transaction Types with Sources

  TRANSACTION TYPES:
  ├── DEPOSITS (Money IN)
  │   ├── deposit_manual (Admin adds AP)
  │   ├── deposit_bank (Bank transfer → AP)
  │   ├── deposit_momo (MoMo → AP)
  │   ├── deposit_usdt (USDT → AP)
  │   ├── deposit_paypal (PayPal → AP)
  │   └── deposit_other (Future gateways)
  │
  ├── PURCHASES (Money MOVE)
  │   ├── purchase (Buyer → Seller pending)
  │   ├── purchase_released (Pending → Seller current)
  │   └── purchase_refunded (Seller → Buyer)
  │
  ├── WITHDRAWALS (Money OUT)
  │   ├── withdraw_bank
  │   ├── withdraw_momo
  │   └── withdraw_crypto
  │
  └── ADJUSTMENTS
      ├── admin_add (Admin correction +)
      ├── admin_deduct (Admin correction -)
      ├── commission_earn (Affiliate commission)
      └── commission_deduct (Platform fee)

  3.2 Money Flow Ledger

  Questions:
  - Do you want double-entry accounting (debit/credit)?
  - Should we track "system wallet" balance (money pool)?
  - How do you want to reconcile third-party gateway balances with system AP?

  Example scenario:
  User deposits 1,000,000 VND via Bank
  → Gateway receives: 1,000,000 VND
  → System credits: 1,000 AP (if rate is 1 AP = 1,000 VND)
  → System wallet: 1,000,000 VND
  → User wallet: 1,000 AP

  We need to track BOTH:
  - Real money in gateway accounts
  - AP in user wallets

  ---
  4. Database Design Proposal

  Based on your requirements, here's my initial schema:

  4.1 Core Tables

  -- User Wallets (One per user)
  wallets
  - id
  - user_id (unique)
  - ap_current (available AP)
  - ap_pending_cashout (escrow AP)
  - ap_lifetime_earned (seller total)
  - ap_lifetime_spent (buyer total)
  - ap_lifetime_deposited
  - ap_lifetime_withdrawn

  -- Transaction Ledger (Every money movement)
  wallet_transactions
  - id
  - transaction_number (TXN-YYYYMMDD-XXXXX)
  - user_id
  - type (deposit, purchase, withdraw, etc.)
  - source_type (manual, bank, momo, system, etc.)
  - amount_ap (in AP)
  - balance_before_ap
  - balance_after_ap
  - description
  - metadata (JSON: gateway_ref, bank_info, etc.)
  - related_user_id (for P2P transactions)
  - related_order_id
  - admin_id (if manual)
  - status (pending, completed, failed)
  - created_at, completed_at

  -- Money Flow Tracking (Real currency tracking)
  money_flow_entries
  - id
  - transaction_id (FK to wallet_transactions)
  - flow_type (in, out, hold)
  - entity_type (user, seller, admin, gateway, system)
  - entity_id
  - currency (VND, USD, USDT)
  - amount_real (real currency amount)
  - amount_ap (AP equivalent)
  - exchange_rate
  - gateway_name (momo, bank, paypal, etc.)
  - gateway_reference
  - created_at

  -- Escrow Tracking (Pending cashouts)
  escrow_holds
  - id
  - order_id
  - seller_id
  - amount_ap
  - hold_until (3-5 days from order)
  - status (holding, released, refunded)
  - release_trigger (auto, manual, dispute_resolved)
  - created_at, released_at

  -- Withdrawal Requests
  withdrawal_requests
  - id
  - user_id
  - amount_ap
  - amount_real (converted amount)
  - currency (VND, USD, etc.)
  - exchange_rate_at_request
  - method (bank, momo, crypto)
  - destination_details (JSON)
  - fee_ap
  - net_amount_real
  - status (pending, processing, completed, rejected)
  - processed_by_admin_id
  - processed_at
  - gateway_reference
  - created_at

  -- Deposit Requests
  deposit_requests
  - id
  - user_id
  - amount_real
  - currency
  - amount_ap (after conversion)
  - exchange_rate
  - method (bank, momo, usdt, manual)
  - source_reference (bank txn id, etc.)
  - proof_url (screenshot if manual)
  - status (pending, verified, completed, rejected)
  - verified_by_admin_id
  - created_at, verified_at

  -- System Balances (Reconciliation)
  system_balance_snapshots
  - id
  - snapshot_date
  - gateway_name
  - real_balance (actual money in gateway)
  - expected_balance (sum of deposits - withdrawals)
  - ap_equivalent
  - difference (for alerts)
  - notes
  - created_at

  Questions:
  - Do you like this structure?
  - Should we separate "buyer wallet" and "seller wallet" into different tables?
  - Do you want audit logs separate from transactions?

  ---
  5. Admin Manual Input System

  You mentioned: "amounts divided into types entered manually by admin, from third-party payment gateways"

  My understanding:
  Admin can manually add/deduct AP for:
  1. Manual deposits (user sent money but auto-detect failed)
  2. Manual withdrawals (send money to user)
  3. Corrections/adjustments
  4. Promotional credits
  5. Penalty deductions

  Questions:
  - Should manual entries require dual approval (Maker-Checker)?
  - Do you want spending limits for admin adjustments?
  - Should all manual entries require proof/receipt upload?

  ---
  6. Reporting & Money Flow Visibility

  You want to see:
  - Seller money flow: earnings, holds, releases, withdrawals
  - Admin system flow: total deposits, withdrawals, system balance
  - User current balance: real-time AP balance

  Proposed Reports:
  1. User Wallet Statement: Like bank statement, all transactions
  2. Seller Earnings Report: Sales, pending, released, withdrawn
  3. System Reconciliation: Gateway balances vs AP issued
  4. Daily Transaction Summary: Deposits, withdrawals, purchases
  5. Escrow Report: All pending releases, aging analysis
  6. Admin Action Audit: All manual interventions

  Questions:
  - Do you need real-time dashboard or daily reports are fine?
  - Should reports be exportable (CSV, Excel)?
  - Do you need webhook notifications for external accounting systems?

  ---
  7. Exness-like Features

  Since you mentioned Exness, here are features they have:

  - Internal Transfer: Move money between accounts
  - Payment History: Detailed filterable history
  - Multiple Payment Methods: Many deposit/withdraw options
  - Auto-conversion: Automatic currency conversion
  - Instant Withdrawals: For verified users
  - Payment System Status: Real-time gateway health

  Questions:
  - Which Exness features are must-have for v2?
  - Do you want internal transfers between users?
  - Should we support multiple currencies or just AP?

  ---
  8. Security & Compliance

  Considerations:
  - 2FA for withdrawals above threshold
  - Daily withdrawal limits per user
  - Suspicious activity detection
  - AML (Anti-Money Laundering) flags
  - Tax reporting (if required)

  Questions:
  - What are your withdrawal limits?
  - Do you need KYC (Know Your Customer) levels?
  - Should we implement velocity checks (too many transactions)?

  ---
  My Proposed Implementation Plan

  Let me know your feedback on the above, then I'll create a detailed implementation plan. Here's the rough outline:

  Phase 1: Core Wallet Infrastructure

  - AP currency system
  - Wallet balances (current + pending)
  - Transaction ledger
  - Basic deposit/withdrawal

  Phase 2: Money Flow Tracking

  - Money flow entries table
  - Gateway integration tracking
  - Reconciliation system
  - Admin manual operations

  Phase 3: Escrow & Release

  - Escrow holds table
  - Auto-release cron job
  - Configurable hold periods
  - Dispute handling

  Phase 4: Reporting & Dashboard

  - User wallet statement
  - Seller earnings report
  - Admin reconciliation dashboard
  - Export capabilities

  Phase 5: Advanced Features

  - Multiple payment gateways
  - Auto-conversion
  - Advanced security
  - Audit trails

  ---
  Questions I Need Answered

  Please answer these so I can refine the design:

  1. AP Definition: What is 1 AP worth? Fixed or floating rate?
  2. Hold Period: Is 3-5 days fixed or varies by order type/amount?
  3. Gateway Integration: Which payment gateways are priority for v2?
  4. Reconciliation: How often do you want to reconcile gateway balances?
  5. Admin Controls: Should manual operations require approval workflow?
  6. User Roles: Do all users have one wallet or separate buyer/seller wallets?
  7. Currency Support: Just VND or multi-currency?
  8. Migration: How to migrate v1 balances to v2?

  Once you answer these, I'll create the detailed technical specification and database schema