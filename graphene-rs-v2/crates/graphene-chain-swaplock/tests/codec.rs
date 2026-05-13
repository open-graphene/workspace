use std::cell::RefCell;
use std::str::FromStr;

use graphene_chain_swaplock::{
    broadcast, broadcast_signed_transaction, broadcast_signed_transaction_synchronous,
    sign_transaction, validate_signed_transaction, Asset, AssetAssetId, ExtensionsType, Operation,
    ProcessedTransaction, Transaction, TransferOperation, TransferOperationFrom,
    TransferOperationTo,
};
use graphene_codec::{to_graphene_bytes, GrapheneEncode};
use graphene_rpc::{
    GrapheneInt64, GrapheneTimePointSec, OpenRpcParams, RpcClient, RpcError, RpcTransport,
};
use graphene_signing::{
    signing_digest, ChainId, PublicKey, SignError, Signature, Signer, SigningDigest,
};

fn transfer_without_memo() -> TransferOperation {
    TransferOperation {
        fee: Asset {
            amount: GrapheneInt64::new(200_000),
            asset_id: AssetAssetId::from_str("1.3.0").expect("valid asset id"),
        },
        from: TransferOperationFrom::from_str("1.2.100").expect("valid from account id"),
        to: TransferOperationTo::from_str("1.2.101").expect("valid to account id"),
        amount: Asset {
            amount: GrapheneInt64::new(12_345),
            asset_id: AssetAssetId::from_str("1.3.0").expect("valid asset id"),
        },
        memo: None,
        extensions: ExtensionsType(vec![]),
    }
}

fn transfer_operation_bytes() -> Vec<u8> {
    vec![
        // fee: asset { amount: 200000, asset_id: 1.3.0 }
        0x40, 0x0d, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        // from: 1.2.100, to: 1.2.101
        0x64, 0x65, // amount: asset { amount: 12345, asset_id: 1.3.0 }
        0x39, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        // memo: None, extensions: empty set
        0x00, 0x00,
    ]
}

fn swaplock_chain_id() -> ChainId {
    ChainId::try_from("2267f694d96b7ffdcba1a98c63c09e720a18a85ad34954e299c66d5a42234098")
        .expect("valid Swaplock chain id")
}

fn fixed_signature() -> Signature {
    Signature::try_from(concat!(
        "1f",
        "2222222222222222222222222222222222222222222222222222222222222222",
        "3333333333333333333333333333333333333333333333333333333333333333"
    ))
    .expect("valid compact signature")
}

struct RecordingSigner {
    signature: Signature,
    last_digest: RefCell<Option<SigningDigest>>,
}

#[derive(Default)]
struct RecordingTransport {
    calls: RefCell<Vec<(String, Vec<serde_json::Value>)>>,
}

impl RpcTransport for RecordingTransport {
    fn call_raw(
        &self,
        method: &str,
        params: Vec<serde_json::Value>,
    ) -> Result<serde_json::Value, RpcError> {
        self.calls.borrow_mut().push((method.to_owned(), params));
        if method == "broadcast_transaction" {
            return Ok(serde_json::Value::Null);
        }
        if method == "broadcast_transaction_synchronous" {
            return Ok(serde_json::json!({
                "id": "fixture-transaction-id",
                "block_num": 42,
                "trx_num": 0
            }));
        }
        Ok(serde_json::json!({
            "expiration": "2023-11-14T22:13:20",
            "extensions": [],
            "operation_results": [],
            "operations": [[0, {
                "fee": { "amount": 200000, "asset_id": "1.3.0" },
                "from": "1.2.100",
                "to": "1.2.101",
                "amount": { "amount": 12345, "asset_id": "1.3.0" },
                "memo": null,
                "extensions": []
            }]],
            "ref_block_num": 4660,
            "ref_block_prefix": 2309737967u32,
            "signatures": [fixed_signature().to_hex()]
        }))
    }
}

impl RecordingSigner {
    fn new(signature: Signature) -> Self {
        Self {
            signature,
            last_digest: RefCell::new(None),
        }
    }
}

impl Signer for RecordingSigner {
    fn public_key(&self) -> PublicKey {
        PublicKey::try_from(concat!(
            "02",
            "1111111111111111111111111111111111111111111111111111111111111111"
        ))
        .expect("valid compressed public key")
    }

