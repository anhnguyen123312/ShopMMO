//! Wallet V3 HTTP Handlers
//!
//! Actix-web handlers for all wallet endpoints

use actix_web::{web, HttpResponse};
use actix_web_grants::protect;
use std::sync::Arc;
use validator::Validate;

use crate::{
    core::{ApiError, ApiResponse},
    middleware::{AuthUser, AdminUser},
};
use super::{dto::*, service::WalletService, service_cron::WalletCronManager};
// Domain types with ToSchema for OpenAPI responses
use super::domain::ValidationResult;

// ========================================================================
// WALLET MANAGEMENT
// ========================================================================

/// GET /api/wallet/balance - Get wallet balance
#[utoipa::path(
    get,
    path = "/api/wallet/balance",
    tag = "Wallet - User",
    description = "Get wallet balance for the authenticated user. Returns available balance and pending balance (for vendors).",
    responses(
        (status = 200, description = "Wallet balance retrieved successfully", body = ApiResponse<WalletBalanceResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - insufficient permissions"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[protect("BUYER", "SELLER")]
pub async fn get_balance(
    service: web::Data<Arc<WalletService>>,
    auth: AuthUser,
) -> Result<HttpResponse, ApiError> {
    let balance = service.get_wallet_balance(&auth.wallet_id).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(balance)))
}

// ========================================================================
// DEPOSIT FLOW
// ========================================================================

