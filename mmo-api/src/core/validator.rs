//! Input validation utilities
//!
//! Provides custom validators and validation helpers for request validation.

use validator::ValidationError;

/// Validates MongoDB ObjectId format
///
/// # Arguments
/// * `value` - String to validate as ObjectId
///
/// # Returns
/// * `Result<(), ValidationError>` - Ok if valid, Err otherwise
///
/// # Examples
/// ```
/// #[derive(Validate)]
/// struct Request {
///     #[validate(custom = "validate_object_id")]
///     user_id: String,
/// }
/// ```
pub fn validate_object_id(value: &str) -> Result<(), ValidationError> {
    if value.len() != 24 {
        return Err(ValidationError::new("invalid_object_id"));
    }

    if !value.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ValidationError::new("invalid_object_id"));
    }

    Ok(())
}

/// Validates password strength
///
/// Requirements:
/// - Minimum 8 characters
/// - At least one uppercase letter
/// - At least one lowercase letter
/// - At least one number
///
/// # Arguments
/// * `password` - Password to validate
///
/// # Examples
/// ```
/// #[derive(Validate)]
/// struct RegisterRequest {
///     #[validate(custom = "validate_password_strength")]
///     password: String,
/// }
/// ```
pub fn validate_password_strength(password: &str) -> Result<(), ValidationError> {
    let mut has_upper = false;
    let mut has_lower = false;
    let mut has_digit = false;

    if password.len() < 8 {
        return Err(ValidationError::new("password_too_short"));
    }

    for c in password.chars() {
        if c.is_uppercase() {
            has_upper = true;
        }
        if c.is_lowercase() {
            has_lower = true;
        }
        if c.is_ascii_digit() {
            has_digit = true;
        }
    }

    if !has_upper {
        return Err(ValidationError::new("password_missing_uppercase"));
    }
    if !has_lower {
        return Err(ValidationError::new("password_missing_lowercase"));
    }
    if !has_digit {
        return Err(ValidationError::new("password_missing_digit"));
    }

    Ok(())
}

/// Validates amount is positive
///
/// # Arguments
/// * `amount` - Amount to validate
pub fn validate_positive_amount(amount: &i64) -> Result<(), ValidationError> {
    if *amount <= 0 {
        return Err(ValidationError::new("amount_must_be_positive"));
    }
    Ok(())
}

/// Validates email format (additional to validator's email validator)
///
/// # Arguments
/// * `email` - Email to validate
pub fn validate_email_domain(email: &str) -> Result<(), ValidationError> {
    // Additional custom email validation if needed
    // For example, blocking certain domains
    let blocked_domains = ["tempmail.com", "throwaway.email"];

    if let Some(domain) = email.split('@').nth(1) {
        if blocked_domains.contains(&domain) {
            return Err(ValidationError::new("email_domain_not_allowed"));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_object_id() {
        // Valid ObjectId
        assert!(validate_object_id("507f1f77bcf86cd799439011").is_ok());

        // Invalid length
        assert!(validate_object_id("123").is_err());

        // Invalid characters
        assert!(validate_object_id("507f1f77bcf86cd79943901z").is_err());
    }

    #[test]
    fn test_validate_password_strength() {
        // Valid password
        assert!(validate_password_strength("Password123").is_ok());

        // Too short
        assert!(validate_password_strength("Pass1").is_err());

        // Missing uppercase
        assert!(validate_password_strength("password123").is_err());

        // Missing lowercase
        assert!(validate_password_strength("PASSWORD123").is_err());

        // Missing digit
        assert!(validate_password_strength("Password").is_err());
    }

    #[test]
    fn test_validate_positive_amount() {
        assert!(validate_positive_amount(&100).is_ok());
        assert!(validate_positive_amount(&0).is_err());
        assert!(validate_positive_amount(&-100).is_err());
    }
}
