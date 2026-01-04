//! Wallet V3 Repository
//!
//! MongoDB database operations for wallet system

use bson::{doc, oid::ObjectId, DateTime as BsonDateTime, Document};
use mongodb::{Collection, Database, Client, options::{FindOneOptions, FindOptions}, ClientSession};
use futures::stream::TryStreamExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use chrono::{TimeZone, Datelike, Timelike};

use crate::core::error::DbError;
use crate::database::MongoDB;
use super::domain::*;

/// Wallet repository - handles all database operations
#[derive(Clone)]
pub struct WalletRepository {
    client: Client,
    wallets: Collection<Wallet>,
    transactions: Collection<Transaction>,
    withdrawal_requests: Collection<WithdrawalRequest>,
    deposit_requests: Collection<DepositRequest>,
    escrow_holds: Collection<EscrowHold>,
    monthly_snapshots: Collection<MonthlySnapshot>,
    admin_operation_logs: Collection<AdminOperationLog>,
    shop_commission_configs: Collection<ShopCommissionConfig>,
    usdt_deposits: Collection<UsdtDeposit>,
}

impl WalletRepository {
    pub fn new(db: Arc<MongoDB>) -> Self {
        let database = db.database();
        Self {
            client: database.client().clone(),
            wallets: database.collection("wallets"),
            transactions: database.collection("wallet_transactions"),
            withdrawal_requests: database.collection("withdrawal_requests"),
            deposit_requests: database.collection("deposit_requests"),
            escrow_holds: database.collection("escrow_holds"),
            monthly_snapshots: database.collection("monthly_snapshots"),
            admin_operation_logs: database.collection("admin_operation_logs"),
            shop_commission_configs: database.collection("shop_commission_configs"),
            usdt_deposits: database.collection("usdt_deposits"),
        }
    }

    // ========================================================================
    // WALLET OPERATIONS
    // ========================================================================

    /// Create new wallet
    pub async fn create_wallet(&self, wallet: Wallet) -> Result<Wallet, DbError> {
        self.wallets.insert_one(&wallet).await?;
        Ok(wallet)
    }

    /// Find wallet by wallet_id
    pub async fn find_wallet_by_id(&self, wallet_id: &str) -> Result<Option<Wallet>, DbError> {
        self.wallets
            .find_one(doc! { "wallet_id": wallet_id })
            .await
            .map_err(DbError::from)
    }

    /// Find wallet by user_id
    pub async fn find_wallet_by_user_id(&self, user_id: &str) -> Result<Option<Wallet>, DbError> {
        self.wallets
            .find_one(doc! { "user_id": user_id })
            .await
            .map_err(DbError::from)
    }

    /// Update wallet
    pub async fn update_wallet(&self, wallet: &Wallet) -> Result<(), DbError> {
        let mut update_wallet = wallet.clone();
        update_wallet.updated_at = BsonDateTime::now();

        self.wallets
            .replace_one(
                doc! { "wallet_id": &wallet.wallet_id },
                &update_wallet,
            )
            .await?;
        Ok(())
    }

    /// Find wallet by user_id with session (for transactions)
    pub async fn find_wallet_by_user_id_with_session(
        &self,
        user_id: &str,
        session: &mut ClientSession,
    ) -> Result<Option<Wallet>, DbError> {
        self.wallets
            .find_one(doc! { "user_id": user_id })
            .session(&mut *session)
            .await
            .map_err(DbError::from)
    }

    /// Update wallet with session (for atomic operations)
    pub async fn update_wallet_with_session(
        &self,
        wallet: &Wallet,
        session: &mut ClientSession,
    ) -> Result<(), DbError> {
        let mut update_wallet = wallet.clone();
        update_wallet.updated_at = BsonDateTime::now();

        self.wallets
            .replace_one(doc! { "wallet_id": &wallet.wallet_id }, &update_wallet)
            .session(&mut *session)
            .await?;
        Ok(())
    }

    /// Get platform wallet
    pub async fn get_platform_wallet(&self) -> Result<Wallet, DbError> {
        self.wallets
            .find_one(doc! { "user_id": "PLATFORM" })
            .await?
            .ok_or_else(|| DbError::NotFound("Platform wallet not found".to_string()))
    }

    /// Get platform wallet with session
    pub async fn get_platform_wallet_with_session(
        &self,
        session: &mut ClientSession,
    ) -> Result<Wallet, DbError> {
        self.wallets
            .find_one(doc! { "user_id": "PLATFORM" })
            .session(&mut *session)
            .await?
            .ok_or_else(|| DbError::NotFound("Platform wallet not found".to_string()))
    }

    /// Alias for get_platform_wallet - for compatibility
    pub async fn find_platform_wallet(&self) -> Result<Wallet, DbError> {
        self.get_platform_wallet().await
    }

