//! Collection items: requests of all protocols, folders, script files and apps.

use serde::{Deserialize, Serialize};

use crate::common::{Description, Documentation, Sequence, Tag};
use crate::request::{GraphQlRequest, GrpcRequest, HttpRequest, RequestDefaults, WebSocketRequest};

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
    App(App),
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
            Item::App(app) => app.info.as_ref()?.name.as_deref(),
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

impl From<App> for Item {
    fn from(app: App) -> Self {
        Item::App(app)
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

/// The literal item type `"app"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AppTypeTag {
    #[default]
    #[serde(rename = "app")]
    App,
}

/// App metadata.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppInfo {
    /// The display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Description>,
    /// The item type discriminator.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub item_type: Option<AppTypeTag>,
    /// Sequence number for ordering in a UI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seq: Option<Sequence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<Tag>>,
}

/// An app item: an embedded application with its own code.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct App {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<AppInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}
