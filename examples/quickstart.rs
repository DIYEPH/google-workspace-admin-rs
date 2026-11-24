//! Quickstart example: Create, list, suspend, delete, and undelete a Google Workspace user.
//!
//! Run with:
//! ```bash
//! GOOGLE_SERVICE_ACCOUNT_FILE=service-account.json \
//! GOOGLE_ADMIN_EMAIL=admin@yourdomain.org \
//! GOOGLE_WORKSPACE_DOMAIN=yourdomain.org \
//! cargo run --example quickstart
//! ```

use google_workspace_admin::models::{CreateUserRequest, ListUsersParams};
use google_workspace_admin::{scopes, AdminClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let key_file = std::env::var("GOOGLE_SERVICE_ACCOUNT_FILE")
        .unwrap_or_else(|_| "config/service-account.json".to_string());
    let admin_email = std::env::var("GOOGLE_ADMIN_EMAIL")
        .expect("set GOOGLE_ADMIN_EMAIL env var");
    let domain = std::env::var("GOOGLE_WORKSPACE_DOMAIN")
        .expect("set GOOGLE_WORKSPACE_DOMAIN env var");

    let client = AdminClient::builder()
        .service_account_file(key_file)?
        .admin_email(admin_email)
        .scopes(&[scopes::USER])
        .build()?;

    // 1. Auto-paginate users in the domain
    let users = client
        .users()
        .list_all(&ListUsersParams::new().domain(&domain), Some(10))
        .await?;
    println!("Found {} users (showing up to 10):", users.len());
    for u in &users {
        println!("  - {} (suspended={}, ou={})", u.primary_email, u.suspended, u.org_unit_path);
    }

    // 2. Create a test user
    let test_email = format!("sdk-test-user@{domain}");
    let req = CreateUserRequest::builder(&test_email, "SDK", "Test", "StrongPass@2026!")
        .org_unit_path("/")
        .change_password_at_next_login(true)
        .build();

    let created = client.users().create(&req).await?;
    println!("Created {} with immutable id={}", created.primary_email, created.id);

    // 3. Suspend -> Delete -> Undelete (using immutable id)
    client.users().suspend(&test_email, None).await?;
    client.users().delete(&test_email).await?;
    client.users().undelete(&created.id, "/").await?;
    println!("Successfully tested suspend -> delete -> 20-day undelete!");

    Ok(())
}
