//! Category service
//!
//! Business logic for category management including per-category inventory collection management.

use bson::oid::ObjectId;
use std::sync::Arc;

use crate::{
    core::{ServiceError, DbError},
};

use super::{
    domain::{Category, CategoryStatus},
    dto::{
        CategoryOrderUpdate, CategoryResponse, CategoryTreeResponse, CreateCategoryRequest,
        ReorderCategoriesRequest, UpdateCategoryRequest,
    },
    repository::CategoryRepository,
};

/// Category service
pub struct CategoryService {
    repo: Arc<CategoryRepository>,
}

impl CategoryService {
    /// Creates a new category service
    pub fn new(repo: Arc<CategoryRepository>) -> Self {
        Self { repo }
    }

    /// Creates a new category with auto-collection creation
    ///
    /// # Arguments
    /// * `req` - Create category request
    ///
    /// # Returns
    /// * `Result<CategoryResponse, ServiceError>` - Created category
    pub async fn create_category(
        &self,
        req: CreateCategoryRequest,
    ) -> Result<CategoryResponse, ServiceError> {
        // Check if slug already exists
        if self.repo.slug_exists(&req.slug).await? {
            return Err(ServiceError::BadRequest(format!(
                "Category with slug '{}' already exists",
                req.slug
            )));
        }

        // Check if name already exists
        if self.repo.name_exists(&req.name).await? {
            return Err(ServiceError::BadRequest(format!(
                "Category with name '{}' already exists",
                req.name
            )));
        }

        // Parse parent_id if provided
        let parent_id = if let Some(ref parent_id_str) = req.parent_id {
            Some(
                ObjectId::parse_str(parent_id_str)
                    .map_err(|_| ServiceError::BadRequest("Invalid parent ID".to_string()))?,
            )
        } else {
            None
        };

        // Validate parent exists if provided
        if let Some(ref parent) = parent_id {
            if self.repo.find_by_id(parent).await?.is_none() {
                return Err(ServiceError::NotFound("Parent category not found".to_string()));
            }
        }

        // Create category
        let mut category = Category::new(
            req.name.clone(),
            req.slug.clone(),
            req.commission_rate,
            parent_id,
        );
        category.icon = req.icon;
        category.description = req.description;
        category.sort_order = req.sort_order.unwrap_or(0);

        // Insert into database
        let created = self.repo.create(category).await?;

        // Create inventory collection
        let collection_name = created.inventory_collection_name();
        self.repo
            .create_inventory_collection(&collection_name)
            .await
            .map_err(|e| ServiceError::InternalError(format!("Failed to create inventory collection: {}", e)))?;

        tracing::info!(
            id = %created.id.unwrap().to_hex(),
            name = %created.name,
            slug = %created.slug,
            collection = %collection_name,
            "Category created with inventory collection"
        );

        Ok(CategoryResponse::from(created))
    }

    /// Gets a category by ID
    ///
    /// # Arguments
    /// * `id` - Category ID string
    ///
    /// # Returns
    /// * `Result<CategoryResponse, ServiceError>` - Category details
    pub async fn get_category(&self, id: &str) -> Result<CategoryResponse, ServiceError> {
        let oid = ObjectId::parse_str(id)
            .map_err(|_| ServiceError::BadRequest("Invalid category ID".to_string()))?;

        let category = self
            .repo
            .find_by_id(&oid)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Category not found".to_string()))?;

        Ok(CategoryResponse::from(category))
    }

    /// Lists categories as a tree structure
    ///
    /// # Arguments
    /// * `include_inactive` - Whether to include inactive categories
    ///
    /// # Returns
    /// * `Result<Vec<CategoryTreeResponse>, ServiceError>` - Category tree
    pub async fn list_categories_tree(
        &self,
        include_inactive: bool,
    ) -> Result<Vec<CategoryTreeResponse>, ServiceError> {
        let categories = self.repo.list_all(include_inactive).await?;

        // Build tree structure (only root categories at top level)
        let mut root_categories: Vec<(CategoryTreeResponse, i32)> = categories
            .iter()
            .filter(|c| c.parent_id.is_none())
            .map(|c| (self.build_tree_node(c, &categories), c.sort_order))
            .collect();

        // Sort by sort_order
        root_categories.sort_by_key(|(_, order)| *order);

        Ok(root_categories.into_iter().map(|(node, _)| node).collect())
    }

