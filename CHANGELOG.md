# Changelog

All notable changes to `google-workspace-admin` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-11-25

### Added
- Native Google Cloud Service Account RS256 JWT signing with Domain-Wide Delegation (`sub` admin impersonation).
- Thread-safe OAuth2 access token caching and automatic refresh before expiry (`TokenProvider` trait + `ServiceAccountAuth` + `StaticTokenProvider`).
- Pure-Rust TLS (`reqwest` with `rustls-tls`) — zero OpenSSL system dependencies.
- Automatic exponential backoff with jitter for HTTP `429`, retryable `403` (`rateLimitExceeded`, `userRateLimitExceeded`, `quotaExceeded`, `backendError`), and `5xx`.
- Strongly typed **Users API**: `get`, `list`, `list_all` (auto-pagination), `create`, `patch`, `suspend`, `unsuspend`, `reset_password`, `delete`, `undelete` (20-day recovery), `make_admin`.
- Strongly typed **Groups & Members API**: `get`, `list`, `create`, `delete`, `list_members`, `add_member`, `remove_member`.
- Strongly typed **Organizational Units (OUs) API**: `list`, `create`, `delete`.
- Integration test suite using `wiremock`.
