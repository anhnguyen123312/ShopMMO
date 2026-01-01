//! Permission and Role repository
//!
//! Database operations for permissions and roles.

use mongodb::{
    bson::{doc, oid::ObjectId, DateTime},
    Collection, Database,
};
use super::domain::*;

/// Repository errors
pub type DbError = crate::core::ApiError;

/// Permission repository
pub struct PermissionRepository {
    collection: Collection<Permission>,
}

impl PermissionRepository {
    pub fn new(db: Database) -> Self {
        Self {
            collection: db.collection("permissions"),
        }
    }

    pub async fn create_permission(&self, mut perm: Permission) -> Result<Permission, DbError> {
        let now = DateTime::now();
        perm.created_at = now;
        perm.updated_at = now;

        self.collection
            .insert_one(&perm)
            .await
            .map(|_| perm)
            .map_err(|e| crate::core::ApiError::internal(format!("Failed to create permission: {}", e)))
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Option<Permission>, DbError> {
        self.collection
            .find_one(doc! { "name": name, "is_active": true })
            .await
            .map_err(|e| crate::core::ApiError::internal(format!("Failed to find permission: {}", e)))
    }

    pub async fn list_all(&self) -> Result<Vec<Permission>, DbError> {
        use futures::StreamExt;

        let mut cursor = self.collection
            .find(doc! { "is_active": true })
            .await
            .map_err(|e| crate::core::ApiError::internal(format!("Failed to list permissions: {}", e)))?;

        let mut results = Vec::new();
        while let Some(result) = cursor.next().await {
            match result {
                Ok(item) => results.push(item),
                Err(e) => return Err(crate::core::ApiError::internal(format!("Failed to fetch permission: {}", e))),
            }
        }

        Ok(results)
    }
}

/// Role repository
pub struct RoleRepository {
    collection: Collection<Role>,
    users_collection: Collection<mongodb::bson::Document>,
}

impl RoleRepository {
    pub fn new(db: Database) -> Self {
        Self {
            collection: db.collection("roles"),
            users_collection: db.collection("users"),
        }
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Option<Role>, DbError> {
        self.collection
            .find_one(doc! { "name": name, "is_active": true })
            .await
            .map_err(|e| crate::core::ApiError::internal(format!("Failed to find role: {}", e)))
    }

    pub async fn get_role_permissions(&self, role_name: &str) -> Result<Vec<String>, DbError> {
        let role = self.find_by_name(role_name).await?
            .ok_or_else(|| crate::core::ApiError::not_found("Role not found"))?;

        Ok(role.flattened_permissions)
    }

    pub async fn get_user_permissions(&self, user_id: &str) -> Result<UserPermissions, DbError> {
        use mongodb::bson::Bson;

        let user_doc = self.users_collection
            .find_one(doc! { "_id": user_id })
            .await
            .map_err(|e| crate::core::ApiError::internal(format!("Failed to find user: {}", e)))?
            .ok_or_else(|| crate::core::ApiError::not_found("User not found"))?;

        let empty = Vec::new();
        let roles: Vec<String> = user_doc
            .get_array("roles")
            .unwrap_or(&empty)
            .iter()
            .filter_map(|b| b.as_str().map(String::from))
            .collect();

        let effective_permissions: Vec<String> = user_doc
            .get_array("effective_permissions")
            .unwrap_or(&empty)
            .iter()
            .filter_map(|b| b.as_str().map(String::from))
            .collect();

        let perm_version = user_doc
            .get_i32("perm_version")
            .unwrap_or(0);

        Ok(UserPermissions {
            user_id: user_id.to_string(),
            roles,
            direct_permissions: vec![],
            effective_permissions,
            perm_version,
        })
    }
}
