---
name: brainstorm-p2pmmo-module
description: Use when brainstorming new modules for P2PMMO V2 - reads V1 docs, breaks down into parts, draws workflows, suggests improvements, manages permissions by role
---

# Brainstorming P2PMMO V2 Modules

## Purpose
Help brainstorm new modules for P2PMMO V2 by:
1. Reading V1 documentation for context
2. Drawing workflows for each flow/feature
3. Suggesting improvements from V1
4. Organizing by actor (Buyer, Vendor, Admin)
5. Defining permission codes for each action

## When to Use
- User says "brainstorm [module]" or similar phrases
- Starting work on a new module
- Designing new features for existing modules

## The Process

### Step 1: Read V1 Documentation

Before brainstorming, ALWAYS read the relevant V1 docs:

```bash
# Available V1 docs:
- docs/v1/01-authentication.md
- docs/v1/02-user-roles.md
- docs/v1/03-shop-management.md
- docs/v1/04-products-inventory.md
- docs/v1/05-orders.md
- docs/v1/06-wallet-payment.md
- docs/v1/07-preorder.md
- docs/v1/08-disputes.md
- docs/v1/09-reviews.md
```

Identify:
- What worked well in V1
- What didn't work or was missing
- Pain points users experienced
- Complex flows that could be simplified

### Step 2: Draw Workflows for Each Flow/Feature

For EACH part, create an ASCII workflow diagram showing:

**Example format:**
```
┌─────────────────────────────────────────────────────────────────┐
│                  WALLET - DEPOSIT FLOW                          │
└─────────────────────────────────────────────────────────────────┘

BUYER VIEW:
  ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
  │  BROWSE  │───▶│  SELECT  │───▶│ TRANSFER │───▶│  WAIT    │
  │  WALLET  │    │  METHOD  │    │  MONEY   │    │  WEBHOOK │
  └──────────┘    └──────────┘    └──────────┘    └─────┬────┘
                                                         │
                                                         ▼
                                                  ┌──────────┐
                                                  │ BALANCE  │
                                                  │ UPDATED  │
                                                  └──────────┘

VENDOR VIEW:
  (Not involved)

ADMIN VIEW:
  ┌──────────┐    ┌──────────┐    ┌──────────┐
  │  VIEW    │───▶│ APPROVE  │───▶│ CREDIT   │
  │  PENDING │    │ MANUAL   │    │ WALLET   │
  │ REQUESTS │    │ DEPOSIT  │    │          │
  └──────────┘    └──────────┘    └──────────┘
```

### Step 3: Organize by Actor (Buyer, Vendor, Admin)

For EACH workflow, clearly separate:

**BUYER Actions:**
- What can the buyer do?
- What permissions are needed?
- What data can they see?

**VENDOR Actions:**
- What can the vendor do?
- What permissions are needed?
- What data can they see?

**ADMIN Actions:**
- What can the admin do?
- What permissions are needed?
- What oversight is needed?

### Step 4: Define Permission Codes

For EVERY action in the module, define a permission code following:

```
[MODULE:ACTION]
```

**Examples:**
```
[WALLET:VIEW]          - View wallet balance
[WALLET:DEPOSIT]       - Initiate deposit
[WALLET:WITHDRAW]      - Request withdrawal
[WALLET:TRANSFER]      - Transfer to another user
[WALLET:ADJUST]        - Admin adjust wallet (admin only)
```

**Permission Mapping:**
```rust
// In handler, use:
#[permission_required("WALLET:DEPOSIT")]
async fn deposit_handler(...) -> Result<HttpResponse>

// Reference this code in docs:
// "See [WALLET:DEPOSIT] permission requirements"
```

### Step 5: Suggest Improvements from V1

After analyzing V1, suggest improvements:

**Categories:**
1. **Simplification** - Can we remove steps?
2. **Security** - Can we add protections?
3. **UX** - Can we improve user experience?
4. **Performance** - Can we optimize?
5. **New Features** - What was missing?

**For EACH suggestion, ASK USER:**
```
📌 SUGGESTION: [Brief title]
   V1: [How it worked in V1]
   V2 Proposal: [How it could work]
   Benefit: [Why this is better]
   Complexity: [Low/Medium/High]

   Add this feature? (yes/no/skip for now)
```

### Step 6: Create the Brainstorm Document

Structure the output document:

