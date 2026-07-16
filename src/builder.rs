//! Fluent construction helpers.
//!
//! These are plain methods on the model types (all fields stay `pub`), so you
//! can freely mix builder calls with direct field access.

use crate::OpenCollection;
use crate::auth::{Auth, AuthBasic, AuthBearer};
use crate::body::HttpRequestBody;
use crate::common::{
    Description, Script, ScriptPhase, SecretVariable, True, Variable, VariableValue,
    VariableValueOrVariants,
};
use crate::common::{EnvironmentVariable, Info};
use crate::config::{CollectionConfig, Environment};
use crate::request::{
    Folder, GraphQlRequest, GraphQlRequestDetails, GraphQlRequestInfo, GraphQlTypeTag, GrpcRequest,
    GrpcRequestDetails, GrpcRequestInfo, GrpcTypeTag, HttpRequest, HttpRequestDetails,
    HttpRequestHeader, HttpRequestInfo, HttpRequestParam, HttpTypeTag, Item, ParamType,
    WebSocketRequest, WebSocketRequestDetails, WebSocketRequestInfo, WebSocketTypeTag,
};

impl OpenCollection {
    /// Create a new collection with the given name, targeting spec version `1.0.0`.
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
}

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

impl GraphQlRequest {
    /// Create a GraphQL request against the given URL.
    pub fn new(url: impl Into<String>) -> Self {
        GraphQlRequest {
            info: Some(GraphQlRequestInfo {
                item_type: Some(GraphQlTypeTag::GraphQl),
                ..GraphQlRequestInfo::default()
            }),
            graphql: Some(GraphQlRequestDetails {
                method: Some("POST".to_owned()),
                url: Some(url.into()),
                ..GraphQlRequestDetails::default()
            }),
            ..GraphQlRequest::default()
        }
    }

    /// Set the display name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.info
            .get_or_insert_with(GraphQlRequestInfo::default)
            .name = Some(name.into());
        self
    }

    /// Set the GraphQL query (and optional variables as a JSON string).
    pub fn query(mut self, query: impl Into<String>, variables: Option<String>) -> Self {
        self.graphql
            .get_or_insert_with(GraphQlRequestDetails::default)
            .body = Some(
            crate::body::GraphQlBody {
                query: Some(query.into()),
                variables,
            }
            .into(),
        );
        self
    }
}

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

impl Folder {
    /// Create a folder with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Folder {
            info: Some(crate::request::FolderInfo {
                name: Some(name.into()),
                item_type: Some(crate::request::FolderTypeTag::Folder),
                ..crate::request::FolderInfo::default()
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

impl Environment {
    /// Create an environment with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Environment {
            name: name.into(),
            color: None,
            description: None,
            variables: None,
            client_certificates: None,
            extends: None,
            dot_env_file_path: None,
        }
    }

    /// Append a plain variable.
    pub fn variable(mut self, name: impl Into<String>, value: impl Into<VariableValue>) -> Self {
        self.variables
            .get_or_insert_with(Vec::new)
            .push(EnvironmentVariable::Plain(Variable::new(name, value)));
        self
    }

    /// Append a secret variable (name only; the value lives outside the collection).
    pub fn secret(mut self, name: impl Into<String>) -> Self {
        self.variables
            .get_or_insert_with(Vec::new)
            .push(EnvironmentVariable::Secret(SecretVariable {
                secret: True,
                name: Some(name.into()),
                ..SecretVariable::default()
            }));
        self
    }
}

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

impl Auth {
    /// HTTP Basic auth with username and password.
    pub fn basic(username: impl Into<String>, password: impl Into<String>) -> Self {
        Auth::Basic(AuthBasic {
            username: Some(username.into()),
            password: Some(password.into()),
        })
    }

    /// Bearer token auth.
    pub fn bearer(token: impl Into<String>) -> Self {
        Auth::Bearer(AuthBearer {
            token: Some(token.into()),
        })
    }
}
