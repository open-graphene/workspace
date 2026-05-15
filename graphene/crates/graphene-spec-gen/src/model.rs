use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct OpenGrapheneDocument {
    #[serde(rename = "openGraphene")]
    pub open_graphene: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub openrpc: Option<String>,
    pub chain: ChainMetadata,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub apis: BTreeMap<String, ApiSurface>,
    #[serde(
        default,
        rename = "methodBindings",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub method_bindings: BTreeMap<String, MethodBinding>,
    #[serde(default)]
    pub codec: CodecSection,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub operations: BTreeMap<String, Vec<OperationVariant>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transaction: Option<TransactionContract>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub callbacks: BTreeMap<String, CallbackContract>,
    #[serde(
        default,
        rename = "shapeClassifications",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub shape_classifications: Vec<ShapeClassificationEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ShapeClassificationEvidence {
    pub path: String,
    pub classification: ShapeClassification,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ShapeClassification {
    ApprovedRawFallback,
    UnsupportedShape,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ChainMetadata {
    pub name: String,
    #[serde(default, rename = "chainId", skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<String>,
    #[serde(default, rename = "wsEndpoints", skip_serializing_if = "Vec::is_empty")]
    pub ws_endpoints: Vec<String>,
    #[serde(
        default,
        rename = "httpEndpoints",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub http_endpoints: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ApiSurface {
    #[serde(rename = "grapheneName")]
    pub graphene_name: String,
    #[serde(default)]
    pub access: ApiAccess,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ApiAccess {
    #[default]
    Default,
    LoginApi,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MethodBinding {
    pub api: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CodecSection {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub types: BTreeMap<String, CodecType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CodecType {
    pub kind: CodecKind,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<CodecField>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CodecKind {
    Struct,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CodecField {
    pub name: String,
    #[serde(rename = "type")]
    pub type_ref: CodecTypeRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum CodecTypeRef {
    Named(String),
    Array { array: String },
    Optional { optional: String },
}

impl CodecTypeRef {
    pub fn referenced_name(&self) -> &str {
        match self {
            Self::Named(name) | Self::Array { array: name } | Self::Optional { optional: name } => {
                name
            }
        }
    }
}

pub fn is_builtin_codec_type(name: &str) -> bool {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct OperationVariant {
    pub id: u16,
    pub name: String,
    #[serde(rename = "type")]
    pub type_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TransactionContract {
    #[serde(rename = "type")]
    pub type_ref: String,
    #[serde(rename = "operationVariant")]
    pub operation_variant: String,
    pub digest: DigestContract,
    pub signature: SignatureContract,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DigestContract {
    pub algorithm: DigestAlgorithm,
    pub preimage: Vec<DigestPreimagePart>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DigestAlgorithm {
    Sha256,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DigestPreimagePart {
    ChainId,
    SerializedTransaction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SignatureContract {
    pub curve: SignatureCurve,
    pub format: SignatureFormat,
    #[serde(default)]
    pub canonical: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SignatureCurve {
    Secp256k1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SignatureFormat {
    GrapheneCompactRecoverable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CallbackContract {
    pub api: String,
    #[serde(
        default,
        rename = "requestParams",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub request_params: Vec<String>,
    #[serde(
        default,
        rename = "directResult",
        skip_serializing_if = "Option::is_none"
    )]
    pub direct_result: Option<String>,
    #[serde(rename = "callbackParam")]
    pub callback_param: CallbackParam,
    #[serde(rename = "callbackPayload")]
    pub callback_payload: String,
    #[serde(rename = "callbackLifetime")]
    pub callback_lifetime: CallbackLifetime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CallbackParam {
    pub position: usize,
    #[serde(rename = "allocatedBy")]
    pub allocated_by: CallbackAllocator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CallbackAllocator {
    Client,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CallbackLifetime {
    Once,
    Persistent,
}
