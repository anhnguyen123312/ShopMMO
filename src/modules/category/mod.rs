//! Category module
//!
//! Manages product categories with per-category inventory collections.

pub mod domain;
pub mod dto;
pub mod handler;
pub mod repository;
pub mod routes;
pub mod service;

pub use repository::CategoryRepository;
pub use service::CategoryService;
