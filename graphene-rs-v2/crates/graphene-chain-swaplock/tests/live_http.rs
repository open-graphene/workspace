use std::collections::BTreeSet;

use graphene_chain_swaplock::{
    GetAccountCountParams, GetAssetCountParams, GetBlockParams, GetChainIdParams,
    GetChainPropertiesParams, GetCommitteeCountParams, GetConfigParams,
    GetDynamicGlobalPropertiesParams, GetGlobalPropertiesParams, GetWitnessCountParams,
    GetWorkerCountParams, OPENRPC_METHODS,
};
use graphene_rpc::{GrapheneUInt64, HttpTransport, RpcClient};

const DEFAULT_SWAPLOCK_RPC_URL: &str = "https://node01.swaplock.chainpool.online:8090";

/// Methods currently exercised end-to-end against the public Swaplock database RPC node.
///
/// This list is deliberately conservative: it contains read-only methods with
/// stable parameters and typed responses. The coverage test below ensures every
/// generated method is either here or in `SKIPPED_LIVE_METHODS`, so adding new
/// RPC methods forces an explicit live-test decision.
const TYPED_LIVE_METHODS: &[&str] = &[
    "get_account_count",
    "get_asset_count",
    "get_block",
    "get_chain_id",
    "get_chain_properties",
    "get_committee_count",
    "get_config",
    "get_dynamic_global_properties",
    "get_global_properties",
    "get_witness_count",
    "get_worker_count",
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
    "get_assets",
    "get_assets_by_issuer",
    "get_balance_objects",
    "get_blinded_balances",
    "get_block_header",
    "get_block_header_batch",
    "get_call_orders",
    "get_call_orders_by_account",
    "get_collateral_bids",
    "get_committee_member_by_account",
    "get_committee_members",
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
    "get_objects",
    "get_order_book",
    "get_potential_address_signatures",
    "get_potential_signatures",
    "get_proposed_transactions",
    "get_recent_transaction_by_id",
    "get_required_fees",
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
    "get_transaction_hex_without_sig",
    "get_vested_balances",
    "get_vesting_balances",
    "get_withdraw_permissions_by_giver",
    "get_withdraw_permissions_by_recipient",
    "get_witness_by_account",
    "get_witnesses",
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
    "lookup_accounts",
    "lookup_asset_symbols",
    "lookup_committee_member_accounts",
    "lookup_vote_ids",
    "lookup_witness_accounts",
    "set_auto_subscription",
    "unsubscribe_from_market",
    "validate_transaction",
    "verify_account_authority",
    "verify_authority",
];

fn live_client() -> RpcClient<HttpTransport> {
    let endpoint =
        std::env::var("SWAPLOCK_RPC_URL").unwrap_or_else(|_| DEFAULT_SWAPLOCK_RPC_URL.to_owned());
    RpcClient::new(HttpTransport::new(endpoint))
}

fn assert_positive(value: GrapheneUInt64, method: &str) {
    assert!(
        value.as_u64() > 0,
        "{method} should return a positive count"
    );
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
fn live_decodes_read_only_rpc_methods_over_http() {
    let client = live_client();

    let account_count = client.call(GetAccountCountParams).unwrap();
    println!("get_account_count => {account_count}");
    assert_positive(account_count, "get_account_count");

    let asset_count = client.call(GetAssetCountParams).unwrap();
    println!("get_asset_count => {asset_count}");
    assert_positive(asset_count, "get_asset_count");

    let committee_count = client.call(GetCommitteeCountParams).unwrap();
    println!("get_committee_count => {committee_count}");
    assert_positive(committee_count, "get_committee_count");

    let witness_count = client.call(GetWitnessCountParams).unwrap();
    println!("get_witness_count => {witness_count}");
    assert_positive(witness_count, "get_witness_count");

    let worker_count = client.call(GetWorkerCountParams).unwrap();
    println!("get_worker_count => {worker_count}");

    let chain_id = client.call(GetChainIdParams).unwrap();
    println!("get_chain_id => {chain_id:?}");

    let chain_properties = client.call(GetChainPropertiesParams).unwrap();
    println!("get_chain_properties => {chain_properties:#?}");

    let config = client.call(GetConfigParams).unwrap();
    println!("get_config => {config:#}");
    assert_eq!(
        config
            .get("GRAPHENE_SYMBOL")
            .and_then(|value| value.as_str()),
        Some("BTS")
    );

    let dynamic = client.call(GetDynamicGlobalPropertiesParams).unwrap();
    println!("get_dynamic_global_properties => {dynamic:#?}");
    assert!(dynamic.head_block_number > 0);

    let global = client.call(GetGlobalPropertiesParams).unwrap();
    println!("get_global_properties => {global:#?}");
    assert!(!global.active_witnesses.is_empty());

    let block_num = dynamic.head_block_number.saturating_sub(1);
    let block = client
        .call(GetBlockParams { block_num })
        .unwrap()
        .expect("recent block should be available");
    println!("get_block({block_num}) => {block:#?}");
    assert!(!block.witness_signature.is_empty());
}
