# Category Module - Brainstorm & Design

## Overview
Quản lý danh mục sản phẩm cho P2PMMO V2 - Category system với per-category inventory collections, hierarchy support, và commission rates.

## V2 Design Philosophy

### Key Design Decisions
1. **Per-Category Inventory Collections**: Mỗi category có collection riêng trong MongoDB
   - Collection naming: `inventory_{category_slug}`
   - Ví dụ: `inventory_netflix_accounts`, `inventory_spotify_accounts`

2. **Category Hierarchy**: Hỗ trợ parent → child categories
   - Ví dụ: Streaming → Netflix, Spotify, Disney+

3. **Commission Rates**: Mỗi category có commission rate riêng
   - Admin có thể set commission % theo category
   - Mặc định: 5-10%

4. **Inventory Collection Management**:
   - Khi tạo category → Tạo collection mới
   - Khi xóa category → Drop collection (sau backup)
   - Khi update slug → Rename collection

---

## Flows & Features

### Flow 1: CREATE CATEGORY (Admin)
**Description:** Admin tạo category mới với auto-collection creation

**Workflow:**
```
┌─────────────────────────────────────────────────────────────────┐
│                  ADMIN - CREATE CATEGORY                        │
└─────────────────────────────────────────────────────────────────┘

  ADMIN VIEW:
    ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
    │  CLICK   │───▶│  FILL    │───▶│  AUTO    │───▶│ CATEGORY │
    │ "CREATE  │    │  FORM    │    │  CREATE  │    │ CREATED  │
    │ CATEGORY"│    │          │    │  COLLECTION │   │          │
    └──────────┘    └────┬─────┘    └────┬─────┘    └────┬─────┘
                         │                │                │
                         ▼                ▼                ▼
                  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
                  │ - Name      │  │ Generate    │  │ Category    │
                  │ - Slug      │  │ slug from   │  │ saved to DB │
                  │ -   Icon    │  │ name        │  │ Collection  │
                  │                │ Create      │  │ created     │
                  │             │  │ MongoDB     │  │             │
                  │             │  │ collection  │  │             │
                  │             │  │             │  │             │
                  └─────────────┘  └─────────────┘  └─────────────┘
```

**By Actor:**
- **Admin:**
  - Fill form: name, icon, description
  - Slug auto-generated from name (can edit)
  - System tự động tạo MongoDB collection
  - Category created with status: active

- **Vendor:** Not involved

- **Buyer:** Not involved

**Permissions:**
- `[CATEGORY:CREATE]` - Create category (admin only)

**Validation:**
```rust
// Required fields
name: 3-50 chars, unique
slug: 3-50 chars, unique, lowercase, hyphens only

// Optional
parent_id: ObjectId (must exist)
commission_rate: Decimal (0-100%, default: 10)
icon: String (emoji or icon name)
description: String (max 500 chars)
sort_order: i32 (default: 0)
```

**Auto-Collection Creation:**
```rust
// When creating category
async fn create_category(request: CreateCategoryRequest) -> Result<Category> {
  // 1. Validate slug uniqueness
  if slug_exists(request.slug) {
    return Err("Slug already exists");
  }

  // 2. Create category in DB
  let category = Category {
    id: ObjectId::new(),
    name: request.name,
    slug: request.slug,
    parent_id: request.parent_id,
    commission_rate: request.commission_rate,
    icon: request.icon,
    description: request.description,
    sort_order: request.sort_order,
    status: "active",
    created_at: Utc::now(),
  };

  insert_category(&category).await?;

  // 3. Create MongoDB collection for inventory
  let collection_name = format!("inventory_{}", category.slug);
  create_collection(&collection_name).await?;

  // 4. Create indexes on new collection
  create_inventory_indexes(&collection_name).await?;

  Ok(category)
}
```

**Related Files:**
- domain.rs: Category struct
- dto.rs: CreateCategoryRequest, CategoryResponse
- handler.rs: create_category_handler
- service.rs: validate_category_data, create_category, create_inventory_collection
- repository.rs: insert_category, find_category_by_slug, create_mongo_collection

---

### Flow 2: LIST CATEGORIES (Public)
**Description:** Public view tất cả categories (tree structure)

