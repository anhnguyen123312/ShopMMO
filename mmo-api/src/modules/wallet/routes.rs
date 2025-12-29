//! Wallet routes
//!
//! Route configuration for wallet endpoints.

use actix_web::web;

use super::handler;

/// Configures wallet routes
///
/// All routes require authentication
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/wallet")
            .route("/balance", web::get().to(handler::get_balance)),
        // TODO: Add more routes:
        // .route("/transfer", web::post().to(handler::transfer_ap))
        // .route("/transactions", web::get().to(handler::get_transactions))
        // .route("/withdraw", web::post().to(handler::request_withdrawal))
    );
}
