---
name: mongodb-rust
description: Use when working with MongoDB in Rust - CRUD operations, aggregation, transactions, connection pooling, indexing, or repository patterns with the official MongoDB Rust driver
---

# MongoDB + Rust Patterns

## Overview

Production-ready patterns for **MongoDB Rust driver v3.1+**. Focus: connection pooling, type safety, error handling, transactions, aggregation, and performance.

**Core principles:**
- ONE `Client` instance per application lifetime
- Connection pooling (min/max pool size)
- Type-safe with `serde` BSON serialization
- Transactions for multi-document ACID
- Index optimization for query patterns
- ALL public APIs need `#[utoipa::path]` docs
- ALL protected handlers need `#[protect()]` with permissions
- Define permissions in `common/permissions.rs` before using

## 1. Connection Setup (CRITICAL)

**NEVER create multiple Client instances. Create ONE at startup.**

```rust
// config/mongodb.rs
use mongodb::{Client, Database, options::ClientOptions};
use std::time::Duration;

pub struct MongoDb {
    pub client: Client,
    pub database: Database,
}

impl MongoDb {
    pub async fn new(uri: &str, db_name: &str) -> Result<Self, mongodb::error::Error> {
        let mut options = ClientOptions::parse(uri).await?;

        // Connection pool
        options.max_pool_size = Some(100);
        options.min_pool_size = Some(10);
        options.max_idle_time = Some(Duration::from_secs(90));

        // Timeouts
        options.connect_timeout = Some(Duration::from_secs(10));
        options.server_selection_timeout = Some(Duration::from_secs(30));

        let client = Client::with_options(options)?;
        let database = client.database(db_name);

        // Verify connection
        database.run_command(bson::doc! { "ping": 1 }, None).await?;

        Ok(Self { client, database })
    }

    pub fn collection<T>(&self, name: &str) -> mongodb::Collection<T>
    where T: serde::de::DeserializeOwned + serde::Serialize {
        self.database.collection(name)
    }
}
```

## 2. Domain Models

```rust
use serde::{Deserialize, Serialize};
use mongodb::bson::{oid::ObjectId, DateTime};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: Option<ObjectId>,
    pub email: String,
    pub username: String,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub status: UserStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserStatus { Active, Inactive, Suspended }
```

## 3. Base Repository

```rust
use mongodb::{Collection, Database, bson::{doc, oid::ObjectId}, options::FindOptions};
use anyhow::Result;

pub struct BaseRepository<T> {
    collection: Collection<T>,
}

impl<T> BaseRepository<T>
where T: serde::de::DeserializeOwned + serde::Serialize {
    pub fn new(db: &Database, name: &str) -> Self {
        Self { collection: db.collection(name) }
    }

    pub async fn insert(&self, doc: T) -> Result<ObjectId> {
        Ok(self.collection.insert_one(doc, None).await?.inserted_id.as_object_id().unwrap())
    }

    pub async fn find_by_id(&self, id: &ObjectId) -> Result<Option<T>> {
        Ok(self.collection.find_one(doc! { "_id": id }, None).await?)
    }

    pub async fn find_one(&self, filter: bson::Document) -> Result<Option<T>> {
        Ok(self.collection.find_one(filter, None).await?)
    }

    pub async fn find_paginated(
        &self, filter: bson::Document, page: u32, per_page: u32
    ) -> Result<(Vec<T>, u64)> {
        let skip = (page - 1) * per_page;
        let opts = FindOptions::builder().skip(skip as u64).limit(per_page as i64).build();

        let total = self.collection.count_documents(filter.clone(), None).await?;
        let cursor = self.collection.find(filter, opts).await?;
        let docs = cursor.try_collect().await?;

        Ok((docs, total))
    }

    pub async fn update_one(&self, filter: bson::Document, update: bson::Document) -> Result<u64> {
        Ok(self.collection.update_one(filter, update, None).await?.modified_count)
    }

    pub async fn delete_one(&self, filter: bson::Document) -> Result<u64> {
        Ok(self.collection.delete_one(filter, None).await?.deleted_count)
    }

    pub async fn count(&self, filter: bson::Document) -> Result<u64> {
        Ok(self.collection.count_documents(filter, None).await?)
    }
}
```

## 4. Aggregation Pipelines

```rust
use mongodb::bson::doc;

// Daily revenue
let pipeline = vec![
    doc! { "$match": { "status": "completed" } },
    doc! {
        "$group": {
            "_id": { "$dateToString": { "format": "%Y-%m-%d", "date": "$created_at" } },
            "total": doc! { "$sum": "$amount" },
            "count": doc! { "$sum": 1 }
        }
    },
    doc! { "$sort": { "_id": 1 } },
];

let results: Vec<DailyRevenue> = collection.aggregate(pipeline, None).await?.try_collect().await?;
```

