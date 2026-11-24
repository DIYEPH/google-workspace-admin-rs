//! Google Service Account JWT (RS256) authentication with Domain-Wide Delegation (DWD).

use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use chrono::Utc;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::error::{AdminError, Result};

const DEFAULT_TOKEN_URI: &str = "https://oauth2.googleapis.com/token";
const JWT_GRANT_TYPE: &str = "urn:ietf:params:oauth:grant-type:jwt-bearer";
const TOKEN_REFRESH_MARGIN_SECS: u64 = 120;

/// Parsed contents of a Google Cloud Service Account JSON key file.
#[derive(Debug, Clone, Deserialize)]
pub struct ServiceAccountKey {
    pub client_email: String,
    pub private_key: String,
    pub private_key_id: Option<String>,
    pub project_id: Option<String>,
    pub client_id: Option<String>,
    pub token_uri: Option<String>,
}

impl ServiceAccountKey {
    /// Load and parse a Service Account JSON key from disk.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            AdminError::ServiceAccountKey(format!(
                "failed to read '{}': {}",
                path.as_ref().display(),
                e
            ))
        })?;
        Self::from_json_str(&content)
    }

    /// Parse a Service Account key from a JSON string.
    pub fn from_json_str(json: &str) -> Result<Self> {
        let key: Self = serde_json::from_str(json).map_err(|e| {
            AdminError::ServiceAccountKey(format!("invalid service account JSON: {e}"))
        })?;
        if key.client_email.trim().is_empty() || key.private_key.trim().is_empty() {
            return Err(AdminError::ServiceAccountKey(
                "service account JSON missing client_email or private_key".to_string(),
            ));
        }
        Ok(key)
    }
}

/// Trait abstracting OAuth2 Bearer access token retrieval.
///
/// Implement this trait to plug in custom auth (e.g., Workload Identity Federation,
/// GCP Metadata Server, or mock tokens in tests).
#[async_trait]
pub trait TokenProvider: Send + Sync {
    async fn access_token(&self) -> Result<String>;
}

/// Simple static Bearer token provider (useful for testing or externally managed tokens).
#[derive(Debug, Clone)]
pub struct StaticTokenProvider {
    token: String,
}

impl StaticTokenProvider {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
        }
    }
}

#[async_trait]
impl TokenProvider for StaticTokenProvider {
    async fn access_token(&self) -> Result<String> {
        Ok(self.token.clone())
    }
}

#[derive(Debug, Serialize)]
struct JwtClaims<'a> {
    iss: &'a str,
    sub: &'a str,
    scope: &'a str,
    aud: &'a str,
    exp: i64,
    iat: i64,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: Option<u64>,
}

#[derive(Debug, Clone)]
struct CachedToken {
    access_token: String,
    expires_at: Instant,
}

/// Thread-safe Service Account authenticator supporting Domain-Wide Delegation (DWD).
///
/// Automatically caches the OAuth2 access token and refreshes it 2 minutes before expiry.
pub struct ServiceAccountAuth {
    key: ServiceAccountKey,
    admin_email: String,
    scope: String,
    http: Client,
    cache: Arc<RwLock<Option<CachedToken>>>,
}

impl ServiceAccountAuth {
    /// Create a new DWD authenticator.
    pub fn new(
        key: ServiceAccountKey,
        admin_email: impl Into<String>,
        scopes: &[String],
        http: Client,
    ) -> Result<Self> {
        let admin_email = admin_email.into().trim().to_string();
        if admin_email.is_empty() || !admin_email.contains('@') {
            return Err(AdminError::Config(
                "admin_email must be a valid Google Workspace administrator email".to_string(),
            ));
        }
        if scopes.is_empty() {
            return Err(AdminError::Config(
                "at least one OAuth scope must be configured".to_string(),
            ));
        }
        Ok(Self {
            key,
            admin_email,
            scope: scopes.join(" "),
            http,
            cache: Arc::new(RwLock::new(None)),
        })
    }

    fn token_uri(&self) -> &str {
        self.key
            .token_uri
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or(DEFAULT_TOKEN_URI)
    }

    fn build_signed_jwt(&self) -> Result<String> {
        let now = Utc::now().timestamp();
        let claims = JwtClaims {
            iss: &self.key.client_email,
            sub: &self.admin_email,
            scope: &self.scope,
            aud: self.token_uri(),
            iat: now,
            exp: now + 3600,
        };

        let mut header = Header::new(Algorithm::RS256);
        header.kid = self.key.private_key_id.clone();

        let encoding_key = EncodingKey::from_rsa_pem(self.key.private_key.as_bytes())
            .map_err(|e| AdminError::Auth(format!("invalid RSA private key PEM: {e}")))?;

        encode(&header, &claims, &encoding_key)
            .map_err(|e| AdminError::Auth(format!("failed to sign JWT assertion: {e}")))
    }

    async fn fetch_new_token(&self) -> Result<CachedToken> {
        let assertion = self.build_signed_jwt()?;
        let resp = self
            .http
            .post(self.token_uri())
            .form(&[("grant_type", JWT_GRANT_TYPE), ("assertion", &assertion)])
            .send()
            .await?;

        let status = resp.status();
        let body = resp.text().await?;
        if !status.is_success() {
            return Err(AdminError::Auth(format!(
                "OAuth2 token exchange failed (HTTP {}): {}",
                status.as_u16(),
                body
            )));
        }

        let parsed: TokenResponse = serde_json::from_str(&body)
            .map_err(|e| AdminError::Auth(format!("invalid token response JSON: {e}")))?;

        let ttl_secs = parsed
            .expires_in
            .unwrap_or(3600)
            .saturating_sub(TOKEN_REFRESH_MARGIN_SECS)
            .max(30);

        Ok(CachedToken {
            access_token: parsed.access_token,
            expires_at: Instant::now() + Duration::from_secs(ttl_secs),
        })
    }
}

#[async_trait]
impl TokenProvider for ServiceAccountAuth {
    async fn access_token(&self) -> Result<String> {
        // Fast path: read lock
        {
            let guard = self.cache.read().await;
            if let Some(cached) = guard.as_ref() {
                if Instant::now() < cached.expires_at {
                    return Ok(cached.access_token.clone());
                }
            }
        }

        // Slow path: write lock with double-checked refresh
        let mut guard = self.cache.write().await;
        if let Some(cached) = guard.as_ref() {
            if Instant::now() < cached.expires_at {
                return Ok(cached.access_token.clone());
            }
        }

        let fresh = self.fetch_new_token().await?;
        let token = fresh.access_token.clone();
        *guard = Some(fresh);
        Ok(token)
    }
}
