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
