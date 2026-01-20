//! Authentication routes

use actix_web::web;

use super::handler;

/// Public auth routes (no authentication required)
pub fn configure_public(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/register", web::post().to(handler::register))
            .route("/login", web::post().to(handler::login))
            .route("/refresh", web::post().to(handler::refresh_token)),
    );
}

/// Protected auth routes (authentication required)
pub fn configure_protected(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/auth")
            .route("/logout", web::post().to(handler::logout))
            .route("/me", web::get().to(handler::get_me))
            .route("/change-password", web::post().to(handler::change_password))
            .service(
                web::scope("/admin")
                    .route("/assign-roles", web::post().to(handler::assign_roles))
                    .route(
                        "/users/{user_id}/roles",
                        web::get().to(handler::get_user_roles),
                    ),
            ),
    );
}
