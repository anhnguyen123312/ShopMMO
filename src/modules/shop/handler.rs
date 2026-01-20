//! Shop HTTP Handlers - P2PMMO V2
//!
//! Actix-web handlers for Shop endpoints

use actix_web::{web, HttpRequest, HttpResponse};
use validator::Validate;

use crate::core::{ApiError, ApiResponse};
use crate::config::AppConfig;
use crate::middleware::{AuthUser, AdminUser};
use super::dto::*;
use super::service::ShopService;
use std::sync::Arc;

// ============================================================================
// CREATE SHOP (Vendor only)
// ============================================================================

/// POST /api/vendor/shop/create - Create new shop
#[utoipa::path(
    post,
    path = "/api/vendor/shop/create",
    tag = "Shop - Vendor",
    description = "Create a new shop for the authenticated vendor. Auto-approves and generates Telegram verification code.",
    request_body = CreateShopRequest,
    responses(
        (status = 200, description = "Shop created successfully", body = ApiResponse<CreateShopResponse>),
        (status = 400, description = "Bad request - validation failed or shop already exists"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_shop(
    service: web::Data<Arc<ShopService>>,
    auth: AuthUser,
    req: web::Json<CreateShopRequest>,
) -> Result<HttpResponse, ApiError> {
    req.validate()?;

    let response = service
        .create_shop(auth.user_id.clone(), req.into_inner())
        .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

// ============================================================================
// GET SHOP (Public & Vendor)
// ============================================================================

/// GET /api/vendor/shop/dashboard - Get vendor dashboard
#[utoipa::path(
    get,
    path = "/api/vendor/shop/dashboard",
    tag = "Shop - Vendor",
    description = "Get vendor's shop dashboard with completion status and statistics.",
    responses(
        (status = 200, description = "Dashboard data retrieved", body = ApiResponse<ShopDashboardResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Shop not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_dashboard(
    service: web::Data<Arc<ShopService>>,
    auth: AuthUser,
) -> Result<HttpResponse, ApiError> {
    let response = service.get_shop_dashboard(&auth.user_id).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

/// GET /api/vendor/shop/verification - Get Telegram verification info
#[utoipa::path(
    get,
    path = "/api/vendor/shop/verification",
    tag = "Shop - Vendor",
    description = "Get Telegram verification status and instructions for the vendor's shop.",
    responses(
        (status = 200, description = "Verification info retrieved", body = ApiResponse<ShopVerificationResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Shop not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_verification_info(
    service: web::Data<Arc<ShopService>>,
    auth: AuthUser,
) -> Result<HttpResponse, ApiError> {
    let response = service.get_verification_info(&auth.user_id).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

/// GET /api/shops/{shop_id} - Get shop by ID (public)
#[utoipa::path(
    get,
    path = "/api/shops/{shop_id}",
    tag = "Shop - Public",
    description = "Get shop details by ID. Public endpoint.",
    params(
        ("shop_id" = String, Path, description = "Shop ID")
    ),
    responses(
        (status = 200, description = "Shop details retrieved", body = ApiResponse<ShopDetailResponse>),
        (status = 404, description = "Shop not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_shop(
    service: web::Data<Arc<ShopService>>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let shop_id = path.into_inner();
    let response = service.get_shop(&shop_id).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

/// GET /api/shops/slug/{slug} - Get shop by slug (public)
#[utoipa::path(
    get,
    path = "/api/shops/slug/{slug}",
    tag = "Shop - Public",
    description = "Get shop details by slug. Public endpoint.",
    params(
        ("slug" = String, Path, description = "Shop slug")
    ),
    responses(
        (status = 200, description = "Shop details retrieved", body = ApiResponse<ShopDetailResponse>),
        (status = 404, description = "Shop not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_shop_by_slug(
    service: web::Data<Arc<ShopService>>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let slug = path.into_inner();
    let response = service.get_shop_by_slug(&slug).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

// ============================================================================
// UPDATE SHOP (Vendor only)
// ============================================================================

/// PUT /api/vendor/shop/update - Update shop basic info
#[utoipa::path(
    put,
    path = "/api/vendor/shop/update",
    tag = "Shop - Vendor",
    description = "Update shop basic information (name, description, logo, banner).",
    request_body = UpdateShopRequest,
    responses(
        (status = 200, description = "Shop updated successfully", body = ApiResponse<UpdateShopResponse>),
        (status = 400, description = "Bad request - validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - not your shop"),
        (status = 404, description = "Shop not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_shop(
    service: web::Data<Arc<ShopService>>,
    auth: AuthUser,
    req: web::Json<UpdateShopRequest>,
) -> Result<HttpResponse, ApiError> {
    req.validate()?;

    // Get vendor's shop first
    let dashboard = service.get_shop_dashboard(&auth.user_id).await?;
    let shop_id = dashboard.shop_id;

    let response = service
        .update_shop(&shop_id, &auth.user_id, req.into_inner())
        .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

/// PUT /api/vendor/shop/policies - Update shop policies
#[utoipa::path(
    put,
    path = "/api/vendor/shop/policies",
    tag = "Shop - Vendor",
    description = "Update shop policies (warranty, refund, support hours). Required for shop completion.",
    request_body = UpdateShopPoliciesRequest,
    responses(
        (status = 200, description = "Policies updated successfully", body = ApiResponse<UpdateShopResponse>),
        (status = 400, description = "Bad request - validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - not your shop"),
        (status = 404, description = "Shop not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_policies(
    service: web::Data<Arc<ShopService>>,
    auth: AuthUser,
    req: web::Json<UpdateShopPoliciesRequest>,
) -> Result<HttpResponse, ApiError> {
    req.validate()?;

    // Get vendor's shop first
    let dashboard = service.get_shop_dashboard(&auth.user_id).await?;
    let shop_id = dashboard.shop_id;

    let response = service
        .update_policies(&shop_id, &auth.user_id, req.into_inner())
        .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

// ============================================================================
// LIST SHOPS (Public)
// ============================================================================

/// GET /api/shops - List shops with filters
#[utoipa::path(
    get,
    path = "/api/shops",
    tag = "Shop - Public",
    description = "List shops with pagination and filters. Public endpoint.",
    params(
        ("page" = Option<i64>, Query, description = "Page number (default: 1)"),
        ("per_page" = Option<i64>, Query, description = "Page size (default: 20, max: 100)"),
        ("search" = Option<String>, Query, description = "Search by name or slug"),
        ("status" = Option<String>, Query, description = "Filter by status"),
        ("level" = Option<String>, Query, description = "Filter by level"),
        ("vendor_id" = Option<String>, Query, description = "Filter by vendor ID"),
        ("sort_by" = Option<String>, Query, description = "Sort by field (default: created_at)"),
        ("sort_order" = Option<String>, Query, description = "Sort order: asc or desc (default: desc)")
    ),
    responses(
        (status = 200, description = "Shop list retrieved", body = ApiResponse<ShopListResponse>),
        (status = 400, description = "Bad request - invalid query parameters"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_shops(
    service: web::Data<Arc<ShopService>>,
    query: web::Query<ShopListQuery>,
) -> Result<HttpResponse, ApiError> {
    let response = service.list_shops(query.into_inner()).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

/// GET /api/shops/search/{term} - Search shops
#[utoipa::path(
    get,
    path = "/api/shops/search/{term}",
    tag = "Shop - Public",
    description = "Search shops by name or slug. Public endpoint.",
    params(
        ("term" = String, Path, description = "Search term"),
        ("page" = Option<i64>, Query, description = "Page number (default: 1)"),
        ("per_page" = Option<i64>, Query, description = "Page size (default: 20)")
    ),
    responses(
        (status = 200, description = "Search results", body = ApiResponse<ShopListResponse>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn search_shops(
    service: web::Data<Arc<ShopService>>,
    path: web::Path<String>,
    query: web::Query<SearchShopsQuery>,
) -> Result<HttpResponse, ApiError> {
    let term = path.into_inner();
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);

    let response = service.search_shops(&term, page, per_page).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

// ============================================================================
// TELEGRAM VERIFICATION (Internal - Bot)
// ============================================================================

/// POST /api/shop/telegram/verify - Verify Telegram (internal)
#[utoipa::path(
    post,
    path = "/api/shop/telegram/verify",
    tag = "Shop - Internal",
    description = "Internal endpoint called by Telegram bot to verify shop. Requires bot API key.",
    request_body = TelegramVerifyRequest,
    responses(
        (status = 200, description = "Telegram verified successfully", body = ApiResponse<TelegramVerifyResponse>),
        (status = 400, description = "Invalid or expired verification code"),
        (status = 401, description = "Invalid or missing bot API key"),
        (status = 500, description = "Internal server error")
    ),
    security(("bot_api_key" = []))
)]
pub async fn verify_telegram(
    service: web::Data<Arc<ShopService>>,
    config: web::Data<Arc<AppConfig>>,
    req: HttpRequest,
    body: web::Json<TelegramVerifyInternalRequest>,
) -> Result<HttpResponse, ApiError> {
    body.validate()?;

    // Verify bot API key from X-Bot-API-Key header
    let bot_api_key = req.headers().get("X-Bot-API-Key")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| ApiError::unauthorized("Missing bot API key"))?;

    if bot_api_key != config.telegram.bot_api_key {
        tracing::warn!("Invalid bot API key provided");
        return Err(ApiError::unauthorized("Invalid bot API key"));
    }

    let response = service
        .verify_telegram(
            &body.verification_code,
            body.chat_id.clone(),
            body.username.clone(),
        )
        .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

// ============================================================================
// SHOP STATS (Admin)
// ============================================================================

/// GET /admin/api/shops/stats - Get shop statistics
#[utoipa::path(
    get,
    path = "/admin/api/shops/stats",
    tag = "Shop - Admin",
    description = "Get shop statistics for admin dashboard. Requires admin role.",
    responses(
        (status = 200, description = "Statistics retrieved", body = ApiResponse<ShopStatsResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin only"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_stats(
    _admin: AdminUser,
    service: web::Data<Arc<ShopService>>,
) -> Result<HttpResponse, ApiError> {
    let response = service.get_stats().await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

// ============================================================================
// HELPER STRUCTS
// ============================================================================

/// Internal request for Telegram verification (with username)
#[derive(Debug, serde::Deserialize, validator::Validate)]
#[serde(rename_all = "camelCase")]
pub struct TelegramVerifyInternalRequest {
    pub verification_code: String,
    pub chat_id: String,
    pub username: Option<String>,
}

/// Query params for search shops
#[derive(Debug, serde::Deserialize)]
pub struct SearchShopsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}
