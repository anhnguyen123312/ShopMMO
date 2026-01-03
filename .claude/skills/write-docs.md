# Skill: Write Technical Documentation

## Khi nào dùng
- Khi viết feature docs cho `mmo-api/docs/v1/` hoặc `docs/v2/`
- Khi document API endpoints, flows, database schema
- Khi tạo technical specs cho developer + product collaboration

## Format Standard (Reference: `docs/v1/06-wallet-payment.md`)

### 1. Structure Template

```markdown
# [Feature Name] (Vietnamese)

## Tổng quan
- Brief mô tả feature
- Mục đích, scope
- Actors involved (Buyer, Vendor, Admin, System)

---

## 1. [Concept 1]
### 1.1 Sub-concept
- Tables/Lists để so sánh
- ASCII diagrams cho structure

---

## 2. [Main Flow 1]
### 2.1 Conditions/Requirements
```
┌─────────────────────────────────────────┐
│            TITLE BLOCK                  │
└─────────────────────────────────────────┘

1. Condition 1
   ├── Detail 1.1
   └── Detail 1.2

2. Condition 2
   └── Note: ⚠️ Warning
```

### 2.2 Flow
```
┌─────────────────────────────────────────┐
│              FLOW NAME                   │
└─────────────────────────────────────────┘

[Bước 1] Action description
         │
         ▼
[Bước 2] Next action with details
         │
         ├── Option A ──► Result A
         ├── Option B ──► Result B
         │
         ▼
[Bước 3] Continue

─────────────────────────────────────────
[Parallel/Admin Flow]
         │
         ▼
[A1] Admin step 1
```

### 2.3 UI Mockup (nếu applicable)
```
         ╔═════════════════════════════════╗
         ║  UI TITLE                       ║
         ╠═════════════════════════════════╣
         ║  Field: Value                   ║
         ║  ─────────────────────────────  ║
         ║  ⚠️ Warning message             ║
         ║  [Button]                       ║
         ╚═════════════════════════════════╝
```

---

## 5. Flow Drawing Guidelines (QUAN TRỌNG)

### 5.1 Structure của một Flow Section

MỖI flow PHẢI có structure sau:

```
## X. [Tên Flow]

### X.1 Conditions/Requirements (BẮT BUỘC)
┌─────────────────────────────────────────┐
│         ĐIỀU KIỆN THỰC HIỆN            │
└─────────────────────────────────────────┘

1. Preconditions (Trước khi flow bắt đầu)
   ├── Actor: Ai thực hiện? (Buyer/Vendor/Admin/System)
   ├── State: State hiện tại là gì?
   └── Data: Cần data gì?

2. Input Requirements (Dữ liệu đầu vào)
   ├── Required fields
   ├── Validation rules
   └── Constraints (min, max, format)

3. Business Rules (Luật kinh doanh)
   ├── Condition 1
   │   ├── Detail 1.1
   │   └── Detail 1.2
   ├── Condition 2
   └── Note: ⚠️ Warning quan trọng

4. Edge Cases (Các trường hợp đặc biệt)
   ├── Case A ──► Handling A
   ├── Case B ──► Handling B
   └── Case C ──► Ask user (CHỖ NÀY THIẾU INFO)

### X.2 Flow (BẮT BUỘC)
┌─────────────────────────────────────────┐
│              FLOW NAME                   │
└─────────────────────────────────────────┘

[Bước 1] Action description
         │
         ▼
[Bước 2] Next action with details
         │
         ├── Condition A ──► [Bước 3A] Result A
         ├── Condition B ──► [Bước 3B] Result B
         │                   │
         │                   ▼
         │             [Bước 4B] Continue
         │                   │
         │                   └──► Merge back to main flow
         │
         ▼
[Bước 3] Continue (for Condition A)
         │
         ▼
...

─────────────────────────────────────────
[Parallel/Admin Flow]
         │
         ▼
[A1] Admin step 1
```

### 5.2 Decision Tree Rules (BẮT BUỘC)

**MỌI if/else branch PHẢI được vẽ:**

```
✅ CORRECT:
         ├── Valid ──► Continue
         ├── Invalid ──► Return error
         └── Pending ──► Wait for processing

❌ WRONG:
         └── Validate data (không show branches)
