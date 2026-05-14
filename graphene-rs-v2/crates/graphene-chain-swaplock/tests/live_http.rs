use std::collections::BTreeSet;

use graphene_chain_swaplock::{
    broadcast_signed_transaction_synchronous_typed,
    broadcast_signed_transaction_with_callback_typed, build_transfer_operation,
    fetch_required_fee_for_transfer, lookup_exact_account_id, prepare_transaction,
    set_block_applied_callback, GetAccountCountParams, GetAssetCountParams, GetAssetsParams,
    GetBlockHeaderBatchParams, GetBlockHeaderParams, GetBlockParams, GetChainIdParams,
    GetChainPropertiesParams, GetCommitteeCountParams, GetCommitteeMembersParams, GetConfigParams,
    GetDynamicGlobalPropertiesParams, GetGlobalPropertiesParams, GetObjectResult, GetObjectsParams,
    GetRequiredFeesParams, GetTransactionHexWithoutSigParams, GetWitnessCountParams,
    GetWitnessesParams, GetWorkerCountParams, LookupAccountsParams, LookupAssetSymbolsParams,
    LookupCommitteeMemberAccountsParams, LookupVoteIdObject, LookupVoteIdsParams,
    LookupWitnessAccountsParams, Operation, RequiredFee, Transaction, TransferDraft,
    OPENRPC_METHODS, PUBLIC_SWAPLOCK_TESTNET_WIF, SWAPLOCK_TESTNET_FROM_ACCOUNT,
    SWAPLOCK_TESTNET_HTTP_URL, SWAPLOCK_TESTNET_TO_ACCOUNT, SWAPLOCK_TESTNET_WS_URL,
};
use graphene_codec::to_graphene_bytes;
use graphene_rpc::{
    GrapheneTimePointSec, GrapheneUInt64, GrapheneWebSocketApiTransport, GrapheneWebSocketSession,
    HttpTransport, RpcClient,
};
use graphene_signing::{ChainId, WifSigner};
use std::sync::{mpsc, Arc};

/// Methods currently exercised end-to-end against the public Swaplock database RPC node.
///
/// This list is deliberately conservative: it contains read-only methods with
/// stable parameters and typed responses. The coverage test below ensures every
/// generated method is either here or in `SKIPPED_LIVE_METHODS`, so adding new
/// RPC methods forces an explicit live-test decision.
const TYPED_LIVE_METHODS: &[&str] = &[
    "get_account_count",
    "get_asset_count",
    "get_assets",
    "get_block",
    "get_block_header",
    "get_block_header_batch",
    "get_chain_id",
    "get_chain_properties",
    "get_committee_count",
    "get_committee_members",
    "get_config",
    "get_dynamic_global_properties",
    "get_global_properties",
    "get_objects",
    "get_required_fees",
    "get_transaction_hex_without_sig",
    "get_witness_count",
    "get_witnesses",
    "get_worker_count",
    "lookup_accounts",
    "lookup_asset_symbols",
    "lookup_committee_member_accounts",
    "lookup_vote_ids",
    "lookup_witness_accounts",
];

