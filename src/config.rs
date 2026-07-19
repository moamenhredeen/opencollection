//! Collection-level configuration: environments, protobuf, proxy and
//! client certificates.

use serde::{Deserialize, Serialize};

use crate::common::{
    Description, EnvironmentVariable, Number, SecretVariable, True, Variable, VariableValue,
};

/// Configuration for the collection.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CollectionConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environments: Option<Vec<Environment>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protobuf: Option<Protobuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<Proxy>,
    /// Client certificates for mutual TLS authentication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_certificates: Option<Vec<ClientCertificate>>,
}

/// An environment configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Environment {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Description>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<Vec<EnvironmentVariable>>,
    /// Client certificates for mutual TLS authentication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_certificates: Option<Vec<ClientCertificate>>,
    /// The name of the environment to extend from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extends: Option<String>,
    /// Path to a `.env` file to load variables from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dot_env_file_path: Option<String>,
}

/// A client certificate for mutual TLS, tagged by `type`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientCertificate {
    /// Separate PEM-encoded certificate and key files.
    #[serde(rename = "pem")]
    Pem(PemCertificate),
    /// PKCS#12 (PFX) bundle.
    #[serde(rename = "pkcs12")]
    Pkcs12(Pkcs12Certificate),
}

/// A client certificate using separate PEM-encoded cert and key files.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PemCertificate {
    /// The domain this certificate applies to.
    pub domain: String,
    pub certificate_file_path: String,
    pub private_key_file_path: String,
    /// Passphrase for the private key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passphrase: Option<String>,
}

/// A client certificate in PKCS#12 (PFX) format.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Pkcs12Certificate {
    /// The domain this certificate applies to.
    pub domain: String,
    pub pkcs12_file_path: String,
    /// Passphrase for the PKCS#12 file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passphrase: Option<String>,
}

/// Protobuf configuration.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Protobuf {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proto_files: Option<Vec<ProtoFileItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_paths: Option<Vec<ProtoFileImportPath>>,
}

/// A proto file reference, tagged by `type`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ProtoFileItem {
    /// A proto file on disk.
    #[serde(rename = "file")]
    File(ProtoFile),
}

/// A proto file reference.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProtoFile {
    /// Path to the proto file.
    pub path: String,
}

/// A proto file import path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProtoFileImportPath {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
}

/// Proxy configuration for the collection.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Proxy {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    /// Whether to inherit system proxy settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inherit: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<ProxyConnectionConfig>,
}

/// Proxy connection details.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProxyConnectionConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<Number>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<ProxyAuth>,
    /// Bypass proxy string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bypass_proxy: Option<String>,
}

/// Proxy authentication.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProxyAuth {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

/// Fluent construction helpers.
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

    /// Append a variable with a value stored in the collection.
    ///
    /// Use [`secret`](Self::secret) for anything that must not be committed.
    ///
    /// ```
    /// use opencollection::{Environment, EnvironmentVariable};
    ///
    /// let environment = Environment::new("prod")
    ///     .variable("baseUrl", "https://api.example.com")
    ///     .secret("apiKey");
    ///
    /// let variables = environment.variables.unwrap();
    /// assert!(matches!(variables[0], EnvironmentVariable::Plain(_)));
    /// assert!(matches!(variables[1], EnvironmentVariable::Secret(_)));
    /// ```
    pub fn variable(mut self, name: impl Into<String>, value: impl Into<VariableValue>) -> Self {
        self.variables
            .get_or_insert_with(Vec::new)
            .push(EnvironmentVariable::Plain(Variable::new(name, value)));
        self
    }

    /// Declare a secret variable: the name is recorded, the value is not.
    ///
    /// This is how the spec keeps credentials out of a collection that gets
    /// committed or shared. The variable is declared so tooling knows it exists;
    /// supplying the value at run time is the client's job, often from the
    /// environment's [`dot_env_file_path`](Environment::dot_env_file_path).
    ///
    /// See [`variable`](Self::variable) for an example of both together.
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