    /// Start a new MongoDB session for transactions
    pub async fn start_session(&self) -> Result<ClientSession, DbError> {
        self.client
            .start_session()
            .await
            .map_err(|e| DbError::ConnectionError(format!("Failed to start session: {}", e)))
    }

    // ========================================================================
    // TRANSACTION OPERATIONS
    // ========================================================================

    /// Create transaction
    pub async fn create_transaction(&self, tx: Transaction) -> Result<Transaction, DbError> {
        self.transactions.insert_one(&tx).await?;
        Ok(tx)
    }

    /// Create transaction with session
    pub async fn create_transaction_with_session(
        &self,
        tx: Transaction,
        session: &mut ClientSession,
    ) -> Result<Transaction, DbError> {
        self.transactions
            .insert_one(&tx)
            .session(&mut *session)
            .await?;
        Ok(tx)
    }

    /// Find transaction by ID
    pub async fn find_transaction_by_id(&self, tx_id: &str) -> Result<Option<Transaction>, DbError> {
        let tx = self
            .transactions
            .find_one(doc! { "tx_id": tx_id })
            .await?;
        Ok(tx)
    }

    /// Update transaction
    pub async fn update_transaction(&self, tx: &Transaction) -> Result<(), DbError> {
        self.transactions
            .update_one(doc! { "tx_id": &tx.tx_id }, doc! { "$set": mongodb::bson::to_document(tx)? })
            .await?;
        Ok(())
    }

    /// Find transactions by user_id
    pub async fn find_transactions_by_user(
        &self,
        user_id: &str,
        skip: u64,
        limit: u64,
    ) -> Result<Vec<Transaction>, DbError> {
        let cursor = self.transactions
            .find(doc! { "user_id": user_id })
            .sort(doc! { "created_at": -1 })
            .skip(skip as u64)
            .limit(limit as i64)
            .await?;
        let transactions: Vec<Transaction> = cursor.try_collect().await?;
        Ok(transactions)
    }

    /// Count transactions by user_id
    pub async fn count_transactions_by_user(&self, user_id: &str) -> Result<u64, DbError> {
        let count = self.transactions
            .count_documents(doc! { "user_id": user_id })
            .await?;
        Ok(count as u64)
    }

    /// Find all transactions (with pagination)
    pub async fn find_all_transactions(
        &self,
        skip: u64,
        limit: u64,
    ) -> Result<Vec<Transaction>, DbError> {
        let cursor = self.transactions
            .find(doc! {})
            .sort(doc! { "created_at": -1 })
            .skip(skip as u64)
            .limit(limit as i64)
            .await?;
        let transactions: Vec<Transaction> = cursor.try_collect().await?;
        Ok(transactions)
    }

    /// Find transactions by wallet_id
    pub async fn find_transactions_by_wallet(
        &self,
        wallet_id: &str,
        start_date: Option<BsonDateTime>,
        end_date: Option<BsonDateTime>,
        limit: i64,
        skip: i64,
    ) -> Result<Vec<Transaction>, DbError> {
        let mut filter = doc! { "wallet_id": wallet_id };

        if let (Some(start), Some(end)) = (start_date, end_date) {
            filter.insert("created_at", doc! {
                "$gte": start,
                "$lte": end,
            });
        }

        let cursor = self.transactions
            .find(filter)
            .sort(doc! { "created_at": -1 })
            .limit(limit)
            .skip(skip as u64)
            .await?;
        let transactions: Vec<Transaction> = cursor.try_collect().await?;
        Ok(transactions)
    }

    /// Count transactions by wallet
    pub async fn count_transactions_by_wallet(&self, wallet_id: &str) -> Result<i64, DbError> {
        let count = self
            .transactions
            .count_documents(doc! { "wallet_id": wallet_id })
            .await? as i64;
        Ok(count)
    }

    /// Get transactions for current month (for validation)
    pub async fn get_transactions_current_month(
        &self,
        wallet_id: &str,
        month_start: BsonDateTime,
    ) -> Result<Vec<Transaction>, DbError> {
        let filter = doc! {
            "wallet_id": wallet_id,
            "created_at": { "$gte": month_start },
            "status": "COMPLETED"
        };

        let cursor = self.transactions.find(filter).await?;
        let transactions: Vec<Transaction> = cursor.try_collect().await?;
        Ok(transactions)
    }

    // ========================================================================
    // WITHDRAWAL OPERATIONS
    // ========================================================================

    /// Create withdrawal request
    pub async fn create_withdrawal_request(
        &self,
        req: WithdrawalRequest,
    ) -> Result<WithdrawalRequest, DbError> {
        self.withdrawal_requests.insert_one(&req).await?;
        Ok(req)
    }

