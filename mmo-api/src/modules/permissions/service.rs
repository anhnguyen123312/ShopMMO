//! Permission service
//!
//! Business logic for permission and role management.

use std::sync::Arc;
use mongodb::Database;

use super::repository::{PermissionRepository, RoleRepository};
use super::domain::*;
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
}
