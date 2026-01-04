//! Shop DTOs - P2PMMO V2
//!
//! Request and Response structures cho Shop endpoints
//! Theo flow: Create Shop → Telegram Verify → Complete

use serde::{Deserialize, Serialize};
use validator::Validate;
use utoipa::ToSchema;

use super::domain::{Shop, ShopLevel, ShopStatus};

// ============================================================================
// CREATE SHOP DTOs (4-step Wizard)
// ============================================================================

/// Complete shop creation request (all 4 steps combined)
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateShopRequest {
    // === Step 1: Basic Info (REQUIRED) ===
    /// Shop name (3-50 characters)
    #[validate(length(min = 3, max = 50))]
    pub shop_name: String,

    /// Shop description (max 500 characters)
    #[validate(length(max = 500))]
    pub shop_description: String,

    // === Step 2: Branding (REQUIRED) ===
    /// Shop logo URL (or base64 for upload)
    #[validate(length(min = 1))]
    pub shop_logo: String,

    /// Shop banner URL (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(max = 500))]
    pub shop_banner: Option<String>,

    // === Step 3: Telegram (REQUIRED) ===
    /// Telegram username with @ format (@username, 11-32 chars)
    #[validate(length(min = 11, max = 32))]
    pub telegram_username: String,

    // === Step 4: Policies (OPTIONAL) ===
    /// Warranty policy
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(max = 2000))]
    pub warranty_policy: Option<String>,

    /// Refund policy
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(max = 2000))]
    pub refund_policy: Option<String>,

    /// Support hours
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(max = 200))]
    pub support_hours: Option<String>,
}

/// Create shop response
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateShopResponse {
    pub shop_id: String,
    pub vendor_id: String,
    pub shop_name: String,
    pub shop_slug: String,

    /// Telegram verification code (UUID)
    /// User needs to send: /start {code} to @p2pmmo bot
    pub telegram_verification_code: String,

    /// Instruction for user
    pub telegram_instruction: String,

    /// Telegram verification status
    pub telegram_verified: bool,

    /// Shop status
    pub status: ShopStatus,

    /// Storage path
    pub storage_path: String,

    pub created_at: String,
}

// ============================================================================
// UPDATE SHOP DTOs
// ============================================================================

/// Update shop basic info
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateShopRequest {
    /// Shop name (3-50 characters)
    #[validate(length(min = 3, max = 50))]
    pub shop_name: Option<String>,

    /// Shop description (max 500 characters)
    #[validate(length(max = 500))]
    pub shop_description: Option<String>,

    /// Shop logo URL
    #[validate(length(max = 500))]
    pub shop_logo: Option<String>,

    /// Shop banner URL
    #[validate(length(max = 500))]
    pub shop_banner: Option<String>,
}

/// Update shop policies
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateShopPoliciesRequest {
    /// Warranty policy
    #[validate(length(max = 2000))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warranty_policy: Option<String>,

    /// Refund policy
    #[validate(length(max = 2000))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refund_policy: Option<String>,

    /// Support hours
    #[validate(length(max = 200))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub support_hours: Option<String>,
}

/// Update shop response
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateShopResponse {
    pub shop_id: String,
    pub updated_fields: Vec<String>,
    pub updated_at: String,
}

// ============================================================================
// TELEGRAM VERIFICATION DTOs
// ============================================================================

/// Telegram verification request (internal - called by bot)
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TelegramVerifyRequest {
    /// Verification code from /start command
    #[validate(length(min = 1))]
    pub verification_code: String,

    /// Chat ID from Telegram
    #[validate(length(min = 1))]
    pub chat_id: String,

    /// Username from Telegram (for soft check)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
}

/// Telegram verification response
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TelegramVerifyResponse {
    pub shop_id: String,
    pub shop_name: String,
    pub success: bool,

    /// Warning if username mismatch
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username_mismatch_warning: Option<String>,

    /// Verified at
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified_at: Option<String>,

    /// Shop completion status after verification
    pub completion_status: super::domain::ShopCompletionStatus,
}

