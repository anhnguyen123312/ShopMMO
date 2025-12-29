//! CORS configuration
//!
//! Configures Cross-Origin Resource Sharing for the API.

use actix_cors::Cors;
use actix_web::http;

use crate::config::AppConfig;

/// Creates CORS middleware based on configuration
///
/// # Arguments
/// * `config` - Application configuration
///
/// # Returns
/// * `Cors` - Configured CORS middleware
///
/// # Examples
/// ```
/// let cors = configure_cors(&config);
/// App::new().wrap(cors)
/// ```
pub fn configure_cors(config: &AppConfig) -> Cors {
    let origins: Vec<&str> = config.cors.allowed_origins.split(',').collect();

    let mut cors = Cors::default()
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS"])
        .allowed_headers(vec![
            http::header::AUTHORIZATION,
            http::header::ACCEPT,
            http::header::CONTENT_TYPE,
        ])
        .expose_headers(vec!["X-Request-ID"])
        .max_age(3600);

    // Add allowed origins
    for origin in origins {
        cors = cors.allowed_origin(origin.trim());
    }

    // Allow credentials if configured
    if config.cors.allow_credentials {
        cors = cors.supports_credentials();
    }

    cors
}
