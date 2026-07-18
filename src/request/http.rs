//! HTTP requests, and the header/parameter/settings types shared with the
//! other request protocols.

use serde::{Deserialize, Serialize};

use crate::auth::Auth;
use crate::body::{HttpBodyOrVariants, HttpRequestBody};
use crate::common::{
    Action, Assertion, Description, Inheritable, Number, Script, ScriptPhase, Scripts, Sequence,
    Tag, Variable,
};

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

/// The literal item type `"http"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum HttpTypeTag {
    #[default]
    #[serde(rename = "http")]
    Http,
}

/// HTTP request metadata.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HttpRequestInfo {
    /// The display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Description>,
    /// The item type discriminator.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub item_type: Option<HttpTypeTag>,
    /// Sequence number for ordering in a UI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seq: Option<Sequence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<Tag>>,
}

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

/// Fluent construction helpers.
///
/// These are plain methods on the model types (all fields stay `pub`), so you
/// can freely mix builder calls with direct field access.
impl HttpRequest {
    /// Create a request with the given HTTP method and URL.
    pub fn new(method: impl Into<String>, url: impl Into<String>) -> Self {
        HttpRequest {
            info: Some(HttpRequestInfo {
                item_type: Some(HttpTypeTag::Http),
                ..HttpRequestInfo::default()
            }),
            http: Some(HttpRequestDetails {
                method: Some(method.into()),
                url: Some(url.into()),
                ..HttpRequestDetails::default()
            }),
            ..HttpRequest::default()
        }
    }

    /// Create a GET request.
    pub fn get(url: impl Into<String>) -> Self {
        HttpRequest::new("GET", url)
    }

    /// Create a POST request.
    pub fn post(url: impl Into<String>) -> Self {
        HttpRequest::new("POST", url)
    }

    /// Create a PUT request.
    pub fn put(url: impl Into<String>) -> Self {
        HttpRequest::new("PUT", url)
    }

    /// Create a PATCH request.
    pub fn patch(url: impl Into<String>) -> Self {
        HttpRequest::new("PATCH", url)
    }

    /// Create a DELETE request.
    pub fn delete(url: impl Into<String>) -> Self {
        HttpRequest::new("DELETE", url)
    }

    /// Set the display name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.info.get_or_insert_with(HttpRequestInfo::default).name = Some(name.into());
        self
    }

    /// Set the description.
    pub fn description(mut self, description: impl Into<Description>) -> Self {
        self.info
            .get_or_insert_with(HttpRequestInfo::default)
            .description = Some(description.into());
        self
    }

    /// Append a header.
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.details_mut()
            .headers
            .get_or_insert_with(Vec::new)
            .push(HttpRequestHeader {
                name: name.into(),
                value: value.into(),
                description: None,
                disabled: None,
            });
        self
    }

    /// Append a query parameter.
    pub fn query(self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.param(name, value, ParamType::Query)
    }

    /// Append a path parameter.
    pub fn path_param(self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.param(name, value, ParamType::Path)
    }

    fn param(
        mut self,
        name: impl Into<String>,
        value: impl Into<String>,
        param_type: ParamType,
    ) -> Self {
        self.details_mut()
            .params
            .get_or_insert_with(Vec::new)
            .push(HttpRequestParam {
                name: name.into(),
                value: value.into(),
                description: None,
                param_type,
                disabled: None,
            });
        self
    }

    /// Set the request body.
    pub fn body(mut self, body: HttpRequestBody) -> Self {
        self.details_mut().body = Some(body.into());
        self
    }

    /// Set a raw JSON body.
    pub fn json_body(self, data: impl Into<String>) -> Self {
        self.body(HttpRequestBody::Json { data: data.into() })
    }

    /// Set a raw text body.
    pub fn text_body(self, data: impl Into<String>) -> Self {
        self.body(HttpRequestBody::Text { data: data.into() })
    }

    /// Set the authentication.
    pub fn auth(mut self, auth: Auth) -> Self {
        self.details_mut().auth = Some(auth);
        self
    }

    /// Append a lifecycle script.
    pub fn script(mut self, phase: ScriptPhase, code: impl Into<String>) -> Self {
        self.runtime
            .get_or_insert_with(Default::default)
            .scripts
            .get_or_insert_with(Vec::new)
            .push(Script {
                phase,
                code: code.into(),
            });
        self
    }

    fn details_mut(&mut self) -> &mut HttpRequestDetails {
        self.http.get_or_insert_with(HttpRequestDetails::default)
    }
}