    fn sign_digest(&self, digest: &SigningDigest) -> Result<Signature, SignError> {
        *self.last_digest.borrow_mut() = Some(*digest);
        Ok(self.signature)
    }
}

#[test]
fn transfer_operation_encodes_in_fc_reflect_order() {
    let operation = transfer_without_memo();

    assert_eq!(
        operation
            .to_graphene_bytes()
            .expect("transfer should encode"),
        transfer_operation_bytes()
    );
}

#[test]
fn operation_static_variant_prefixes_transfer_with_index_zero() {
    let operation = Operation::Transfer(transfer_without_memo());
    let mut expected = vec![0x00];
    expected.extend(transfer_operation_bytes());

    assert_eq!(
        to_graphene_bytes(&operation).expect("operation should encode"),
        expected
    );
}

fn unsigned_transfer_transaction() -> Transaction {
    Transaction {
        ref_block_num: 0x1234,
        ref_block_prefix: 0x89ab_cdef,
        expiration: "2023-11-14T22:13:20"
            .parse::<GrapheneTimePointSec>()
            .expect("valid Graphene timestamp"),
        operations: vec![Operation::Transfer(transfer_without_memo())],
        extensions: ExtensionsType(vec![]),
    }
}

fn unsigned_transfer_transaction_bytes() -> Vec<u8> {
    let mut expected = vec![
        // ref_block_num: 0x1234, ref_block_prefix: 0x89abcdef
        0x34, 0x12, 0xef, 0xcd, 0xab, 0x89,
        // expiration: 1_700_000_000 seconds since epoch
        0x00, 0xf1, 0x53, 0x65, // operations: one static_variant operation follows
        0x01, 0x00,
    ];
    expected.extend(transfer_operation_bytes());
    // transaction extensions: empty set
    expected.push(0x00);
    expected
}

#[test]
fn transaction_encodes_unsigned_transfer_envelope() {
    assert_eq!(
        to_graphene_bytes(&unsigned_transfer_transaction()).expect("transaction should encode"),
        unsigned_transfer_transaction_bytes()
    );
}

#[test]
fn unsigned_transfer_transaction_has_stable_signing_digest() {
    let chain_id = swaplock_chain_id();
    let transaction_bytes = to_graphene_bytes(&unsigned_transfer_transaction())
        .expect("transaction should encode before digesting");

    assert_eq!(transaction_bytes, unsigned_transfer_transaction_bytes());
    assert_eq!(
        signing_digest(&chain_id, &transaction_bytes).to_hex(),
        "6c664c46da08b6dcbd0cdf9e4f24dc051afaa6096a23db23b15377377c60805e"
    );
}

#[test]
fn sign_transaction_builds_signed_envelope_from_signer_trait() {
    let signer = RecordingSigner::new(fixed_signature());
    let transaction = unsigned_transfer_transaction();

    let signed = sign_transaction(&swaplock_chain_id(), transaction.clone(), &signer)
        .expect("fixed signer should sign transaction");

    assert_eq!(
        signer
            .last_digest
            .borrow()
            .expect("signer should receive digest")
            .to_hex(),
        "6c664c46da08b6dcbd0cdf9e4f24dc051afaa6096a23db23b15377377c60805e"
    );
    assert_eq!(signed.signatures, vec![fixed_signature()]);
    assert_eq!(
        to_graphene_bytes(&signed.transaction).expect("signed envelope transaction should encode"),
        unsigned_transfer_transaction_bytes()
    );
}

