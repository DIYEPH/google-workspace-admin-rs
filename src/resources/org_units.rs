//! Directory API v1 Organizational Units (OUs) resource client.

use reqwest::Method;
use urlencoding::encode;

use crate::client::AdminClient;
use crate::error::Result;
use crate::models::{CreateOrgUnitRequest, ListOrgUnitsResponse, OrgUnit};

/// Client for Google Workspace Organizational Units API (`/customer/{customerId}/orgunits`).
pub struct OrgUnitsClient<'a> {
    client: &'a AdminClient,
}

impl<'a> OrgUnitsClient<'a> {
    pub(crate) fn new(client: &'a AdminClient) -> Self {
        Self { client }
    }

    /// List all Organizational Units for a customer (`my_customer` by default).
    pub async fn list(&self, customer_id: &str) -> Result<ListOrgUnitsResponse> {
        let path = format!("customer/{}/orgunits", encode(customer_id));
        let query = vec![("type".to_string(), "all".to_string())];
        self.client
            .request_json(Method::GET, &path, Some(&query), None::<&()>)
            .await
    }

    /// Create a new Organizational Unit.
    pub async fn create(
        &self,
        customer_id: &str,
        request: &CreateOrgUnitRequest,
    ) -> Result<OrgUnit> {
        let path = format!("customer/{}/orgunits", encode(customer_id));
        self.client
            .request_json(Method::POST, &path, None, Some(request))
            .await
    }

    /// Delete an Organizational Unit by path (without leading slash, e.g. `Volunteers`).
    pub async fn delete(&self, customer_id: &str, org_unit_path: &str) -> Result<()> {
        let clean = org_unit_path.trim_start_matches('/');
        let path = format!(
            "customer/{}/orgunits/{}",
            encode(customer_id),
            encode(clean)
        );
        self.client
            .request_empty(Method::DELETE, &path, None, None::<&()>)
            .await
    }
}