    /// Find withdrawal by request_id
    pub async fn find_withdrawal_by_id(
        &self,
        request_id: &str,
    ) -> Result<Option<WithdrawalRequest>, DbError> {
        self.withdrawal_requests
            .find_one(doc! { "request_id": request_id })
            .await
            .map_err(DbError::from)
    }

    /// Update withdrawal request
    pub async fn update_withdrawal_request(&self, req: &WithdrawalRequest) -> Result<(), DbError> {
        let mut update_req = req.clone();
        update_req.updated_at = BsonDateTime::now();

        self.withdrawal_requests
            .replace_one(doc! { "request_id": &req.request_id }, &update_req)
            .await?;
        Ok(())
    }

    /// Find pending withdrawals for admin review
    pub async fn find_pending_withdrawals_for_review(
        &self,
        limit: i64,
    ) -> Result<Vec<WithdrawalRequest>, DbError> {
        let filter = doc! { "status": "AWAITING_APPROVAL" };

        let cursor = self.withdrawal_requests
            .find(filter)
            .sort(doc! { "created_at": -1 })
            .limit(limit)
            .await?;
        let requests: Vec<WithdrawalRequest> = cursor.try_collect().await?;
        Ok(requests)
    }

    /// Count today's withdrawals for a wallet
    pub async fn count_today_withdrawals(&self, wallet_id: &str) -> Result<i64, DbError> {
        let today_start = chrono::Utc::now()
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();
        let today_start_bson = BsonDateTime::from_millis(today_start.timestamp_millis());

        let count = self
            .withdrawal_requests
            .count_documents(
                doc! {
                    "wallet_id": wallet_id,
                    "created_at": { "$gte": today_start_bson },
                    "status": { "$in": ["COMPLETED", "PROCESSING", "APPROVED"] }
                },
            )
            .await? as i64;
        Ok(count)
    }

    // ========================================================================
    // DEPOSIT OPERATIONS
    // ========================================================================

    /// Create deposit request
    pub async fn create_deposit_request(
        &self,
        req: DepositRequest,
    ) -> Result<DepositRequest, DbError> {
        self.deposit_requests.insert_one(&req).await?;
        Ok(req)
    }

    /// Find deposit by deposit_id
    pub async fn find_deposit_by_id(
        &self,
        deposit_id: &str,
    ) -> Result<Option<DepositRequest>, DbError> {
        self.deposit_requests
            .find_one(doc! { "deposit_id": deposit_id })
            .await
            .map_err(DbError::from)
    }

    /// Find deposit by gateway reference
    pub async fn find_deposit_by_gateway_ref(
        &self,
        gateway_ref: &str,
    ) -> Result<Option<DepositRequest>, DbError> {
        self.deposit_requests
            .find_one(doc! { "payment_gateway_ref": gateway_ref })
            .await
            .map_err(DbError::from)
    }

    /// Update deposit request
    pub async fn update_deposit_request(&self, req: &DepositRequest) -> Result<(), DbError> {
        let mut update_req = req.clone();
        update_req.updated_at = BsonDateTime::now();

        self.deposit_requests
            .replace_one(doc! { "deposit_id": &req.deposit_id }, &update_req)
            .await?;
        Ok(())
    }

    // ========================================================================
    // ESCROW OPERATIONS
    // ========================================================================

    /// Create escrow hold
    pub async fn create_escrow_hold(&self, escrow: EscrowHold) -> Result<EscrowHold, DbError> {
        self.escrow_holds.insert_one(&escrow).await?;
        Ok(escrow)
    }

    /// Create escrow hold with session
    pub async fn create_escrow_hold_with_session(
        &self,
        escrow: EscrowHold,
        session: &mut ClientSession,
    ) -> Result<EscrowHold, DbError> {
        self.escrow_holds
            .insert_one(&escrow)
            .session(&mut *session)
            .await?;
        Ok(escrow)
    }

    /// Find escrow by order_id
    pub async fn find_escrow_by_order_id(
        &self,
        order_id: &str,
    ) -> Result<Option<EscrowHold>, DbError> {
        self.escrow_holds
            .find_one(doc! { "order_id": order_id })
            .await
            .map_err(DbError::from)
    }

    /// Find escrow by escrow_id
    pub async fn find_escrow_by_id(
        &self,
        escrow_id: &str,
    ) -> Result<Option<EscrowHold>, DbError> {
        self.escrow_holds
            .find_one(doc! { "escrow_id": escrow_id })
            .await
            .map_err(DbError::from)
    }

    /// Find escrow by escrow_id with session
    pub async fn find_escrow_by_id_with_session(
        &self,
        escrow_id: &str,
        session: &mut ClientSession,
    ) -> Result<Option<EscrowHold>, DbError> {
        self.escrow_holds
            .find_one(doc! { "escrow_id": escrow_id })
            .session(&mut *session)
            .await
            .map_err(DbError::from)
    }

