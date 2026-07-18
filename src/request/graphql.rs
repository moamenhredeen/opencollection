//! GraphQL requests.

use serde::{Deserialize, Serialize};

use crate::auth::Auth;
use crate::body::{GraphQlBody, GraphQlBodyOrVariants};
use crate::common::{Description, Sequence, Tag};
use crate::request::{
    HttpRequestHeader, HttpRequestParam, HttpRequestRuntime, HttpRequestSettings,
};

/// Settings for GraphQL request execution (same shape as HTTP settings).
pub type GraphQlRequestSettings = HttpRequestSettings;

/// GraphQL request runtime configuration (same shape as HTTP runtime).
pub type GraphQlRequestRuntime = HttpRequestRuntime;

/// The literal item type `"graphql"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GraphQlTypeTag {
    #[default]
    #[serde(rename = "graphql")]
    GraphQl,
}

/// GraphQL request metadata.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphQlRequestInfo {
    /// The display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Description>,
    /// The item type discriminator.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub item_type: Option<GraphQlTypeTag>,
    /// Sequence number for ordering in a UI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seq: Option<Sequence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<Tag>>,
}

/// A GraphQL request.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphQlRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<GraphQlRequestInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graphql: Option<GraphQlRequestDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<GraphQlRequestRuntime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<GraphQlRequestSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs: Option<String>,
}

/// GraphQL request protocol details.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphQlRequestDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<Vec<HttpRequestHeader>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Vec<HttpRequestParam>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<GraphQlBodyOrVariants>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<Auth>,
}

/// Fluent construction helpers.
impl GraphQlRequest {
    /// Create a GraphQL request against the given URL.
    pub fn new(url: impl Into<String>) -> Self {
        GraphQlRequest {
            info: Some(GraphQlRequestInfo {
                item_type: Some(GraphQlTypeTag::GraphQl),
                ..GraphQlRequestInfo::default()
            }),
            graphql: Some(GraphQlRequestDetails {
                method: Some("POST".to_owned()),
                url: Some(url.into()),
                ..GraphQlRequestDetails::default()
            }),
            ..GraphQlRequest::default()
        }
    }

    /// Set the display name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.info
            .get_or_insert_with(GraphQlRequestInfo::default)
            .name = Some(name.into());
        self
    }

    /// Set the GraphQL query (and optional variables as a JSON string).
    pub fn query(mut self, query: impl Into<String>, variables: Option<String>) -> Self {
        self.graphql
            .get_or_insert_with(GraphQlRequestDetails::default)
            .body = Some(
            GraphQlBody {
                query: Some(query.into()),
                variables,
            }
            .into(),
        );
        self
    }
}