```markdown
# {Module Name} - Brainstorm & Design

## Overview
[Brief description of what this module does]

## V1 Analysis
### What Worked
- [List features that worked well]

### What Didn't Work
- [List pain points and issues]

### Missing Features
- [List what was missing in V1]

## Flows & Features

### Flow 1: [Feature Name]
**Description:** [What this flow does]

**Workflow:**
```
[ASCII diagram showing data flow]
```

**By Actor:**
- **Buyer:** [What buyer can do]
- **Vendor:** [What vendor can do]
- **Admin:** [What admin can do]

**Permissions:**
- `[MODULE:ACTION]` - [Description]

**Related Files:**
- domain.rs: [Models needed]
- dto.rs: [Request/Response types]
- handler.rs: [HTTP endpoints]
- service.rs: [Business logic]
- repository.rs: [DB operations]

---

[Continue for each flow/feature...]

---

## Permission Matrix

| Action | Permission Code | Buyer | Vendor | Admin |
|--------|----------------|-------|--------|-------|
| View wallet | [WALLET:VIEW] | ✅ | ✅ | ✅ |
| Deposit | [WALLET:DEPOSIT] | ✅ | ✅ | ❌ |
| Withdraw | [WALLET:WITHDRAW] | ❌ | ✅ | ❌ |
| Adjust balance | [WALLET:ADJUST] | ❌ | ❌ | ✅ |

---

## API Endpoints

| Method | Endpoint | Permission | Description |
|--------|----------|------------|-------------|
| GET | /api/wallet | [WALLET:VIEW] | Get wallet balance |
| POST | /api/wallet/deposit | [WALLET:DEPOSIT] | Initiate deposit |

---

## V2 Improvements

### ✅ Approved Changes
1. [Feature description]
2. [Feature description]

### 🔄 Pending Review
1. [Feature awaiting user decision]

### ❌ Rejected
1. [Feature user decided against]

---

## Dependencies

- **Required by:** [Which modules need this]
- **Depends on:** [What this module needs]
- **External services:** [APIs, webhooks, etc.]

---

## Open Questions

1. [Question 1]
2. [Question 2]

---

## Implementation Phases

### Phase 1: Foundation (Domain Models + Repository)
**Priority:** HIGH - Must be done first

**Tasks:**
1. [ ] Create `domain.rs` with all MongoDB models
2. [ ] Create `repository.rs` with basic CRUD operations
3. [ ] Write unit tests for repository
4. [ ] Test database connections and queries

**Dependencies:** None

**Estimated Files:**
- `src/modules/{module}/domain.rs`
- `src/modules/{module}/repository.rs`

---

### Phase 2: Data Transfer Objects (DTOs)
**Priority:** HIGH - Required by handlers and services

**Tasks:**
1. [ ] Create request DTOs (all input structures)
2. [ ] Create response DTOs (all output structures)
3. [ ] Add validation rules
4. [ ] Write tests for DTO validation

**Dependencies:** Phase 1 (domain models)

**Estimated Files:**
- `src/modules/{module}/dto.rs`

---

### Phase 3: Business Logic (Service Layer)
**Priority:** HIGH - Core functionality

**Tasks:**
1. [ ] Implement service functions for each flow
2. [ ] Add error handling
3. [ ] Write unit tests for business logic
4. [ ] Mock repository for testing

**Dependencies:** Phase 1, Phase 2

**Estimated Files:**
- `src/modules/{module}/service.rs`

---

### Phase 4: HTTP Handlers
**Priority:** HIGH - API endpoints

**Tasks:**
1. [ ] Create handler functions for each endpoint
2. [ ] Add permission checks
3. [ ] Add input validation
4. [ ] Write integration tests

**Dependencies:** Phase 2, Phase 3

**Estimated Files:**
- `src/modules/{module}/handler.rs`

---

### Phase 5: Routes Configuration
**Priority:** MEDIUM - Wire everything together

**Tasks:**
1. [ ] Configure all routes
2. [ ] Add middleware (auth, permissions)
3. [ ] Test routing with curl/Postman

**Dependencies:** Phase 4

**Estimated Files:**
- `src/modules/{module}/routes.rs`
- `src/modules/{module}/mod.rs` (export everything)

---

### Phase 6: Integration & Testing
**Priority:** MEDIUM - Ensure everything works

**Tasks:**
1. [ ] Integration tests (full flow)
2. [ ] Load testing (if applicable)
3. [ ] Security testing
4. [ ] Fix bugs found during testing

**Dependencies:** Phase 5

---

### Phase 7: Documentation
**Priority:** LOW - But important

**Tasks:**
1. [ ] Add inline documentation (rustdoc)
2. [ ] Update API docs (OpenAPI/Swagger)
3. [ ] Write usage examples
4. [ ] Update project docs

**Dependencies:** Phase 6

---

## Implementation Order (Within Each Phase)

**For each file, follow this order:**

1. **Structs/Models** - Define data structures first
2. **Implementations** - Add methods
3. **Tests** - Write tests immediately after
4. **Documentation** - Add docs as you code

**Example for Flow: [WALLET:DEPOSIT]**

```
Phase 1 - Foundation:
  ├─ Create Wallet struct (domain.rs)
  ├─ Create Transaction struct (domain.rs)
  ├─ Implement WalletRepository (repository.rs)
  └─ Test: Insert wallet, find by user