    /// Find escrows ready for release
    pub async fn find_escrows_ready_for_release(&self) -> Result<Vec<EscrowHold>, DbError> {
        let now = BsonDateTime::now();
        let filter = doc! {
            "status": "HOLDING",
            "release_at": { "$lte": now }
        };

        let cursor = self.escrow_holds.find(filter).await?;
        let escrows: Vec<EscrowHold> = cursor.try_collect().await?;
        Ok(escrows)
    }

    /// Update escrow hold
    pub async fn update_escrow_hold(&self, escrow: &EscrowHold) -> Result<(), DbError> {
        let mut update_escrow = escrow.clone();
        update_escrow.updated_at = BsonDateTime::now();

        self.escrow_holds
            .replace_one(
                doc! { "escrow_id": &escrow.escrow_id },
                &update_escrow,
            )
            .await?;
        Ok(())
    }

    /// Alias for update_escrow_hold - for compatibility
    pub async fn update_escrow(&self, escrow: &EscrowHold) -> Result<(), DbError> {
        self.update_escrow_hold(escrow).await
    }

    /// Update escrow hold with session
    pub async fn update_escrow_hold_with_session(
        &self,
        escrow: &EscrowHold,
        session: &mut ClientSession,
    ) -> Result<(), DbError> {
        let mut update_escrow = escrow.clone();
        update_escrow.updated_at = BsonDateTime::now();

        self.escrow_holds
            .replace_one(doc! { "escrow_id": &escrow.escrow_id }, &update_escrow)
            .session(&mut *session)
            .await?;
        Ok(())
    }

    /// Sum active escrows
    pub async fn sum_active_escrows(&self) -> Result<i64, DbError> {
        let pipeline = vec![
            doc! { "$match": { "status": "HOLDING" } },
            doc! { "$group": { "_id": null, "total": { "$sum": "$amount" } } },
        ];

        let mut cursor = self.escrow_holds.aggregate(pipeline).await?;
        if let Some(result) = cursor.try_next().await? {
            Ok(result.get_i64("total").unwrap_or(0))
        } else {
            Ok(0)
        }
    }

    // ========================================================================
    // SNAPSHOT OPERATIONS
    // ========================================================================

    /// Create monthly snapshot
    pub async fn create_monthly_snapshot(
        &self,
        snapshot: MonthlySnapshot,
    ) -> Result<MonthlySnapshot, DbError> {
        self.monthly_snapshots.insert_one(&snapshot).await?;
        Ok(snapshot)
    }

    /// Find snapshot by wallet_id and month
    pub async fn find_snapshot(
        &self,
        wallet_id: &str,
        month: &str,
    ) -> Result<Option<MonthlySnapshot>, DbError> {
        self.monthly_snapshots
            .find_one(
                doc! {
                    "wallet_id": wallet_id,
                    "month": month
                },
            )
            .await
            .map_err(DbError::from)
    }

    /// Find latest verified snapshot for wallet
    pub async fn find_latest_verified_snapshot(
        &self,
        wallet_id: &str,
    ) -> Result<Option<MonthlySnapshot>, DbError> {
        self.monthly_snapshots
            .find_one(
                doc! {
                    "wallet_id": wallet_id,
                    "status": "VERIFIED"
                },
            )
            .sort(doc! { "month": -1 })
            .await
            .map_err(DbError::from)
    }

    // ========================================================================
    // COMMISSION CONFIG OPERATIONS
    // ========================================================================

    /// Get active commission config for shop
    pub async fn get_active_commission_config(
        &self,
        shop_id: &str,
    ) -> Result<Option<ShopCommissionConfig>, DbError> {
        let now = BsonDateTime::now();
        let filter = doc! {
            "shop_id": shop_id,
            "effective_from": { "$lte": now },
            "$or": [
                { "effective_to": { "$exists": false } },
                { "effective_to": { "$gte": now } }
            ]
        };

        self.shop_commission_configs
            .find_one(filter)
            .sort(doc! { "effective_from": -1 })
            .await
            .map_err(DbError::from)
    }

    /// Create commission config
    pub async fn create_commission_config(
        &self,
        config: ShopCommissionConfig,
    ) -> Result<ShopCommissionConfig, DbError> {
        self.shop_commission_configs
            .insert_one(&config)
            .await?;
        Ok(config)
    }

    /// Deactivate old commission configs
    pub async fn deactivate_old_commission_configs(&self, shop_id: &str) -> Result<(), DbError> {
        let now = BsonDateTime::now();
        self.shop_commission_configs
            .update_many(
                doc! {
                    "shop_id": shop_id,
                    "effective_to": { "$exists": false }
                },
                doc! {
                    "$set": { "effective_to": now }
                },
            )
            .await?;
        Ok(())
    }

