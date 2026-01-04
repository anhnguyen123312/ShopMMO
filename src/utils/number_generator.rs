//! Number generator utilities
//!
//! Generates unique transaction numbers, order numbers, etc.

use chrono::Utc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Global counter for sequence numbers
static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Generates a transaction number
///
/// Format: TXN-YYYYMMDD-NNNNN
///
/// # Returns
/// * `String` - Unique transaction number
///
/// # Examples
/// ```
/// let txn = generate_transaction_number();
/// // Example: "TXN-20250130-00001"
/// ```
pub fn generate_transaction_number() -> String {
    let date = Utc::now().format("%Y%m%d");
    let seq = next_sequence();
    format!("TXN-{}-{:05}", date, seq)
}

/// Generates an escrow number
///
/// Format: ESC-YYYYMMDD-NNNNN
///
/// # Returns
/// * `String` - Unique escrow number
///
/// # Examples
/// ```
/// let escrow = generate_escrow_number();
/// // Example: "ESC-20250130-00001"
/// ```
pub fn generate_escrow_number() -> String {
    let date = Utc::now().format("%Y%m%d");
    let seq = next_sequence();
    format!("ESC-{}-{:05}", date, seq)
}

/// Generates a withdrawal number
///
/// Format: WTD-YYYYMMDD-NNNNN
///
/// # Returns
/// * `String` - Unique withdrawal number
///
/// # Examples
/// ```
/// let withdrawal = generate_withdrawal_number();
/// // Example: "WTD-20250130-00001"
/// ```
pub fn generate_withdrawal_number() -> String {
    let date = Utc::now().format("%Y%m%d");
    let seq = next_sequence();
    format!("WTD-{}-{:05}", date, seq)
}

/// Generates a deposit number
///
/// Format: DEP-YYYYMMDD-NNNNN
///
/// # Returns
/// * `String` - Unique deposit number
///
/// # Examples
/// ```
/// let deposit = generate_deposit_number();
/// // Example: "DEP-20250130-00001"
/// ```
pub fn generate_deposit_number() -> String {
    let date = Utc::now().format("%Y%m%d");
    let seq = next_sequence();
    format!("DEP-{}-{:05}", date, seq)
}

/// Generates an order number
///
/// Format: ORD-YYYYMMDD-NNNNN
///
/// # Returns
/// * `String` - Unique order number
///
/// # Examples
/// ```
/// let order = generate_order_number();
/// // Example: "ORD-20250130-00001"
/// ```
pub fn generate_order_number() -> String {
    let date = Utc::now().format("%Y%m%d");
    let seq = next_sequence();
    format!("ORD-{}-{:05}", date, seq)
}

/// Gets the next sequence number
///
/// Uses atomic operations for thread-safe increment.
/// Resets daily based on application restart.
///
/// Note: For production, consider using MongoDB's auto-increment
/// or a distributed sequence service for multi-instance deployments.
fn next_sequence() -> u64 {
    COUNTER.fetch_add(1, Ordering::SeqCst) % 100000
}

/// Generates a unique request ID for tracing
///
/// Format: UUID v4
///
/// # Returns
/// * `String` - UUID string
///
/// # Examples
/// ```
/// let request_id = generate_request_id();
/// // Example: "550e8400-e29b-41d4-a716-446655440000"
/// ```
pub fn generate_request_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_number_format() {
        let txn = generate_transaction_number();
        assert!(txn.starts_with("TXN-"));
        assert_eq!(txn.len(), 18); // TXN-YYYYMMDD-NNNNN
    }

    #[test]
    fn test_unique_numbers() {
        let txn1 = generate_transaction_number();
        let txn2 = generate_transaction_number();
        assert_ne!(txn1, txn2);
    }

    #[test]
    fn test_escrow_number_format() {
        let escrow = generate_escrow_number();
        assert!(escrow.starts_with("ESC-"));
    }

    #[test]
    fn test_withdrawal_number_format() {
        let withdrawal = generate_withdrawal_number();
        assert!(withdrawal.starts_with("WTD-"));
    }

    #[test]
    fn test_deposit_number_format() {
        let deposit = generate_deposit_number();
        assert!(deposit.starts_with("DEP-"));
    }

    #[test]
    fn test_order_number_format() {
        let order = generate_order_number();
        assert!(order.starts_with("ORD-"));
    }

    #[test]
    fn test_request_id() {
        let id = generate_request_id();
        assert_eq!(id.len(), 36); // UUID format with hyphens
    }
}
