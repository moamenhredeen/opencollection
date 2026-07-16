//! Collection items: requests of all protocols, folders and script files.

use serde::{Deserialize, Serialize};

use crate::auth::Auth;
use crate::body::{GraphQlBodyOrVariants, HttpBodyOrVariants, HttpRequestBody};
use crate::common::{
    Action, Assertion, Description, Documentation, Inheritable, Number, Scripts, Sequence, Tag,
    Variable,
};

/// An item in a collection or folder.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Item {
    Http(HttpRequest),
    GraphQl(GraphQlRequest),
    Grpc(GrpcRequest),
    WebSocket(WebSocketRequest),
    Folder(Folder),
    Script(ScriptFile),
}

impl Item {
    /// The item's display name, if it has one.
    pub fn name(&self) -> Option<&str> {
        match self {
            Item::Http(request) => request.info.as_ref()?.name.as_deref(),
            Item::GraphQl(request) => request.info.as_ref()?.name.as_deref(),
            Item::Grpc(request) => request.info.as_ref()?.name.as_deref(),
            Item::WebSocket(request) => request.info.as_ref()?.name.as_deref(),
            Item::Folder(folder) => folder.info.as_ref()?.name.as_deref(),
            Item::Script(_) => None,
        }
    }

    /// Whether this item is a request (HTTP, GraphQL, gRPC or WebSocket).
    pub fn is_request(&self) -> bool {
        matches!(
            self,
            Item::Http(_) | Item::GraphQl(_) | Item::Grpc(_) | Item::WebSocket(_)
        )
    }
}

impl From<HttpRequest> for Item {
    fn from(request: HttpRequest) -> Self {
        Item::Http(request)
    }
}

impl From<GraphQlRequest> for Item {
    fn from(request: GraphQlRequest) -> Self {
        Item::GraphQl(request)
    }
}

impl From<GrpcRequest> for Item {
    fn from(request: GrpcRequest) -> Self {
        Item::Grpc(request)
    }
}

impl From<WebSocketRequest> for Item {
    fn from(request: WebSocketRequest) -> Self {
        Item::WebSocket(request)
    }
}

impl From<Folder> for Item {
    fn from(folder: Folder) -> Self {
        Item::Folder(folder)
    }
}

impl From<ScriptFile> for Item {
    fn from(script: ScriptFile) -> Self {
        Item::Script(script)
    }
}

/// An HTTP header with name, value, description and disabled state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HttpRequestHeader {
    pub name: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Description>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
}

/// An HTTP response header (used in examples).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HttpResponseHeader {
    pub name: String,
    pub value: String,
}

/// A request parameter (query or path).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HttpRequestParam {
    pub name: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Description>,
    #[serde(rename = "type")]
    pub param_type: ParamType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
}

/// The kind of a request parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParamType {
    Query,
    Path,
}

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

/// Settings for HTTP (and GraphQL) request execution.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HttpRequestSettings {
    /// Whether to encode the URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encode_url: Option<Inheritable<bool>>,
    /// Request timeout in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<Inheritable<Number>>,
    /// Whether to follow redirects.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub follow_redirects: Option<Inheritable<bool>>,
    /// Maximum number of redirects to follow.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_redirects: Option<Inheritable<Number>>,
}

/// Settings for GraphQL request execution (same shape as HTTP settings).
pub type GraphQlRequestSettings = HttpRequestSettings;

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

