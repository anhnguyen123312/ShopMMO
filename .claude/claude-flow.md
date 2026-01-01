# Claude API Development Flow

## Quy trình phát triển API tuân thủ coding standards

---

## PHASE 1: PLANNING & USER VALIDATION

### 1.1. Đọc context trước
```
1. Check .claude/context/{module}.md nếu có
2. Đọc docs/v1/{feature}.md để hiểu V1 flow
3. Đọc docs/CODING_STANDARDS.md
4. Đọc docs/ARCHITECTURE.md
```

### 1.2. Brainstorm với user (sử dụng skill `superpowers:brainstorming`)
```
- Yêu cầu: "Tôi cần implement feature X"
- Claude PHẢI dùng brainstorming skill TRƯỚC
- Output: Feature design, flow diagrams, edge cases
```

### 1.3. Liệt kê APIs cần implement

**Template:**
```markdown
## APIs cần implement:

| Method | Endpoint | Description | Request | Response | Auth |
|--------|----------|-------------|---------|----------|------|
| POST | /api/xxx | Create xxx | CreateXxxRequest | XxxResponse | User |
| GET | /api/xxx/:id | Get xxx by id | - | XxxResponse | User |
| ... | ... | ... | ... | ... | ... |
```

### 1.4. Hỏi user confirm trước khi code
```
Questions:
1. Authentication requirement? (None, User, Admin)
2. Validation rules?
3. Error handling specifics?
4. Business logic edge cases?
5. Database transaction needs?
```

---

## PHASE 2: IMPLEMENTATION

### 2.1. Structure tuân thủ STRICT
```
src/modules/{module}/
├── domain.rs     # MongoDB models + ToSchema cho enums/simple structs
├── dto.rs        # Request/Response + ToSchema (KHÔNG domain types với ObjectId/BsonDateTime)
├── handler.rs    # HTTP handlers + utoipa::path annotations
├── service.rs    # Business logic
├── repository.rs # DB operations
└── routes.rs     # Route config
```

### 2.2. Coding Rules (CRITICAL)

#### Rule 1: ToSchema compatibility
```rust
// ❌ WRONG - BsonDateTime/ObjectId không có ToSchema
#[derive(ToSchema)]
pub struct Wallet {
    pub id: ObjectId,        // ❌
    pub created_at: BsonDateTime, // ❌
}

// ✅ CORRECT - Chỉ ToSchema cho enums và simple structs
#[derive(ToSchema)]
pub enum WalletStatus {
    Active,
    Frozen,
}

// ✅ CORRECT - Domain structs KHÔNG có ToSchema
#[derive(Serialize, Deserialize)]  // KHÔNG ToSchema
pub struct Wallet {
    pub id: ObjectId,
    pub created_at: BsonDateTime,
}

// ✅ CORRECT - DTOs có ToSchema và KHÔNG chứa domain types
#[derive(ToSchema)]
pub struct WalletResponse {
    pub wallet_id: String,     // ✅ String thay vì ObjectId
    pub created_at: String,    // ✅ ISO string thay vì BsonDateTime
}
```

#### Rule 2: Domain → DTO mapping
```rust
// Trong handler/service, map domain → DTO
impl From<Wallet> for WalletResponse {
    fn from(wallet: Wallet) -> Self {
        Self {
            wallet_id: wallet.wallet_id,
            created_at: wallet.created_at.to_rfc3339(), // Convert BsonDateTime → String
        }
    }
}
```

#### Rule 3: Swagger annotations
```rust
/// POST /api/wallet/create - Create wallet
#[utoipa::path(
    post,
    path = "/api/wallet/create",
    tag = "Wallet",
    request_body = CreateWalletRequest,
    responses(
        (status = 200, description = "Wallet created", body = ApiResponse<WalletResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = [])
)]
pub async fn create_wallet(...) -> Result<HttpResponse, ApiError> {
    // ...
}
```

#### Rule 4: Error chain
```
DbError → ServiceError → ApiError
```

---

## PHASE 3: SWAGGER DOCUMENTATION

### 3.1. Checklist ToSchema

**Domain types (`domain.rs`):**
- ✅ Enums: `ToSchema` OK
- ✅ Simple structs (chỉ chứa primitives): `ToSchema` OK
- ❌ Structs với `ObjectId`/`BsonDateTime`: KHÔNG `ToSchema`

**DTOs (`dto.rs`):**
- ✅ Tất cả DTOs: `ToSchema` (nhưng KHÔNG chứa domain types với ObjectId/BsonDateTime)
- ✅ Use `String` thay vì `ObjectId`
- ✅ Use `String` (ISO format) thay vì `BsonDateTime`

### 3.2. Update openapi.rs
```rust
#[openapi(
    paths(
        // Add tất cả handler functions
        crate::modules::wallet::handler::create_wallet,
        // ...
    ),
    components(
        schemas(
            // Add DTOs
            crate::modules::wallet::dto::WalletResponse,
            // Add domain enums/simple structs
            crate::modules::wallet::domain::WalletStatus,
            // KHÔNG add domain structs với ObjectId/BsonDateTime
        )
    )
)]
```

