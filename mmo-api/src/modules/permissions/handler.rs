//! Permission handlers
//!
//! HTTP handlers for permission and role management APIs.

use actix_web::{web, HttpResponse};
use std::sync::Arc;

use super::service::PermissionService;
use super::dto::*;
use crate::core::{ApiError, ApiResponse};

/// List all permissions
#[utoipa::path(
    get,
    path = "/api/v1/permissions",
    tag = "Permissions",
    responses(
        (status = 200, description = "List of permissions", body = PermissionResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_permissions(
    service: web::Data<Arc<PermissionService>>,
) -> Result<HttpResponse, ApiError> {
    let permissions = service.list_permissions().await?;
    let response: Vec<PermissionResponse> = permissions
        .into_iter()
        .map(|p| PermissionResponse {
            id: p.id.map(|id| id.to_hex()).unwrap_or_default(),
            name: p.name,
            display_name: p.display_name,
            description: p.description,
            resource: p.resource,
            action: p.action,
            category: p.category,
            is_active: p.is_active,
        })
        .collect();

    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}
