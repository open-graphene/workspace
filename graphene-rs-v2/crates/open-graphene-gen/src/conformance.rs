use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ir::{IrDigestAlgorithm, IrSignature, IrSignatureCurve, IrSignatureFormat};

/// Stable JSON fixture used to prove codec and signing behavior across generators.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ConformanceFixture {
    #[serde(rename = "schemaVersion")]
    pub schema_version: String,
    pub name: String,
    #[serde(rename = "chainId")]
    pub chain_id: String,
    pub input: ConformanceFixtureInput,
    pub expected: ConformanceFixtureExpected,
}

/// JSON-ish inputs consumed by the Rust reference path before byte-level outputs are produced.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ConformanceFixtureInput {
    #[serde(rename = "operationName")]
    pub operation_name: String,
    #[serde(rename = "operationJson")]
    pub operation_json: Value,
    #[serde(rename = "transactionJson")]
    pub transaction_json: Value,
    #[serde(rename = "privateKeyWif")]
    pub private_key_wif: String,
}

/// Byte-level reference outputs that non-Rust implementations must reproduce exactly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ConformanceFixtureExpected {
    #[serde(rename = "operationHex")]
    pub operation_hex: String,
    #[serde(rename = "transactionHex")]
    pub transaction_hex: String,
    #[serde(rename = "digestHex")]
    pub digest_hex: String,
    #[serde(rename = "publicKey")]
    pub public_key: String,
    #[serde(rename = "signatureHex")]
    pub signature_hex: String,
    #[serde(rename = "signatureMetadata")]
    pub signature_metadata: SignatureMetadata,
    #[serde(rename = "broadcastPayload")]
    pub broadcast_payload: BroadcastPayload,
}

/// Signature contract metadata stored next to the raw signature bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SignatureMetadata {
    pub digest: DigestMetadata,
    pub curve: IrSignatureCurve,
    pub format: IrSignatureFormat,
    pub canonical: bool,
}

impl SignatureMetadata {
    pub fn from_ir(signature: &IrSignature, digest_algorithm: IrDigestAlgorithm) -> Self {
        Self {
            digest: DigestMetadata {
                algorithm: digest_algorithm,
            },
            curve: signature.curve,
            format: signature.format,
            canonical: signature.canonical,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DigestMetadata {
    pub algorithm: IrDigestAlgorithm,
}

/// JSON-RPC broadcast call shape produced from the signed transaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BroadcastPayload {
    pub method: BroadcastMethod,
    pub params: BroadcastParams,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BroadcastMethod {
    BroadcastTransaction,
    BroadcastTransactionWithCallback,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BroadcastParams {
    #[serde(rename = "apiName")]
    pub api_name: String,
    #[serde(rename = "transactionJson")]
    pub transaction_json: Value,
}

/// Generation phases used in fixture errors and diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConformanceFixturePhase {
    OperationEncoding,
    TransactionEncoding,
    Digest,
    KeySignature,
    JsonEnvelope,
}

#[cfg(test)]
mod tests {
    use super::*;
    use schemars::schema_for;
    use serde_json::{json, Value};

    fn contains_string(value: &Value, needle: &str) -> bool {
        match value {
            Value::String(text) => text == needle,
            Value::Array(items) => items.iter().any(|item| contains_string(item, needle)),
            Value::Object(map) => map.values().any(|item| contains_string(item, needle)),
            _ => false,
        }
    }

    #[test]
    fn conformance_fixture_round_trips_stable_json_shape() {
        let fixture = ConformanceFixture {
            schema_version: "1".to_owned(),
            name: "swaplock-transfer".to_owned(),
            chain_id: "00".repeat(32),
            input: ConformanceFixtureInput {
                operation_name: "transfer".to_owned(),
                operation_json: json!({ "from": "1.2.3", "to": "1.2.4" }),
                transaction_json: json!({ "operations": [[0, { "from": "1.2.3" }]] }),
                private_key_wif: "5K...fixture".to_owned(),
            },
            expected: ConformanceFixtureExpected {
                operation_hex: "0102".to_owned(),
                transaction_hex: "0304".to_owned(),
                digest_hex: "05".repeat(32),
                public_key: "GPH6MRy...fixture".to_owned(),
                signature_hex: "20".repeat(65),
                signature_metadata: SignatureMetadata {
                    digest: DigestMetadata {
                        algorithm: IrDigestAlgorithm::Sha256,
                    },
                    curve: IrSignatureCurve::Secp256k1,
                    format: IrSignatureFormat::GrapheneCompactRecoverable,
                    canonical: true,
                },
                broadcast_payload: BroadcastPayload {
                    method: BroadcastMethod::BroadcastTransaction,
                    params: BroadcastParams {
                        api_name: "network_broadcast".to_owned(),
                        transaction_json: json!({ "signatures": ["20"] }),
                    },
                },
            },
        };

        let value = serde_json::to_value(&fixture).expect("fixture should serialize");
        assert_eq!(value["chainId"], json!("00".repeat(32)));
        assert_eq!(
            value["input"]["transactionJson"]["operations"][0][0],
            json!(0)
        );
        assert_eq!(
            value["expected"]["signatureMetadata"]["format"],
            json!("graphene_compact_recoverable")
        );
        assert_eq!(
            value["expected"]["broadcastPayload"]["method"],
            json!("broadcast_transaction")
        );

        let decoded: ConformanceFixture = serde_json::from_value(value).unwrap();
        assert_eq!(decoded, fixture);
    }

    #[test]
    fn conformance_schema_exposes_phase_names_for_diagnostics() {
        let schema = schema_for!(ConformanceFixturePhase);
        let value = serde_json::to_value(&schema).expect("schema should serialize");

        for phase in [
            "operation_encoding",
            "transaction_encoding",
            "digest",
            "key_signature",
            "json_envelope",
        ] {
            assert!(
                contains_string(&value, phase),
                "schema missing phase name {phase}"
            );
        }
    }

    #[test]
    fn signature_metadata_can_be_derived_from_ir_signature_contract() {
        let ir_signature = IrSignature {
            curve: IrSignatureCurve::Secp256k1,
            format: IrSignatureFormat::GrapheneCompactRecoverable,
            canonical: true,
        };

        let metadata = SignatureMetadata::from_ir(&ir_signature, IrDigestAlgorithm::Sha256);

        assert_eq!(metadata.digest.algorithm, IrDigestAlgorithm::Sha256);
        assert_eq!(metadata.curve, IrSignatureCurve::Secp256k1);
        assert_eq!(
            metadata.format,
            IrSignatureFormat::GrapheneCompactRecoverable
        );
        assert!(metadata.canonical);
    }
}
