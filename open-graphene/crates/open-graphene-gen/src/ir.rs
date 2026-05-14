use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::model;
use crate::openrpc::OpenRpcDocument;
use crate::validation::{ValidationError, ValidationWarning};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct IrDocument {
    pub contract_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub openrpc_source: Option<String>,
    pub chain: IrChain,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub apis: BTreeMap<String, IrApiSurface>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub methods: BTreeMap<String, IrMethod>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub codec_types: BTreeMap<String, IrCodecType>,
    #[serde(
        default,
        rename = "operations",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub operation_variants: BTreeMap<String, Vec<IrOperationVariant>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transaction: Option<IrTransaction>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub callbacks: BTreeMap<String, IrCallback>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<IrDiagnostic>,
}

impl IrDocument {
    pub fn from_open_graphene(document: &model::OpenGrapheneDocument) -> Self {
        let callbacks = document
            .callbacks
            .iter()
            .map(|(name, callback)| (name.clone(), IrCallback::from_model(name, callback)))
            .collect::<BTreeMap<_, _>>();

        let mut methods = document
            .method_bindings
            .iter()
            .map(|(name, binding)| {
                let callback = callbacks.get(name);
                (
                    name.clone(),
                    IrMethod {
                        name: name.clone(),
                        api: binding.api.clone(),
                        params: callback
                            .map(|callback| callback.request_params.clone())
                            .unwrap_or_default(),
                        result: callback.and_then(|callback| callback.direct_result.clone()),
                        callback: callback.map(|_| name.clone()),
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();

        for (name, callback) in &callbacks {
            methods.entry(name.clone()).or_insert_with(|| IrMethod {
                name: name.clone(),
                api: callback.api.clone(),
                params: callback.request_params.clone(),
                result: callback.direct_result.clone(),
                callback: Some(name.clone()),
            });
        }

        Self {
            contract_version: document.open_graphene.clone(),
            openrpc_source: document.openrpc.clone(),
            chain: IrChain::from(&document.chain),
            apis: document
                .apis
                .iter()
                .map(|(name, api)| (name.clone(), IrApiSurface::from_model(name, api)))
                .collect(),
            methods,
            codec_types: document
                .codec
                .types
                .iter()
                .map(|(name, codec_type)| (name.clone(), IrCodecType::from_model(name, codec_type)))
                .collect(),
            operation_variants: document
                .operations
                .iter()
                .map(|(name, variants)| {
                    (
                        name.clone(),
                        variants
                            .iter()
                            .map(IrOperationVariant::from)
                            .collect::<Vec<_>>(),
                    )
                })
                .collect(),
            transaction: document.transaction.as_ref().map(IrTransaction::from),
            callbacks,
            diagnostics: Vec::new(),
        }
    }

    pub fn with_validation_errors(mut self, errors: &[ValidationError]) -> Self {
        self.diagnostics
            .extend(errors.iter().map(IrDiagnostic::from_validation_error));
        self
    }

    pub fn with_validation_warnings(mut self, warnings: &[ValidationWarning]) -> Self {
        self.diagnostics
            .extend(warnings.iter().map(IrDiagnostic::from_validation_warning));
        self
    }

    pub fn bind_openrpc_methods(mut self, openrpc: &OpenRpcDocument) -> Self {
        for (name, method) in &openrpc.methods {
            if let Some(ir_method) = self.methods.get_mut(name) {
                if ir_method.params.is_empty() {
                    ir_method.params = method.params.iter().map(IrTypeRef::from).collect();
                }
                if ir_method.result.is_none() {
                    ir_method.result = method.result.as_ref().map(IrTypeRef::from);
                }
            }
        }
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct IrChain {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub websocket_endpoints: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub http_endpoints: Vec<String>,
}

impl From<&model::ChainMetadata> for IrChain {
    fn from(chain: &model::ChainMetadata) -> Self {
        Self {
            name: chain.name.clone(),
            chain_id: chain.chain_id.clone(),
            websocket_endpoints: chain.ws_endpoints.clone(),
            http_endpoints: chain.http_endpoints.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct IrApiSurface {
    pub name: String,
    pub graphene_name: String,
    pub access: IrApiAccess,
}

impl IrApiSurface {
    fn from_model(name: &str, api: &model::ApiSurface) -> Self {
        Self {
            name: name.to_owned(),
            graphene_name: api.graphene_name.clone(),
            access: IrApiAccess::from(api.access),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IrApiAccess {
    #[default]
    Default,
    LoginApi,
}

impl From<model::ApiAccess> for IrApiAccess {
    fn from(access: model::ApiAccess) -> Self {
        match access {
            model::ApiAccess::Default => Self::Default,
            model::ApiAccess::LoginApi => Self::LoginApi,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct IrMethod {
    pub name: String,
    pub api: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub params: Vec<IrTypeRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<IrTypeRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub callback: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct IrCodecType {
    pub name: String,
    pub kind: IrCodecKind,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<IrCodecField>,
}

impl IrCodecType {
    fn from_model(name: &str, codec_type: &model::CodecType) -> Self {
        Self {
            name: name.to_owned(),
            kind: IrCodecKind::from(codec_type.kind),
            fields: codec_type.fields.iter().map(IrCodecField::from).collect(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IrCodecKind {
    Struct,
}

impl From<model::CodecKind> for IrCodecKind {
    fn from(kind: model::CodecKind) -> Self {
        match kind {
            model::CodecKind::Struct => Self::Struct,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct IrCodecField {
    pub name: String,
    pub type_ref: IrTypeRef,
}

impl From<&model::CodecField> for IrCodecField {
    fn from(field: &model::CodecField) -> Self {
        Self {
            name: field.name.clone(),
            type_ref: IrTypeRef::from(&field.type_ref),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum IrTypeRef {
    Named { name: String },
    Array { item: Box<IrTypeRef> },
    Optional { item: Box<IrTypeRef> },
}

impl IrTypeRef {
    pub fn named(name: impl Into<String>) -> Self {
        Self::Named { name: name.into() }
    }

    pub fn array(item: IrTypeRef) -> Self {
        Self::Array {
            item: Box::new(item),
        }
    }

    pub fn optional(item: IrTypeRef) -> Self {
        Self::Optional {
            item: Box::new(item),
        }
    }
}

impl From<&model::CodecTypeRef> for IrTypeRef {
    fn from(type_ref: &model::CodecTypeRef) -> Self {
        match type_ref {
            model::CodecTypeRef::Named(name) => Self::named(name),
            model::CodecTypeRef::Array { array } => Self::array(Self::named(array)),
            model::CodecTypeRef::Optional { optional } => Self::optional(Self::named(optional)),
        }
    }
}

impl From<&String> for IrTypeRef {
    fn from(name: &String) -> Self {
        Self::named(name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct IrOperationVariant {
    pub id: u16,
    pub name: String,
    pub type_ref: IrTypeRef,
}

impl From<&model::OperationVariant> for IrOperationVariant {
    fn from(variant: &model::OperationVariant) -> Self {
        Self {
            id: variant.id,
            name: variant.name.clone(),
            type_ref: IrTypeRef::named(&variant.type_ref),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct IrTransaction {
    pub type_ref: IrTypeRef,
    pub operation_variant: String,
    pub digest: IrDigest,
    pub signature: IrSignature,
}

impl From<&model::TransactionContract> for IrTransaction {
    fn from(transaction: &model::TransactionContract) -> Self {
        Self {
            type_ref: IrTypeRef::named(&transaction.type_ref),
            operation_variant: transaction.operation_variant.clone(),
            digest: IrDigest::from(&transaction.digest),
            signature: IrSignature::from(&transaction.signature),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct IrDigest {
    pub algorithm: IrDigestAlgorithm,
    pub preimage: Vec<IrDigestPreimagePart>,
}

impl From<&model::DigestContract> for IrDigest {
    fn from(digest: &model::DigestContract) -> Self {
        Self {
            algorithm: IrDigestAlgorithm::from(digest.algorithm),
            preimage: digest
                .preimage
                .iter()
                .copied()
                .map(IrDigestPreimagePart::from)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IrDigestAlgorithm {
    Sha256,
}

impl From<model::DigestAlgorithm> for IrDigestAlgorithm {
    fn from(algorithm: model::DigestAlgorithm) -> Self {
        match algorithm {
            model::DigestAlgorithm::Sha256 => Self::Sha256,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IrDigestPreimagePart {
    ChainId,
    SerializedTransaction,
}

impl From<model::DigestPreimagePart> for IrDigestPreimagePart {
    fn from(part: model::DigestPreimagePart) -> Self {
        match part {
            model::DigestPreimagePart::ChainId => Self::ChainId,
            model::DigestPreimagePart::SerializedTransaction => Self::SerializedTransaction,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct IrSignature {
    pub curve: IrSignatureCurve,
    pub format: IrSignatureFormat,
    pub canonical: bool,
}

impl From<&model::SignatureContract> for IrSignature {
    fn from(signature: &model::SignatureContract) -> Self {
        Self {
            curve: IrSignatureCurve::from(signature.curve),
            format: IrSignatureFormat::from(signature.format),
            canonical: signature.canonical,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IrSignatureCurve {
    Secp256k1,
}

impl From<model::SignatureCurve> for IrSignatureCurve {
    fn from(curve: model::SignatureCurve) -> Self {
        match curve {
            model::SignatureCurve::Secp256k1 => Self::Secp256k1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IrSignatureFormat {
    GrapheneCompactRecoverable,
}

impl From<model::SignatureFormat> for IrSignatureFormat {
    fn from(format: model::SignatureFormat) -> Self {
        match format {
            model::SignatureFormat::GrapheneCompactRecoverable => Self::GrapheneCompactRecoverable,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct IrCallback {
    pub name: String,
    pub api: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub request_params: Vec<IrTypeRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direct_result: Option<IrTypeRef>,
    pub callback_param: IrCallbackParam,
    pub callback_payload: IrTypeRef,
    pub callback_lifetime: IrCallbackLifetime,
}

impl IrCallback {
    fn from_model(name: &str, callback: &model::CallbackContract) -> Self {
        Self {
            name: name.to_owned(),
            api: callback.api.clone(),
            request_params: callback
                .request_params
                .iter()
                .map(IrTypeRef::from)
                .collect(),
            direct_result: callback.direct_result.as_ref().map(IrTypeRef::from),
            callback_param: IrCallbackParam::from(&callback.callback_param),
            callback_payload: IrTypeRef::named(&callback.callback_payload),
            callback_lifetime: IrCallbackLifetime::from(callback.callback_lifetime),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct IrCallbackParam {
    pub position: usize,
    pub allocated_by: IrCallbackAllocator,
}

impl From<&model::CallbackParam> for IrCallbackParam {
    fn from(param: &model::CallbackParam) -> Self {
        Self {
            position: param.position,
            allocated_by: IrCallbackAllocator::from(param.allocated_by),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IrCallbackAllocator {
    Client,
}

impl From<model::CallbackAllocator> for IrCallbackAllocator {
    fn from(allocator: model::CallbackAllocator) -> Self {
        match allocator {
            model::CallbackAllocator::Client => Self::Client,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IrCallbackLifetime {
    Once,
    Persistent,
}

impl From<model::CallbackLifetime> for IrCallbackLifetime {
    fn from(lifetime: model::CallbackLifetime) -> Self {
        match lifetime {
            model::CallbackLifetime::Once => Self::Once,
            model::CallbackLifetime::Persistent => Self::Persistent,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct IrDiagnostic {
    pub severity: IrDiagnosticSeverity,
    pub path: String,
    pub message: String,
}

impl IrDiagnostic {
    pub fn from_validation_error(error: &ValidationError) -> Self {
        Self {
            severity: IrDiagnosticSeverity::Error,
            path: error.path.clone(),
            message: error.message.clone(),
        }
    }

    pub fn from_validation_warning(warning: &ValidationWarning) -> Self {
        Self {
            severity: IrDiagnosticSeverity::Warning,
            path: warning.path.clone(),
            message: format!("{}: {}", warning.classification, warning.message),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IrDiagnosticSeverity {
    Error,
    Warning,
    Info,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::OpenGrapheneDocument;

    #[test]
    fn ir_basic_construction_is_language_neutral_and_sorted() {
        let mut apis = BTreeMap::new();
        apis.insert(
            "database".to_owned(),
            IrApiSurface {
                name: "database".to_owned(),
                graphene_name: "database".to_owned(),
                access: IrApiAccess::Default,
            },
        );

        let mut codec_types = BTreeMap::new();
        codec_types.insert(
            "Asset".to_owned(),
            IrCodecType {
                name: "Asset".to_owned(),
                kind: IrCodecKind::Struct,
                fields: vec![IrCodecField {
                    name: "amount".to_owned(),
                    type_ref: IrTypeRef::named("int64"),
                }],
            },
        );

        let ir = IrDocument {
            contract_version: "0.1".to_owned(),
            openrpc_source: Some("./chain.openrpc.json".to_owned()),
            chain: IrChain {
                name: "chain".to_owned(),
                chain_id: None,
                websocket_endpoints: vec!["wss://node.test/ws".to_owned()],
                http_endpoints: Vec::new(),
            },
            apis,
            methods: BTreeMap::new(),
            codec_types,
            operation_variants: BTreeMap::new(),
            transaction: None,
            callbacks: BTreeMap::new(),
            diagnostics: Vec::new(),
        };

        assert_eq!(ir.apis.keys().collect::<Vec<_>>(), vec!["database"]);
        assert_eq!(
            ir.codec_types["Asset"].fields[0].type_ref,
            IrTypeRef::named("int64")
        );
    }

    #[test]
    fn ir_lowers_open_graphene_fixture_deterministically() {
        let fixture = include_str!("../fixtures/swaplock.opengraphene.json");
        let document: OpenGrapheneDocument = serde_json::from_str(fixture).unwrap();
        let ir = IrDocument::from_open_graphene(&document);

        assert_eq!(ir.chain.name, "swaplock");
        assert_eq!(
            ir.apis.keys().cloned().collect::<Vec<_>>(),
            vec!["database", "network_broadcast"]
        );
        assert_eq!(
            ir.methods.keys().cloned().collect::<Vec<_>>(),
            vec![
                "broadcast_transaction",
                "broadcast_transaction_with_callback",
                "get_dynamic_global_properties",
                "set_block_applied_callback"
            ]
        );
        assert_eq!(
            ir.codec_types["TransferOperation"].fields[4].type_ref,
            IrTypeRef::optional(IrTypeRef::named("bytes"))
        );
        assert_eq!(ir.operation_variants["Operation"][0].name, "transfer");
        assert_eq!(
            ir.transaction.as_ref().unwrap().type_ref,
            IrTypeRef::named("SignedTransaction")
        );
        assert_eq!(
            ir.callbacks["set_block_applied_callback"].callback_lifetime,
            IrCallbackLifetime::Persistent
        );
        assert!(ir.diagnostics.is_empty());
    }

    #[test]
    fn ir_can_carry_validation_diagnostics() {
        let document: OpenGrapheneDocument = serde_json::from_str(
            r#"{
                "openGraphene": "0.1",
                "chain": { "name": "chain" },
                "methodBindings": { "missing": { "api": "database" } }
            }"#,
        )
        .unwrap();
        let errors = crate::validation::validate_document(&document).unwrap_err();
        let ir = IrDocument::from_open_graphene(&document).with_validation_errors(&errors);

        assert_eq!(ir.diagnostics.len(), 1);
        assert_eq!(ir.diagnostics[0].severity, IrDiagnosticSeverity::Error);
        assert_eq!(ir.diagnostics[0].path, "methodBindings.missing.api");
    }
}