    // ========================================================================
    // ADMIN LOG OPERATIONS
    // ========================================================================

    /// Create admin operation log
    pub async fn create_admin_log(
        &self,
        log: AdminOperationLog,
    ) -> Result<AdminOperationLog, DbError> {
        self.admin_operation_logs.insert_one(&log).await?;
        Ok(log)
    }

    /// Find admin logs by target
    pub async fn find_admin_logs_by_target(
        &self,
        target_id: &str,
        limit: i64,
    ) -> Result<Vec<AdminOperationLog>, DbError> {
        let cursor = self
            .admin_operation_logs
            .find(doc! { "target_id": target_id })
            .sort(doc! { "created_at": -1 })
            .limit(limit)
            .await?;
        let logs: Vec<AdminOperationLog> = cursor.try_collect().await?;
        Ok(logs)
    }

    /// Get recent admin logs
    pub async fn get_recent_admin_logs(
        &self,
        limit: i64,
    ) -> Result<Vec<AdminOperationLog>, DbError> {
        let cursor = self
            .admin_operation_logs
            .find(doc! {})
            .sort(doc! { "created_at": -1 })
            .limit(limit)
            .await?;
        let logs: Vec<AdminOperationLog> = cursor.try_collect().await?;
        Ok(logs)
    }

    /// Create admin log with session
    pub async fn create_admin_log_with_session(
        &self,
        log: AdminOperationLog,
        session: &mut ClientSession,
    ) -> Result<AdminOperationLog, DbError> {
        self.admin_operation_logs.insert_one(&log).session(&mut *session).await?;
        Ok(log)
    }

    /// Update withdrawal with session
    pub async fn update_withdrawal_with_session(
        &self,
        withdrawal: &WithdrawalRequest,
        session: &mut ClientSession,
    ) -> Result<(), DbError> {
        let mut updated = withdrawal.clone();
        updated.updated_at = BsonDateTime::now();

        self.withdrawal_requests
            .replace_one(doc! { "request_id": &withdrawal.request_id }, &updated)
            .session(&mut *session)
            .await?;
        Ok(())
    }

    // ========================================================================
    // USDT DEPOSIT REPOSITORY METHODS
    // ========================================================================

    /// Create USDT deposit
    pub async fn create_usdt_deposit(&self, deposit: &UsdtDeposit) -> Result<UsdtDeposit, DbError> {
        self.usdt_deposits.insert_one(deposit).await?;
        Ok(deposit.clone())
    }

    /// Find USDT deposit by ID
    pub async fn find_usdt_deposit_by_id(&self, deposit_id: &str) -> Result<Option<UsdtDeposit>, DbError> {
        self.usdt_deposits
            .find_one(doc! { "deposit_id": deposit_id })
            .await
            .map_err(Into::into)
    }

    /// Find USDT deposit by transaction hash (idempotent check)
    pub async fn find_usdt_deposit_by_tx_hash(
        &self,
        transaction_hash: &str,
    ) -> Result<Option<UsdtDeposit>, DbError> {
        self.usdt_deposits
            .find_one(doc! { "transaction_hash": transaction_hash })
            .await
            .map_err(Into::into)
    }

    /// Update USDT deposit
    pub async fn update_usdt_deposit(&self, deposit: &UsdtDeposit) -> Result<UsdtDeposit, DbError> {
        let updated_deposit = UsdtDeposit {
            id: deposit.id.clone(),
            deposit_id: deposit.deposit_id.clone(),
            wallet_id: deposit.wallet_id.clone(),
            user_id: deposit.user_id.clone(),
            usdt_amount: deposit.usdt_amount,
            network: deposit.network.clone(),
            sender_address: deposit.sender_address.clone(),
            transaction_hash: deposit.transaction_hash.clone(),
            block_number: deposit.block_number,
            vnd_amount: deposit.vnd_amount,
            trust_amount: deposit.trust_amount,
            exchange_rate: deposit.exchange_rate,
            status: deposit.status.clone(),
            confirmations: deposit.confirmations,
            required_confirmations: deposit.required_confirmations,
            credited_at: deposit.credited_at,
            failed_reason: deposit.failed_reason.clone(),
            memo: deposit.memo.clone(),
            transaction_id: deposit.transaction_id.clone(),
            updated_at: BsonDateTime::now(),
            created_at: deposit.created_at,
        };

        self.usdt_deposits
            .update_one(
                doc! { "deposit_id": &deposit.deposit_id },
                doc! { "$set": bson::to_bson(&updated_deposit).map_err(|e| DbError::SerializationError(e.to_string()))? }
            )
            .await?;

        Ok(updated_deposit)
    }

