//! Standard API response structures
//!
//! This module provides consistent response formats for all API endpoints.
//! All responses follow a unified structure for better client-side handling.

use serde::Serialize;

/// Standard API response wrapper
///
/// # Type Parameters
/// * `T` - The data type to be returned in successful responses
///
/// # Examples
/// ```
/// // Success response
/// let response = ApiResponse::success(user);
///
/// // Error response
/// let response = ApiResponse::<()>::error("User not found");
/// ```
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T> {
    /// Indicates if the request was successful
    pub success: bool,

    /// Response message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Response data (only present on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,

    /// Error details (only present on error)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<serde_json::Value>,
}

impl<T: Serialize> ApiResponse<T> {
    /// Creates a successful response with data
    ///
    /// # Arguments
    /// * `data` - The data to return
    ///
    /// # Examples
    /// ```
    /// let user = User { id: 1, name: "John" };
    /// let response = ApiResponse::success(user);
    /// ```
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            message: None,
            data: Some(data),
            error: None,
        }
    }

    /// Creates a successful response with data and custom message
    ///
    /// # Arguments
    /// * `data` - The data to return
    /// * `message` - Success message
    pub fn success_with_message(data: T, message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: Some(message.into()),
            data: Some(data),
            error: None,
        }
    }
}

impl<T> ApiResponse<T> {
    /// Creates an error response with message
    ///
    /// # Arguments
    /// * `message` - Error message
    ///
    /// # Examples
    /// ```
    /// let response = ApiResponse::<()>::error("Invalid credentials");
    /// ```
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: Some(message.into()),
            data: None,
            error: None,
        }
    }

    /// Creates an error response with detailed error information
    ///
    /// # Arguments
    /// * `message` - Error message
    /// * `details` - Additional error details
    pub fn error_detailed(
        message: impl Into<String>,
        details: Option<impl Serialize>,
    ) -> Self {
        Self {
            success: false,
            message: Some(message.into()),
            data: None,
            error: details.and_then(|d| serde_json::to_value(d).ok()),
        }
    }
}

/// Paginated response wrapper
///
/// Used for list endpoints that support pagination
///
/// # Examples
/// ```
/// let users = vec![user1, user2, user3];
/// let response = PaginatedResponse::new(users, 1, 20, 100);
/// ```
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedResponse<T> {
    /// Array of items
    pub items: Vec<T>,

    /// Pagination metadata
    pub pagination: PaginationMeta,
}

/// Pagination metadata
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationMeta {
    /// Current page number (1-indexed)
    pub current_page: u32,

    /// Number of items per page
    pub page_size: u32,

    /// Total number of items
    pub total_items: u64,

    /// Total number of pages
    pub total_pages: u32,

    /// Whether there's a next page
    pub has_next: bool,

    /// Whether there's a previous page
    pub has_prev: bool,
}

impl<T> PaginatedResponse<T> {
    /// Creates a new paginated response
    ///
    /// # Arguments
    /// * `items` - The items for the current page
    /// * `current_page` - Current page number (1-indexed)
    /// * `page_size` - Number of items per page
    /// * `total_items` - Total number of items across all pages
    ///
    /// # Examples
    /// ```
    /// let users = vec![user1, user2];
    /// let response = PaginatedResponse::new(users, 1, 20, 45);
    /// ```
    pub fn new(items: Vec<T>, current_page: u32, page_size: u32, total_items: u64) -> Self {
        let total_pages = ((total_items as f64) / (page_size as f64)).ceil() as u32;

        Self {
            items,
            pagination: PaginationMeta {
                current_page,
                page_size,
                total_items,
                total_pages,
                has_next: current_page < total_pages,
                has_prev: current_page > 1,
            },
        }
    }
}

/// Success message response (no data)
///
/// Used for operations that don't return data (e.g., DELETE)
#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message: String,
}

impl MessageResponse {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}
