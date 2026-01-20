//! Shop Repository - P2PMMO V2
//!
//! MongoDB database operations for Shop module

use bson::{doc, Document};
use futures::stream::TryStreamExt;
use mongodb::Collection;
use std::sync::Arc;

use crate::core::error::DbError;
use crate::database::MongoDB;
use super::domain::Shop;

/// Shop repository - handles all database operations
#[derive(Clone)]
pub struct ShopRepository {
    shops: Collection<Shop>,
}

impl ShopRepository {
    pub fn new(db: Arc<MongoDB>) -> Self {
        let database = db.database();
        Self {
            shops: database.collection("shops"),
        }
    }

    // ========================================================================
    // CRUD OPERATIONS
    // ========================================================================

    /// Create new shop
    pub async fn create(&self, shop: Shop) -> Result<Shop, DbError> {
        self.shops.insert_one(&shop).await?;
        Ok(shop)
    }

    /// Find shop by shop_id
    pub async fn find_by_id(&self, shop_id: &str) -> Result<Option<Shop>, DbError> {
        self.shops
            .find_one(doc! { "shop_id": shop_id })
            .await
            .map_err(DbError::from)
    }

    /// Find shop by shop_slug
    pub async fn find_by_slug(&self, slug: &str) -> Result<Option<Shop>, DbError> {
        self.shops
            .find_one(doc! { "shop_slug": slug })
            .await
            .map_err(DbError::from)
    }

    /// Find shop by vendor_id (user can only have 1 shop)
    pub async fn find_by_vendor_id(&self, vendor_id: &str) -> Result<Option<Shop>, DbError> {
        self.shops
            .find_one(doc! { "vendor_id": vendor_id })
            .await
            .map_err(DbError::from)
    }

    /// Update shop
    pub async fn update(&self, shop: &Shop) -> Result<Shop, DbError> {
        let mut updated = shop.clone();
        updated.updated_at = bson::DateTime::now();

        self.shops
            .replace_one(doc! { "shop_id": &shop.shop_id }, &updated)
            .await?;

        Ok(updated)
    }

    /// Delete shop (soft delete - update status)
    pub async fn soft_delete(&self, shop_id: &str) -> Result<(), DbError> {
        
        self.shops
            .update_one(
                doc! { "shop_id": shop_id },
                doc! {
                    "$set": {
                        "status": "INACTIVE",
                        "updated_at": bson::DateTime::now()
                    }
                },
            )
            .await?;
        Ok(())
    }

    // ========================================================================
    // LIST & SEARCH OPERATIONS
    // ========================================================================

    /// List shops with pagination and filters
    pub async fn list(
        &self,
        filter: Option<Document>,
        sort: Option<Document>,
        skip: u64,
        limit: i64,
    ) -> Result<Vec<Shop>, DbError> {
        let query_filter = filter.unwrap_or_else(|| doc! {});
        let query_sort = sort.unwrap_or_else(|| doc! { "created_at": -1 });

        let cursor = self
            .shops
            .find(query_filter)
            .sort(query_sort)
            .skip(skip)
            .limit(limit)
            .await?;

        let shops: Vec<Shop> = cursor.try_collect().await?;
        Ok(shops)
    }

    /// Count shops with filter
    pub async fn count(&self, filter: Option<Document>) -> Result<i64, DbError> {
        let query_filter = filter.unwrap_or_else(|| doc! {});
        let count = self.shops.count_documents(query_filter).await? as i64;
        Ok(count)
    }

    /// Search shops by name or slug
    pub async fn search(
        &self,
        search_term: &str,
        skip: u64,
        limit: i64,
    ) -> Result<Vec<Shop>, DbError> {
        let filter = doc! {
            "$or": [
                { "shop_name": { "$regex": search_term, "$options": "i" } },
                { "shop_slug": { "$regex": search_term, "$options": "i" } }
            ],
            "status": "ACTIVE"
        };

        let cursor = self
            .shops
            .find(filter)
            .sort(doc! { "total_sales": -1 })
            .skip(skip)
            .limit(limit)
            .await?;

        let shops: Vec<Shop> = cursor.try_collect().await?;
        Ok(shops)
    }

    // ========================================================================
    // STATISTICS OPERATIONS
    // ========================================================================

    /// Get total shops count
    pub async fn count_total(&self) -> Result<i64, DbError> {
        self.count(None).await
    }

    /// Count active shops
    pub async fn count_active(&self) -> Result<i64, DbError> {
        let filter = doc! { "status": "ACTIVE" };
        self.count(Some(filter)).await
    }

    /// Count complete shops
    pub async fn count_complete(&self) -> Result<i64, DbError> {
        let filter = doc! { "is_complete": true };
        self.count(Some(filter)).await
    }

    /// Count telegram verified shops
    pub async fn count_telegram_verified(&self) -> Result<i64, DbError> {
        let filter = doc! { "telegram_verified": true };
        self.count(Some(filter)).await
    }

    /// Count shops created today
    pub async fn count_new_today(&self) -> Result<i64, DbError> {
        let now = bson::DateTime::now();
        let today_start = bson::DateTime::from_millis(
            now.timestamp_millis() - (now.timestamp_millis() % (24 * 60 * 60 * 1000))
        );

        let filter = doc! {
            "created_at": { "$gte": today_start }
        };

        self.count(Some(filter)).await
    }

