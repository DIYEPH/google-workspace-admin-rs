//! Error types for the Google Workspace Admin SDK.

use serde::Deserialize;
use thiserror::Error;

/// Convenience result alias for SDK operations.
pub type Result<T> = std::result::Result<T, AdminError>;

/// Structured error enum returned by all SDK methods.
#[derive(Debug, Error)]
pub enum AdminError {
    /// Invalid or incomplete client configuration.
    #[error("configuration error: {0}")]
    Config(String),

    /// Failed to read or parse the Service Account JSON key.
    #[error("service account key error: {0}")]
    ServiceAccountKey(String),

    /// OAuth2 JWT signing or token exchange failure.
    #[error("authentication error: {0}")]
    Auth(String),

    /// HTTP transport or network failure.
    #[error("http transport error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON serialization/deserialization failure.
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    /// Target user, group, or OU was not found (HTTP 404).
    #[error("resource not found: {0}")]
    NotFound(String),

    /// Resource already exists / duplicate entity (HTTP 409).
    #[error("resource already exists: {0}")]
    AlreadyExists(String),

    /// Permission denied or insufficient OAuth scope / DWD authorization (HTTP 403).
    #[error("permission denied ({reason}): {message}")]
    PermissionDenied { reason: String, message: String },

    /// Quota or rate limit exceeded even after retries (HTTP 429 / 403 rateLimitExceeded).
    #[error("rate limit exceeded ({reason}): {message}")]
    RateLimited { reason: String, message: String },

    /// Any other Google API error response.
    #[error("google api error (HTTP {status}, {reason}): {message}")]
    Api {
        status: u16,
        reason: String,
        message: String,
    },
}

#[derive(Debug, Deserialize)]
struct GoogleErrorEnvelope {
    error: Option<GoogleErrorBody>,
}

#[derive(Debug, Deserialize)]
struct GoogleErrorBody {
    message: Option<String>,
    errors: Option<Vec<GoogleErrorDetail>>,
}

#[derive(Debug, Deserialize)]
struct GoogleErrorDetail {
    reason: Option<String>,
}

const RETRYABLE_403_REASONS: &[&str] = &[
    "rateLimitExceeded",
    "userRateLimitExceeded",
    "quotaExceeded",
    "backendError",
];

impl AdminError {
    /// Parse a Google API error HTTP status and body into a typed [`AdminError`].
    pub(crate) fn from_google_response(status: u16, body: &str) -> Self {
        let (reason, message) = serde_json::from_str::<GoogleErrorEnvelope>(body)
            .ok()
            .and_then(|env| env.error)
            .map(|err| {
                let reason = err
                    .errors
                    .and_then(|v| v.into_iter().next())
                    .and_then(|d| d.reason)
                    .unwrap_or_default();
                let msg = err.message.unwrap_or_else(|| body.to_string());
                (reason, msg)
            })
            .unwrap_or_else(|| (String::new(), body.to_string()));

        match status {
            404 => AdminError::NotFound(message),
            409 => AdminError::AlreadyExists(message),
            429 => AdminError::RateLimited {
                reason: if reason.is_empty() {
                    "rateLimitExceeded".to_string()
                } else {
                    reason
                },
                message,
            },
            403 if RETRYABLE_403_REASONS.contains(&reason.as_str()) => {
                AdminError::RateLimited { reason, message }
            }
            403 => AdminError::PermissionDenied { reason, message },
            _ if reason == "duplicate" => AdminError::AlreadyExists(message),
            _ => AdminError::Api {
                status,
                reason,
                message,
            },
        }
    }

    /// Returns `true` if this error is transient and safe to retry with backoff.
    pub fn is_retryable(&self) -> bool {
        match self {
            AdminError::RateLimited { .. } => true,
            AdminError::Api { status, .. } => matches!(*status, 500 | 502 | 503 | 504),
            AdminError::Http(err) => err.is_timeout() || err.is_connect(),
            _ => false,
        }
    }
}
