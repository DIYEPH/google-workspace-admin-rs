//! Core HTTP client with automatic OAuth2 token injection and exponential backoff.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use rand::Rng;
use reqwest::{Client, Method, RequestBuilder};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::auth::{ServiceAccountAuth, ServiceAccountKey, StaticTokenProvider, TokenProvider};
use crate::error::{AdminError, Result};
use crate::resources::{GroupsClient, OrgUnitsClient, UsersClient};
use crate::scopes;

/// Default Google Workspace Admin SDK Directory API v1 base URL.
pub const DEFAULT_BASE_URL: &str = "https://admin.googleapis.com/admin/directory/v1";

/// Retry and exponential backoff configuration.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts after the initial request fails.
    pub max_retries: u32,
    /// Initial backoff delay before the first retry.
    pub base_delay: Duration,
    /// Maximum backoff cap per retry.
    pub max_delay: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 5,
            base_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
        }
    }
}

/// Async client for Google Workspace Admin SDK Directory API v1.
///
/// Cloning [`AdminClient`] is cheap (backed by internal `Arc`s and connection pool).
#[derive(Clone)]
pub struct AdminClient {
    http: Client,
    auth: Arc<dyn TokenProvider>,
    base_url: String,
    retry: RetryConfig,
}

impl AdminClient {
    /// Create a new [`AdminClientBuilder`].
    pub fn builder() -> AdminClientBuilder {
        AdminClientBuilder::default()
    }

    /// Access the **Users** resource API (`users.get`, `users.list`, `users.insert`, `users.delete`, `users.undelete`, ...).
    pub fn users(&self) -> UsersClient<'_> {
        UsersClient::new(self)
    }

    /// Access the **Groups & Members** resource API.
    pub fn groups(&self) -> GroupsClient<'_> {
        GroupsClient::new(self)
    }

    /// Access the **Organizational Units (OUs)** resource API.
    pub fn org_units(&self) -> OrgUnitsClient<'_> {
        OrgUnitsClient::new(self)
    }

    pub(crate) fn url(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    pub(crate) async fn request_json<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        query: Option<&[(String, String)]>,
        body: Option<&(impl Serialize + ?Sized)>,
    ) -> Result<T> {
        let text = self.request_raw(method, path, query, body).await?;
        Ok(serde_json::from_str(&text)?)
    }

    pub(crate) async fn request_empty(
        &self,
        method: Method,
        path: &str,
        query: Option<&[(String, String)]>,
        body: Option<&(impl Serialize + ?Sized)>,
    ) -> Result<()> {
        self.request_raw(method, path, query, body).await?;
        Ok(())
    }

    async fn request_raw(
        &self,
        method: Method,
        path: &str,
        query: Option<&[(String, String)]>,
        body: Option<&(impl Serialize + ?Sized)>,
    ) -> Result<String> {
        let url = self.url(path);
        let mut attempt: u32 = 0;

        loop {
            let token = self.auth.access_token().await?;
            let mut req: RequestBuilder =
                self.http.request(method.clone(), &url).bearer_auth(token);

            if let Some(q) = query {
                if !q.is_empty() {
                    req = req.query(q);
                }
            }
            if let Some(b) = body {
                req = req.json(b);
            }

            let outcome = match req.send().await {
                Ok(resp) => {
                    let status = resp.status();
                    let text = resp.text().await?;
                    if status.is_success() {
                        return Ok(text);
                    }
                    Err(AdminError::from_google_response(status.as_u16(), &text))
                }
                Err(err) => Err(AdminError::Http(err)),
            };

            match outcome {
                Ok(text) => return Ok(text),
                Err(err) if err.is_retryable() && attempt < self.retry.max_retries => {
                    let exp_ms = self
                        .retry
                        .base_delay
                        .as_millis()
                        .saturating_mul(1u128 << attempt.min(10));
                    let capped_ms = exp_ms.min(self.retry.max_delay.as_millis()) as u64;
                    let jitter_ms: u64 = rand::thread_rng().gen_range(0..=250);
                    tokio::time::sleep(Duration::from_millis(capped_ms + jitter_ms)).await;
                    attempt += 1;
                }
                Err(err) => return Err(err),
            }
        }
    }
}

/// Fluent builder for [`AdminClient`].
#[derive(Default)]
pub struct AdminClientBuilder {
    service_account_key: Option<ServiceAccountKey>,
    admin_email: Option<String>,
    scopes: Vec<String>,
    token_provider: Option<Arc<dyn TokenProvider>>,
    base_url: Option<String>,
    retry: RetryConfig,
    timeout: Option<Duration>,
}

impl AdminClientBuilder {
    /// Load Service Account credentials from a JSON file path.
    pub fn service_account_file(mut self, path: impl AsRef<Path>) -> Result<Self> {
        self.service_account_key = Some(ServiceAccountKey::from_file(path)?);
        Ok(self)
    }

    /// Load Service Account credentials from an in-memory [`ServiceAccountKey`].
    pub fn service_account_key(mut self, key: ServiceAccountKey) -> Self {
        self.service_account_key = Some(key);
        self
    }

    /// Set the Workspace administrator email to impersonate via Domain-Wide Delegation.
    pub fn admin_email(mut self, email: impl Into<String>) -> Self {
        self.admin_email = Some(email.into());
        self
    }

    /// Set OAuth 2.0 scopes (defaults to `[scopes::USER]` if omitted).
    pub fn scopes(mut self, scopes_list: &[&str]) -> Self {
        self.scopes = scopes_list.iter().map(|s| (*s).to_string()).collect();
        self
    }

    /// Supply a custom [`TokenProvider`] (overrides Service Account auth).
    pub fn token_provider(mut self, provider: Arc<dyn TokenProvider>) -> Self {
        self.token_provider = Some(provider);
        self
    }

    /// Use a static Bearer token (handy for tests or externally minted tokens).
    pub fn static_token(mut self, token: impl Into<String>) -> Self {
        self.token_provider = Some(Arc::new(StaticTokenProvider::new(token)));
        self
    }

    /// Override the Directory API base URL (handy for mock servers in tests).
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Customize retry and exponential backoff behavior.
    pub fn retry_config(mut self, config: RetryConfig) -> Self {
        self.retry = config;
        self
    }

    /// Set per-request HTTP timeout (default: 30 seconds).
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Build the [`AdminClient`].
    pub fn build(self) -> Result<AdminClient> {
        let http = Client::builder()
            .timeout(self.timeout.unwrap_or_else(|| Duration::from_secs(30)))
            .user_agent(concat!(
                "google-workspace-admin-rs/",
                env!("CARGO_PKG_VERSION")
            ))
            .build()?;

        let auth: Arc<dyn TokenProvider> = if let Some(provider) = self.token_provider {
            provider
        } else {
            let key = self.service_account_key.ok_or_else(|| {
                AdminError::Config(
                    "must provide service_account_file/key or a custom token_provider".to_string(),
                )
            })?;
            let admin_email = self.admin_email.ok_or_else(|| {
                AdminError::Config("admin_email is required for Domain-Wide Delegation".to_string())
            })?;
            let scopes_list = if self.scopes.is_empty() {
                vec![scopes::USER.to_string()]
            } else {
                self.scopes
            };
            Arc::new(ServiceAccountAuth::new(
                key,
                admin_email,
                &scopes_list,
                http.clone(),
            )?)
        };

        Ok(AdminClient {
            http,
            auth,
            base_url: self
                .base_url
                .unwrap_or_else(|| DEFAULT_BASE_URL.to_string()),
            retry: self.retry,
        })
    }
}
