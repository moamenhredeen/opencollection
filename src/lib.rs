//! A typed data model and YAML (de)serializer for the
//! [OpenCollection](https://spec.opencollection.com) API-collection format,
//! v1.0.0.
//!
//! OpenCollection is an open, YAML-based specification for API collections
//! (HTTP, GraphQL, gRPC and WebSocket requests, folders, environments,
//! scripts, …). This crate models the full schema with serde, so collections
//! can be parsed, inspected, modified and written back losslessly.
//!
//! # Example
//!
//! ```
//! use opencollection::{Auth, Environment, Folder, HttpRequest, OpenCollection};
//!
//! let collection = OpenCollection::new("Petstore")
//!     .summary("Example collection")
//!     .environment(
//!         Environment::new("prod")
//!             .variable("baseUrl", "https://api.example.com")
//!             .secret("apiKey"),
//!     )
//!     .item(
//!         Folder::new("Pets")
//!             .item(
//!                 HttpRequest::get("{{baseUrl}}/pets")
//!                     .name("List pets")
//!                     .query("limit", "10")
//!                     .auth(Auth::bearer("{{apiKey}}")),
//!             )
//!             .item(
//!                 HttpRequest::post("{{baseUrl}}/pets")
//!                     .name("Create pet")
//!                     .header("Content-Type", "application/json")
//!                     .json_body(r#"{"name": "Rex"}"#),
//!             ),
//!     );
//!
//! let yaml = collection.to_yaml().unwrap();
//! let parsed = OpenCollection::from_yaml(&yaml).unwrap();
//! assert_eq!(collection, parsed);
//! assert_eq!(parsed.iter().filter(|item| item.is_request()).count(), 2);
//! ```

mod auth;
mod body;
mod common;
mod config;
mod item;
mod request;
mod walk;

pub use auth::*;
pub use body::*;
pub use common::*;
pub use config::*;
pub use item::*;
pub use request::*;
pub use walk::ItemIter;

use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Free-form YAML value, re-exported for the [`OpenCollection::extensions`] field.
pub use serde_yaml_ng::Value;

/// An OpenCollection document (the root object of an `opencollection.yml`).
///
/// Unlike the nested types, the root object accepts unknown fields, matching
/// the schema.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct OpenCollection {
    /// Info about the collection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<Info>,
    /// The version of the OpenCollection spec this document targets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opencollection: Option<String>,
    /// Configuration for the collection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<CollectionConfig>,
    /// The items in the collection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<Item>>,
    /// Default request configuration for the collection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<RequestDefaults>,
    /// Documentation for the collection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs: Option<Documentation>,
    /// `true` if the collection is a standalone file, `false` if stored on the
    /// filesystem as a nested structure of folders and files.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundled: Option<bool>,
    /// Free-form extension data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<BTreeMap<String, Value>>,
}

impl OpenCollection {
    /// Create a new collection with the given name, targeting spec version `1.0.0`.
    ///
    /// This and the other fluent helpers are plain methods on the model types
    /// (all fields stay `pub`), so builder calls and direct field access mix
    /// freely.
    pub fn new(name: impl Into<String>) -> Self {
        OpenCollection {
            opencollection: Some("1.0.0".to_owned()),
            info: Some(Info {
                name: Some(name.into()),
                ..Info::default()
            }),
            ..OpenCollection::default()
        }
    }

    /// Set the collection summary.
    pub fn summary(mut self, summary: impl Into<String>) -> Self {
        self.info.get_or_insert_with(Info::default).summary = Some(summary.into());
        self
    }

    /// Set the collection version (the collection's own version, not the spec's).
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.info.get_or_insert_with(Info::default).version = Some(version.into());
        self
    }

    /// Append an item (request, folder or script file).
    pub fn item(mut self, item: impl Into<Item>) -> Self {
        self.items.get_or_insert_with(Vec::new).push(item.into());
        self
    }

    /// Append an environment to the collection config.
    pub fn environment(mut self, environment: Environment) -> Self {
        self.config
            .get_or_insert_with(CollectionConfig::default)
            .environments
            .get_or_insert_with(Vec::new)
            .push(environment);
        self
    }

    /// Parse a collection from YAML text.
    pub fn from_yaml(yaml: &str) -> Result<Self, Error> {
        Ok(serde_yaml_ng::from_str(yaml)?)
    }

    /// Read and parse a collection from a file (e.g. an `opencollection.yml`).
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, Error> {
        let yaml = std::fs::read_to_string(path)?;
        Self::from_yaml(&yaml)
    }

    /// Serialize the collection to YAML.
    pub fn to_yaml(&self) -> Result<String, Error> {
        Ok(serde_yaml_ng::to_string(self)?)
    }

    /// Serialize the collection to YAML and write it to a file.
    pub fn write_to_path(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        std::fs::write(path, self.to_yaml()?)?;
        Ok(())
    }

    /// Iterate depth-first over all items, descending into folders.
    pub fn iter(&self) -> ItemIter<'_> {
        ItemIter::new(self.items.as_deref().unwrap_or_default())
    }

    /// Iterate over all request items (HTTP, GraphQL, gRPC, WebSocket),
    /// descending into folders.
    pub fn requests(&self) -> impl Iterator<Item = &Item> {
        self.iter().filter(|item| item.is_request())
    }
}