// ============================================================================
// SHOP RESPONSE DTOs
// ============================================================================

/// Basic shop response (for lists)
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ShopResponse {
    pub shop_id: String,
    pub vendor_id: String,
    pub shop_name: String,
    pub shop_slug: String,
    pub shop_logo: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub shop_banner: Option<String>,

    pub status: ShopStatus,
    pub level: ShopLevel,

    /// Rating and reviews
    pub avg_rating: f64,
    pub total_reviews: i64,

    /// Stats
    pub total_products: i64,
    pub total_sales: i64,

    /// Telegram verified
    pub telegram_verified: bool,

    pub created_at: String,
}

/// Detailed shop response (for single shop view)
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ShopDetailResponse {
    pub shop_id: String,
    pub vendor_id: String,
    pub shop_name: String,
    pub shop_slug: String,
    pub shop_description: String,
    pub shop_logo: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub shop_banner: Option<String>,

    // Telegram info
    pub telegram_username: String,
    pub telegram_verified: bool,

    // Policies
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warranty_policy: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub refund_policy: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub support_hours: Option<String>,

    // Status & Level
    pub status: ShopStatus,
    pub level: ShopLevel,
    pub is_complete: bool,

    // Stats
    pub total_products: i64,
    pub total_sales: i64,
    pub total_revenue: i64,
    pub avg_rating: f64,
    pub total_reviews: i64,
    pub active_disputes: i64,

    // Commission
    pub commission_rate: f64,

    // Completion status
    pub completion_status: super::domain::ShopCompletionStatus,

    // Timestamps
    pub created_at: String,
    pub updated_at: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
}

/// Shop dashboard response (for vendor)
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ShopDashboardResponse {
    pub shop_id: String,
    pub shop_name: String,

    // Completion status
    pub is_complete: bool,
    pub completion_status: super::domain::ShopCompletionStatus,

    // Telegram status
    pub telegram_verified: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub telegram_verification_code: Option<String>,

    // Quick stats
    pub total_products: i64,
    pub total_sales: i64,
    pub total_revenue: i64,
    pub avg_rating: f64,
    pub total_reviews: i64,
    pub active_disputes: i64,

    // Level & Commission
    pub level: ShopLevel,
    pub commission_rate: f64,

    // Next level info
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_level: Option<ShopLevel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sales_to_next_level: Option<i64>,

    // Storage
    pub storage_path: String,

    pub updated_at: String,
}

/// Shop verification info response
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ShopVerificationResponse {
    pub shop_id: String,
    pub shop_name: String,
    pub telegram_username: String,
    pub telegram_verified: bool,

    /// Verification code (if not verified yet)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_code: Option<String>,

    /// Verification code expires at (if not verified)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_expires_at: Option<String>,

    /// Instruction
    pub instruction: String,

    /// Verified at
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified_at: Option<String>,
}

// ============================================================================
// SHOP LIST/QUERY DTOs
// ============================================================================

/// Shop list query parameters
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ShopListQuery {
    /// Page number (default: 1)
    #[serde(default = "default_page")]
    pub page: i64,

    /// Page size (default: 20, max: 100)
    #[serde(default = "default_per_page")]
    pub per_page: i64,

    /// Search by name or slug
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,

    /// Filter by status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// Filter by level
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,

    /// Filter by vendor ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor_id: Option<String>,

    /// Sort by: created_at, total_sales, avg_rating, total_products
    #[serde(default = "default_sort")]
    pub sort_by: String,

    /// Sort order: asc, desc
    #[serde(default = "default_sort_order")]
    pub sort_order: String,
}

