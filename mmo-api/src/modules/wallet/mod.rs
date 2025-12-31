//! Wallet V3 Module
//!
//! Complete Trust Currency wallet system with deposit, withdrawal, purchase, escrow, and admin operations

pub mod domain;
pub mod dto;
pub mod handler;
pub mod repository;
pub mod routes;
pub mod service;
pub mod service_escrow;
pub mod service_admin;

pub use domain::*;
pub use dto::*;
pub use repository::WalletRepository;
pub use service::WalletService;