/// POST /admin/api/wallet/deposit/auto - Create auto deposit request
#[utoipa::path(
    post,
    path = "/admin/api/wallet/deposit/auto",
    tag = "Wallet - Admin",
    description = "Create auto deposit request for monitoring specific payment methods. Admin only.",
    request_body = AutoDepositRequest,
    responses(
        (status = 200, description = "Deposit request created", body = ApiResponse<DepositResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin only"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_auto_deposit(
    service: web::Data<Arc<WalletService>>,
    auth: AuthUser,
    req: web::Json<AutoDepositRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = service
        .create_auto_deposit(&auth.wallet_id, req.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

/// POST /admin/api/wallet/deposit/manual - Manual deposit by admin
#[utoipa::path(
    post,
    path = "/admin/api/wallet/deposit/manual",
    tag = "Wallet - Admin",
    description = "Admin manually credits a user's wallet. Used for processing manual deposit requests with proof.",
    request_body = ManualDepositRequest,
    responses(
        (status = 200, description = "Manual deposit completed", body = ApiResponse<DepositResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin only"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn manual_deposit(
    service: web::Data<Arc<WalletService>>,
    admin: AdminUser,
    req: web::Json<ManualDepositRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = service
        .manual_deposit(req.into_inner(), admin.user_id)
        .await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

// ========================================================================
// 3RD PARTY DEPOSIT FLOW
// ========================================================================

/// POST /api/wallet/deposit/initiate - Initiate deposit via payment gateway
#[utoipa::path(
    post,
    path = "/api/wallet/deposit/initiate",
    tag = "Wallet - User",
    description = "Initiate a new deposit request through payment gateway (bank transfer, Momo, USDT, etc.)",
    request_body = DepositInitiateRequest,
    responses(
        (status = 200, description = "Deposit initiated successfully", body = ApiResponse<DepositInitiateResponse>),
        (status = 400, description = "Bad request - invalid amount or payment method"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[protect("BUYER", "SELLER")]
pub async fn initiate_deposit(
    service: web::Data<Arc<WalletService>>,
    auth: AuthUser,
    req: web::Json<DepositInitiateRequest>,
) -> Result<HttpResponse, ApiError> {
    req.validate()?;
    let response = service
        .initiate_deposit(&auth.wallet_id, &auth.user_id, req.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

/// POST /admin/api/wallet/deposit/webhook - Payment gateway webhook callback
#[utoipa::path(
    post,
    path = "/admin/api/wallet/deposit/webhook",
    tag = "Wallet - Admin",
    description = "Payment gateway webhook callback to process deposit status updates. Admin endpoint for security.",
    request_body = PaymentWebhookPayload,
    responses(
        (status = 200, description = "Webhook processed successfully"),
        (status = 400, description = "Bad request - invalid signature or payload"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn deposit_webhook(
    service: web::Data<Arc<WalletService>>,
    req: web::Json<PaymentWebhookPayload>,
) -> Result<HttpResponse, ApiError> {
    service
        .process_deposit_webhook(req.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success("Webhook processed")))
}

/// GET /api/wallet/deposit/status/{tx_id} - Check deposit status
#[utoipa::path(
    get,
    path = "/api/wallet/deposit/status/{tx_id}",
    tag = "Wallet - User",
    description = "Check the status of a specific deposit transaction",
    params(
        ("tx_id" = String, Path, description = "Deposit transaction ID")
    ),
    responses(
        (status = 200, description = "Deposit status retrieved", body = ApiResponse<DepositStatusResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Deposit not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[protect("BUYER", "SELLER")]
pub async fn get_deposit_status(
    service: web::Data<Arc<WalletService>>,
    auth: AuthUser,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let tx_id = path.into_inner();
    let response = service
        .get_deposit_status(&auth.user_id, &tx_id)
        .await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

/// GET /api/wallet/deposits/history - Get user's deposit history
#[utoipa::path(
    get,
    path = "/api/wallet/deposits/history",
    tag = "Wallet - User",
    description = "Get the current user's deposit history with pagination",
    params(
        ("page" = Option<i64>, Query, description = "Page number (default: 1)"),
        ("per_page" = Option<i64>, Query, description = "Items per page (default: 20)")
    ),
    responses(
        (status = 200, description = "Deposit history retrieved", body = ApiResponse<DepositHistoryResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[protect("BUYER", "SELLER")]
pub async fn get_deposit_history(
    service: web::Data<Arc<WalletService>>,
    auth: AuthUser,
    query: web::Query<DepositHistoryQuery>,
) -> Result<HttpResponse, ApiError> {
    let response = service
        .get_deposit_history(&auth.user_id, query.page, query.per_page)
        .await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

/// POST /admin/api/wallet/deposit - Manual deposit by admin
#[utoipa::path(
    post,
    path = "/admin/api/wallet/deposit",
    tag = "Wallet - Admin",
    description = "Manually credit funds to a user's wallet (admin operation)",
    request_body = AdminManualDepositRequest,
    responses(
        (status = 200, description = "Manual deposit completed", body = ApiResponse<DepositStatusResponse>),
        (status = 400, description = "Bad request - invalid amount or reason"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin only"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[protect("ADMIN")]
pub async fn admin_manual_deposit(
    service: web::Data<Arc<WalletService>>,
    admin: AdminUser,
    req: web::Json<AdminManualDepositRequest>,
) -> Result<HttpResponse, ApiError> {
    req.validate()?;
    let response = service
        .admin_manual_deposit(req.into_inner(), admin.user_id)
        .await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

/// GET /admin/api/wallet/deposits/history - List all deposits (admin)
#[utoipa::path(
    get,
    path = "/admin/api/wallet/deposits/history",
    tag = "Wallet - Admin",
    description = "List all deposits across the platform with filtering and pagination",
    params(
        ("page" = Option<i64>, Query, description = "Page number (default: 1)"),
        ("per_page" = Option<i64>, Query, description = "Items per page (default: 50)"),
        ("status" = Option<String>, Query, description = "Filter by status"),
        ("user_id" = Option<String>, Query, description = "Filter by user ID")
    ),
    responses(
        (status = 200, description = "Deposits history retrieved", body = ApiResponse<DepositHistoryResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin only"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[protect("ADMIN")]
pub async fn admin_get_deposits_history(
    service: web::Data<Arc<WalletService>>,
    _admin: AdminUser,
    query: web::Query<AdminDepositHistoryQuery>,
) -> Result<HttpResponse, ApiError> {
    let response = service
        .admin_get_deposits_history(query.page, query.per_page, query.status.clone(), query.user_id.clone())
        .await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

// ========================================================================
// WITHDRAWAL FLOW
// ========================================================================

/// POST /api/wallet/withdrawal - Create withdrawal request
#[utoipa::path(
    post,
    path = "/api/wallet/withdrawal",
    tag = "Wallet - User",
    description = "Create a withdrawal request to transfer funds from wallet to bank account. Vendor only.",
    request_body = WithdrawalRequest,
    responses(
        (status = 200, description = "Withdrawal request created", body = ApiResponse<WithdrawalResponse>),
        (status = 400, description = "Bad request - insufficient balance or invalid amount"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Vendor only"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_withdrawal(
    service: web::Data<Arc<WalletService>>,
    auth: AuthUser,
    req: web::Json<WithdrawalRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = service
        .create_withdrawal(&auth.wallet_id, req.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

/// GET /admin/api/wallet/withdrawal/{request_id}/validate - Validate withdrawal
#[utoipa::path(
    get,
    path = "/admin/api/wallet/withdrawal/{request_id}/validate",
    tag = "Wallet - Admin",
    description = "Validate withdrawal request before approval. Checks balance, limits, and bank info.",
    params(
        ("request_id" = String, Path, description = "Withdrawal request ID")
    ),
    responses(
        (status = 200, description = "Withdrawal validated", body = ApiResponse<ValidationResult>),
        (status = 404, description = "Withdrawal not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn validate_withdrawal(
    service: web::Data<Arc<WalletService>>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let request_id = path.into_inner();
    let validation = service.validate_withdrawal(&request_id).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(validation)))
}

/// POST /admin/api/wallet/withdrawal/{request_id}/approve - Approve withdrawal
#[utoipa::path(
    post,
    path = "/admin/api/wallet/withdrawal/{request_id}/approve",
    tag = "Wallet - Admin",
    description = "Approve withdrawal request and mark it for processing. Admin only.",
    params(
        ("request_id" = String, Path, description = "Withdrawal request ID")
    ),
    responses(
        (status = 200, description = "Withdrawal approved", body = ApiResponse<SuccessResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin only"),
        (status = 404, description = "Withdrawal not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn approve_withdrawal(
    service: web::Data<Arc<WalletService>>,
    admin: AdminUser,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let request_id = path.into_inner();
    let response = service.approve_withdrawal(&request_id, admin.user_id).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

/// POST /admin/api/wallet/withdrawal/{request_id}/reject - Reject withdrawal
#[utoipa::path(
    post,
    path = "/admin/api/wallet/withdrawal/{request_id}/reject",
    tag = "Wallet - Admin",
    description = "Reject withdrawal request and refund amount to vendor's wallet. Admin only.",
    params(
        ("request_id" = String, Path, description = "Withdrawal request ID")
    ),
    request_body = RejectWithdrawalRequest,
    responses(
        (status = 200, description = "Withdrawal rejected", body = ApiResponse<SuccessResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin only"),
        (status = 404, description = "Withdrawal not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn reject_withdrawal(
    service: web::Data<Arc<WalletService>>,
    admin: AdminUser,
    path: web::Path<String>,
    req: web::Json<RejectWithdrawalRequest>,
) -> Result<HttpResponse, ApiError> {
    let request_id = path.into_inner();
    let response = service
        .reject_withdrawal(&request_id, req.into_inner(), admin.user_id)
        .await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

/// POST /admin/api/wallet/withdrawal/{request_id}/complete - Complete bank transfer
#[utoipa::path(
    post,
    path = "/admin/api/wallet/withdrawal/{request_id}/complete",
    tag = "Wallet - Admin",
    description = "Mark withdrawal as completed after bank transfer is done. Admin only.",
    params(
        ("request_id" = String, Path, description = "Withdrawal request ID")
    ),
    request_body = CompleteBankTransferRequest,
    responses(
        (status = 200, description = "Bank transfer completed", body = ApiResponse<SuccessResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin only"),
        (status = 404, description = "Withdrawal not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn complete_bank_transfer(
    service: web::Data<Arc<WalletService>>,
    admin: AdminUser,
    path: web::Path<String>,
    req: web::Json<CompleteBankTransferRequest>,
) -> Result<HttpResponse, ApiError> {
    let request_id = path.into_inner();
    let response = service
        .complete_bank_transfer(&request_id, req.into_inner(), admin.user_id)
        .await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

// ========================================================================
// PURCHASE & ESCROW FLOW
// ========================================================================

/// POST /api/wallet/purchase - Create purchase
#[utoipa::path(
    post,
    path = "/api/wallet/purchase",
    tag = "Wallet - User",
    description = "Create a purchase order and hold funds in escrow. Buyer only.",
    request_body = PurchaseRequest,
    responses(
        (status = 200, description = "Purchase created", body = ApiResponse<PurchaseResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_purchase(
    service: web::Data<Arc<WalletService>>,
    auth: AuthUser,
    req: web::Json<PurchaseRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = service
        .create_purchase(&auth.wallet_id, req.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

/// POST /api/wallet/escrow/{escrow_id}/early-release - Early release escrow
#[utoipa::path(
    post,
    path = "/api/wallet/escrow/{escrow_id}/early-release",
    tag = "Wallet - User",
    description = "Buyer confirms satisfaction and releases escrow funds to seller early (before 72h auto-release).",
    params(
        ("escrow_id" = String, Path, description = "Escrow ID")
    ),
    responses(
        (status = 200, description = "Escrow released early", body = ApiResponse<SuccessResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Escrow not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn early_release_escrow(
    service: web::Data<Arc<WalletService>>,
    auth: AuthUser,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let escrow_id = path.into_inner();
    let response = service
        .early_release_escrow(&escrow_id, auth.user_id)
        .await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

/// POST /api/wallet/escrow/{escrow_id}/dispute - Buyer creates dispute
#[utoipa::path(
    post,
    path = "/api/wallet/escrow/{escrow_id}/dispute",
    tag = "Wallet - User",
    description = "Buyer creates a dispute for an escrow hold. Must be done within 72 hours of purchase.",
    params(
        ("escrow_id" = String, Path, description = "Escrow ID")
    ),
    request_body = CreateDisputeRequest,
    responses(
        (status = 200, description = "Dispute created successfully", body = ApiResponse<DisputeInfoResponse>),
        (status = 400, description = "Bad request - invalid evidence images or escrow status"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - not your escrow"),
        (status = 404, description = "Escrow not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_dispute(
    service: web::Data<Arc<WalletService>>,
    auth: AuthUser,
    path: web::Path<String>,
    req: web::Json<CreateDisputeRequest>,
) -> Result<HttpResponse, ApiError> {
    let escrow_id = path.into_inner();
    let response = service
        .create_dispute_case(&escrow_id, auth.user_id, req.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

/// POST /admin/api/wallet/escrow/{escrow_id}/resolve/refund - Admin resolves dispute with refund
#[utoipa::path(
    post,
    path = "/admin/api/wallet/escrow/{escrow_id}/resolve/refund",
    tag = "Wallet - Admin",
    description = "Admin resolves a dispute by refunding the buyer. Full amount refunded to buyer, escrow released.",
    params(
        ("escrow_id" = String, Path, description = "Escrow ID")
    ),
    request_body = ResolveDisputeRequest,
    responses(
        (status = 200, description = "Dispute resolved with refund", body = ApiResponse<SuccessResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin only"),
        (status = 404, description = "Escrow not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn resolve_dispute_refund(
    service: web::Data<Arc<WalletService>>,
    admin: AdminUser,
    path: web::Path<String>,
    req: web::Json<ResolveDisputeRequest>,
) -> Result<HttpResponse, ApiError> {
    let escrow_id = path.into_inner();
    let response = service
        .resolve_dispute_refund(&escrow_id, admin.user_id, req.reason.clone())
        .await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

/// POST /admin/api/wallet/escrow/{escrow_id}/resolve/release - Admin resolves dispute with release to seller
#[utoipa::path(
    post,
    path = "/admin/api/wallet/escrow/{escrow_id}/resolve/release",
    tag = "Wallet - Admin",
    description = "Admin resolves a dispute by releasing funds to seller. Commission deducted, net amount to seller.",
    params(
        ("escrow_id" = String, Path, description = "Escrow ID")
    ),
    request_body = ResolveDisputeRequest,
    responses(
        (status = 200, description = "Dispute resolved with release", body = ApiResponse<SuccessResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin only"),
        (status = 404, description = "Escrow not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn resolve_dispute_release(
    service: web::Data<Arc<WalletService>>,
    admin: AdminUser,
    path: web::Path<String>,
    req: web::Json<ResolveDisputeRequest>,
) -> Result<HttpResponse, ApiError> {
    let escrow_id = path.into_inner();
    let response = service
        .resolve_dispute_release(&escrow_id, admin.user_id, req.reason.clone())
        .await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

// ========================================================================
// ADMIN OPERATIONS
// ========================================================================

/// POST /admin/api/wallet/debit - Manual debit
#[utoipa::path(
    post,
    path = "/admin/api/wallet/debit",
    tag = "Wallet - Admin",
    description = "Admin manually debits from a user's wallet. Used for penalties or corrections.",
    request_body = ManualDebitRequest,
    responses(
        (status = 200, description = "Manual debit completed", body = ApiResponse<SuccessResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin only"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn manual_debit(
    service: web::Data<Arc<WalletService>>,
    admin: AdminUser,
    req: web::Json<ManualDebitRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = service.manual_debit(req.into_inner(), admin.user_id).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

/// POST /admin/api/wallet/freeze - Freeze wallet
#[utoipa::path(
    post,
    path = "/admin/api/wallet/freeze",
    tag = "Wallet - Admin",
    description = "Freeze wallet to prevent all transactions. Admin only.",
    request_body = FreezeWalletRequest,
    responses(
        (status = 200, description = "Wallet frozen", body = ApiResponse<SuccessResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin only"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[protect("ADMIN", "SUPER_ADMIN")]
pub async fn freeze_wallet(
    service: web::Data<Arc<WalletService>>,
    admin: AdminUser,
    req: web::Json<FreezeWalletRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = service.freeze_wallet(req.into_inner(), admin.user_id).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

/// POST /admin/api/wallet/unfreeze - Unfreeze wallet
#[utoipa::path(
    post,
    path = "/admin/api/wallet/unfreeze",
    tag = "Wallet - Admin",
    description = "Unfreeze wallet to allow transactions. Admin only.",
    request_body = UnfreezeWalletRequest,
    responses(
        (status = 200, description = "Wallet unfrozen", body = ApiResponse<SuccessResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin only"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[protect("ADMIN", "SUPER_ADMIN")]
pub async fn unfreeze_wallet(
    service: web::Data<Arc<WalletService>>,
    admin: AdminUser,
    req: web::Json<UnfreezeWalletRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = service
        .unfreeze_wallet(req.into_inner(), admin.user_id)
        .await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

/// POST /admin/api/wallet/commission - Set shop commission rate
#[utoipa::path(
    post,
    path = "/admin/api/wallet/commission",
    tag = "Wallet - Admin",
    description = "Set custom commission rate for a specific shop. Admin only.",
    request_body = SetShopCommissionRequest,
    responses(
        (status = 200, description = "Commission rate set", body = ApiResponse<SuccessResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin only"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[protect("ADMIN", "SUPER_ADMIN")]
pub async fn set_shop_commission(
    service: web::Data<Arc<WalletService>>,
    admin: AdminUser,
    req: web::Json<SetShopCommissionRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = service
        .set_shop_commission(req.into_inner(), admin.user_id)
        .await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

/// GET /admin/api/wallet/logs - Get admin operation logs
#[utoipa::path(
    get,
    path = "/admin/api/wallet/logs",
    tag = "Wallet - Admin",
    description = "Get admin operation logs for audit trail. Supports filtering by target ID.",
    params(
        ("target_id" = Option<String>, Query, description = "Filter by target ID"),
        ("limit" = Option<i64>, Query, description = "Limit number of results")
    ),
    responses(
        (status = 200, description = "Admin logs retrieved", body = ApiResponse<serde_json::Value>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin only"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_admin_logs(
    service: web::Data<Arc<WalletService>>,
    _admin: AdminUser,
    query: web::Query<AdminLogQuery>,
) -> Result<HttpResponse, ApiError> {
    let response = service
        .get_admin_logs(query.target_id.clone(), query.limit)
        .await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

// ========================================================================
// TRANSACTION HISTORY
// ========================================================================

/// GET /api/wallet/transactions - Get transaction history
#[utoipa::path(
    get,
    path = "/api/wallet/transactions",
    tag = "Wallet - User",
    description = "Get transaction history for the authenticated user's wallet. Supports pagination and filtering.",
    params(
        ("page" = Option<i64>, Query, description = "Page number"),
        ("page_size" = Option<i64>, Query, description = "Page size"),
        ("tx_type" = Option<String>, Query, description = "Transaction type filter"),
        ("status" = Option<String>, Query, description = "Status filter"),
        ("start_date" = Option<String>, Query, description = "Start date (ISO format)"),
        ("end_date" = Option<String>, Query, description = "End date (ISO format)")
    ),
    responses(
        (status = 200, description = "Transaction history retrieved", body = ApiResponse<TransactionListResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_transaction_history(
    service: web::Data<Arc<WalletService>>,
    auth: AuthUser,
    query: web::Query<TransactionHistoryQuery>,
) -> Result<HttpResponse, ApiError> {
    let page_size = query.page_size.unwrap_or(20);
    let response = service
        .get_transaction_history(&auth.wallet_id, page_size)
        .await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

// ========================================================================
// BACKGROUND JOB ENDPOINTS (Internal/Cron)
// ========================================================================

/// POST /admin/api/wallet/escrow/jobs/auto-release - Process auto-release escrows (cron job)
#[utoipa::path(
    post,
    path = "/admin/api/wallet/escrow/jobs/auto-release",
    tag = "Wallet - Admin",
    description = "Background job (cron) to auto-release escrows after 72 hours. Releases funds to seller when no disputes are created within the dispute period.",
    responses(
        (status = 200, description = "Auto-release processed", body = ApiResponse<ProcessAutoReleaseResponse>),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn process_auto_releases(
    service: web::Data<Arc<WalletService>>,
) -> Result<HttpResponse, ApiError> {
    let response = service.process_auto_releases().await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

// ========================================================================
// ADMIN DASHBOARD & RECONCILIATION
// ========================================================================

/// GET /admin/api/wallet/dashboard - Get admin dashboard stats
#[utoipa::path(
    get,
    path = "/admin/api/wallet/dashboard",
    tag = "Wallet - Admin",
    description = "Get wallet dashboard statistics for admin monitoring and analytics. Includes total wallets, pending escrows, dispute counts, and more.",
    responses(
        (status = 200, description = "Dashboard stats retrieved", body = ApiResponse<DashboardStatsResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - admin only"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[protect("ADMIN")]
pub async fn get_dashboard_stats(
    service: web::Data<Arc<WalletService>>,
) -> Result<HttpResponse, ApiError> {
    let stats = service.get_dashboard_stats().await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(stats)))
}

/// POST /admin/api/wallet/reconcile - Trigger daily reconciliation
#[utoipa::path(
    post,
    path = "/admin/api/wallet/reconcile",
    tag = "Wallet - Admin",
    description = "Trigger daily wallet reconciliation to verify balance integrity and detect discrepancies between wallet balances and transaction history.",
    responses(
        (status = 200, description = "Reconciliation completed", body = ApiResponse<ReconciliationResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - admin only"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[protect("ADMIN")]
pub async fn trigger_reconciliation(
    service: web::Data<Arc<WalletService>>,
) -> Result<HttpResponse, ApiError> {
    let result = service.daily_reconciliation().await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(result)))
}

/// POST /admin/api/wallet/cron/start - Start wallet cron jobs
#[utoipa::path(
    post,
    path = "/admin/api/wallet/cron/start",
    tag = "Wallet - Admin",
    description = "Start wallet background jobs (escrow auto-release, reconciliation, USDT monitoring, auto-escalate)",
    responses(
        (status = 200, description = "Cron jobs started", body = ApiResponse<SuccessResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - admin only"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[protect("ADMIN")]
pub async fn start_cron_jobs(
    cron_manager: web::Data<WalletCronManager>,
) -> Result<HttpResponse, ApiError> {
    cron_manager.start();
    Ok(HttpResponse::Ok().json(ApiResponse::success(SuccessResponse::new("Cron jobs started successfully"))))
}

/// POST /admin/api/wallet/cron/stop - Stop wallet cron jobs
#[utoipa::path(
    post,
    path = "/admin/api/wallet/cron/stop",
    tag = "Wallet - Admin",
    description = "Stop wallet background jobs (escrow auto-release, reconciliation, USDT monitoring, auto-escalate)",
    responses(
        (status = 200, description = "Cron jobs stopped", body = ApiResponse<SuccessResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - admin only"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[protect("ADMIN")]
pub async fn stop_cron_jobs(
    cron_manager: web::Data<WalletCronManager>,
) -> Result<HttpResponse, ApiError> {
    cron_manager.stop();
    Ok(HttpResponse::Ok().json(ApiResponse::success(SuccessResponse::new("Cron jobs stop requested"))))
}

// ============================================================================
// DISPUTE SYSTEM V2 - Enhanced Dispute Handlers
// ============================================================================

/// GET /api/wallet/disputes - List disputes for current user
#[utoipa::path(
    get,
    path = "/api/wallet/disputes",
    tag = "Wallet - User",
    description = "List all disputes for the current user (as buyer or seller). Supports pagination and filtering.",
    params(
        ("page" = Option<i64>, Query, description = "Page number (default: 1)"),
        ("per_page" = Option<i64>, Query, description = "Items per page (default: 20, max: 100)"),
        ("status" = Option<String>, Query, description = "Filter by status (PENDING, SELLER_RESPONDED, BUYER_RESPONDED, ESCALATED, RESOLVED, REFUNDED, PARTIAL_REFUND, REJECTED, CLOSED, EXTENDED)"),
        ("order_id" = Option<String>, Query, description = "Filter by order ID")
    ),
    responses(
        (status = 200, description = "Disputes list retrieved", body = ApiResponse<DisputeListResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_disputes_list(
    service: web::Data<Arc<WalletService>>,
    auth: AuthUser,
    query: web::Query<DisputeListQuery>,
) -> Result<HttpResponse, ApiError> {
    let response = service
        .get_disputes_list(&auth.user_id, query.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

/// GET /api/wallet/disputes/{dispute_id} - Get dispute detail
#[utoipa::path(
    get,
    path = "/api/wallet/disputes/{dispute_id}",
    tag = "Wallet - User",
    description = "Get detailed dispute information by ID. Both buyer and seller can view their disputes.",
    params(
        ("dispute_id" = String, Path, description = "Dispute ID (DSP-xxx)")
    ),
    responses(
        (status = 200, description = "Dispute detail retrieved", body = ApiResponse<DisputeInfoResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - not your dispute"),
        (status = 404, description = "Dispute not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_dispute_detail(
    service: web::Data<Arc<WalletService>>,
    _auth: AuthUser,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let dispute_id = path.into_inner();
    let response = service.get_dispute_detail(&dispute_id).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

/// POST /api/wallet/disputes/{dispute_id}/seller/respond - Seller responds to dispute
#[utoipa::path(
    post,
    path = "/api/wallet/disputes/{dispute_id}/seller/respond",
    tag = "Wallet - User",
    description = "Seller responds to buyer dispute with one of 4 actions: ACCEPT (full refund), PARTIAL_ACCEPT (offer partial refund), REJECT (dispute), REPLACEMENT (offer new items). Seller must respond within 48 hours.",
    params(
        ("dispute_id" = String, Path, description = "Dispute ID (DSP-xxx)")
    ),
    request_body = SellerDisputeResponseRequest,
    responses(
        (status = 200, description = "Seller response recorded", body = ApiResponse<DisputeInfoResponse>),
        (status = 400, description = "Bad request - invalid action, amount, or images"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - not your dispute"),
        (status = 404, description = "Dispute not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn seller_respond_dispute(
    service: web::Data<Arc<WalletService>>,
    auth: AuthUser,
    path: web::Path<String>,
    req: web::Json<SellerDisputeResponseRequest>,
) -> Result<HttpResponse, ApiError> {
    let dispute_id = path.into_inner();
    let response = service
        .seller_respond_dispute(&dispute_id, auth.user_id, req.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

/// POST /api/wallet/disputes/{dispute_id}/buyer/respond - Buyer responds to seller
#[utoipa::path(
    post,
    path = "/api/wallet/disputes/{dispute_id}/buyer/respond",
    tag = "Wallet - User",
    description = "Buyer responds to seller's action. Options: ACCEPT_OFFER (accept seller's partial/full refund offer) or ESCALATE (escalate to admin). Buyer must respond within 24 hours of seller response. Max 3 exchange rounds.",
    params(
        ("dispute_id" = String, Path, description = "Dispute ID (DSP-xxx)")
    ),
    request_body = BuyerDisputeResponseRequest,
    responses(
        (status = 200, description = "Buyer response recorded", body = ApiResponse<DisputeInfoResponse>),
        (status = 400, description = "Bad request - invalid decision or escalation message"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - not your dispute"),
        (status = 404, description = "Dispute not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn buyer_respond_dispute(
    service: web::Data<Arc<WalletService>>,
    auth: AuthUser,
    path: web::Path<String>,
    req: web::Json<BuyerDisputeResponseRequest>,
) -> Result<HttpResponse, ApiError> {
    let dispute_id = path.into_inner();
    let response = service
        .buyer_respond_dispute(&dispute_id, auth.user_id, req.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

/// POST /admin/api/wallet/disputes/{dispute_id}/extend - Admin extends dispute deadline
#[utoipa::path(
    post,
    path = "/admin/api/wallet/disputes/{dispute_id}/extend",
    tag = "Wallet - Admin",
    description = "Admin extends dispute deadline by 1-7 days. Used when more time is needed for resolution.",
    params(
        ("dispute_id" = String, Path, description = "Dispute ID (DSP-xxx)")
    ),
    request_body = AdminExtendDeadlineRequest,
    responses(
        (status = 200, description = "Deadline extended", body = ApiResponse<DisputeInfoResponse>),
        (status = 400, description = "Bad request - invalid extension days"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin only"),
        (status = 404, description = "Dispute not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[protect("ADMIN")]
pub async fn admin_extend_deadline(
    service: web::Data<Arc<WalletService>>,
    auth: AuthUser,
    path: web::Path<String>,
    req: web::Json<AdminExtendDeadlineRequest>,
) -> Result<HttpResponse, ApiError> {
    let _dispute_id = path.into_inner(); // Not used, req.dispute_id is used instead
    let response = service
        .admin_extend_deadline(auth.user_id, req.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

/// POST /admin/api/wallet/disputes/partial-refund - Admin processes partial refund
#[utoipa::path(
    post,
    path = "/admin/api/wallet/disputes/partial-refund",
    tag = "Wallet - Admin",
    description = "Admin processes partial refund with buyer/seller percentage split. Commission is deducted from seller's portion (5%). Example: buyer_percent=70 means buyer gets 70%, seller gets 25% (after 5% commission).",
    request_body = AdminPartialRefundRequest,
    responses(
        (status = 200, description = "Partial refund processed", body = ApiResponse<DisputeInfoResponse>),
        (status = 400, description = "Bad request - invalid percentage"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin only"),
        (status = 404, description = "Dispute not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[protect("ADMIN")]
pub async fn admin_partial_refund(
    service: web::Data<Arc<WalletService>>,
    auth: AuthUser,
    req: web::Json<AdminPartialRefundRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = service
        .admin_partial_refund(auth.user_id, req.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

/// POST /admin/api/wallet/disputes/jobs/auto-escalate - Process auto-escalate (cron job)
#[utoipa::path(
    post,
    path = "/admin/api/wallet/disputes/jobs/auto-escalate",
    tag = "Wallet - Admin",
    description = "Background job (cron) to process auto-escalate and auto-resolve disputes. Runs every 30 minutes. Handles: (1) Seller no response within 48h → auto-escalate, (2) Buyer no response within 24h → auto-escalate OR auto-resolve if seller accepted.",
    responses(
        (status = 200, description = "Auto-escalate processed", body = ApiResponse<ProcessAutoEscalateResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin or internal service only"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn process_auto_escalate(
    service: web::Data<Arc<WalletService>>,
) -> Result<HttpResponse, ApiError> {
    let response = service.process_auto_escalate().await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}
