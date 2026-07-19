//! Shared building blocks used across the collection model: descriptions,
//! variables, scripts, assertions and runtime actions.

use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Where a collection or item lives on disk.
///
/// Set by [`OpenCollection::load`](crate::OpenCollection::load) for unbundled
/// collections and used when saving, so an item is written back to the file it
/// came from rather than to a name derived afresh from its title.
///
/// That is what makes load → save → load idempotent. Deriving names at save
/// time renames every file in a tree whose names follow a different convention
/// — the normal case for collections written by other tools — and since the old
/// files are still there, every item is then read twice.
///
/// On an item this is a path *relative to the collection root*
/// (`pets/create-pet.yml`, or just `pets` for a folder, whose home is its
/// directory). On [`OpenCollection`](crate::OpenCollection) it is that root.
/// Relative item paths keep "load from A, save into B" correct at any nesting
/// depth, and survive moving the collection directory.
///
/// Not part of the schema, so it is skipped during (de)serialization. Assign
/// [`Source::default`] to clear it — a cloned item must be cleared, or the copy
/// and the original both claim one file.
#[derive(Clone, Default)]
pub struct Source(Option<PathBuf>);

impl Source {
    /// The path this was read from: relative to the collection root for an
    /// item, the root itself for a collection.
    ///
    /// `None` if it has never been saved.
    ///
    /// ```
    /// use opencollection::{HttpRequest, OpenCollection};
    /// # let dir = std::env::temp_dir().join("opencollection-doc-source-path");
    /// # let _ = std::fs::remove_dir_all(&dir);
    /// # let mut seed = OpenCollection::new("Petstore")
    /// #     .item(HttpRequest::get("https://example.com/pets").name("List pets"));
    /// # seed.bundled = Some(false);
    /// # seed.save(&dir)?;
    /// let collection = OpenCollection::load(&dir)?;
    ///
    /// // The collection remembers its root; items remember their place in it.
    /// assert_eq!(collection.source.path(), Some(dir.as_path()));
    ///
    /// // A collection built in memory has neither.
    /// assert_eq!(OpenCollection::new("Fresh").source.path(), None);
    /// # let _ = std::fs::remove_dir_all(&dir);
    /// # Ok::<(), opencollection::Error>(())
    /// ```
    pub fn path(&self) -> Option<&Path> {
        self.0.as_deref()
    }

    pub(crate) fn at(path: impl Into<PathBuf>) -> Self {
        Source(Some(path.into()))
    }
}

/// Provenance is deliberately invisible to equality.
///
/// These types model a *document*, and where a copy of it happened to be read
/// from is not part of the document: two collections with the same content are
/// equal whether they came from disk, from a builder, or from two different
/// directories. Tests comparing a built collection against a loaded one depend
/// on this.
///
/// It also keeps `#[derive(PartialEq)]` working on all six item types. The
/// alternative — hand-written impls that skip one field — is six chances to
/// forget a *new* field when the schema grows.
impl PartialEq for Source {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

/// Prints just the file name, so an `assert_eq!` failure on a collection is not
/// buried in absolute paths repeated inside every item.
impl fmt::Debug for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.as_deref().and_then(Path::file_name) {
            Some(name) => write!(f, "Source({name:?})"),
            None => f.write_str("Source(None)"),
        }
    }
}

/// A description, either as plain text or as content with an explicit MIME type.
///
/// The schema also allows `null`; model that by wrapping in `Option<Description>`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Description {
    /// Plain text description.
    Text(String),
    /// Description content with an explicit MIME type (e.g. `text/markdown`).
    Content(DescriptionContent),
}

/// The object form of a [`Description`]: content plus an explicit MIME type.
///
/// This is a named struct rather than an inline enum variant so that
/// `deny_unknown_fields` applies — serde ignores the attribute on variants of
/// an `untagged` enum, which would silently accept and drop unknown keys.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DescriptionContent {
    /// The description text.
    pub content: String,
    /// The MIME type of the content (e.g. `text/markdown`).
    #[serde(rename = "type")]
    pub mime_type: String,
}

impl From<String> for Description {
    fn from(value: String) -> Self {
        Description::Text(value)
    }
}

impl From<&str> for Description {
    fn from(value: &str) -> Self {
        Description::Text(value.to_owned())
    }
}

/// Documentation shares the exact same shape as [`Description`].
pub type Documentation = Description;

/// A JSON/YAML number, preserving whether it was written as an integer or a
/// float so round-trips are lossless.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Number {
    Int(i64),
    Float(f64),
}

impl Number {
    /// The value as an `f64` (integers are converted).
    pub fn as_f64(self) -> f64 {
        match self {
            Number::Int(value) => value as f64,
            Number::Float(value) => value,
        }
    }
}

impl From<i64> for Number {
    fn from(value: i64) -> Self {
        Number::Int(value)
    }
}

impl From<f64> for Number {
    fn from(value: f64) -> Self {
        Number::Float(value)
    }
}

impl std::fmt::Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Number::Int(value) => value.fmt(f),
            Number::Float(value) => value.fmt(f),
        }
    }
}

