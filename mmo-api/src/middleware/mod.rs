//! Middleware module
//!
//! Contains middleware for cross-cutting concerns like authentication,
//! authorization, request tracking, and CORS.

pub mod auth;
pub mod authorization;
pub mod cors;
pub mod request_id;

pub use auth::{AuthMiddleware, AuthUser, AdminUser};
pub use authorization::{RequireRole, UserRole};
pub use cors::configure_cors;
pub use request_id::RequestId;
