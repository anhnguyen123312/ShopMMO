//! Category routes
//!
//! Route configuration for category endpoints.

use actix_web::web;

use super::handler;

/// Configures category routes
///
/// # Arguments
/// * `cfg` - Service configuration
///
/// # Public Routes
/// - GET /api/categories/tree - List categories as tree
/// - GET /api/categories/{id} - Get category by ID
///
/// # Admin Routes (require authentication + admin role)
/// - POST /api/admin/categories - Create new category
/// - PUT /api/admin/categories/{id} - Update category
/// - DELETE /api/admin/categories/{id} - Delete category
/// - POST /api/admin/categories/reorder - Reorder categories
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/categories")
            .route("/tree", web::get().to(handler::list_categories_tree))
            .route("/{id}", web::get().to(handler::get_category)),
    )
    .service(
        web::scope("/admin/categories")
            .route("", web::post().to(handler::create_category))
            .route("/reorder", web::post().to(handler::reorder_categories))
            .route("/{id}", web::put().to(handler::update_category))
            .route("/{id}", web::delete().to(handler::delete_category)),
    );
}
