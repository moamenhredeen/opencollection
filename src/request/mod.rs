//! Requests of all protocols (HTTP, GraphQL, gRPC, WebSocket).
//!
//! Each protocol lives in its own submodule. The shared HTTP header and
//! parameter types are owned by [`http`] and reused by the other protocols,
//! mirroring the OpenCollection schema.

mod graphql;
mod grpc;
mod http;
mod websocket;

pub use graphql::*;
pub use grpc::*;
pub use http::*;
pub use websocket::*;

use serde::{Deserialize, Serialize};

use crate::auth::Auth;
use crate::common::{Scripts, Variable};

/// Request settings for different request types.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequestSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http: Option<HttpRequestSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graphql: Option<GraphQlRequestSettings>,
}

/// Default request configuration for a collection or folder.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequestDefaults {
    /// HTTP headers, sent with http, graphql and websocket requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<Vec<HttpRequestHeader>>,
    /// gRPC metadata, sent with grpc requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Vec<GrpcMetadata>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<Auth>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<Vec<Variable>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scripts: Option<Scripts>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<RequestSettings>,
}
