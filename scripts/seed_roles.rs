//! Role Seeding Script
//!
//! Creates default roles with permissions for the MMO API.
//!
//! Run with:
//!   cargo run --bin seed_roles

use bson::oid::ObjectId;
use mongodb::{bson::doc, Client, Collection};
use serde::{Deserialize, Serialize};
use tokio::main;

#[derive(Debug, Serialize, Deserialize)]
struct Role {
    #[serde(rename = "_id")]
    id: Option<ObjectId>,
    name: String,
    display_name: String,
    level: i32,
    parent_role_id: Option<ObjectId>,
    inherits_from: Vec<String>,
    direct_permissions: Vec<ObjectId>,
    flattened_permissions: Vec<String>,
    is_system: bool,
    is_active: bool,
    version: i32,
    created_at: mongodb::bson::DateTime,
    updated_at: mongodb::bson::DateTime,
}

// Define permissions (must match constants.rs)
const PERM_PRODUCT_CREATE: &str = "product:create";
const PERM_PRODUCT_READ: &str = "product:read";
const PERM_PRODUCT_UPDATE: &str = "product:update";
const PERM_PRODUCT_DELETE: &str = "product:delete";
const PERM_PRODUCT_LIST: &str = "product:list";

const PERM_ORDER_CREATE: &str = "order:create";
const PERM_ORDER_READ: &str = "order:read";
const PERM_ORDER_UPDATE: &str = "order:update";
const PERM_ORDER_CANCEL: &str = "order:cancel";
const PERM_ORDER_LIST: &str = "order:list";

const PERM_WALLET_READ: &str = "wallet:read";
const PERM_WALLET_WITHDRAW: &str = "wallet:withdraw";
const PERM_WALLET_DEPOSIT: &str = "wallet:deposit";
const PERM_WALLET_LIST: &str = "wallet:list";

const PERM_USER_CREATE: &str = "user:create";
const PERM_USER_READ: &str = "user:read";
const PERM_USER_UPDATE: &str = "user:update";
const PERM_USER_DELETE: &str = "user:delete";
const PERM_USER_ASSIGN_ROLES: &str = "user:assign_roles";

const PERM_ROLE_CREATE: &str = "role:create";
const PERM_ROLE_READ: &str = "role:read";
const PERM_ROLE_UPDATE: &str = "role:update";
const PERM_ROLE_DELETE: &str = "role:delete";
const PERM_ROLE_ASSIGN_PERMISSIONS: &str = "role:assign_permissions";

