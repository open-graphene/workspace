use std::str::FromStr;

use graphene_chain_swaplock::{
    Asset, AssetAssetId, ExtensionsType, Operation, Transaction, TransferOperation,
    TransferOperationFrom, TransferOperationTo,
};
use graphene_codec::{to_graphene_bytes, GrapheneEncode};
use graphene_rpc::{GrapheneInt64, GrapheneTimePointSec};

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

#[test]
fn transaction_encodes_unsigned_transfer_envelope() {
    let transaction = Transaction {
        ref_block_num: 0x1234,
        ref_block_prefix: 0x89ab_cdef,
        expiration: "2023-11-14T22:13:20"
            .parse::<GrapheneTimePointSec>()
            .expect("valid Graphene timestamp"),
        operations: vec![Operation::Transfer(transfer_without_memo())],
        extensions: ExtensionsType(vec![]),
    };

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

    assert_eq!(
        to_graphene_bytes(&transaction).expect("transaction should encode"),
        expected
    );
}
