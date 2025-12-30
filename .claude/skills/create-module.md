# Skill: Create New Module

## When to Use
Khi cần tạo module mới trong `src/modules/`

## Pre-flight Checklist
1. ✅ Check V1 docs: `docs/v1/{feature}.md`
2. ✅ Check existing context: `.claude/context/{module}.md`
3. ✅ Review related modules for patterns

## Steps

### 1. Create Module Directory
```bash
mkdir -p src/modules/{module_name}
```

### 2. Create Files (Order Matters)
```
1. domain.rs   # Data models first
2. dto.rs      # API contracts
3. repository.rs
4. service.rs
5. handler.rs
6. routes.rs
7. mod.rs
```

### 3. Domain Template
```rust
// src/modules/{module}/domain.rs
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Primary entity for {module}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    
    // fields...
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Entity {
    pub const COLLECTION: &'static str = "entities";
}
```

### 4. DTO Template
```rust
// src/modules/{module}/dto.rs
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateRequest {
    #[validate(length(min = 1, max = 100))]
    pub field: String,
}

#[derive(Debug, Serialize)]
pub struct Response {
    pub id: String,
    // response fields
}
```

### 5. Register Module
```rust
// src/modules/mod.rs
pub mod {module_name};

// src/main.rs - add routes
.configure(modules::{module_name}::routes::configure)
```

## Post-flight
1. Create context file: `.claude/context/{module}.md`
2. Update lessons if learned something new
3. Test endpoints work

## Refs
- Pattern: [context/auth.md](../context/auth.md) (reference impl)
- Standards: [docs/CODING_STANDARDS.md](../../docs/CODING_STANDARDS.md)