/// Shop list response
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ShopListResponse {
    pub shops: Vec<ShopResponse>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

// ============================================================================
// SHOP STATS DTOs
// ============================================================================

/// Shop statistics response
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ShopStatsResponse {
    pub total_shops: i64,
    pub active_shops: i64,
    pub new_shops_today: i64,
    pub complete_shops: i64,
    pub telegram_verified_shops: i64,

    /// By level
    pub by_level: ShopLevelStats,

    /// Top shops
    pub top_shops: Vec<TopShopStats>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ShopLevelStats {
    pub new: i64,
    pub silver: i64,
    pub gold: i64,
    pub diamond: i64,
    pub partner: i64,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TopShopStats {
    pub shop_id: String,
    pub shop_name: String,
    pub total_sales: i64,
    pub total_revenue: i64,
    pub avg_rating: f64,
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn default_page() -> i64 { 1 }
fn default_per_page() -> i64 { 20 }
fn default_sort() -> String { "created_at".to_string() }
fn default_sort_order() -> String { "desc".to_string() }

// ============================================================================
// CONVERSIONS
// ============================================================================

impl From<Shop> for ShopResponse {
    fn from(shop: Shop) -> Self {
        Self {
            shop_id: shop.shop_id,
            vendor_id: shop.vendor_id,
            shop_name: shop.shop_name,
            shop_slug: shop.shop_slug,
            shop_logo: shop.shop_logo,
            shop_banner: shop.shop_banner,
            status: shop.status,
            level: shop.level,
            avg_rating: shop.avg_rating,
            total_reviews: shop.total_reviews,
            total_products: shop.total_products,
            total_sales: shop.total_sales,
            telegram_verified: shop.telegram_verified,
            created_at: shop.created_at.to_string(),
        }
    }
}

impl From<Shop> for ShopDetailResponse {
    fn from(shop: Shop) -> Self {
        let completion_status = super::domain::ShopCompletionStatus::from_shop(&shop);

        Self {
            shop_id: shop.shop_id,
            vendor_id: shop.vendor_id,
            shop_name: shop.shop_name,
            shop_slug: shop.shop_slug,
            shop_description: shop.shop_description,
            shop_logo: shop.shop_logo,
            shop_banner: shop.shop_banner,
            telegram_username: shop.telegram_username,
            telegram_verified: shop.telegram_verified,
            warranty_policy: shop.warranty_policy,
            refund_policy: shop.refund_policy,
            support_hours: shop.support_hours,
            status: shop.status,
            level: shop.level,
            is_complete: shop.is_complete,
            total_products: shop.total_products,
            total_sales: shop.total_sales,
            total_revenue: shop.total_revenue,
            avg_rating: shop.avg_rating,
            total_reviews: shop.total_reviews,
            active_disputes: shop.active_disputes,
            commission_rate: shop.commission_rate,
            completion_status,
            created_at: shop.created_at.to_string(),
            updated_at: shop.updated_at.to_string(),
            completed_at: shop.completed_at.map(|dt| dt.to_string()),
        }
    }
}

impl From<Shop> for ShopDashboardResponse {
    fn from(shop: Shop) -> Self {
        let completion_status = super::domain::ShopCompletionStatus::from_shop(&shop);

        // Calculate next level
        let (next_level, sales_to_next) = match shop.level {
            ShopLevel::New => (Some(ShopLevel::Silver), Some(101 - shop.total_sales)),
            ShopLevel::Silver => (Some(ShopLevel::Gold), Some(501 - shop.total_sales)),
            ShopLevel::Gold => (Some(ShopLevel::Diamond), Some(2001 - shop.total_sales)),
            ShopLevel::Diamond => (Some(ShopLevel::Partner), Some(10001 - shop.total_sales)),
            ShopLevel::Partner => (None, None),
        };

        Self {
            shop_id: shop.shop_id,
            shop_name: shop.shop_name,
            is_complete: shop.is_complete,
            completion_status,
            telegram_verified: shop.telegram_verified,
            telegram_verification_code: shop.telegram_verification_code,
            total_products: shop.total_products,
            total_sales: shop.total_sales,
            total_revenue: shop.total_revenue,
            avg_rating: shop.avg_rating,
            total_reviews: shop.total_reviews,
            active_disputes: shop.active_disputes,
            level: shop.level,
            commission_rate: shop.commission_rate,
            next_level,
            sales_to_next_level: if sales_to_next.map(|s| s > 0).unwrap_or(false) { sales_to_next } else { Some(0) },
            storage_path: shop.storage_path,
            updated_at: shop.updated_at.to_string(),
        }
    }
}
