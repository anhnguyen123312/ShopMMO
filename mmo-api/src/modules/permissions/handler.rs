//! Permission handlers
//!
//! HTTP handlers for permission and role management APIs.

use actix_web::{web, HttpResponse};
use std::sync::Arc;
use validator::Validate;

use super::service::PermissionService;
use super::dto::*;
use crate::core::{ApiError, ApiResponse, MessageResponse};

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

// ========== Role Management Handlers ==========

/// Create a new role (admin only)
#[utoipa::path(
    post,
    path = "/api/permissions/roles",
    tag = "Permissions",
    security(("bearer_auth" = [])),
    request_body = CreateRoleRequest,
    responses(
        (status = 201, description = "Role created", body = ApiResponse<RoleResponse>),
        (status = 400, description = "Validation error", body = ApiError),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Forbidden - Admin only", body = ApiError)
    )
)]
pub async fn create_role(
    service: web::Data<Arc<PermissionService>>,
    req: web::Json<CreateRoleRequest>,
) -> Result<HttpResponse, ApiError> {
    req.validate()?;

    let role = service
        .create_role(
            req.name.clone(),
            req.display_name.clone(),
            req.level,
            req.permissions.clone(),
        )
        .await?;

    Ok(HttpResponse::Created().json(ApiResponse::success(RoleResponse::from(role))))
}

/// List all roles
#[utoipa::path(
    get,
    path = "/api/permissions/roles",
    tag = "Permissions",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of roles", body = ApiResponse<Vec<RoleResponse>>),
        (status = 401, description = "Unauthorized", body = ApiError)
    )
)]
pub async fn list_roles(
    service: web::Data<Arc<PermissionService>>,
) -> Result<HttpResponse, ApiError> {
    let roles = service.list_roles().await?;
    let response: Vec<RoleResponse> = roles.into_iter().map(RoleResponse::from).collect();

    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

/// Update role permissions (admin only)
#[utoipa::path(
    put,
    path = "/api/permissions/roles/{role_name}/permissions",
    tag = "Permissions",
    security(("bearer_auth" = [])),
    params(
        ("role_name" = String, Path, description = "Role name")
    ),
    request_body = UpdateRolePermissionsRequest,
    responses(
        (status = 200, description = "Permissions updated", body = ApiResponse<MessageResponse>),
        (status = 400, description = "Validation error", body = ApiError),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Forbidden - Admin only", body = ApiError),
        (status = 404, description = "Role not found", body = ApiError)
    )
)]
pub async fn update_role_permissions(
    service: web::Data<Arc<PermissionService>>,
    path: web::Path<String>,
    req: web::Json<UpdateRolePermissionsRequest>,
) -> Result<HttpResponse, ApiError> {
    let role_name = path.into_inner();
    req.validate()?;

    service
        .update_role_permissions(&role_name, req.permissions.clone())
        .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(MessageResponse::new(
        "Role permissions updated successfully",
    ))))
}

/// Delete a role (admin only)
#[utoipa::path(
    delete,
    path = "/api/permissions/roles/{role_name}",
    tag = "Permissions",
    security(("bearer_auth" = [])),
    params(
        ("role_name" = String, Path, description = "Role name")
    ),
    responses(
        (status = 200, description = "Role deleted", body = ApiResponse<MessageResponse>),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Forbidden - Admin only", body = ApiError),
        (status = 404, description = "Role not found", body = ApiError)
    )
)]
pub async fn delete_role(
    service: web::Data<Arc<PermissionService>>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let role_name = path.into_inner();

    service.delete_role(&role_name).await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(MessageResponse::new(
        "Role deleted successfully",
    ))))
}

/// Assign role to user (admin only)
#[utoipa::path(
    post,
    path = "/api/permissions/roles/assign",
    tag = "Permissions",
    security(("bearer_auth" = [])),
    request_body = AssignUserRoleRequest,
    responses(
        (status = 200, description = "Role assigned", body = ApiResponse<MessageResponse>),
        (status = 400, description = "Validation error", body = ApiError),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Forbidden - Admin only", body = ApiError),
        (status = 404, description = "Role not found", body = ApiError)
    )
)]
pub async fn assign_role(
    service: web::Data<Arc<PermissionService>>,
    req: web::Json<AssignUserRoleRequest>,
) -> Result<HttpResponse, ApiError> {
    req.validate()?;

    service
        .assign_role_to_user(&req.user_id, &req.role_name)
        .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(MessageResponse::new(
        "Role assigned successfully",
    ))))
}

