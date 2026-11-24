//! Directory API v1 Users resource client.

use reqwest::Method;
use urlencoding::encode;

use crate::client::AdminClient;
use crate::error::Result;
use crate::models::{
    CreateUserRequest, ListUsersParams, ListUsersResponse, MakeAdminRequest, UndeleteUserRequest,
    UpdateUserRequest, User,
};

/// Client for Google Workspace Users API (`/users`).
pub struct UsersClient<'a> {
    client: &'a AdminClient,
}

impl<'a> UsersClient<'a> {
    pub(crate) fn new(client: &'a AdminClient) -> Self {
        Self { client }
    }

    /// Retrieve a single user by primary email, alias, or immutable 21-digit `userId`.
    pub async fn get(&self, user_key: &str) -> Result<User> {
        let path = format!("users/{}", encode(user_key));
        self.client
            .request_json(Method::GET, &path, None, None::<&()>)
            .await
    }

    /// Fetch a single page of users.
    pub async fn list(&self, params: &ListUsersParams) -> Result<ListUsersResponse> {
        let query = params.to_query_pairs();
        self.client
            .request_json(Method::GET, "users", Some(&query), None::<&()>)
            .await
    }

    /// Automatically paginate through all matching users (up to `max_total` if provided).
    pub async fn list_all(
        &self,
        params: &ListUsersParams,
        max_total: Option<usize>,
    ) -> Result<Vec<User>> {
        let mut all = Vec::new();
        let mut next_params = params.clone();

        loop {
            if let Some(limit) = max_total {
                let remaining = limit.saturating_sub(all.len());
                if remaining == 0 {
                    break;
                }
                next_params.max_results = Some((remaining.min(500)) as u32);
            } else if next_params.max_results.is_none() {
                next_params.max_results = Some(500);
            }

            let page = self.list(&next_params).await?;
            all.extend(page.users);

            match page.next_page_token {
                Some(token) if !token.is_empty() => next_params.page_token = Some(token),
                _ => break,
            }
        }

        if let Some(limit) = max_total {
            all.truncate(limit);
        }
        Ok(all)
    }

    /// Create a new Google Workspace user (`POST /users`).
    pub async fn create(&self, request: &CreateUserRequest) -> Result<User> {
        self.client
            .request_json(Method::POST, "users", None, Some(request))
            .await
    }

    /// Partially update a user (`PATCH /users/{userKey}`).
    pub async fn patch(&self, user_key: &str, request: &UpdateUserRequest) -> Result<User> {
        let path = format!("users/{}", encode(user_key));
        self.client
            .request_json(Method::PATCH, &path, None, Some(request))
            .await
    }

    /// Convenience helper: suspend a user (optionally moving them to a suspended OU).
    pub async fn suspend(&self, user_key: &str, suspended_ou: Option<&str>) -> Result<User> {
        let req = UpdateUserRequest {
            suspended: Some(true),
            org_unit_path: suspended_ou.filter(|s| !s.is_empty()).map(str::to_string),
            ..Default::default()
        };
        self.patch(user_key, &req).await
    }

    /// Convenience helper: unsuspend a user (optionally restoring their OU).
    pub async fn unsuspend(&self, user_key: &str, restore_ou: Option<&str>) -> Result<User> {
        let req = UpdateUserRequest {
            suspended: Some(false),
            org_unit_path: restore_ou.filter(|s| !s.is_empty()).map(str::to_string),
            ..Default::default()
        };
        self.patch(user_key, &req).await
    }

    /// Convenience helper: reset a user's password.
    pub async fn reset_password(
        &self,
        user_key: &str,
        new_password: &str,
        change_at_next_login: bool,
    ) -> Result<User> {
        let req = UpdateUserRequest {
            password: Some(new_password.to_string()),
            change_password_at_next_login: Some(change_at_next_login),
            ..Default::default()
        };
        self.patch(user_key, &req).await
    }

    /// Delete a user (`DELETE /users/{userKey}`).
    ///
    /// Deleted users enter Google's 20-day soft-delete recovery window. Save their immutable
    /// `id` before calling this if you want to use [`UsersClient::undelete`].
    pub async fn delete(&self, user_key: &str) -> Result<()> {
        let path = format!("users/{}", encode(user_key));
        self.client
            .request_empty(Method::DELETE, &path, None, None::<&()>)
            .await
    }

    /// Restore a soft-deleted user within 20 days (`POST /users/{userId}/undelete`).
    ///
    /// **Important:** Google requires the immutable 21-digit `user_id` here, not the email address.
    pub async fn undelete(&self, user_id: &str, org_unit_path: &str) -> Result<()> {
        let path = format!("users/{}/undelete", encode(user_id));
        let body = UndeleteUserRequest {
            org_unit_path: if org_unit_path.trim().is_empty() {
                "/".to_string()
            } else {
                org_unit_path.to_string()
            },
        };
        self.client
            .request_empty(Method::POST, &path, None, Some(&body))
            .await
    }

    /// Grant or revoke Super Admin privileges (`POST /users/{userKey}/makeAdmin`).
    pub async fn make_admin(&self, user_key: &str, is_admin: bool) -> Result<()> {
        let path = format!("users/{}/makeAdmin", encode(user_key));
        let body = MakeAdminRequest { status: is_admin };
        self.client
            .request_empty(Method::POST, &path, None, Some(&body))
            .await
    }
}