impl std::str::FromStr for OpenCollection {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_yaml(s)
    }
}

/// Errors returned by this crate.
#[derive(Debug)]
pub enum Error {
    /// YAML (de)serialization failed.
    Yaml(serde_yaml_ng::Error),
    /// Reading or writing a file failed.
    Io(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Yaml(error) => write!(f, "YAML error: {error}"),
            Error::Io(error) => write!(f, "I/O error: {error}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Yaml(error) => Some(error),
            Error::Io(error) => Some(error),
        }
    }
}

impl From<serde_yaml_ng::Error> for Error {
    fn from(error: serde_yaml_ng::Error) -> Self {
        Error::Yaml(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Io(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_discrimination() {
        let yaml = r#"
- info:
    name: Get user
    type: http
  http:
    method: GET
    url: https://example.com/users/1
- info:
    name: Users
    type: folder
  items:
    - info:
        name: nested
      graphql:
        url: https://example.com/graphql
- type: script
  script: console.log("hi");
"#;
        let items: Vec<Item> = serde_yaml_ng::from_str(yaml).unwrap();
        assert!(matches!(items[0], Item::Http(_)));
        assert!(matches!(items[1], Item::Folder(_)));
        assert!(matches!(items[2], Item::Script(_)));
        let Item::Folder(folder) = &items[1] else {
            unreachable!()
        };
        assert!(matches!(
            folder.items.as_ref().unwrap()[0],
            Item::GraphQl(_)
        ));
    }

    #[test]
    fn name_only_folder_is_not_a_request() {
        let yaml = "info:\n  name: Just a folder\n  type: folder\n";
        let item: Item = serde_yaml_ng::from_str(yaml).unwrap();
        assert!(matches!(item, Item::Folder(_)));
    }

    #[test]
    fn auth_inherit_round_trip() {
        let auth: Auth = serde_yaml_ng::from_str("inherit").unwrap();
        assert_eq!(auth, Auth::Inherit);
        assert_eq!(serde_yaml_ng::to_string(&auth).unwrap().trim(), "inherit");
    }

    #[test]
    fn auth_oauth2_nested_tags() {
        let yaml = "type: oauth2\nflow: authorization_code\naccessTokenUrl: https://example.com/token\npkce:\n  method: S256\n";
        let auth: Auth = serde_yaml_ng::from_str(yaml).unwrap();
        let Auth::OAuth2(AuthOAuth2::AuthorizationCode(flow)) = &auth else {
            panic!("expected authorization code flow, got {auth:?}");
        };
        assert_eq!(flow.pkce.as_ref().unwrap().method, Some(PkceMethod::S256));
        let round_tripped: Auth =
            serde_yaml_ng::from_str(&serde_yaml_ng::to_string(&auth).unwrap()).unwrap();
        assert_eq!(auth, round_tripped);
    }

    #[test]
    fn secret_vs_plain_variables() {
        let yaml = "- name: plain\n  value: v\n- secret: true\n  name: hidden\n";
        let variables: Vec<EnvironmentVariable> = serde_yaml_ng::from_str(yaml).unwrap();
        assert!(matches!(variables[0], EnvironmentVariable::Plain(_)));
        assert!(matches!(variables[1], EnvironmentVariable::Secret(_)));
    }

    #[test]
    fn inheritable_settings() {
        let yaml = "timeout: inherit\nmaxRedirects: 5\n";
        let settings: HttpRequestSettings = serde_yaml_ng::from_str(yaml).unwrap();
        assert_eq!(settings.timeout, Some(Inheritable::Inherit));
        assert_eq!(
            settings.max_redirects,
            Some(Inheritable::Value(Number::Int(5)))
        );
        let round_tripped: HttpRequestSettings =
            serde_yaml_ng::from_str(&serde_yaml_ng::to_string(&settings).unwrap()).unwrap();
        assert_eq!(settings, round_tripped);
    }

    #[test]
    fn http_method_enum_round_trip() {
        // Known methods serialize as uppercase strings.
        assert_eq!(
            serde_yaml_ng::to_string(&HttpMethod::Get).unwrap().trim(),
            "GET"
        );
        // Any string parses; case is normalized for known methods.
        assert_eq!(HttpMethod::from("post"), HttpMethod::Post);
        // Unknown methods round-trip losslessly through `Other`.
        let custom: HttpMethod = serde_yaml_ng::from_str("PROPFIND").unwrap();
        assert_eq!(custom, HttpMethod::Other("PROPFIND".to_owned()));
        assert_eq!(
            serde_yaml_ng::to_string(&custom).unwrap().trim(),
            "PROPFIND"
        );
    }

    #[test]
    fn unknown_fields_rejected() {
        let yaml = "info:\n  name: x\nhttp:\n  method: GET\n  bogus: true\n";
        assert!(serde_yaml_ng::from_str::<HttpRequest>(yaml).is_err());
    }
}
