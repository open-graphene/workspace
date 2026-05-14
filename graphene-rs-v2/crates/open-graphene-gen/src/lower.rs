use std::fmt;

use crate::ir::IrDocument;
use crate::model::OpenGrapheneDocument;
use crate::validation::{validate_document, ValidationError};

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
    validate_document(document).map_err(LowerError::new)?;
    Ok(IrDocument::from_open_graphene(document))
}