**Best practices:**
- ✅ Use `$match` early to filter
- ✅ Use `$project` to limit fields
- ✅ Create indexes on `$match`, `$sort`, `$group` fields
- ❌ Avoid `$unwind` unless necessary (creates N documents)
- ❌ Avoid excessive `$lookup` (joins are expensive)

## 5. Transactions (ACID)

**Requires:** Replica set, collections must exist first.

```rust
use mongodb::{Client, bson::doc, options::TransactionOptions};

pub async fn transfer(&self, from: ObjectId, to: ObjectId, amount: i64) -> Result<()> {
    let mut session = self.client.start_session(None).await?;

    let txn_opts = TransactionOptions::builder()
        .read_concern(mongodb::options::ReadConcern::local())
        .write_concern(mongodb::options::WriteConcern::majority())
        .build();

    self.client
        .database(&self.db_name)
        .run_transaction(&mut session, txn_opts, |session| async move {
            // Pessimistic lock
            let from_wallet = db.collection::<Wallet>("wallets")
                .find_one_with_session(doc! { "user_id": from }, None, &session)
                .await?
                .ok_or(anyhow::anyhow!("Not found"))?;

            if from_wallet.balance < amount {
                anyhow::bail!("Insufficient");
            }

            // Deduct
            db.collection::<Wallet>("wallets")
                .update_one_with_session(
                    doc! { "user_id": from },
                    doc! { "$inc": { "balance": -amount } },
                    None, &session,
                )
                .await?;

            // Add
            db.collection::<Wallet>("wallets")
                .update_one_with_session(
                    doc! { "user_id": to },
                    doc! { "$inc": { "balance": amount } },
                    None, &session,
                )
                .await?;

            Ok(())
        })
        .await?;

    Ok(())
}
```

## 6. Index Management

```rust
use mongodb::{IndexModel, options::IndexOptions};

// Users
let user_indexes = vec![
    IndexModel::builder()
        .keys(doc! { "email": 1 })
        .options(IndexOptions::builder().unique(true).build())
        .build(),
    IndexModel::builder()
        .keys(doc! { "status": 1, "created_at": -1 })
        .build(),
];

// Orders
let order_indexes = vec![
    IndexModel::builder()
        .keys(doc! { "buyer_id": 1, "created_at": -1 })
        .build(),
    IndexModel::builder()
        .keys(doc! { "shop_id": 1, "status": 1 })
        .build(),
];

db.collection("users").create_indexes(user_indexes, None).await?;
db.collection("orders").create_indexes(order_indexes, None).await?;
```

## 7. Error Handling

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Duplicate: {0}")]
    Duplicate(String),
    #[error("MongoDB: {0}")]
    Mongo(#[from] mongodb::error::Error),
}

impl RepositoryError {
    pub fn is_transient(&self) -> bool {
        matches!(self, RepositoryError::Mongo(e) if e.to_string().contains("network"))
    }
}
```

## 8. Performance Tips

| ❌ Bad | ✅ Good |
|-------|--------|
| `col.find(doc! {}, None)` | Use `limit()` + `projection` |
| Multiple `insert_one()` | Use `insert_many()` |
| Aggregate without `$match` | Filter early with `$match` |
| Sort without index | Create index first |
| `$unwind` on large arrays | Group directly on array field |

## Quick Reference

| Operation | Method |
|-----------|--------|
| Insert one | `col.insert_one(doc, None).await` |
| Find one | `col.find_one(doc! {"_id": id}, None).await` |
| Update | `col.update_one(filter, update, None).await` |
| Delete | `col.delete_one(filter, None).await` |
| Count | `col.count_documents(filter, None).await` |
| Aggregate | `col.aggregate(pipeline, None).await` |
| Transaction | `db.run_transaction(&mut session, opts, callback).await` |

## Common Mistakes

| ❌ Mistake | ✅ Fix |
|------------|-------|
| Client per request | ONE client instance at startup |
| No indexes | Index query fields |
| `$unwind` everywhere | Group on array directly |
| Fetch all fields | Use `projection` |
| No pagination | Always `limit()` + `skip()` |
| Missing `#[protect()]` | ALL protected handlers need permissions |

## Dependencies

```toml
[dependencies]
mongodb = "3.1"
bson = "2.13"
serde = { version = "1.0", features = ["derive"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "2.0"
anyhow = "1.0"
tokio = { version = "1", features = ["full"] }
futures = "0.3"

# OpenAPI & Auth (REQUIRED)
utoipa = { version = "5.4", features = ["actix_extras"] }
utoipa-swagger-ui = "8"
validator = { version = "0.18", features = ["derive"] }
actix-web-grants = "4"
```
