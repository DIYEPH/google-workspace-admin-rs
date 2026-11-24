//! Group and Member resource models.

use serde::{Deserialize, Serialize};

/// Google Workspace Group object.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direct_members_count: Option<String>,
}

/// Request body for `groups.insert`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGroupRequest {
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Paginated response for `groups.list`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListGroupsResponse {
    #[serde(default)]
    pub groups: Vec<Group>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
}

/// Google Workspace Group Member object.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Member {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub role: String,
    #[serde(rename = "type", default)]
    pub member_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// Request body for `members.insert`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddMemberRequest {
    pub email: String,
    /// `MEMBER`, `MANAGER`, or `OWNER`.
    pub role: String,
}

impl AddMemberRequest {
    pub fn member(email: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            role: "MEMBER".to_string(),
        }
    }

    pub fn manager(email: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            role: "MANAGER".to_string(),
        }
    }

    pub fn owner(email: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            role: "OWNER".to_string(),
        }
    }
}

/// Paginated response for `members.list`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListMembersResponse {
    #[serde(default)]
    pub members: Vec<Member>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
}