**Workflow:**
```
┌─────────────────────────────────────────────────────────────────┐
│                  PUBLIC - VIEW CATEGORIES                       │
└─────────────────────────────────────────────────────────────────┘

  GUEST/BUYER/VENDOR VIEW:
    ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
    │  BROWSE  │───▶│  TREE    │───▶│  SELECT  │───▶│  VIEW    │
    │/categories│   │  VIEW    │    │ CATEGORY │    │ PRODUCTS │
    └──────────┘    └────┬─────┘    └────┬─────┘    └────┬─────┘
                         │                │                │
                         ▼                ▼                ▼
                  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
                  │ 📺 Streaming│  │ Category    │  │ Filter      │
                  │  ├─ Netflix │  │ detail page │  │ products by │
                  │  ├─ Spotify │  │ with:       │  │ category    │
                  │  └─ Disney+ │  │ - desc      │  │             │
                  │ 🎮 Gaming   │  │ - icon      │  │             │
                  │  ├─ Steam   │  │ - product   │  │             │
                  │  └─ Xbox    │  │   count     │  │             │
                  └─────────────┘  └─────────────┘  └─────────────┘
```

**By Actor:**
- **Buyer/Guest/Vendor:**
  - Browse all categories (status: active)
  - View tree structure (parent → children)
  - Click category → View products in that category
  - See product count per category

- **Admin:** Same + Can see inactive categories

**Permissions:**
- `[CATEGORY:VIEW]` - View categories (public)
- `[CATEGORY:VIEW_ALL]` - View all including inactive (admin)

**Response Format:**
```rust
// Tree structure
pub struct CategoryTreeResponse {
  pub id: ObjectId,
  pub name: String,
  pub slug: String,
  pub icon: String,
  pub description: String,
  pub product_count: i32,
  pub children: Vec<CategoryTreeResponse>,  // Recursive
}

// Example:
{
  id: "123",
  name: "Streaming",
  slug: "streaming",
  icon: "📺",
  product_count: 150,
  children: [
    {
      id: "124",
      name: "Netflix",
      slug: "netflix",
      icon: "🎬",
      product_count: 80,
      children: []
    },
    {
      id: "125",
      name: "Spotify",
      slug: "spotify",
      icon: "🎵",
      product_count: 70,
      children: []
    }
  ]
}
```

**Related Files:**
- domain.rs: Category
- dto.rs: CategoryTreeResponse, CategoryListResponse
- handler.rs: list_categories_handler, get_category_tree_handler
- service.rs: build_category_tree, count_products_by_category
- repository.rs: find_all_categories, aggregate_product_counts

---

### Flow 3: UPDATE CATEGORY (Admin)
**Description:** Admin chỉnh sửa category (có thể đổi slug → rename collection)

**Workflow:**
```
┌─────────────────────────────────────────────────────────────────┐
│                  ADMIN - UPDATE CATEGORY                        │
└─────────────────────────────────────────────────────────────────┘

  ADMIN VIEW:
    ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
    │  CLICK   │───▶│  EDIT    │───▶│  SAVE    │───▶│  UPDATED │
    │ "Edit"   │    │  FIELDS  │    │ CHANGES  │    │          │
    └──────────┘    └────┬─────┘    └────┬─────┘    └────┬─────┘
                         │                │                │
                         ▼                ▼                ▼
                  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
                  │ Editable:   │  │ If slug     │  │ If slug     │
                  │ - name      │  │ changed:    │  │ unchanged:  │
                  │ - slug      │  │ - Rename    │  │ - Update    │
                  │ - parent    │  │   MongoDB   │  │   category  │
                  │ - commission│  │   collection│  │   only      │
                  │ - icon      │  │ - Migrate   │  │             │
                  │ - desc      │  │   inventory │  │             │
                  │ - sort_order│  │ - Update    │  │             │
                  └─────────────┘  │   product   │  │             │
                                   │   refs      │  │             │
                                   └─────────────┘  └─────────────┘
```

**By Actor:**
- **Admin:**
  - Edit category info
  - Change slug → Triggers collection rename
  - Change commission rate → Affects future sales
  - Cannot edit: id, created_at

- **Vendor:** Not involved

- **Buyer:** Not involved

**Permissions:**
- `[CATEGORY:UPDATE]` - Update category (admin only)

**Slug Change Handling:**
```rust
// When updating slug
async fn update_category(id: ObjectId, request: UpdateCategoryRequest) -> Result<Category> {
  let category = find_category_by_id(id).await?;

  // 1. Check if slug changed
  if category.slug != request.slug {
    // 2. Validate new slug is unique
    if slug_exists(&request.slug)? {
      return Err("New slug already exists");
    }

    // 3. Rename MongoDB collection
    let old_collection = format!("inventory_{}", category.slug);
    let new_collection = format!("inventory_{}", request.slug);
    rename_collection(&old_collection, &new_collection).await?;

    // 4. Update all products referencing this category
    update_products_category_slug(&category.slug, &request.slug).await?;

    // 5. Update category slug
    category.slug = request.slug;
  }

  // 6. Update other fields
  category.name = request.name;
  category.commission_rate = request.commission_rate;
  // ... etc

  update_category(&category).await?;
  Ok(category)
}
```

