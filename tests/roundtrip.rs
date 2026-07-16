//! Lossless round-trip of a collection exercising every schema construct.

use opencollection::{Auth, AuthOAuth2, Item, OpenCollection};

const FULL: &str = include_str!("data/full.yml");

#[test]
fn full_collection_round_trips_losslessly() {
    let collection = OpenCollection::from_yaml(FULL).expect("fixture should parse");
    let serialized = collection.to_yaml().expect("collection should serialize");

    let original: serde_yaml_ng::Value = serde_yaml_ng::from_str(FULL).unwrap();
    let round_tripped: serde_yaml_ng::Value = serde_yaml_ng::from_str(&serialized).unwrap();
    assert_eq!(
        original, round_tripped,
        "serialized YAML should be value-identical to the fixture"
    );

    let reparsed = OpenCollection::from_yaml(&serialized).unwrap();
    assert_eq!(collection, reparsed);
}

#[test]
fn traversal_sees_nested_items() {
    let collection = OpenCollection::from_yaml(FULL).unwrap();

    let names: Vec<_> = collection.iter().filter_map(Item::name).collect();
    assert!(names.contains(&"List pets"));
    assert!(names.contains(&"Pet by owner"), "folders are yielded too");
    assert!(
        names.contains(&"Watch pets (gRPC)"),
        "items nested in folders are reached"
    );

    // 2 top-level + 6 in "Pet by owner" + 6 in "Auth zoo" = 14 requests,
    // and iter() additionally yields 2 folders + 1 script file.
    assert_eq!(collection.requests().count(), 14);
    assert_eq!(collection.iter().count(), 17);
}

#[test]
fn parsed_details_are_typed() {
    let collection = OpenCollection::from_yaml(FULL).unwrap();

    // Collection defaults carry an OAuth2 client-credentials auth.
    let defaults = collection.request.as_ref().unwrap();
    assert!(matches!(
        defaults.auth,
        Some(Auth::OAuth2(AuthOAuth2::ClientCredentials(_)))
    ));

    // The first request inherits auth.
    let Some(Item::Http(list_pets)) = collection.iter().next() else {
        panic!("first item should be an HTTP request");
    };
    assert_eq!(list_pets.http.as_ref().unwrap().auth, Some(Auth::Inherit));
}