/// Sequence number used to represent the order of an item when rendered in a UI.
pub type Sequence = Number;

/// A tag for categorizing or labeling items.
pub type Tag = String;

/// The literal string `"inherit"`. Used internally to model `oneOf` schemas
/// that allow a value or the string `"inherit"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum InheritTag {
    #[serde(rename = "inherit")]
    Inherit,
}

/// A setting that is either an explicit value or inherited from the parent scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Inheritable<T> {
    /// An explicit value.
    Value(T),
    /// The literal string `"inherit"`.
    Inherit,
}

impl<T: Serialize> Serialize for Inheritable<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Inheritable::Value(value) => value.serialize(serializer),
            Inheritable::Inherit => serializer.serialize_str("inherit"),
        }
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Inheritable<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Repr<T> {
            Value(T),
            Inherit(InheritTag),
        }
        Ok(match Repr::deserialize(deserializer)? {
            Repr::Value(value) => Inheritable::Value(value),
            Repr::Inherit(_) => Inheritable::Inherit,
        })
    }
}

/// The JSON literal `true`. Marker type for [`SecretVariable::secret`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct True;

impl Serialize for True {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bool(true)
    }
}

impl<'de> Deserialize<'de> for True {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if bool::deserialize(deserializer)? {
            Ok(True)
        } else {
            Err(serde::de::Error::custom("expected the literal `true`"))
        }
    }
}

/// Info about the collection.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Info {
    /// The name of the collection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// A short summary of the collection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// The version of the collection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// The authors of the collection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authors: Option<Vec<Author>>,
}

/// An author of the collection.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Author {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// A variable with name, value, description and state flags.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Variable {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<VariableValueOrVariants>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Description>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
}

/// A variable value: either a single value or a list of selectable variants.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VariableValueOrVariants {
    Value(VariableValue),
    Variants(Vec<VariableValueVariant>),
}

/// A variable value (plain string or typed value).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VariableValue {
    /// A plain string value.
    Text(String),
    /// A typed value with its string representation.
    Typed {
        #[serde(rename = "type")]
        value_type: ValueType,
        data: String,
    },
}

impl From<String> for VariableValue {
    fn from(value: String) -> Self {
        VariableValue::Text(value)
    }
}

impl From<&str> for VariableValue {
    fn from(value: &str) -> Self {
        VariableValue::Text(value.to_owned())
    }
}

/// The data type of a typed variable value or secret variable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValueType {
    String,
    Number,
    Boolean,
    Null,
    Object,
}

/// A variant of a variable value with title, selected state, and value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VariableValueVariant {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected: Option<bool>,
    pub value: VariableValue,
}

/// A secret variable: its value is not stored in the collection.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SecretVariable {
    /// Always `true`; marks this entry as a secret variable.
    pub secret: True,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Description>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub value_type: Option<ValueType>,
}

/// An environment variable entry: a plain variable or a secret variable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EnvironmentVariable {
    Secret(SecretVariable),
    Plain(Variable),
}

/// A script to execute at a specific lifecycle stage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Script {
    /// The lifecycle stage when this script executes.
    #[serde(rename = "type")]
    pub phase: ScriptPhase,
    /// The script code.
    pub code: String,
}

/// Lifecycle stage of a [`Script`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ScriptPhase {
    BeforeRequest,
    AfterResponse,
    Tests,
    Hooks,
}

/// Scripts for the collection execution lifecycle.
pub type Scripts = Vec<Script>;

/// An assertion for response validation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Assertion {
    /// The expression to evaluate.
    pub expression: String,
    /// The comparison operator.
    pub operator: String,
    /// The expected value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Description>,
}

/// A runtime action.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Action {
    /// Set a variable using a selector result.
    #[serde(rename = "set-variable")]
    SetVariable(ActionSetVariable),
}

/// Set a variable using a selector result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionSetVariable {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Description>,
    /// When to execute the action relative to the request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<ActionPhase>,
    pub selector: ActionSelector,
    pub variable: ActionVariable,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
}

/// When an [`Action`] executes relative to the request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ActionPhase {
    BeforeRequest,
    AfterResponse,
}

/// Selector used by an action.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionSelector {
    /// Selector expression to evaluate.
    pub expression: String,
    /// Selector evaluation method.
    pub method: SelectorMethod,
}

/// Selector evaluation method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SelectorMethod {
    #[default]
    #[serde(rename = "jsonq")]
    Jsonq,
}

/// Target variable details for an action.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionVariable {
    /// Variable name to set.
    pub name: String,
    /// Scope in which to set the variable.
    pub scope: VariableScope,
}

/// Scope in which an action sets a variable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VariableScope {
    Runtime,
    Request,
    Folder,
    Collection,
    Environment,
}

/// Fluent construction helpers.
impl Variable {
    /// Create a variable with a name and value.
    pub fn new(name: impl Into<String>, value: impl Into<VariableValue>) -> Self {
        Variable {
            name: Some(name.into()),
            value: Some(VariableValueOrVariants::Value(value.into())),
            ..Variable::default()
        }
    }
}