**Related Files:**
- domain.rs: Category
- dto.rs: UpdateCategoryRequest
- handler.rs: update_category_handler
- service.rs: validate_update, update_category, rename_collection_on_slug_change
- repository.rs: update_category, rename_mongo_collection, update_products_category_ref

---

### Flow 4: DELETE CATEGORY (Admin)
**Description:** Admin xóa category với collection cleanup

**Workflow:**
```
┌─────────────────────────────────────────────────────────────────┐
│                  ADMIN - DELETE CATEGORY                        │
└─────────────────────────────────────────────────────────────────┘

  ADMIN VIEW:
    ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
    │  CLICK   │───▶│  CONFIRM │───▶│  CHECK   │───▶│  SOFT    │
    │ "Delete" │    │  ACTION  │    │  PRODUCTS │    │  DELETE  │
    └──────────┘    └────┬─────┘    └────┬─────┘    └────┬─────┘
                         │                │                │
                         ▼                ▼                ▼
                  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
                  │ Show        │  │ If has      │  │ status =    │
                  │ warning     │  │ products →  │  │ deleted     │
                  │ with info   │  │ BLOCK       │  │ Collection  │
                  └─────────────┘  │ (must move  │  │ KEPT for   │
                                    │  products   │  │ 30 days    │
                                    │  first)     │  └─────┬──────┘
                                    └─────────────┘           
```

**By Actor:**
- **Admin:**
  - Request delete (with confirmation)
  - Blocked if has active products
  - Soft delete: status = deleted
  - Collection kept for 30 days

- **Vendor:** Not involved

- **Buyer:** Not involved

**Permissions:**
- `[CATEGORY:DELETE]` - Delete category (admin only)

**Pre-delete Check:**
```rust
// Before deleting
async fn can_delete_category(id: ObjectId) -> Result<bool> {
  // 1. Check for active products
  let product_count = count_products_by_category(id).await?;
  if product_count > 0 {
    return Err(format!(
      "Cannot delete: {} products in this category. Move or delete them first.",
      product_count
    ));
  }

  // 2. Check for child categories
  let child_count = count_child_categories(id).await?;
  if child_count > 0 {
    return Err(format!(
      "Cannot delete: {} child categories. Move or delete them first.",
      child_count
    ));
  }

  Ok(true)
}
```

**Soft Delete Behavior:**
```rust
// Soft delete category
async fn soft_delete_category(id: ObjectId) -> Result<()> {
  // 1. Update category status
  update_category_status(id, "deleted").await?;

  // 2. Set deleted_at
  set_category_deleted_at(id, Utc::now()).await?;

  // 3. Collection is NOT dropped yet
  // It will be dropped after 30 days by cron job

  Ok(())
}

// Hard delete (after 30 days)
async fn hard_delete_category(id: ObjectId) -> Result<()> {
  let category = find_category_by_id(id).await?;

  // 1. Drop MongoDB collection
  let collection_name = format!("inventory_{}", category.slug);
  drop_collection(&collection_name).await?;

  // 2. Delete category from DB
  delete_category(id).await?;

  Ok(())
}
```

**Related Files:**
- domain.rs: Category
- dto.rs: DeleteCategoryRequest
- handler.rs: delete_category_handler
- service.rs: can_delete_category, soft_delete_category, hard_delete_category
- repository.rs: update_category_status, count_products_by_category, drop_mongo_collection


### Flow 6: REORDER CATEGORIES (Admin)
**Description:** Admin thay đổi thứ tự hiển thị categories

**Workflow:**
```
┌─────────────────────────────────────────────────────────────────┐
│                  ADMIN - REORDER CATEGORIES                     │
└─────────────────────────────────────────────────────────────────┘

  ADMIN VIEW:
    ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
    │  DRAG &  │───▶│  SAVE    │───▶│  UPDATE  │───▶│  NEW     │
    │  DROP    │    │  ORDER   │    │  SORT_   │    │  ORDER   │
    │  ITEMS   │    │          │    │  ORDER    │    │ SAVED    │
    └──────────┘    └────┬─────┘    └────┬─────┘    └────┬─────┘
                         │                │                │
                         ▼                ▼                ▼
                  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
                  │ Reorder:    │  │ Batch       │  │ Public      │
                  │ 1. Gaming   │  │ update      │  │ sees new    │
                  │ 2. Streaming│  │ sort_order  │  │ order       │
                  │ 3. Shopping │  │ for all     │  │ immediately │
                  │             │  │ categories  │  │             │
                  └─────────────┘  └─────────────┘  └─────────────┘
```

