# Skill: Brainstorm V2 Features

## When to Use
Khi user yêu cầu "brainstorm" hoặc thảo luận tính năng mới

## Process

### 1. Load V1 Context
Đọc `docs/v1/all.md` hoặc specific feature doc để hiểu:
- Current flow
- Pain points
- Limitations

### 2. Identify Improvements
So sánh V1 với best practices:

| Aspect | V1 | V2 Potential |
|--------|----|----|
| Performance | ? | Caching, async |
| Security | ? | Encryption, audit |
| UX | ? | Better flow |
| Scalability | ? | Event-driven |

### 3. Draw Flow (NOT Code)
```
[User Action] → [System Step] → [Result]
      ↓
  [Branch]
```

### 4. Output Format
```markdown
## Feature: {name}

### V1 Analysis
- Current: ...
- Limitation: ...

### V2 Proposal

#### Flow
[ASCII diagram]

#### Changes
- Point 1
- Point 2

#### Data Changes
```rust
// Chỉ struct định nghĩa, không impl
struct NewEntity {
    field: Type,
}
```

#### Refs
- Related: [link]
```

## DON'T
- ❌ Write full implementation code
- ❌ Quá dài dòng, lý thuyết nhiều
- ❌ Bỏ qua V1 context

## DO
- ✅ Vẽ flow diagram
- ✅ Liệt kê bullet points ngắn gọn
- ✅ Define struct nếu cần
- ✅ Link refs

## Example Topics
- Wallet V2: escrow improvements
- Pre-order: smart matching
- Dispute: auto-resolution rules
- 2FA: product-level authentication
