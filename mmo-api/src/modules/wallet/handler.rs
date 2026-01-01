//! Wallet V3 HTTP Handlers
//!
//! Actix-web handlers for all wallet endpoints

use actix_web::{web, HttpResponse};
use std::sync::Arc;

use crate::{
    core::{ApiError, ApiResponse},
    middleware::{AuthUser, AdminUser},
};
use super::{dto::*, service::WalletService};

// ========================================================================
// WALLET MANAGEMENT
// ========================================================================

/// GET /api/wallet/balance - Get wallet balance
pub async fn get_balance(
    service: web::Data<Arc<WalletService>>,
    auth: AuthUser,
) -> Result<HttpResponse, ApiError> {
    let balance = service.get_wallet_balance(&auth.wallet_id).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(balance)))
}

/// POST /api/wallet/create - Create wallet (auto-created on first login usually)
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

/// POST /api/wallet/deposit/auto - Create auto deposit request
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

/// POST /api/wallet/deposit/manual - Manual deposit (admin only)
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
// WITHDRAWAL FLOW
// ========================================================================

/// POST /api/wallet/withdrawal - Create withdrawal request
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

/// GET /api/wallet/withdrawal/:request_id/validate - Validate withdrawal
pub async fn validate_withdrawal(
    service: web::Data<Arc<WalletService>>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let request_id = path.into_inner();
    let validation = service.validate_withdrawal(&request_id).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(validation)))
}

/// POST /api/wallet/withdrawal/:request_id/approve - Approve withdrawal (admin)
pub async fn approve_withdrawal(
    service: web::Data<Arc<WalletService>>,
    admin: AdminUser,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let request_id = path.into_inner();
    let response = service.approve_withdrawal(&request_id, admin.user_id).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

/// POST /api/wallet/withdrawal/:request_id/reject - Reject withdrawal (admin)
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

/// POST /api/wallet/withdrawal/:request_id/complete - Complete bank transfer (admin)
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

/// POST /api/wallet/purchase - Create purchase (buyer pays)
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

/// POST /api/wallet/escrow/:escrow_id/early-release - Early release escrow (buyer)
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

/// POST /api/wallet/escrow/:escrow_id/dispute - Create dispute
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

/// POST /api/wallet/escrow/:escrow_id/resolve/refund - Resolve dispute with refund (admin)
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

/// POST /api/wallet/escrow/:escrow_id/resolve/release - Resolve dispute with release (admin)
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

/// POST /api/wallet/admin/debit - Manual debit (admin)
pub async fn manual_debit(
    service: web::Data<Arc<WalletService>>,
    admin: AdminUser,
    req: web::Json<ManualDebitRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = service.manual_debit(req.into_inner(), admin.user_id).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

/// POST /api/wallet/admin/freeze - Freeze wallet (admin)
pub async fn freeze_wallet(
    service: web::Data<Arc<WalletService>>,
    admin: AdminUser,
    req: web::Json<FreezeWalletRequest>,
) -> Result<HttpResponse, ApiError> {
    let response = service.freeze_wallet(req.into_inner(), admin.user_id).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

/// POST /api/wallet/admin/unfreeze - Unfreeze wallet (admin)
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

/// POST /api/wallet/admin/commission - Set shop commission (admin)
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

/// GET /api/wallet/admin/logs - Get admin operation logs (admin)
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
pub async fn get_transaction_history(
    service: web::Data<Arc<WalletService>>,
    auth: AuthUser,
    query: web::Query<TransactionHistoryQuery>,
) -> Result<HttpResponse, ApiError> {
    let response = service
        .get_transaction_history(&auth.wallet_id, query.page_size)
        .await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

// ========================================================================
// BACKGROUND JOB ENDPOINTS (Internal/Cron)
// ========================================================================

/// POST /api/wallet/jobs/auto-release - Process auto-release escrows (internal)
pub async fn process_auto_releases(
    service: web::Data<Arc<WalletService>>,
) -> Result<HttpResponse, ApiError> {
    let response = service.process_auto_releases().await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}