**By Actor:**
- **Admin:**
  - Drag & drop reorder
  - Or manual enter sort_order numbers
  - Batch update all categories
  - Changes visible immediately

- **Vendor:** Not involved

- **Buyer:** Not involved

**Permissions:**
- `[CATEGORY:REORDER]` - Reorder categories (admin only)

**Reorder Logic:**
```rust
// Batch update sort_order
async fn reorder_categories(updates: Vec<CategoryReorderUpdate>) -> Result<()> {
  for update in updates {
    update_category_sort_order(update.id, update.sort_order).await?;
  }
  Ok(())
}

// When listing categories for public
async fn list_categories_ordered() -> Result<Vec<Category>> {
  find_all_categories()
    .sort_by("sort_order", 1)  // Ascending
    .sort_by("name", 1)        // Then alphabetically
    .await
}
```

**Related Files:**
- domain.rs: Category
- dto.rs: CategoryReorderUpdate
- handler.rs: reorder_categories_handler
- service.rs: batch_update_sort_order
- repository.rs: update_category_sort_order

---

## Permission Matrix

| Action | Permission Code | Buyer | Vendor | Admin |
|--------|----------------|-------|--------|-------|
| View categories | [CATEGORY:VIEW] | ✅ | ✅ | ✅ |
| View all categories | [CATEGORY:VIEW_ALL] | ❌ | ❌ | ✅ |
| Create category | [CATEGORY:CREATE] | ❌ | ❌ | ✅ |
| Update category | [CATEGORY:UPDATE] | ❌ | ❌ | ✅ |
| Delete category | [CATEGORY:DELETE] | ❌ | ❌ | ✅ |
| Set commission rate | [CATEGORY:SET_COMMISSION] | ❌ | ❌ | ✅ |
| Reorder categories | [CATEGORY:REORDER] | ❌ | ❌ | ✅ |

---

## API Endpoints

| Method | Endpoint | Permission | Description |
|--------|----------|------------|-------------|
| **Public** | | | |
| GET | /api/categories | [CATEGORY:VIEW] | List all categories (tree) |
| **Admin** | | | |
| POST | /api/admin/categories | [CATEGORY:CREATE] | Create new category |
| PUT | /api/admin/categories/{id} | [CATEGORY:UPDATE] | Update category |
| DELETE | /api/admin/categories/{id} | [CATEGORY:DELETE] | Delete category |
| GET | /api/admin/categories/all | [CATEGORY:VIEW_ALL] | List all including inactive |

---

## MongoDB Collection Strategy

### Per-Category Collections

**Naming Convention:**
```javascript
// Collection name format: inventory_{category_slug}

Examples:
- Category: "Netflix" (slug: netflix)
  → Collection: inventory_netflix

- Category: "Gaming Accounts" (slug: gaming-accounts)
  → Collection: inventory_gaming_accounts

- Category: "Streaming Services" (slug: streaming-services)
  → Collection: inventory_streaming_services
```

**Collection Schema:**
```javascript
// inventory_{category_slug}
{
  _id: ObjectId,
  product_id: ObjectId,      // Reference to products collection
  shop_id: ObjectId,         // Reference to shops collection
  content: String,           // Plain text inventory content
  is_sold: Boolean,          // Sold status
  order_id: ObjectId,        // Reference to order (if sold)
  hold_until: DateTime,      // For pre-orders
  sold_at: DateTime,         // When sold
  created_at: DateTime,
}

// Indexes
db.inventory_netflix.createIndex({ product_id: 1 })
db.inventory_netflix.createIndex({ shop_id: 1 })
db.inventory_netflix.createIndex({ is_sold: 1 })
db.inventory_netflix.createIndex({ content: 1 })  // For duplicate check
db.inventory_netflix.createIndex({ created_at: -1 })
```

**Why Per-Category Collections?**

**Pros:**
- ✅ Better performance (smaller collections per query)
- ✅ Easier to manage (backup, restore per category)
- ✅ Better scaling (sharding by category)
- ✅ Can archive old categories easily
- ✅ Faster queries (scan smaller dataset)

