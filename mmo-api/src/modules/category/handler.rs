//! Category handlers
//!
//! HTTP request handlers for category endpoints.

use actix_web::{web, HttpResponse};
use actix_web_grants::protect;
use std::sync::Arc;
use validator::Validate;

use crate::{
    core::{ApiError, ApiResponse},
};

use super::{
    dto::{
        CategoryListQuery, CategoryResponse, CategoryTreeResponse, CreateCategoryRequest,
        ReorderCategoriesRequest, UpdateCategoryRequest,
    },
    service::CategoryService,
};

/// Create a new category (admin only)
///
/// POST /api/admin/categories
#[utoipa::path(
    post,
    path = "/api/admin/categories",
    tag = "Categories - Admin",
    security(
        ("bearer_auth" = [])
    ),
    request_body = CreateCategoryRequest,
    responses(
        (status = 201, description = "Category created successfully", body = ApiResponse<CategoryResponse>),
        (status = 400, description = "Validation error", body = ApiError),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Forbidden - Admin only", body = ApiError),
        (status = 409, description = "Category already exists", body = ApiError)
    )
)]
#[protect("ADMIN", "SUPER_ADMIN")]
pub async fn create_category(
    service: web::Data<Arc<CategoryService>>,
    req: web::Json<CreateCategoryRequest>,
) -> Result<HttpResponse, ApiError> {
    req.validate()?;
    let result = service.create_category(req.into_inner()).await?;
    Ok(HttpResponse::Created().json(ApiResponse::success(result)))
}

/// Get a category by ID
///
/// GET /api/categories/{id}
#[utoipa::path(
    get,
    path = "/api/categories/{id}",
    tag = "Categories - Public",
    params(
        ("id" = String, Path, description = "Category ID")
    ),
    responses(
        (status = 200, description = "Category retrieved", body = ApiResponse<CategoryResponse>),
        (status = 404, description = "Category not found", body = ApiError)
    )
)]
pub async fn get_category(
    service: web::Data<Arc<CategoryService>>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let id = path.into_inner();
    let result = service.get_category(&id).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(result)))
}

/// List categories as tree structure
///
/// GET /api/categories/tree
#[utoipa::path(
    get,
    path = "/api/categories/tree",
    tag = "Categories - Public",
    params(
        ("include_inactive" = Option<bool>, Query, description = "Include inactive categories (admin only)")
    ),
    responses(
        (status = 200, description = "Categories retrieved", body = ApiResponse<Vec<CategoryTreeResponse>>)
    )
)]
pub async fn list_categories_tree(
    service: web::Data<Arc<CategoryService>>,
    query: web::Query<CategoryListQuery>,
) -> Result<HttpResponse, ApiError> {
    let result = service.list_categories_tree(query.include_inactive).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(result)))
}

/// Update a category (admin only)
///
/// PUT /api/admin/categories/{id}
#[utoipa::path(
    put,
    path = "/api/admin/categories/{id}",
    tag = "Categories - Admin",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = String, Path, description = "Category ID")
    ),
    request_body = UpdateCategoryRequest,
    responses(
        (status = 200, description = "Category updated", body = ApiResponse<CategoryResponse>),
        (status = 400, description = "Validation error", body = ApiError),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Forbidden - Admin only", body = ApiError),
        (status = 404, description = "Category not found", body = ApiError)
    )
)]
#[protect("ADMIN", "SUPER_ADMIN")]
pub async fn update_category(
    service: web::Data<Arc<CategoryService>>,
    path: web::Path<String>,
    req: web::Json<UpdateCategoryRequest>,
) -> Result<HttpResponse, ApiError> {
    req.validate()?;
    let id = path.into_inner();
    let result = service.update_category(&id, req.into_inner()).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(result)))
}

/// Delete a category (admin only)
///
/// DELETE /api/admin/categories/{id}
#[utoipa::path(
    delete,
    path = "/api/admin/categories/{id}",
    tag = "Categories - Admin",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = String, Path, description = "Category ID")
    ),
    responses(
        (status = 200, description = "Category deleted", body = ApiResponse<String>),
        (status = 400, description = "Cannot delete (has children or products)", body = ApiError),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Forbidden - Admin only", body = ApiError),
        (status = 404, description = "Category not found", body = ApiError)
    )
)]
#[protect("ADMIN", "SUPER_ADMIN")]
pub async fn delete_category(
    service: web::Data<Arc<CategoryService>>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let id = path.into_inner();
    service.delete_category(&id).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success("Category deleted".to_string())))
}

/// Reorder categories (admin only)
///
/// POST /api/admin/categories/reorder
#[utoipa::path(
    post,
    path = "/api/admin/categories/reorder",
    tag = "Categories - Admin",
    security(
        ("bearer_auth" = [])
    ),
    request_body = ReorderCategoriesRequest,
    responses(
        (status = 200, description = "Categories reordered", body = ApiResponse<String>),
        (status = 400, description = "Validation error", body = ApiError),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Forbidden - Admin only", body = ApiError)
    )
)]
#[protect("ADMIN", "SUPER_ADMIN")]
pub async fn reorder_categories(
    service: web::Data<Arc<CategoryService>>,
    req: web::Json<ReorderCategoriesRequest>,
) -> Result<HttpResponse, ApiError> {
    req.validate()?;
    service.reorder_categories(req.into_inner()).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success("Categories reordered".to_string())))
}