---

## PHASE 4: RUN & DEBUG

### 4.1. Build check
```bash
cargo build 2>&1 | grep "error\[" | head -20
```

### 4.2. Common errors & fixes

| Error | Cause | Fix |
|-------|-------|-----|
| `the trait bound 'bson::oid::ObjectId: ToSchema' is not satisfied` | Domain struct với ObjectId có ToSchema | Remove ToSchema từ struct |
| `the trait bound 'wallet::domain::Transaction: ToSchema' is not satisfied` | DTO chứa domain type | Remove domain type, tạo DTO field |
| `securitySchemes not valid inside components()` | utoipa 5.x syntax | Use `Modify` trait |

### 4.3. Runtime check
```bash
# Start server
cargo run

# Check Swagger UI
curl http://localhost:8080/api/doc/openapi.json | jq '.paths | keys'
```

### 4.4. Debug systematic (dùng `superpowers:systematic-debugging`)
```
1. Phase 1: Root Cause Investigation
   - Read error messages COMPLETELY
   - Reproduce consistently
   - Check recent changes

2. Phase 2: Pattern Analysis
   - Find working examples
   - Compare differences

3. Phase 3: Hypothesis & Testing
   - Single hypothesis
   - Minimal change
   - Verify before continuing

4. Phase 4: Implementation
   - Create failing test case
   - Fix root cause ONLY
   - No "while I'm here" changes
```

---

## PHASE 5: VERIFICATION

### 5.1. Pre-completion checklist
```bash
# 1. Build succeeds
cargo build

# 2. Swagger accessible
curl -f http://localhost:8080/api/doc/openapi.json

# 3. All endpoints documented
curl http://localhost:8080/api/doc/openapi.json | jq '.paths | keys | length'

# 4. No ToSchema errors
cargo build 2>&1 | grep -i "toschema" | grep "error"
```

### 5.2. Update documentation
```bash
# Update context file
.vim .claude/context/{module}.md

# Update lessons learned
vim .claude/lessons.md
```

---

## EXAMPLE FLOW: Implement Wallet API

### Step 1: Planning
```
Claude: "Dùng brainstorming skill để design wallet system..."
Output:
- Wallet types (User, Seller, Platform)
- Balance states (available, locked, escrow)
- Transaction types (deposit, withdrawal, purchase)
```

### Step 2: Ask user
```
Claude: "Tôi sẽ implement:
1. POST /api/wallet/create - Create wallet
2. GET /api/wallet/balance - Get balance
3. POST /api/wallet/deposit - Deposit money
4. POST /api/wallet/withdrawal - Request withdrawal
5. POST /api/wallet/purchase - Purchase with escrow

Confirm?"
```

### Step 3: Code domain.rs
```rust
// Enums với ToSchema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum WalletStatus { Active, Frozen, }

// Structs KHÔNG ToSchema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub id: Option<ObjectId>,  // No ToSchema
    pub wallet_id: String,
    pub balance: i64,
}
```

### Step 4: Code dto.rs
```rust
// All DTOs với ToSchema
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateWalletRequest {
    pub wallet_type: WalletType,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WalletResponse {
    pub wallet_id: String,  // String thay vì ObjectId
    pub balance: i64,
}
```

### Step 5: Code handler.rs với utoipa
```rust
#[utoipa::path(
    post,
    path = "/api/wallet/create",
    tag = "Wallet",
    request_body = CreateWalletRequest,
    responses(
        (status = 200, description = "Wallet created", body = ApiResponse<WalletResponse>),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_wallet(...) -> Result<HttpResponse, ApiError> {
    // ...
}
```

### Step 6: Update openapi.rs
```rust
#[openapi(
    paths(
        crate::modules::wallet::handler::create_wallet,
    ),
    components(
        schemas(
            crate::modules::wallet::dto::WalletResponse,
            crate::modules::wallet::domain::WalletStatus,
        )
    )
)]
```

### Step 7: Build & verify
```bash
cargo build
# Fix ToSchema errors
cargo build
# Verify Swagger
curl http://localhost:8080/api/doc/openapi.json
```

---

## CRITICAL REMINDERS

1. **ALWAYS brainstorm before coding**
2. **NEVER add ToSchema to structs with ObjectId/BsonDateTime**
3. **ALWAYS map domain → DTO in handlers**
4. **NEVER skip utoipa::path annotations**
5. **ALWAYS verify build succeeds before claiming done**
6. **USE systematic debugging skill when errors occur**

---

## Flow Decision Tree

```
User request API
    ↓
Invoke brainstorming skill
    ↓
Present API list to user
    ↓
User confirms?
    ↓ YES
Plan implementation (files, types, flow)
    ↓
Code domain.rs (no ToSchema on ObjectId structs)
    ↓
Code dto.rs (all ToSchema, no domain types with ObjectId)
    ↓
Code handler.rs (with utoipa::path)
    ↓
Update openapi.rs
    ↓
Build: cargo build
    ↓
Errors?
    ↓ YES
Use systematic-debugging skill
    ↓ NO
Verify Swagger UI
    ↓
DONE
```