    /// Updates a category
    ///
    /// # Arguments
    /// * `id` - Category ID string
    /// * `req` - Update category request
    ///
    /// # Returns
    /// * `Result<CategoryResponse, ServiceError>` - Updated category
    pub async fn update_category(
        &self,
        id: &str,
        req: UpdateCategoryRequest,
    ) -> Result<CategoryResponse, ServiceError> {
        let oid = ObjectId::parse_str(id)
            .map_err(|_| ServiceError::BadRequest("Invalid category ID".to_string()))?;

        let mut category = self
            .repo
            .find_by_id(&oid)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Category not found".to_string()))?;

        let old_slug = category.slug.clone();

        // Check if slug changed
        if let Some(new_slug) = req.slug {
            if new_slug != old_slug {
                // Validate new slug is unique
                if self.repo.slug_exists(&new_slug).await? {
                    return Err(ServiceError::BadRequest(
                        "Category with new slug already exists".to_string(),
                    ));
                }

                // Rename inventory collection
                let old_collection = format!("inventory_{}", old_slug);
                let new_collection = format!("inventory_{}", new_slug);

                self.repo
                    .rename_inventory_collection(&old_collection, &new_collection)
                    .await
                    .map_err(|e| {
                        ServiceError::InternalError(format!("Failed to rename collection: {}", e))
                    })?;

                tracing::info!(
                    old = %old_collection,
                    new = %new_collection,
                    "Renamed inventory collection"
                );

                category.slug = new_slug;
            }
        }

        // Update other fields
        if let Some(name) = req.name {
            category.name = name;
        }
        if let Some(parent_id_str) = req.parent_id {
            let parent_oid = ObjectId::parse_str(&parent_id_str)
                .map_err(|_| ServiceError::BadRequest("Invalid parent ID".to_string()))?;
            category.parent_id = Some(parent_oid);
        }
        if let Some(commission_rate) = req.commission_rate {
            category.commission_rate = commission_rate;
        }
        if let Some(icon) = req.icon {
            category.icon = Some(icon);
        }
        if let Some(description) = req.description {
            category.description = Some(description);
        }
        if let Some(sort_order) = req.sort_order {
            category.sort_order = sort_order;
        }

        category.updated_at = bson::DateTime::now();

        self.repo.update(&oid, &category).await?;

        Ok(CategoryResponse::from(category))
    }

    /// Soft deletes a category
    ///
    /// # Arguments
    /// * `id` - Category ID string
    ///
    /// # Returns
    /// * `Result<(), ServiceError>` - Success
    pub async fn delete_category(&self, id: &str) -> Result<(), ServiceError> {
        let oid = ObjectId::parse_str(id)
            .map_err(|_| ServiceError::BadRequest("Invalid category ID".to_string()))?;

        // Check for child categories
        let child_count = self.repo.count_children(&oid).await?;
        if child_count > 0 {
            return Err(ServiceError::BadRequest(format!(
                "Cannot delete: {} child categories exist. Move or delete them first.",
                child_count
            )));
        }

        // TODO: Check for products in this category's inventory collection
        // For now, we'll soft delete and keep the collection

        self.repo.soft_delete(&oid).await?;

        tracing::info!(
            id = %id,
            "Category soft deleted (collection kept for 30 days)"
        );

        Ok(())
    }

    /// Reorders categories
    ///
    /// # Arguments
    /// * `req` - Reorder categories request
    ///
    /// # Returns
    /// * `Result<(), ServiceError>` - Success
    pub async fn reorder_categories(
        &self,
        req: ReorderCategoriesRequest,
    ) -> Result<(), ServiceError> {
        for update in req.updates {
            let oid = ObjectId::parse_str(&update.id)
                .map_err(|_| ServiceError::BadRequest(format!("Invalid category ID: {}", update.id)))?;

            self.repo.update_sort_order(&oid, update.sort_order).await?;
        }

        tracing::info!("Categories reordered");

        Ok(())
    }

    /// Builds a tree node from a category (recursive)
    fn build_tree_node(&self, category: &Category, all_categories: &[Category]) -> CategoryTreeResponse {
        let children: Vec<CategoryTreeResponse> = all_categories
            .iter()
            .filter(|c| c.parent_id.as_ref() == category.id.as_ref())
            .map(|c| self.build_tree_node(c, all_categories))
            .collect();

        CategoryTreeResponse {
            id: category.id.unwrap().to_hex(),
            name: category.name.clone(),
            slug: category.slug.clone(),
            icon: category.icon.clone(),
            description: category.description.clone(),
            product_count: 0, // TODO: Implement product count
            children,
        }
    }
}
