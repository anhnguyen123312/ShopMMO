//! Seed Test Data Script
//!
//! Creates test users matching Swagger examples for API testing.
//!
//! Run with:
//!   cargo run --bin seed_test_data

use bson::{oid::ObjectId, DateTime};
use mongodb::{bson::doc, Client, Collection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum UserStatus {
    Active,
    Suspended,
    PendingVerification,
}

#[derive(Debug, Serialize, Deserialize)]
struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    username: String,
    email: String,
    password_hash: String,
    name: String,
    role: String,
    #[serde(default)]
    roles: Vec<String>,
    #[serde(default)]
    perm_version: u32,
    status: UserStatus,
    email_verified: bool,
    last_login_at: Option<DateTime>,
    created_at: DateTime,
    updated_at: DateTime,
}

#[derive(Debug, Serialize, Deserialize)]
struct Wallet {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    wallet_id: String,
    user_id: ObjectId,
    wallet_type: String,
    available_trust: i64,
    withdrawal_locked: i64,
    dispute_locked: i64,
    total_trust: i64,
    lifetime_deposited: i64,
    lifetime_withdrawn: i64,
    lifetime_spent: i64,
    lifetime_received: i64,
    commission_debt: i64,
    commission_rate: Option<f64>,
    admin_debt: i64,
    admin_debt_reason: Option<String>,
    last_snapshot_month: Option<String>,
    last_snapshot_balance: Option<i64>,
    status: String,
    freeze_reason: Option<String>,
    created_at: DateTime,
    updated_at: DateTime,
}

fn hash_password(password: &str) -> String {
    bcrypt::hash(password, 12).expect("Failed to hash password")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================");
    println!("MMO API - Seed Test Data");
    println!("========================================");
    println!("\nThis creates test users matching Swagger examples:");
    println!("  - johndoe123 / SecurePass123! (BUYER)");
    println!("  - seller001 / SecurePass123! (SELLER)");
    println!("  - admin001 / AdminPass123! (ADMIN)");
    println!("  - superadmin / SuperPass123! (SUPER_ADMIN)");

    let mongo_url = std::env::var("MONGODB_URL")
        .or_else(|_| std::env::var("MONGODB_URI"))
        .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());

    let db_name = std::env::var("MONGODB_DATABASE").unwrap_or_else(|_| "mmo_db".to_string());
    println!("\nConnecting to MongoDB at {}...", mongo_url);
    println!("Database: {}", db_name);
    let client = Client::with_uri_str(&mongo_url).await?;
    let db = client.database(&db_name);
    let users_collection: Collection<User> = db.collection("users");
    let wallets_collection: Collection<Wallet> = db.collection("wallets");

    let now = DateTime::now();

    let test_users = vec![
        // BUYER - matches Swagger login example
        User {
            id: None,
            username: "johndoe123".to_string(),
            email: "john.doe@example.com".to_string(),
            password_hash: hash_password("SecurePass123!"),
            name: "John Doe".to_string(),
            role: "BUYER".to_string(),
            roles: vec!["BUYER".to_string()],
            perm_version: 1,
            status: UserStatus::Active,
            email_verified: true,
            last_login_at: None,
            created_at: now,
            updated_at: now,
        },
        // SELLER
        User {
            id: None,
            username: "seller001".to_string(),
            email: "seller@example.com".to_string(),
            password_hash: hash_password("SecurePass123!"),
            name: "Test Seller".to_string(),
            role: "SELLER".to_string(),
            roles: vec!["BUYER".to_string(), "SELLER".to_string()],
            perm_version: 1,
            status: UserStatus::Active,
            email_verified: true,
            last_login_at: None,
            created_at: now,
            updated_at: now,
        },
        // ADMIN
        User {
            id: None,
            username: "admin001".to_string(),
            email: "admin@example.com".to_string(),
            password_hash: hash_password("AdminPass123!"),
            name: "Test Admin".to_string(),
            role: "ADMIN".to_string(),
            roles: vec!["BUYER".to_string(), "SELLER".to_string(), "ADMIN".to_string()],
            perm_version: 1,
            status: UserStatus::Active,
            email_verified: true,
            last_login_at: None,
            created_at: now,
            updated_at: now,
        },
        // SUPER_ADMIN
        User {
            id: None,
            username: "superadmin".to_string(),
            email: "superadmin@example.com".to_string(),
            password_hash: hash_password("SuperPass123!"),
            name: "Super Admin".to_string(),
            role: "SUPER_ADMIN".to_string(),
            roles: vec!["SUPER_ADMIN".to_string()],
            perm_version: 1,
            status: UserStatus::Active,
            email_verified: true,
            last_login_at: None,
            created_at: now,
            updated_at: now,
        },
    ];

    println!("\nSeeding users...");
    for user in &test_users {
        let existing = users_collection
            .find_one(doc! { "username": &user.username })
            .await?;

        if existing.is_some() {
            println!("  User '{}' already exists, skipping...", user.username);
        } else {
            let result = users_collection.insert_one(user).await?;
            let user_id = result.inserted_id.as_object_id().unwrap();
            println!("  Created user: {} ({}) - ID: {}", user.username, user.role, user_id);

            // Create wallet for user
            let wallet = Wallet {
                id: None,
                wallet_id: format!("WLT-{}", user_id.to_hex()),
                user_id,
                wallet_type: if user.role == "SELLER" { "Seller".to_string() } else { "User".to_string() },
                available_trust: 100000, // 100k Trust for testing
                withdrawal_locked: 0,
                dispute_locked: 0,
                total_trust: 100000,
                lifetime_deposited: 100000,
                lifetime_withdrawn: 0,
                lifetime_spent: 0,
                lifetime_received: 0,
                commission_debt: 0,
                commission_rate: if user.role == "SELLER" { Some(5.0) } else { None },
                admin_debt: 0,
                admin_debt_reason: None,
                last_snapshot_month: None,
                last_snapshot_balance: None,
                status: "Active".to_string(),
                freeze_reason: None,
                created_at: now,
                updated_at: now,
            };

            wallets_collection.insert_one(&wallet).await?;
            println!("    Created wallet: {} (100,000 Trust)", wallet.wallet_id);
        }
    }

    println!("\n========================================");
    println!("Test data seeding completed!");
    println!("========================================");
    println!("\nTest Accounts:");
    println!("┌─────────────────┬───────────────────┬─────────────┐");
    println!("│ Username        │ Password          │ Role        │");
    println!("├─────────────────┼───────────────────┼─────────────┤");
    println!("│ johndoe123      │ SecurePass123!    │ BUYER       │");
    println!("│ seller001       │ SecurePass123!    │ SELLER      │");
    println!("│ admin001        │ AdminPass123!     │ ADMIN       │");
    println!("│ superadmin      │ SuperPass123!     │ SUPER_ADMIN │");
    println!("└─────────────────┴───────────────────┴─────────────┘");
    println!("\nAll users have 100,000 Trust in their wallets.");
    println!("\nSwagger Login Example:");
    println!("  POST /api/auth/login");
    println!("  {{\"identifier\": \"johndoe123\", \"password\": \"SecurePass123!\"}}");
    println!();

    Ok(())
}