```

**Complex decision tree:**

```
         ├── Type = "deposit"
         │    ├── Method = "bank" ──► Flow A
         │    ├── Method = "momo" ──► Flow B
         │    └── Method = "usdt" ──► Flow C
         ├── Type = "withdraw"
         │    ├── Has bank info ──► Continue
         │    └── No bank info ──► Redirect to setup
         └── Type = "adjustment"
              └── Admin only ──► Verify permissions
```

### 5.3 Spacing Rules

```
[Bước 1] Description
         │                          ← 1 dòng connector
         ▼                          ← 1 dòng arrow
[Bước 2] Description
         │
         ├── Branch 1 ──► Result    ← Decision tree
         ├── Branch 2 ──► Result
         │                          ← 1 dòng connector sau branches
         ▼
[Bước 3] Description

───────────────────────────────────── ← Section separator (3+ dashes)
[Parallel Flow]                      ← 1 dòng trống trước section mới
```

### 5.4 UI Mockup Placement

**UI mockup PHẢI đi ngay sau step hiển thị UI:**

```
[Bước 3] Hệ thống hiển thị thông tin:

         ╔═════════════════════════════════╗
         ║  UI TITLE                       ║
         ╠═════════════════════════════════╣
         ║  Field: [Input]                 ║
         ║  ─────────────────────────────  ║
         ║  ⚠️ Warning message             ║
         ║  [Button 1] [Button 2]         ║
         ╚═════════════════════════════════╝
         │
         ▼
[Bước 4] User action
```

**UI Elements Reference:**
```
Button:         [Click me]
Input field:    [________________]
Checkbox:       ☑ Checked  ☐ Unchecked
Radio:          ○ Selected  ☐ Not selected
Copy button:    [📋 Copy]
Warning:        ⚠️ Note
Info:           ℹ️ Info
Separator:      ─────────────────────────────
```

### 5.5 Context Requirements Checklist

TRƯỚC KHI vẽ flow, PHẢI có context sau:

| Context | Required? | Example |
|---------|-----------|---------|
| **Actors** | ✅ Bắt buộc | Buyer, Vendor, Admin, System, Cron |
| **Preconditions** | ✅ Bắt buộc | User logged in, wallet exists |
| **Inputs** | ✅ Bắt buộc | amount, method, bank_info |
| **Outputs** | ✅ Bắt buộc | success/error response |
| **Side Effects** | ✅ Bắt buộc | Update DB, send notification |
| **Error Cases** | ✅ Bắt buộc | Invalid input, insufficient balance |
| **Edge Cases** | ✅ Bắt buộc | Duplicate transaction, timeout |
| **Parallel Flows** | Tùy theo | Admin approval, webhook callback |

**NẾU THIẾU CONTEXT → DỪNG LẠI VÀ HỎI USER:**

```
❌ DON'T: Viết flow với placeholder [TODO: check condition]
❌ DON'T: Assume business logic
❌ DON'T: Skip error handling

✅ DO: Dừng và hỏi:
   "Tôi cần thêm context để vẽ flow này:
    - Nếu user chưa có bank info thì flow như thế nào?
    - Có timeout cho payment không? Timeout thì xử lý sao?
    - Nếu transaction duplicate thì system xử lý ra sao?"
```

### 5.6 Edge Cases Documentation

Sau mỗi flow, PHẢI có section edge cases:

```
### X.3 Edge Cases & Error Handling

| Case | Condition | Handling | User Message |
|------|-----------|----------|--------------|
| Insufficient balance | balance < amount | Reject with 400 | "Số dư không đủ" |
| Duplicate transaction | Same ref + time < 5min | Ignore | - |
| Timeout | No response in 30s | Mark as failed | "Giao dịch timeout" |
| Invalid bank info | Invalid account number | Return error | "Thông tin ngân hàng không hợp lệ" |
```

### 5.7 Admin Flow Convention

Admin flows PHẢI dùng prefix [A1], [A2], [A3]...:

```
─────────────────────────────────────────
[Admin Flow - Xử lý yêu cầu]
         │
         ▼
[A1] Admin vào panel
         │
         ├── Approve ──► [A2] Process payment
         ├── Reject ──► [A3] Refund user
         └── Need info ──► [A4] Request more info
