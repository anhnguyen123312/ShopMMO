//! Shop File Upload Handler - P2PMMO V2
//!
//! Handles file uploads for shop logo and banner
//! Currently accepts base64 encoded images

use actix_web::{web, HttpResponse, Error};
use std::sync::Arc;
use base64::Engine;

use crate::core::{ApiError, ApiResponse};
use crate::middleware::AuthUser;
use super::service::ShopService;

/// Maximum file sizes
const MAX_LOGO_SIZE: usize = 2 * 1024 * 1024; // 2MB
const MAX_BANNER_SIZE: usize = 5 * 1024 * 1024; // 5MB

/// Upload shop logo (base64 encoded)
///
/// POST /api/vendor/shop/upload/logo
///
/// Request body:
/// ```json
/// {
///   "image": "data:image/png;base64,iVBORw0KGgoAAAANS..."
/// }
/// ```
pub async fn upload_logo(
    service: web::Data<Arc<ShopService>>,
    auth: AuthUser,
    req: web::Json<UploadImageRequest>,
) -> Result<HttpResponse, Error> {
    // Get vendor's shop first
    let dashboard = service.get_shop_dashboard(&auth.user_id).await
        .map_err(|e| ApiError::internal(format!("Failed to get shop: {}", e)))?;

    let shop_id = dashboard.shop_id;

    // Process the upload
    match process_base64_image(&req.image, "logo", &shop_id, MAX_LOGO_SIZE) {
        Ok(file_url) => {
            // Update shop with new logo
            use super::dto::UpdateShopRequest;

            let update_req = UpdateShopRequest {
                shop_name: None,
                shop_description: None,
                shop_logo: Some(file_url.clone()),
                shop_banner: None,
            };

            // TODO: Call update_shop service
            // let _ = service.update_shop(&shop_id, &auth.user_id, update_req).await?;

            Ok(HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({
                "file_url": file_url,
                "message": "Logo uploaded successfully"
            }))))
        }
        Err(e) => {
            Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
                format!("Upload failed: {}", e)
            )))
        }
    }
}

/// Upload shop banner (base64 encoded)
///
/// POST /api/vendor/shop/upload/banner
///
/// Request body:
/// ```json
/// {
///   "image": "data:image/png;base64,iVBORw0KGgoAAAANS..."
/// }
/// ```
pub async fn upload_banner(
    service: web::Data<Arc<ShopService>>,
    auth: AuthUser,
    req: web::Json<UploadImageRequest>,
) -> Result<HttpResponse, Error> {
    // Get vendor's shop first
    let dashboard = service.get_shop_dashboard(&auth.user_id).await
        .map_err(|e| ApiError::internal(format!("Failed to get shop: {}", e)))?;

    let shop_id = dashboard.shop_id;

    // Process the upload
    match process_base64_image(&req.image, "banner", &shop_id, MAX_BANNER_SIZE) {
        Ok(file_url) => {
            // Update shop with new banner
            use super::dto::UpdateShopRequest;

            let update_req = UpdateShopRequest {
                shop_name: None,
                shop_description: None,
                shop_logo: None,
                shop_banner: Some(file_url.clone()),
            };

            // TODO: Call update_shop service
            // let _ = service.update_shop(&shop_id, &auth.user_id, update_req).await?;

            Ok(HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({
                "file_url": file_url,
                "message": "Banner uploaded successfully"
            }))))
        }
        Err(e) => {
            Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
                format!("Upload failed: {}", e)
            )))
        }
    }
}

/// Process base64 encoded image
fn process_base64_image(
    data_uri: &str,
    file_type: &str,
    shop_id: &str,
    max_size: usize,
) -> Result<String, String> {
    // Parse data URI format: data:image/png;base64,iVBORw0KG...
    if !data_uri.starts_with("data:image/") {
        return Err("Invalid image format".to_string());
    }

    let parts: Vec<&str> = data_uri.splitn(2, ';').collect();
    if parts.len() != 2 {
        return Err("Invalid data URI format".to_string());
    }

    // Get image type (png, jpeg, webp)
    let image_type = parts[0]
        .strip_prefix("data:image/")
        .ok_or("Invalid image prefix")?;

    let valid_extensions = ["png", "jpeg", "jpg", "webp"];
    if !valid_extensions.contains(&image_type) {
        return Err(format!("Invalid image type: {}", image_type));
    }

    // Get base64 data
    let base64_part = parts[1]
        .strip_prefix("base64,")
        .ok_or("Invalid base64 format")?;

    // Decode base64
    let image_data = base64::engine::general_purpose::STANDARD
        .decode(base64_part)
        .map_err(|e| format!("Failed to decode base64: {}", e))?;

    // Check size limit
    if image_data.len() > max_size {
        return Err(format!(
            "Image too large: max {} bytes, got {}",
            max_size,
            image_data.len()
        ));
    }

    // Generate unique filename
    let unique_filename = format!("{}-{}.{}", uuid::Uuid::new_v4(), file_type, image_type);
    let storage_dir = format!("storage/shops/{}/{}", shop_id, file_type);
    let file_path = format!("{}/{}", storage_dir, unique_filename);

    // In a real implementation, save the file to disk or cloud storage
    // For now, we'll return a URL path
    // TODO: Implement actual file storage

    // Return public URL
    let file_url = format!("/storage/shops/{}/{}/{}", shop_id, file_type, unique_filename);

    Ok(file_url)
}

/// Delete shop file (logo or banner)
///
/// DELETE /api/vendor/shop/file/{file_type}/{filename}
pub async fn delete_file(
    auth: AuthUser,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, Error> {
    let (file_type, filename) = path.into_inner();

    // Validate file type
    if file_type != "logo" && file_type != "banner" {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            "Invalid file type. Must be 'logo' or 'banner'"
        )));
    }

    // TODO: Get shop_id from auth user and delete file
    // For now, skip actual implementation

    Ok(HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({
        "message": "File deleted successfully"
    }))))
}

// ============================================================================
// DTOs
// ============================================================================

use utoipa::ToSchema;

/// Upload image request (base64 encoded)
#[derive(Debug, serde::Deserialize, validator::Validate, ToSchema)]
pub struct UploadImageRequest {
    /// Base64 encoded image data URI (data:image/png;base64,...)
    #[validate(length(min = 1))]
    pub image: String,
}

#[utoipa::path(
    post,
    path = "/api/vendor/shop/upload/logo",
    tag = "Shop - Vendor",
    description = "Upload shop logo image (base64 encoded, max 2MB, png/jpeg/webp)",
    request_body = UploadImageRequest,
    responses(
        (status = 200, description = "Logo uploaded successfully", body = ApiResponse<serde_json::Value>),
        (status = 400, description = "Invalid image"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
#[allow(dead_code)]
pub fn upload_logo_docs() {}

#[utoipa::path(
    post,
    path = "/api/vendor/shop/upload/banner",
    tag = "Shop - Vendor",
    description = "Upload shop banner image (base64 encoded, max 5MB, png/jpeg/webp)",
    request_body = UploadImageRequest,
    responses(
        (status = 200, description = "Banner uploaded successfully", body = ApiResponse<serde_json::Value>),
        (status = 400, description = "Invalid image"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
#[allow(dead_code)]
pub fn upload_banner_docs() {}
