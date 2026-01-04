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
use super::domain::{
    WalletType, WalletStatus, TransactionType, Direction, BalanceType,
    TransactionStatus, ReferenceType, WithdrawalStatus, Severity, SnapshotStatus,
    EscrowStatus, ReleaseType, AdminOperation, TargetType, DepositStatus,
    ValidationResult, CheckResult, ValidationError
};

// ========================================================================
// WALLET MANAGEMENT
// ========================================================================

/// GET /internal/wallet/balance - Get wallet balance (internal service call)
#[utoipa::path(
    get,
    path = "/internal/wallet/balance",
    tag = "Wallet - Internal",
    description = "Get wallet balance for internal service-to-service communication",
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

/// POST /internal/wallet/create - Create wallet (internal service call)
#[utoipa::path(
    post,
    path = "/internal/wallet/create",
    tag = "Wallet - Internal",
    description = "Create wallet for internal service-to-service communication",
    request_body = CreateWalletRequest,
    responses(
        (status = 200, description = "Wallet created successfully", body = ApiResponse<WalletInfoResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - insufficient permissions"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[protect("BUYER", "SELLER")]
pub async fn create_wallet(
    service: web::Data<Arc<WalletService>>,
    auth: AuthUser,
    req: web::Json<CreateWalletRequest>,
) -> Result<HttpResponse, ApiError> {
    let wallet = service
        .create_wallet(auth.user_id.clone(), req.wallet_type.clone())
        .await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(wallet)))
}

// ========================================================================
// DEPOSIT FLOW
// ========================================================================

