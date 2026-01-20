//! Integration tests for the Authentication system
//!
//! Run with: cargo test --test auth_integration_test -- --ignored

use bson::{doc, oid::ObjectId, DateTime};
use mongodb::{Client, Collection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestUser {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    username: String,
    email: String,
    password_hash: String,
    name: String,
    role: String,
    roles: Vec<String>,
    perm_version: i32,
    status: String,
    email_verified: bool,
    last_login_at: Option<DateTime>,
    created_at: DateTime,
    updated_at: DateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestRefreshToken {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    user_id: ObjectId,
    token: String,
    expires_at: DateTime,
    revoked: bool,
    created_at: DateTime,
    ip_address: Option<String>,
    user_agent: Option<String>,
}

async fn get_test_db() -> mongodb::Database {
    let mongo_url = std::env::var("MONGODB_URI")
        .or_else(|_| std::env::var("MONGODB_URL"))
        .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    
    let client = Client::with_uri_str(&mongo_url)
        .await
        .expect("Failed to connect to MongoDB");
    
    client.database("mmo_api_auth_test")
}

async fn cleanup_test_data(db: &mongodb::Database) {
    let _ = db.collection::<TestUser>("users")
        .delete_many(doc! { "email": { "$regex": "^test_" } })
        .await;
    let _ = db.collection::<TestRefreshToken>("refresh_tokens")
        .delete_many(doc! { "token": { "$regex": "^TEST_" } })
        .await;
}

fn generate_test_id(prefix: &str) -> String {
    format!("TEST_{}_{}", prefix, ObjectId::new().to_hex())
}

#[tokio::test]
#[ignore]
async fn test_create_user() {
    let db = get_test_db().await;
    cleanup_test_data(&db).await;
    
    let users: Collection<TestUser> = db.collection("users");
    
    let user = TestUser {
        id: None,
        username: "test_newuser".to_string(),
        email: "test_newuser@example.com".to_string(),
        password_hash: "hashed_password_here".to_string(),
        name: "Test New User".to_string(),
        role: "BUYER".to_string(),
        roles: vec!["BUYER".to_string()],
        perm_version: 1,
        status: "pending_verification".to_string(),
        email_verified: false,
        last_login_at: None,
        created_at: DateTime::now(),
        updated_at: DateTime::now(),
    };
    
    users.insert_one(&user).await.expect("Failed to insert user");
    
    let found = users.find_one(doc! { "email": "test_newuser@example.com" })
        .await
        .expect("Query failed")
        .expect("User not found");
    
    assert_eq!(found.username, "test_newuser");
    assert_eq!(found.email, "test_newuser@example.com");
    assert_eq!(found.role, "BUYER");
    assert_eq!(found.roles, vec!["BUYER"]);
    assert_eq!(found.status, "pending_verification");
    assert!(!found.email_verified);
    
    cleanup_test_data(&db).await;
}

#[tokio::test]
#[ignore]
async fn test_unique_email_constraint() {
    let db = get_test_db().await;
    cleanup_test_data(&db).await;
    
    let users: Collection<TestUser> = db.collection("users");
    
    let _ = users.create_index(
        mongodb::IndexModel::builder()
            .keys(doc! { "email": 1 })
            .options(mongodb::options::IndexOptions::builder().unique(true).build())
            .build()
    ).await;
    
    let user1 = TestUser {
        id: None,
        username: "test_user1".to_string(),
        email: "test_duplicate@example.com".to_string(),
        password_hash: "hash1".to_string(),
        name: "User One".to_string(),
        role: "BUYER".to_string(),
        roles: vec!["BUYER".to_string()],
        perm_version: 1,
        status: "active".to_string(),
        email_verified: true,
        last_login_at: None,
        created_at: DateTime::now(),
        updated_at: DateTime::now(),
    };
    
    users.insert_one(&user1).await.expect("Failed to insert first user");
    
    let user2 = TestUser {
        id: None,
        username: "test_user2".to_string(),
        email: "test_duplicate@example.com".to_string(),
        password_hash: "hash2".to_string(),
        name: "User Two".to_string(),
        role: "BUYER".to_string(),
        roles: vec!["BUYER".to_string()],
        perm_version: 1,
        status: "active".to_string(),
        email_verified: true,
        last_login_at: None,
        created_at: DateTime::now(),
        updated_at: DateTime::now(),
    };
    
    let result = users.insert_one(&user2).await;
    assert!(result.is_err());
    
    cleanup_test_data(&db).await;
}

#[tokio::test]
#[ignore]
async fn test_user_login_update() {
    let db = get_test_db().await;
    cleanup_test_data(&db).await;
    
    let users: Collection<TestUser> = db.collection("users");
    
    let user = TestUser {
        id: None,
        username: "test_loginuser".to_string(),
        email: "test_loginuser@example.com".to_string(),
        password_hash: "hashed_password".to_string(),
        name: "Login User".to_string(),
        role: "BUYER".to_string(),
        roles: vec!["BUYER".to_string()],
        perm_version: 1,
        status: "active".to_string(),
        email_verified: true,
        last_login_at: None,
        created_at: DateTime::now(),
        updated_at: DateTime::now(),
    };
    
    users.insert_one(&user).await.expect("Failed to insert user");
    
    let login_time = DateTime::now();
    users.update_one(
        doc! { "email": "test_loginuser@example.com" },
        doc! {
            "$set": {
                "last_login_at": login_time,
                "updated_at": DateTime::now()
            }
        },
    ).await.expect("Failed to update login time");
    
    let found = users.find_one(doc! { "email": "test_loginuser@example.com" })
        .await
        .expect("Query failed")
        .expect("User not found");
    
    assert!(found.last_login_at.is_some());
    
    cleanup_test_data(&db).await;
}

#[tokio::test]
#[ignore]
async fn test_create_refresh_token() {
    let db = get_test_db().await;
    cleanup_test_data(&db).await;
    
    let tokens: Collection<TestRefreshToken> = db.collection("refresh_tokens");
    let user_id = ObjectId::new();
    let token_str = generate_test_id("TOKEN");
    
    let future_time = DateTime::from_millis(
        DateTime::now().timestamp_millis() + 7 * 24 * 60 * 60 * 1000
    );
    
    let token = TestRefreshToken {
        id: None,
        user_id,
        token: token_str.clone(),
        expires_at: future_time,
        revoked: false,
        created_at: DateTime::now(),
        ip_address: Some("192.168.1.1".to_string()),
        user_agent: Some("Mozilla/5.0".to_string()),
    };
    
    tokens.insert_one(&token).await.expect("Failed to insert token");
    
    let found = tokens.find_one(doc! { "token": &token_str })
        .await
        .expect("Query failed")
        .expect("Token not found");
    
    assert_eq!(found.user_id, user_id);
    assert_eq!(found.token, token_str);
    assert!(!found.revoked);
    assert_eq!(found.ip_address, Some("192.168.1.1".to_string()));
    
    cleanup_test_data(&db).await;
}

#[tokio::test]
#[ignore]
async fn test_revoke_refresh_token() {
    let db = get_test_db().await;
    cleanup_test_data(&db).await;
    
    let tokens: Collection<TestRefreshToken> = db.collection("refresh_tokens");
    let token_str = generate_test_id("TOKEN");
    
    let token = TestRefreshToken {
        id: None,
        user_id: ObjectId::new(),
        token: token_str.clone(),
        expires_at: DateTime::from_millis(
            DateTime::now().timestamp_millis() + 3600 * 1000
        ),
        revoked: false,
        created_at: DateTime::now(),
        ip_address: None,
        user_agent: None,
    };
    
    tokens.insert_one(&token).await.expect("Failed to insert token");
    
    tokens.update_one(
        doc! { "token": &token_str },
        doc! { "$set": { "revoked": true } },
    ).await.expect("Failed to revoke token");
    
    let revoked = tokens.find_one(doc! { "token": &token_str })
        .await
        .expect("Query failed")
        .expect("Token not found");
    
    assert!(revoked.revoked);
    
    cleanup_test_data(&db).await;
}

#[tokio::test]
#[ignore]
async fn test_find_valid_token() {
    let db = get_test_db().await;
    cleanup_test_data(&db).await;
    
    let tokens: Collection<TestRefreshToken> = db.collection("refresh_tokens");
    let user_id = ObjectId::new();
    
    let valid_token_str = generate_test_id("TOKEN");
    let expired_token_str = generate_test_id("TOKEN");
    let revoked_token_str = generate_test_id("TOKEN");
    
    let future = DateTime::from_millis(
        DateTime::now().timestamp_millis() + 3600 * 1000
    );
    let past = DateTime::from_millis(
        DateTime::now().timestamp_millis() - 3600 * 1000
    );
    
    let valid_token = TestRefreshToken {
        id: None,
        user_id,
        token: valid_token_str.clone(),
        expires_at: future,
        revoked: false,
        created_at: DateTime::now(),
        ip_address: None,
        user_agent: None,
    };
    
    let expired_token = TestRefreshToken {
        id: None,
        user_id,
        token: expired_token_str.clone(),
        expires_at: past,
        revoked: false,
        created_at: DateTime::now(),
        ip_address: None,
        user_agent: None,
    };
    
    let revoked_token = TestRefreshToken {
        id: None,
        user_id,
        token: revoked_token_str.clone(),
        expires_at: future,
        revoked: true,
        created_at: DateTime::now(),
        ip_address: None,
        user_agent: None,
    };
    
    tokens.insert_one(&valid_token).await.expect("Failed to insert valid token");
    tokens.insert_one(&expired_token).await.expect("Failed to insert expired token");
    tokens.insert_one(&revoked_token).await.expect("Failed to insert revoked token");
    
    let valid_result = tokens.find_one(doc! {
        "token": &valid_token_str,
        "revoked": false,
        "expires_at": { "$gt": DateTime::now() }
    }).await.expect("Query failed");
    assert!(valid_result.is_some());
    
    let expired_result = tokens.find_one(doc! {
        "token": &expired_token_str,
        "revoked": false,
        "expires_at": { "$gt": DateTime::now() }
    }).await.expect("Query failed");
    assert!(expired_result.is_none());
    
    let revoked_result = tokens.find_one(doc! {
        "token": &revoked_token_str,
        "revoked": false,
        "expires_at": { "$gt": DateTime::now() }
    }).await.expect("Query failed");
    assert!(revoked_result.is_none());
    
    cleanup_test_data(&db).await;
}

#[tokio::test]
#[ignore]
async fn test_revoke_all_user_tokens() {
    let db = get_test_db().await;
    cleanup_test_data(&db).await;
    
    let tokens: Collection<TestRefreshToken> = db.collection("refresh_tokens");
    let user_id = ObjectId::new();
    
    let future = DateTime::from_millis(
        DateTime::now().timestamp_millis() + 3600 * 1000
    );
    
    for i in 0..3 {
        let token = TestRefreshToken {
            id: None,
            user_id,
            token: generate_test_id(&format!("TOKEN{}", i)),
            expires_at: future,
            revoked: false,
            created_at: DateTime::now(),
            ip_address: None,
            user_agent: None,
        };
        tokens.insert_one(&token).await.expect("Failed to insert token");
    }
    
    let result = tokens.update_many(
        doc! { "user_id": user_id },
        doc! { "$set": { "revoked": true } },
    ).await.expect("Failed to revoke all tokens");
    
    assert_eq!(result.modified_count, 3);
    
    let active_tokens: Vec<TestRefreshToken> = {
        let mut cursor = tokens.find(
            doc! { "user_id": user_id, "revoked": false }
        ).await.expect("Query failed");
        
        let mut results = Vec::new();
        while cursor.advance().await.expect("Cursor error") {
            results.push(cursor.deserialize_current().expect("Deserialize error"));
        }
        results
    };
    
    assert!(active_tokens.is_empty());
    
    cleanup_test_data(&db).await;
}

#[tokio::test]
#[ignore]
async fn test_user_status_transitions() {
    let db = get_test_db().await;
    cleanup_test_data(&db).await;
    
    let users: Collection<TestUser> = db.collection("users");
    
    let user = TestUser {
        id: None,
        username: "test_statususer".to_string(),
        email: "test_statususer@example.com".to_string(),
        password_hash: "hash".to_string(),
        name: "Status User".to_string(),
        role: "BUYER".to_string(),
        roles: vec!["BUYER".to_string()],
        perm_version: 1,
        status: "pending_verification".to_string(),
        email_verified: false,
        last_login_at: None,
        created_at: DateTime::now(),
        updated_at: DateTime::now(),
    };
    
    users.insert_one(&user).await.expect("Failed to insert user");
    
    users.update_one(
        doc! { "email": "test_statususer@example.com" },
        doc! {
            "$set": {
                "status": "active",
                "email_verified": true,
                "updated_at": DateTime::now()
            }
        },
    ).await.expect("Failed to activate user");
    
    let activated = users.find_one(doc! { "email": "test_statususer@example.com" })
        .await.expect("Query failed").expect("User not found");
    assert_eq!(activated.status, "active");
    assert!(activated.email_verified);
    
    users.update_one(
        doc! { "email": "test_statususer@example.com" },
        doc! { "$set": { "status": "suspended", "updated_at": DateTime::now() } },
    ).await.expect("Failed to suspend user");
    
    let suspended = users.find_one(doc! { "email": "test_statususer@example.com" })
        .await.expect("Query failed").expect("User not found");
    assert_eq!(suspended.status, "suspended");
    
    cleanup_test_data(&db).await;
}

#[tokio::test]
#[ignore]
async fn test_assign_multiple_roles() {
    let db = get_test_db().await;
    cleanup_test_data(&db).await;
    
    let users: Collection<TestUser> = db.collection("users");
    
    let user = TestUser {
        id: None,
        username: "test_multiuser".to_string(),
        email: "test_multiuser@example.com".to_string(),
        password_hash: "hash".to_string(),
        name: "Multi Role User".to_string(),
        role: "BUYER".to_string(),
        roles: vec!["BUYER".to_string()],
        perm_version: 1,
        status: "active".to_string(),
        email_verified: true,
        last_login_at: None,
        created_at: DateTime::now(),
        updated_at: DateTime::now(),
    };
    
    users.insert_one(&user).await.expect("Failed to insert user");
    
    users.update_one(
        doc! { "email": "test_multiuser@example.com" },
        doc! {
            "$addToSet": { "roles": "SELLER" },
            "$inc": { "perm_version": 1 },
            "$set": { "updated_at": DateTime::now() }
        },
    ).await.expect("Failed to add SELLER role");
    
    users.update_one(
        doc! { "email": "test_multiuser@example.com" },
        doc! {
            "$addToSet": { "roles": "MODERATOR" },
            "$inc": { "perm_version": 1 },
            "$set": { "updated_at": DateTime::now() }
        },
    ).await.expect("Failed to add MODERATOR role");
    
    let updated = users.find_one(doc! { "email": "test_multiuser@example.com" })
        .await.expect("Query failed").expect("User not found");
    
    assert!(updated.roles.contains(&"BUYER".to_string()));
    assert!(updated.roles.contains(&"SELLER".to_string()));
    assert!(updated.roles.contains(&"MODERATOR".to_string()));
    assert_eq!(updated.roles.len(), 3);
    assert_eq!(updated.perm_version, 3);
    
    cleanup_test_data(&db).await;
}

#[tokio::test]
#[ignore]
async fn test_remove_role() {
    let db = get_test_db().await;
    cleanup_test_data(&db).await;
    
    let users: Collection<TestUser> = db.collection("users");
    
    let user = TestUser {
        id: None,
        username: "test_removeuser".to_string(),
        email: "test_removeuser@example.com".to_string(),
        password_hash: "hash".to_string(),
        name: "Remove Role User".to_string(),
        role: "BUYER".to_string(),
        roles: vec!["BUYER".to_string(), "SELLER".to_string(), "MODERATOR".to_string()],
        perm_version: 1,
        status: "active".to_string(),
        email_verified: true,
        last_login_at: None,
        created_at: DateTime::now(),
        updated_at: DateTime::now(),
    };
    
    users.insert_one(&user).await.expect("Failed to insert user");
    
    users.update_one(
        doc! { "email": "test_removeuser@example.com" },
        doc! {
            "$pull": { "roles": "MODERATOR" },
            "$inc": { "perm_version": 1 },
            "$set": { "updated_at": DateTime::now() }
        },
    ).await.expect("Failed to remove MODERATOR role");
    
    let updated = users.find_one(doc! { "email": "test_removeuser@example.com" })
        .await.expect("Query failed").expect("User not found");
    
    assert!(updated.roles.contains(&"BUYER".to_string()));
    assert!(updated.roles.contains(&"SELLER".to_string()));
    assert!(!updated.roles.contains(&"MODERATOR".to_string()));
    assert_eq!(updated.roles.len(), 2);
    assert_eq!(updated.perm_version, 2);
    
    cleanup_test_data(&db).await;
}

#[tokio::test]
#[ignore]
async fn test_change_password() {
    let db = get_test_db().await;
    cleanup_test_data(&db).await;
    
    let users: Collection<TestUser> = db.collection("users");
    
    let old_hash = "old_password_hash";
    let new_hash = "new_password_hash";
    
    let user = TestUser {
        id: None,
        username: "test_pwduser".to_string(),
        email: "test_pwduser@example.com".to_string(),
        password_hash: old_hash.to_string(),
        name: "Password User".to_string(),
        role: "BUYER".to_string(),
        roles: vec!["BUYER".to_string()],
        perm_version: 1,
        status: "active".to_string(),
        email_verified: true,
        last_login_at: None,
        created_at: DateTime::now(),
        updated_at: DateTime::now(),
    };
    
    users.insert_one(&user).await.expect("Failed to insert user");
    
    users.update_one(
        doc! { "email": "test_pwduser@example.com" },
        doc! {
            "$set": {
                "password_hash": new_hash,
                "updated_at": DateTime::now()
            }
        },
    ).await.expect("Failed to update password");
    
    let updated = users.find_one(doc! { "email": "test_pwduser@example.com" })
        .await.expect("Query failed").expect("User not found");
    
    assert_eq!(updated.password_hash, new_hash);
    assert_ne!(updated.password_hash, old_hash);
    
    cleanup_test_data(&db).await;
}