```

---

## 8. Components Reference

### ASCII Art Box Characters
```
Horizontal: ─ │
Vertical: │
Corners: ┌ ┐ └ ┘
T-junctions: ├ ┤ ┬ ┴
Cross: ┼
Double box: ╔ ═ ╗ ║ ╚ ╝
```

### Flow Indicators
```
Sequential:     │
Decision tree:  ├── Option ──► Result
Down arrow:     ▼
Section break:  ─────────────────────────────────────
```

### UI Elements
```
Button:         [Click me]
Input field:    [________________]
Checkbox:       ☑ Checked  ☐ Unchecked
Radio:          ○ Selected  ☐ Not selected
Copy button:    [📋 Copy]
Warning:        ⚠️ Note
Info:           ℹ️ Info
```

### Emoji Legend (optional)
```
🟢 Positive/Success  (deposit, refund, commission)
🔴 Negative/Deduct   (purchase, withdraw)
🔵 Neutral/Refund
🟡 Processing/Pending
```

---

## 9. Writing Conventions

### Language
- **Tiếng Việt**: Cho descriptions, flows, explanations
- **English**: Cho code, field names, API endpoints, technical terms
- **Mixed**: Variable names, table names = English; context = Vietnamese

### Code Blocks
- SQL: `sql` language
- JSON/Response: `json` language
- Request/Response examples: use realistic data

### Numbering
- Level 1: `## 1.`, `## 2.`, ...
- Level 2: `### 1.1`, `### 2.1`, ...
- Steps: `[Bước 1]`, `[Bước 2]`, ...
- Admin steps: `[A1]`, `[A2]`, ...

### Tables
- Use markdown tables với headers
- Alignment: left align default
- Status tables quan trọng cho state machines

---

## 10. What to Include

### For Feature Docs
1. **Tổng quan**: Why, what, who
2. **Data Structure**: Tables, entities, relationships
3. **Flows**: Step-by-step với decision trees
4. **Database**: Full SQL schema
5. **Edge Cases**: Error handling, validation
6. **Admin Features**: Nếu có admin panel

### For API Docs
1. Endpoint table
2. Request/Response examples
3. Error codes
4. Authentication requirements

---

## 11. Reference System (MANDATORY)

### 11.1 What is the Reference System?

Reference system cho phép:
- **Link code locations**: Trỏ đến file, function, line cụ thể
- **Cross-document links**: Reference docs khác
- **Dependency tracking**: Track dependencies giữa features
- **Impact analysis**: Biết docs nào bị ảnh hưởng khi code thay đổi

### 11.2 Reference Format (STRICT - PHẢI TUÂN THỦ)

**Format cơ bản:**
```
[{TYPE}{ID}@[FILE]?{SECTION}?{LINE}]
```

**QUAN TRỌNG: MỌI reference PHẢI match với regex patterns trong `docs/refs/_types.yaml`**

**Reference Types:**

| Type | Format | Scope | Example | Description |
|------|--------|-------|---------|-------------|
| `[F{num}]` | Flow Step | Local | `[F1]`, `[F2]` | Flow step trong doc hiện tại |
| `[A{num}]` | Admin Step | Local | `[A1]`, `[A2]` | Admin flow step trong doc hiện tại |
| `[FILE@...]` | Code File | Global | `[FILE@wallet:service.rs:45]` | Trỏ đến line cụ thể trong code |
| `[FN@...]` | Function | Global | `[FN@wallet:service.rs::create_deposit]` | Trỏ đến function |
| `[FIELD@...]` | Field | Global | `[FIELD@Wallet.balance]` | Trỏ đến field trong domain |
| `[API@...]` | API Endpoint | Global | `[API@POST /api/v1/wallet/deposit]` | Trỏ đến API endpoint |
| `[DOC@...]` | Document | Global | `[DOC@v2/wallet/deposit:admin-flow]` | Trỏ đến section trong doc khác |
| `[DEP@...]` | Dependency | Global | `[DEP@wallet]` | Trỏ đến feature dependency |
| `[TABLE@...]` | Table | Global | `[TABLE@wallets]` | Trỏ đến database collection |

### 11.3 Type Format Validation Rules (STRICT)

**MỌI reference PHẢI tuân thủ format sau (định nghĩa trong `docs/refs/_types.yaml`):**