**Cons:**
- ❌ More complex queries (need to know category)
- ❌ Duplicate check across collections (need to query multiple)
- ❌ Migration needed if category changes

**Implementation:**
```rust
// Get collection for category
async fn get_inventory_collection(category_slug: &str) -> Collection<ProductItem> {
    let collection_name = format!("inventory_{}", category_slug);
    db.collection::<ProductItem>(&collection_name)
}

// Insert item
async fn insert_inventory_item(category_slug: &str, item: ProductItem) -> Result<()> {
    let collection = get_inventory_collection(category_slug).await;
    collection.insert_one(item).await?;
    Ok(())
}

// Duplicate check across all category collections
async fn check_duplicate_across_all_collections(content: &str) -> Result<bool> {
    let all_collections = get_all_inventory_collections().await?;

    for collection_name in all_collections {
        let collection = db.collection::<ProductItem>(&collection_name);
        let exists = collection
            .find_one(doc! { "content": content })
            .await?;

        if exists.is_some() {
            return Ok(true);  // Found duplicate
        }
    }

    Ok(false)  // No duplicate
}
```

---

## Data Models

### Category Model

```rust
use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub _id: ObjectId,
    pub name: String,              // 3-50 chars, unique
    pub slug: String,              // 3-50 chars, unique, lowercase
    pub parent_id: Option<ObjectId>, // For hierarchy
    pub commission_rate: Decimal,  // 0-100%, default: 10
    pub icon: Option<String>,      // Emoji or icon name
    pub description: Option<String>, // Max 500 chars
    pub sort_order: i32,           // For display order
    pub status: String,            // "active" | "deleted"
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// Indexes
// - slug: unique
// - parent_id
// - status
// - sort_order
```

### Product Model (Reference to Category)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub _id: ObjectId,
    pub shop_id: ObjectId,
    pub category_id: ObjectId,     // Reference to category
    pub category_slug: String,     // Denormalized for collection lookup
    pub name: String,
    pub slug: String,
    pub description: String,
    pub price: Decimal,
    pub stock: i32,
    pub status: String,
    // ...
}
```

### ProductItem Model (Per-Category Collection)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductItem {
    pub _id: ObjectId,
    pub product_id: ObjectId,
    pub shop_id: ObjectId,
    pub content: String,           // Plain text, NO encryption
    pub is_sold: bool,
    pub order_id: Option<ObjectId>,
    pub hold_until: Option<DateTime<Utc>>,
    pub sold_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
```

---

## Dependencies

### Required By
- **Product Module**: Products belong to categories
- **Inventory Module**: Items stored in per-category collections
- **Order Module**: Commission calculated from category rate

### Depends On
- **MongoDB**: For category storage and per-category inventory collections
- **Auth Module**: Permission checks for admin operations

### External Services
- None (pure database operations)

---

## Open Questions

1. **Category Depth Limit**: Should we limit hierarchy depth (e.g., max 2 levels)?
2. **Slug Change Impact**: If category slug changes, do we update all products' category_slug?
3. **Archive Strategy**: How to handle old/inactive category collections?
4. **Commission History**: Do we track commission rate changes over time?
5. **Bulk Operations**: How to handle bulk category updates?

---

## Implementation Phases

### Phase 1: Foundation (Domain Models + Repository)
**Priority:** HIGH

**Tasks:**
1. [ ] Create `domain.rs` with Category struct
2. [ ] Create `repository.rs` with CRUD operations
3. [ ] Implement collection management (create, rename, drop)
4. [ ] Write unit tests

**Estimated Files:**
- `src/modules/category/domain.rs`
- `src/modules/category/repository.rs`

---

### Phase 2: Data Transfer Objects (DTOs)
**Priority:** HIGH

**Tasks:**
1. [ ] Create request DTOs: CreateCategoryRequest, UpdateCategoryRequest
2. [ ] Create response DTOs: CategoryResponse, CategoryTreeResponse
3. [ ] Add validation rules
4. [ ] Write tests

**Estimated Files:**
- `src/modules/category/dto.rs`

---

### Phase 3: Business Logic (Service Layer)
**Priority:** HIGH

**Tasks:**
1. [ ] Implement CategoryService: create, update, delete, reorder
2. [ ] Implement collection management service
3. [ ] Implement tree building logic
4. [ ] Implement commission rate management
5. [ ] Write unit tests

**Estimated Files:**
- `src/modules/category/service.rs`

---

### Phase 4: HTTP Handlers
**Priority:** HIGH

