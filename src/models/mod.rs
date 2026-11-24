//! Strongly typed Google Workspace Directory API v1 data models.

pub mod group;
pub mod org_unit;
pub mod user;

pub use group::{
    AddMemberRequest, CreateGroupRequest, Group, ListGroupsResponse, ListMembersResponse, Member,
};
pub use org_unit::{CreateOrgUnitRequest, ListOrgUnitsResponse, OrgUnit};
pub use user::{
    CreateUserRequest, CreateUserRequestBuilder, ListUsersParams, ListUsersResponse,
    MakeAdminRequest, UndeleteUserRequest, UpdateUserRequest, User, UserName,
};