```yaml
# Copy từ docs/refs/_types.yaml - PHẢI SYNC khi update
regex_patterns:
  # Local references
  flow_step:     "\\[F([0-9]+)\\]"           # [F1], [F2], [F123]
  admin_step:    "\\[A([0-9]+)\\]"           # [A1], [A2], [A123]

  # Code references
  file_ref:      "\\[FILE@([a-z_]+):([a-z_]+\\.rs):([0-9]+)\\]"
  fn_ref:        "\\[FN@([a-z_]+):([a-z_]+\\.rs)::([a-z_]+)\\]"
  field_ref:     "\\[FIELD@([A-Z][a-zA-Z0-9]*)\\.([a-z_]+)\\]"

  # API & Docs
  api_ref:       "\\[API@(GET|POST|PUT|DELETE|PATCH) ([^\\]]+)\\]"
  doc_ref:       "\\[DOC@([^:]+):([^#]+)#?([^\\]]*)\\]"
  dep_ref:       "\\[DEP@([a-z_]+)\\]"
  table_ref:     "\\[TABLE@([a-z_]+)\\]"
```

**Validation Rules (BẮT BUỘC):**

| Rule | Pattern | Valid | Invalid |
|------|---------|-------|---------|
| Flow step | `[F{number}]` | `[F1]`, `[F23]` | `[f1]`, `[F]`, `[F01]` |
| Admin step | `[A{number}]` | `[A1]`, `[A5]` | `[a1]`, `[A]` |
| Field ref | `[FIELD@{Struct}.{field}]` | `[FIELD@Wallet.balance]` | `[Field@wallet.balance]`, `[FIELD@wallet.Balance]` |
| Function ref | `[FN@{module}:{file}::{fn}]` | `[FN@wallet:service.rs::create]` | `[fn@wallet:service.rs::create]` |
| API ref | `[API@{METHOD} {path}]` | `[API@POST /api/v1/wallet]` | `[api@POST /wallet]` |
| Dep ref | `[DEP@{feature}]` | `[DEP@wallet]` | `[dep@Wallet]` |

**Naming Conventions:**
- **Struct names**: PascalCase (`Wallet`, `User`, `Transaction`)
- **Field names**: snake_case (`balance`, `is_active`, `user_id`)
- **Module names**: snake_case (`wallet`, `auth`, `shop_management`)
- **Function names**: snake_case (`create_deposit`, `verify_user`)
- **Collection names**: snake_case, plural (`wallets`, `users`, `transactions`)

**Pre-Writing Checklist - Reference Validation:**

```
- [ ] Trước khi viết docs, load docs/refs/_types.yaml
- [ ] MỌI reference PHẢI match regex pattern
- [ ] Struct names = PascalCase
- [ ] Field/module/function names = snake_case
- [ ] File paths = .rs extension
- [ ] API methods = UPPERCASE (GET, POST, PUT, DELETE, PATCH)
- [ ] KHÔNG tạo custom reference format
- [ ] Nếu cần format mới → PHẢI update _types.yaml trước
```

### 11.4 Reference Declaration (DECLARE:REF)

KHI TẠO reference mới trong flow, PHẢI declare:

**NOTE: MỌI reference trong declaration PHẢI tuân thủ validation rules ở 11.3**

```
[DECLARE:REF:F1]
name: "Create deposit request"
location:
  file: src/modules/wallet/service.rs
  function: create_deposit_request
  line: 23
depends_on:
  - [FIELD@User.id]
  - [FIELD@Wallet.id]
affects:
  - [FIELD@Deposit.status]
related_docs:
  - docs/v2/wallet/wallet.md
```

### 11.4 Using References in Flows

**Example:**

