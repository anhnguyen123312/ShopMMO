//! Shop Domain Models - P2PMMO V2
//!
//! MongoDB schema cho Shop với các tính năng V2:
//! - Auto vendor role khi tạo shop
//! - Telegram verification REQUIRED
//! - Shop completion tracking

use bson::{oid::ObjectId, DateTime as BsonDateTime};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ============================================================================
// SHOP MODEL
// ============================================================================

/// Shop document - MongoDB collection: shops
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shop {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// Unique shop identifier: SHOP-{ULID}
    pub shop_id: String,

    /// Vendor/Owner user ID
    pub vendor_id: String,

    // === Basic Info ===
    /// Shop display name (3-50 chars)
    pub shop_name: String,

    /// URL-friendly slug (unique, auto-generated from name)
    pub shop_slug: String,

    /// Shop description (max 500 chars)
    pub shop_description: String,

    // === Branding ===
    /// Logo image URL (REQUIRED)
    pub shop_logo: String,

    /// Banner image URL (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shop_banner: Option<String>,

    // === Telegram (REQUIRED in V2) ===
    /// Telegram username (@format, 11-32 chars)
    pub telegram_username: String,

    /// Telegram chat_id (populated after verification)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub telegram_chat_id: Option<String>,

    /// Telegram verification status
    pub telegram_verified: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub telegram_verified_at: Option<BsonDateTime>,

    /// Generated verification code (UUID, stored in Redis)
    /// Not persisted in MongoDB, only used during creation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub telegram_verification_code: Option<String>,

    // === Policies (Optional but required for completion) ===
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warranty_policy: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub refund_policy: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub support_hours: Option<String>,

    // === Status & Level ===
    pub status: ShopStatus,

    /// Shop level: new -> silver -> gold -> diamond -> partner
    pub level: ShopLevel,

    // === Statistics ===
    /// Total products count
    #[serde(default)]
    pub total_products: i64,

    /// Total sales count
    #[serde(default)]
    pub total_sales: i64,

    /// Total revenue (in Trust)
    #[serde(default)]
    pub total_revenue: i64,

    /// Average rating (1-5)
    #[serde(default)]
    pub avg_rating: f64,

    /// Total reviews count
    #[serde(default)]
    pub total_reviews: i64,

    /// Active disputes count
    #[serde(default)]
    pub active_disputes: i64,

    // === Completion Tracking ===
    /// Shop is complete when: telegram_verified + total_products > 0 + policies set
    #[serde(default)]
    pub is_complete: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<BsonDateTime>,

    // === Commission (from platform config) ===
    /// Commission rate (0.05 = 5%, override from config)
    #[serde(default = "default_commission_rate")]
    pub commission_rate: f64,

    // === Storage ===
    /// Storage directory path
    pub storage_path: String,

    // === Timestamps ===
    pub created_at: BsonDateTime,
    pub updated_at: BsonDateTime,
}

/// Shop status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ShopStatus {
    /// Shop is active and selling
    Active,
    /// Shop suspended by admin
    Suspended,
    /// Shop deactivated by vendor
    Inactive,
    /// Shop under review
    UnderReview,
}

/// Shop level progression
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ShopLevel {
    /// New shop (0-100 sales)
    New,
    /// Silver (101-500 sales)
    Silver,
    /// Gold (501-2000 sales)
    Gold,
    /// Diamond (2001-10000 sales)
    Diamond,
    /// Partner (10000+ sales)
    Partner,
}

fn default_commission_rate() -> f64 {
    0.05 // 5% default
}

impl Shop {
    /// Create new shop (draft state, not saved yet)
    pub fn new(
        shop_id: String,
        vendor_id: String,
        shop_name: String,
        shop_slug: String,
        shop_description: String,
        shop_logo: String,
        telegram_username: String,
        telegram_verification_code: String,
        storage_path: String,
    ) -> Self {
        let now = BsonDateTime::now();
        Self {
            id: None,
            shop_id,
            vendor_id,
            shop_name,
            shop_slug,
            shop_description,
            shop_logo,
            shop_banner: None,
            telegram_username,
            telegram_chat_id: None,
            telegram_verified: false,
            telegram_verified_at: None,
            telegram_verification_code: Some(telegram_verification_code),
            warranty_policy: None,
            refund_policy: None,
            support_hours: None,
            status: ShopStatus::Active,
            level: ShopLevel::New,
            total_products: 0,
            total_sales: 0,
            total_revenue: 0,
            avg_rating: 0.0,
            total_reviews: 0,
            active_disputes: 0,
            is_complete: false,
            completed_at: None,
            commission_rate: default_commission_rate(),
            storage_path,
            created_at: now,
            updated_at: now,
        }
    }

    /// Mark shop as complete (called when all conditions met)
    pub fn mark_complete(&mut self) {
        if !self.is_complete {
            self.is_complete = true;
            self.completed_at = Some(BsonDateTime::now());
            self.updated_at = BsonDateTime::now();
        }
    }

