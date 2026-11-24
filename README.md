# `google-workspace-admin`

[![Crates.io](https://img.shields.io/crates/v/google-workspace-admin.svg)](https://crates.io/crates/google-workspace-admin)
[![Docs.rs](https://docs.rs/google-workspace-admin/badge.svg)](https://docs.rs/google-workspace-admin)
[![CI](https://github.com/DIYEPH/google-workspace-admin-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/DIYEPH/google-workspace-admin-rs/actions)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

An ergonomic, async, **pure-Rust SDK** for the **Google Workspace Admin SDK (Directory API v1)** — built for production user/group/OU automation with native **Service Account + Domain-Wide Delegation (DWD)**, automatic OAuth2 token refresh, pure-Rust TLS (`rustls`), and exponential backoff.

## Why `google-workspace-admin`?

| Feature | `google-workspace-admin` | Auto-generated Google API crates |
| :--- | :---: | :---: |
| Ergonomic Rust builders & typed errors | ✅ | ❌ Verbose raw structs |
| Native DWD RS256 JWT + auto token cache | ✅ Built-in | ❌ External setup |
| Zero OpenSSL (`rustls` static binary friendly) | ✅ | ⚠️ Often requires OpenSSL |
| Automatic exponential backoff + jitter (`429`/`5xx`/`403`) | ✅ Built-in | ❌ Manual |
| Auto-pagination (`users().list_all()`) | ✅ Built-in | ❌ Manual loop |
| First-class 20-day `undelete` & `suspend`/`unsuspend` helpers | ✅ | ❌ Raw patches |

## Installation

```toml
[dependencies]
google-workspace-admin = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Quick Start

```rust
use google_workspace_admin::{
    models::{CreateUserRequest, ListUsersParams},
    scopes, AdminClient,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AdminClient::builder()
        .service_account_file("config/service-account.json")?
        .admin_email("automation-admin@yourdomain.org")
        .scopes(&[scopes::USER, scopes::GROUP, scopes::ORGUNIT])
        .build()?;

    // 1. Create a user
    let req = CreateUserRequest::builder(
        "alice@yourdomain.org",
        "Alice",
        "Nguyen",
        "StrongPass@2026!",
    )
    .org_unit_path("/Volunteers")
    .change_password_at_next_login(true)
    .build();

    let created = client.users().create(&req).await?;
    println!("Created {} (immutable id={})", created.primary_email, created.id);

    // 2. Auto-paginate all users in domain
    let all_users = client
        .users()
        .list_all(&ListUsersParams::new().domain("yourdomain.org"), None)
        .await?;
    println!("Total users: {}", all_users.len());

    // 3. Suspend, Delete & 20-day Undelete
    client.users().suspend("alice@yourdomain.org", None).await?;
    client.users().delete("alice@yourdomain.org").await?;
    client.users().undelete(&created.id, "/Volunteers").await?;

    Ok(())
}
```

## Google Cloud & Workspace Setup (Domain-Wide Delegation)

1. Enable **Admin SDK API** in your Google Cloud project.
2. Create a **Service Account**, copy its numeric **Unique ID** (Client ID), and download its JSON key.
3. In **Google Admin Console** → *Security → Access and data control → API controls → Manage Domain Wide Delegation*, click **Add new**:
   - **Client ID:** your Service Account Unique ID
   - **OAuth Scopes:** `https://www.googleapis.com/auth/admin.directory.user` (plus `admin.directory.group` / `admin.directory.orgunit` if needed).

## License

Dual-licensed under either of:
- [MIT License](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.
