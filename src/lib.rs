//! # `google-workspace-admin`
//!
//! An ergonomic, async, pure-Rust SDK for the **Google Workspace Admin SDK (Directory API v1)**.
//!
//! ## Features
//! - **Service Account + Domain-Wide Delegation (DWD):** Native RS256 JWT signing with admin
//!   impersonation (`sub` claim) and thread-safe automatic OAuth2 token caching & refresh.
//! - **Pure Rust TLS (`rustls`):** Zero OpenSSL system dependencies — compiles cleanly to a
//!   single static binary on Linux, macOS, and Windows.
//! - **Resilient Transport:** Built-in exponential backoff with jitter for Google rate limits
//!   (`429`, retryable `403`, and `5xx`).
//! - **Typed Resources:** Full support for **Users** (create, list/auto-paginate, suspend,
//!   unsuspend, reset password, delete, 20-day undelete, makeAdmin), **Groups & Members**, and
//!   **Organizational Units (OUs)**.
//!
//! ## Quick Start
//! ```no_run
//! use google_workspace_admin::{AdminClient, scopes, models::CreateUserRequest};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = AdminClient::builder()
//!         .service_account_file("config/service-account.json")?
//!         .admin_email("admin@yourdomain.org")
//!         .scopes(&[scopes::USER])
//!         .build()?;
//!
//!     let req = CreateUserRequest::builder("alice@yourdomain.org", "Alice", "Nguyen", "StrongPass@2026!")
//!         .org_unit_path("/Volunteers")
//!         .change_password_at_next_login(true)
//!         .build();
//!
//!     let user = client.users().create(&req).await?;
//!     println!("Created user: {} (id={:?})", user.primary_email, user.id);
//!     Ok(())
//! }
//! ```

pub mod auth;
pub mod client;
pub mod error;
pub mod models;
pub mod resources;

pub use auth::{ServiceAccountAuth, ServiceAccountKey, StaticTokenProvider, TokenProvider};
pub use client::{AdminClient, AdminClientBuilder, RetryConfig};
pub use error::{AdminError, Result};

/// Official Google Workspace Admin SDK Directory API OAuth 2.0 scopes.
pub mod scopes {
    /// Full read/write access to users (create, update, suspend, delete, undelete).
    pub const USER: &str = "https://www.googleapis.com/auth/admin.directory.user";
    /// Read-only access to users.
    pub const USER_READONLY: &str = "https://www.googleapis.com/auth/admin.directory.user.readonly";
    /// Full read/write access to groups and group memberships.
    pub const GROUP: &str = "https://www.googleapis.com/auth/admin.directory.group";
    /// Read-only access to groups.
    pub const GROUP_READONLY: &str =
        "https://www.googleapis.com/auth/admin.directory.group.readonly";
    /// Full read/write access to Organizational Units (OUs).
    pub const ORGUNIT: &str = "https://www.googleapis.com/auth/admin.directory.orgunit";
    /// Read-only access to Organizational Units.
    pub const ORGUNIT_READONLY: &str =
        "https://www.googleapis.com/auth/admin.directory.orgunit.readonly";
}