/// Generated database RPC methods not yet executed against the public node.
///
/// Most require object/account/asset fixtures, market fixtures, history ranges,
/// or subscription state. They stay classified here until we add safe live
/// fixtures for them.
const SKIPPED_LIVE_METHODS: &[&str] = &[
    "cancel_all_subscriptions",
    "get_24_volume",
    "get_account_balances",
    "get_account_by_name",
    "get_account_id_from_string",
    "get_account_limit_orders",
    "get_account_references",
    "get_accounts",
    "get_all_workers",
    "get_asset_id_from_string",
    "get_assets_by_issuer",
    "get_balance_objects",
    "get_blinded_balances",
    "get_call_orders",
    "get_call_orders_by_account",
    "get_collateral_bids",
    "get_committee_member_by_account",
    "get_credit_deals_by_borrower",
    "get_credit_deals_by_collateral_asset",
    "get_credit_deals_by_debt_asset",
    "get_credit_deals_by_offer_id",
    "get_credit_deals_by_offer_owner",
    "get_credit_offers_by_asset",
    "get_credit_offers_by_owner",
    "get_full_accounts",
    "get_htlc",
    "get_htlc_by_from",
    "get_htlc_by_to",
    "get_key_references",
    "get_limit_orders",
    "get_limit_orders_by_account",
    "get_liquidity_pools",
    "get_liquidity_pools_by_asset_a",
    "get_liquidity_pools_by_asset_b",
    "get_liquidity_pools_by_both_assets",
    "get_liquidity_pools_by_one_asset",
    "get_liquidity_pools_by_owner",
    "get_liquidity_pools_by_share_asset",
    "get_margin_positions",
    "get_named_account_balances",
    "get_next_object_id",
    "get_order_book",
    "get_potential_address_signatures",
    "get_potential_signatures",
    "get_proposed_transactions",
    "get_recent_transaction_by_id",
    "get_required_signatures",
    "get_samet_funds_by_asset",
    "get_samet_funds_by_owner",
    "get_settle_orders",
    "get_settle_orders_by_account",
    "get_ticker",
    "get_tickets_by_account",
    "get_top_markets",
    "get_top_voters",
    "get_trade_history",
    "get_trade_history_by_sequence",
    "get_transaction",
    "get_transaction_hex",
    "get_vested_balances",
    "get_vesting_balances",
    "get_withdraw_permissions_by_giver",
    "get_withdraw_permissions_by_recipient",
    "get_witness_by_account",
    "get_workers_by_account",
    "is_public_key_registered",
    "list_assets",
    "list_credit_deals",
    "list_credit_offers",
    "list_htlcs",
    "list_liquidity_pools",
    "list_samet_funds",
    "list_tickets",
    "lookup_account_names",
    "set_auto_subscription",
    "unsubscribe_from_market",
    "validate_transaction",
    "verify_account_authority",
    "verify_authority",
];

fn live_client() -> RpcClient<HttpTransport> {
    RpcClient::new(HttpTransport::new(SWAPLOCK_TESTNET_HTTP_URL))
}

fn live_broadcast_client() -> RpcClient<GrapheneWebSocketApiTransport> {
    let session = Arc::new(
        GrapheneWebSocketSession::connect(SWAPLOCK_TESTNET_WS_URL)
            .expect("Swaplock WebSocket session should connect"),
    );
    let api = session
        .login_api("network_broadcast")
        .expect("network_broadcast API should log in");
    RpcClient::new(session.api_transport(&api))
}

fn assert_positive(value: GrapheneUInt64, method: &str) {
    assert!(
        value.as_u64() > 0,
        "{method} should return a positive count"
    );
}

fn assert_id_like(value: &str, prefix: &str, method: &str) {
    assert!(
        value.starts_with(prefix),
        "{method} should return an id starting with {prefix}, got {value}"
    );
}