    /// Count shops by level
    pub async fn count_by_level(&self) -> Result<ShopLevelStats, DbError> {
        

        let pipeline = vec![
            doc! {
                "$group": {
                    "_id": "$level",
                    "count": doc! { "$sum": 1 }
                }
            },
        ];

        let mut cursor = self.shops.aggregate(pipeline).await?;
        let mut stats = ShopLevelStats::default();

        while let Some(result) = cursor.try_next().await? {
            if let Ok(level_str) = result.get_str("_id") {
                let count = result.get_i64("count").unwrap_or(0);

                match level_str {
                    "NEW" => stats.new = count,
                    "SILVER" => stats.silver = count,
                    "GOLD" => stats.gold = count,
                    "DIAMOND" => stats.diamond = count,
                    "PARTNER" => stats.partner = count,
                    _ => {}
                }
            }
        }

        Ok(stats)
    }

    /// Get top shops by sales
    pub async fn get_top_shops(&self, limit: i64) -> Result<Vec<TopShopStats>, DbError> {
        let pipeline = vec![
            doc! {
                "$match": { "status": "ACTIVE" }
            },
            doc! {
                "$sort": { "total_sales": -1 }
            },
            doc! {
                "$limit": limit
            },
            doc! {
                "$project": {
                    "shop_id": 1,
                    "shop_name": 1,
                    "total_sales": 1,
                    "total_revenue": 1,
                    "avg_rating": 1
                }
            },
        ];

        let mut cursor = self.shops.aggregate(pipeline).await?;
        let mut top_shops = Vec::new();

        while let Some(doc) = cursor.try_next().await? {
            if let Ok(shop_id) = doc.get_str("shop_id") {
                let shop_name = doc.get_str("shop_name").unwrap_or("");
                let total_sales = doc.get_i64("total_sales").unwrap_or(0);
                let total_revenue = doc.get_i64("total_revenue").unwrap_or(0);
                let avg_rating = doc.get_f64("avg_rating").unwrap_or(0.0);

                top_shops.push(TopShopStats {
                    shop_id: shop_id.to_string(),
                    shop_name: shop_name.to_string(),
                    total_sales,
                    total_revenue,
                    avg_rating,
                });
            }
        }

        Ok(top_shops)
    }

    // ========================================================================
    // SPECIFIC OPERATIONS
    // ========================================================================

    /// Check if slug exists
    pub async fn slug_exists(&self, slug: &str, exclude_shop_id: Option<&str>) -> Result<bool, DbError> {
        let mut filter = doc! { "shop_slug": slug };

        if let Some(exclude_id) = exclude_shop_id {
            filter.insert("shop_id", doc! { "$ne": exclude_id });
        }

        let count = self.shops.count_documents(filter).await?;
        Ok(count > 0)
    }

    /// Update shop stats (increment products, sales, etc.)
    pub async fn increment_products(&self, shop_id: &str, count: i64) -> Result<(), DbError> {
        self.shops
            .update_one(
                doc! { "shop_id": shop_id },
                doc! {
                    "$inc": { "total_products": count },
                    "$set": { "updated_at": bson::DateTime::now() }
                },
            )
            .await?;
        Ok(())
    }

    /// Update shop rating
    pub async fn update_rating(&self, shop_id: &str, _new_rating: f64, increment_review: bool) -> Result<(), DbError> {
        let mut update_doc = doc! {
            "$set": { "updated_at": bson::DateTime::now() }
        };

        if increment_review {
            update_doc.insert("$inc", doc! {
                "total_reviews": 1
            });
        }

        // Note: Average rating calculation should be done in service layer
        // This is just a placeholder for the update
        self.shops
            .update_one(doc! { "shop_id": shop_id }, update_doc)
            .await?;
        Ok(())
    }

    /// Update shop completion status
    pub async fn mark_complete(&self, shop_id: &str) -> Result<(), DbError> {
        self.shops
            .update_one(
                doc! { "shop_id": shop_id },
                doc! {
                    "$set": {
                        "is_complete": true,
                        "completed_at": bson::DateTime::now(),
                        "updated_at": bson::DateTime::now()
                    }
                },
            )
            .await?;
        Ok(())
    }

    /// Increment/decrement active disputes
    pub async fn update_disputes(&self, shop_id: &str, increment: bool) -> Result<(), DbError> {
        let amount = if increment { 1 } else { -1 };

        self.shops
            .update_one(
                doc! { "shop_id": shop_id },
                doc! {
                    "$inc": { "active_disputes": amount },
                    "$set": { "updated_at": bson::DateTime::now() }
                },
            )
            .await?;
        Ok(())
    }
}

// ============================================================================
// HELPER STRUCTS
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct ShopLevelStats {
    pub new: i64,
    pub silver: i64,
    pub gold: i64,
    pub diamond: i64,
    pub partner: i64,
}

#[derive(Debug, Clone)]
pub struct TopShopStats {
    pub shop_id: String,
    pub shop_name: String,
    pub total_sales: i64,
    pub total_revenue: i64,
    pub avg_rating: f64,
}
