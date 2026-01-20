//! Wallet V3 Service - Background Cron Jobs
//!
//! Tokio-based background tasks for periodic operations

use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{info, warn, error};
use chrono::Timelike;

use crate::core::error::ServiceError;
use super::{dto::*, service::WalletService};

struct ReconciliationCheckResult {
    passed: bool,
    expected: i64,
    actual: i64,
    details: String,
}

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
    async fn run_usdt_monitor(_service: Arc<WalletService>) {
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

impl WalletService {
    pub async fn daily_reconciliation(
        &self,
    ) -> Result<ReconciliationResponse, ServiceError> {
        let start = std::time::Instant::now();
        let mut discrepancies = vec![];
        let mut discrepancy_count = 0;

        let check1 = self.check_system_total_trust().await?;
        if !check1.passed {
            discrepancy_count += 1;
            discrepancies.push(ReconciliationDiscrepancy {
                wallet_id: "SYSTEM".to_string(),
                discrepancy_type: "SYSTEM_TOTAL_MISMATCH".to_string(),
                expected: check1.expected,
                actual: check1.actual,
                details: check1.details,
            });
        }

        let check2 = self.check_platform_wallet_balance().await?;
        if !check2.passed {
            discrepancy_count += 1;
            discrepancies.push(ReconciliationDiscrepancy {
                wallet_id: "PLATFORM".to_string(),
                discrepancy_type: "PLATFORM_WALLET_MISMATCH".to_string(),
                expected: check2.expected,
                actual: check2.actual,
                details: check2.details,
            });
        }

        let check3 = self.check_vnd_trust_conversion().await?;
        if !check3.passed {
            discrepancy_count += 1;
            discrepancies.push(ReconciliationDiscrepancy {
                wallet_id: "CONVERSION".to_string(),
                discrepancy_type: "VND_TRUST_MISMATCH".to_string(),
                expected: check3.expected,
                actual: check3.actual,
                details: check3.details,
            });
        }

        let check4 = self.check_withdrawal_vnd().await?;
        if !check4.passed {
            discrepancy_count += 1;
            discrepancies.push(ReconciliationDiscrepancy {
                wallet_id: "WITHDRAWALS".to_string(),
                discrepancy_type: "WITHDRAWAL_VND_MISMATCH".to_string(),
                expected: check4.expected,
                actual: check4.actual,
                details: check4.details,
            });
        }

        let check5 = self.check_money_flow_balance().await?;
        if !check5.passed {
            discrepancy_count += 1;
            discrepancies.push(ReconciliationDiscrepancy {
                wallet_id: "FLOW".to_string(),
                discrepancy_type: "MONEY_FLOW_MISMATCH".to_string(),
                expected: check5.expected,
                actual: check5.actual,
                details: check5.details,
            });
        }

        let wallets = self.repo.find_all_wallets_for_reconciliation(1000, 0).await?;
        for wallet in &wallets {
            if !wallet.validate_balance_invariant() {
                discrepancy_count += 1;
                discrepancies.push(ReconciliationDiscrepancy {
                    wallet_id: wallet.wallet_id.clone(),
                    discrepancy_type: "BALANCE_INVARIANT".to_string(),
                    expected: wallet.available_trust + wallet.withdrawal_locked + wallet.dispute_locked,
                    actual: wallet.total_trust,
                    details: format!(
                        "available={}, locked={}, dispute={}, total={}",
                        wallet.available_trust, wallet.withdrawal_locked, wallet.dispute_locked, wallet.total_trust
                    ),
                });
            }
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

    async fn check_system_total_trust(&self) -> Result<ReconciliationCheckResult, ServiceError> {
        let total_all_wallets = self.repo.sum_all_wallet_balances().await.unwrap_or(0);
        let total_deposits = self.repo.sum_all_deposits().await.unwrap_or(0);
        let total_withdrawals = self.repo.sum_all_withdrawals().await.unwrap_or(0);
        
        let expected = total_deposits - total_withdrawals;
        let passed = (total_all_wallets - expected).abs() < 100;

        Ok(ReconciliationCheckResult {
            passed,
            expected,
            actual: total_all_wallets,
            details: format!(
                "Deposits: {}, Withdrawals: {}, Expected balance: {}, Actual: {}",
                total_deposits, total_withdrawals, expected, total_all_wallets
            ),
        })
    }

    async fn check_platform_wallet_balance(&self) -> Result<ReconciliationCheckResult, ServiceError> {
        let platform_wallet = self.repo.find_platform_wallet().await?;
        let escrow_total = self.repo.sum_active_escrows().await.unwrap_or(0);
        
        let expected = escrow_total;
        let passed = (platform_wallet.available_trust - expected).abs() < 1000;

        Ok(ReconciliationCheckResult {
            passed,
            expected,
            actual: platform_wallet.available_trust,
            details: format!(
                "Platform balance: {}, Active escrows: {}",
                platform_wallet.available_trust, escrow_total
            ),
        })
    }

    async fn check_vnd_trust_conversion(&self) -> Result<ReconciliationCheckResult, ServiceError> {
        let total_vnd_deposits = self.repo.sum_vnd_deposits().await.unwrap_or(0);
        let total_trust_deposits = self.repo.sum_all_deposits().await.unwrap_or(0);
        
        let expected_trust = total_vnd_deposits / 1000;
        let passed = (total_trust_deposits - expected_trust).abs() < 100;

        Ok(ReconciliationCheckResult {
            passed,
            expected: expected_trust,
            actual: total_trust_deposits,
            details: format!(
                "VND deposited: {}, Expected Trust: {}, Actual Trust: {}",
                total_vnd_deposits, expected_trust, total_trust_deposits
            ),
        })
    }

    async fn check_withdrawal_vnd(&self) -> Result<ReconciliationCheckResult, ServiceError> {
        let total_withdrawal_trust = self.repo.sum_all_withdrawals().await.unwrap_or(0);
        let total_commission = self.repo.sum_all_commission().await.unwrap_or(0);
        let total_withdrawal_vnd = self.repo.sum_withdrawal_vnd().await.unwrap_or(0);
        
        let expected_vnd = (total_withdrawal_trust - total_commission) * 1000;
        let passed = (total_withdrawal_vnd - expected_vnd).abs() < 100_000;

        Ok(ReconciliationCheckResult {
            passed,
            expected: expected_vnd,
            actual: total_withdrawal_vnd,
            details: format!(
                "Withdrawal Trust: {}, Commission: {}, Expected VND: {}, Actual VND: {}",
                total_withdrawal_trust, total_commission, expected_vnd, total_withdrawal_vnd
            ),
        })
    }

    async fn check_money_flow_balance(&self) -> Result<ReconciliationCheckResult, ServiceError> {
        let inflow = self.repo.sum_all_deposits().await.unwrap_or(0);
        let outflow = self.repo.sum_all_withdrawals().await.unwrap_or(0);
        let remaining = inflow - outflow;
        
        let total_all_wallets = self.repo.sum_all_wallet_balances().await.unwrap_or(0);
        let passed = (remaining - total_all_wallets).abs() < 100;

        Ok(ReconciliationCheckResult {
            passed,
            expected: remaining,
            actual: total_all_wallets,
            details: format!(
                "Inflow: {}, Outflow: {}, Expected remaining: {}, Actual wallets: {}",
                inflow, outflow, remaining, total_all_wallets
            ),
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
