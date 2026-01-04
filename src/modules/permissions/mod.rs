//! Permissions module for V2 authorization system
//!
//! This module implements dynamic RBAC (Role-Based Access Control) with:
//! - Permission-based access control using `resource:action` format
//! - Role hierarchy with inheritance
//! - Redis caching for performance
//! - Ownership-based resource access

pub mod constants;
pub mod domain;
pub mod dto;
pub mod repository;
pub mod service;
pub mod handler;
pub mod routes;
pub mod cache;

pub use constants::{Permission, all_permissions, all_permissions_set, is_valid_permission};
pub use domain::*;
