//! Permission routes
//!
//! Route configuration for permission APIs.

use actix_web::web;

use super::handler::*;

/// Configure permission routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1/permissions")
            .route("", web::get().to(list_permissions))
            // TODO: Add more routes as needed:
            // .route("", web::post().to(create_permission))
            // .route("/{id}", web::get().to(get_permission))
            // .route("/{id}", web::put().to(update_permission))
            // .route("/{id}", web::delete().to(delete_permission))
    );
}
