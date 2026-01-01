//! Permission cache module
//!
//! Redis-based caching for user permissions with version validation.

use redis::{aio::ConnectionManager, Client, AsyncCommands};

/// Permission cache using Redis
///
/// Caches user permissions with version tracking to support
/// efficient permission validation with automatic invalidation.
pub struct PermissionCache {
    client: Client,
}

impl PermissionCache {
    /// Create a new permission cache
    pub async fn new(url: &str) -> Result<Self, redis::RedisError> {
        let client = Client::open(url)?;
        // Test connection by getting a connection manager
        let _conn = ConnectionManager::new(client.clone()).await?;
        Ok(Self { client })
    }

    /// Set user permissions in cache with version
    pub async fn set_permissions(
        &self,
        user_id: &str,
        permissions: &[String],
        version: i32,
    ) -> Result<(), redis::RedisError> {
        let mut conn = ConnectionManager::new(self.client.clone()).await?;

        let key = format!("user:{}:permissions", user_id);
        let _: i32 = conn.del(&key).await?;

        for perm in permissions {
            let _: i32 = conn.sadd(&key, perm).await?;
        }

        let _: i32 = conn.expire(&key, 600).await?; // 10 minutes

        let version_key = format!("user:{}:perm_version", user_id);
        let _: i32 = conn.set(&version_key, version).await?;
        let _: i32 = conn.expire(&version_key, 600).await?;

        Ok(())
    }

    /// Get all user permissions from cache
    pub async fn get_permissions(&self, user_id: &str) -> Result<Vec<String>, redis::RedisError> {
        let mut conn = ConnectionManager::new(self.client.clone()).await?;
        let key = format!("user:{}:permissions", user_id);

        let perms: Vec<String> = conn.smembers(&key).await?;
        Ok(perms)
    }

    /// Check if user has specific permission with version validation
    pub async fn check_permission(
        &self,
        user_id: &str,
        permission: &str,
        jwt_version: i32,
    ) -> Result<bool, redis::RedisError> {
        let mut conn = ConnectionManager::new(self.client.clone()).await?;

        let version_key = format!("user:{}:perm_version", user_id);
        let cached_version: Option<String> = conn.get(&version_key).await?;

        match cached_version {
            Some(v) => {
                let cached: i32 = v.parse().unwrap_or(0);
                if cached != jwt_version {
                    return Ok(false); // Stale cache
                }
            }
            None => return Ok(false), // Cache miss
        }

        let key = format!("user:{}:permissions", user_id);
        let exists: bool = conn.sismember(&key, permission).await?;
        Ok(exists)
    }

    /// Invalidate user permissions cache
    pub async fn invalidate_user(&self, user_id: &str) -> Result<(), redis::RedisError> {
        let mut conn = ConnectionManager::new(self.client.clone()).await?;

        let _: i32 = conn.del(format!("user:{}:permissions", user_id)).await?;
        let _: i32 = conn.del(format!("user:{}:perm_version", user_id)).await?;

        Ok(())
    }

    /// Invalidate role permissions cache
    pub async fn invalidate_role(&self, role_name: &str) -> Result<(), redis::RedisError> {
        let mut conn = ConnectionManager::new(self.client.clone()).await?;
        let _: i32 = conn.del(format!("role:{}:permissions", role_name)).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore] // Requires Redis running
    #[tokio::test]
    async fn test_cache_and_retrieve_permissions() {
        let cache = PermissionCache::new("redis://localhost:6379")
            .await
            .unwrap();

        let permissions = vec![
            "products:read".to_string(),
            "products:create".to_string(),
        ];

        cache.set_permissions("test_user", &permissions, 1)
            .await
            .unwrap();

        let cached = cache.get_permissions("test_user").await.unwrap();
        assert_eq!(cached.len(), 2);
        assert!(cached.contains(&"products:read".to_string()));
    }

    #[ignore] // Requires Redis running
    #[tokio::test]
    async fn test_check_permission_with_version() {
        let cache = PermissionCache::new("redis://localhost:6379")
            .await
            .unwrap();

        cache.set_permissions("user_1", &vec!["products:read".to_string()], 5)
            .await
            .unwrap();

        let result = cache.check_permission("user_1", "products:read", 5).await.unwrap();
        assert_eq!(result, true);

        let result = cache.check_permission("user_1", "products:read", 6).await.unwrap();
        assert_eq!(result, false); // Stale version
    }
}
