//! OpenAPI/Swagger configuration
//!
//! Generates OpenAPI documentation for the MMO API

use utoipa::OpenApi;

/// OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    info(
        title = "MMO API",
        version = "1.0.0",
        description = "Production-ready Rust API server with JWT authentication, MongoDB, and Redis",
        contact(
            name = "MMO Team",
            email = "support@mmo.example.com"
        ),
        license(
            name = "MIT",
        )
    ),
    servers(
        (url = "http://localhost:8080", description = "Local development server"),
        (url = "https://api.mmo.example.com", description = "Production server")
    ),
    tags(
        (name = "Auth", description = "Authentication endpoints"),
        (name = "Wallet", description = "Wallet management endpoints"),
        (name = "Admin", description = "Admin-only endpoints")
    ),
    paths(
        // Auth endpoints
        crate::modules::auth::handler::register,
        crate::modules::auth::handler::login,
        crate::modules::auth::handler::refresh_token,
        crate::modules::auth::handler::logout,
        crate::modules::auth::handler::get_me,
        crate::modules::auth::handler::change_password,
    ),
    components(
        schemas(
            // Auth schemas
            crate::modules::auth::dto::RegisterRequest,
            crate::modules::auth::dto::LoginRequest,
            crate::modules::auth::dto::RefreshTokenRequest,
            crate::modules::auth::dto::LogoutRequest,
            crate::modules::auth::dto::ChangePasswordRequest,
            crate::modules::auth::dto::AuthResponse,
            crate::modules::auth::dto::UserResponse,
        ),
        securitySchemes(
            ("bearer_auth" = (
                ty = "http",
                scheme = "bearer",
                bearer_format = "JWT",
                description = "JWT access token"
            ))
        )
    )
)]
pub struct ApiDoc;
