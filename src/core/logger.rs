//! Logging configuration module
//!
//! Sets up structured logging using tracing and tracing-subscriber.
//! Supports both human-readable and JSON formats.

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initializes the logging system
///
/// Sets up tracing with:
/// - Environment-based log level filtering (RUST_LOG)
/// - Structured logging with context
/// - Optional JSON formatting for production
///
/// # Examples
/// ```
/// init_logger();
/// tracing::info!("Server starting");
/// ```
pub fn init_logger() {
    // Create environment filter
    // Falls back to "info" if RUST_LOG is not set
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,mmo_api=debug"));

    // Determine if we should use JSON format (typically for production)
    let use_json = std::env::var("LOG_FORMAT")
        .map(|v| v.to_lowercase() == "json")
        .unwrap_or(false);

    if use_json {
        // JSON format for production (machine-readable)
        let json_layer = tracing_subscriber::fmt::layer()
            .json()
            .with_current_span(true)
            .with_span_list(true);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(json_layer)
            .init();
    } else {
        // Human-readable format for development
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .init();
    }

    tracing::info!("Logger initialized successfully");
}

/// Logging macros usage examples:
///
/// # Examples
/// ```
/// // Info level
/// tracing::info!("User logged in", user_id = 123);
///
/// // Debug level with structured fields
/// tracing::debug!(
///     user_id = user.id,
///     email = %user.email,
///     "Processing user request"
/// );
///
/// // Error level with error context
/// tracing::error!(
///     error = %err,
///     "Failed to process payment"
/// );
///
/// // Span for tracking request flow
/// let span = tracing::info_span!("process_order", order_id = order.id);
/// let _enter = span.enter();
/// // ... code here will be associated with this span
/// ```
#[allow(dead_code)]
const _LOGGER_USAGE_EXAMPLE: () = ();
