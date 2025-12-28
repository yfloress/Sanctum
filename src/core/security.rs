//! Security logging module for Sanctum
//!
//! Provides secure logging functionality for security-relevant events
//! without exposing sensitive information.

use chrono::Utc;
use log::{info, warn};
use std::sync::atomic::{AtomicU64, Ordering};

/// Counter for security events (for correlation)
static EVENT_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Security event types
#[derive(Debug, Clone, Copy)]
pub enum SecurityEvent {
    /// Vault operations
    VaultCreated,
    VaultOpened,
    VaultClosed,
    VaultOpenFailed,

    /// Authentication
    AuthAttemptBlocked,
    RateLimitTriggered,

    /// Data operations
    WalletCreated,
    WalletDeleted,
    TransactionCreated,
    TransactionDeleted,
    CryptoTransactionCreated,

    /// API operations
    ExternalApiRequest,
    ExternalApiRateLimited,
}

impl SecurityEvent {
    /// Returns a string representation of the event type
    fn as_str(&self) -> &'static str {
        match self {
            SecurityEvent::VaultCreated => "VAULT_CREATED",
            SecurityEvent::VaultOpened => "VAULT_OPENED",
            SecurityEvent::VaultClosed => "VAULT_CLOSED",
            SecurityEvent::VaultOpenFailed => "VAULT_OPEN_FAILED",
            SecurityEvent::AuthAttemptBlocked => "AUTH_BLOCKED",
            SecurityEvent::RateLimitTriggered => "RATE_LIMIT",
            SecurityEvent::WalletCreated => "WALLET_CREATED",
            SecurityEvent::WalletDeleted => "WALLET_DELETED",
            SecurityEvent::TransactionCreated => "TX_CREATED",
            SecurityEvent::TransactionDeleted => "TX_DELETED",
            SecurityEvent::CryptoTransactionCreated => "CRYPTO_TX_CREATED",
            SecurityEvent::ExternalApiRequest => "API_REQUEST",
            SecurityEvent::ExternalApiRateLimited => "API_RATE_LIMITED",
        }
    }

    /// Returns the severity level of the event
    fn severity(&self) -> Severity {
        match self {
            SecurityEvent::VaultOpenFailed
            | SecurityEvent::AuthAttemptBlocked
            | SecurityEvent::RateLimitTriggered => Severity::Warning,
            _ => Severity::Info,
        }
    }
}

/// Severity levels for security events
#[derive(Debug, Clone, Copy)]
enum Severity {
    Info,
    Warning,
}

/// Generates a unique event ID for correlation
fn generate_event_id() -> u64 {
    EVENT_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Logs a security event with optional context
///
/// # Arguments
/// * `event` - The type of security event
/// * `context` - Optional context information (must not contain sensitive data)
pub fn log_security_event(event: SecurityEvent, context: Option<&str>) {
    let event_id = generate_event_id();
    let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ");
    let event_type = event.as_str();

    let message = match context {
        Some(ctx) => format!(
            "[SECURITY] id={} ts={} event={} context={}",
            event_id, timestamp, event_type, ctx
        ),
        None => format!(
            "[SECURITY] id={} ts={} event={}",
            event_id, timestamp, event_type
        ),
    };

    match event.severity() {
        Severity::Info => info!("{}", message),
        Severity::Warning => warn!("{}", message),
    }
}

/// Logs a failed authentication attempt
///
/// # Arguments
/// * `attempts` - Number of failed attempts so far
/// * `locked` - Whether the account is now locked
pub fn log_auth_failure(attempts: u32, locked: bool) {
    let event_id = generate_event_id();
    let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ");

    if locked {
        warn!(
            "[SECURITY] id={} ts={} event=AUTH_LOCKED attempts={} action=account_locked",
            event_id, timestamp, attempts
        );
    } else {
        warn!(
            "[SECURITY] id={} ts={} event=AUTH_FAILED attempts={}",
            event_id, timestamp, attempts
        );
    }
}

/// Logs rate limiting events
///
/// # Arguments
/// * `source` - The source being rate limited (e.g., "api", "auth")
/// * `wait_seconds` - Seconds until the limit resets
pub fn log_rate_limit(source: &str, wait_seconds: u64) {
    let event_id = generate_event_id();
    let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ");

    warn!(
        "[SECURITY] id={} ts={} event=RATE_LIMITED source={} wait_seconds={}",
        event_id, timestamp, source, wait_seconds
    );
}

/// Initializes the security logger
///
/// Should be called once at application startup.
pub fn init_security_logger() {
    static INIT: std::sync::Once = std::sync::Once::new();

    INIT.call_once(|| {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
            .format_timestamp_millis()
            .init();

        info!(
            "[SECURITY] id=0 ts={} event=LOGGER_INITIALIZED version={}",
            Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ"),
            env!("CARGO_PKG_VERSION")
        );
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_id_increments() {
        let id1 = generate_event_id();
        let id2 = generate_event_id();
        assert!(id2 > id1);
    }

    #[test]
    fn test_security_event_str() {
        assert_eq!(SecurityEvent::VaultCreated.as_str(), "VAULT_CREATED");
        assert_eq!(SecurityEvent::VaultOpenFailed.as_str(), "VAULT_OPEN_FAILED");
    }

    #[test]
    fn test_severity_levels() {
        assert!(matches!(
            SecurityEvent::VaultOpened.severity(),
            Severity::Info
        ));
        assert!(matches!(
            SecurityEvent::VaultOpenFailed.severity(),
            Severity::Warning
        ));
    }
}