```
┌─────────────────────────────────────────┐
│         DEPOSIT FLOW                    │
└─────────────────────────────────────────┘

[Bước 1] User creates deposit
         │
         ├── [REF:DEP@auth]  # Requires auth
         │
         ├── Logged in ──► Continue
         └── Not logged ──► [REF:API@POST /api/v1/auth/login]
         │
         ▼
[Bước 2] Validate amount
         │
         ├── [REF:FIELD@Wallet.min_deposit]
         ├── [REF:FIELD@Wallet.max_deposit]
         │
         ├── Valid ──► [REF:F3]
         └── Invalid ──► Return error
         │
         ▼
[Bước 3] Create deposit transaction
         │
[DECLARE:REF:F3]
name: "Create deposit transaction"
location:
  file: src/modules/wallet/service.rs
  function: create_deposit
  line: 45
depends_on:
  - [FIELD@User.id]
  - [FIELD@Wallet.id]
affects:
  - [FIELD@Deposit.status]
  - [TABLE@transactions]
         │
         ▼
[ADMIN PARALLEL]
         │
         ▼
[A1] Admin sees pending deposit
         │
[DECLARE:REF:A1]
location:
  file: src/modules/wallet/handler.rs
  function: list_pending_deposits
  line: 120
         │
         ├── [REF:DEP@authorization]
         │
         ├── Approve ──► [REF:A2]
         └── Reject ──► [REF:A3]
```

### 11.5 Using References in Flows

**Example - Flow with proper reference formatting:**

```
┌─────────────────────────────────────────┐
│         DEPOSIT FLOW                    │
└─────────────────────────────────────────┘

[Bước 1] User creates deposit
         │
         ├── [REF:DEP@auth]  # Requires auth
         │
         ├── Logged in ──► Continue
         └── Not logged ──► [REF:API@POST /api/v1/auth/login]
         │
         ▼
[Bước 2] Validate amount
         │
         ├── [REF:FIELD@Wallet.min_deposit]
         ├── [REF:FIELD@Wallet.max_deposit]
         │
         ├── Valid ──► [REF:F3]
         └── Invalid ──► Return error
         │
         ▼
[Bước 3] Create deposit transaction
         │
[DECLARE:REF:F3]
name: "Create deposit transaction"
location:
  file: src/modules/wallet/service.rs
  function: create_deposit
  line: 45
depends_on:
  - [FIELD@User.id]
  - [FIELD@Wallet.id]
affects:
  - [FIELD@Deposit.status]
  - [TABLE@transactions]
         │
         ▼
[ADMIN PARALLEL]
         │
         ▼
[A1] Admin sees pending deposit
         │
[DECLARE:REF:A1]
location:
  file: src/modules/wallet/handler.rs
  function: list_pending_deposits
  line: 120
         │
         ├── [REF:DEP@authorization]
         │
         ├── Approve ──► [REF:A2]
         └── Reject ──► [REF:A3]
```

### 11.6 Pre-Writing Context Check (MANDATORY)

TRƯỚC KHI viết docs, agent PHẢI:

1. **Load `docs/refs/_types.yaml`** - Verify reference format patterns
2. **Load feature manifest** từ `docs/refs/features/{feature}.yaml`
3. **Resolve dependencies** - Load manifests của dependencies
4. **Build context map**
5. **SHOW context to user** trước khi viết

**Context Output Format:**

```
╔═══════════════════════════════════════════════╗
║  FEATURE CONTEXT: {feature_name}              ║
╠═══════════════════════════════════════════════╣
║  Type: {feature_type} (resource/transaction/..)║
║  Version: {v1/v2}                              ║
╠═══════════════════════════════════════════════╣
║  REQUIRED DEPENDENCIES:                        ║
║  ├─ auth (verify ownership)                    ║
║  │   └─ [FN@auth:service.rs::verify_user]     ║
║  ├─ user_profile (get user info)               ║
║  │   └─ [FIELD@User.id]                       ║
║  └─ wallet (check balance)                     ║
║      └─ [FIELD@Wallet.balance]                ║
╠═══════════════════════════════════════════════╣
║  AFFECTS FEATURES:                             ║
║  ├─ order (escrow logic) [BLOCKING]            ║
║  │   └─ [FN@wallet:service.rs::lock_funds]    ║
║  └─ shop (commission) [CRITICAL]               ║
║      └─ [FN@shop:service.rs::calculate_commission]║
╠═══════════════════════════════════════════════╣
║  RELATED DOCS:                                 ║
║  ├─ docs/v2/wallet/escrow.md                   ║
║  └─ docs/v1/06-wallet-payment.md               ║
╠═══════════════════════════════════════════════╣
║  CODE FILES:                                   ║
║  ├─ src/modules/wallet/domain.rs              ║
║  ├─ src/modules/wallet/service.rs             ║
║  └─ src/modules/wallet/handler.rs             ║
╚═══════════════════════════════════════════════╝

Continue to write docs? (y/n)
```

