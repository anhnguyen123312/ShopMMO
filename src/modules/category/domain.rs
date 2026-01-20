//! Category domain models
//!
//! Contains MongoDB document structures for categories.

use bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Category document in MongoDB
///
/// Categories organize products into hierarchical groups.
/// Each category has its own inventory collection (inventory_{slug}).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    /// MongoDB ObjectId
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// Category name (3-50 chars, unique)
    pub name: String,

    /// URL-friendly slug (3-50 chars, unique, lowercase, hyphens only)
    pub slug: String,

    /// Parent category ID for hierarchical structure
    pub parent_id: Option<ObjectId>,

    /// Commission rate for this category (0-100%)
    pub commission_rate: f64,

    /// Icon (emoji or icon name)
    pub icon: Option<String>,

    /// Category description
    pub description: Option<String>,

    /// Display order (lower = first)
    pub sort_order: i32,

    /// Category status
    pub status: CategoryStatus,

    /// Created timestamp
    pub created_at: DateTime,

    /// Updated timestamp
    pub updated_at: DateTime,

    /// Soft delete timestamp
    pub deleted_at: Option<DateTime>,
}

/// Category status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum CategoryStatus {
    /// Category is active and visible
    #[default]
    Active,

    /// Category has been soft deleted
    Deleted,
}


impl Category {
    /// Creates a new category
    ///
    /// # Arguments
    /// * `name` - Category name
    /// * `slug` - URL-friendly slug
    /// * `commission_rate` - Commission rate (0-100)
    /// * `parent_id` - Optional parent category ID
    pub fn new(
        name: String,
        slug: String,
        commission_rate: f64,
        parent_id: Option<ObjectId>,
    ) -> Self {
        let now = DateTime::now();
        Self {
            id: None,
            name,
            slug,
            parent_id,
            commission_rate,
            icon: None,
            description: None,
            sort_order: 0,
            status: CategoryStatus::Active,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }

    /// Checks if category is active
    pub fn is_active(&self) -> bool {
        self.status == CategoryStatus::Active
    }

    /// Gets the inventory collection name for this category
    ///
    /// Returns `inventory_{slug}` format
    pub fn inventory_collection_name(&self) -> String {
        format!("inventory_{}", self.slug)
    }

    /// Marks category as deleted (soft delete)
    pub fn mark_deleted(&mut self) {
        self.status = CategoryStatus::Deleted;
        self.deleted_at = Some(DateTime::now());
        self.updated_at = DateTime::now();
    }
}
