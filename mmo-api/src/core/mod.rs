//! Core module
//!
//! Contains fundamental infrastructure components used across the application:
//! - Error handling and types
//! - Standard API response structures
//! - Logging configuration
//! - Input validation utilities

pub mod errors;
pub mod logger;
pub mod response;
pub mod validator;

// Create error module alias for backward compatibility
pub use errors as error;

// Re-export commonly used types
pub use errors::{ApiError, DbError, ServiceError};
pub use response::{ApiResponse, MessageResponse, PaginatedResponse};
