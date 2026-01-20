use super::handler;
use actix_web::web;

pub fn configure_user(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/wallet")
            .route("/balance", web::get().to(handler::get_balance))
            .route(
                "/transactions",
                web::get().to(handler::get_transaction_history),
            )
            .route(
                "/deposit/initiate",
                web::post().to(handler::initiate_deposit),
            )
            .route(
                "/deposit/status/{tx_id}",
                web::get().to(handler::get_deposit_status),
            )
            .route(
                "/deposits/history",
                web::get().to(handler::get_deposit_history),
            )
            .route("/withdrawal", web::post().to(handler::create_withdrawal))
            .route("/purchase", web::post().to(handler::create_purchase))
            .route(
                "/escrow/{escrow_id}/early-release",
                web::post().to(handler::early_release_escrow),
            )
            .route(
                "/escrow/{escrow_id}/dispute",
                web::post().to(handler::create_dispute),
            )
            .route("/disputes", web::get().to(handler::get_disputes_list))
            .route(
                "/disputes/{dispute_id}",
                web::get().to(handler::get_dispute_detail),
            )
            .route(
                "/disputes/{dispute_id}/seller/respond",
                web::post().to(handler::seller_respond_dispute),
            )
            .route(
                "/disputes/{dispute_id}/buyer/respond",
                web::post().to(handler::buyer_respond_dispute),
            ),
    );
}

pub fn configure_admin(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin/wallet")
            .route("/freeze", web::post().to(handler::freeze_wallet))
            .route("/unfreeze", web::post().to(handler::unfreeze_wallet))
            .route("/debit", web::post().to(handler::manual_debit))
            .route("/commission", web::post().to(handler::set_shop_commission))
            .route("/deposit", web::post().to(handler::admin_manual_deposit))
            .route("/deposit/manual", web::post().to(handler::manual_deposit))
            .route(
                "/deposit/auto",
                web::post().to(handler::create_auto_deposit),
            )
            .route("/deposit/webhook", web::post().to(handler::deposit_webhook))
            .route(
                "/deposits/history",
                web::get().to(handler::admin_get_deposits_history),
            )
            .route(
                "/withdrawal/{request_id}/validate",
                web::get().to(handler::validate_withdrawal),
            )
            .route(
                "/withdrawal/{request_id}/approve",
                web::post().to(handler::approve_withdrawal),
            )
            .route(
                "/withdrawal/{request_id}/reject",
                web::post().to(handler::reject_withdrawal),
            )
            .route(
                "/withdrawal/{request_id}/complete",
                web::post().to(handler::complete_bank_transfer),
            )
            .route(
                "/escrow/{escrow_id}/resolve/refund",
                web::post().to(handler::resolve_dispute_refund),
            )
            .route(
                "/escrow/{escrow_id}/resolve/release",
                web::post().to(handler::resolve_dispute_release),
            )
            .route(
                "/escrow/jobs/auto-release",
                web::post().to(handler::process_auto_releases),
            )
            .route(
                "/disputes/{dispute_id}/extend",
                web::post().to(handler::admin_extend_deadline),
            )
            .route(
                "/disputes/partial-refund",
                web::post().to(handler::admin_partial_refund),
            )
            .route(
                "/disputes/jobs/auto-escalate",
                web::post().to(handler::process_auto_escalate),
            )
            .route("/logs", web::get().to(handler::get_admin_logs))
            .route("/dashboard", web::get().to(handler::get_dashboard_stats))
            .route(
                "/reconcile",
                web::post().to(handler::trigger_reconciliation),
            )
            .route("/cron/start", web::post().to(handler::start_cron_jobs))
            .route("/cron/stop", web::post().to(handler::stop_cron_jobs)),
    );
}
