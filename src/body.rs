//! Request body types for HTTP and GraphQL requests.

use serde::{Deserialize, Serialize};

use crate::common::Description;

/// An HTTP request body, tagged by `type`.
///
/// The schema's `RawBody` (`json`/`text`/`xml`/`sparql`) is flattened into
/// individual variants alongside the structured body kinds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum HttpRequestBody {
    /// Raw JSON body.
    #[serde(rename = "json")]
    Json { data: String },
    /// Raw text body.
    #[serde(rename = "text")]
    Text { data: String },
    /// Raw XML body.
    #[serde(rename = "xml")]
    Xml { data: String },
    /// Raw SPARQL body.
    #[serde(rename = "sparql")]
    Sparql { data: String },
    /// `application/x-www-form-urlencoded` body.
    #[serde(rename = "form-urlencoded")]
    FormUrlEncoded { data: Vec<FormField> },
    /// `multipart/form-data` body.
    #[serde(rename = "multipart-form")]
    MultipartForm { data: Vec<MultipartFormPart> },
    /// File body (one of the listed files is selected).
    #[serde(rename = "file")]
    File { data: Vec<FileBodyVariant> },
}

/// A form field in a [`HttpRequestBody::FormUrlEncoded`] body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormField {
    pub name: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Description>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
}

/// A part of a [`HttpRequestBody::MultipartForm`] body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MultipartFormPart {
    pub name: String,
    /// Whether this part is a text value or a file reference.
    #[serde(rename = "type")]
    pub part_type: MultipartPartType,
    /// The part value: a single string, or multiple (e.g. several file paths).
    pub value: MultipartValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Description>,
    /// The MIME type of the form part.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
}

/// The kind of a multipart form part.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MultipartPartType {
    Text,
    File,
}

/// A multipart form part value: one string or several.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MultipartValue {
    Single(String),
    Multiple(Vec<String>),
}

impl From<String> for MultipartValue {
    fn from(value: String) -> Self {
        MultipartValue::Single(value)
    }
}

impl From<&str> for MultipartValue {
    fn from(value: &str) -> Self {
        MultipartValue::Single(value.to_owned())
    }
}

/// A file variant in a [`HttpRequestBody::File`] body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FileBodyVariant {
    /// Path to the file.
    pub file_path: String,
    /// MIME type of the file.
    pub content_type: String,
    /// Whether this file is the selected one.
    pub selected: bool,
}

/// A named variant of an HTTP request body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HttpRequestBodyVariant {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected: Option<bool>,
    pub body: HttpRequestBody,
}

/// An HTTP request body: a single body or a list of selectable variants.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HttpBodyOrVariants {
    Body(HttpRequestBody),
    Variants(Vec<HttpRequestBodyVariant>),
}

impl From<HttpRequestBody> for HttpBodyOrVariants {
    fn from(body: HttpRequestBody) -> Self {
        HttpBodyOrVariants::Body(body)
    }
}

/// A GraphQL request body with query and variables.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphQlBody {
    /// The GraphQL query or mutation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    /// JSON string containing GraphQL variables.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<String>,
}

/// A named variant of a GraphQL body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphQlBodyVariant {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected: Option<bool>,
    pub body: GraphQlBody,
}

/// A GraphQL body: a single body or a list of selectable variants.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GraphQlBodyOrVariants {
    Body(GraphQlBody),
    Variants(Vec<GraphQlBodyVariant>),
}

impl From<GraphQlBody> for GraphQlBodyOrVariants {
    fn from(body: GraphQlBody) -> Self {
        GraphQlBodyOrVariants::Body(body)
    }
}
