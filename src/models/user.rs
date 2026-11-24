//! User resource models and builders.

use serde::{Deserialize, Serialize};

/// User display name components.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UserName {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub given_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_name: Option<String>,
}

/// Google Workspace Directory User object.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub primary_email: String,
    #[serde(default)]
    pub name: UserName,
    #[serde(default)]
    pub is_admin: bool,
    #[serde(default)]
    pub is_delegated_admin: bool,
    #[serde(default)]
    pub suspended: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suspension_reason: Option<String>,
    #[serde(default = "default_ou")]
    pub org_unit_path: String,
    #[serde(default)]
    pub change_password_at_next_login: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creation_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_login_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deletion_time: Option<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
}

fn default_ou() -> String {
    "/".to_string()
}

/// Request body for `users.insert`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserRequest {
    pub primary_email: String,
    pub name: UserName,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash_function: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org_unit_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change_password_at_next_login: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suspended: Option<bool>,
}

impl CreateUserRequest {
    /// Create a fluent builder for a new user.
    pub fn builder(
        primary_email: impl Into<String>,
        given_name: impl Into<String>,
        family_name: impl Into<String>,
        password: impl Into<String>,
    ) -> CreateUserRequestBuilder {
        CreateUserRequestBuilder {
            req: Self {
                primary_email: primary_email.into(),
                name: UserName {
                    given_name: Some(given_name.into()),
                    family_name: Some(family_name.into()),
                    full_name: None,
                },
                password: password.into(),
                hash_function: None,
                org_unit_path: Some("/".to_string()),
                change_password_at_next_login: Some(true),
                recovery_email: None,
                recovery_phone: None,
                suspended: None,
            },
        }
    }
}

/// Fluent builder for [`CreateUserRequest`].
pub struct CreateUserRequestBuilder {
    req: CreateUserRequest,
}

impl CreateUserRequestBuilder {
    pub fn org_unit_path(mut self, ou: impl Into<String>) -> Self {
        let s = ou.into();
        let normalized = if s.trim().is_empty() {
            "/".to_string()
        } else if s.starts_with('/') {
            s
        } else {
            format!("/{s}")
        };
        self.req.org_unit_path = Some(normalized);
        self
    }

    pub fn change_password_at_next_login(mut self, force: bool) -> Self {
        self.req.change_password_at_next_login = Some(force);
        self
    }

    pub fn recovery_email(mut self, email: impl Into<String>) -> Self {
        let s = email.into();
        if !s.trim().is_empty() {
            self.req.recovery_email = Some(s);
        }
        self
    }

    pub fn recovery_phone(mut self, phone: impl Into<String>) -> Self {
        let s = phone.into();
        if !s.trim().is_empty() {
            self.req.recovery_phone = Some(s);
        }
        self
    }

    pub fn hash_function(mut self, hash_fn: impl Into<String>) -> Self {
        self.req.hash_function = Some(hash_fn.into());
        self
    }

    pub fn suspended(mut self, suspended: bool) -> Self {
        self.req.suspended = Some(suspended);
        self
    }

    pub fn build(self) -> CreateUserRequest {
        self.req
    }
}

/// Request body for `users.patch` / `users.update`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<UserName>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suspended: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org_unit_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change_password_at_next_login: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_email: Option<String>,
}

/// Request body for `users.undelete`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UndeleteUserRequest {
    pub org_unit_path: String,
}

/// Request body for `users.makeAdmin`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MakeAdminRequest {
    pub status: bool,
}

/// Query parameters for `users.list`.
#[derive(Debug, Clone, Default)]
pub struct ListUsersParams {
    pub customer: Option<String>,
    pub domain: Option<String>,
    pub query: Option<String>,
    pub show_deleted: bool,
    pub max_results: Option<u32>,
    pub order_by: Option<String>,
    pub page_token: Option<String>,
}

impl ListUsersParams {
    pub fn new() -> Self {
        Self {
            customer: Some("my_customer".to_string()),
            order_by: Some("email".to_string()),
            ..Default::default()
        }
    }

    pub fn customer(mut self, customer: impl Into<String>) -> Self {
        self.customer = Some(customer.into());
        self
    }

    pub fn domain(mut self, domain: impl Into<String>) -> Self {
        self.domain = Some(domain.into());
        self
    }

    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    pub fn show_deleted(mut self, show: bool) -> Self {
        self.show_deleted = show;
        self
    }

    pub fn max_results(mut self, max: u32) -> Self {
        self.max_results = Some(max.clamp(1, 500));
        self
    }

    pub fn page_token(mut self, token: impl Into<String>) -> Self {
        self.page_token = Some(token.into());
        self
    }

    pub(crate) fn to_query_pairs(&self) -> Vec<(String, String)> {
        let mut pairs = Vec::new();
        if let Some(c) = &self.customer {
            pairs.push(("customer".to_string(), c.clone()));
        }
        if let Some(d) = &self.domain {
            if !d.is_empty() {
                pairs.push(("domain".to_string(), d.clone()));
            }
        }
        if let Some(q) = &self.query {
            if !q.is_empty() {
                pairs.push(("query".to_string(), q.clone()));
            }
        }
        if self.show_deleted {
            pairs.push(("showDeleted".to_string(), "true".to_string()));
        }
        if let Some(m) = self.max_results {
            pairs.push(("maxResults".to_string(), m.to_string()));
        }
        if let Some(o) = &self.order_by {
            pairs.push(("orderBy".to_string(), o.clone()));
        }
        if let Some(pt) = &self.page_token {
            pairs.push(("pageToken".to_string(), pt.clone()));
        }
        pairs
    }
}

/// Paginated response from `users.list`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListUsersResponse {
    #[serde(default)]
    pub users: Vec<User>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
}
