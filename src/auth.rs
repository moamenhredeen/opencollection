//! Authentication schemes.

use serde::{Deserialize, Serialize};

use crate::common::InheritTag;

/// Authentication configuration for a request, folder or collection.
///
/// Matches the schema's `Auth` union: one of the concrete schemes, or the
/// literal string `"inherit"` to inherit auth from the parent scope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(from = "AuthRepr", into = "AuthRepr")]
pub enum Auth {
    /// Inherit authentication from the parent scope (the string `"inherit"`).
    Inherit,
    /// AWS Signature Version 4.
    AwsV4(AuthAwsV4),
    /// HTTP Basic authentication.
    Basic(AuthBasic),
    /// WSSE authentication.
    Wsse(AuthWsse),
    /// Bearer token authentication.
    Bearer(AuthBearer),
    /// HTTP Digest authentication.
    Digest(AuthDigest),
    /// NTLM authentication.
    Ntlm(AuthNtlm),
    /// API key authentication.
    ApiKey(AuthApiKey),
    /// OAuth 1.0 authentication.
    OAuth1(AuthOAuth1),
    /// OAuth 2.0 authentication.
    OAuth2(AuthOAuth2),
}

/// Private serde representation of [`Auth`]: either the `type`-tagged object
/// or the literal string `"inherit"`.
#[derive(Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum AuthRepr {
    Tagged(Box<TaggedAuth>),
    Inherit(InheritTag),
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum TaggedAuth {
    #[serde(rename = "awsv4")]
    AwsV4(AuthAwsV4),
    #[serde(rename = "basic")]
    Basic(AuthBasic),
    #[serde(rename = "wsse")]
    Wsse(AuthWsse),
    #[serde(rename = "bearer")]
    Bearer(AuthBearer),
    #[serde(rename = "digest")]
    Digest(AuthDigest),
    #[serde(rename = "ntlm")]
    Ntlm(AuthNtlm),
    #[serde(rename = "apikey")]
    ApiKey(AuthApiKey),
    #[serde(rename = "oauth1")]
    OAuth1(AuthOAuth1),
    #[serde(rename = "oauth2")]
    OAuth2(AuthOAuth2),
}

impl From<AuthRepr> for Auth {
    fn from(repr: AuthRepr) -> Self {
        match repr {
            AuthRepr::Inherit(_) => Auth::Inherit,
            AuthRepr::Tagged(tagged) => match *tagged {
                TaggedAuth::AwsV4(auth) => Auth::AwsV4(auth),
                TaggedAuth::Basic(auth) => Auth::Basic(auth),
                TaggedAuth::Wsse(auth) => Auth::Wsse(auth),
                TaggedAuth::Bearer(auth) => Auth::Bearer(auth),
                TaggedAuth::Digest(auth) => Auth::Digest(auth),
                TaggedAuth::Ntlm(auth) => Auth::Ntlm(auth),
                TaggedAuth::ApiKey(auth) => Auth::ApiKey(auth),
                TaggedAuth::OAuth1(auth) => Auth::OAuth1(auth),
                TaggedAuth::OAuth2(auth) => Auth::OAuth2(auth),
            },
        }
    }
}

impl From<Auth> for AuthRepr {
    fn from(auth: Auth) -> Self {
        match auth {
            Auth::Inherit => AuthRepr::Inherit(InheritTag::Inherit),
            Auth::AwsV4(auth) => AuthRepr::Tagged(Box::new(TaggedAuth::AwsV4(auth))),
            Auth::Basic(auth) => AuthRepr::Tagged(Box::new(TaggedAuth::Basic(auth))),
            Auth::Wsse(auth) => AuthRepr::Tagged(Box::new(TaggedAuth::Wsse(auth))),
            Auth::Bearer(auth) => AuthRepr::Tagged(Box::new(TaggedAuth::Bearer(auth))),
            Auth::Digest(auth) => AuthRepr::Tagged(Box::new(TaggedAuth::Digest(auth))),
            Auth::Ntlm(auth) => AuthRepr::Tagged(Box::new(TaggedAuth::Ntlm(auth))),
            Auth::ApiKey(auth) => AuthRepr::Tagged(Box::new(TaggedAuth::ApiKey(auth))),
            Auth::OAuth1(auth) => AuthRepr::Tagged(Box::new(TaggedAuth::OAuth1(auth))),
            Auth::OAuth2(auth) => AuthRepr::Tagged(Box::new(TaggedAuth::OAuth2(auth))),
        }
    }
}

