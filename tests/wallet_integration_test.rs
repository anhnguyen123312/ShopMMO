//! Integration tests for the Wallet system
//!
//! Tests cover:
//! - Wallet CRUD operations
//! - Transaction creation and queries
//! - Deposit/withdrawal flows
//! - Escrow lifecycle
//!
//! Run with: cargo test --test wallet_integration_test -- --ignored

use bson::{doc, oid::ObjectId, DateTime};
use mongodb::{Client, Collection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestWallet {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    wallet_id: String,
    user_id: ObjectId,
    wallet_type: String,
    balance: i64,
    pending_in: i64,
    pending_out: i64,
    locked: i64,
    total_deposited: i64,
    total_withdrawn: i64,
    is_active: bool,
    created_at: DateTime,
    updated_at: DateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestTransaction {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    tx_id: String,
    wallet_id: String,
    tx_type: String,
    direction: String,
    amount: i64,
    balance_before: i64,
    balance_after: i64,
    status: String,
    created_at: DateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestEscrow {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    escrow_id: String,
    order_id: String,
    buyer_wallet_id: String,
    seller_wallet_id: String,
    amount: i64,
    platform_fee: i64,
    status: String,
    created_at: DateTime,
    updated_at: DateTime,
}

async fn get_test_db() -> mongodb::Database {
    let mongo_url = std::env::var("MONGODB_URI")
        .or_else(|_| std::env::var("MONGODB_URL"))
        .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    
    let client = Client::with_uri_str(&mongo_url)
        .await
        .expect("Failed to connect to MongoDB");
    
    client.database("mmo_api_wallet_test")
}

async fn cleanup_test_data(db: &mongodb::Database) {
    let _ = db.collection::<TestWallet>("wallets")
        .delete_many(doc! { "wallet_id": { "$regex": "^TEST_" } })
        .await;
    let _ = db.collection::<TestTransaction>("wallet_transactions")
        .delete_many(doc! { "tx_id": { "$regex": "^TEST_" } })
        .await;
    let _ = db.collection::<TestEscrow>("escrow_holds")
        .delete_many(doc! { "escrow_id": { "$regex": "^TEST_" } })
        .await;
}

fn generate_test_id(prefix: &str) -> String {
    format!("TEST_{}_{}", prefix, ObjectId::new().to_hex())
}

#[tokio::test]
#[ignore]
async fn test_create_wallet() {
    let db = get_test_db().await;
    cleanup_test_data(&db).await;
    
    let wallets: Collection<TestWallet> = db.collection("wallets");
    let user_id = ObjectId::new();
    let wallet_id = generate_test_id("WAL");
    
    let wallet = TestWallet {
        id: None,
        wallet_id: wallet_id.clone(),
        user_id,
        wallet_type: "USER".to_string(),
        balance: 0,
        pending_in: 0,
        pending_out: 0,
        locked: 0,
        total_deposited: 0,
        total_withdrawn: 0,
        is_active: true,
        created_at: DateTime::now(),
        updated_at: DateTime::now(),
    };
    
    wallets.insert_one(&wallet).await.expect("Failed to insert wallet");
    
    let found = wallets.find_one(doc! { "wallet_id": &wallet_id })
        .await
        .expect("Query failed")
        .expect("Wallet not found");
    
    assert_eq!(found.wallet_id, wallet_id);
    assert_eq!(found.user_id, user_id);
    assert_eq!(found.wallet_type, "USER");
    assert_eq!(found.balance, 0);
    assert!(found.is_active);
    
    cleanup_test_data(&db).await;
}

#[tokio::test]
#[ignore]
async fn test_wallet_balance_invariant() {
    let db = get_test_db().await;
    cleanup_test_data(&db).await;
    
    let wallets: Collection<TestWallet> = db.collection("wallets");
    let wallet_id = generate_test_id("WAL");
    
    let wallet = TestWallet {
        id: None,
        wallet_id: wallet_id.clone(),
        user_id: ObjectId::new(),
        wallet_type: "USER".to_string(),
        balance: 100_000,
        pending_in: 20_000,
        pending_out: 10_000,
        locked: 5_000,
        total_deposited: 150_000,
        total_withdrawn: 50_000,
        is_active: true,
        created_at: DateTime::now(),
        updated_at: DateTime::now(),
    };
    
    wallets.insert_one(&wallet).await.expect("Failed to insert wallet");
    
    let found = wallets.find_one(doc! { "wallet_id": &wallet_id })
        .await
        .expect("Query failed")
        .expect("Wallet not found");
    
    let computed_balance = found.total_deposited - found.total_withdrawn;
    assert_eq!(computed_balance, 100_000);
    assert_eq!(found.balance, computed_balance);
    
    cleanup_test_data(&db).await;
}

#[tokio::test]
#[ignore]
async fn test_create_transaction() {
    let db = get_test_db().await;
    cleanup_test_data(&db).await;
    
    let transactions: Collection<TestTransaction> = db.collection("wallet_transactions");
    let tx_id = generate_test_id("TXN");
    let wallet_id = generate_test_id("WAL");
    
    let tx = TestTransaction {
        id: None,
        tx_id: tx_id.clone(),
        wallet_id: wallet_id.clone(),
        tx_type: "DEPOSIT".to_string(),
        direction: "IN".to_string(),
        amount: 100_000,
        balance_before: 0,
        balance_after: 100_000,
        status: "COMPLETED".to_string(),
        created_at: DateTime::now(),
    };
    
    transactions.insert_one(&tx).await.expect("Failed to insert transaction");
    
    let found = transactions.find_one(doc! { "tx_id": &tx_id })
        .await
        .expect("Query failed")
        .expect("Transaction not found");
    
    assert_eq!(found.tx_id, tx_id);
    assert_eq!(found.wallet_id, wallet_id);
    assert_eq!(found.tx_type, "DEPOSIT");
    assert_eq!(found.direction, "IN");
    assert_eq!(found.amount, 100_000);
    assert_eq!(found.balance_after - found.balance_before, 100_000);
    
    cleanup_test_data(&db).await;
}

#[tokio::test]
#[ignore]
async fn test_deposit_updates_wallet_balance() {
    let db = get_test_db().await;
    cleanup_test_data(&db).await;
    
    let wallets: Collection<TestWallet> = db.collection("wallets");
    let transactions: Collection<TestTransaction> = db.collection("wallet_transactions");
    let wallet_id = generate_test_id("WAL");
    
    let wallet = TestWallet {
        id: None,
        wallet_id: wallet_id.clone(),
        user_id: ObjectId::new(),
        wallet_type: "USER".to_string(),
        balance: 0,
        pending_in: 0,
        pending_out: 0,
        locked: 0,
        total_deposited: 0,
        total_withdrawn: 0,
        is_active: true,
        created_at: DateTime::now(),
        updated_at: DateTime::now(),
    };
    wallets.insert_one(&wallet).await.expect("Failed to insert wallet");
    
    let deposit_amount: i64 = 500_000;
    
    wallets.update_one(
        doc! { "wallet_id": &wallet_id },
        doc! {
            "$inc": {
                "balance": deposit_amount,
                "total_deposited": deposit_amount
            },
            "$set": { "updated_at": DateTime::now() }
        },
    ).await.expect("Failed to update wallet");
    
    let tx_id = generate_test_id("TXN");
    let tx = TestTransaction {
        id: None,
        tx_id: tx_id.clone(),
        wallet_id: wallet_id.clone(),
        tx_type: "DEPOSIT".to_string(),
        direction: "IN".to_string(),
        amount: deposit_amount,
        balance_before: 0,
        balance_after: deposit_amount,
        status: "COMPLETED".to_string(),
        created_at: DateTime::now(),
    };
    transactions.insert_one(&tx).await.expect("Failed to insert transaction");
    
    let updated_wallet = wallets.find_one(doc! { "wallet_id": &wallet_id })
        .await
        .expect("Query failed")
        .expect("Wallet not found");
    
    assert_eq!(updated_wallet.balance, deposit_amount);
    assert_eq!(updated_wallet.total_deposited, deposit_amount);
    
    cleanup_test_data(&db).await;
}

#[tokio::test]
#[ignore]
async fn test_withdrawal_with_balance_check() {
    let db = get_test_db().await;
    cleanup_test_data(&db).await;
    
    let wallets: Collection<TestWallet> = db.collection("wallets");
    let wallet_id = generate_test_id("WAL");
    
    let wallet = TestWallet {
        id: None,
        wallet_id: wallet_id.clone(),
        user_id: ObjectId::new(),
        wallet_type: "USER".to_string(),
        balance: 1_000_000,
        pending_in: 0,
        pending_out: 0,
        locked: 0,
        total_deposited: 1_000_000,
        total_withdrawn: 0,
        is_active: true,
        created_at: DateTime::now(),
        updated_at: DateTime::now(),
    };
    wallets.insert_one(&wallet).await.expect("Failed to insert wallet");
    
    let withdrawal_amount: i64 = 300_000;
    
    let result = wallets.update_one(
        doc! {
            "wallet_id": &wallet_id,
            "balance": { "$gte": withdrawal_amount }
        },
        doc! {
            "$inc": {
                "balance": -withdrawal_amount,
                "total_withdrawn": withdrawal_amount
            },
            "$set": { "updated_at": DateTime::now() }
        },
    ).await.expect("Failed to update wallet");
    
    assert_eq!(result.modified_count, 1);
    
    let updated_wallet = wallets.find_one(doc! { "wallet_id": &wallet_id })
        .await
        .expect("Query failed")
        .expect("Wallet not found");
    
    assert_eq!(updated_wallet.balance, 700_000);
    assert_eq!(updated_wallet.total_withdrawn, 300_000);
    
    cleanup_test_data(&db).await;
}

#[tokio::test]
#[ignore]
async fn test_withdrawal_fails_insufficient_balance() {
    let db = get_test_db().await;
    cleanup_test_data(&db).await;
    
    let wallets: Collection<TestWallet> = db.collection("wallets");
    let wallet_id = generate_test_id("WAL");
    
    let wallet = TestWallet {
        id: None,
        wallet_id: wallet_id.clone(),
        user_id: ObjectId::new(),
        wallet_type: "USER".to_string(),
        balance: 100_000,
        pending_in: 0,
        pending_out: 0,
        locked: 0,
        total_deposited: 100_000,
        total_withdrawn: 0,
        is_active: true,
        created_at: DateTime::now(),
        updated_at: DateTime::now(),
    };
    wallets.insert_one(&wallet).await.expect("Failed to insert wallet");
    
    let withdrawal_amount: i64 = 500_000;
    
    let result = wallets.update_one(
        doc! {
            "wallet_id": &wallet_id,
            "balance": { "$gte": withdrawal_amount }
        },
        doc! {
            "$inc": {
                "balance": -withdrawal_amount,
                "total_withdrawn": withdrawal_amount
            }
        },
    ).await.expect("Failed to execute update");
    
    assert_eq!(result.modified_count, 0);
    
    let unchanged_wallet = wallets.find_one(doc! { "wallet_id": &wallet_id })
        .await
        .expect("Query failed")
        .expect("Wallet not found");
    
    assert_eq!(unchanged_wallet.balance, 100_000);
    assert_eq!(unchanged_wallet.total_withdrawn, 0);
    
    cleanup_test_data(&db).await;
}

#[tokio::test]
#[ignore]
async fn test_create_escrow_hold() {
    let db = get_test_db().await;
    cleanup_test_data(&db).await;
    
    let escrows: Collection<TestEscrow> = db.collection("escrow_holds");
    let escrow_id = generate_test_id("ESC");
    let order_id = generate_test_id("ORD");
    
    let escrow = TestEscrow {
        id: None,
        escrow_id: escrow_id.clone(),
        order_id: order_id.clone(),
        buyer_wallet_id: generate_test_id("WAL"),
        seller_wallet_id: generate_test_id("WAL"),
        amount: 500_000,
        platform_fee: 25_000,
        status: "HELD".to_string(),
        created_at: DateTime::now(),
        updated_at: DateTime::now(),
    };
    
    escrows.insert_one(&escrow).await.expect("Failed to insert escrow");
    
    let found = escrows.find_one(doc! { "escrow_id": &escrow_id })
        .await
        .expect("Query failed")
        .expect("Escrow not found");
    
    assert_eq!(found.escrow_id, escrow_id);
    assert_eq!(found.order_id, order_id);
    assert_eq!(found.amount, 500_000);
    assert_eq!(found.platform_fee, 25_000);
    assert_eq!(found.status, "HELD");
    
    cleanup_test_data(&db).await;
}

#[tokio::test]
#[ignore]
async fn test_escrow_release_flow() {
    let db = get_test_db().await;
    cleanup_test_data(&db).await;
    
    let wallets: Collection<TestWallet> = db.collection("wallets");
    let escrows: Collection<TestEscrow> = db.collection("escrow_holds");
    
    let buyer_wallet_id = generate_test_id("WAL");
    let seller_wallet_id = generate_test_id("WAL");
    let platform_wallet_id = generate_test_id("WAL");
    
    let buyer_wallet = TestWallet {
        id: None,
        wallet_id: buyer_wallet_id.clone(),
        user_id: ObjectId::new(),
        wallet_type: "USER".to_string(),
        balance: 1_000_000,
        pending_in: 0,
        pending_out: 0,
        locked: 500_000,
        total_deposited: 1_000_000,
        total_withdrawn: 0,
        is_active: true,
        created_at: DateTime::now(),
        updated_at: DateTime::now(),
    };
    
    let seller_wallet = TestWallet {
        id: None,
        wallet_id: seller_wallet_id.clone(),
        user_id: ObjectId::new(),
        wallet_type: "SELLER".to_string(),
        balance: 0,
        pending_in: 475_000,
        pending_out: 0,
        locked: 0,
        total_deposited: 0,
        total_withdrawn: 0,
        is_active: true,
        created_at: DateTime::now(),
        updated_at: DateTime::now(),
    };
    
    let platform_wallet = TestWallet {
        id: None,
        wallet_id: platform_wallet_id.clone(),
        user_id: ObjectId::new(),
        wallet_type: "PLATFORM".to_string(),
        balance: 0,
        pending_in: 0,
        pending_out: 0,
        locked: 0,
        total_deposited: 0,
        total_withdrawn: 0,
        is_active: true,
        created_at: DateTime::now(),
        updated_at: DateTime::now(),
    };
    
    wallets.insert_one(&buyer_wallet).await.expect("Failed to insert buyer wallet");
    wallets.insert_one(&seller_wallet).await.expect("Failed to insert seller wallet");
    wallets.insert_one(&platform_wallet).await.expect("Failed to insert platform wallet");
    
    let escrow_id = generate_test_id("ESC");
    let escrow_amount: i64 = 500_000;
    let platform_fee: i64 = 25_000;
    let seller_amount: i64 = escrow_amount - platform_fee;
    
    let escrow = TestEscrow {
        id: None,
        escrow_id: escrow_id.clone(),
        order_id: generate_test_id("ORD"),
        buyer_wallet_id: buyer_wallet_id.clone(),
        seller_wallet_id: seller_wallet_id.clone(),
        amount: escrow_amount,
        platform_fee,
        status: "HELD".to_string(),
        created_at: DateTime::now(),
        updated_at: DateTime::now(),
    };
    escrows.insert_one(&escrow).await.expect("Failed to insert escrow");
    
    wallets.update_one(
        doc! { "wallet_id": &buyer_wallet_id },
        doc! { "$inc": { "locked": -escrow_amount } },
    ).await.expect("Failed to update buyer");
    
    wallets.update_one(
        doc! { "wallet_id": &seller_wallet_id },
        doc! {
            "$inc": {
                "balance": seller_amount,
                "pending_in": -seller_amount,
                "total_deposited": seller_amount
            }
        },
    ).await.expect("Failed to update seller");
    
    wallets.update_one(
        doc! { "wallet_id": &platform_wallet_id },
        doc! {
            "$inc": {
                "balance": platform_fee,
                "total_deposited": platform_fee
            }
        },
    ).await.expect("Failed to update platform");
    
    escrows.update_one(
        doc! { "escrow_id": &escrow_id },
        doc! {
            "$set": {
                "status": "RELEASED",
                "updated_at": DateTime::now()
            }
        },
    ).await.expect("Failed to update escrow");
    
    let final_buyer = wallets.find_one(doc! { "wallet_id": &buyer_wallet_id })
        .await.expect("Query failed").expect("Buyer not found");
    let final_seller = wallets.find_one(doc! { "wallet_id": &seller_wallet_id })
        .await.expect("Query failed").expect("Seller not found");
    let final_platform = wallets.find_one(doc! { "wallet_id": &platform_wallet_id })
        .await.expect("Query failed").expect("Platform not found");
    let final_escrow = escrows.find_one(doc! { "escrow_id": &escrow_id })
        .await.expect("Query failed").expect("Escrow not found");
    
    assert_eq!(final_buyer.locked, 0);
    assert_eq!(final_seller.balance, seller_amount);
    assert_eq!(final_seller.pending_in, 0);
    assert_eq!(final_platform.balance, platform_fee);
    assert_eq!(final_escrow.status, "RELEASED");
    
    cleanup_test_data(&db).await;
}

#[tokio::test]
#[ignore]
async fn test_escrow_refund_flow() {
    let db = get_test_db().await;
    cleanup_test_data(&db).await;
    
    let wallets: Collection<TestWallet> = db.collection("wallets");
    let escrows: Collection<TestEscrow> = db.collection("escrow_holds");
    
    let buyer_wallet_id = generate_test_id("WAL");
    let seller_wallet_id = generate_test_id("WAL");
    
    let escrow_amount: i64 = 300_000;
    
    let buyer_wallet = TestWallet {
        id: None,
        wallet_id: buyer_wallet_id.clone(),
        user_id: ObjectId::new(),
        wallet_type: "USER".to_string(),
        balance: 500_000,
        pending_in: 0,
        pending_out: 0,
        locked: escrow_amount,
        total_deposited: 800_000,
        total_withdrawn: 0,
        is_active: true,
        created_at: DateTime::now(),
        updated_at: DateTime::now(),
    };
    
    let seller_wallet = TestWallet {
        id: None,
        wallet_id: seller_wallet_id.clone(),
        user_id: ObjectId::new(),
        wallet_type: "SELLER".to_string(),
        balance: 100_000,
        pending_in: 285_000,
        pending_out: 0,
        locked: 0,
        total_deposited: 100_000,
        total_withdrawn: 0,
        is_active: true,
        created_at: DateTime::now(),
        updated_at: DateTime::now(),
    };
    
    wallets.insert_one(&buyer_wallet).await.expect("Failed to insert buyer");
    wallets.insert_one(&seller_wallet).await.expect("Failed to insert seller");
    
    let escrow_id = generate_test_id("ESC");
    let escrow = TestEscrow {
        id: None,
        escrow_id: escrow_id.clone(),
        order_id: generate_test_id("ORD"),
        buyer_wallet_id: buyer_wallet_id.clone(),
        seller_wallet_id: seller_wallet_id.clone(),
        amount: escrow_amount,
        platform_fee: 15_000,
        status: "HELD".to_string(),
        created_at: DateTime::now(),
        updated_at: DateTime::now(),
    };
    escrows.insert_one(&escrow).await.expect("Failed to insert escrow");
    
    wallets.update_one(
        doc! { "wallet_id": &buyer_wallet_id },
        doc! { "$inc": { "locked": -escrow_amount } },
    ).await.expect("Failed to refund buyer");
    
    wallets.update_one(
        doc! { "wallet_id": &seller_wallet_id },
        doc! { "$inc": { "pending_in": -(escrow_amount - 15_000) } },
    ).await.expect("Failed to update seller pending");
    
    escrows.update_one(
        doc! { "escrow_id": &escrow_id },
        doc! { "$set": { "status": "REFUNDED", "updated_at": DateTime::now() } },
    ).await.expect("Failed to update escrow");
    
    let final_buyer = wallets.find_one(doc! { "wallet_id": &buyer_wallet_id })
        .await.expect("Query failed").expect("Buyer not found");
    let final_seller = wallets.find_one(doc! { "wallet_id": &seller_wallet_id })
        .await.expect("Query failed").expect("Seller not found");
    let final_escrow = escrows.find_one(doc! { "escrow_id": &escrow_id })
        .await.expect("Query failed").expect("Escrow not found");
    
    assert_eq!(final_buyer.locked, 0);
    assert_eq!(final_buyer.balance, 500_000);
    assert_eq!(final_seller.pending_in, 0);
    assert_eq!(final_escrow.status, "REFUNDED");
    
    cleanup_test_data(&db).await;
}

#[tokio::test]
#[ignore]
async fn test_transaction_history_query() {
    let db = get_test_db().await;
    cleanup_test_data(&db).await;
    
    let transactions: Collection<TestTransaction> = db.collection("wallet_transactions");
    let wallet_id = generate_test_id("WAL");
    
    let tx_types = vec!["DEPOSIT", "WITHDRAWAL", "ESCROW_HOLD", "ESCROW_RELEASE"];
    for (i, tx_type) in tx_types.iter().enumerate() {
        let tx = TestTransaction {
            id: None,
            tx_id: generate_test_id("TXN"),
            wallet_id: wallet_id.clone(),
            tx_type: tx_type.to_string(),
            direction: if i % 2 == 0 { "IN" } else { "OUT" }.to_string(),
            amount: (i as i64 + 1) * 100_000,
            balance_before: 0,
            balance_after: 100_000,
            status: "COMPLETED".to_string(),
            created_at: DateTime::now(),
        };
        transactions.insert_one(&tx).await.expect("Failed to insert transaction");
    }
    
    let mut cursor = transactions.find(doc! { "wallet_id": &wallet_id })
        .await.expect("Query failed");
    
    let mut count = 0;
    while cursor.advance().await.expect("Cursor error") {
        count += 1;
    }
    
    assert_eq!(count, 4);
    
    let deposits: Vec<TestTransaction> = {
        let mut cursor = transactions.find(
            doc! { "wallet_id": &wallet_id, "tx_type": "DEPOSIT" }
        ).await.expect("Query failed");
        
        let mut results = Vec::new();
        while cursor.advance().await.expect("Cursor error") {
            results.push(cursor.deserialize_current().expect("Deserialize error"));
        }
        results
    };
    
    assert_eq!(deposits.len(), 1);
    assert_eq!(deposits[0].tx_type, "DEPOSIT");
    
    cleanup_test_data(&db).await;
}

#[tokio::test]
#[ignore]
async fn test_multiple_wallet_types() {
    let db = get_test_db().await;
    cleanup_test_data(&db).await;
    
    let wallets: Collection<TestWallet> = db.collection("wallets");
    
    let wallet_types = vec!["USER", "SELLER", "PLATFORM"];
    for wallet_type in &wallet_types {
        let wallet = TestWallet {
            id: None,
            wallet_id: generate_test_id("WAL"),
            user_id: ObjectId::new(),
            wallet_type: wallet_type.to_string(),
            balance: 0,
            pending_in: 0,
            pending_out: 0,
            locked: 0,
            total_deposited: 0,
            total_withdrawn: 0,
            is_active: true,
            created_at: DateTime::now(),
            updated_at: DateTime::now(),
        };
        wallets.insert_one(&wallet).await.expect("Failed to insert wallet");
    }
    
    for wallet_type in &wallet_types {
        let found = wallets.find_one(
            doc! { "wallet_type": wallet_type, "wallet_id": { "$regex": "^TEST_" } }
        ).await.expect("Query failed");
        
        assert!(found.is_some(), "Wallet type {} not found", wallet_type);
    }
    
    cleanup_test_data(&db).await;
}

#[tokio::test]
#[ignore]
async fn test_wallet_deactivation() {
    let db = get_test_db().await;
    cleanup_test_data(&db).await;
    
    let wallets: Collection<TestWallet> = db.collection("wallets");
    let wallet_id = generate_test_id("WAL");
    
    let wallet = TestWallet {
        id: None,
        wallet_id: wallet_id.clone(),
        user_id: ObjectId::new(),
        wallet_type: "USER".to_string(),
        balance: 100_000,
        pending_in: 0,
        pending_out: 0,
        locked: 0,
        total_deposited: 100_000,
        total_withdrawn: 0,
        is_active: true,
        created_at: DateTime::now(),
        updated_at: DateTime::now(),
    };
    wallets.insert_one(&wallet).await.expect("Failed to insert wallet");
    
    wallets.update_one(
        doc! { "wallet_id": &wallet_id },
        doc! { "$set": { "is_active": false, "updated_at": DateTime::now() } },
    ).await.expect("Failed to deactivate wallet");
    
    let active_wallet = wallets.find_one(
        doc! { "wallet_id": &wallet_id, "is_active": true }
    ).await.expect("Query failed");
    assert!(active_wallet.is_none());
    
    let inactive_wallet = wallets.find_one(
        doc! { "wallet_id": &wallet_id, "is_active": false }
    ).await.expect("Query failed");
    assert!(inactive_wallet.is_some());
    assert!(!inactive_wallet.unwrap().is_active);
    
    cleanup_test_data(&db).await;
}

#[tokio::test]
#[ignore]
async fn test_concurrent_balance_check() {
    let db = get_test_db().await;
    cleanup_test_data(&db).await;
    
    let wallets: Collection<TestWallet> = db.collection("wallets");
    let wallet_id = generate_test_id("WAL");
    
    let wallet = TestWallet {
        id: None,
        wallet_id: wallet_id.clone(),
        user_id: ObjectId::new(),
        wallet_type: "USER".to_string(),
        balance: 100_000,
        pending_in: 0,
        pending_out: 0,
        locked: 0,
        total_deposited: 100_000,
        total_withdrawn: 0,
        is_active: true,
        created_at: DateTime::now(),
        updated_at: DateTime::now(),
    };
    wallets.insert_one(&wallet).await.expect("Failed to insert wallet");
    
    let amount: i64 = 60_000;
    
    let result1 = wallets.update_one(
        doc! {
            "wallet_id": &wallet_id,
            "balance": { "$gte": amount }
        },
        doc! {
            "$inc": { "balance": -amount, "total_withdrawn": amount }
        },
    ).await.expect("First update failed");
    
    let result2 = wallets.update_one(
        doc! {
            "wallet_id": &wallet_id,
            "balance": { "$gte": amount }
        },
        doc! {
            "$inc": { "balance": -amount, "total_withdrawn": amount }
        },
    ).await.expect("Second update failed");
    
    assert_eq!(result1.modified_count, 1);
    assert_eq!(result2.modified_count, 0);
    
    let final_wallet = wallets.find_one(doc! { "wallet_id": &wallet_id })
        .await.expect("Query failed").expect("Wallet not found");
    
    assert_eq!(final_wallet.balance, 40_000);
    assert_eq!(final_wallet.total_withdrawn, 60_000);
    
    cleanup_test_data(&db).await;
}
