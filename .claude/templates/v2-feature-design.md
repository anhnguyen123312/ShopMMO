# Template: V2 Feature Design

Copy this template when designing new V2 features.

---

# Feature: {Feature Name}

## Overview
1-2 sentences mô tả feature.

## V1 Reference
- Doc: [docs/v1/{file}.md](../../docs/v1/{file}.md)
- Current flow: brief summary
- Limitations: what needs improvement

## V2 Goals
- [ ] Goal 1
- [ ] Goal 2

## Flow Diagram
```
[Start] → [Step 1] → [Step 2] → [End]
              ↓
         [Branch]
```

## Data Models

### New/Modified Entities
```rust
struct Entity {
    id: ObjectId,
    // fields
    created_at: DateTime,
}
```

### Database Indexes
```
Collection: entities
- { field: 1 } unique
- { created_at: -1 }
```

## API Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | /path | Bearer | ... |

## Business Rules
1. Rule 1
2. Rule 2

## Error Cases
| Code | When |
|------|------|
| 400 | Invalid input |
| 404 | Not found |

## Implementation Phases
1. **Phase 1**: Core models + basic CRUD
2. **Phase 2**: Business logic
3. **Phase 3**: Edge cases + optimization

## Refs
- Related modules: [link]
- External deps: [link]

## Notes
Any additional considerations.

---

**Save to**: `docs/v2/{feature-name}.md`