    /// Update USDT deposit confirmations
    pub async fn update_usdt_deposit_confirmations(
        &self,
        deposit_id: &str,
        confirmations: i32,
        status: UsdtDepositStatus,
    ) -> Result<(), DbError> {
        use serde_json;
        let status_str = json!(status).as_str().unwrap_or("Pending").to_string();
        self.usdt_deposits
            .update_one(
                doc! { "deposit_id": deposit_id },
                doc! {
                    "$set": {
                        "confirmations": confirmations,
                        "status": status_str,
                        "updated_at": BsonDateTime::now()
                    }
                },
            )
            .await?;
        Ok(())
    }

    /// Get USDT deposits by user
    pub async fn get_usdt_deposits_by_user(
        &self,
        user_id: &str,
        limit: i64,
    ) -> Result<Vec<UsdtDeposit>, DbError> {
        let cursor = self
            .usdt_deposits
            .find(doc! { "user_id": user_id })
            .sort(doc! { "created_at": -1 })
            .limit(limit)
            .await?;
        cursor.try_collect().await.map_err(Into::into)
    }

    /// Get USDT deposits by status (for monitoring)
    pub async fn get_usdt_deposits_by_status(
        &self,
        status: UsdtDepositStatus,
        limit: i64,
    ) -> Result<Vec<UsdtDeposit>, DbError> {
        use serde_json;
        let status_str = json!(status).as_str().unwrap_or("Pending").to_string();
        let cursor = self
            .usdt_deposits
            .find(doc! { "status": status_str })
            .sort(doc! { "created_at": 1 })
            .limit(limit)
            .await?;
        cursor.try_collect().await.map_err(Into::into)
    }

    /// Get pending USDT deposits older than specified blocks
    pub async fn get_old_pending_deposits(
        &self,
        max_block_age: i64,
    ) -> Result<Vec<UsdtDeposit>, DbError> {
        // Get latest block number would be passed from service
        // For now, return all pending deposits
        self.get_usdt_deposits_by_status(UsdtDepositStatus::Pending, 100)
            .await
    }

    /// Get all USDT deposits (admin)
    pub async fn get_all_usdt_deposits(
        &self,
        page: i64,
        per_page: i64,
    ) -> Result<(Vec<UsdtDeposit>, i64), DbError> {
        let skip = (page - 1) * per_page;

        let total = self
            .usdt_deposits
            .count_documents(doc! {})
            .await
            .map_err::<mongodb::error::Error, _>(Into::into)?;

        let cursor = self
            .usdt_deposits
            .find(doc! {})
            .sort(doc! { "created_at": -1 })
            .skip(skip as u64)
            .limit(per_page)
            .await?;

        let deposits: Vec<UsdtDeposit> = cursor.try_collect().await.map_err::<mongodb::error::Error, _>(Into::into)?;

        Ok((deposits, total as i64))
    }

    /// Get USDT deposits summary
    pub async fn get_usdt_deposits_summary(&self) -> Result<UsdtDepositsSummary, DbError> {
        let pipeline = vec![
            doc! {
                "$group": {
                    "_id": "$status",
                    "count": doc! { "$sum": 1 },
                    "total_usdt": doc! { "$sum": "$usdt_amount" },
                    "total_vnd": doc! { "$sum": "$vnd_amount" }
                }
            },
        ];

        let mut cursor = self.usdt_deposits.aggregate(pipeline).await?;

        let mut summary = UsdtDepositsSummary::default();

        use mongodb::bson::{Document, Bson};
        while let Some(result) = cursor.try_next().await.map_err(|e| DbError::MongoError(e.to_string()))? {
            let status_str = result.get_str("_id").unwrap_or("Unknown");
            let count = result.get_i64("count").unwrap_or(0);
            let total_usdt = result.get_f64("total_usdt").unwrap_or(0.0);
            let total_vnd = result.get_i64("total_vnd").unwrap_or(0);

            match status_str {
                "Credited" => {
                    summary.credited_count = count;
                    summary.credited_usdt = total_usdt;
                    summary.credited_vnd = total_vnd;
                }
                "Pending" | "Confirming" => {
                    summary.pending_count += count;
                    summary.pending_usdt += total_usdt;
                    summary.pending_vnd += total_vnd;
                }
                "Failed" | "Ignored" => {
                    summary.failed_count += count;
                    summary.failed_usdt += total_usdt;
                    summary.failed_vnd += total_vnd;
                }
                _ => {}
            }
        }

        Ok(summary)
    }

    /// Update USDT deposit with session
    pub async fn update_usdt_deposit_with_session(
        &self,
        deposit: &UsdtDeposit,
        session: &mut ClientSession,
    ) -> Result<(), DbError> {
        let mut updated = deposit.clone();
        updated.updated_at = BsonDateTime::now();

        self.usdt_deposits
            .replace_one(doc! { "deposit_id": &deposit.deposit_id }, &updated)
            .session(&mut *session)
            .await?;
        Ok(())
    }

