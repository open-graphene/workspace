use std::collections::BTreeSet;
use std::fmt;

use crate::model::{is_builtin_codec_type, CodecTypeRef, OpenGrapheneDocument};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
}

impl ValidationError {
    pub fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}

impl std::error::Error for ValidationError {}

pub fn validate_document(document: &OpenGrapheneDocument) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    require_non_empty(&mut errors, "openGraphene", &document.open_graphene);
    require_non_empty(&mut errors, "chain.name", &document.chain.name);

    if let Some(chain_id) = &document.chain.chain_id {
        if chain_id.len() != 64 || !chain_id.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            errors.push(ValidationError::new(
                "chain.chainId",
                "must be a 32-byte lowercase or uppercase hex string",
            ));
        }
    }

    for (api_name, api) in &document.apis {
        let path = format!("apis.{api_name}.grapheneName");
        require_non_empty(&mut errors, path, &api.graphene_name);
    }

    for (method_name, binding) in &document.method_bindings {
        let path = format!("methodBindings.{method_name}.api");
        validate_api_ref(&mut errors, &path, &binding.api, document);
    }

    let operation_variant_names: BTreeSet<_> = document.operations.keys().cloned().collect();
    for (operation_name, variants) in &document.operations {
        let mut ids = BTreeSet::new();
        let mut names = BTreeSet::new();
        for (idx, variant) in variants.iter().enumerate() {
            let base = format!("operations.{operation_name}[{idx}]");
            if !ids.insert(variant.id) {
                errors.push(ValidationError::new(
                    format!("{base}.id"),
                    format!("duplicate operation variant id {}", variant.id),
                ));
            }
            if !names.insert(variant.name.as_str()) {
                errors.push(ValidationError::new(
                    format!("{base}.name"),
                    format!("duplicate operation variant name {}", variant.name),
                ));
            }
            validate_named_type(
                &mut errors,
                &format!("{base}.type"),
                &variant.type_ref,
                document,
            );
        }
    }

    for (type_name, codec_type) in &document.codec.types {
        let mut fields = BTreeSet::new();
        for (idx, field) in codec_type.fields.iter().enumerate() {
            let base = format!("codec.types.{type_name}.fields[{idx}]");
            require_non_empty(&mut errors, format!("{base}.name"), &field.name);
            if !fields.insert(field.name.as_str()) {
                errors.push(ValidationError::new(
                    format!("{base}.name"),
                    format!("duplicate field name {}", field.name),
                ));
            }
            validate_type_ref(
                &mut errors,
                &format!("{base}.type"),
                &field.type_ref,
                document,
            );
        }
    }

    if let Some(transaction) = &document.transaction {
        validate_named_type(
            &mut errors,
            "transaction.type",
            &transaction.type_ref,
            document,
        );
        if !operation_variant_names.contains(&transaction.operation_variant) {
            errors.push(ValidationError::new(
                "transaction.operationVariant",
                format!(
                    "references unknown operation variant {}",
                    transaction.operation_variant
                ),
            ));
        }
        if transaction.digest.preimage.is_empty() {
            errors.push(ValidationError::new(
                "transaction.digest.preimage",
                "must contain at least one preimage part",
            ));
        }
    }

    for (callback_name, callback) in &document.callbacks {
        validate_api_ref(
            &mut errors,
            &format!("callbacks.{callback_name}.api"),
            &callback.api,
            document,
        );
        for (idx, param) in callback.request_params.iter().enumerate() {
            validate_named_type(
                &mut errors,
                &format!("callbacks.{callback_name}.requestParams[{idx}]"),
                param,
                document,
            );
        }
        if let Some(direct_result) = &callback.direct_result {
            validate_named_type(
                &mut errors,
                &format!("callbacks.{callback_name}.directResult"),
                direct_result,
                document,
            );
        }
        validate_named_type(
            &mut errors,
            &format!("callbacks.{callback_name}.callbackPayload"),
            &callback.callback_payload,
            document,
        );
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn require_non_empty(errors: &mut Vec<ValidationError>, path: impl Into<String>, value: &str) {
    if value.trim().is_empty() {
        errors.push(ValidationError::new(path, "must not be empty"));
    }
}

fn validate_api_ref(
    errors: &mut Vec<ValidationError>,
    path: &str,
    api_name: &str,
    document: &OpenGrapheneDocument,
) {
    if !document.apis.contains_key(api_name) {
        errors.push(ValidationError::new(
            path,
            format!("references unknown API {api_name}"),
        ));
    }
}

fn validate_type_ref(
    errors: &mut Vec<ValidationError>,
    path: &str,
    type_ref: &CodecTypeRef,
    document: &OpenGrapheneDocument,
) {
    validate_named_type(errors, path, type_ref.referenced_name(), document);
}

fn validate_named_type(
    errors: &mut Vec<ValidationError>,
    path: &str,
    type_name: &str,
    document: &OpenGrapheneDocument,
) {
    if !is_builtin_codec_type(type_name)
        && !document.codec.types.contains_key(type_name)
        && !document.operations.contains_key(type_name)
    {
        errors.push(ValidationError::new(
            path,
            format!("references unknown codec type {type_name}"),
        ));
    }
}
