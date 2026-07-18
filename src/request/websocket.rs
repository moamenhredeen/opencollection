//! WebSocket requests.

use serde::{Deserialize, Serialize};

use crate::auth::Auth;
use crate::common::{Description, Inheritable, Number, Scripts, Sequence, Tag, Variable};
use crate::request::HttpRequestHeader;

/// The literal item type `"websocket"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WebSocketTypeTag {
    #[default]
    #[serde(rename = "websocket")]
    WebSocket,
}

/// WebSocket request metadata.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WebSocketRequestInfo {
    /// The display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Description>,
    /// The item type discriminator.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub item_type: Option<WebSocketTypeTag>,
    /// Sequence number for ordering in a UI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seq: Option<Sequence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<Tag>>,
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

/// Fluent construction helpers.
impl WebSocketRequest {
    /// Create a WebSocket request against the given URL.
    pub fn new(url: impl Into<String>) -> Self {
        WebSocketRequest {
            info: Some(WebSocketRequestInfo {
                item_type: Some(WebSocketTypeTag::WebSocket),
                ..WebSocketRequestInfo::default()
            }),
            websocket: Some(WebSocketRequestDetails {
                url: Some(url.into()),
                ..WebSocketRequestDetails::default()
            }),
            ..WebSocketRequest::default()
        }
    }

    /// Set the display name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.info
            .get_or_insert_with(WebSocketRequestInfo::default)
            .name = Some(name.into());
        self
    }
}
