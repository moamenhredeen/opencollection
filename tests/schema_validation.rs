//! Validate serialized collections against the official OpenCollection
//! v1.0.0 JSON Schema.

use opencollection::{Auth, Environment, Folder, HttpRequest, OpenCollection};

const SCHEMA: &str = include_str!("data/opencollection-v1.0.0.json");
const FULL: &str = include_str!("data/full.yml");

fn assert_valid(collection: &OpenCollection) {
    let schema: serde_json::Value = serde_json::from_str(SCHEMA).unwrap();
    let validator = jsonschema::validator_for(&schema).expect("official schema should compile");

    let instance = serde_json::to_value(collection).expect("model should transcode to JSON");
    let errors: Vec<String> = validator
        .iter_errors(&instance)
        .map(|error| format!("{} at {}", error, error.instance_path()))
        .collect();
    assert!(
        errors.is_empty(),
        "schema violations:\n{}",
        errors.join("\n")
    );
}

#[test]
fn full_fixture_validates_against_official_schema() {
    let collection = OpenCollection::from_yaml(FULL).unwrap();
    assert_valid(&collection);
}

#[test]
fn builder_output_validates_against_official_schema() {
    let collection = OpenCollection::new("Built")
        .summary("Constructed via the builder API")
        .version("0.1.0")
        .environment(
            Environment::new("prod")
                .variable("baseUrl", "https://api.example.com")
                .secret("apiKey"),
        )
        .item(
            Folder::new("Pets")
                .item(
                    HttpRequest::get("{{baseUrl}}/pets")
                        .name("List pets")
                        .query("limit", "10")
                        .auth(Auth::bearer("{{apiKey}}")),
                )
                .item(
                    HttpRequest::post("{{baseUrl}}/pets")
                        .name("Create pet")
                        .header("Content-Type", "application/json")
                        .json_body(r#"{"name": "Rex"}"#)
                        .auth(Auth::basic("admin", "changeme")),
                ),
        );
    assert_valid(&collection);
}
