//! Authentication routes
//!
//! Route configuration for authentication endpoints.

use actix_web::web;

use super::handler;

/// Configures authentication routes
///
/// # Arguments
/// * `cfg` - Service configuration
///
/// # Routes
/// - POST /auth/register - Register new user
/// - POST /auth/login - Login
/// - POST /auth/refresh - Refresh access token
/// - POST /auth/logout - Logout (requires auth)
/// - GET /auth/me - Get current user (requires auth)
/// - POST /auth/change-password - Change password (requires auth)
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/register", web::post().to(handler::register))
            .route("/login", web::post().to(handler::login))
            .route("/refresh", web::post().to(handler::refresh_token))
            .route("/logout", web::post().to(handler::logout))
            .route("/me", web::get().to(handler::get_me))
            .route("/change-password", web::post().to(handler::change_password)),
    );
}