macro_rules! info_type_tag {
    ($(#[$doc:meta])* $name:ident, $tag:literal, $variant:ident) => {
        $(#[$doc])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
        pub enum $name {
            #[default]
            #[serde(rename = $tag)]
            $variant,
        }
    };
}

info_type_tag!(
    /// The literal item type `"http"`.
    HttpTypeTag, "http", Http
);
info_type_tag!(
    /// The literal item type `"graphql"`.
    GraphQlTypeTag, "graphql", GraphQl
);
info_type_tag!(
    /// The literal item type `"grpc"`.
    GrpcTypeTag, "grpc", Grpc
);
info_type_tag!(
    /// The literal item type `"websocket"`.
    WebSocketTypeTag, "websocket", WebSocket
);
info_type_tag!(
    /// The literal item type `"folder"`.
    FolderTypeTag, "folder", Folder
);
info_type_tag!(
    /// The literal item type `"script"`.
    ScriptTypeTag, "script", Script
);

macro_rules! request_info {
    ($(#[$doc:meta])* $name:ident, $tag:ty) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct $name {
            /// The display name.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub name: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub description: Option<Description>,
            /// The item type discriminator.
            #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
            pub item_type: Option<$tag>,
            /// Sequence number for ordering in a UI.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub seq: Option<Sequence>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub tags: Option<Vec<Tag>>,
        }
    };
}

request_info!(
    /// HTTP request metadata.
    HttpRequestInfo, HttpTypeTag
);
request_info!(
    /// GraphQL request metadata.
    GraphQlRequestInfo, GraphQlTypeTag
);
request_info!(
    /// gRPC request metadata.
    GrpcRequestInfo, GrpcTypeTag
);
request_info!(
    /// WebSocket request metadata.
    WebSocketRequestInfo, WebSocketTypeTag
);
request_info!(
    /// Folder metadata.
    FolderInfo, FolderTypeTag
);

/// An HTTP request.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HttpRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<HttpRequestInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http: Option<HttpRequestDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<HttpRequestRuntime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<HttpRequestSettings>,
    /// Example request/response pairs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples: Option<Vec<HttpRequestExample>>,
    /// Documentation for this request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs: Option<String>,
}

/// HTTP request protocol details.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HttpRequestDetails {
    /// HTTP method.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<Vec<HttpRequestHeader>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Vec<HttpRequestParam>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<HttpBodyOrVariants>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<Auth>,
}

/// HTTP request runtime configuration.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HttpRequestRuntime {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<Vec<Variable>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scripts: Option<Scripts>,
    /// Assertions for response validation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assertions: Option<Vec<Assertion>>,
    /// Runtime actions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actions: Option<Vec<Action>>,
}

/// An example HTTP request/response pair.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HttpRequestExample {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Description>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<HttpExampleRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<HttpExampleResponse>,
}

/// The request half of an [`HttpRequestExample`].
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HttpExampleRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<Vec<HttpRequestHeader>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Vec<HttpRequestParam>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<HttpRequestBody>,
}

/// The response half of an [`HttpRequestExample`].
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HttpExampleResponse {
    /// HTTP status code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Number>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<Vec<HttpResponseHeader>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<HttpExampleResponseBody>,
}

/// The body of an example response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HttpExampleResponseBody {
    #[serde(rename = "type")]
    pub body_type: ExampleResponseBodyType,
    pub data: String,
}

/// The kind of an example response body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExampleResponseBodyType {
    Json,
    Text,
    Xml,
    Html,
    Binary,
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

/// GraphQL request runtime configuration (same shape as HTTP runtime).
pub type GraphQlRequestRuntime = HttpRequestRuntime;

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

/// A WebSocket request.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WebSocketRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<WebSocketRequestInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub websocket: Option<WebSocketRequestDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<WebSocketRequestRuntime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<WebSocketRequestSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs: Option<String>,
}

/// WebSocket request protocol details.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WebSocketRequestDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<Vec<HttpRequestHeader>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<WebSocketMessageOrVariants>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<Auth>,
}

/// A WebSocket message with type and data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WebSocketMessage {
    #[serde(rename = "type")]
    pub message_type: WebSocketMessageType,
    pub data: String,
}

/// The kind of a WebSocket message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WebSocketMessageType {
    Text,
    Json,
    Xml,
    Binary,
}

/// A WebSocket message: a single message or a list of selectable variants.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WebSocketMessageOrVariants {
    Message(WebSocketMessage),
    Variants(Vec<WebSocketMessageVariant>),
}

/// A named variant of a WebSocket message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WebSocketMessageVariant {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected: Option<bool>,
    pub message: WebSocketMessage,
}

/// WebSocket request runtime configuration.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WebSocketRequestRuntime {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<Vec<Variable>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scripts: Option<Scripts>,
}

/// WebSocket request settings.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WebSocketRequestSettings {
    /// Connection timeout in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<Inheritable<Number>>,
    /// Keep-alive interval in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_alive_interval: Option<Inheritable<Number>>,
}

/// A folder for organizing collection items.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Folder {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<FolderInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<Item>>,
    /// Default request configuration for items in this folder.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<RequestDefaults>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs: Option<Documentation>,
}

/// A JavaScript module or shared collection script.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScriptFile {
    /// The item type discriminator (`"script"`).
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub item_type: Option<ScriptTypeTag>,
    /// The script source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script: Option<String>,
}
