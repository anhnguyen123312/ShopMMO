//! Utility module
//!
//! Common utility functions used across the application.

pub mod datetime;
pub mod hash;
pub mod jwt;
pub mod number_generator;

// Re-export commonly used functions
pub use datetime::add_days;
pub use hash::{hash_password, verify_password};
pub use jwt::{
    generate_access_token_v2, generate_refresh_token_v2,
    verify_token
};
pub use number_generator::generate_request_id;
