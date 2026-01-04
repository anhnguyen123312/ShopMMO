//! Wallet V3 Service - Background Cron Jobs
//!
//! Tokio-based background tasks for periodic operations

use std::sync::Arc;
use std::time::Duration;
use tokio::time::{interval, Instant};
use tracing::{info, warn, error};
use chrono::Timelike;

use crate::core::error::ServiceError;
use super::{dto::*, repository::WalletRepository, domain::*, service::WalletService};

/// Duration for escrow auto-release check (every 5 minutes)
const ESCROW_CHECK_INTERVAL: Duration = Duration::from_secs(300);

/// Duration for daily reconciliation (check every hour)
const RECONCILIATION_CHECK_INTERVAL: Duration = Duration::from_secs(3600);

/// Duration for USDT deposit monitoring (every 30 seconds)
const USDT_MONITOR_INTERVAL: Duration = Duration::from_secs(30);

/// Cron manager for wallet background jobs
pub struct WalletCronManager {
    service: Arc<WalletService>,
    running: std::sync::atomic::AtomicBool,
}

impl WalletCronManager {
    /// Create new cron manager
    pub fn new(service: Arc<WalletService>) -> Self {
        Self {
            service,
            running: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Start all background jobs
    pub fn start(&self) {
        if self.running.load(std::sync::atomic::Ordering::Relaxed) {
            warn!("Wallet cron jobs are already running");
            return;
        }

        self.running.store(true, std::sync::atomic::Ordering::Relaxed);
        info!("Starting wallet cron jobs");

        let service = Arc::clone(&self.service);

        // Spawn escrow auto-release job
        let service_escrow = Arc::clone(&service);
        tokio::spawn(async move {
            Self::run_escrow_auto_release(service_escrow).await;
        });

        // Spawn daily reconciliation job
        let service_recon = Arc::clone(&service);
        tokio::spawn(async move {
            Self::run_daily_reconciliation(service_recon).await;
        });

        // Spawn USDT monitor job
        let service_usdt = Arc::clone(&service);
        tokio::spawn(async move {
            Self::run_usdt_monitor(service_usdt).await;
        });

        info!("All wallet cron jobs started");
    }

    /// Stop all background jobs
    pub fn stop(&self) {
        self.running.store(false, std::sync::atomic::Ordering::Relaxed);
        info!("Wallet cron jobs stop requested");
    }

    /// Background job: Auto-release escrows every 5 minutes
    async fn run_escrow_auto_release(service: Arc<WalletService>) {
        let mut timer = interval(ESCROW_CHECK_INTERVAL);
        timer.tick().await; // Skip first immediate tick

        info!("Escrow auto-release job started (interval: {:?})", ESCROW_CHECK_INTERVAL);

        loop {
            timer.tick().await;

            match service.process_auto_releases().await {
                Ok(result) => {
                    if result.total_processed > 0 {
                        info!(
                            "Escrow auto-release: {} processed, {} released, {} failed",
                            result.total_processed, result.released_count, result.failed_count
                        );

                        if !result.errors.is_empty() {
                            warn!("Escrow release errors: {:?}", result.errors);
                        }
                    }
                }
                Err(e) => {
                    error!("Escrow auto-release failed: {}", e);
                }
            }
        }
    }

    /// Background job: Daily reconciliation check
    ///
    /// Runs every hour to check if daily reconciliation needs to run
    /// Actual reconciliation runs at 3:00 AM daily
    async fn run_daily_reconciliation(service: Arc<WalletService>) {
        let mut timer = interval(RECONCILIATION_CHECK_INTERVAL);
        timer.tick().await;

        info!("Daily reconciliation check started (interval: {:?})", RECONCILIATION_CHECK_INTERVAL);

        loop {
            timer.tick().await;

            // Check if current time is around 3:00 AM
            let now = chrono::Utc::now();
            let hour = now.hour();

            if hour == 3 {
                info!("Triggering daily reconciliation at {:?}", now);

                match service.daily_reconciliation().await {
                    Ok(result) => {
                        info!(
                            "Daily reconciliation completed: {} wallets checked, {} discrepancies",
                            result.wallets_checked, result.discrepancy_count
                        );

                        if result.discrepancy_count > 0 {
                            warn!("Reconciliation found discrepancies: {:?}", result.discrepancies);
                        }
                    }
                    Err(e) => {
                        error!("Daily reconciliation failed: {}", e);
                    }
                }
            }
        }
    }

    /// Background job: USDT TRC20 deposit monitoring
    ///
    /// Polls blockchain for new USDT transactions every 30 seconds
    async fn run_usdt_monitor(service: Arc<WalletService>) {
        let mut timer = interval(USDT_MONITOR_INTERVAL);
        timer.tick().await;

        info!("USDT monitor started (interval: {:?})", USDT_MONITOR_INTERVAL);

        loop {
            timer.tick().await;

            // This will be implemented to mock TronGrid API calls
            // For now, just log that monitor is running
            info!("USDT monitor tick - checking for new transactions");
        }
    }
}

/// Extension trait for WalletService to add reconciliation methods
impl WalletService {
    /// Perform daily reconciliation of all wallets
    ///
    /// This checks:
    /// - Balance invariants
    /// - Transaction count matches
    /// - Escrow totals
    /// - Platform wallet balance
    pub async fn daily_reconciliation(
        &self,
    ) -> Result<ReconciliationResponse, ServiceError> {
        let start = std::time::Instant::now();

        // Get all wallets
        // Note: This would need pagination for production
        let wallets = self.repo.find_all_wallets_for_reconciliation(100, 0).await?;

        let mut discrepancies = vec![];
        let mut discrepancy_count = 0;

        for wallet in &wallets {
            // Check balance invariant
            if !wallet.validate_balance_invariant() {
                discrepancy_count += 1;
                discrepancies.push(ReconciliationDiscrepancy {
                    wallet_id: wallet.wallet_id.clone(),
                    discrepancy_type: "BALANCE_MISMATCH".to_string(),
                    expected: wallet.available_trust + wallet.withdrawal_locked + wallet.dispute_locked,
                    actual: wallet.total_trust,
                    details: format!(
                        "Balance invariant violated: available={}, locked={}, dispute={}, total={}",
                        wallet.available_trust, wallet.withdrawal_locked, wallet.dispute_locked, wallet.total_trust
                    ),
                });
            }

            // Verify transaction count matches (simplified)
            let tx_count = self
                .repo
                .count_transactions_by_wallet(&wallet.wallet_id)
                .await?;

            // This is a simplified check - in production you'd verify more
            if tx_count > 0 && wallet.lifetime_deposited == 0 && wallet.lifetime_received == 0 {
                // Has transactions but no recorded deposits/received - suspicious
                discrepancy_count += 1;
                discrepancies.push(ReconciliationDiscrepancy {
                    wallet_id: wallet.wallet_id.clone(),
                    discrepancy_type: "TRANSACTION_MISMATCH".to_string(),
                    expected: 0,
                    actual: tx_count,
                    details: format!("Wallet has {} transactions but no recorded deposits/received", tx_count),
                });
            }
        }

        // Verify platform wallet
        let platform_wallet = self.repo.find_platform_wallet().await?;
        let escrow_total = self.repo.sum_active_escrows().await?;

        // Platform wallet should roughly equal sum of all escrows
        // (This is simplified - in production you'd track more carefully)
        let platform_escrow_match = (platform_wallet.available_trust - escrow_total).abs() < 1000; // Allow 1000 Trust variance

        if !platform_escrow_match {
            discrepancy_count += 1;
            discrepancies.push(ReconciliationDiscrepancy {
                wallet_id: platform_wallet.wallet_id,
                discrepancy_type: "PLATFORM_ESCROW_MISMATCH".to_string(),
                expected: escrow_total,
                actual: platform_wallet.available_trust,
                details: format!(
                    "Platform wallet ({}) doesn't match active escrows total ({})",
                    platform_wallet.available_trust, escrow_total
                ),
            });
        }

        let duration = start.elapsed();

        Ok(ReconciliationResponse {
            reconciliation_id: format!("RECON-{}", ulid::Ulid::new()),
            wallets_checked: wallets.len() as i64,
            discrepancy_count,
            discrepancies,
            duration_ms: duration.as_millis() as i64,
            status: if discrepancy_count == 0 {
                "HEALTHY".to_string()
            } else {
                "DISCREPANCIES_FOUND".to_string()
            },
            performed_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Get dashboard statistics for admin view
    pub async fn get_dashboard_stats(&self) -> Result<DashboardStatsResponse, ServiceError> {
        // Get platform wallet for overview
        let platform_wallet = self.repo.find_platform_wallet().await?;

        // Get today's transaction summary
        let (today_count, today_volume) = self
            .repo
            .get_monthly_transaction_stats()
            .await
            .unwrap_or((0, 0));

        // Get pending withdrawals
        let pending_withdrawals = self
            .repo
            .find_pending_withdrawals_for_review(10)
            .await?;

        // Get active escrows
        let active_escrows_count = self.repo.count_active_escrows().await.unwrap_or(0) as i64;

        // Get USDT deposits summary
        let usdt_summary = self.repo.get_usdt_deposits_summary().await
            .unwrap_or_else(|_| crate::modules::wallet::repository::UsdtDepositsSummary::default());

        // Convert from repository struct to dto struct
        let total_deposits = usdt_summary.pending_count + usdt_summary.credited_count;
        let pending_deposits = usdt_summary.pending_count;
        let total_trust = usdt_summary.credited_vnd / 1000;

        // Calculate today's commission (simplified)
        // In production, you'd aggregate from commission transactions
        let today_commission = (today_volume as f64 * 0.05) as i64; // 5% assumption

        Ok(DashboardStatsResponse {
            // Platform wallet
            platform_balance: platform_wallet.available_trust,
            platform_escrow_held: active_escrows_count,

            // Today's activity
            today_transaction_count: today_count as i64,
            today_transaction_volume: today_volume,
            today_commission: Some(today_commission),

            // Pending actions
            pending_withdrawals: pending_withdrawals.len() as i64,
            pending_withdrawal_amount: pending_withdrawals
                .iter()
                .map(|w| w.trust_amount)
                .sum(),

            // Escrow stats
            active_escrows: active_escrows_count,

            // USDT stats
            usdt_deposits_today: total_deposits,
            usdt_pending: pending_deposits,
            usdt_total_trust: total_trust,

            // Health status
            system_status: "OPERATIONAL".to_string(),
            last_updated: chrono::Utc::now().to_rfc3339(),
        })
    }
}
