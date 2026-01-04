//! Redis connection and management
//!
//! Handles Redis client initialization for caching and session storage.

use redis::{aio::ConnectionManager, Client};

use crate::config::AppConfig;

/// Redis database wrapper
///
/// Provides connection pooling and async access to Redis.
#[derive(Clone)]
pub struct RedisDB {
    client: Client,
    connection_manager: ConnectionManager,
}

impl RedisDB {
    /// Creates a new Redis connection
    ///
    /// # Arguments
    /// * `config` - Application configuration
    ///
    /// # Returns
    /// * `Result<Self, redis::RedisError>` - Redis instance or error
    ///
    /// # Examples
    /// ```
    /// let config = AppConfig::from_env()?;
    /// let redis = RedisDB::connect(&config).await?;
    /// ```
    pub async fn connect(config: &AppConfig) -> Result<Self, redis::RedisError> {
        tracing::info!(uri = %config.database.redis_uri, "Connecting to Redis");

        // Create client
        let client = Client::open(config.database.redis_uri.as_str())?;

        // Create connection manager for connection pooling
        let connection_manager = ConnectionManager::new(client.clone()).await?;

        // Verify connection with ping
        redis::cmd("PING")
            .query_async::<String>(&mut connection_manager.clone())
            .await?;

        tracing::info!("Successfully connected to Redis");

        Ok(Self {
            client,
            connection_manager,
        })
    }

    /// Gets a connection from the pool
    ///
    /// # Returns
    /// * `ConnectionManager` - Async connection manager
    ///
    /// # Examples
    /// ```
    /// let mut conn = redis.get_connection();
    /// redis::cmd("SET").arg("key").arg("value").query_async(&mut conn).await?;
    /// ```
    pub fn get_connection(&self) -> ConnectionManager {
        self.connection_manager.clone()
    }

    /// Sets a key-value pair with optional expiration
    ///
    /// # Arguments
    /// * `key` - Redis key
    /// * `value` - Value to store (must be serializable)
    /// * `expiration_secs` - Optional expiration in seconds
    ///
    /// # Examples
    /// ```
    /// redis.set("user:123", "John Doe", Some(3600)).await?;
    /// ```
    pub async fn set(
        &self,
        key: &str,
        value: &str,
        expiration_secs: Option<usize>,
    ) -> Result<(), redis::RedisError> {
        let mut conn = self.get_connection();

        match expiration_secs {
            Some(exp) => {
                redis::cmd("SETEX")
                    .arg(key)
                    .arg(exp)
                    .arg(value)
                    .query_async(&mut conn)
                    .await
            }
            None => redis::cmd("SET").arg(key).arg(value).query_async(&mut conn).await,
        }
    }

    /// Gets a value by key
    ///
    /// # Arguments
    /// * `key` - Redis key
    ///
    /// # Returns
    /// * `Result<Option<String>, redis::RedisError>` - Value if exists
    ///
    /// # Examples
    /// ```
    /// if let Some(value) = redis.get("user:123").await? {
    ///     println!("User: {}", value);
    /// }
    /// ```
    pub async fn get(&self, key: &str) -> Result<Option<String>, redis::RedisError> {
        let mut conn = self.get_connection();
        redis::cmd("GET").arg(key).query_async(&mut conn).await
    }

    /// Deletes a key
    ///
    /// # Arguments
    /// * `key` - Redis key to delete
    ///
    /// # Examples
    /// ```
    /// redis.delete("user:123").await?;
    /// ```
    pub async fn delete(&self, key: &str) -> Result<(), redis::RedisError> {
        let mut conn = self.get_connection();
        redis::cmd("DEL").arg(key).query_async(&mut conn).await
    }

    /// Checks if a key exists
    ///
    /// # Arguments
    /// * `key` - Redis key
    ///
    /// # Returns
    /// * `Result<bool, redis::RedisError>` - True if key exists
    pub async fn exists(&self, key: &str) -> Result<bool, redis::RedisError> {
        let mut conn = self.get_connection();
        redis::cmd("EXISTS").arg(key).query_async(&mut conn).await
    }

    /// Sets expiration on a key
    ///
    /// # Arguments
    /// * `key` - Redis key
    /// * `seconds` - Expiration time in seconds
    pub async fn expire(&self, key: &str, seconds: usize) -> Result<(), redis::RedisError> {
        let mut conn = self.get_connection();
        redis::cmd("EXPIRE")
            .arg(key)
            .arg(seconds)
            .query_async(&mut conn)
            .await
    }

    /// Checks Redis health
    ///
    /// # Returns
    /// * `Result<bool, redis::RedisError>` - True if healthy
    pub async fn health_check(&self) -> Result<bool, redis::RedisError> {
        let mut conn = self.get_connection();
        let _: String = redis::cmd("PING").query_async(&mut conn).await?;
        Ok(true)
    }
}

/// Redis key prefixes for different use cases
///
/// Using prefixes helps organize keys and avoid collisions.
pub mod keys {
    /// Session key prefix
    ///
    /// Format: session:{user_id}
    pub fn session(user_id: &str) -> String {
        format!("session:{}", user_id)
    }

    /// Refresh token key prefix
    ///
    /// Format: refresh_token:{token_id}
    pub fn refresh_token(token_id: &str) -> String {
        format!("refresh_token:{}", token_id)
    }

    /// Cache key prefix
    ///
    /// Format: cache:{resource}:{id}
    pub fn cache(resource: &str, id: &str) -> String {
        format!("cache:{}:{}", resource, id)
    }

    /// Rate limit key prefix
    ///
    /// Format: rate_limit:{ip}
    pub fn rate_limit(ip: &str) -> String {
        format!("rate_limit:{}", ip)
    }

    /// OTP key prefix
    ///
    /// Format: otp:{email}
    pub fn otp(email: &str) -> String {
        format!("otp:{}", email)
    }

    /// Telegram verification key prefix
    ///
    /// Format: telegram:verify:{shop_id}
    pub fn telegram_verify(shop_id: &str) -> String {
        format!("telegram:verify:{}", shop_id)
    }

    /// Telegram verification code lookup (reverse mapping)
    ///
    /// Format: telegram:code:{verification_code}
    pub fn telegram_code(verification_code: &str) -> String {
        format!("telegram:code:{}", verification_code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        assert_eq!(keys::session("123"), "session:123");
        assert_eq!(keys::cache("user", "456"), "cache:user:456");
        assert_eq!(keys::rate_limit("127.0.0.1"), "rate_limit:127.0.0.1");
    }
}
