use graphene_chain_swaplock::{
    GetBlockHeaderBatchParams, GetBlockParams, LookupVoteIdObject, LookupVoteIdsParams,
    MaybeSignedBlockHeader, SignedBlock,
};
use graphene_rpc::{GrapheneTimePointSec, OpenRpcParams};
use serde_json::json;

#[test]
fn get_block_params_emit_method_response_and_positional_params() {
    fn assert_response_type<T: OpenRpcParams<Response = Option<SignedBlock>>>() {}
    assert_response_type::<GetBlockParams>();

    let params = GetBlockParams { block_num: 123 };

    assert_eq!(GetBlockParams::METHOD, "get_block");
    assert_eq!(params.into_positional_params(), vec![json!(123)]);
}

#[test]
fn get_block_header_batch_decodes_fc_map_as_array_pairs() {
    fn assert_response_type<
        T: OpenRpcParams<Response = Vec<(u32, Option<MaybeSignedBlockHeader>)>>,
    >() {
    }
    assert_response_type::<GetBlockHeaderBatchParams>();

    let params = GetBlockHeaderBatchParams {
        block_nums: vec![123],
        with_witness_signatures: Some(true),
    };

    assert_eq!(GetBlockHeaderBatchParams::METHOD, "get_block_header_batch");
    assert_eq!(
        params.into_positional_params(),
        vec![json!([123]), json!(true)]
    );

    let decoded: <GetBlockHeaderBatchParams as OpenRpcParams>::Response = serde_json::from_value(
        json!([
            [
                123,
                {
                    "previous": "0000007a00000000000000000000000000000000",
                    "timestamp": "2026-05-13T16:22:45",
                    "witness": "1.6.1",
                    "transaction_merkle_root": "0000000000000000000000000000000000000000",
                    "extensions": [],
                    "witness_signature": "203ceb512eecc33f2471deece3fe413e0ac4563dc89fe7f062368e790929db73a131ddc7dbac560e7a38aa1749ea4f3cbf545c3bfcb100e4cadb1d3d5718f69322"
                }
            ]
        ]),
    )
    .expect("FC map should decode from array pairs");

    assert_eq!(decoded.len(), 1);
    assert_eq!(decoded[0].0, 123);
    let header = decoded[0]
        .1
        .as_ref()
        .expect("fixture header should be present");
    assert_eq!(
        header.timestamp,
        "2026-05-13T16:22:45"
            .parse::<GrapheneTimePointSec>()
            .unwrap()
    );
    assert!(header.witness.starts_with("1.6."));
}

#[test]
fn lookup_vote_ids_decodes_mixed_vote_target_objects() {
    fn assert_response_type<T: OpenRpcParams<Response = Vec<LookupVoteIdObject>>>() {}
    assert_response_type::<LookupVoteIdsParams>();

    let params = LookupVoteIdsParams {
        votes: vec!["0:5".to_owned(), "1:0".to_owned()],
    };

    assert_eq!(LookupVoteIdsParams::METHOD, "lookup_vote_ids");
    assert_eq!(params.into_positional_params(), vec![json!(["0:5", "1:0"])]);

    let decoded: <LookupVoteIdsParams as OpenRpcParams>::Response = serde_json::from_value(json!([
        {
            "committee_member_account": "1.2.102",
            "total_votes": 0,
            "url": "",
            "vote_id": "0:5"
        },
        {
            "last_aslot": 326046,
            "last_confirmed_block_num": 326046,
            "pay_vb": "1.13.2",
            "signing_key": "BTS66b8UMB5rbMPGkGyS3sfj7do8ZcGrRkC4W2hs2o1DE4EHvWa7L",
            "total_missed": 0,
            "total_votes": 0,
            "url": "",
            "vote_id": "1:0",
            "witness_account": "1.2.102"
        }
    ]))
    .expect("mixed lookup_vote_ids response should decode into typed variants");

    assert_eq!(decoded.len(), 2);
    match &decoded[0] {
        LookupVoteIdObject::CommitteeMember(member) => {
            assert_eq!(member.committee_member_account.as_str(), "1.2.102");
            assert_eq!(member.vote_id.as_str(), "0:5");
        }
        other => panic!("expected committee member vote object, got {other:#?}"),
    }
    match &decoded[1] {
        LookupVoteIdObject::Witness(witness) => {
            assert_eq!(witness.witness_account.as_str(), "1.2.102");
            assert_eq!(witness.vote_id.as_str(), "1:0");
        }
        other => panic!("expected witness vote object, got {other:#?}"),
    }
}