    /// Update telegram verification
    pub fn verify_telegram(&mut self, chat_id: String) {
        self.telegram_verified = true;
        self.telegram_chat_id = Some(chat_id);
        self.telegram_verified_at = Some(BsonDateTime::now());
        self.telegram_verification_code = None;
        self.updated_at = BsonDateTime::now();
        self.check_completion();
    }

    /// Add product and check completion
    pub fn increment_products(&mut self, count: i64) {
        self.total_products += count;
        self.updated_at = BsonDateTime::now();
        self.check_completion();
    }

    /// Update policies and check completion
    pub fn update_policies(
        &mut self,
        warranty: Option<String>,
        refund: Option<String>,
        support: Option<String>,
    ) {
        self.warranty_policy = warranty;
        self.refund_policy = refund;
        self.support_hours = support;
        self.updated_at = BsonDateTime::now();
        self.check_completion();
    }

    /// Check if shop meets completion criteria
    fn check_completion(&mut self) {
        if self.is_complete {
            return;
        }

        let has_telegram = self.telegram_verified;
        let has_products = self.total_products > 0;
        let has_policies = self.warranty_policy.is_some()
            || self.refund_policy.is_some()
            || self.support_hours.is_some();

        if has_telegram && has_products && has_policies {
            self.mark_complete();
        }
    }

    /// Check if shop is active
    pub fn is_active(&self) -> bool {
        self.status == ShopStatus::Active
    }

    /// Calculate level based on sales
    pub fn update_level(&mut self) {
        self.level = match self.total_sales {
            0..=100 => ShopLevel::New,
            101..=500 => ShopLevel::Silver,
            501..=2000 => ShopLevel::Gold,
            2001..=10000 => ShopLevel::Diamond,
            _ => ShopLevel::Partner,
        };
        self.updated_at = BsonDateTime::now();
    }

    /// Update rating
    pub fn update_rating(&mut self, new_rating: f64) {
        // Calculate new average: (avg * n + new) / (n + 1)
        let total = self.total_reviews as f64;
        self.avg_rating = (self.avg_rating * total + new_rating) / (total + 1.0);
        self.total_reviews += 1;
        self.updated_at = BsonDateTime::now();
    }

    /// Add sale
    pub fn add_sale(&mut self, amount: i64) {
        self.total_sales += 1;
        self.total_revenue += amount;
        self.update_level();
    }

    /// Increment active disputes
    pub fn increment_disputes(&mut self) {
        self.active_disputes += 1;
        self.updated_at = BsonDateTime::now();
    }

    /// Decrement active disputes
    pub fn decrement_disputes(&mut self) {
        if self.active_disputes > 0 {
            self.active_disputes -= 1;
        }
        self.updated_at = BsonDateTime::now();
    }
}

// ============================================================================
// TELEGRAM VERIFICATION CODE (Redis)
// ============================================================================

/// Telegram verification code stored in Redis
/// Key: telegram:verify:{shop_id}
/// TTL: 24 hours
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramVerificationCode {
    pub shop_id: String,
    pub code: String,
    pub created_at: i64,
    pub expires_at: i64,
}

impl TelegramVerificationCode {
    pub fn new(shop_id: String, code: String) -> Self {
        let now = BsonDateTime::now();
        let created_ms = now.timestamp_millis();
        let expires_ms = created_ms + (24 * 60 * 60 * 1000); // 24 hours

        Self {
            shop_id,
            code,
            created_at: created_ms,
            expires_at: expires_ms,
        }
    }

    pub fn is_expired(&self) -> bool {
        BsonDateTime::now().timestamp_millis() > self.expires_at
    }
}

// ============================================================================
// SHOP COMPLETION TRACKING
// ============================================================================

/// Shop completion status
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CompletionStatus {
    /// Shop not complete yet
    Incomplete,
    /// Shop ready to sell
    Complete,
}

/// Shop completion check result
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ShopCompletionStatus {
    pub is_complete: bool,

    /// Telegram verified
    pub has_telegram: bool,

    /// Has at least 1 product
    pub has_products: bool,

    /// Has policies set
    pub has_policies: bool,

    /// Missing items
    pub missing_requirements: Vec<String>,

    /// Completion percentage (0-100)
    pub completion_percentage: i32,
}

impl ShopCompletionStatus {
    pub fn from_shop(shop: &Shop) -> Self {
        let has_telegram = shop.telegram_verified;
        let has_products = shop.total_products > 0;
        let has_policies = shop.warranty_policy.is_some()
            || shop.refund_policy.is_some()
            || shop.support_hours.is_some();

        let mut missing_requirements = Vec::new();
        if !has_telegram {
            missing_requirements.push("telegram_verification".to_string());
        }
        if !has_products {
            missing_requirements.push("products".to_string());
        }
        if !has_policies {
            missing_requirements.push("policies".to_string());
        }

        let completion_percentage = match (has_telegram, has_products, has_policies) {
            (true, true, true) => 100,
            (true, true, false) | (true, false, true) | (false, true, true) => 66,
            (true, false, false) | (false, true, false) | (false, false, true) => 33,
            (false, false, false) => 0,
        };

        Self {
            is_complete: shop.is_complete,
            has_telegram,
            has_products,
            has_policies,
            missing_requirements,
            completion_percentage,
        }
    }
}
