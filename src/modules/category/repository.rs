//! Category repository
//!
//! Handles database operations for categories and per-category inventory collections.

use bson::{doc, oid::ObjectId};
use mongodb::{Collection, IndexModel};
use futures::stream::TryStreamExt;
use std::sync::Arc;

use crate::{
    core::DbError,
    database::{mongodb::collections, MongoDB},
};

use super::domain::Category;

/// Category repository
#[derive(Clone)]
pub struct CategoryRepository {
    collection: Collection<Category>,
    db: Arc<MongoDB>,
}

impl CategoryRepository {
    /// Creates a new category repository
    pub fn new(db: Arc<MongoDB>) -> Self {
        let collection = db.database().collection::<Category>(collections::CATEGORIES);
        Self { collection, db }
    }

    /// Creates a new category
    ///
    /// # Arguments
    /// * `category` - Category to create
    ///
    /// # Returns
    /// * `Result<Category, DbError>` - Created category with ID
    pub async fn create(&self, mut category: Category) -> Result<Category, DbError> {
        let result = self.collection.insert_one(&category).await?;
        category.id = Some(result.inserted_id.as_object_id().unwrap());
        Ok(category)
    }

    /// Finds category by ID
    ///
    /// # Arguments
    /// * `id` - Category's ObjectId
    ///
    /// # Returns
    /// * `Result<Option<Category>, DbError>` - Category if found
    pub async fn find_by_id(&self, id: &ObjectId) -> Result<Option<Category>, DbError> {
        self.collection
            .find_one(doc! { "_id": id })
            .await
            .map_err(DbError::from)
    }

    /// Finds category by slug
    ///
    /// # Arguments
    /// * `slug` - Category slug
    ///
    /// # Returns
    /// * `Result<Option<Category>, DbError>` - Category if found
    pub async fn find_by_slug(&self, slug: &str) -> Result<Option<Category>, DbError> {
        self.collection
            .find_one(doc! { "slug": slug })
            .await
            .map_err(DbError::from)
    }

    /// Lists all categories
    ///
    /// # Arguments
    /// * `include_inactive` - Whether to include deleted categories
    ///
    /// # Returns
    /// * `Result<Vec<Category>, DbError>` - List of categories
    pub async fn list_all(&self, include_inactive: bool) -> Result<Vec<Category>, DbError> {
        let filter = if include_inactive {
            doc! {}
        } else {
            doc! { "status": { "$ne": "deleted" } }
        };

        let mut cursor = self.collection.find(filter).await.map_err(DbError::from)?;
        let mut categories = Vec::new();
        while let Some(category) = cursor.try_next().await.map_err(DbError::from)? {
            categories.push(category);
        }
        Ok(categories)
    }

    /// Lists root categories (no parent)
    ///
    /// # Returns
    /// * `Result<Vec<Category>, DbError>` - List of root categories
    pub async fn list_root_categories(&self) -> Result<Vec<Category>, DbError> {
        let mut cursor = self
            .collection
            .find(doc! {
                "parent_id": null,
                "status": { "$ne": "deleted" }
            })
            .await
            .map_err(DbError::from)?;
        let mut categories = Vec::new();
        while let Some(category) = cursor.try_next().await.map_err(DbError::from)? {
            categories.push(category);
        }
        Ok(categories)
    }

    /// Lists child categories of a parent
    ///
    /// # Arguments
    /// * `parent_id` - Parent category ID
    ///
    /// # Returns
    /// * `Result<Vec<Category>, DbError>` - List of child categories
    pub async fn list_by_parent(&self, parent_id: &ObjectId) -> Result<Vec<Category>, DbError> {
        let mut cursor = self
            .collection
            .find(doc! {
                "parent_id": parent_id,
                "status": { "$ne": "deleted" }
            })
            .await
            .map_err(DbError::from)?;
        let mut categories = Vec::new();
        while let Some(category) = cursor.try_next().await.map_err(DbError::from)? {
            categories.push(category);
        }
        Ok(categories)
    }

    /// Updates a category
    ///
    /// # Arguments
    /// * `id` - Category's ObjectId
    /// * `category` - Updated category data
    ///
    /// # Returns
    /// * `Result<(), DbError>` - Success
    pub async fn update(&self, id: &ObjectId, category: &Category) -> Result<(), DbError> {
        self.collection
            .replace_one(doc! { "_id": id }, category)
            .await?;
        Ok(())
    }

    /// Soft deletes a category
    ///
    /// # Arguments
    /// * `id` - Category's ObjectId
    ///
    /// # Returns
    /// * `Result<(), DbError>` - Success
    pub async fn soft_delete(&self, id: &ObjectId) -> Result<(), DbError> {
        self.collection
            .update_one(
                doc! { "_id": id },
                doc! {
                    "$set": {
                        "status": "deleted",
                        "deleted_at": bson::DateTime::now(),
                        "updated_at": bson::DateTime::now()
                    }
                },
            )
            .await?;
        Ok(())
    }