/// POST /internal/wallet/deposit/auto - Create auto deposit request (internal)
#[utoipa::path(
    post,
    path = "/internal/wallet/deposit/auto",
    tag = "Wallet - Internal",
    description = "Create auto deposit request for internal service-to-service communication",
    request_body = AutoDepositRequest,
    responses(
        (status = 200, description = "Deposit request created", body = ApiResponse<DepositResponse>),
        (status = 401, description = "Unauthorized"),
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

/// POST /internal/wallet/deposit/manual - Manual deposit (internal/admin)
#[utoipa::path(
    post,
    path = "/internal/wallet/deposit/manual",
    tag = "Wallet - Internal",
    description = "Manual deposit by admin for internal service-to-service communication",
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

/// POST /api/v3/wallet/deposit/initiate - Initiate deposit via 3rd party payment gateway
#[utoipa::path(
    post,
    path = "/api/v3/wallet/deposit/initiate",
    tag = "User Wallet",
    description = "Initiate a new deposit request through 3rd party payment gateway (bank transfer, Momo, USDT, etc.)",
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

/// POST /api/v3/wallet/deposit/webhook - Payment gateway webhook callback
#[utoipa::path(
    post,
    path = "/api/v3/wallet/deposit/webhook",
    tag = "User Wallet",
    description = "Payment gateway webhook callback to process deposit status updates",
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

/// GET /api/v3/wallet/deposit/status/{tx_id} - Check deposit status
#[utoipa::path(
    get,
    path = "/api/v3/wallet/deposit/status/{tx_id}",
    tag = "User Wallet",
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

/// GET /api/v3/wallet/deposits/history - Get user's deposit history
#[utoipa::path(
    get,
    path = "/api/v3/wallet/deposits/history",
    tag = "User Wallet",
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

/// POST /api/v3/admin/wallets/deposit - Manual deposit by admin
#[utoipa::path(
    post,
    path = "/api/v3/admin/wallets/deposit",
    tag = "Admin - Wallet Management",
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

/// GET /api/v3/admin/wallets/deposits/history - List all deposits (admin)
#[utoipa::path(
    get,
    path = "/api/v3/admin/wallets/deposits/history",
    tag = "Admin - Wallet Management",
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
    admin: AdminUser,
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

/// POST /internal/wallet/withdrawal - Create withdrawal request (internal)
#[utoipa::path(
    post,
    path = "/internal/wallet/withdrawal",
    tag = "Wallet - Internal",
    description = "Create withdrawal request for internal service-to-service communication",
    request_body = WithdrawalRequest,
    responses(
        (status = 200, description = "Withdrawal request created", body = ApiResponse<WithdrawalResponse>),
        (status = 401, description = "Unauthorized"),
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

/// GET /internal/wallet/withdrawal/{request_id}/validate - Validate withdrawal (internal)
#[utoipa::path(
    get,
    path = "/internal/wallet/withdrawal/{request_id}/validate",
    tag = "Wallet - Internal",
    description = "Validate withdrawal request for internal service-to-service communication",
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

/// POST /internal/wallet/withdrawal/{request_id}/approve - Approve withdrawal (internal/admin)
#[utoipa::path(
    post,
    path = "/internal/wallet/withdrawal/{request_id}/approve",
    tag = "Wallet - Internal",
    description = "Approve withdrawal request for internal service-to-service communication",
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

/// POST /internal/wallet/withdrawal/{request_id}/reject - Reject withdrawal (internal/admin)
#[utoipa::path(
    post,
    path = "/internal/wallet/withdrawal/{request_id}/reject",
    tag = "Wallet - Internal",
    description = "Reject withdrawal request for internal service-to-service communication",
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

/// POST /internal/wallet/withdrawal/{request_id}/complete - Complete bank transfer (internal/admin)
#[utoipa::path(
    post,
    path = "/internal/wallet/withdrawal/{request_id}/complete",
    tag = "Wallet - Internal",
    description = "Complete bank transfer for internal service-to-service communication",
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

/// POST /internal/wallet/purchase - Create purchase (internal)
#[utoipa::path(
    post,
    path = "/internal/wallet/purchase",
    tag = "Wallet - Internal",
    description = "Create purchase for internal service-to-service communication",
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

/// POST /internal/wallet/escrow/{escrow_id}/early-release - Early release escrow (internal)
#[utoipa::path(
    post,
    path = "/internal/wallet/escrow/{escrow_id}/early-release",
    tag = "Wallet - Internal",
    description = "Early release escrow for internal service-to-service communication",
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

/// POST /internal/wallet/escrow/{escrow_id}/dispute - Create dispute (internal)
#[utoipa::path(
    post,
    path = "/internal/wallet/escrow/{escrow_id}/dispute",
    tag = "Wallet - Internal",
    description = "Create dispute for internal service-to-service communication",
    params(
        ("escrow_id" = String, Path, description = "Escrow ID")
    ),
    request_body = DisputeRequest,
    responses(
        (status = 200, description = "Dispute created", body = ApiResponse<SuccessResponse>),
        (status = 401, description = "Unauthorized"),
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
    req: web::Json<DisputeRequest>,
) -> Result<HttpResponse, ApiError> {
    let escrow_id = path.into_inner();
    let response = service
        .create_dispute(&escrow_id, req.into_inner(), auth.user_id)
        .await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

/// POST /internal/wallet/escrow/{escrow_id}/resolve/refund - Resolve dispute with refund (internal/admin)
#[utoipa::path(
    post,
    path = "/internal/wallet/escrow/{escrow_id}/resolve/refund",
    tag = "Wallet - Internal",
    description = "Resolve dispute with refund for internal service-to-service communication",
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

/// POST /internal/wallet/escrow/{escrow_id}/resolve/release - Resolve dispute with release (internal/admin)
#[utoipa::path(
    post,
    path = "/internal/wallet/escrow/{escrow_id}/resolve/release",
    tag = "Wallet - Internal",
    description = "Resolve dispute with release for internal service-to-service communication",
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

/// POST /internal/wallet/admin/debit - Manual debit (internal/admin)
#[utoipa::path(
    post,
    path = "/internal/wallet/admin/debit",
    tag = "Wallet - Internal",
    description = "Manual debit by admin for internal service-to-service communication",
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

/// POST /internal/wallet/admin/freeze - Freeze wallet (internal/admin)
#[utoipa::path(
    post,
    path = "/internal/wallet/admin/freeze",
    tag = "Wallet - Internal",
    description = "Freeze wallet by admin for internal service-to-service communication",
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

/// POST /internal/wallet/admin/unfreeze - Unfreeze wallet (internal/admin)
#[utoipa::path(
    post,
    path = "/internal/wallet/admin/unfreeze",
    tag = "Wallet - Internal",
    description = "Unfreeze wallet by admin for internal service-to-service communication",
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

/// POST /internal/wallet/admin/commission - Set shop commission (internal/admin)
#[utoipa::path(
    post,
    path = "/internal/wallet/admin/commission",
    tag = "Wallet - Internal",
    description = "Set shop commission rate by admin for internal service-to-service communication",
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

/// GET /internal/wallet/admin/logs - Get admin operation logs (internal/admin)
#[utoipa::path(
    get,
    path = "/internal/wallet/admin/logs",
    tag = "Wallet - Internal",
    description = "Get admin operation logs for internal service-to-service communication",
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

/// GET /internal/wallet/transactions - Get transaction history (internal)
#[utoipa::path(
    get,
    path = "/internal/wallet/transactions",
    tag = "Wallet - Internal",
    description = "Get transaction history for internal service-to-service communication",
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

/// POST /internal/wallet/jobs/auto-release - Process auto-release escrows (internal/cron)
#[utoipa::path(
    post,
    path = "/internal/wallet/jobs/auto-release",
    tag = "Wallet - Internal",
    description = "Process auto-release escrows for internal service-to-service communication or cron triggers",
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

/// GET /api/v3/admin/wallets/dashboard - Get admin dashboard stats
#[utoipa::path(
    get,
    path = "/api/v3/admin/wallets/dashboard",
    tag = "Admin - Wallet Management",
    description = "Get wallet dashboard statistics for admin monitoring and analytics",
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

/// POST /api/v3/admin/wallets/reconcile - Trigger daily reconciliation
#[utoipa::path(
    post,
    path = "/api/v3/admin/wallets/reconcile",
    tag = "Admin - Wallet Management",
    description = "Trigger daily wallet reconciliation to verify balance integrity and detect discrepancies",
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

/// POST /api/v3/admin/wallets/cron/start - Start wallet cron jobs
#[utoipa::path(
    post,
    path = "/api/v3/admin/wallets/cron/start",
    tag = "Admin - Wallet Management",
    description = "Start wallet background jobs (escrow auto-release, reconciliation, USDT monitoring)",
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

/// POST /api/v3/admin/wallets/cron/stop - Stop wallet cron jobs
#[utoipa::path(
    post,
    path = "/api/v3/admin/wallets/cron/stop",
    tag = "Admin - Wallet Management",
    description = "Stop wallet background jobs (escrow auto-release, reconciliation, USDT monitoring)",
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
