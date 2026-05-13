use std::collections::BTreeSet;

use graphene_chain_swaplock::{
    GetAccountCountParams, GetAssetCountParams, GetDynamicGlobalPropertiesParams,
    GetGlobalPropertiesParams, OPENRPC_METHODS,
};
use graphene_rpc::{HttpTransport, RpcClient, RpcTransport};
use serde_json::json;

const DEFAULT_SWAPLOCK_RPC_URL: &str = "https://node01.swaplock.chainpool.online:8090";

/// Methods currently exercised end-to-end against the public Swaplock RPC node.
///
/// This list is deliberately conservative: it contains read-only methods with
/// stable parameters and typed responses. The coverage test below ensures every
/// generated method is either here, in `RAW_LIVE_METHODS`, or in
/// `SKIPPED_LIVE_METHODS`, so adding new RPC methods forces an explicit
/// live-test decision.
const TYPED_LIVE_METHODS: &[&str] = &[
    "get_account_count",
    "get_asset_count",
    "get_dynamic_global_properties",
    "get_global_properties",
];

/// Methods probed against the public node at the raw JSON layer.
///
/// `get_block` is intentionally raw here: the public node exposes the database
/// API shape (`signed_block`), while the wallet OpenRPC contract names
/// `signed_block_with_info` for the same method.
const RAW_LIVE_METHODS: &[&str] = &["get_block"];

/// Generated RPC methods not yet executed against the public node.
///
/// Most require account/key/object fixtures, wallet state, builder handles, or
/// are transaction/broadcast/admin surfaces. They stay classified here until we
/// add safe live fixtures for them.
const SKIPPED_LIVE_METHODS: &[&str] = &[
    "about",
    "account_store_map",
    "add_operation_to_builder_transaction",
    "add_transaction_signature",
    "approve_proposal",
    "begin_builder_transaction",
    "bid_collateral",
    "blind_history",
    "blind_transfer",
    "borrow_asset",
    "borrow_asset_ext",
    "broadcast_transaction",
    "cancel_order",
    "claim_asset_fee_pool",
    "create_account_with_brain_key",
    "create_asset",
    "create_blind_account",
    "create_committee_member",
    "create_witness",
    "create_worker",
    "dbg_generate_blocks",
    "dbg_make_mia",
    "dbg_make_uia",
    "dbg_push_blocks",
    "dbg_stream_json_objects",
    "dbg_update_object",
    "derive_owner_keys_from_brain_key",
    "dump_private_keys",
    "flood_network",
    "fund_asset_fee_pool",
    "get_account",
    "get_account_history",
    "get_account_history_by_operations",
    "get_account_id",
    "get_account_limit_orders",
    "get_account_name",
    "get_account_storage",
    "get_asset",
    "get_asset_id",
    "get_asset_name",
    "get_asset_symbol",
    "get_bitasset_data",
    "get_blind_accounts",
    "get_blind_balances",
    "get_call_orders",
    "get_collateral_bids",
    "get_committee_member",
    "get_full_account",
    "get_htlc",
    "get_key_label",
    "get_key_references",
    "get_limit_orders",
    "get_market_history",
    "get_my_blind_accounts",
    "get_object",
    "get_order_book",
    "get_private_key",
    "get_prototype_operation",
    "get_public_key",
    "get_relative_account_history",
    "get_settle_orders",
    "get_transaction_id",
    "get_transaction_signers",
    "get_vesting_balances",
    "get_witness",
    "gethelp",
    "global_settle_asset",
    "help",
    "htlc_create",
    "htlc_extend",
    "htlc_redeem",
    "import_account_keys",
    "import_accounts",
    "import_balance",
    "import_key",
    "info",
    "is_locked",
    "is_new",
    "is_public_key_registered",
    "issue_asset",
    "list_account_balances",
    "list_accounts",
    "list_assets",
    "list_committee_members",
    "list_my_accounts",
    "list_witnesses",
    "load_wallet_file",
    "lock",
    "network_add_nodes",
    "network_get_connected_peers",
    "normalize_brain_key",
    "preview_builder_transaction",
    "propose_builder_transaction",
    "propose_builder_transaction2",
    "propose_fee_change",
    "propose_parameter_change",
    "publish_asset_feed",
    "quit",
    "read_memo",
    "receive_blind_transfer",
    "register_account",
    "remove_builder_transaction",
    "replace_operation_in_builder_transaction",
    "reserve_asset",
    "save_wallet_file",
    "sell_asset",
    "serialize_transaction",
    "set_desired_witness_and_committee_member_count",
    "set_fees_on_builder_transaction",
    "set_key_label",
    "set_password",
    "set_voting_proxy",
    "settle_asset",
    "sign_builder_transaction",
    "sign_builder_transaction2",
    "sign_memo",
    "sign_message",
    "sign_transaction",
    "sign_transaction2",
    "suggest_brain_key",
    "transfer",
    "transfer2",
    "transfer_from_blind",
    "transfer_to_blind",
    "unlock",
    "update_asset",
    "update_asset_feed_producers",
    "update_asset_issuer",
    "update_bitasset",
    "update_witness",
    "update_worker_votes",
    "upgrade_account",
    "verify_encapsulated_message",
    "verify_message",
    "verify_signed_message",
    "vote_for_committee_member",
    "vote_for_witness",
    "whitelist_account",
    "withdraw_vesting",
];

fn live_client() -> RpcClient<HttpTransport> {
    let endpoint =
        std::env::var("SWAPLOCK_RPC_URL").unwrap_or_else(|_| DEFAULT_SWAPLOCK_RPC_URL.to_owned());
    RpcClient::new(HttpTransport::new(endpoint))
}

#[test]
fn every_generated_method_has_a_live_test_decision() {
    let generated = OPENRPC_METHODS.iter().copied().collect::<BTreeSet<_>>();
    let classified = TYPED_LIVE_METHODS
        .iter()
        .chain(RAW_LIVE_METHODS.iter())
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
fn live_decodes_read_only_rpc_methods_over_http() {
    let client = live_client();

    let account_count = client.call(GetAccountCountParams).unwrap();
    println!("get_account_count => {account_count}");
    assert!(account_count > 0);

    let asset_count = client.call(GetAssetCountParams).unwrap();
    println!("get_asset_count => {asset_count}");
    assert!(asset_count > 0);

    let dynamic = client.call(GetDynamicGlobalPropertiesParams).unwrap();
    println!("get_dynamic_global_properties => {dynamic:#?}");
    assert!(dynamic.head_block_number > 0);

    let global = client.call(GetGlobalPropertiesParams).unwrap();
    println!("get_global_properties => {global:#?}");
    assert!(!global.active_witnesses.is_empty());

    let block = client
        .transport()
        .call_raw("get_block", vec![json!(dynamic.head_block_number)])
        .unwrap();
    println!("get_block({}) => {block:#}", dynamic.head_block_number);
    assert!(block.get("timestamp").is_some());
    assert!(block.get("witness_signature").is_some());
}
