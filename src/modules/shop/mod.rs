//! Shop Module - P2PMMO V2
//!
//! Module quản lý shop/vendor theo flow V2:
//! - Auto approve khi tạo shop (không cần admin duyệt)
//! - Telegram verification là REQUIRED
//! - Shop completion cần: Telegram + Products + Policies

pub mod domain;
pub mod dto;
pub mod repository;
pub mod service;
pub mod handler;
pub mod routes;
pub mod upload;

// Re-export commonly used types