Phase 2 - DTOs:
  ├─ Create DepositRequest (dto.rs)
  ├─ Create DepositResponse (dto.rs)
  ├─ Add validation: amount > 0
  └─ Test: Validation rules

Phase 3 - Service:
  ├─ Implement WalletService::deposit (service.rs)
  ├─ Add: Check balance before deposit
  ├─ Add: Create transaction record
  └─ Test: Deposit increases balance

Phase 4 - Handler:
  ├─ Implement deposit_handler (handler.rs)
  ├─ Add: #[permission_required("WALLET:DEPOSIT")]
  ├─ Add: Extract user from JWT
  └─ Test: POST /api/wallet/deposit

Phase 5 - Routes:
  ├─ Add route: POST /api/wallet/deposit (routes.rs)
  ├─ Add middleware: Auth
  └─ Test: Full request flow

Phase 6 - Integration:
  ├─ Test: Deposit → Balance updated
  ├─ Test: Invalid amount → 400 error
  ├─ Test: No permission → 403 error
  └─ Test: Concurrent deposits (race condition)
```

---

## Dependencies Map

```
┌─────────────────────────────────────────────────────────┐
│              MODULE IMPLEMENTATION DEPENDENCIES         │
└─────────────────────────────────────────────────────────┘

                    ┌──────────────┐
                    │   PHASE 1    │
                    │   Domain +   │
                    │  Repository  │
                    └──────┬───────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
        ▼                  ▼                  ▼
  ┌──────────┐      ┌──────────┐      ┌──────────┐
  │  PHASE 2 │      │  PHASE 3 │      │          │
  │   DTOs   │─────▶│ Service  │◀─────│          │
  └─────┬────┘      └─────┬────┘      │          │
        │                  │           │          │
        └──────────────────┼───────────┘          │
                           ▼                       │
                    ┌──────────────┐              │
                    │   PHASE 4    │              │
                    │   Handlers   │              │
                    └──────┬───────┘              │
                           │                      │
                           ▼                      │
                    ┌──────────────┐              │
                    │   PHASE 5    │              │
                    │    Routes    │              │
                    └──────┬───────┘              │
                           │                      │
                           ▼                      │
                    ┌──────────────┐              │
                    │   PHASE 6    │              │
                    │ Integration  │              │
                    └──────┬───────┘              │
                           │                      │
                           ▼                      │
                    ┌──────────────┐              │
                    │   PHASE 7    │◀─────────────┘
                    │    Docs     │
                    └──────────────┘
```

---

## Next Steps

1. [ ] Review and approve design
2. [ ] Create implementation plan
3. [ ] Set up git worktree
4. [ ] Start Phase 1: Foundation
```

### Step 7: Save Document

Save to: `.claude/context/v2/{module}-brainstorm.md`

Also update: `.claude/context/modules.md` with link to new brainstorm.

### Step 8: Iterate

After presenting the brainstorm:
1. Ask: "Does this look right so far?"
2. Be ready to modify any section
3. Re-save as user provides feedback

## Key Principles

1. **ALWAYS read V1 first** - Don't guess, use actual V1 docs
2. **Organize by flows/features** - Not by files (domain, dto, handler...)
3. **Visual workflows** - ASCII diagrams make flows clear
4. **Permission-first** - Define permissions before implementation
5. **Actor separation** - Always think in Buyer/Vendor/Admin terms
6. **Ask before adding** - Never add features without user approval
7. **Reference codes** - Use `[MODULE:ACTION]` everywhere related

## Example: Brainstorming Wallet Module

```bash
User: "brainstorm wallet"

# Skill reads V1 wallet doc
# Skill reads V2 wallet design doc
# Skill identifies flows:
#   - Deposit (bank, momo, usdt, manual)
#   - Withdraw (vendor only)
#   - View balance & history
#   - Admin adjustments

# For EACH flow, draw workflow
# For EACH action, define permission code
# Organize by actor (Buyer, Vendor, Admin)
# Suggest improvements from V1
# ASK user before adding new features
# Save to .claude/context/v2/wallet-brainstorm.md
```

## Output Format

When done, show user:
```
✅ Brainstorm complete!

📄 Document: .claude/context/v2/{module}-brainstorm.md
📊 Flows identified: {count}
🔐 Permissions defined: {count}
🎨 Workflows created: {count}

Next steps:
1. Review the brainstorm document
2. Run: /write-plan {module}  (to create implementation plan)
3. Or: Ask me to start implementation
```

## Troubleshooting

**If V1 doc doesn't exist:**
- Check docs/v1/all.md for overview
- Ask user if this is a brand new feature

**If module is too complex:**
- Break into sub-modules
- Create separate brainstorm for each

**If user rejects suggestions:**
- Move rejected items to "Future Enhancements" section
- Don't delete, just mark as deferred
