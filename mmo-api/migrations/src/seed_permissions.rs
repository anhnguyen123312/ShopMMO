//! Seed script for authorization system
//!
//! Run with: cargo run --bin seed_permissions

use mongodb::{
    bson::{doc, oid::ObjectId, DateTime as BsonDateTime},
    Client, Collection,
};
use tokio::main;
use futures::stream::TryStreamExt;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Permission {
    #[serde(rename = "_id")]
    id: Option<ObjectId>,
    name: String,
    display_name: String,
    description: String,
    resource: String,
    action: String,
    category: String,
    is_active: bool,
    created_at: mongodb::bson::DateTime,
    updated_at: mongodb::bson::DateTime,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Role {
    #[serde(rename = "_id")]
    id: Option<ObjectId>,
    name: String,
    display_name: String,
    description: String,
    level: i32,
    inherits_from: Vec<String>,
    direct_permissions: Vec<ObjectId>,
    flattened_permissions: Vec<String>,
    is_system: bool,
    is_active: bool,
    version: i32,
    created_at: mongodb::bson::DateTime,
    updated_at: mongodb::bson::DateTime,
}

#[main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================");
    println!("MMO API - Authorization System V2 Seed");
    println!("========================================");

    // Load MongoDB URL from environment or use default
    // Supports both MONGODB_URL and MONGODB_URI environment variables
    let mongo_url = std::env::var("MONGODB_URL")
        .or_else(|_| std::env::var("MONGODB_URI"))
        .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());

    println!("\nConnecting to MongoDB...");
    let client = Client::with_uri_str(&mongo_url).await?;
    let db = client.database("mmo_api");

    // ========================================================================
    // 1. CREATE PERMISSIONS
    // ========================================================================

    println!("\n1. Seeding permissions...");
    let permissions_collection: Collection<Permission> = db.collection("permissions");

    // Clean up any documents with null _id first
    let _ = permissions_collection.delete_many(doc! { "_id": null }).await;

    // Delete existing permissions (optional - remove if you want to keep existing)
    let delete_result = permissions_collection.delete_many(doc! {}).await?;
    println!("   Deleted {} existing permissions", delete_result.deleted_count);

    let now = mongodb::bson::DateTime::now();

    let permissions = vec![
        // User Management
        Permission {
            id: Some(bson::oid::ObjectId::new()),
            name: "users:read".to_string(),
            display_name: "Read Users".to_string(),
            description: "View user information".to_string(),
            resource: "users".to_string(),
            action: "read".to_string(),
            category: "user_management".to_string(),
            is_active: true,
            created_at: now,
            updated_at: now,
        },
        Permission {
            id: Some(bson::oid::ObjectId::new()),
            name: "users:update".to_string(),
            display_name: "Update Users".to_string(),
            description: "Update user information".to_string(),
            resource: "users".to_string(),
            action: "update".to_string(),
            category: "user_management".to_string(),
            is_active: true,
            created_at: now,
            updated_at: now,
        },
        // Product Management
        Permission {
            id: Some(bson::oid::ObjectId::new()),
            name: "products:read".to_string(),
            display_name: "Read Products".to_string(),
            description: "View products".to_string(),
            resource: "products".to_string(),
            action: "read".to_string(),
            category: "product_management".to_string(),
            is_active: true,
            created_at: now,
            updated_at: now,
        },
        Permission {
            id: Some(bson::oid::ObjectId::new()),
            name: "products:create".to_string(),
            display_name: "Create Products".to_string(),
            description: "Create new products".to_string(),
            resource: "products".to_string(),
            action: "create".to_string(),
            category: "product_management".to_string(),
            is_active: true,
            created_at: now,
            updated_at: now,
        },
        Permission {
            id: Some(bson::oid::ObjectId::new()),
            name: "products:update".to_string(),
            display_name: "Update Products".to_string(),
            description: "Update product information".to_string(),
            resource: "products".to_string(),
            action: "update".to_string(),
            category: "product_management".to_string(),
            is_active: true,
            created_at: now,
            updated_at: now,
        },
        Permission {
            id: Some(bson::oid::ObjectId::new()),
            name: "products:delete".to_string(),
            display_name: "Delete Products".to_string(),
            description: "Delete products".to_string(),
            resource: "products".to_string(),
            action: "delete".to_string(),
            category: "product_management".to_string(),
            is_active: true,
            created_at: now,
            updated_at: now,
        },
        // Order Management
        Permission {
            id: Some(bson::oid::ObjectId::new()),
            name: "orders:read".to_string(),
            display_name: "Read Orders".to_string(),
            description: "View orders".to_string(),
            resource: "orders".to_string(),
            action: "read".to_string(),
            category: "order_management".to_string(),
            is_active: true,
            created_at: now,
            updated_at: now,
        },
        Permission {
            id: Some(bson::oid::ObjectId::new()),
            name: "orders:create".to_string(),
            display_name: "Create Orders".to_string(),
            description: "Create new orders".to_string(),
            resource: "orders".to_string(),
            action: "create".to_string(),
            category: "order_management".to_string(),
            is_active: true,
            created_at: now,
            updated_at: now,
        },
        Permission {
            id: Some(bson::oid::ObjectId::new()),
            name: "orders:update".to_string(),
            display_name: "Update Orders".to_string(),
            description: "Update order status".to_string(),
            resource: "orders".to_string(),
            action: "update".to_string(),
            category: "order_management".to_string(),
            is_active: true,
            created_at: now,
            updated_at: now,
        },
        // Wallet Management
        Permission {
            id: Some(bson::oid::ObjectId::new()),
            name: "wallets:read".to_string(),
            display_name: "Read Wallets".to_string(),
            description: "View wallet information".to_string(),
            resource: "wallets".to_string(),
            action: "read".to_string(),
            category: "wallet_management".to_string(),
            is_active: true,
            created_at: now,
            updated_at: now,
        },
        Permission {
            id: Some(bson::oid::ObjectId::new()),
            name: "wallets:manage".to_string(),
            display_name: "Manage Wallets".to_string(),
            description: "Manage wallet operations".to_string(),
            resource: "wallets".to_string(),
            action: "manage".to_string(),
            category: "wallet_management".to_string(),
            is_active: true,
            created_at: now,
            updated_at: now,
        },
        // Admin
        Permission {
            id: Some(bson::oid::ObjectId::new()),
            name: "admin:users".to_string(),
            display_name: "Admin Users".to_string(),
            description: "Administrative access to user management".to_string(),
            resource: "admin".to_string(),
            action: "users".to_string(),
            category: "administration".to_string(),
            is_active: true,
            created_at: now,
            updated_at: now,
        },
        Permission {
            id: Some(bson::oid::ObjectId::new()),
            name: "admin:system".to_string(),
            display_name: "Admin System".to_string(),
            description: "Full system administration".to_string(),
            resource: "admin".to_string(),
            action: "system".to_string(),
            category: "administration".to_string(),
            is_active: true,
            created_at: now,
            updated_at: now,
        },
    ];

    // Insert permissions one by one to avoid duplicate key issues
    for permission in permissions {
        permissions_collection.insert_one(permission).await?;
    }
    println!("   Inserted permissions");

    // Get permission IDs for roles
    let cursor = permissions_collection.find(doc! {}).await?;
    let all_permissions: Vec<Permission> = cursor.try_collect().await.map_err(|e| format!("Failed to collect: {}", e))?;

    let buyer_perm_names = vec!["products:read", "orders:read", "orders:create", "wallets:read"];
    let seller_perm_names = vec!["products:read", "products:create", "products:update", "products:delete", "orders:read", "orders:update", "wallets:read", "wallets:manage"];

    let buyer_perm_ids: Vec<ObjectId> = all_permissions
        .iter()
        .filter(|p| buyer_perm_names.contains(&p.name.as_str()))
        .filter_map(|p| p.id)
        .collect();

    let seller_perm_ids: Vec<ObjectId> = all_permissions
        .iter()
        .filter(|p| seller_perm_names.contains(&p.name.as_str()))
        .filter_map(|p| p.id)
        .collect();

    let admin_perm_ids: Vec<ObjectId> = all_permissions
        .iter()
        .filter_map(|p| p.id)
        .collect();

    // ========================================================================
    // 2. CREATE ROLES
    // ========================================================================

    println!("\n2. Seeding roles...");
    let roles_collection: Collection<Role> = db.collection("roles");

    let roles = vec![
        Role {
            id: Some(bson::oid::ObjectId::new()),
            name: "BUYER".to_string(),
            display_name: "Buyer".to_string(),
            description: "Regular buyer role".to_string(),
            level: 0,
            inherits_from: vec![],
            direct_permissions: buyer_perm_ids.clone(),
            flattened_permissions: buyer_perm_names.iter().map(|s| s.to_string()).collect(),
            is_system: true,
            is_active: true,
            version: 1,
            created_at: now,
            updated_at: now,
        },
        Role {
            id: Some(bson::oid::ObjectId::new()),
            name: "SELLER".to_string(),
            display_name: "Seller".to_string(),
            description: "Seller role".to_string(),
            level: 1,
            inherits_from: vec!["BUYER".to_string()],
            direct_permissions: seller_perm_ids.clone(),
            flattened_permissions: seller_perm_names.iter().map(|s| s.to_string()).collect(),
            is_system: true,
            is_active: true,
            version: 1,
            created_at: now,
            updated_at: now,
        },
        Role {
            id: Some(bson::oid::ObjectId::new()),
            name: "ADMIN".to_string(),
            display_name: "Administrator".to_string(),
            description: "Administrator role".to_string(),
            level: 2,
            inherits_from: vec!["SELLER".to_string()],
            direct_permissions: admin_perm_ids.clone(),
            flattened_permissions: all_permissions.iter().map(|p| p.name.clone()).collect(),
            is_system: true,
            is_active: true,
            version: 1,
            created_at: now,
            updated_at: now,
        },
        Role {
            id: Some(bson::oid::ObjectId::new()),
            name: "SUPER_ADMIN".to_string(),
            display_name: "Super Administrator".to_string(),
            description: "Super administrator role".to_string(),
            level: 3,
            inherits_from: vec![],
            direct_permissions: admin_perm_ids.clone(),
            flattened_permissions: all_permissions.iter().map(|p| p.name.clone()).collect(),
            is_system: true,
            is_active: true,
            version: 1,
            created_at: now,
            updated_at: now,
        },
    ];

    // Delete existing roles
    let delete_result = roles_collection.delete_many(doc! {}).await?;
    println!("   Deleted {} existing roles", delete_result.deleted_count);

    // Insert roles
    let insert_result = roles_collection.insert_many(roles).await?;
    println!("   Inserted {} roles", insert_result.inserted_ids.len());

    // ========================================================================
    // CREATE INDEXES
    // ========================================================================

    println!("\n3. Creating indexes...");

    use mongodb::IndexModel;

    // Permissions indexes
    let perm_indexes = vec![
        IndexModel::builder().keys(doc! { "name": 1 }).build(),
        IndexModel::builder().keys(doc! { "resource": 1, "action": 1 }).build(),
        IndexModel::builder().keys(doc! { "category": 1 }).build(),
        IndexModel::builder().keys(doc! { "is_active": 1 }).build(),
    ];
    permissions_collection.create_indexes(perm_indexes).await?;
    println!("   Permissions indexes created");

    // Roles indexes
    let role_indexes = vec![
        IndexModel::builder().keys(doc! { "name": 1 }).build(),
        IndexModel::builder().keys(doc! { "level": 1 }).build(),
        IndexModel::builder().keys(doc! { "is_active": 1 }).build(),
        IndexModel::builder().keys(doc! { "is_system": 1 }).build(),
    ];
    roles_collection.create_indexes(role_indexes).await?;
    println!("   Roles indexes created");

    // ========================================================================
    // SUMMARY
    // ========================================================================

    println!("\n========================================");
    println!("Seed completed successfully!");
    println!("========================================");
    println!("Permissions: {}", all_permissions.len());
    println!("Roles: 4");
    println!("\nRole hierarchy:");
    println!("  SUPER_ADMIN (level 3)");
    println!("    └─ ADMIN (level 2)");
    println!("         └─ SELLER (level 1)");
    println!("              └─ BUYER (level 0)");
    println!("\n========================================");

    Ok(())
}
