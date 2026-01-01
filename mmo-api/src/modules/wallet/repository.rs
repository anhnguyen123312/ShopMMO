//! Wallet V3 Repository
//!
//! MongoDB database operations for wallet system

use bson::{doc, oid::ObjectId, DateTime as BsonDateTime};
use mongodb::{Collection, Database, Client, options::{FindOneOptions, FindOptions}, ClientSession};
use futures::stream::TryStreamExt;
use std::sync::Arc;

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
}
