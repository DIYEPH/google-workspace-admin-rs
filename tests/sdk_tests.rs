//! Integration tests for `google-workspace-admin` using `wiremock`.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use google_workspace_admin::models::{
    AddMemberRequest, CreateUserRequest, ListUsersParams,
};
use google_workspace_admin::{AdminClient, AdminError, RetryConfig};
use serde_json::json;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

#[tokio::test]
async fn test_user_crud_and_undelete_lifecycle() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/users"))
        .and(header("authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "112233445566778899001",
            "primaryEmail": "alice@nonprofit.org",
            "name": { "givenName": "Alice", "familyName": "Nguyen", "fullName": "Alice Nguyen" },
            "orgUnitPath": "/Volunteers",
            "suspended": false,
            "isAdmin": false
        })))
        .mount(&server)
        .await;

    Mock::given(method("PATCH"))
        .and(path("/users/alice%40nonprofit.org"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "112233445566778899001",
            "primaryEmail": "alice@nonprofit.org",
            "suspended": true,
            "orgUnitPath": "/Suspended"
        })))
        .mount(&server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/users/alice%40nonprofit.org"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/users/112233445566778899001/undelete"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = AdminClient::builder()
        .static_token("test-token")
        .base_url(server.uri())
        .build()
        .unwrap();

    let req = CreateUserRequest::builder(
        "alice@nonprofit.org",
        "Alice",
        "Nguyen",
        "StrongPass@2026!",
    )
    .org_unit_path("/Volunteers")
    .build();

    let created = client.users().create(&req).await.unwrap();
    assert_eq!(created.id, "112233445566778899001");
    assert_eq!(created.org_unit_path, "/Volunteers");

    let suspended = client
        .users()
        .suspend("alice@nonprofit.org", Some("/Suspended"))
        .await
        .unwrap();
    assert!(suspended.suspended);

    client.users().delete("alice@nonprofit.org").await.unwrap();
    client
        .users()
        .undelete("112233445566778899001", "/Volunteers")
        .await
        .unwrap();
}

#[tokio::test]
async fn test_auto_pagination_list_all() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/users"))
        .and(query_param("pageToken", "page-2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "users": [{ "id": "2", "primaryEmail": "u2@nonprofit.org" }]
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/users"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "users": [{ "id": "1", "primaryEmail": "u1@nonprofit.org" }],
            "nextPageToken": "page-2"
        })))
        .mount(&server)
        .await;

    let client = AdminClient::builder()
        .static_token("test-token")
        .base_url(server.uri())
        .build()
        .unwrap();

    let all = client
        .users()
        .list_all(&ListUsersParams::new().domain("nonprofit.org"), None)
        .await
        .unwrap();
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].primary_email, "u1@nonprofit.org");
    assert_eq!(all[1].primary_email, "u2@nonprofit.org");
}

struct Flaky429Responder {
    hits: Arc<AtomicUsize>,
}

impl Respond for Flaky429Responder {
    fn respond(&self, _request: &Request) -> ResponseTemplate {
        let n = self.hits.fetch_add(1, Ordering::SeqCst);
        if n < 2 {
            ResponseTemplate::new(429).set_body_json(json!({
                "error": {
                    "code": 429,
                    "message": "Rate limit exceeded",
                    "errors": [{ "reason": "rateLimitExceeded" }]
                }
            }))
        } else {
            ResponseTemplate::new(200).set_body_json(json!({
                "id": "42",
                "primaryEmail": "retry@nonprofit.org"
            }))
        }
    }
}

#[tokio::test]
async fn test_exponential_backoff_on_429() {
    let server = MockServer::start().await;
    let hits = Arc::new(AtomicUsize::new(0));

    Mock::given(method("GET"))
        .and(path("/users/retry%40nonprofit.org"))
        .respond_with(Flaky429Responder { hits: hits.clone() })
        .mount(&server)
        .await;

    let client = AdminClient::builder()
        .static_token("test-token")
        .base_url(server.uri())
        .retry_config(RetryConfig {
            max_retries: 3,
            base_delay: Duration::from_millis(5),
            max_delay: Duration::from_millis(20),
        })
        .build()
        .unwrap();

    let user = client.users().get("retry@nonprofit.org").await.unwrap();
    assert_eq!(user.id, "42");
    assert_eq!(hits.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn test_typed_errors_and_groups() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/users/missing%40nonprofit.org"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": { "code": 404, "message": "Resource Not Found: userKey" }
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/groups/volunteers%40nonprofit.org/members"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "m1",
            "email": "alice@nonprofit.org",
            "role": "MEMBER",
            "type": "USER"
        })))
        .mount(&server)
        .await;

    let client = AdminClient::builder()
        .static_token("test-token")
        .base_url(server.uri())
        .build()
        .unwrap();

    let err = client.users().get("missing@nonprofit.org").await.unwrap_err();
    assert!(matches!(err, AdminError::NotFound(_)));

    let member = client
        .groups()
        .add_member(
            "volunteers@nonprofit.org",
            &AddMemberRequest::member("alice@nonprofit.org"),
        )
        .await
        .unwrap();
    assert_eq!(member.role, "MEMBER");
}
