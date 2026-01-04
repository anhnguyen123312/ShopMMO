//! Wallet V3 Routes
//!
//! Route configuration for all wallet endpoints

use actix_web::web;
use super::handler;

/// Configure ALL wallet routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    // ========================================================================
    // INTERNAL APIs (Admin tools, microservice communication)
    // ========================================================================
    cfg.service(
        web::scope("/internal/wallet")
            // ============== WALLET MANAGEMENT ==============
            .route("/balance", web::get().to(handler::get_balance))
            .route("/create", web::post().to(handler::create_wallet))

            // ============== DEPOSIT FLOW ==============
            .route("/deposit/auto", web::post().to(handler::create_auto_deposit))
            .route("/deposit/manual", web::post().to(handler::manual_deposit))

            // ============== WITHDRAWAL FLOW ==============
            .route("/withdrawal", web::post().to(handler::create_withdrawal))
            .route("/withdrawal/{request_id}/validate", web::get().to(handler::validate_withdrawal))
            .route("/withdrawal/{request_id}/approve", web::post().to(handler::approve_withdrawal))
            .route("/withdrawal/{request_id}/reject", web::post().to(handler::reject_withdrawal))
            .route("/withdrawal/{request_id}/complete", web::post().to(handler::complete_bank_transfer))

            // ============== PURCHASE & ESCROW ==============
            .route("/purchase", web::post().to(handler::create_purchase))
            .route("/escrow/{escrow_id}/early-release", web::post().to(handler::early_release_escrow))
            .route("/escrow/{escrow_id}/dispute", web::post().to(handler::create_dispute))
            .route("/escrow/{escrow_id}/resolve/refund", web::post().to(handler::resolve_dispute_refund))
            .route("/escrow/{escrow_id}/resolve/release", web::post().to(handler::resolve_dispute_release))

            // ============== ADMIN OPERATIONS ==============
            .route("/admin/debit", web::post().to(handler::manual_debit))
            .route("/admin/freeze", web::post().to(handler::freeze_wallet))
            .route("/admin/unfreeze", web::post().to(handler::unfreeze_wallet))
            .route("/admin/commission", web::post().to(handler::set_shop_commission))
            .route("/admin/logs", web::get().to(handler::get_admin_logs))

            // ============== TRANSACTION HISTORY ==============
            .route("/transactions", web::get().to(handler::get_transaction_history))

            // ============== BACKGROUND JOBS (Internal/Cron) ==============
            .route("/jobs/auto-release", web::post().to(handler::process_auto_releases))
    );

    // ========================================================================
    // PUBLIC APIs (V3) - External clients (Mobile, Web, 3rd parties)
    // ========================================================================

    // ============== V3 USER WALLET APIs ==============
    cfg.service(
        web::scope("/api/v3/wallet")
            // ============== DEPOSIT ENDPOINTS ==============
            .route("/deposit/initiate", web::post().to(handler::initiate_deposit))
            .route("/deposit/webhook", web::post().to(handler::deposit_webhook))
            .route("/deposit/status/{tx_id}", web::get().to(handler::get_deposit_status))
            .route("/deposits/history", web::get().to(handler::get_deposit_history))
    );

    // ============== V3 ADMIN WALLET APIs ==============
    cfg.service(
        web::scope("/api/v3/admin/wallets")
            // ============== DEPOSIT MANAGEMENT ==============
            .route("/deposit", web::post().to(handler::admin_manual_deposit))
            .route("/deposits/history", web::get().to(handler::admin_get_deposits_history))

            // ============== DASHBOARD & MONITORING ==============
            .route("/dashboard", web::get().to(handler::get_dashboard_stats))
            .route("/reconcile", web::post().to(handler::trigger_reconciliation))

            // ============== CRON JOB CONTROLS ==============
            .route("/cron/start", web::post().to(handler::start_cron_jobs))
            .route("/cron/stop", web::post().to(handler::stop_cron_jobs))
    );
}
