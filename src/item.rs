//! Collection items: requests of all protocols, folders and script files.

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};
use serde_yaml_ng::Value;

use crate::common::{Description, Documentation, Sequence, Tag};
use crate::request::{GraphQlRequest, GrpcRequest, HttpRequest, RequestDefaults, WebSocketRequest};

/// An item in a collection or folder.
///
/// Serializes untagged (each variant is written as its own object). On the way
/// in, the `type` discriminator selects the variant when present — see the
/// hand-written [`Deserialize`] impl below.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Item {
    Http(HttpRequest),
    GraphQl(GraphQlRequest),
    Grpc(GrpcRequest),
    WebSocket(WebSocketRequest),
    Folder(Folder),
    Script(ScriptFile),
}

/// Deserializes an item by dispatching on its `type` discriminator.
///
/// A plain `#[serde(untagged)]` derive cannot do this: the tag lives at
/// `info.type` for requests and folders but at the top level for script files,
/// and serde's `tag = "..."` only supports a single fixed location. Untagged
/// matching also collapses every failure into one message —
/// *"data did not match any variant of untagged enum Item"* — which hides the
/// actual error (a typo in a nested field of a GraphQL request reported nothing
/// about GraphQL).
///
/// Dispatching on the tag propagates each variant's real error instead. The
/// schema leaves `type` optional, so items that omit it are selected by the
/// key identifying their shape — see [`shape_key_fallback`].
impl<'de> Deserialize<'de> for Item {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        let tag = value
            .get("info")
            .and_then(|info| info.get("type"))
            .or_else(|| value.get("type"))
            .and_then(Value::as_str);

        match tag {
            Some("http") => from_value(value).map(Item::Http),
            Some("graphql") => from_value(value).map(Item::GraphQl),
            Some("grpc") => from_value(value).map(Item::Grpc),
            Some("websocket") => from_value(value).map(Item::WebSocket),
            Some("folder") => from_value(value).map(Item::Folder),
            Some("script") => from_value(value).map(Item::Script),
            Some(unknown) => Err(D::Error::custom(format!(
                "unknown item type `{unknown}`, expected one of \
                 `http`, `graphql`, `grpc`, `websocket`, `folder`, `script`"
            ))),
            // `type` is optional in the schema; select on the shape key instead.
            None => shape_key_fallback(value),
        }
    }
}

fn from_value<T, E>(value: Value) -> Result<T, E>
where
    T: serde::de::DeserializeOwned,
    E: serde::de::Error,
{
    T::deserialize(value).map_err(E::custom)
}

/// Selects the variant for an item that carries no `type` tag, using the key
/// that identifies its shape.
///
/// This is a lookup, not a trial-and-error search. The schema defines `Item`
/// as a `oneOf`, meaning exactly one branch may match — and any item that is
/// actually valid carries exactly one of these keys. An object with none of
/// them (`{}`, or `{info: {name: x}}`) matches five or six branches at once,
/// so it is not a legal item; reporting that beats silently picking the first
/// shape that happens to fit.
fn shape_key_fallback<E>(value: Value) -> Result<Item, E>
where
    E: serde::de::Error,
{
    if value.get("http").is_some() {
        from_value(value).map(Item::Http)
    } else if value.get("graphql").is_some() {
        from_value(value).map(Item::GraphQl)
    } else if value.get("grpc").is_some() {
        from_value(value).map(Item::Grpc)
    } else if value.get("websocket").is_some() {
        from_value(value).map(Item::WebSocket)
    } else if value.get("items").is_some() || value.get("request").is_some() {
        from_value(value).map(Item::Folder)
    } else if value.get("script").is_some() {
        from_value(value).map(Item::Script)
    } else {
        Err(E::custom(
            "cannot tell what kind of item this is: it has no `type` \
             discriminator and none of the keys that identify a shape \
             (`http`, `graphql`, `grpc`, `websocket`, `items` or `script`). \
             The schema defines items as a `oneOf`, so such an object matches \
             several branches at once and is not a valid item",
        ))
    }
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

/// The literal item type `"folder"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FolderTypeTag {
    #[default]
    #[serde(rename = "folder")]
    Folder,
}

/// Folder metadata.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FolderInfo {
    /// The display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Description>,
    /// The item type discriminator.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub item_type: Option<FolderTypeTag>,
    /// Sequence number for ordering in a UI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seq: Option<Sequence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<Tag>>,
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

/// Fluent construction helpers.
impl Folder {
    /// Create a folder with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Folder {
            info: Some(FolderInfo {
                name: Some(name.into()),
                item_type: Some(FolderTypeTag::Folder),
                ..FolderInfo::default()
            }),
            ..Folder::default()
        }
    }

    /// Append an item to the folder.
    pub fn item(mut self, item: impl Into<Item>) -> Self {
        self.items.get_or_insert_with(Vec::new).push(item.into());
        self
    }
}

/// The literal item type `"script"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ScriptTypeTag {
    #[default]
    #[serde(rename = "script")]
    Script,
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
