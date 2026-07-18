//! gRPC requests.

use serde::{Deserialize, Serialize};

use crate::auth::Auth;
use crate::common::{Assertion, Description, Scripts, Sequence, Tag, Variable};

/// A gRPC metadata entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrpcMetadata {
    pub name: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Description>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
}

/// The literal item type `"grpc"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GrpcTypeTag {
    #[default]
    #[serde(rename = "grpc")]
    Grpc,
}

/// gRPC request metadata.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrpcRequestInfo {
    /// The display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Description>,
    /// The item type discriminator.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub item_type: Option<GrpcTypeTag>,
    /// Sequence number for ordering in a UI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seq: Option<Sequence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<Tag>>,
}

/// A gRPC request.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrpcRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<GrpcRequestInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grpc: Option<GrpcRequestDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<GrpcRequestRuntime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs: Option<String>,
}

/// gRPC request protocol details.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GrpcRequestDetails {
    /// The gRPC service URL or endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Full RPC method name (`package.Service/Method`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    /// Method streaming type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method_type: Option<GrpcMethodType>,
    /// Path to the proto file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proto_file_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Vec<GrpcMetadata>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<GrpcMessageOrVariants>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<Auth>,
}

/// gRPC method streaming type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GrpcMethodType {
    Unary,
    ClientStreaming,
    ServerStreaming,
    BidiStreaming,
}

/// A gRPC message: a single message string or a list of selectable variants.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GrpcMessageOrVariants {
    Message(String),
    Variants(Vec<GrpcMessageVariant>),
}

/// A named variant of a gRPC message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrpcMessageVariant {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected: Option<bool>,
    pub message: String,
}

/// gRPC request runtime configuration.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrpcRequestRuntime {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<Vec<Variable>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scripts: Option<Scripts>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assertions: Option<Vec<Assertion>>,
}

/// Fluent construction helpers.
impl GrpcRequest {
    /// Create a gRPC request for `url` calling `method` (`package.Service/Method`).
    pub fn new(url: impl Into<String>, method: impl Into<String>) -> Self {
        GrpcRequest {
            info: Some(GrpcRequestInfo {
                item_type: Some(GrpcTypeTag::Grpc),
                ..GrpcRequestInfo::default()
            }),
            grpc: Some(GrpcRequestDetails {
                url: Some(url.into()),
                method: Some(method.into()),
                ..GrpcRequestDetails::default()
            }),
            ..GrpcRequest::default()
        }
    }

    /// Set the display name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.info.get_or_insert_with(GrpcRequestInfo::default).name = Some(name.into());
        self
    }
}
