//! Integration tests for the Permission/Authorization system
//!
//! These tests verify the full authorization flow:
//! 1. Create roles with permissions
//! 2. Assign roles to users
//! 3. Verify permission checks work correctly
//!
//! NOTE: These tests require MongoDB and Redis to be running.
//! Run with: cargo test --test permissions_integration_test -- --ignored

use mongodb::{bson::doc, Client, Collection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestRole {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<bson::oid::ObjectId>,
    name: String,
    display_name: String,
    level: i32,
    parent_role_id: Option<bson::oid::ObjectId>,
    inherits_from: Vec<String>,
    direct_permissions: Vec<bson::oid::ObjectId>,
    flattened_permissions: Vec<String>,
    is_system: bool,
    is_active: bool,
    version: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    created_at: Option<mongodb::bson::DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    updated_at: Option<mongodb::bson::DateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestUser {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<bson::oid::ObjectId>,
    email: String,
    username: String,
    roles: Vec<String>,
    perm_version: i32,
}

async fn get_test_db() -> mongodb::Database {
    let mongo_url = std::env::var("MONGODB_URI")
        .or_else(|_| std::env::var("MONGODB_URL"))
        .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    
    let client = Client::with_uri_str(&mongo_url)
        .await
        .expect("Failed to connect to MongoDB");
    
    client.database("mmo_api_test")
}

async fn cleanup_test_data(db: &mongodb::Database) {
    let _ = db.collection::<TestRole>("roles")
        .delete_many(doc! { "name": { "$regex": "^TEST_" } })
        .await;
    let _ = db.collection::<TestUser>("users")
        .delete_many(doc! { "email": { "$regex": "^test_" } })
        .await;
}

#[tokio::test]
#[ignore]
async fn test_create_role_and_verify_permissions() {
    let db = get_test_db().await;
    cleanup_test_data(&db).await;
    
    let roles: Collection<TestRole> = db.collection("roles");
    
    let test_role = TestRole {
        id: None,
        name: "TEST_CUSTOM_ROLE".to_string(),
        display_name: "Test Custom Role".to_string(),
        level: 1,
        parent_role_id: None,
        inherits_from: vec![],
        direct_permissions: vec![],
        flattened_permissions: vec![
            "product:create".to_string(),
            "product:read".to_string(),
            "order:read".to_string(),
        ],
        is_system: false,
        is_active: true,
        version: 1,
        created_at: Some(mongodb::bson::DateTime::now()),
        updated_at: Some(mongodb::bson::DateTime::now()),
    };
    
    roles.insert_one(&test_role).await.expect("Failed to insert role");
    
    let found = roles.find_one(doc! { "name": "TEST_CUSTOM_ROLE" })
        .await
        .expect("Query failed")
        .expect("Role not found");
    
    assert_eq!(found.name, "TEST_CUSTOM_ROLE");
    assert_eq!(found.flattened_permissions.len(), 3);
    assert!(found.flattened_permissions.contains(&"product:create".to_string()));
    assert!(found.flattened_permissions.contains(&"product:read".to_string()));
    assert!(found.flattened_permissions.contains(&"order:read".to_string()));
    
    cleanup_test_data(&db).await;
}

#[tokio::test]
#[ignore]
async fn test_assign_role_to_user() {
    let db = get_test_db().await;
    cleanup_test_data(&db).await;
    
    let users: Collection<TestUser> = db.collection("users");
    let roles: Collection<TestRole> = db.collection("roles");
    
    let test_role = TestRole {
        id: None,
        name: "TEST_SELLER".to_string(),
        display_name: "Test Seller".to_string(),
        level: 1,
        parent_role_id: None,
        inherits_from: vec!["TEST_BUYER".to_string()],
        direct_permissions: vec![],
        flattened_permissions: vec![
            "product:create".to_string(),
            "product:read".to_string(),
            "product:update".to_string(),
            "product:delete".to_string(),
        ],
        is_system: false,
        is_active: true,
        version: 1,
        created_at: Some(mongodb::bson::DateTime::now()),
        updated_at: Some(mongodb::bson::DateTime::now()),
    };
    roles.insert_one(&test_role).await.expect("Failed to insert role");
    
    let test_user = TestUser {
        id: None,
        email: "test_user@example.com".to_string(),
        username: "test_user".to_string(),
        roles: vec!["TEST_BUYER".to_string()],
        perm_version: 1,
    };
    users.insert_one(&test_user).await.expect("Failed to insert user");
    
    users.update_one(
        doc! { "email": "test_user@example.com" },
        doc! { 
            "$addToSet": { "roles": "TEST_SELLER" },
            "$inc": { "perm_version": 1 }
        },
    ).await.expect("Failed to assign role");
    
    let updated_user = users.find_one(doc! { "email": "test_user@example.com" })
        .await
        .expect("Query failed")
        .expect("User not found");
    
    assert!(updated_user.roles.contains(&"TEST_BUYER".to_string()));
    assert!(updated_user.roles.contains(&"TEST_SELLER".to_string()));
    assert_eq!(updated_user.perm_version, 2);
    
    cleanup_test_data(&db).await;
}

#[tokio::test]
#[ignore]
async fn test_role_hierarchy_permissions() {
    let db = get_test_db().await;
    cleanup_test_data(&db).await;
    
    let roles: Collection<TestRole> = db.collection("roles");
    
    let buyer_perms = vec![
        "product:read".to_string(),
        "product:list".to_string(),
        "order:create".to_string(),
        "order:read".to_string(),
    ];
    
    let buyer_role = TestRole {
        id: None,
        name: "TEST_BUYER".to_string(),
        display_name: "Test Buyer".to_string(),
        level: 0,
        parent_role_id: None,
        inherits_from: vec![],
        direct_permissions: vec![],
        flattened_permissions: buyer_perms.clone(),
        is_system: false,
        is_active: true,
        version: 1,
        created_at: Some(mongodb::bson::DateTime::now()),
        updated_at: Some(mongodb::bson::DateTime::now()),
    };
    roles.insert_one(&buyer_role).await.expect("Failed to insert buyer role");
    
    let mut seller_perms = buyer_perms.clone();
    seller_perms.extend(vec![
        "product:create".to_string(),
        "product:update".to_string(),
        "product:delete".to_string(),
    ]);
    
    let seller_role = TestRole {
        id: None,
        name: "TEST_SELLER".to_string(),
        display_name: "Test Seller".to_string(),
        level: 1,
        parent_role_id: None,
        inherits_from: vec!["TEST_BUYER".to_string()],
        direct_permissions: vec![],
        flattened_permissions: seller_perms.clone(),
        is_system: false,
        is_active: true,
        version: 1,
        created_at: Some(mongodb::bson::DateTime::now()),
        updated_at: Some(mongodb::bson::DateTime::now()),
    };
    roles.insert_one(&seller_role).await.expect("Failed to insert seller role");
    
    let buyer = roles.find_one(doc! { "name": "TEST_BUYER" })
        .await.expect("Query failed").expect("Buyer not found");
    let seller = roles.find_one(doc! { "name": "TEST_SELLER" })
        .await.expect("Query failed").expect("Seller not found");
    
    assert!(buyer.level < seller.level);
    
    assert!(buyer.flattened_permissions.contains(&"product:read".to_string()));
    assert!(!buyer.flattened_permissions.contains(&"product:create".to_string()));
    
    assert!(seller.flattened_permissions.contains(&"product:read".to_string()));
    assert!(seller.flattened_permissions.contains(&"product:create".to_string()));
    assert!(seller.flattened_permissions.contains(&"product:update".to_string()));
    
    cleanup_test_data(&db).await;
}

#[tokio::test]
#[ignore]
async fn test_update_role_permissions() {
    let db = get_test_db().await;
    cleanup_test_data(&db).await;
    
    let roles: Collection<TestRole> = db.collection("roles");
    
    let test_role = TestRole {
        id: None,
        name: "TEST_UPDATABLE_ROLE".to_string(),
        display_name: "Test Updatable Role".to_string(),
        level: 1,
        parent_role_id: None,
        inherits_from: vec![],
        direct_permissions: vec![],
        flattened_permissions: vec!["product:read".to_string()],
        is_system: false,
        is_active: true,
        version: 1,
        created_at: Some(mongodb::bson::DateTime::now()),
        updated_at: Some(mongodb::bson::DateTime::now()),
    };
    roles.insert_one(&test_role).await.expect("Failed to insert role");
    
    let new_permissions = vec![
        "product:read".to_string(),
        "product:create".to_string(),
        "product:update".to_string(),
    ];
    
    roles.update_one(
        doc! { "name": "TEST_UPDATABLE_ROLE" },
        doc! { 
            "$set": { 
                "flattened_permissions": &new_permissions,
                "updated_at": mongodb::bson::DateTime::now()
            },
            "$inc": { "version": 1 }
        },
    ).await.expect("Failed to update role");
    
    let updated = roles.find_one(doc! { "name": "TEST_UPDATABLE_ROLE" })
        .await.expect("Query failed").expect("Role not found");
    
    assert_eq!(updated.flattened_permissions.len(), 3);
    assert_eq!(updated.version, 2);
    assert!(updated.flattened_permissions.contains(&"product:create".to_string()));
    
    cleanup_test_data(&db).await;
}

#[tokio::test]
#[ignore]
async fn test_delete_role_soft_delete() {
    let db = get_test_db().await;
    cleanup_test_data(&db).await;
    
    let roles: Collection<TestRole> = db.collection("roles");
    
    let test_role = TestRole {
        id: None,
        name: "TEST_DELETABLE_ROLE".to_string(),
        display_name: "Test Deletable Role".to_string(),
        level: 1,
        parent_role_id: None,
        inherits_from: vec![],
        direct_permissions: vec![],
        flattened_permissions: vec!["product:read".to_string()],
        is_system: false,
        is_active: true,
        version: 1,
        created_at: Some(mongodb::bson::DateTime::now()),
        updated_at: Some(mongodb::bson::DateTime::now()),
    };
    roles.insert_one(&test_role).await.expect("Failed to insert role");
    
    roles.update_one(
        doc! { "name": "TEST_DELETABLE_ROLE" },
        doc! { "$set": { "is_active": false } },
    ).await.expect("Failed to soft delete role");
    
    let deleted = roles.find_one(doc! { "name": "TEST_DELETABLE_ROLE", "is_active": true })
        .await.expect("Query failed");
    assert!(deleted.is_none());
    
    let still_exists = roles.find_one(doc! { "name": "TEST_DELETABLE_ROLE" })
        .await.expect("Query failed");
    assert!(still_exists.is_some());
    assert!(!still_exists.unwrap().is_active);
    
    cleanup_test_data(&db).await;
}

#[tokio::test]
#[ignore]
async fn test_full_authorization_flow() {
    let db = get_test_db().await;
    cleanup_test_data(&db).await;
    
    let roles: Collection<TestRole> = db.collection("roles");
    let users: Collection<TestUser> = db.collection("users");
    
    let buyer_role = TestRole {
        id: None,
        name: "TEST_BUYER".to_string(),
        display_name: "Test Buyer".to_string(),
        level: 0,
        parent_role_id: None,
        inherits_from: vec![],
        direct_permissions: vec![],
        flattened_permissions: vec![
            "product:read".to_string(),
            "order:create".to_string(),
        ],
        is_system: true,
        is_active: true,
        version: 1,
        created_at: Some(mongodb::bson::DateTime::now()),
        updated_at: Some(mongodb::bson::DateTime::now()),
    };
    
    let seller_role = TestRole {
        id: None,
        name: "TEST_SELLER".to_string(),
        display_name: "Test Seller".to_string(),
        level: 1,
        parent_role_id: None,
        inherits_from: vec!["TEST_BUYER".to_string()],
        direct_permissions: vec![],
        flattened_permissions: vec![
            "product:read".to_string(),
            "product:create".to_string(),
            "product:update".to_string(),
            "product:delete".to_string(),
            "order:create".to_string(),
            "order:read".to_string(),
        ],
        is_system: true,
        is_active: true,
        version: 1,
        created_at: Some(mongodb::bson::DateTime::now()),
        updated_at: Some(mongodb::bson::DateTime::now()),
    };
    
    roles.insert_one(&buyer_role).await.expect("Failed to create buyer role");
    roles.insert_one(&seller_role).await.expect("Failed to create seller role");
    
    let buyer_user = TestUser {
        id: None,
        email: "test_buyer@example.com".to_string(),
        username: "test_buyer".to_string(),
        roles: vec!["TEST_BUYER".to_string()],
        perm_version: 1,
    };
    
    let seller_user = TestUser {
        id: None,
        email: "test_seller@example.com".to_string(),
        username: "test_seller".to_string(),
        roles: vec!["TEST_SELLER".to_string()],
        perm_version: 1,
    };
    
    users.insert_one(&buyer_user).await.expect("Failed to create buyer");
    users.insert_one(&seller_user).await.expect("Failed to create seller");
    
    let buyer = users.find_one(doc! { "email": "test_buyer@example.com" })
        .await.expect("Query failed").expect("Buyer not found");
    let seller = users.find_one(doc! { "email": "test_seller@example.com" })
        .await.expect("Query failed").expect("Seller not found");
    
    let buyer_role_doc = roles.find_one(doc! { "name": "TEST_BUYER", "is_active": true })
        .await.expect("Query failed").expect("Buyer role not found");
    let seller_role_doc = roles.find_one(doc! { "name": "TEST_SELLER", "is_active": true })
        .await.expect("Query failed").expect("Seller role not found");
    
    assert!(buyer.roles.contains(&"TEST_BUYER".to_string()));
    assert!(buyer_role_doc.flattened_permissions.contains(&"product:read".to_string()));
    assert!(!buyer_role_doc.flattened_permissions.contains(&"product:create".to_string()));
    
    assert!(seller.roles.contains(&"TEST_SELLER".to_string()));
    assert!(seller_role_doc.flattened_permissions.contains(&"product:read".to_string()));
    assert!(seller_role_doc.flattened_permissions.contains(&"product:create".to_string()));
    
    users.update_one(
        doc! { "email": "test_buyer@example.com" },
        doc! { 
            "$addToSet": { "roles": "TEST_SELLER" },
            "$inc": { "perm_version": 1 }
        },
    ).await.expect("Failed to upgrade user");
    
    let upgraded_buyer = users.find_one(doc! { "email": "test_buyer@example.com" })
        .await.expect("Query failed").expect("User not found");
    
    assert!(upgraded_buyer.roles.contains(&"TEST_BUYER".to_string()));
    assert!(upgraded_buyer.roles.contains(&"TEST_SELLER".to_string()));
    assert_eq!(upgraded_buyer.perm_version, 2);
    
    cleanup_test_data(&db).await;
}