### 11.7 Reference Resolution Priority

Khi agent encounter `[REF:...]`, resolution theo thứ tự:

1. **Local declaration** - Check `[DECLARE:REF:...]` trong current doc
2. **Local registry** - Check `{doc}._refs.yaml`
3. **Global registry** - Check `docs/refs/_registry.yaml`
4. **Feature manifest** - Check `docs/refs/features/{name}.yaml`
5. **Code search** - Fallback: search trong codebase

### 11.8 Creating Per-Document Reference Registry

SAU KHI viết docs xong, PHẢI tạo `{doc}._refs.yaml`:

```yaml
# docs/v2/wallet/deposit._refs.yaml
document: docs/v2/wallet/deposit.md
version: v2.0
updated_at: 2026-01-04

local_refs:
  F1:
    name: "Create deposit request"
    location:
      file: src/modules/wallet/service.rs
      function: create_deposit_request
      line: 23
    depends_on:
      - [FIELD@User.id]
      - [FIELD@Wallet.id]
    affects:
      - [FIELD@Deposit.status]

external_refs:
  - [DEP@auth]
  - [DEP@authorization]
  - [FIELD@Wallet.balance]
  - [API@POST /api/v1/wallet/deposit]

related_features:
  - payment
  - authorization
```

### 11.9 Reference Examples

**Cross-document reference:**
```markdown
[Bước 2] Verify user has wallet
         │
         ├── [REF:DOC@v2/wallet/wallet.md#create-wallet]
         │
         ├── Has wallet ──► Continue
         └── No wallet ──► [REF:F3] Create wallet
```

**API endpoint reference:**
```markdown
[Bước 1] User submits deposit request via [API@POST /api/v1/wallet/deposit]
         │
         ▼
[Bước 2] Handler validates request [FN@wallet:handler.rs::create_deposit]
```

**Field impact tracking:**
```markdown
⚠️ IMPORTANT: This step affects [FIELD@Wallet.balance]
Any change to this field will impact:
  - Order placement (check balance)
  - Purchase flow (deduct balance)
  - Withdrawal (check available balance)
```

---

## 7. Checklist (Flow Completeness)

### 7.1 Pre-Writing Checklist
TRƯỚC KHI BẮT ĐẦU VIẾT, verify:

- [ ] **Context collected**: Actors, preconditions, inputs, outputs
- [ ] **Business rules clarified**: All conditions documented
- [ ] **Edge cases identified**: Error paths defined
- [ ] **Parallel flows known**: Admin/webhook/async flows
- [ ] **UI requirements**: Screens, fields, buttons

### 7.2 Flow Structure Checklist
MỖI flow PHẢI có:

- [ ] **Conditions/Requirements section**:
  - [ ] Actors specified
  - [ ] Preconditions listed
  - [ ] Input requirements (fields, validation, constraints)
  - [ ] Business rules (all conditions with tree structure)
  - [ ] Edge cases listed (ask user if missing)

- [ ] **Flow section**:
  - [ ] Title block with flow name
  - [ ] Sequential steps ([Bước 1], [Bước 2], ...)
  - [ ] ALL decision trees drawn (no "validate data" without branches)
  - [ ] Arrow connectors (│, ▼)
  - [ ] Section separators for parallel flows
  - [ ] Admin steps marked ([A1], [A2], ...)

- [ ] **UI Mockups** (if applicable):
  - [ ] ASCII box format (╔═╗ ║ ╚╝)
  - [ ] All fields labeled
  - [ ] Buttons shown
  - [ ] Warnings/info messages included

- [ ] **Edge Cases section**:
  - [ ] Table format (Case | Condition | Handling | Message)
  - [ ] All error paths covered
  - [ ] User messages in Vietnamese

### 7.3 Final Verification
Khi hoàn thành docs, verify:

- [ ] Structure follows template
- [ ] Flows có decision trees (KHÔNG skip branches)
- [ ] Conditions/Requirements section complete
- [ ] SQL schema complete (FK, indexes, constraints)
- [ ] UI mockups clear (nếu applicable)
- [ ] Error cases covered trong table
- [ ] Language consistent (VN descriptions, EN technical)
- [ ] No broken ASCII art
- [ ] Tables properly formatted
- [ ] Context không bị thiếu (nếu thiếu thì đã hỏi user)
