use std::fmt;

use crate::ir::IrDocument;
use crate::model::OpenGrapheneDocument;
use crate::openrpc::OpenRpcDocument;
use crate::validation::{validate_document_report, ValidationError};

/// Errors produced while lowering an OpenGraphene contract into generator IR.
///
/// Lowering intentionally reuses domain validation as its first phase so every
/// rejected internal reference keeps the source path reported by validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LowerError {
    pub errors: Vec<ValidationError>,
}

impl LowerError {
    pub fn new(errors: Vec<ValidationError>) -> Self {
        Self { errors }
    }

    pub fn errors(&self) -> &[ValidationError] {
        &self.errors
    }

    pub fn into_errors(self) -> Vec<ValidationError> {
        self.errors
    }
}

impl fmt::Display for LowerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.errors.as_slice() {
            [] => write!(f, "lowering failed"),
            [error] => write!(f, "lowering failed: {error}"),
            errors => write!(f, "lowering failed with {} validation errors", errors.len()),
        }
    }
}

impl std::error::Error for LowerError {}

/// Validate and lower an OpenGraphene document into language-neutral IR.
///
/// This is the intended generator entry point: callers either receive a fully
/// resolved deterministic IR document, or path-specific validation errors that
/// explain which contract reference or field must be fixed.
pub fn lower_document_to_ir(document: &OpenGrapheneDocument) -> Result<IrDocument, LowerError> {
    let report = validate_document_report(document);
    if !report.errors.is_empty() {
        return Err(LowerError::new(report.errors));
    }
    Ok(IrDocument::from_open_graphene(document).with_validation_warnings(&report.warnings))
}

/// Validate and lower an OpenGraphene document, binding method metadata from a
/// parsed OpenRPC document.
///
/// OpenRPC is intentionally treated as a narrow companion contract here: it
/// contributes method parameter/result schema names after OpenGraphene domain
/// validation succeeds. Callback-specific OpenGraphene metadata remains
/// authoritative so generated SDK surfaces do not expose client-allocated
/// Graphene callback identifiers as user parameters.
pub fn lower_document_with_openrpc_to_ir(
    document: &OpenGrapheneDocument,
    openrpc: &OpenRpcDocument,
) -> Result<IrDocument, LowerError> {
    let mut errors = Vec::new();

    let report = validate_document_report(document);
    errors.extend(report.errors);

    if let Err(openrpc_errors) = openrpc.validate_refs() {
        errors.extend(openrpc_errors);
    }

    for method_name in document.method_bindings.keys() {
        if !openrpc.methods.contains_key(method_name) {
            errors.push(ValidationError::new(
                format!("methodBindings.{method_name}"),
                format!("references unknown OpenRPC method {method_name}"),
            ));
        }
    }

    if !errors.is_empty() {
        return Err(LowerError::new(errors));
    }

    Ok(IrDocument::from_open_graphene(document)
        .bind_openrpc_methods(openrpc)
        .with_validation_warnings(&report.warnings))
}
