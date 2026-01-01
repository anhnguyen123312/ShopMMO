//! Permission routes
//!
//! Route configuration for permission and role management APIs.

use actix_web::web;

use super::handler::*;

/// Configure permission routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/permissions")
            // Permission endpoints
            .route("", web::get().to(list_permissions))
            // Role management endpoints
            .service(
                web::scope("/roles")
                    .route("", web::post().to(create_role))
                    .route("", web::get().to(list_roles))
                    .route("/{role_name}", web::delete().to(delete_role))
                    .route("/{role_name}/permissions", web::put().to(update_role_permissions))
                    .route("/assign", web::post().to(assign_role))
            ),
    );
}
