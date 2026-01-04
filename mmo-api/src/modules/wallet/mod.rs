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
pub mod service_usdt;
pub mod service_cron;

pub use domain::*;
pub use dto::*;
pub use repository::WalletRepository;
pub use service::WalletService;
pub use service_cron::WalletCronManager;
pub use service_usdt::{WalletUsdtService, UsdtConfig};
