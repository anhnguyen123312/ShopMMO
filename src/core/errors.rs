//! Error handling module for the MMO API
//!
//! This module provides a comprehensive error handling system with:
//! - Type-safe error variants
//! - Automatic HTTP response mapping
//! - Error logging integration
//! - User-friendly error messages

use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use serde::Serialize;
use thiserror::Error;
use utoipa::ToSchema;

use super::response::ApiResponse;

/// Main API error type
///
/// All errors in the application should be convertible to this type.
/// This ensures consistent error handling across all layers.
#[derive(Debug, Error, ToSchema)]
pub enum ApiError {
    /// Resource not found (404)
    #[error("Resource not found: {message}")]
    NotFound { message: String },

    /// Bad request - validation errors (400)
    #[error("Validation error: {message}")]
    BadRequest { message: String },

    /// Unauthorized - missing or invalid authentication (401)
    #[error("Unauthorized: {message}")]
    Unauthorized { message: String },

    /// Forbidden - insufficient permissions (403)
    #[error("Forbidden: {message}")]
    Forbidden { message: String },

    /// Conflict - resource already exists (409)
    #[error("Conflict: {message}")]
    Conflict { message: String },

    /// Internal server error (500)
    #[error("Internal server error: {message}")]
    InternalError { message: String },

    /// Database error (500)
    #[error("Database error: {message}")]
    DatabaseError { message: String },

    /// External service error (502)
    #[error("External service error: {message}")]
    ExternalServiceError { message: String },
}

impl ApiError {
    /// Creates a NotFound error
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound {
            message: message.into(),
        }
    }

    /// Creates a BadRequest error
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest {
            message: message.into(),
        }
    }

    /// Creates an Unauthorized error
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::Unauthorized {
            message: message.into(),
        }
    }

    /// Creates a Forbidden error
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::Forbidden {
            message: message.into(),
        }
    }

    /// Creates a Conflict error
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict {
            message: message.into(),
        }
    }

    /// Creates an InternalError
    pub fn internal(message: impl Into<String>) -> Self {
        Self::InternalError {
            message: message.into(),
        }
    }

    /// Creates a DatabaseError
    pub fn database(message: impl Into<String>) -> Self {
        Self::DatabaseError {
            message: message.into(),
        }
    }

    /// Gets the HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound { .. } => StatusCode::NOT_FOUND,
            Self::BadRequest { .. } => StatusCode::BAD_REQUEST,
            Self::Unauthorized { .. } => StatusCode::UNAUTHORIZED,
            Self::Forbidden { .. } => StatusCode::FORBIDDEN,
            Self::Conflict { .. } => StatusCode::CONFLICT,
            Self::InternalError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Self::DatabaseError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ExternalServiceError { .. } => StatusCode::BAD_GATEWAY,
        }
    }
}

/// Implement ResponseError trait for ApiError
///
/// This allows ApiError to be returned directly from handlers and
/// automatically converted to HTTP responses.
impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        self.status_code()
    }

    fn error_response(&self) -> HttpResponse {
        // Log the error
        tracing::error!(
            error = %self,
            status_code = %self.status_code(),
            "API error occurred"
        );

        // Create error response
        let error_response = ErrorResponse {
            error: self.to_string(),
            status_code: self.status_code().as_u16(),
        };

        HttpResponse::build(self.status_code()).json(ApiResponse::<()>::error_detailed(
            self.to_string(),
            Some(error_response),
        ))
    }
}

/// Error response structure for detailed error information
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub status_code: u16,
}

/// Service layer error type
///
/// Used for business logic errors that need to be converted to ApiError
#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Insufficient balance")]
    InsufficientBalance,

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Database operation failed: {0}")]
    DatabaseError(String),

    #[error("Internal service error: {0}")]
    InternalError(String),
}

/// Convert ServiceError to ApiError
impl From<ServiceError> for ApiError {
    fn from(err: ServiceError) -> Self {
        match err {
            ServiceError::NotFound(msg) => ApiError::not_found(msg),
            ServiceError::BadRequest(msg) => ApiError::bad_request(msg),
            ServiceError::ValidationFailed(msg) => ApiError::bad_request(msg),
            ServiceError::InsufficientBalance => {
                ApiError::bad_request("Insufficient balance for this operation")
            }
            ServiceError::Unauthorized(msg) => ApiError::unauthorized(msg),
            ServiceError::Forbidden(msg) => ApiError::forbidden(msg),
            ServiceError::DatabaseError(msg) => ApiError::database(msg),
            ServiceError::InternalError(msg) => ApiError::internal(msg),
        }
    }
}

/// Database layer error type
#[derive(Debug, Error)]
pub enum DbError {
    #[error("MongoDB error: {0}")]
    MongoError(String),

    #[error("Redis error: {0}")]
    RedisError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Resource not found: {0}")]
    NotFound(String),
}

/// Convert MongoDB errors
impl From<mongodb::error::Error> for DbError {
    fn from(err: mongodb::error::Error) -> Self {
        DbError::MongoError(err.to_string())
    }
}

/// Convert BSON errors
impl From<bson::ser::Error> for DbError {
    fn from(err: bson::ser::Error) -> Self {
        DbError::SerializationError(err.to_string())
    }
}

impl From<bson::de::Error> for DbError {
    fn from(err: bson::de::Error) -> Self {
        DbError::SerializationError(err.to_string())
    }
}

/// Convert DbError to ServiceError
impl From<DbError> for ServiceError {
    fn from(err: DbError) -> Self {
        ServiceError::DatabaseError(err.to_string())
    }
}

/// Convert DbError to ApiError
impl From<DbError> for ApiError {
    fn from(err: DbError) -> Self {
        ApiError::database(err.to_string())
    }
}

/// Validation error type
#[derive(Debug, Error)]
#[error("Validation error: {0}")]
pub struct ValidationError(pub String);

impl From<ValidationError> for ApiError {
    fn from(err: ValidationError) -> Self {
        ApiError::bad_request(err.0)
    }
}

/// Convert validator errors
impl From<validator::ValidationErrors> for ApiError {
    fn from(err: validator::ValidationErrors) -> Self {
        ApiError::bad_request(format!("Validation failed: {}", err))
    }
}
