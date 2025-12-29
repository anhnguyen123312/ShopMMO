//! Wallet handlers
//!
//! HTTP request handlers for wallet endpoints.

use actix_web::{web, HttpResponse};
use std::sync::Arc;

use crate::{
    core::{ApiError, ApiResponse},
    middleware::AuthUser,
};

use super::service::WalletService;

/// Get wallet balance
///
/// GET /api/wallet/balance
pub async fn get_balance(
    service: web::Data<Arc<WalletService>>,
    auth: AuthUser,
) -> Result<HttpResponse, ApiError> {
    let user_id = bson::oid::ObjectId::parse_str(&auth.user_id)
        .map_err(|_| ApiError::bad_request("Invalid user ID"))?;

    let balance = service.get_balance(&user_id).await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(balance)))
}

// TODO: Implement other handlers:
// - transfer_ap
// - request_withdrawal
// - get_transactions
