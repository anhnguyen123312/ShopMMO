//! Shop Service - P2PMMO V2
//!
//! Business logic for Shop module
//! Flow: Create Shop → Telegram Verify → Complete

use bson::DateTime as BsonDateTime;
use std::sync::Arc;
use ulid::Ulid;
use uuid::Uuid;
use serde_json::json;

use crate::core::error::ServiceError;
use crate::database::RedisDB;
use super::{repository::ShopRepository, domain::{Shop, ShopCompletionStatus}};
use super::dto::*;

/// Shop service
#[derive(Clone)]
pub struct ShopService {
    repo: Arc<ShopRepository>,
    redis: Arc<RedisDB>,
}

impl ShopService {
    pub fn new(repo: Arc<ShopRepository>, redis: Arc<RedisDB>) -> Self {
        Self { repo, redis }
    }

    // ========================================================================
    // CREATE SHOP (4-step Wizard combined)
    // ========================================================================

    /// Create new shop for vendor
    /// Flow according to V2: Auto approve, telegram verification required
    pub async fn create_shop(
        &self,
        vendor_id: String,
        req: CreateShopRequest,
    ) -> Result<CreateShopResponse, ServiceError> {
        // 1. Check if user already has a shop
        if let Some(_) = self.repo.find_by_vendor_id(&vendor_id).await? {
            return Err(ServiceError::BadRequest(
                "User already has a shop".to_string(),
            ));
        }

        // 2. Generate shop slug from name
        let shop_slug = Self::generate_slug(&req.shop_name);

        // 3. Check if slug exists
        if self.repo.slug_exists(&shop_slug, None).await? {
            return Err(ServiceError::BadRequest(
                "Shop slug already exists".to_string(),
            ));
        }

        // 4. Generate shop ID and verification code
        let shop_id = Self::generate_id("SHOP");
        let telegram_verification_code = Uuid::new_v4().to_string();

        // 5. Generate storage path
        let storage_path = format!("/storage/shops/{}/", shop_id);

        // 6. Normalize telegram username
        let telegram_username = if req.telegram_username.starts_with('@') {
            req.telegram_username
        } else {
            format!("@{}", req.telegram_username)
        };

        // 7. Create shop
        let shop = Shop::new(
            shop_id.clone(),
            vendor_id.clone(),
            req.shop_name.clone(),
            shop_slug.clone(),
            req.shop_description.clone(),
            req.shop_logo.clone(),
            telegram_username.clone(),
            telegram_verification_code.clone(),
            storage_path,
        );

        // 8. Add policies if provided
        let mut shop_with_policies = shop;
        if req.warranty_policy.is_some()
            || req.refund_policy.is_some()
            || req.support_hours.is_some()
        {
            shop_with_policies.update_policies(
                req.warranty_policy,
                req.refund_policy,
                req.support_hours,
            );
        }

        // 9. Save to database
        let created = self.repo.create(shop_with_policies).await?;

        // 10. Store verification code in Redis (24 hours TTL)
        let verification_data = json!({
            "shop_id": &shop_id,
            "code": &telegram_verification_code,
            "created_at": BsonDateTime::now().timestamp_millis(),
            "expires_at": BsonDateTime::now().timestamp_millis() + (24 * 60 * 60 * 1000)
        });

        // Store by shop_id and by code (for reverse lookup)
        let shop_key = crate::database::redis::keys::telegram_verify(&shop_id);
        let code_key = crate::database::redis::keys::telegram_code(&telegram_verification_code);

        self.redis
            .set(&shop_key, &verification_data.to_string(), Some(24 * 60 * 60))
            .await
            .map_err(|e| ServiceError::InternalError(format!("Redis error: {}", e)))?;

        self.redis
            .set(&code_key, &shop_id, Some(24 * 60 * 60))
            .await
            .map_err(|e| ServiceError::InternalError(format!("Redis error: {}", e)))?;

        // 11. TODO: Create storage directory
        // fs::create_dir_all(&format!("{}products", storage_path))?;
        // fs::create_dir_all(&format!("{}banners", storage_path))?;

        // 12. TODO: Update user role to add "vendor"
        // user_repo.add_role(vendor_id, "vendor").await?;

        Ok(CreateShopResponse {
            shop_id: created.shop_id.clone(),
            vendor_id,
            shop_name: created.shop_name,
            shop_slug: created.shop_slug,
            telegram_verification_code: telegram_verification_code.clone(),
            telegram_instruction: format!(
                "Send /start {} to @p2pmmo bot to verify your Telegram account",
                telegram_verification_code
            ),
            telegram_verified: false,
            status: created.status,
            storage_path: created.storage_path,
            created_at: created.created_at.to_string(),
        })
    }