**Tasks:**
1. [ ] Create public handlers: list_categories, get_category
2. [ ] Create admin handlers: create, update, delete, reorder
3. [ ] Add permission checks
4. [ ] Write integration tests

**Estimated Files:**
- `src/modules/category/handler.rs`

---

### Phase 5: Routes Configuration
**Priority:** MEDIUM

**Tasks:**
1. [ ] Configure public routes (/api/categories)
2. [ ] Configure admin routes (/api/admin/categories)
3. [ ] Add middleware
4. [ ] Test routing

**Estimated Files:**
- `src/modules/category/routes.rs`
- `src/modules/category/mod.rs`

---

### Phase 6: Integration & Testing
**Priority:** MEDIUM

**Tasks:**
1. [ ] Test collection creation on category create
2. [ ] Test collection rename on slug change
3. [ ] Test collection drop on category delete
4. [ ] Test tree building
5. [ ] Test commission rate application

---

### Phase 7: Documentation
**Priority:** LOW

**Tasks:**
1. [ ] Add rustdoc comments
2. [ ] Update OpenAPI/Swagger docs
3. [ ] Write usage examples

---

## Implementation Order

```
Phase 1 - Foundation:
  ├─ Create Category struct (domain.rs)
  ├─ Implement CategoryRepository (repository.rs)
  ├─ Implement MongoDB collection management (repository.rs)
  └─ Test: Create collection, insert, find, update, delete

Phase 2 - DTOs:
  ├─ Create CreateCategoryRequest (dto.rs)
  ├─ Create CategoryResponse, CategoryTreeResponse (dto.rs)
  ├─ Add validation: name, slug uniqueness
  └─ Test: Validation rules

Phase 3 - Service:
  ├─ Implement CategoryService::create (service.rs)
  ├─ Implement create_inventory_collection (service.rs)
  ├─ Implement build_category_tree (service.rs)
  ├─ Implement update_category (service.rs)
  ├─ Implement rename_collection_on_slug_change (service.rs)
  ├─ Implement soft_delete_category (service.rs)
  ├─ Implement reorder_categories (service.rs)
  └─ Test: All service functions

Phase 4 - Handler:
  ├─ Implement list_categories_handler (public)
  ├─ Implement get_category_handler (public)
  ├─ Implement create_category_handler (admin)
  ├─ Implement update_category_handler (admin)
  ├─ Implement delete_category_handler (admin)
  ├─ Implement reorder_categories_handler (admin)
  └─ Add permission checks

Phase 5 - Routes:
  ├─ Add public routes (/api/categories)
  ├─ Add admin routes (/api/admin/categories)
  └─ Add middleware

Phase 6 - Integration:
  ├─ Test: Create category → Collection created
  ├─ Test: Update slug → Collection renamed
  ├─ Test: Delete category → Soft delete → Hard delete after 30 days
  ├─ Test: Build tree with 3 levels
  └─ Test: Set commission → Affects new orders

Phase 7 - Docs:
  ├─ Add rustdoc
  ├─ Update OpenAPI
  └─ Write examples
```

---

## Dependencies Map

```
┌─────────────────────────────────────────────────────────┐
│              CATEGORY MODULE DEPENDENCIES                │
└─────────────────────────────────────────────────────────┘

                    ┌──────────────┐
                    │ AUTH MODULE  │
                    │ (Permissions)│
                    └──────┬───────┘
                           │
                    ┌──────▼───────┐
                    │ CATEGORY     │
                    │ MODULE       │
                    └──────┬───────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
        ▼                  ▼                  ▼
  ┌──────────┐      ┌──────────┐      ┌──────────┐
  │ PRODUCT  │      │INVENTORY │      │ ORDER    │
  │ MODULE   │      │MANAGEMENT│      │ MODULE   │
  │          │      │(Collections)│      │          │
  └──────────┘      └──────────┘      └──────────┘
```

---

## Next Steps

1. [ ] **Review Design** - Check if per-category collection approach is suitable
2. [ ] **Approve Brainstorm** - Sign off on this design
3. [ ] **Create Implementation Plan** - Run `/write-plan category`
4. [ ] **Set Up Git Worktree** - Create isolated branch
5. [ ] **Start Phase 1** - Begin with domain models

---

## Document Info

**Created:** 2025-01-04
**Module:** Category Module
**Status:** Draft - Pending Review
**Related Documents:**
- [Product Brainstorm](./product-brainstorm.md)
- [V2 Full Flows](../../full-flows.md)
