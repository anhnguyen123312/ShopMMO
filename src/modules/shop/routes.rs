//! Shop Routes - P2PMMO V2
//!
//! Route configuration for all Shop endpoints
//!
//! # API Structure:
//! - `/api/vendor/shop/*` - Vendor endpoints (require vendor role)
//! - `/api/shops/*` - Public endpoints
//! - `/admin/api/shops/*` - Admin endpoints

use actix_web::web;
use super::{handler, upload::upload_logo, upload::upload_banner};

/// Configure ALL shop routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    // ========================================================================
    // VENDOR SHOP APIs
    // ========================================================================
    cfg.service(
        web::scope("/api/vendor/shop")
            // ============== CREATE & DASHBOARD ==============
            .route("/create", web::post().to(handler::create_shop))
            .route("/dashboard", web::get().to(handler::get_dashboard))
            .route("/verification", web::get().to(handler::get_verification_info))

            // ============== UPDATE SHOP ==============
            .route("/update", web::put().to(handler::update_shop))
            .route("/policies", web::put().to(handler::update_policies))

            // ============== FILE UPLOAD ==============
            .route("/upload/logo", web::post().to(upload_logo))
            .route("/upload/banner", web::post().to(upload_banner))
    );

    // ========================================================================
    // PUBLIC SHOP APIs
    // ========================================================================
    cfg.service(
        web::scope("/api/shops")
            // ============== GET SHOP ==============
            .route("/{shop_id}", web::get().to(handler::get_shop))
            .route("/slug/{slug}", web::get().to(handler::get_shop_by_slug))

            // ============== LIST & SEARCH ==============
            .route("", web::get().to(handler::list_shops))
            .route("/search/{term}", web::get().to(handler::search_shops))
    );

    // ========================================================================
    // INTERNAL TELEGRAM BOT API
    // ========================================================================
    cfg.service(
        web::scope("/api/shop")
            // ============== TELEGRAM VERIFICATION ==============
            .route("/telegram/verify", web::post().to(handler::verify_telegram))
    );

    // ========================================================================
    // ADMIN SHOP APIs
    // ========================================================================
    cfg.service(
        web::scope("/admin/api/shops")
            // ============== STATISTICS ==============
            .route("/stats", web::get().to(handler::get_stats))
    );
}
