# Lessons Learned

## Format
```
### [Date] - Topic
- What: brief description
- Learned: key insight
- Apply: how to use next time
```

---

## Rust + Actix

### 2024-12 - Module Structure
- What: Setup module architecture
- Learned: Strict separation Handler→Service→Repository prevents circular deps
- Apply: Always create files in order: domain → dto → repo → service → handler

### 2024-12 - Error Handling
- What: Unified error chain
- Learned: DbError → ServiceError → ApiError với `From` impl cho clean conversion
- Apply: Never expose internal errors to API response

### 2024-12 - MongoDB ObjectId
- What: ObjectId serialization
- Learned: Use `#[serde(rename = "_id")]` và `skip_serializing_if = "Option::is_none"`
- Apply: Template trong domain.rs

---

## Business Logic

### 2024-12 - Escrow 3-day Hold
- What: V1 payment release mechanism
- Learned: Hold period starts từ order completion, not creation
- Apply: Track `completed_at` separately from `created_at`

### 2024-12 - Duplicate Item Check
- What: Prevent selling same digital item twice
- Learned: Hash content và check global, không chỉ per-product
- Apply: Create unique index on content hash

---

## Research

### 2024-12 - TaphoaMMO V1 Analysis
- What: Competitive research
- Learned: 4 roles (Buyer/Vendor/Reseller/Admin), text-based inventory, auto-delivery
- Apply: Documented in docs/v1/

---

## To Add
Sau mỗi task hoàn thành, thêm entry mới theo format trên.
Keep it SHORT và actionable.
