//! Application configuration
//!
//! Loads and manages application settings from environment variables.

use serde::Deserialize;
use std::env;

/// Main application configuration
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    /// Server configuration
    pub server: ServerConfig,

    /// Database configuration
    pub database: DatabaseConfig,

    /// JWT configuration
    pub jwt: JwtConfig,

    /// Security configuration
    pub security: SecurityConfig,

    /// Rate limiting configuration
    pub rate_limit: RateLimitConfig,

    /// CORS configuration
    pub cors: CorsConfig,

    /// Telegram configuration
    pub telegram: TelegramConfig,
}

/// Server configuration
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    /// Server host (e.g., "127.0.0.1")
    pub host: String,

    /// Server port (e.g., 8080)
    pub port: u16,

    /// Environment (development, staging, production)
    pub environment: String,

    /// Number of worker threads for HTTP server
    /// Default: number of logical CPU cores
    pub workers: Option<usize>,

    /// Number of threads for Tokio runtime
    /// Default: number of logical CPU cores
    pub runtime_threads: Option<usize>,
}

/// Database configuration
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    /// MongoDB connection URI
    pub mongodb_uri: String,

    /// MongoDB database name
    pub mongodb_database: String,

    /// MongoDB max pool size
    pub mongodb_max_pool_size: u32,

    /// MongoDB min pool size
    pub mongodb_min_pool_size: u32,

    /// Redis connection URI
    pub redis_uri: String,

    /// Redis pool size
    pub redis_pool_size: u32,
}

/// JWT configuration
#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    /// JWT secret key
    pub secret: String,

    /// Access token expiration (e.g., "15m")
    pub access_token_expires_in: String,

    /// Refresh token expiration (e.g., "7d")
    pub refresh_token_expires_in: String,
}

/// Security configuration
#[derive(Debug, Clone, Deserialize)]
pub struct SecurityConfig {
    /// Bcrypt cost factor
    pub bcrypt_cost: u32,

    /// Minimum password length
    pub password_min_length: usize,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitConfig {
    /// Requests per second
    pub per_second: u64,

    /// Burst size
    pub burst_size: u32,
}

/// CORS configuration
#[derive(Debug, Clone, Deserialize)]
pub struct CorsConfig {
    /// Allowed origins (comma-separated)
    pub allowed_origins: String,

    /// Allow credentials
    pub allow_credentials: bool,
}

/// Telegram configuration
#[derive(Debug, Clone, Deserialize)]
pub struct TelegramConfig {
    /// Bot API key for verification endpoint
    pub bot_api_key: String,
}

impl AppConfig {
    /// Loads configuration from environment variables
    ///
    /// # Returns
    /// * `Result<AppConfig, String>` - Configuration or error message
    ///
    /// # Examples
    /// ```
    /// let config = AppConfig::from_env()?;
    /// println!("Server: {}:{}", config.server.host, config.server.port);
    /// ```
    pub fn from_env() -> Result<Self, String> {
        // Load .env file if exists
        dotenv::dotenv().ok();

        Ok(Self {
            server: ServerConfig {
                host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
                port: env::var("PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()
                    .map_err(|_| "Invalid PORT")?,
                environment: env::var("ENVIRONMENT")
                    .unwrap_or_else(|_| "development".to_string()),
                workers: env::var("SERVER_WORKERS")
                    .ok()
                    .map(|v| v.parse().map_err(|_| "Invalid SERVER_WORKERS"))
                    .transpose()?,
                runtime_threads: env::var("RUNTIME_THREADS")
                    .ok()
                    .map(|v| v.parse().map_err(|_| "Invalid RUNTIME_THREADS"))
                    .transpose()?,
            },

            database: DatabaseConfig {
                mongodb_uri: env::var("MONGODB_URI")
                    .unwrap_or_else(|_| "mongodb://localhost:27017".to_string()),
                mongodb_database: env::var("MONGODB_DATABASE")
                    .map_err(|_| "MONGODB_DATABASE is required")?,
                mongodb_max_pool_size: env::var("MONGODB_MAX_POOL_SIZE")
                    .unwrap_or_else(|_| "100".to_string())
                    .parse()
                    .unwrap_or(100),
                mongodb_min_pool_size: env::var("MONGODB_MIN_POOL_SIZE")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10),
                redis_uri: env::var("REDIS_URI")
                    .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
                redis_pool_size: env::var("REDIS_POOL_SIZE")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10),
            },

            jwt: JwtConfig {
                secret: env::var("JWT_SECRET").map_err(|_| "JWT_SECRET is required")?,
                access_token_expires_in: env::var("JWT_ACCESS_TOKEN_EXPIRES_IN")
                    .unwrap_or_else(|_| "15m".to_string()),
                refresh_token_expires_in: env::var("JWT_REFRESH_TOKEN_EXPIRES_IN")
                    .unwrap_or_else(|_| "7d".to_string()),
            },

            security: SecurityConfig {
                bcrypt_cost: env::var("BCRYPT_COST")
                    .unwrap_or_else(|_| "12".to_string())
                    .parse()
                    .unwrap_or(12),
                password_min_length: env::var("PASSWORD_MIN_LENGTH")
                    .unwrap_or_else(|_| "8".to_string())
                    .parse()
                    .unwrap_or(8),
            },

            rate_limit: RateLimitConfig {
                per_second: env::var("RATE_LIMIT_PER_SECOND")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10),
                burst_size: env::var("RATE_LIMIT_BURST_SIZE")
                    .unwrap_or_else(|_| "20".to_string())
                    .parse()
                    .unwrap_or(20),
            },

            cors: CorsConfig {
                allowed_origins: env::var("CORS_ALLOWED_ORIGINS")
                    .unwrap_or_else(|_| "http://localhost:3000".to_string()),
                allow_credentials: env::var("CORS_ALLOW_CREDENTIALS")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
            },

            telegram: TelegramConfig {
                bot_api_key: env::var("TELEGRAM_BOT_API_KEY")
                    .map_err(|_| "TELEGRAM_BOT_API_KEY is required")?,
            },
        })
    }

    /// Checks if running in production
    pub fn is_production(&self) -> bool {
        self.server.environment.to_lowercase() == "production"
    }

    /// Checks if running in development
    pub fn is_development(&self) -> bool {
        self.server.environment.to_lowercase() == "development"
    }
}
