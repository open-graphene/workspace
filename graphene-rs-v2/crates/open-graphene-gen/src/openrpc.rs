use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

use crate::validation::ValidationError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenRpcDocument {
    pub methods: BTreeMap<String, OpenRpcMethod>,
    pub component_schema_names: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenRpcMethod {
    pub name: String,
    pub params: Vec<String>,
    pub result: Option<String>,
}

impl OpenRpcDocument {
    pub fn parse_json(input: &str) -> Result<Self, serde_json::Error> {
        let raw: RawOpenRpcDocument = serde_json::from_str(input)?;
        Ok(Self::from(raw))
    }

    pub fn validate_refs(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        for (method_name, method) in &self.methods {
            for (idx, param) in method.params.iter().enumerate() {
                self.validate_schema_ref(
                    &mut errors,
                    &format!("methods.{method_name}.params[{idx}].schema.$ref"),
                    param,
                );
            }
            if let Some(result) = &method.result {
                self.validate_schema_ref(
                    &mut errors,
                    &format!("methods.{method_name}.result.schema.$ref"),
                    result,
                );
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn validate_schema_ref(&self, errors: &mut Vec<ValidationError>, path: &str, name: &str) {
        if !self.component_schema_names.contains(name) && !is_openrpc_builtin_schema(name) {
            errors.push(ValidationError::new(
                path,
                format!("references missing OpenRPC component schema {name}"),
            ));
        }
    }
}

impl From<RawOpenRpcDocument> for OpenRpcDocument {
    fn from(raw: RawOpenRpcDocument) -> Self {
        let component_schema_names = raw
            .components
            .schemas
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>();
        let methods = raw
            .methods
            .into_iter()
            .map(|method| {
                let name = method.name;
                let params = method
                    .params
                    .into_iter()
                    .filter_map(|param| schema_component_name(param.schema.ref_path.as_deref()))
                    .collect();
                let result = method
                    .result
                    .and_then(|result| schema_component_name(result.schema.ref_path.as_deref()));
                (
                    name.clone(),
                    OpenRpcMethod {
                        name,
                        params,
                        result,
                    },
                )
            })
            .collect();

        Self {
            methods,
            component_schema_names,
        }
    }
}

fn schema_component_name(ref_path: Option<&str>) -> Option<String> {
    ref_path
        .and_then(|path| path.strip_prefix("#/components/schemas/"))
        .map(str::to_owned)
}

fn is_openrpc_builtin_schema(name: &str) -> bool {
    matches!(
        name,
        "bool"
            | "bytes"
            | "int64"
            | "object_id"
            | "public_key"
            | "signature"
            | "string"
            | "time_point_sec"
            | "uint8"
            | "uint16"
            | "uint32"
            | "uint64"
            | "void"
    )
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawOpenRpcDocument {
    #[serde(default)]
    methods: Vec<RawOpenRpcMethod>,
    #[serde(default)]
    components: RawOpenRpcComponents,
}

#[derive(Debug, Default, Deserialize)]
struct RawOpenRpcComponents {
    #[serde(default)]
    schemas: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct RawOpenRpcMethod {
    name: String,
    #[serde(default)]
    params: Vec<RawOpenRpcParam>,
    #[serde(default)]
    result: Option<RawOpenRpcResult>,
}

#[derive(Debug, Deserialize)]
struct RawOpenRpcParam {
    schema: RawOpenRpcSchemaRef,
}

#[derive(Debug, Deserialize)]
struct RawOpenRpcResult {
    schema: RawOpenRpcSchemaRef,
}

#[derive(Debug, Deserialize)]
struct RawOpenRpcSchemaRef {
    #[serde(rename = "$ref")]
    ref_path: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_method_metadata_and_component_names() {
        let openrpc = OpenRpcDocument::parse_json(
            r##"{
                "openrpc": "1.2.6",
                "methods": [{
                    "name": "get_dynamic_global_properties",
                    "params": [],
                    "result": { "name": "result", "schema": { "$ref": "#/components/schemas/DynamicGlobalProperties" } }
                }],
                "components": { "schemas": { "DynamicGlobalProperties": { "type": "object" } } }
            }"##,
        )
        .unwrap();

        assert_eq!(
            openrpc.methods["get_dynamic_global_properties"]
                .result
                .as_deref(),
            Some("DynamicGlobalProperties")
        );
        assert!(openrpc
            .component_schema_names
            .contains("DynamicGlobalProperties"));
        openrpc.validate_refs().unwrap();
    }
}
