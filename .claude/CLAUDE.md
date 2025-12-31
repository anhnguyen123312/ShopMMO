#NAME: `P2PMMO`
# P2PMMO V2 - Claude Rules

## Project Overview
Digital marketplace API - Rust + MongoDB + Redis
**Refs**: [docs/v1/all.md](../docs/v1/all.md), [docs/ARCHITECTURE.md](../docs/ARCHITECTURE.md)

## Tech Stack
| Component | Tech |
|-----------|------|
| Language | Rust 1.75+ (Edition 2021) |
| Web | actix-web 4.9 |
| DB | MongoDB 4.4+ |
| Cache | Redis 6.0+ |
| Auth | JWT (jsonwebtoken) |

## Architecture Pattern
```
Handler → Service → Repository → Database
   ↓         ↓           ↓
  DTO    Domain      MongoDB
```

## Module Structure (STRICT)
```
src/modules/{module}/
├── mod.rs           # Module exports
├── domain.rs        # MongoDB models ONLY
├── dto.rs           # Request/Response ONLY  
├── handler.rs       # HTTP handlers ONLY
├── service.rs       # Business logic ONLY
├── repository.rs    # DB operations ONLY
└── routes.rs        # Route config
```

## Error Chain
```
DbError → ServiceError → ApiError
```

## When Working on Module

### Before Coding
1. Check context trong `.claude/context/{module}.md`
2. Read V1 feature docs: `docs/v1/{feature}.md`
3. Check refs với các module liên quan

### After Completing
1. Update `.claude/context/{module}.md`
2. Ghi lessons learned vào `.claude/lessons.md`
3. Update refs nếu có breaking changes

## Brainstorm V2 Features
Khi cần brainstorm, luôn:
1. Tham khảo `docs/v1/all.md` để hiểu flow V1
2. So sánh và đề xuất improvements
3. Vẽ flow mới, không code chi tiết

## Coding Standards Quick Ref
- Validate input ở handler
- Business rules ở service
- KHÔNG validate ở repository
- Mọi public fn phải có doc comments
- Complex logic phải có inline comments

## Key Refs Map
| Topic | File |
|-------|------|
| All V1 Features | docs/v1/all.md |
| Auth Flow | docs/v1/01-authentication.md |
| User Roles | docs/v1/02-user-roles.md |
| Products | docs/v1/04-products-inventory.md |
| Wallet V1 | docs/v1/06-wallet-payment.md |
| Wallet V2 Design | docs/v2/01-wallet-system-design.md |
| Architecture | docs/ARCHITECTURE.md |
| Coding Standards | docs/CODING_STANDARDS.md |
