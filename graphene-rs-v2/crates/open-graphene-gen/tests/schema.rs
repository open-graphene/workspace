use open_graphene_gen::model::OpenGrapheneDocument;
use schemars::schema_for;
use serde_json::Value;

fn contains_string(value: &Value, needle: &str) -> bool {
    match value {
        Value::String(text) => text == needle,
        Value::Array(items) => items.iter().any(|item| contains_string(item, needle)),
        Value::Object(map) => map.values().any(|item| contains_string(item, needle)),
        _ => false,
    }
}

#[test]
fn schema_includes_required_contract_sections_and_enums() {
    let schema = schema_for!(OpenGrapheneDocument);
    let value = serde_json::to_value(&schema).expect("schema should serialize to JSON");
    let object = value
        .as_object()
        .expect("schema root should be a JSON object");
    let properties = object
        .get("properties")
        .and_then(Value::as_object)
        .expect("root schema should include properties");

    for property in [
        "openGraphene",
        "chain",
        "apis",
        "methodBindings",
        "codec",
        "operations",
        "transaction",
        "callbacks",
        "shapeClassifications",
    ] {
        assert!(
            properties.contains_key(property),
            "schema missing top-level property {property}"
        );
    }

    let required = object
        .get("required")
        .and_then(Value::as_array)
        .expect("root schema should identify required sections");
    for property in ["openGraphene", "chain"] {
        assert!(
            required.iter().any(|item| item.as_str() == Some(property)),
            "schema missing required marker for {property}"
        );
    }

    for enum_value in [
        "login_api",
        "struct",
        "sha256",
        "serialized_transaction",
        "graphene_compact_recoverable",
        "persistent",
        "approved_raw_fallback",
        "unsupported_shape",
    ] {
        assert!(
            contains_string(&value, enum_value),
            "schema missing enum value {enum_value}"
        );
    }
}
