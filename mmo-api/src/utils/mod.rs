//! Utility module
//!
//! Common utility functions used across the application.

pub mod datetime;
pub mod hash;
pub mod jwt;
pub mod number_generator;

// Re-export commonly used functions
pub use datetime::{add_days, add_hours, now_bson};
pub use hash::{hash_password, verify_password};
pub use jwt::{generate_access_token, generate_refresh_token, verify_token, TokenClaims};
pub use number_generator::{
    generate_deposit_number, generate_escrow_number, generate_order_number, generate_request_id,
    generate_transaction_number, generate_withdrawal_number,
};