/// AWS Signature Version 4 authentication.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthAwsV4 {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_key_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_access_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_name: Option<String>,
}

/// HTTP Basic authentication.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthBasic {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

/// WSSE authentication.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthWsse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

/// Bearer token authentication.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthBearer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

/// HTTP Digest authentication.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthDigest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

/// NTLM authentication.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthNtlm {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
}

/// API key authentication.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthApiKey {
    /// API key name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// API key value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Where to place the API key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placement: Option<ApiKeyPlacement>,
}

/// Where an API key is placed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApiKeyPlacement {
    Header,
    Query,
}

/// OAuth 1.0 authentication.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthOAuth1 {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consumer_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consumer_secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token_secret: Option<String>,
    /// Callback URL for the Temporary Credentials Request (RFC 5849 §2.1).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_url: Option<String>,
    /// Verification code from the Resource Owner Authorization step (RFC 5849 §2.2).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_method: Option<OAuth1SignatureMethod>,
    /// Private key (PEM format, required for RSA-* signature methods).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_key: Option<OAuth1PrivateKey>,
    /// Custom timestamp (auto-generated if not provided).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    /// Custom nonce (auto-generated if not provided).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
    /// OAuth version (defaults to `"1.0"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub realm: Option<String>,
    /// Where to add OAuth parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placement: Option<ParameterPlacement>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_body_hash: Option<bool>,
}

/// OAuth 1.0 signature method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OAuth1SignatureMethod {
    #[serde(rename = "HMAC-SHA1")]
    HmacSha1,
    #[serde(rename = "HMAC-SHA256")]
    HmacSha256,
    #[serde(rename = "HMAC-SHA512")]
    HmacSha512,
    #[serde(rename = "RSA-SHA1")]
    RsaSha1,
    #[serde(rename = "RSA-SHA256")]
    RsaSha256,
    #[serde(rename = "RSA-SHA512")]
    RsaSha512,
    #[serde(rename = "PLAINTEXT")]
    Plaintext,
}

/// An OAuth 1.0 private key, inline or referenced by file path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OAuth1PrivateKey {
    /// `text` for an inline key, `file` for a file path.
    #[serde(rename = "type")]
    pub source: KeySource,
    pub value: String,
}

/// How an [`OAuth1PrivateKey`] value is provided.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KeySource {
    File,
    Text,
}

/// Where a parameter is sent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParameterPlacement {
    Header,
    Query,
    Body,
}

/// OAuth 2.0 authentication, tagged by grant flow.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "flow")]
pub enum AuthOAuth2 {
    #[serde(rename = "client_credentials")]
    ClientCredentials(OAuth2ClientCredentialsFlow),
    #[serde(rename = "resource_owner_password_credentials")]
    ResourceOwnerPassword(OAuth2ResourceOwnerPasswordFlow),
    #[serde(rename = "authorization_code")]
    AuthorizationCode(OAuth2AuthorizationCodeFlow),
    #[serde(rename = "implicit")]
    Implicit(OAuth2ImplicitFlow),
}

/// OAuth 2.0 Client Credentials flow.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OAuth2ClientCredentialsFlow {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credentials: Option<OAuth2ClientCredentials>,
    /// Space-delimited OAuth 2.0 scopes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_parameters: Option<OAuth2TokenRequestParameters>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_config: Option<OAuth2TokenConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<OAuth2Settings>,
}