    // ========================================================================
    // TELEGRAM VERIFICATION
    // ========================================================================

    /// Verify Telegram account (internal - called by bot)
    pub async fn verify_telegram(
        &self,
        verification_code: &str,
        chat_id: String,
        username: Option<String>,
    ) -> Result<TelegramVerifyResponse, ServiceError> {
        // Get shop_id from Redis using verification code
        let code_key = crate::database::redis::keys::telegram_code(verification_code);

        let shop_id = self
            .redis
            .get(&code_key)
            .await
            .map_err(|e| ServiceError::InternalError(format!("Redis error: {}", e)))?
            .ok_or_else(|| {
                ServiceError::NotFound("Invalid or expired verification code".to_string())
            })?;

        // Fetch shop
        let shop = self
            .repo
            .find_by_id(&shop_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Shop not found".to_string()))?;

        // Check if already verified
        if shop.telegram_verified {
            return Ok(TelegramVerifyResponse {
                shop_id: shop.shop_id.clone(),
                shop_name: shop.shop_name.clone(),
                success: true,
                username_mismatch_warning: None,
                verified_at: shop.telegram_verified_at.map(|dt| dt.to_string()),
                completion_status: ShopCompletionStatus::from_shop(&shop),
            });
        }

        // Soft check: compare username (warning if mismatch)
        let username_mismatch_warning = if let Some(ref tg_username) = username {
            if shop.telegram_username != *tg_username && shop.telegram_username != format!("@{}", tg_username) {
                Some(format!(
                    "Warning: Telegram username mismatch. Expected: {}, Got: {}",
                    shop.telegram_username, tg_username
                ))
            } else {
                None
            }
        } else {
            None
        };

        // Update shop
        let mut updated_shop = shop.clone();
        updated_shop.verify_telegram(chat_id);

        let saved = self.repo.update(&updated_shop).await?;

        // Delete verification codes from Redis
        let shop_key = crate::database::redis::keys::telegram_verify(&shop_id);

        self.redis
            .delete(&shop_key)
            .await
            .map_err(|e| ServiceError::InternalError(format!("Redis error: {}", e)))?;

        self.redis
            .delete(&code_key)
            .await
            .map_err(|e| ServiceError::InternalError(format!("Redis error: {}", e)))?;

        // TODO: Send test notification via Telegram
        // telegram_bot.send_notification(chat_id, "✅ Verified! You will receive notifications...").await?;

        Ok(TelegramVerifyResponse {
            shop_id: saved.shop_id.clone(),
            shop_name: saved.shop_name.clone(),
            success: true,
            username_mismatch_warning,
            verified_at: saved.telegram_verified_at.map(|dt| dt.to_string()),
            completion_status: ShopCompletionStatus::from_shop(&saved),
        })
    }

    // ========================================================================
    // GET SHOP
    // ========================================================================

    /// Get shop by ID
    pub async fn get_shop(&self, shop_id: &str) -> Result<ShopDetailResponse, ServiceError> {
        let shop = self
            .repo
            .find_by_id(shop_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Shop not found".to_string()))?;

        Ok(ShopDetailResponse::from(shop))
    }

    /// Get shop by slug (public)
    pub async fn get_shop_by_slug(&self, slug: &str) -> Result<ShopDetailResponse, ServiceError> {
        let shop = self
            .repo
            .find_by_slug(slug)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Shop not found".to_string()))?;

        Ok(ShopDetailResponse::from(shop))
    }

    /// Get shop dashboard (vendor only)
    pub async fn get_shop_dashboard(
        &self,
        vendor_id: &str,
    ) -> Result<ShopDashboardResponse, ServiceError> {
        let shop = self
            .repo
            .find_by_vendor_id(vendor_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Shop not found".to_string()))?;

        Ok(ShopDashboardResponse::from(shop))
    }

    /// Get shop verification info
    pub async fn get_verification_info(
        &self,
        vendor_id: &str,
    ) -> Result<ShopVerificationResponse, ServiceError> {
        let shop = self
            .repo
            .find_by_vendor_id(vendor_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Shop not found".to_string()))?;

        let verification_code = if !shop.telegram_verified {
            shop.telegram_verification_code.clone()
        } else {
            None
        };

        // Calculate code expiry (24h from shop creation)
        let code_expires_at = if !shop.telegram_verified {
            Some(
                BsonDateTime::from_millis(
                    shop.created_at.timestamp_millis() + (24 * 60 * 60 * 1000)
                ).to_string()
            )
        } else {
            None
        };

        let instruction = if shop.telegram_verified {
            "Your Telegram account has been verified.".to_string()
        } else {
            format!(
                "Send /start {} to @p2pmmo bot to verify your Telegram account",
                shop.telegram_verification_code.as_ref().unwrap_or(&"N/A".to_string())
            )
        };

        Ok(ShopVerificationResponse {
            shop_id: shop.shop_id,
            shop_name: shop.shop_name,
            telegram_username: shop.telegram_username,
            telegram_verified: shop.telegram_verified,
            verification_code,
            code_expires_at,
            instruction,
            verified_at: shop.telegram_verified_at.map(|dt| dt.to_string()),
        })
    }

    // ========================================================================
    // UPDATE SHOP
    // ========================================================================

    /// Update shop basic info
    pub async fn update_shop(
        &self,
        shop_id: &str,
        vendor_id: &str,
        req: UpdateShopRequest,
    ) -> Result<UpdateShopResponse, ServiceError> {
        // Get shop
        let mut shop = self
            .repo
            .find_by_id(shop_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Shop not found".to_string()))?;

        // Verify ownership
        if shop.vendor_id != vendor_id {
            return Err(ServiceError::Forbidden("Not your shop".to_string()));
        }

        // Track updated fields
        let mut updated_fields = Vec::new();

        // Update fields
        if let Some(name) = req.shop_name {
            shop.shop_name = name.clone();
            updated_fields.push("shop_name".to_string());
            // Note: slug is not updated to avoid breaking links
        }

        if let Some(description) = req.shop_description {
            shop.shop_description = description;
            updated_fields.push("shop_description".to_string());
        }

        if let Some(logo) = req.shop_logo {
            shop.shop_logo = logo;
            updated_fields.push("shop_logo".to_string());
        }

        if let Some(banner) = req.shop_banner {
            shop.shop_banner = Some(banner);
            updated_fields.push("shop_banner".to_string());
        }

        shop.updated_at = BsonDateTime::now();

        // Save
        let updated = self.repo.update(&shop).await?;

        Ok(UpdateShopResponse {
            shop_id: updated.shop_id,
            updated_fields,
            updated_at: updated.updated_at.to_string(),
        })
    }

    /// Update shop policies
    pub async fn update_policies(
        &self,
        shop_id: &str,
        vendor_id: &str,
        req: UpdateShopPoliciesRequest,
    ) -> Result<UpdateShopResponse, ServiceError> {
        // Get shop
        let mut shop = self
            .repo
            .find_by_id(shop_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Shop not found".to_string()))?;

        // Verify ownership
        if shop.vendor_id != vendor_id {
            return Err(ServiceError::Forbidden("Not your shop".to_string()));
        }

        // Update policies
        shop.update_policies(
            req.warranty_policy,
            req.refund_policy,
            req.support_hours,
        );

        // Save
        let updated = self.repo.update(&shop).await?;

        // Check if shop is now complete
        let updated_fields = vec
!["policies".to_string()
];

        Ok(UpdateShopResponse {
            shop_id: updated.shop_id,
            updated_fields,
            updated_at: updated.updated_at.to_string(),
        })
    }

    // ========================================================================
    // LIST SHOPS
    // ========================================================================

    /// List shops with filters
    pub async fn list_shops(
        &self,
        query: ShopListQuery,
    ) -> Result<ShopListResponse, ServiceError> {
        let skip = (query.page - 1) * query.per_page;

        // Build filter
        let mut filter = bson::doc! {};

        if let Some(search) = &query.search {
            filter.insert(
                "$or",
                bson::doc! {
                    "shop_name": bson::Regex { pattern: search.to_string(), options: "i".to_string() },
                    "shop_slug": bson::Regex { pattern: search.to_string(), options: "i".to_string() }
                },
            );
        }

        if let Some(status) = &query.status {
            filter.insert("status", status);
        }

        if let Some(level) = &query.level {
            filter.insert("level", level);
        }

        if let Some(vendor_id) = &query.vendor_id {
            filter.insert("vendor_id", vendor_id);
        }

        // Build sort
        let sort_order = if query.sort_order == "asc" { 1 } else { -1 };
        let sort = bson::doc! { &query.sort_by: sort_order };

        // Get total count
        let total = self.repo.count(Some(filter.clone())).await?;

        // Get shops
        let shops = self
            .repo
            .list(Some(filter), Some(sort), skip as u64, query.per_page)
            .await?;

        let total_pages = (total as f64 / query.per_page as f64).ceil() as i64;

        Ok(ShopListResponse {
            shops: shops.into_iter().map(ShopResponse::from).collect(),
            total,
            page: query.page,
            per_page: query.per_page,
            total_pages,
        })
    }

    /// Search shops
    pub async fn search_shops(
        &self,
        search_term: &str,
        page: i64,
        per_page: i64,
    ) -> Result<ShopListResponse, ServiceError> {
        let skip = (page - 1) * per_page;

        let shops = self
            .repo
            .search(search_term, skip as u64, per_page)
            .await?;

        let total = shops.len() as i64; // Simplified

        Ok(ShopListResponse {
            shops: shops.into_iter().map(ShopResponse::from).collect(),
            total,
            page,
            per_page,
            total_pages: 1,
        })
    }

    // ========================================================================
    // SHOP STATS
    // ========================================================================

    /// Get shop statistics (admin)
    pub async fn get_stats(&self) -> Result<ShopStatsResponse, ServiceError> {
        let total_shops = self.repo.count_total().await?;
        let active_shops = self.repo.count_active().await?;
        let complete_shops = self.repo.count_complete().await?;
        let telegram_verified_shops = self.repo.count_telegram_verified().await?;
        let new_shops_today = self.repo.count_new_today().await?;

        let by_level = self.repo.count_by_level().await?;

        let top_shops = self
            .repo
            .get_top_shops(10)
            .await?
            .into_iter()
            .map(|s| TopShopStats {
                shop_id: s.shop_id,
                shop_name: s.shop_name,
                total_sales: s.total_sales,
                total_revenue: s.total_revenue,
                avg_rating: s.avg_rating,
            })
            .collect();

        Ok(ShopStatsResponse {
            total_shops,
            active_shops,
            new_shops_today,
            complete_shops,
            telegram_verified_shops,
            by_level: crate::modules::shop::dto::ShopLevelStats {
                new: by_level.new,
                silver: by_level.silver,
                gold: by_level.gold,
                diamond: by_level.diamond,
                partner: by_level.partner,
            },
            top_shops,
        })
    }

    // ========================================================================
    // PRODUCT INTEGRATION (called by product module)
    // ========================================================================

    /// Increment product count (called when products are added)
    pub async fn increment_products(
        &self,
        shop_id: &str,
        count: i64,
    ) -> Result<(), ServiceError> {
        let mut shop = self
            .repo
            .find_by_id(shop_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Shop not found".to_string()))?;

        shop.increment_products(count);
        self.repo.update(&shop).await?;

        Ok(())
    }

    /// Add sale (called when order is completed)
    pub async fn add_sale(
        &self,
        shop_id: &str,
        amount: i64,
    ) -> Result<(), ServiceError> {
        let mut shop = self
            .repo
            .find_by_id(shop_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Shop not found".to_string()))?;

        shop.add_sale(amount);
        self.repo.update(&shop).await?;

        Ok(())
    }

    /// Update rating (called when review is created)
    pub async fn update_rating(
        &self,
        shop_id: &str,
        rating: f64,
    ) -> Result<(), ServiceError> {
        let mut shop = self
            .repo
            .find_by_id(shop_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Shop not found".to_string()))?;

        shop.update_rating(rating);
        self.repo.update(&shop).await?;

        Ok(())
    }

    /// Increment active disputes
    pub async fn increment_disputes(
        &self,
        shop_id: &str,
    ) -> Result<(), ServiceError> {
        let mut shop = self
            .repo
            .find_by_id(shop_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Shop not found".to_string()))?;

        shop.increment_disputes();
        self.repo.update(&shop).await?;

        Ok(())
    }

    /// Decrement active disputes
    pub async fn decrement_disputes(
        &self,
        shop_id: &str,
    ) -> Result<(), ServiceError> {
        let mut shop = self
            .repo
            .find_by_id(shop_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Shop not found".to_string()))?;

        shop.decrement_disputes();
        self.repo.update(&shop).await?;

        Ok(())
    }

    // ========================================================================
    // HELPER FUNCTIONS
    // ========================================================================

    /// Generate ULID-based ID
    fn generate_id(prefix: &str) -> String {
        format!("{}-{}", prefix, Ulid::new().to_string().to_lowercase())
    }

    /// Generate URL-friendly slug from shop name
    fn generate_slug(name: &str) -> String {
        // Convert to lowercase and replace spaces with hyphens
        // Remove special characters
        let slug = name
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>();

        // Remove consecutive hyphens and trim
        slug.split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>()
            .join("-")
    }
}
