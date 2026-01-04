//! Date and time utilities
//!
//! Helper functions for working with dates and times.

use bson::DateTime as BsonDateTime;
use chrono::{DateTime, Duration, Utc};

/// Gets the current UTC time as BSON DateTime
///
/// # Returns
/// * `BsonDateTime` - Current time in BSON format
///
/// # Examples
/// ```
/// let now = now_bson();
/// ```
pub fn now_bson() -> BsonDateTime {
    BsonDateTime::now()
}

/// Converts chrono DateTime to BSON DateTime
///
/// # Arguments
/// * `dt` - Chrono DateTime
///
/// # Returns
/// * `BsonDateTime` - BSON DateTime
pub fn to_bson_datetime(dt: DateTime<Utc>) -> BsonDateTime {
    BsonDateTime::from_millis(dt.timestamp_millis())
}

/// Adds days to current time
///
/// # Arguments
/// * `days` - Number of days to add
///
/// # Returns
/// * `BsonDateTime` - Future date
///
/// # Examples
/// ```
/// let release_date = add_days(3); // 3 days from now
/// ```
pub fn add_days(days: i64) -> BsonDateTime {
    let future = Utc::now() + Duration::days(days);
    to_bson_datetime(future)
}

/// Adds hours to current time
///
/// # Arguments
/// * `hours` - Number of hours to add
///
/// # Returns
/// * `BsonDateTime` - Future date
pub fn add_hours(hours: i64) -> BsonDateTime {
    let future = Utc::now() + Duration::hours(hours);
    to_bson_datetime(future)
}

/// Adds minutes to current time
///
/// # Arguments
/// * `minutes` - Number of minutes to add
///
/// # Returns
/// * `BsonDateTime` - Future date
pub fn add_minutes(minutes: i64) -> BsonDateTime {
    let future = Utc::now() + Duration::minutes(minutes);
    to_bson_datetime(future)
}

/// Formats DateTime for display
///
/// # Arguments
/// * `dt` - BSON DateTime
///
/// # Returns
/// * `String` - Formatted date string
///
/// # Examples
/// ```
/// let formatted = format_datetime(&datetime);
/// // Output: "2025-01-30 10:30:45 UTC"
/// ```
pub fn format_datetime(dt: &BsonDateTime) -> String {
    let chrono_dt = dt.to_chrono();
    chrono_dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

/// Checks if a date is in the past
///
/// # Arguments
/// * `dt` - BSON DateTime to check
///
/// # Returns
/// * `bool` - True if date is in the past
pub fn is_past(dt: &BsonDateTime) -> bool {
    dt.timestamp_millis() < Utc::now().timestamp_millis()
}

/// Checks if a date is in the future
///
/// # Arguments
/// * `dt` - BSON DateTime to check
///
/// # Returns
/// * `bool` - True if date is in the future
pub fn is_future(dt: &BsonDateTime) -> bool {
    dt.timestamp_millis() > Utc::now().timestamp_millis()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_days() {
        let future = add_days(7);
        assert!(is_future(&future));
    }

    #[test]
    fn test_is_past_and_future() {
        let past = to_bson_datetime(Utc::now() - Duration::days(1));
        let future = add_days(1);

        assert!(is_past(&past));
        assert!(is_future(&future));
    }
}