    // ========================================================================
    // DISPUTE CASE OPERATIONS V2 - Enhanced Dispute System
    // ========================================================================

    /// Collection for dispute cases
    pub async fn get_dispute_cases_collection(&self) -> Collection<DisputeCase> {
        self.client
            .database("mmo")
            .collection("dispute_cases")
    }

    /// Create dispute case
    pub async fn create_dispute_case(&self, dispute: DisputeCase) -> Result<DisputeCase, DbError> {
        let collection = self.get_dispute_cases_collection().await;
        collection.insert_one(&dispute).await?;
        Ok(dispute)
    }

    /// Create dispute case with session
    pub async fn create_dispute_case_with_session(
        &self,
        dispute: DisputeCase,
        session: &mut ClientSession,
    ) -> Result<DisputeCase, DbError> {
        let collection = self.get_dispute_cases_collection().await;
        collection
            .insert_one(&dispute)
            .session(&mut *session)
            .await?;
        Ok(dispute)
    }

    /// Find dispute by dispute_id
    pub async fn find_dispute_by_id(&self, dispute_id: &str) -> Result<Option<DisputeCase>, DbError> {
        let collection = self.get_dispute_cases_collection().await;
        collection
            .find_one(doc! { "dispute_id": dispute_id })
            .await
            .map_err(DbError::from)
    }

    /// Find dispute by escrow_id
    pub async fn find_dispute_by_escrow_id(&self, escrow_id: &str) -> Result<Option<DisputeCase>, DbError> {
        let collection = self.get_dispute_cases_collection().await;
        collection
            .find_one(doc! { "escrow_id": escrow_id })
            .await
            .map_err(DbError::from)
    }

    /// Find dispute by order_id
    pub async fn find_dispute_by_order_id(&self, order_id: &str) -> Result<Option<DisputeCase>, DbError> {
        let collection = self.get_dispute_cases_collection().await;
        collection
            .find_one(doc! { "order_id": order_id })
            .await
            .map_err(DbError::from)
    }

    /// Update dispute case
    pub async fn update_dispute_case(&self, dispute: &DisputeCase) -> Result<(), DbError> {
        let collection = self.get_dispute_cases_collection().await;
        let mut update = dispute.clone();
        update.updated_at = BsonDateTime::now();
        collection
            .replace_one(doc! { "dispute_id": &dispute.dispute_id }, &update)
            .await?;
        Ok(())
    }

    /// Update dispute case with session
    pub async fn update_dispute_case_with_session(
        &self,
        dispute: &DisputeCase,
        session: &mut ClientSession,
    ) -> Result<(), DbError> {
        let collection = self.get_dispute_cases_collection().await;
        let mut update = dispute.clone();
        update.updated_at = BsonDateTime::now();
        collection
            .replace_one(doc! { "dispute_id": &dispute.dispute_id }, &update)
            .session(&mut *session)
            .await?;
        Ok(())
    }

    /// Find disputes needing seller response (passed deadline)
    pub async fn find_disputes_past_seller_deadline(&self) -> Result<Vec<DisputeCase>, DbError> {
        let collection = self.get_dispute_cases_collection().await;
        let now = BsonDateTime::now();
        let filter = doc! {
            "status": "PENDING",
            "seller_response": { "$exists": false },
            "seller_deadline": { "$lte": now }
        };
        let cursor = collection.find(filter).await?;
        cursor.try_collect().await.map_err(Into::into)
    }

    /// Find disputes needing buyer response (passed deadline)
    pub async fn find_disputes_past_buyer_deadline(&self) -> Result<Vec<DisputeCase>, DbError> {
        let collection = self.get_dispute_cases_collection().await;
        let now = BsonDateTime::now();
        let filter = doc! {
            "status": { "$in": ["SELLER_RESPONDED", "BUYER_RESPONDED"] },
            "buyer_deadline": { "$exists": true, "$lte": now }
        };
        let cursor = collection.find(filter).await?;
        cursor.try_collect().await.map_err(Into::into)
    }

    /// Find disputes by status
    pub async fn find_disputes_by_status(
        &self,
        status: DisputeStatus,
        limit: i64,
    ) -> Result<Vec<DisputeCase>, DbError> {
        let collection = self.get_dispute_cases_collection().await;
        use serde_json;
        let status_str = json!(status).as_str().unwrap_or("PENDING").to_string();
        let cursor = collection
            .find(doc! { "status": status_str })
            .sort(doc! { "created_at": -1 })
            .limit(limit)
            .await?;
        cursor.try_collect().await.map_err(Into::into)
    }