#[main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================");
    println!("MMO API - Seed Default Roles");
    println!("========================================");

    // Load MongoDB URL
    let mongo_url = std::env::var("MONGODB_URL")
        .or_else(|_| std::env::var("MONGODB_URI"))
        .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());

    println!("\nConnecting to MongoDB...");
    let client = Client::with_uri_str(&mongo_url).await?;
    let db = client.database("mmo_api");
    let roles_collection: Collection<Role> = db.collection("roles");

    // Define default roles
    let default_roles = vec![
        // BUYER - Level 0
        Role {
            id: None,
            name: "BUYER".to_string(),
            display_name: "Buyer".to_string(),
            level: 0,
            parent_role_id: None,
            inherits_from: vec![],
            direct_permissions: vec![],
            flattened_permissions: vec![
                PERM_PRODUCT_LIST.to_string(),
                PERM_PRODUCT_READ.to_string(),
                PERM_ORDER_CREATE.to_string(),
                PERM_ORDER_READ.to_string(),
                PERM_ORDER_LIST.to_string(),
                PERM_WALLET_READ.to_string(),
                PERM_WALLET_DEPOSIT.to_string(),
            ],
            is_system: true,
            is_active: true,
            version: 1,
            created_at: mongodb::bson::DateTime::now(),
            updated_at: mongodb::bson::DateTime::now(),
        },
        // SELLER - Level 1
        Role {
            id: None,
            name: "SELLER".to_string(),
            display_name: "Seller".to_string(),
            level: 1,
            parent_role_id: None,
            inherits_from: vec!["BUYER".to_string()],
            direct_permissions: vec![],
            flattened_permissions: vec![
                // Inherited from BUYER
                PERM_PRODUCT_LIST.to_string(),
                PERM_PRODUCT_READ.to_string(),
                PERM_ORDER_CREATE.to_string(),
                PERM_ORDER_READ.to_string(),
                PERM_ORDER_LIST.to_string(),
                PERM_WALLET_READ.to_string(),
                PERM_WALLET_DEPOSIT.to_string(),
                // SELLER-specific
                PERM_PRODUCT_CREATE.to_string(),
                PERM_PRODUCT_UPDATE.to_string(),
                PERM_PRODUCT_DELETE.to_string(),
                PERM_ORDER_UPDATE.to_string(),
                PERM_WALLET_WITHDRAW.to_string(),
            ],
            is_system: true,
            is_active: true,
            version: 1,
            created_at: mongodb::bson::DateTime::now(),
            updated_at: mongodb::bson::DateTime::now(),
        },
        // ADMIN - Level 2
        Role {
            id: None,
            name: "ADMIN".to_string(),
            display_name: "Administrator".to_string(),
            level: 2,
            parent_role_id: None,
            inherits_from: vec!["SELLER".to_string()],
            direct_permissions: vec![],
            flattened_permissions: vec![
                // All product permissions
                PERM_PRODUCT_CREATE.to_string(),
                PERM_PRODUCT_READ.to_string(),
                PERM_PRODUCT_UPDATE.to_string(),
                PERM_PRODUCT_DELETE.to_string(),
                PERM_PRODUCT_LIST.to_string(),
                // All order permissions
                PERM_ORDER_CREATE.to_string(),
                PERM_ORDER_READ.to_string(),
                PERM_ORDER_UPDATE.to_string(),
                PERM_ORDER_CANCEL.to_string(),
                PERM_ORDER_LIST.to_string(),
                // All wallet permissions
                PERM_WALLET_READ.to_string(),
                PERM_WALLET_WITHDRAW.to_string(),
                PERM_WALLET_DEPOSIT.to_string(),
                PERM_WALLET_LIST.to_string(),
                // User management
                PERM_USER_READ.to_string(),
                PERM_USER_UPDATE.to_string(),
                PERM_USER_ASSIGN_ROLES.to_string(),
                // Role management
                PERM_ROLE_READ.to_string(),
            ],
            is_system: true,
            is_active: true,
            version: 1,
            created_at: mongodb::bson::DateTime::now(),
            updated_at: mongodb::bson::DateTime::now(),
        },
        // SUPER_ADMIN - Level 3
        Role {
            id: None,
            name: "SUPER_ADMIN".to_string(),
            display_name: "Super Administrator".to_string(),
            level: 3,
            parent_role_id: None,
            inherits_from: vec![],
            direct_permissions: vec![],
            flattened_permissions: vec![
                // All permissions
                PERM_PRODUCT_CREATE.to_string(),
                PERM_PRODUCT_READ.to_string(),
                PERM_PRODUCT_UPDATE.to_string(),
                PERM_PRODUCT_DELETE.to_string(),
                PERM_PRODUCT_LIST.to_string(),
                PERM_ORDER_CREATE.to_string(),
                PERM_ORDER_READ.to_string(),
                PERM_ORDER_UPDATE.to_string(),
                PERM_ORDER_CANCEL.to_string(),
                PERM_ORDER_LIST.to_string(),
                PERM_WALLET_READ.to_string(),
                PERM_WALLET_WITHDRAW.to_string(),
                PERM_WALLET_DEPOSIT.to_string(),
                PERM_WALLET_LIST.to_string(),
                PERM_USER_CREATE.to_string(),
                PERM_USER_READ.to_string(),
                PERM_USER_UPDATE.to_string(),
                PERM_USER_DELETE.to_string(),
                PERM_USER_ASSIGN_ROLES.to_string(),
                PERM_ROLE_CREATE.to_string(),
                PERM_ROLE_READ.to_string(),
                PERM_ROLE_UPDATE.to_string(),
                PERM_ROLE_DELETE.to_string(),
                PERM_ROLE_ASSIGN_PERMISSIONS.to_string(),
            ],
            is_system: true,
            is_active: true,
            version: 1,
            created_at: mongodb::bson::DateTime::now(),
            updated_at: mongodb::bson::DateTime::now(),
        },
    ];

    // Insert roles
    println!("\nSeeding roles...");
    for role in &default_roles {
        // Check if exists
        let existing = roles_collection
            .find_one(doc! { "name": &role.name })
            .await?;

        if existing.is_some() {
            println!("  Role '{}' already exists, skipping...", role.name);
        } else {
            roles_collection.insert_one(role).await?;
            println!("  Created role: {} (level: {}, {} permissions)",
                role.name, role.level, role.flattened_permissions.len());
        }
    }

    println!("\n========================================");
    println!("Role seeding completed!");
    println!("========================================");
    println!("\nDefault roles:");
    for role in &default_roles {
        println!("  - {} (level: {}, {} permissions)",
            role.name, role.level, role.flattened_permissions.len());
    }
    println!();

    Ok(())
}
