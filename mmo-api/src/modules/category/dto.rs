//! Category DTOs (Data Transfer Objects)
//!
//! Request and response structures for category endpoints.

use serde::{Deserialize, Serialize};
use validator::Validate;
use utoipa::ToSchema;

use super::domain::CategoryStatus;

/// Create category request (admin only)
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateCategoryRequest {
    /// Category name (3-50 chars)
    #[validate(length(min = 3, max = 50, message = "Name must be 3-50 characters"))]
    #[schema(example = "Streaming", min_length = 3, max_length = 50)]
    pub name: String,

    /// URL-friendly slug (3-50 chars, lowercase, hyphens only)
    #[validate(length(min = 3, max = 50, message = "Slug must be 3-50 characters"))]
    #[schema(example = "streaming", min_length = 3, max_length = 50)]
    pub slug: String,

    /// Parent category ID (optional, for hierarchy)
    #[schema(example = "507f1f77bcf86cd799439011")]
    pub parent_id: Option<String>,

    /// Commission rate (0-100%)
    #[validate(range(min = 0.0, max = 100.0, message = "Commission rate must be 0-100"))]
    #[schema(example = 10.0, minimum = 0.0, maximum = 100.0)]
    pub commission_rate: f64,

    /// Icon (emoji or icon name)
    #[validate(length(max = 50, message = "Icon too long"))]
    #[schema(example = "📺")]
    pub icon: Option<String>,

    /// Category description
    #[validate(length(max = 500, message = "Description too long"))]
    #[schema(example = "Streaming services like Netflix, Spotify, Disney+", max_length = 500)]
    pub description: Option<String>,

    /// Display order
    #[schema(example = 1)]
    pub sort_order: Option<i32>,
}

/// Update category request (admin only)
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCategoryRequest {
    /// Category name (3-50 chars)
    #[validate(length(min = 3, max = 50, message = "Name must be 3-50 characters"))]
    pub name: Option<String>,

    /// URL-friendly slug (3-50 chars, lowercase, hyphens only)
    #[validate(length(min = 3, max = 50, message = "Slug must be 3-50 characters"))]
    pub slug: Option<String>,

    /// Parent category ID
    pub parent_id: Option<String>,

    /// Commission rate (0-100%)
    #[validate(range(min = 0.0, max = 100.0))]
    pub commission_rate: Option<f64>,

    /// Icon (emoji or icon name)
    #[validate(length(max = 50))]
    pub icon: Option<String>,

    /// Category description
    #[validate(length(max = 500))]
    pub description: Option<String>,

    /// Display order
    pub sort_order: Option<i32>,
}

/// Category response
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CategoryResponse {
    /// Category ID
    pub id: String,

    /// Category name
    pub name: String,

    /// URL-friendly slug
    pub slug: String,

    /// Parent category ID
    pub parent_id: Option<String>,

    /// Commission rate
    pub commission_rate: f64,

    /// Icon
    pub icon: Option<String>,

    /// Description
    pub description: Option<String>,

    /// Display order
    pub sort_order: i32,

    /// Status
    pub status: String,

    /// Inventory collection name
    pub inventory_collection: String,

    /// Created at
    pub created_at: String,

    /// Updated at
    pub updated_at: String,
}

impl From<crate::modules::category::domain::Category> for CategoryResponse {
    fn from(category: crate::modules::category::domain::Category) -> Self {
        let inventory_collection = category.inventory_collection_name();
        Self {
            id: category.id.unwrap().to_hex(),
            name: category.name,
            slug: category.slug,
            parent_id: category.parent_id.map(|id| id.to_hex()),
            commission_rate: category.commission_rate,
            icon: category.icon,
            description: category.description,
            sort_order: category.sort_order,
            status: format!("{:?}", category.status).to_lowercase(),
            inventory_collection,
            created_at: category.created_at.to_chrono().to_rfc3339(),
            updated_at: category.updated_at.to_chrono().to_rfc3339(),
        }
    }
}

/// Category tree node (recursive structure for hierarchy)
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CategoryTreeResponse {
    /// Category ID
    pub id: String,

    /// Category name
    pub name: String,

    /// URL-friendly slug
    pub slug: String,

    /// Icon
    pub icon: Option<String>,

    /// Description
    pub description: Option<String>,

    /// Product count in this category
    pub product_count: i32,

    /// Child categories (references parent to avoid infinite recursion in schema)
    #[serde(default)]
    #[schema(inline)]
    pub children: Vec<CategoryTreeResponse>,
}

/// Reorder categories request (admin only)
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReorderCategoriesRequest {
    /// Category updates with new sort order
    #[validate(length(min = 1, message = "At least one category required"))]
    pub updates: Vec<CategoryOrderUpdate>,
}

/// Single category order update
#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CategoryOrderUpdate {
    /// Category ID
    #[validate(length(min = 1))]
    pub id: String,

    /// New sort order
    pub sort_order: i32,
}

/// Category list query parameters
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CategoryListQuery {
    /// Include inactive categories (admin only)
    #[serde(default)]
    pub include_inactive: bool,

    /// Parent category filter (null for root categories)
    pub parent_id: Option<String>,
}
