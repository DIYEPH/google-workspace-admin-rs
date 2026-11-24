//! Directory API v1 Groups & Members resource client.

use reqwest::Method;
use urlencoding::encode;

use crate::client::AdminClient;
use crate::error::Result;
use crate::models::{
    AddMemberRequest, CreateGroupRequest, Group, ListGroupsResponse, ListMembersResponse, Member,
};

/// Client for Google Workspace Groups & Members API (`/groups`).
pub struct GroupsClient<'a> {
    client: &'a AdminClient,
}

impl<'a> GroupsClient<'a> {
    pub(crate) fn new(client: &'a AdminClient) -> Self {
        Self { client }
    }

    /// Retrieve a group by email address or group ID.
    pub async fn get(&self, group_key: &str) -> Result<Group> {
        let path = format!("groups/{}", encode(group_key));
        self.client
            .request_json(Method::GET, &path, None, None::<&()>)
            .await
    }

    /// List groups in a domain or customer.
    pub async fn list(&self, customer: &str, domain: Option<&str>) -> Result<ListGroupsResponse> {
        let mut query = vec![("customer".to_string(), customer.to_string())];
        if let Some(d) = domain {
            if !d.is_empty() {
                query.push(("domain".to_string(), d.to_string()));
            }
        }
        self.client
            .request_json(Method::GET, "groups", Some(&query), None::<&()>)
            .await
    }

    /// Create a new Google Group.
    pub async fn create(&self, request: &CreateGroupRequest) -> Result<Group> {
        self.client
            .request_json(Method::POST, "groups", None, Some(request))
            .await
    }

    /// Delete a Google Group.
    pub async fn delete(&self, group_key: &str) -> Result<()> {
        let path = format!("groups/{}", encode(group_key));
        self.client
            .request_empty(Method::DELETE, &path, None, None::<&()>)
            .await
    }

    /// List members of a group.
    pub async fn list_members(&self, group_key: &str) -> Result<ListMembersResponse> {
        let path = format!("groups/{}/members", encode(group_key));
        self.client
            .request_json(Method::GET, &path, None, None::<&()>)
            .await
    }

    /// Add a member to a group.
    pub async fn add_member(
        &self,
        group_key: &str,
        request: &AddMemberRequest,
    ) -> Result<Member> {
        let path = format!("groups/{}/members", encode(group_key));
        self.client
            .request_json(Method::POST, &path, None, Some(request))
            .await
    }

    /// Remove a member from a group.
    pub async fn remove_member(&self, group_key: &str, member_key: &str) -> Result<()> {
        let path = format!(
            "groups/{}/members/{}",
            encode(group_key),
            encode(member_key)
        );
        self.client
            .request_empty(Method::DELETE, &path, None, None::<&()>)
            .await
    }
}
