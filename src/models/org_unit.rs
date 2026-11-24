//! Organizational Unit (OU) resource models.

use serde::{Deserialize, Serialize};

/// Google Workspace Organizational Unit object.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrgUnit {
    #[serde(default)]
    pub org_unit_id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub org_unit_path: String,
    #[serde(default)]
    pub parent_org_unit_path: String,
    #[serde(default)]
    pub description: String,
}

/// Request body for `orgunits.insert`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrgUnitRequest {
    pub name: String,
    pub parent_org_unit_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Response for `orgunits.list`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListOrgUnitsResponse {
    #[serde(default)]
    pub organization_units: Vec<OrgUnit>,
}