    /// Permanently deletes a category (hard delete)
    ///
    /// # Arguments
    /// * `id` - Category's ObjectId
    ///
    /// # Returns
    /// * `Result<(), DbError>` - Success
    pub async fn hard_delete(&self, id: &ObjectId) -> Result<(), DbError> {
        self.collection.delete_one(doc! { "_id": id }).await?;
        Ok(())
    }

    /// Updates sort order for a category
    ///
    /// # Arguments
    /// * `id` - Category's ObjectId
    /// * `sort_order` - New sort order
    ///
    /// # Returns
    /// * `Result<(), DbError>` - Success
    pub async fn update_sort_order(&self, id: &ObjectId, sort_order: i32) -> Result<(), DbError> {
        self.collection
            .update_one(
                doc! { "_id": id },
                doc! {
                    "$set": {
                        "sort_order": sort_order,
                        "updated_at": bson::DateTime::now()
                    }
                },
            )
            .await?;
        Ok(())
    }

    /// Checks if slug exists
    ///
    /// # Arguments
    /// * `slug` - Slug to check
    ///
    /// # Returns
    /// * `Result<bool, DbError>` - True if exists
    pub async fn slug_exists(&self, slug: &str) -> Result<bool, DbError> {
        let count = self
            .collection
            .count_documents(doc! { "slug": slug })
            .await?;
        Ok(count > 0)
    }

    /// Checks if name exists
    ///
    /// # Arguments
    /// * `name` - Name to check
    ///
    /// # Returns
    /// * `Result<bool, DbError>` - True if exists
    pub async fn name_exists(&self, name: &str) -> Result<bool, DbError> {
        let count = self
            .collection
            .count_documents(doc! { "name": name })
            .await?;
        Ok(count > 0)
    }

    /// Counts child categories
    ///
    /// # Arguments
    /// * `parent_id` - Parent category ID
    ///
    /// # Returns
    /// * `Result<u64, DbError>` - Child count
    pub async fn count_children(&self, parent_id: &ObjectId) -> Result<u64, DbError> {
        self.collection
            .count_documents(doc! {
                "parent_id": parent_id,
                "status": { "$ne": "deleted" }
            })
            .await
            .map_err(DbError::from)
    }

    /// Creates inventory collection for a category
    ///
    /// # Arguments
    /// * `collection_name` - Name of the collection (inventory_{slug})
    ///
    /// # Returns
    /// * `Result<(), DbError>` - Success
    pub async fn create_inventory_collection(&self, collection_name: &str) -> Result<(), DbError> {
        // Create the collection
        self.db
            .database()
            .create_collection(collection_name)
            .await?;

        // Create indexes on the new collection
        let indexes = vec![
            IndexModel::builder().keys(doc! { "product_id": 1 }).build(),
            IndexModel::builder().keys(doc! { "shop_id": 1 }).build(),
            IndexModel::builder().keys(doc! { "is_sold": 1 }).build(),
            IndexModel::builder().keys(doc! { "content": 1 }).build(),
            IndexModel::builder().keys(doc! { "created_at": -1 }).build(),
        ];

        self.db
            .database()
            .collection::<mongodb::bson::Document>(collection_name)
            .create_indexes(indexes)
            .await?;

        tracing::info!(
            collection = %collection_name,
            "Created inventory collection with indexes"
        );

        Ok(())
    }

    /// Renames an inventory collection
    ///
    /// # Arguments
    /// * `old_name` - Current collection name
    /// * `new_name` - New collection name
    ///
    /// # Returns
    /// * `Result<(), DbError>` - Success
    ///
    /// # Note
    /// Collection renaming in MongoDB requires admin database access.
    /// For now, this is a placeholder that logs the rename operation.
    pub async fn rename_inventory_collection(
        &self,
        old_name: &str,
        new_name: &str,
    ) -> Result<(), DbError> {
        // TODO: Implement proper collection rename via admin database or MongoDB commands
        tracing::info!(
            old = %old_name,
            new = %new_name,
            "Collection rename requested (needs manual implementation)"
        );

        // For now, just log - actual rename would require admin database access
        // or running MongoDB commands
        Ok(())
    }

    /// Drops an inventory collection
    ///
    /// # Arguments
    /// * `collection_name` - Collection name to drop
    ///
    /// # Returns
    /// * `Result<(), DbError>` - Success
    pub async fn drop_inventory_collection(&self, collection_name: &str) -> Result<(), DbError> {
        self.db
            .database()
            .collection::<mongodb::bson::Document>(collection_name)
            .drop()
            .await?;

        tracing::info!(collection = %collection_name, "Dropped inventory collection");

        Ok(())
    }

    /// Lists all inventory collections
    ///
    /// # Returns
    /// * `Result<Vec<String>, DbError>` - List of collection names starting with "inventory_"
    pub async fn list_inventory_collections(&self) -> Result<Vec<String>, DbError> {
        // For MongoDB, we should use list_collection_names on the database
        let cursor = self
            .db
            .database()
            .list_collection_names()
            .await?;

        // Filter for inventory collections
        let collections: Vec<String> = cursor
            .into_iter()
            .filter(|name| name.starts_with("inventory_"))
            .collect();

        Ok(collections)
    }
}
