//! Password hashing utilities
//!
//! Provides secure password hashing and verification using bcrypt.

use bcrypt::{hash, verify, BcryptError, DEFAULT_COST};

/// Hashes a password using bcrypt
///
/// # Arguments
/// * `password` - Plain text password to hash
/// * `cost` - Optional bcrypt cost (default: 12)
///
/// # Returns
/// * `Result<String, BcryptError>` - Hashed password or error
///
/// # Examples
/// ```
/// let hashed = hash_password("MyP@ssw0rd", None)?;
/// ```
pub fn hash_password(password: &str, cost: Option<u32>) -> Result<String, BcryptError> {
    let cost = cost.unwrap_or(DEFAULT_COST);
    hash(password, cost)
}

/// Verifies a password against a hash
///
/// # Arguments
/// * `password` - Plain text password to verify
/// * `hash` - Bcrypt hash to verify against
///
/// # Returns
/// * `Result<bool, BcryptError>` - True if password matches
///
/// # Examples
/// ```
/// if verify_password("MyP@ssw0rd", &user.password_hash)? {
///     println!("Password correct!");
/// }
/// ```
pub fn verify_password(password: &str, hash: &str) -> Result<bool, BcryptError> {
    verify(password, hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify() {
        let password = "TestPassword123";
        let hashed = hash_password(password, Some(4)).unwrap();

        assert!(verify_password(password, &hashed).unwrap());
        assert!(!verify_password("WrongPassword", &hashed).unwrap());
    }
}
