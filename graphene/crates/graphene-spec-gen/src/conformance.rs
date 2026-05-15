use std::fmt;
use std::str::FromStr;

use graphene_bindings_swaplock::{
    Asset, AssetAssetId, ExtensionsType, Operation, SignedTransaction, Transaction,
    TransferOperation, TransferOperationFrom, TransferOperationTo,
};
use graphene_codec::to_graphene_bytes;
use graphene_rpc::GrapheneInt64;
use graphene_signing::{signing_digest, ChainId, Signer, WifSigner};
use graphene_transaction::signing::SignedTransactionEnvelope;
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

/// Error returned by reference fixture generation with the failed component attached.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformanceFixtureError {
    pub phase: ConformanceFixturePhase,
    pub message: String,
}

impl ConformanceFixtureError {
    fn new(phase: ConformanceFixturePhase, source: impl fmt::Display) -> Self {
        Self {
            phase,
            message: source.to_string(),
        }
    }
}

impl fmt::Display for ConformanceFixtureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "conformance fixture {:?} failed: {}",
            self.phase, self.message
        )
    }
}

impl std::error::Error for ConformanceFixtureError {}

/// Builds the canonical Swaplock transfer fixture from the Rust codec/signing path.
///
/// The values mirror the low-level transfer path covered by
/// `graphene-bindings-swaplock` codec tests: a transfer without memo, a fixed
/// unsigned transaction header, the real Swaplock chain id, and a known WIF key.
/// Every byte-level expected value is generated by the checked-in Rust codec and
/// signer rather than by hand-assembled hex constants.
pub fn build_swaplock_transfer_fixture() -> Result<ConformanceFixture, ConformanceFixtureError> {
    const CHAIN_ID: &str = "2267f694d96b7ffdcba1a98c63c09e720a18a85ad34954e299c66d5a42234098";
    const PRIVATE_KEY_WIF: &str = "KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn";

    let operation = swaplock_transfer_operation()?;
    let operation_json = serde_json::to_value(&operation).map_err(|error| {
        ConformanceFixtureError::new(ConformanceFixturePhase::JsonEnvelope, error)
    })?;

    let variant_operation = Operation::Transfer(operation);
    let operation_hex = encode_hex(
        &variant_operation,
        ConformanceFixturePhase::OperationEncoding,
    )?;

    let transaction = Transaction {
        ref_block_num: 0x1234,
        ref_block_prefix: 0x89ab_cdef,
        expiration: "2023-11-14T22:13:20".parse().map_err(|error| {
            ConformanceFixtureError::new(ConformanceFixturePhase::JsonEnvelope, error)
        })?,
        operations: vec![variant_operation],
        extensions: ExtensionsType(vec![]),
    };
    let transaction_json = serde_json::to_value(&transaction).map_err(|error| {
        ConformanceFixtureError::new(ConformanceFixturePhase::JsonEnvelope, error)
    })?;

    let transaction_bytes = to_graphene_bytes(&transaction).map_err(|error| {
        ConformanceFixtureError::new(ConformanceFixturePhase::TransactionEncoding, error)
    })?;
    let transaction_hex = bytes_to_hex(&transaction_bytes);

    let chain_id = ChainId::from_str(CHAIN_ID)
        .map_err(|error| ConformanceFixtureError::new(ConformanceFixturePhase::Digest, error))?;
    let digest = signing_digest(&chain_id, &transaction_bytes);
    let digest_hex = digest.to_hex();

    let signer = WifSigner::from_wif(PRIVATE_KEY_WIF).map_err(|error| {
        ConformanceFixtureError::new(ConformanceFixturePhase::KeySignature, error)
    })?;
    let public_key = signer.public_key().to_hex();
    let signature = signer.sign_digest(&digest).map_err(|error| {
        ConformanceFixtureError::new(ConformanceFixturePhase::KeySignature, error)
    })?;
    let signature_hex = signature.to_hex();

    let signed_transaction: SignedTransaction = SignedTransactionEnvelope {
        transaction,
        signatures: vec![signature],
    }
    .into();
    let signed_transaction_json = serde_json::to_value(&signed_transaction).map_err(|error| {
        ConformanceFixtureError::new(ConformanceFixturePhase::JsonEnvelope, error)
    })?;

    Ok(ConformanceFixture {
        schema_version: "1".to_owned(),
        name: "swaplock-transfer".to_owned(),
        chain_id: CHAIN_ID.to_owned(),
        input: ConformanceFixtureInput {
            operation_name: "transfer".to_owned(),
            operation_json,
            transaction_json,
            private_key_wif: PRIVATE_KEY_WIF.to_owned(),
        },
        expected: ConformanceFixtureExpected {
            operation_hex,
            transaction_hex,
            digest_hex,
            public_key,
            signature_hex,
            signature_metadata: SignatureMetadata::from_ir(
                &IrSignature {
                    curve: IrSignatureCurve::Secp256k1,
                    format: IrSignatureFormat::GrapheneCompactRecoverable,
                    canonical: true,
                },
                IrDigestAlgorithm::Sha256,
            ),
            broadcast_payload: BroadcastPayload {
                method: BroadcastMethod::BroadcastTransaction,
                params: BroadcastParams {
                    api_name: "network_broadcast".to_owned(),
                    transaction_json: signed_transaction_json,
                },
            },
        },
    })
}

fn swaplock_transfer_operation() -> Result<TransferOperation, ConformanceFixtureError> {
    Ok(TransferOperation {
        fee: Asset {
            amount: GrapheneInt64::new(200_000),
            asset_id: AssetAssetId::from_str("1.3.0").map_err(|error| {
                ConformanceFixtureError::new(ConformanceFixturePhase::JsonEnvelope, error)
            })?,
        },
        from: TransferOperationFrom::from_str("1.2.100").map_err(|error| {
            ConformanceFixtureError::new(ConformanceFixturePhase::JsonEnvelope, error)
        })?,
        to: TransferOperationTo::from_str("1.2.101").map_err(|error| {
            ConformanceFixtureError::new(ConformanceFixturePhase::JsonEnvelope, error)
        })?,
        amount: Asset {
            amount: GrapheneInt64::new(12_345),
            asset_id: AssetAssetId::from_str("1.3.0").map_err(|error| {
                ConformanceFixtureError::new(ConformanceFixturePhase::JsonEnvelope, error)
            })?,
        },
        memo: None,
        extensions: ExtensionsType(vec![]),
    })
}

fn encode_hex<T>(
    value: &T,
    phase: ConformanceFixturePhase,
) -> Result<String, ConformanceFixtureError>
where
    T: graphene_codec::GrapheneEncode,
{
    to_graphene_bytes(value)
        .map(|bytes| bytes_to_hex(&bytes))
        .map_err(|error| ConformanceFixtureError::new(phase, error))
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
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
