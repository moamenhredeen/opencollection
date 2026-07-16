# opencollection

A typed Rust data model and YAML (de)serializer for the
[OpenCollection](https://spec.opencollection.com) API-collection format
(v1.0.0) — the open, YAML-based collection spec created by
[Bruno](https://docs.usebruno.com/opencollection-yaml/overview).

The full JSON Schema is modeled with serde: HTTP, GraphQL, gRPC and WebSocket
requests, folders, script files, environments (with secret variables and value
variants), every auth scheme (Basic, Bearer, Digest, NTLM, WSSE, API key,
AWS SigV4, OAuth 1.0 and all four OAuth 2.0 flows, plus `inherit`), all body
kinds, assertions, runtime actions, proxy/protobuf/client-certificate config,
and `value | "inherit"` settings.

- **Lossless**: parse → modify → serialize round-trips are value-identical.
- **Strict**: unknown fields are rejected wherever the schema forbids them.
- **Validated**: the test suite checks serialized output against the official
  schema at `schema.opencollection.com`.

## Usage

```rust
use opencollection::{Auth, Environment, Folder, HttpRequest, OpenCollection};

// Parse an existing collection
let collection = OpenCollection::from_path("opencollection.yml")?;
for item in collection.requests() {
    println!("{}", item.name().unwrap_or("<unnamed>"));
}

// Or build one fluently
let collection = OpenCollection::new("Petstore")
    .environment(
        Environment::new("prod")
            .variable("baseUrl", "https://api.example.com")
            .secret("apiKey"),
    )
    .item(
        Folder::new("Pets").item(
            HttpRequest::get("{{baseUrl}}/pets")
                .name("List pets")
                .query("limit", "10")
                .auth(Auth::bearer("{{apiKey}}")),
        ),
    );
collection.write_to_path("opencollection.yml")?;
```

All fields are public, so the builder methods can be freely mixed with direct
struct access.

## License

MIT