fn unsigned_transfer_transaction_fixture() -> Transaction {
    serde_json::from_value(serde_json::json!({
        "ref_block_num": 4660,
        "ref_block_prefix": 2309737967u32,
        "expiration": "2023-11-14T22:13:20",
        "operations": [[0, {
            "fee": { "amount": 200000, "asset_id": "1.3.0" },
            "from": "1.2.100",
            "to": "1.2.101",
            "amount": { "amount": 12345, "asset_id": "1.3.0" },
            "memo": null,
            "extensions": []
        }]],
        "extensions": []
    }))
    .expect("unsigned transfer transaction fixture should decode")
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

#[test]
fn every_generated_method_has_a_live_test_decision() {
    let generated = OPENRPC_METHODS.iter().copied().collect::<BTreeSet<_>>();
    let classified = TYPED_LIVE_METHODS
        .iter()
        .chain(SKIPPED_LIVE_METHODS.iter())
        .copied()
        .collect::<BTreeSet<_>>();

    let missing = generated
        .difference(&classified)
        .copied()
        .collect::<Vec<_>>();
    let unknown = classified
        .difference(&generated)
        .copied()
        .collect::<Vec<_>>();

    assert!(
        missing.is_empty(),
        "generated RPC methods without live-test classification: {missing:?}"
    );
    assert!(
        unknown.is_empty(),
        "live-test classification contains unknown generated methods: {unknown:?}"
    );
}

#[test]
#[ignore = "requires network access to the Swaplock testnet RPC endpoint"]
fn live_node_transaction_hex_without_sig_matches_local_binary_codec() {
    let client = live_client();
    let trx = unsigned_transfer_transaction_fixture();
    let local_hex = bytes_to_hex(
        &to_graphene_bytes(&trx).expect("unsigned transaction fixture should encode locally"),
    );

    let node_hex = client
        .call(GetTransactionHexWithoutSigParams { trx })
        .expect("node should serialize unsigned transaction to hex");

    println!("local transaction hex without signatures => {local_hex}");
    println!("node transaction hex without signatures  => {node_hex}");
    assert_eq!(node_hex, local_hex);
}

#[test]
#[ignore = "requires network access and broadcasts a tiny Swaplock testnet transfer"]
fn live_signs_and_broadcasts_tiny_transfer_with_wif() {
    let signer = WifSigner::from_wif(PUBLIC_SWAPLOCK_TESTNET_WIF)
        .expect("public Swaplock testnet WIF should parse");

    let client = live_client();
    let from = lookup_exact_account_id(&client, SWAPLOCK_TESTNET_FROM_ACCOUNT)
        .expect("from account fixture should exist");
    let to = lookup_exact_account_id(&client, SWAPLOCK_TESTNET_TO_ACCOUNT)
        .expect("to account fixture should exist");
    assert_ne!(from, to, "live transfer requires distinct accounts");

    let dynamic = client
        .call(GetDynamicGlobalPropertiesParams)
        .expect("dynamic global properties should decode");
    let chain_id = client
        .call(GetChainIdParams)
        .expect("chain id should decode");
    let chain_id = ChainId::try_from(chain_id.as_str()).expect("chain id should parse");
    let expiration =
        GrapheneTimePointSec::new(dynamic.time.naive_utc() + chrono::Duration::minutes(5));

    let mut transfer = build_transfer_operation(TransferDraft {
        from,
        to,
        amount: 1,
        asset_id: "1.3.0".to_owned(),
    })
    .expect("transfer draft should build");
    transfer.fee = fetch_required_fee_for_transfer(&client, &transfer, "1.3.0")
        .expect("required fee lookup should succeed");

    let prepared = prepare_transaction(&dynamic, vec![Operation::Transfer(transfer)], expiration)
        .expect("transaction should prepare from dynamic global properties");
    let signed = prepared
        .sign(&chain_id, &signer)
        .expect("transaction should sign");
    let broadcast_client = live_broadcast_client();
    let response = broadcast_signed_transaction_synchronous_typed(&broadcast_client, signed)
        .expect("signed transfer should broadcast synchronously");

    println!(
        "broadcast_transaction_synchronous => id={}, block_num={}, trx_num={}",
        response.id, response.block_num, response.trx_num
    );
    assert!(!response.id.is_empty());
}

#[test]
#[ignore = "requires network access and broadcasts a tiny Swaplock testnet transfer with callback"]
fn live_broadcasts_tiny_transfer_with_callback() {
    let signer = WifSigner::from_wif(PUBLIC_SWAPLOCK_TESTNET_WIF)
        .expect("public Swaplock testnet WIF should parse");

    let client = live_client();
    let from = lookup_exact_account_id(&client, SWAPLOCK_TESTNET_FROM_ACCOUNT)
        .expect("from account fixture should exist");
    let to = lookup_exact_account_id(&client, SWAPLOCK_TESTNET_TO_ACCOUNT)
        .expect("to account fixture should exist");
    assert_ne!(from, to, "live transfer requires distinct accounts");

    let dynamic = client
        .call(GetDynamicGlobalPropertiesParams)
        .expect("dynamic global properties should decode");
    let chain_id = client
        .call(GetChainIdParams)
        .expect("chain id should decode");
    let chain_id = ChainId::try_from(chain_id.as_str()).expect("chain id should parse");
    let expiration =
        GrapheneTimePointSec::new(dynamic.time.naive_utc() + chrono::Duration::minutes(5));

    let mut transfer = build_transfer_operation(TransferDraft {
        from,
        to,
        amount: 1,
        asset_id: "1.3.0".to_owned(),
    })
    .expect("transfer draft should build");
    transfer.fee = fetch_required_fee_for_transfer(&client, &transfer, "1.3.0")
        .expect("required fee lookup should succeed");

    let prepared = prepare_transaction(&dynamic, vec![Operation::Transfer(transfer)], expiration)
        .expect("transaction should prepare from dynamic global properties");
    let signed = prepared
        .sign(&chain_id, &signer)
        .expect("transaction should sign");

    let session = GrapheneWebSocketSession::connect(SWAPLOCK_TESTNET_WS_URL)
        .expect("Swaplock WebSocket session should connect");
    let broadcast_api = session
        .login_api("network_broadcast")
        .expect("network_broadcast API should log in");
    let response =
        broadcast_signed_transaction_with_callback_typed(&session, &broadcast_api, signed)
            .expect("signed transfer should broadcast and receive callback");

    println!(
        "broadcast_transaction_with_callback => id={}, block_num={}, trx_num={}",
        response.id, response.block_num, response.trx_num
    );
    assert!(!response.id.is_empty());
}

#[test]
#[ignore = "requires network access and waits for a Swaplock testnet block notice"]
fn live_receives_block_applied_callback_notice() {
    let session = GrapheneWebSocketSession::connect(SWAPLOCK_TESTNET_WS_URL)
        .expect("Swaplock WebSocket session should connect");
    let database_api = session
        .login_api("database")
        .expect("database API should log in");

    let (notice_tx, notice_rx) = mpsc::sync_channel(1);
    let subscription = set_block_applied_callback(&session, &database_api, move |notice| {
        let _ = notice_tx.send(notice);
    })
    .expect("set_block_applied_callback should register callback");

    let notice = notice_rx
        .recv()
        .expect("registered callback should receive the notice payload");

    let block_id = notice.block_id;
    println!(
        "set_block_applied_callback => callback_id={}, block_id={block_id}",
        subscription.id()
    );
    assert_eq!(
        block_id.len(),
        40,
        "Graphene block ids should be 20-byte hex strings"
    );
    assert!(
        block_id
            .chars()
            .all(|character| character.is_ascii_hexdigit()),
        "Graphene block id should be hex"
    );

    subscription
        .unsubscribe()
        .expect("callback subscription should unsubscribe locally");
}

#[test]
#[ignore = "requires network access to the Swaplock testnet RPC endpoint"]
fn live_decodes_read_only_rpc_methods_over_http() {
    let client = live_client();

    let account_count = client.call(GetAccountCountParams).unwrap();
    println!("get_account_count => {account_count}");
    assert_positive(account_count, "get_account_count");

    let account_lookup = client
        .call(LookupAccountsParams {
            lower_bound_name: String::new(),
            limit: 1,
            subscribe: Some(false),
        })
        .unwrap();
    println!("lookup_accounts('', 1, false) => {account_lookup:#?}");
    let (_account_name, account_id) = account_lookup
        .iter()
        .next()
        .expect("lookup_accounts should return at least one account");
    assert_id_like(account_id, "1.2.", "lookup_accounts");

    let asset_count = client.call(GetAssetCountParams).unwrap();
    println!("get_asset_count => {asset_count}");
    assert_positive(asset_count, "get_asset_count");

    let assets = client
        .call(GetAssetsParams {
            asset_symbols_or_ids: vec!["1.3.0".to_owned()],
            subscribe: Some(false),
        })
        .unwrap();
    println!("get_assets([1.3.0], false) => {assets:#?}");
    assert_eq!(assets.len(), 1);
    let core_asset = assets[0].as_ref().expect("core asset should exist");
    assert_eq!(core_asset.symbol, "BTS");

    let lookup_assets = client
        .call(LookupAssetSymbolsParams {
            symbols_or_ids: vec!["BTS".to_owned()],
        })
        .unwrap();
    println!("lookup_asset_symbols([BTS]) => {lookup_assets:#?}");
    assert_eq!(lookup_assets.len(), 1);
    let looked_up_core_asset = lookup_assets[0]
        .as_ref()
        .expect("core asset symbol should resolve");
    assert_eq!(looked_up_core_asset.symbol, "BTS");

    let committee_count = client.call(GetCommitteeCountParams).unwrap();
    println!("get_committee_count => {committee_count}");
    assert_positive(committee_count, "get_committee_count");

    let committee_members = client
        .call(GetCommitteeMembersParams {
            committee_member_ids: vec!["1.5.0".to_owned()],
        })
        .unwrap();
    println!("get_committee_members([1.5.0]) => {committee_members:#?}");
    assert_eq!(committee_members.len(), 1);
    let committee_member = committee_members[0]
        .as_ref()
        .expect("committee member should exist");
    assert!(committee_member
        .committee_member_account
        .starts_with("1.2."));

    let committee_member_lookup = client
        .call(LookupCommitteeMemberAccountsParams {
            lower_bound_name: String::new(),
            limit: 1,
        })
        .unwrap();
    println!("lookup_committee_member_accounts('', 1) => {committee_member_lookup:#?}");
    let (_committee_account_name, committee_member_id) = committee_member_lookup
        .iter()
        .next()
        .expect("lookup_committee_member_accounts should return at least one committee member");
    assert_id_like(
        committee_member_id,
        "1.5.",
        "lookup_committee_member_accounts",
    );

    let witness_count = client.call(GetWitnessCountParams).unwrap();
    println!("get_witness_count => {witness_count}");
    assert_positive(witness_count, "get_witness_count");

    let witnesses = client
        .call(GetWitnessesParams {
            witness_ids: vec!["1.6.1".to_owned()],
        })
        .unwrap();
    println!("get_witnesses([1.6.1]) => {witnesses:#?}");
    assert_eq!(witnesses.len(), 1);
    let witness = witnesses[0].as_ref().expect("witness should exist");
    assert!(witness.witness_account.starts_with("1.2."));

    let witness_lookup = client
        .call(LookupWitnessAccountsParams {
            lower_bound_name: String::new(),
            limit: 1,
        })
        .unwrap();
    println!("lookup_witness_accounts('', 1) => {witness_lookup:#?}");
    let (_witness_account_name, witness_id) = witness_lookup
        .iter()
        .next()
        .expect("lookup_witness_accounts should return at least one witness");
    assert_id_like(witness_id, "1.6.", "lookup_witness_accounts");

    let vote_objects = client
        .call(LookupVoteIdsParams {
            votes: vec!["0:5".to_owned(), "1:0".to_owned()],
        })
        .unwrap();
    println!("lookup_vote_ids([0:5, 1:0]) => {vote_objects:#?}");
    assert_eq!(vote_objects.len(), 2);
    match &vote_objects[0] {
        LookupVoteIdObject::CommitteeMember(member) => {
            assert_eq!(member.committee_member_account.as_str(), "1.2.102");
            assert_eq!(member.vote_id.as_str(), "0:5");
        }
        other => panic!("expected committee member vote object, got {other:#?}"),
    }
    match &vote_objects[1] {
        LookupVoteIdObject::Witness(witness) => {
            assert_eq!(witness.witness_account.as_str(), "1.2.102");
            assert_eq!(witness.vote_id.as_str(), "1:0");
        }
        other => panic!("expected witness vote object, got {other:#?}"),
    }

    let worker_count = client.call(GetWorkerCountParams).unwrap();
    println!("get_worker_count => {worker_count}");

    let chain_id = client.call(GetChainIdParams).unwrap();
    println!("get_chain_id => {chain_id:?}");

    let chain_properties = client.call(GetChainPropertiesParams).unwrap();
    println!("get_chain_properties => {chain_properties:#?}");

    let config = client.call(GetConfigParams).unwrap();
    println!("get_config => {config:#?}");
    assert_eq!(
        config
            .get("GRAPHENE_SYMBOL")
            .and_then(|value| value.as_str()),
        Some("BTS")
    );

    let dynamic = client.call(GetDynamicGlobalPropertiesParams).unwrap();
    println!("get_dynamic_global_properties => {dynamic:#?}");
    assert!(dynamic.head_block_number > 0);

    let objects = client
        .call(GetObjectsParams {
            ids: vec!["2.1.0".to_owned()],
            subscribe: Some(false),
        })
        .unwrap();
    println!("get_objects([2.1.0], false) => {objects:#?}");
    assert_eq!(objects.len(), 1);
    match &objects[0] {
        GetObjectResult::DynamicGlobalPropertyObject(object_dynamic) => {
            assert_eq!(object_dynamic.head_block_number, dynamic.head_block_number);
            assert!(object_dynamic.current_witness.starts_with("1.6."));
        }
        other => panic!("expected dynamic global property object, got {other:#?}"),
    }

    let transfer = serde_json::from_value::<Operation>(serde_json::json!([
        0,
        {
            "fee": { "amount": 0, "asset_id": "1.3.0" },
            "from": "1.2.0",
            "to": "1.2.0",
            "amount": { "amount": 1, "asset_id": "1.3.0" },
            "memo": null,
            "extensions": []
        }
    ]))
    .expect("transfer operation fixture should decode");
    let required_fees = client
        .call(GetRequiredFeesParams {
            ops: vec![transfer],
            asset_symbol_or_id: "1.3.0".to_owned(),
        })
        .unwrap();
    println!("get_required_fees([transfer], 1.3.0) => {required_fees:#?}");
    assert_eq!(required_fees.len(), 1);
    match &required_fees[0] {
        RequiredFee::Asset(fee) => {
            assert!(fee.amount.as_i64() > 0);
            assert_eq!(fee.asset_id.as_str(), "1.3.0");
        }
        other => panic!("expected transfer fee asset, got {other:#?}"),
    }

    let global = client.call(GetGlobalPropertiesParams).unwrap();
    println!("get_global_properties => {global:#?}");
    assert!(!global.active_witnesses.is_empty());

    let block_num = dynamic.head_block_number.saturating_sub(1);

    let block_header = client
        .call(GetBlockHeaderParams {
            block_num,
            with_witness_signature: Some(true),
        })
        .unwrap()
        .expect("recent block header should be available");
    println!("get_block_header({block_num}, true) => {block_header:#?}");
    assert!(block_header.witness.starts_with("1.6."));
    assert!(block_header
        .witness_signature
        .as_ref()
        .is_some_and(|signature| !signature.is_empty()));

    let block_headers = client
        .call(GetBlockHeaderBatchParams {
            block_nums: vec![block_num],
            with_witness_signatures: Some(true),
        })
        .unwrap();
    println!("get_block_header_batch([{block_num}], true) => {block_headers:#?}");
    let (_, batch_header) = block_headers
        .iter()
        .find(|(header_block_num, _)| *header_block_num == block_num)
        .expect("block header batch should contain queried block");
    let batch_header = batch_header
        .as_ref()
        .expect("recent block header in batch should be available");
    assert_eq!(batch_header.previous, block_header.previous);
    assert!(batch_header
        .witness_signature
        .as_ref()
        .is_some_and(|signature| !signature.is_empty()));

    let block = client
        .call(GetBlockParams { block_num })
        .unwrap()
        .expect("recent block should be available");
    println!("get_block({block_num}) => {block:#?}");
    assert!(!block.witness_signature.is_empty());
}
