//! MMO API Server
//!
//! Production-ready Rust API server built with actix-web and MongoDB.
//!
//! # Features
//! - JWT Authentication with refresh tokens
//! - Role-based authorization
//! - MongoDB database with connection pooling
//! - Redis for caching and sessions
//! - Structured logging with tracing
//! - CORS support
//! - Request ID tracking
//! - Error handling
//!
//! # Environment Variables
//! See `.env.example` for required configuration.

use actix_web::{middleware::Logger, web, App, HttpServer};
use actix_web_grants::GrantsMiddleware;
use std::sync::Arc;
use tracing_actix_web::TracingLogger;
use utoipa::OpenApi;

mod config;
mod core;
mod database;
mod middleware;
mod modules;
mod utils;
mod openapi;

use config::AppConfig;
use database::{MongoDB, RedisDB};

/// Application state shared across all handlers
struct AppState {
    config: Arc<AppConfig>,
    mongodb: Arc<MongoDB>,
    redis: Arc<RedisDB>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logger
    core::logger::init_logger();

    tracing::info!("Starting MMO API Server");

    // Load configuration
    let config = AppConfig::from_env().expect("Failed to load configuration");
    tracing::info!(
        host = %config.server.host,
        port = %config.server.port,
        environment = %config.server.environment,
        "Configuration loaded"
    );

    // Connect to MongoDB
    let mongodb = MongoDB::connect(&config)
        .await
        .expect("Failed to connect to MongoDB");
    let mongodb = Arc::new(mongodb);

    // Connect to Redis
    let redis = RedisDB::connect(&config)
        .await
        .expect("Failed to connect to Redis");
    let redis = Arc::new(redis);

    // Initialize repositories
    let user_repo = Arc::new(modules::auth::UserRepository::new(mongodb.clone()));
    let token_repo = Arc::new(modules::auth::RefreshTokenRepository::new(mongodb.clone()));
    let wallet_repo = Arc::new(modules::wallet::WalletRepository::new(mongodb.clone()));

    // Initialize services (wallet before auth since auth depends on wallet)
    let wallet_service = Arc::new(modules::wallet::WalletService::new(wallet_repo));
    let auth_service = Arc::new(modules::auth::AuthService::new(
        user_repo,
        token_repo,
        Arc::new(config.clone()),
        Some(wallet_service.clone()),
    ));
    let permission_service = Arc::new(modules::permissions::service::PermissionService::new(
        mongodb.database().clone(),
    ));

    // Server address
    let server_host = config.server.host.clone();
    let server_port = config.server.port;
    let bind_address = format!("{}:{}", server_host, server_port);

    tracing::info!("Starting HTTP server at {}", bind_address);

    // Create HTTP server
    HttpServer::new(move || {
        // Configure CORS
        let cors = middleware::configure_cors(&config);

        App::new()
            // Middleware
            .wrap(TracingLogger::default())
            .wrap(Logger::default())
            .wrap(middleware::RequestId)
            .wrap(cors)
            // V2 Authorization middleware (must come AFTER AuthMiddleware)
            .wrap(GrantsMiddleware::with_extractor(middleware::extract_permissions))
            // App data (dependency injection)
            .app_data(web::Data::new(auth_service.clone()))
            .app_data(web::Data::new(wallet_service.clone()))
            .app_data(web::Data::new(permission_service.clone()))
            .app_data(web::Data::from(Arc::new(config.clone())))
            // Health check endpoint
            .route("/health", web::get().to(health_check))
            // API routes
            .service(
                web::scope("/api")
                    // Public routes
                    .configure(modules::auth::routes::configure)
                    // Protected routes (require authentication)
                    .service(
                        web::scope("")
                            .wrap(middleware::AuthMiddleware::new(config.clone()))
                            .configure(modules::wallet::routes::configure)
                            .configure(modules::permissions::routes::configure),
                    ),
            )
            // Swagger UI
            .service(
                utoipa_swagger_ui::SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", openapi::ApiDoc::openapi()),
            )
    })
    .bind(&bind_address)?
    .run()
    .await
}

/// Health check endpoint
///
/// GET /health
async fn health_check() -> &'static str {
    "OK"
}