#[test]
fn signed_transaction_envelope_maps_to_generated_json_rpc_shape() {
    let signed = sign_transaction(
        &swaplock_chain_id(),
        unsigned_transfer_transaction(),
        &RecordingSigner::new(fixed_signature()),
    )
    .expect("fixed signer should sign transaction");

    let generated = signed.into_generated();

    assert_eq!(generated.ref_block_num, 0x1234);
    assert_eq!(generated.ref_block_prefix, 0x89ab_cdef);
    assert_eq!(generated.operations.len(), 1);
    assert_eq!(generated.signatures, vec![fixed_signature().to_hex()]);
    assert_eq!(
        serde_json::to_value(&generated).expect("generated signed transaction should serialize"),
        serde_json::json!({
            "expiration": "2023-11-14T22:13:20",
            "extensions": [],
            "operations": [[0, {
                "fee": { "amount": 200000, "asset_id": "1.3.0" },
                "from": "1.2.100",
                "to": "1.2.101",
                "amount": { "amount": 12345, "asset_id": "1.3.0" },
                "memo": null,
                "extensions": []
            }]],
            "ref_block_num": 4660,
            "ref_block_prefix": 2309737967u32,
            "signatures": [fixed_signature().to_hex()]
        })
    );
}

#[test]
fn validate_signed_transaction_calls_generated_database_api_method() {
    let signed = sign_transaction(
        &swaplock_chain_id(),
        unsigned_transfer_transaction(),
        &RecordingSigner::new(fixed_signature()),
    )
    .expect("fixed signer should sign transaction");
    let transport = RecordingTransport::default();
    let client = RpcClient::new(transport);

    let processed: ProcessedTransaction = validate_signed_transaction(&client, signed)
        .expect("mock validate_transaction should decode processed_transaction");

    assert_eq!(processed.signatures, vec![fixed_signature().to_hex()]);
    let calls = client.transport().calls.borrow();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, "validate_transaction");
    assert_eq!(calls[0].1.len(), 1);
    assert_eq!(
        calls[0].1[0]["signatures"],
        serde_json::json!([fixed_signature().to_hex()])
    );
    assert_eq!(calls[0].1[0]["operations"][0][0], serde_json::json!(0));
}

#[test]
fn broadcast_transaction_params_use_network_broadcast_api_method() {
    let generated = sign_transaction(
        &swaplock_chain_id(),
        unsigned_transfer_transaction(),
        &RecordingSigner::new(fixed_signature()),
    )
    .expect("fixed signer should sign transaction")
    .into_generated();

    let params = broadcast::BroadcastTransactionParams { trx: generated };
    let positional = params.into_positional_params();

    assert_eq!(
        broadcast::OPENRPC_METHODS,
        [
            "broadcast_block",
            "broadcast_transaction",
            "broadcast_transaction_synchronous",
        ]
    );
    assert_eq!(
        <broadcast::BroadcastTransactionParams as OpenRpcParams>::METHOD,
        "broadcast_transaction"
    );
    assert_eq!(positional.len(), 1);
    assert_eq!(
        positional[0]["signatures"],
        serde_json::json!([fixed_signature().to_hex()])
    );
    assert_eq!(positional[0]["operations"][0][0], serde_json::json!(0));
}

#[test]
fn broadcast_signed_transaction_calls_network_broadcast_api_method() {
    let signed = sign_transaction(
        &swaplock_chain_id(),
        unsigned_transfer_transaction(),
        &RecordingSigner::new(fixed_signature()),
    )
    .expect("fixed signer should sign transaction");
    let transport = RecordingTransport::default();
    let client = RpcClient::new(transport);

    broadcast_signed_transaction(&client, signed)
        .expect("mock broadcast should decode unit response");

    let calls = client.transport().calls.borrow();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, "broadcast_transaction");
    assert_eq!(calls[0].1.len(), 1);
    assert_eq!(
        calls[0].1[0]["signatures"],
        serde_json::json!([fixed_signature().to_hex()])
    );
    assert_eq!(calls[0].1[0]["operations"][0][0], serde_json::json!(0));
}

#[test]
fn synchronous_broadcast_helper_returns_raw_variant_response() {
    let signed = sign_transaction(
        &swaplock_chain_id(),
        unsigned_transfer_transaction(),
        &RecordingSigner::new(fixed_signature()),
    )
    .expect("fixed signer should sign transaction");
    let transport = RecordingTransport::default();
    let client = RpcClient::new(transport);

    let response = broadcast_signed_transaction_synchronous(&client, signed)
        .expect("mock synchronous broadcast should decode raw variant response");

    assert_eq!(response["id"], serde_json::json!("fixture-transaction-id"));
    let calls = client.transport().calls.borrow();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, "broadcast_transaction_synchronous");
}