    /// Find disputes by user (buyer or seller)
    pub async fn find_disputes_by_user(
        &self,
        user_id: &str,
        page: i64,
        per_page: i64,
    ) -> Result<Vec<DisputeCase>, DbError> {
        let collection = self.get_dispute_cases_collection().await;
        let skip = (page - 1) * per_page;
        let cursor = collection
            .find(doc! {
                "$or": [
                    { "buyer_id": user_id },
                    { "seller_id": user_id }
                ]
            })
            .sort(doc! { "created_at": -1 })
            .skip(skip as u64)
            .limit(per_page)
            .await?;
        cursor.try_collect().await.map_err(Into::into)
    }

    /// Count disputes by user
    pub async fn count_disputes_by_user(&self, user_id: &str) -> Result<i64, DbError> {
        let collection = self.get_dispute_cases_collection().await;
        let count = collection
            .count_documents(doc! {
                "$or": [
                    { "buyer_id": user_id },
                    { "seller_id": user_id }
                ]
            })
            .await? as i64;
        Ok(count)
    }

    /// Find escalated disputes (for admin review)
    pub async fn find_escalated_disputes(&self, limit: i64) -> Result<Vec<DisputeCase>, DbError> {
        let collection = self.get_dispute_cases_collection().await;
        let cursor = collection
            .find(doc! {
                "status": { "$in": ["ESCALATED", "ADMIN_REVIEW"] }
            })
            .sort(doc! { "escalated_at": 1 })
            .limit(limit)
            .await?;
        cursor.try_collect().await.map_err(Into::into)
    }

    /// Find disputes by order_id (list - for admin)
    pub async fn find_disputes_list(
        &self,
        filter: Option<Document>,
        page: i64,
        per_page: i64,
    ) -> Result<(Vec<DisputeCase>, i64), DbError> {
        let collection = self.get_dispute_cases_collection().await;
        let skip = (page - 1) * per_page;

        let query_filter = filter.unwrap_or_else(|| doc! {});
        let total = collection
            .count_documents(query_filter.clone())
            .await
            .map_err::<mongodb::error::Error, _>(Into::into)? as i64;

        let cursor = collection
            .find(query_filter)
            .sort(doc! { "created_at": -1 })
            .skip(skip as u64)
            .limit(per_page)
            .await?;

        let disputes: Vec<DisputeCase> = cursor.try_collect().await.map_err(|e| DbError::MongoError(e.to_string()))?;

        Ok((disputes, total))
    }

    // ========================================================================
    // RECONCILIATION & DASHBOARD METHODS
    // ========================================================================

    /// Get all wallets for reconciliation (paginated)
    pub async fn find_all_wallets_for_reconciliation(
        &self,
        limit: u64,
        skip: u64,
    ) -> Result<Vec<Wallet>, DbError> {
        // Use skip/limit in the query chain directly
        let cursor = self.wallets
            .find(doc! {})
            .skip(skip as u64)
            .limit(limit as i64)
            .await
            .map_err(DbError::from)?;

        let wallets = cursor.try_collect().await?;
        Ok(wallets)
    }

    /// Count active escrows (status = Holding)
    pub async fn count_active_escrows(&self) -> Result<u64, DbError> {
        let count = self
            .escrow_holds
            .count_documents(doc! { "status": "HOLDING" })
            .await?;
        Ok(count)
    }

    /// Get transaction stats for current month (for dashboard stats)
    pub async fn get_monthly_transaction_stats(
        &self,
    ) -> Result<(u64, i64), DbError> {
        let now = chrono::Utc::now();
        // Use first day of current month at 00:00:00
        let start_of_month = now.date_naive()
            .and_hms_opt(0, 0, 0)
            .map(|dt| chrono::Utc.from_utc_datetime(&dt))
            .unwrap_or(now);

        let pipeline = vec![
            doc! {
                "$match": {
                    "created_at": { "$gte": start_of_month }
                }
            },
            doc! {
                "$group": {
                    "_id": null,
                    "count": doc! { "$sum": 1 },
                    "volume": doc! { "$sum": "$amount" }
                }
            },
        ];

        let mut cursor = self.transactions.aggregate(pipeline).await?;
        let result = cursor.try_next().await?;

        match result {
            Some(doc) => {
                let count = doc.get_i64("count").unwrap_or(0) as u64;
                let volume = doc.get_i64("volume").unwrap_or(0);
                Ok((count, volume))
            }
            None => Ok((0, 0)),
        }
    }
}

/// USDT deposits summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsdtDepositsSummary {
    pub pending_count: i64,
    pub pending_usdt: f64,
    pub pending_vnd: i64,

    pub credited_count: i64,
    pub credited_usdt: f64,
    pub credited_vnd: i64,

    pub failed_count: i64,
    pub failed_usdt: f64,
    pub failed_vnd: i64,
}

