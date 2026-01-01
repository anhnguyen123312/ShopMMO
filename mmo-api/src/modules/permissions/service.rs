//! Permission service
//!
//! Business logic for permission and role management.

use std::sync::Arc;
use mongodb::Database;

use super::repository::{PermissionRepository, RoleRepository};
use super::domain::*;
use super::constants::{all_permissions, is_valid_permission};
use crate::core::ApiError;

/// Permission service
pub struct PermissionService {
    perm_repo: PermissionRepository,
    role_repo: RoleRepository,
}

impl PermissionService {
    pub fn new(db: Database) -> Self {
        Self {
            perm_repo: PermissionRepository::new(db.clone()),
            role_repo: RoleRepository::new(db),
        }
    }

    pub async fn list_permissions(&self) -> Result<Vec<Permission>, ApiError> {
        self.perm_repo.list_all().await
    }

    pub async fn get_user_permissions(&self, user_id: &str) -> Result<UserPermissions, ApiError> {
        self.role_repo.get_user_permissions(user_id).await
    }

    // ========== Role CRUD Operations ==========

    /// Create a new role
    ///
    /// # Arguments
    /// * `name` - Unique role name
    /// * `display_name` - Human-readable name
    /// * `level` - Hierarchy level
    /// * `permissions` - List of permission strings to assign
    ///
    /// # Returns
    /// * `Result<Role, ApiError>` - Created role
    pub async fn create_role(
        &self,
        name: String,
        display_name: String,
        level: i32,
        permissions: Vec<String>,
    ) -> Result<Role, ApiError> {
        // Validate permissions
        for perm in &permissions {
            if !is_valid_permission(perm) {
                return Err(ApiError::bad_request(&format!(
                    "Invalid permission: {}. Valid permissions are: {:?}",
                    perm,
                    all_permissions()
                )));
            }
        }

        // Check if role already exists
        let existing = self
            .role_repo
            .find_by_name(&name)
            .await
            .map_err(|e| ApiError::database(e.to_string()))?;

        if existing.is_some() {
            return Err(ApiError::bad_request(&format!(
                "Role '{}' already exists",
                name
            )));
        }

        let role = Role::new(name.clone(), display_name, level, permissions);

        self.role_repo
            .create(role)
            .await
            .map_err(|e| ApiError::database(e.to_string()))?;

        // Fetch and return the created role
        let created = self
            .role_repo
            .find_by_name(&name)
            .await
            .map_err(|e| ApiError::database(e.to_string()))?
            .ok_or_else(|| ApiError::internal("Failed to retrieve created role".to_string()))?;

        Ok(created)
    }

    /// Update role permissions
    ///
    /// # Arguments
    /// * `role_name` - Role name to update
    /// * `permissions` - New list of permissions
    ///
    /// # Returns
    /// * `Result<(), ApiError>` - Success or error
    pub async fn update_role_permissions(
        &self,
        role_name: &str,
        permissions: Vec<String>,
    ) -> Result<(), ApiError> {
        // Validate permissions
        for perm in &permissions {
            if !is_valid_permission(perm) {
                return Err(ApiError::bad_request(&format!(
                    "Invalid permission: {}",
                    perm
                )));
            }
        }

        self.role_repo
            .update_permissions(role_name, permissions)
            .await
            .map_err(|e| ApiError::database(e.to_string()))?;

        Ok(())
    }

    /// Delete a role
    ///
    /// # Arguments
    /// * `role_name` - Role name to delete
    ///
    /// # Returns
    /// * `Result<(), ApiError>` - Success or error
    pub async fn delete_role(&self, role_name: &str) -> Result<(), ApiError> {
        // Check if role exists
        let role = self
            .role_repo
            .find_by_name(role_name)
            .await
            .map_err(|e| ApiError::database(e.to_string()))?
            .ok_or_else(|| ApiError::not_found(&format!("Role '{}' not found", role_name)))?;

        // Prevent deleting system roles
        if role.is_system {
            return Err(ApiError::bad_request("Cannot delete system roles"));
        }

        self.role_repo
            .delete(role_name)
            .await
            .map_err(|e| ApiError::database(e.to_string()))?;

        Ok(())
    }

    /// List all roles
    ///
    /// # Returns
    /// * `Result<Vec<Role>, ApiError>` - List of roles
    pub async fn list_roles(&self) -> Result<Vec<Role>, ApiError> {
        self.role_repo
            .list_all()
            .await
            .map_err(|e| ApiError::database(e.to_string()))
    }

    /// Assign role to user
    ///
    /// # Arguments
    /// * `user_id` - User's ObjectId as string
    /// * `role_name` - Role name to assign
    ///
    /// # Returns
    /// * `Result<(), ApiError>` - Success or error
    pub async fn assign_role_to_user(
        &self,
        user_id: &str,
        role_name: &str,
    ) -> Result<(), ApiError> {
        // Verify role exists
        let role = self
            .role_repo
            .find_by_name(role_name)
            .await
            .map_err(|e| ApiError::database(e.to_string()))?
            .ok_or_else(|| ApiError::not_found(&format!("Role '{}' not found", role_name)))?;

        // Parse user_id
        let oid = bson::oid::ObjectId::parse_str(user_id)
            .map_err(|_| ApiError::bad_request("Invalid user ID"))?;

        self.role_repo
            .assign_role_to_user(&oid, &role)
            .await
            .map_err(|e| ApiError::database(e.to_string()))?;

        Ok(())
    }

    /// Remove role from user
    ///
    /// # Arguments
    /// * `user_id` - User's ObjectId as string
    /// * `role_name` - Role name to remove
    ///
    /// # Returns
    /// * `Result<(), ApiError>` - Success or error
    pub async fn remove_role_from_user(
        &self,
        user_id: &str,
        role_name: &str,
    ) -> Result<(), ApiError> {
        let oid = bson::oid::ObjectId::parse_str(user_id)
            .map_err(|_| ApiError::bad_request("Invalid user ID"))?;

        self.role_repo
            .remove_role_from_user(&oid, role_name)
            .await
            .map_err(|e| ApiError::database(e.to_string()))?;

        Ok(())
    }
}