/// OAuth 2.0 Resource Owner Password Credentials flow.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OAuth2ResourceOwnerPasswordFlow {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credentials: Option<OAuth2ClientCredentials>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_owner: Option<OAuth2ResourceOwner>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_parameters: Option<OAuth2TokenRequestParameters>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_config: Option<OAuth2TokenConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<OAuth2Settings>,
}

/// OAuth 2.0 Authorization Code flow.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OAuth2AuthorizationCodeFlow {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credentials: Option<OAuth2ClientCredentials>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// Opaque value used for CSRF protection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pkce: Option<OAuth2Pkce>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_parameters: Option<OAuth2AuthorizationCodeParameters>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_config: Option<OAuth2TokenConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<OAuth2Settings>,
}

/// OAuth 2.0 Implicit flow.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OAuth2ImplicitFlow {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_url: Option<String>,
    /// Client credentials (the implicit flow only needs a client id).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credentials: Option<OAuth2ImplicitCredentials>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_parameters: Option<OAuth2AuthorizationRequestParameters>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_config: Option<OAuth2TokenConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<OAuth2Settings>,
}

/// OAuth 2.0 client credentials configuration.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OAuth2ClientCredentials {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
    /// Where credentials are placed in the request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placement: Option<CredentialsPlacement>,
}

/// Where OAuth 2.0 client credentials are placed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialsPlacement {
    BasicAuthHeader,
    Body,
}

/// Client credentials for the implicit flow (client id only).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OAuth2ImplicitCredentials {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
}

/// Resource owner credentials.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OAuth2ResourceOwner {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

/// PKCE (Proof Key for Code Exchange) configuration.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OAuth2Pkce {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    /// Code challenge method.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<PkceMethod>,
}

/// PKCE code challenge method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PkceMethod {
    S256,
    #[serde(rename = "plain")]
    Plain,
}

/// Additional parameter for OAuth 2.0 requests.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OAuth2AdditionalParameter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Where to send this parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placement: Option<ParameterPlacement>,
}

/// Additional parameters for flows with token requests only
/// (client credentials, resource owner password).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OAuth2TokenRequestParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token_request: Option<Vec<OAuth2AdditionalParameter>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token_request: Option<Vec<OAuth2AdditionalParameter>>,
}

/// Additional parameters for the authorization code flow.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OAuth2AuthorizationCodeParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_request: Option<Vec<OAuth2AdditionalParameter>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token_request: Option<Vec<OAuth2AdditionalParameter>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token_request: Option<Vec<OAuth2AdditionalParameter>>,
}

/// Additional parameters for the implicit flow (authorization request only).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OAuth2AuthorizationRequestParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_request: Option<Vec<OAuth2AdditionalParameter>>,
}

/// Configuration for how OAuth 2.0 tokens are stored and transported.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OAuth2TokenConfig {
    /// Reference identifier for the token (used to access it via scripting APIs).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Where the token is placed in requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placement: Option<OAuth2TokenPlacement>,
    /// Which token to use for authorization (defaults to `access_token`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<OAuth2TokenSource>,
}

/// Where an OAuth 2.0 token is placed in requests.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OAuth2TokenPlacement {
    /// Token placed in an HTTP header (e.g. `Authorization`).
    Header { header: String },
    /// Token placed in a query parameter (e.g. `access_token`).
    Query { query: String },
}

/// Which OAuth 2.0 token is used for authorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OAuth2TokenSource {
    AccessToken,
    IdToken,
}

/// OAuth 2.0 automation settings.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OAuth2Settings {
    /// Automatically fetch a new token when accessing the resource without one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_fetch_token: Option<bool>,
    /// Automatically refresh the token using `refreshTokenUrl` when it expires.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_refresh_token: Option<bool>,
}

/// Fluent construction helpers.
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
