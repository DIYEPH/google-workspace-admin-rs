//! Example: List Organizational Units and manage Google Group memberships.

use google_workspace_admin::models::AddMemberRequest;
use google_workspace_admin::{scopes, AdminClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let key_file = std::env::var("GOOGLE_SERVICE_ACCOUNT_FILE")
        .unwrap_or_else(|_| "config/service-account.json".to_string());
    let admin_email = std::env::var("GOOGLE_ADMIN_EMAIL").expect("set GOOGLE_ADMIN_EMAIL env var");
    let domain =
        std::env::var("GOOGLE_WORKSPACE_DOMAIN").expect("set GOOGLE_WORKSPACE_DOMAIN env var");

    let client = AdminClient::builder()
        .service_account_file(key_file)?
        .admin_email(admin_email)
        .scopes(&[scopes::GROUP, scopes::ORGUNIT])
        .build()?;

    // List Organizational Units
    let ous = client.org_units().list("my_customer").await?;
    println!("Organizational Units ({}):", ous.organization_units.len());
    for ou in ous.organization_units {
        println!("  - {} ({})", ou.org_unit_path, ou.name);
    }

    // List Groups
    let groups = client.groups().list("my_customer", Some(&domain)).await?;
    println!("Groups ({}):", groups.groups.len());
    for g in &groups.groups {
        println!("  - {} ({})", g.email, g.name);
    }

    // Example: Add a member to the first group (if GROUP_TEST_EMAIL is set)
    if let (Some(first_group), Ok(member_email)) =
        (groups.groups.first(), std::env::var("GROUP_TEST_EMAIL"))
    {
        let member = client
            .groups()
            .add_member(&first_group.email, &AddMemberRequest::member(member_email))
            .await?;
        println!(
            "Added {} as {} to {}",
            member.email, member.role, first_group.email
        );
    }

    Ok(())
}
