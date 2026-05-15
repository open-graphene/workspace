// AUTO-GENERATED legacy Graphene Rust RPC bindings; do not edit by hand.
// Source: legacy generated OpenRPC swaplock/spec.json (methods)

use graphene_rpc::OpenRpcParams;

/// OpenRPC method names generated for this chain crate.
pub const OPENRPC_METHODS: &[&str] = &[
    "cancel_all_subscriptions",
    "get_24_volume",
    "get_account_balances",
    "get_account_by_name",
    "get_account_count",
    "get_account_id_from_string",
    "get_account_limit_orders",
    "get_account_references",
    "get_accounts",
    "get_all_workers",
    "get_asset_count",
    "get_asset_id_from_string",
    "get_assets",
    "get_assets_by_issuer",
    "get_balance_objects",
    "get_blinded_balances",
    "get_block",
    "get_block_header",
    "get_block_header_batch",
    "get_call_orders",
    "get_call_orders_by_account",
    "get_chain_id",
    "get_chain_properties",
    "get_collateral_bids",
    "get_committee_count",
    "get_committee_member_by_account",
    "get_committee_members",
    "get_config",
    "get_credit_deals_by_borrower",
    "get_credit_deals_by_collateral_asset",
    "get_credit_deals_by_debt_asset",
    "get_credit_deals_by_offer_id",
    "get_credit_deals_by_offer_owner",
    "get_credit_offers_by_asset",
    "get_credit_offers_by_owner",
    "get_dynamic_global_properties",
    "get_full_accounts",
    "get_global_properties",
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
    "get_witness_count",
    "get_witnesses",
    "get_worker_count",
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

/// Typed object union returned by `get_objects`.
///
/// Graphene `get_objects` accepts arbitrary object IDs and returns the
/// corresponding concrete chain object. Missing objects may be returned as
/// JSON `null` by some nodes.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum GetObjectResult {
    AccountBalanceObject(AccountBalanceObject),
    AccountObject(AccountObject),
    AccountStatisticsObject(AccountStatisticsObject),
    BalanceObject(BalanceObject),
    BlindedBalanceObject(BlindedBalanceObject),
    CallOrderObject(CallOrderObject),
    ChainPropertyObject(ChainPropertyObject),
    CollateralBidObject(CollateralBidObject),
    CommitteeMemberObject(CommitteeMemberObject),
    CreditDealObject(CreditDealObject),
    CreditOfferObject(CreditOfferObject),
    DynamicGlobalPropertyObject(DynamicGlobalPropertyObject),
    ExtendedAssetObject(ExtendedAssetObject),
    ExtendedLiquidityPoolObject(ExtendedLiquidityPoolObject),
    ForceSettlementObject(ForceSettlementObject),
    GlobalPropertyObject(GlobalPropertyObject),
    HtlcObject(HtlcObject),
    LimitOrderObject(LimitOrderObject),
    MarketHistoryLiquidityPoolTickerObject(MarketHistoryLiquidityPoolTickerObject),
    ProposalObject(ProposalObject),
    SametFundObject(SametFundObject),
    TicketObject(TicketObject),
    VestingBalanceObject(VestingBalanceObject),
    WithdrawPermissionObject(WithdrawPermissionObject),
    WitnessObject(WitnessObject),
    WorkerObject(WorkerObject),
    Null(()),
}

/// Typed fee result returned by `get_required_fees`.
///
/// Normal operations return a single `asset` fee. Proposal-create operations
/// return an FC pair of the proposal fee and recursively nested proposed
/// operation fees: `[fee, [nested_fee, ...]]`.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum RequiredFee {
    Asset(Asset),
    ProposalCreate((Asset, Vec<RequiredFee>)),
}

/// Typed object union returned by `lookup_vote_ids`.
///
/// Graphene returns concrete vote target objects in one heterogeneous array.
/// The object `id` space identifies the concrete shape: committee members
/// use `1.5.x`, witnesses use `1.6.x`, and workers use the worker object space.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum LookupVoteIdObject {
    CommitteeMember(CommitteeMemberObject),
    Witness(WitnessObject),
    Worker(WorkerObject),
}

/// This unsubscribes from all subscribed markets and objects.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CancelAllSubscriptionsParams;

impl OpenRpcParams for CancelAllSubscriptionsParams {
    const METHOD: &'static str = "cancel_all_subscriptions";
    type Response = ();

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        Vec::new()
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Get24VolumeParams {
    /// symbol name or ID of the base asset
    pub base: String,
    /// symbol name or ID of the quote asset
    pub quote: String,
}

impl OpenRpcParams for Get24VolumeParams {
    const METHOD: &'static str = "get_24_volume";
    type Response = MarketVolume;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.base),
            serde_json::json!(self.quote),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetAccountBalancesParams {
    /// name or ID of the account to get balances for
    pub account_name_or_id: String,
    /// IDs of the assets to get balances of; if empty, get all assets account has a balance in
    pub assets: Vec<String>,
}

impl OpenRpcParams for GetAccountBalancesParams {
    const METHOD: &'static str = "get_account_balances";
    type Response = Vec<Asset>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.account_name_or_id),
            serde_json::json!(self.assets),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetAccountByNameParams {
    /// Name of the account to retrieve
    pub name: String,
}

impl OpenRpcParams for GetAccountByNameParams {
    const METHOD: &'static str = "get_account_by_name";
    type Response = Option<AccountObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.name),
        ]
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GetAccountCountParams;

impl OpenRpcParams for GetAccountCountParams {
    const METHOD: &'static str = "get_account_count";
    type Response = ::graphene_rpc::GrapheneUInt64;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        Vec::new()
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetAccountIdFromStringParams {
    /// name or ID of the account
    pub name_or_id: String,
}

impl OpenRpcParams for GetAccountIdFromStringParams {
    const METHOD: &'static str = "get_account_id_from_string";
    type Response = String;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.name_or_id),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetAccountLimitOrdersParams {
    /// The name or ID of an account to retrieve
    pub account_name_or_id: String,
    /// Base asset
    pub base: String,
    /// Quote asset
    pub quote: String,
    /// The limitation of items each query can fetch, not greater than the configured value of api_limit_get_account_limit_orders
    pub limit: u32,
    /// Start order id, fetch orders which price lower than this order, or price equal to this order but order ID greater than this order
    pub ostart_id: Option<String>,
    /// Fetch orders with price lower than or equal to this price
    pub ostart_price: Option<Price>,
}

impl OpenRpcParams for GetAccountLimitOrdersParams {
    const METHOD: &'static str = "get_account_limit_orders";
    type Response = Vec<LimitOrderObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.account_name_or_id),
            serde_json::json!(self.base),
            serde_json::json!(self.quote),
            serde_json::json!(self.limit),
            serde_json::json!(self.ostart_id),
            serde_json::json!(self.ostart_price),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetAccountReferencesParams {
    /// Account name or ID to query
    pub account_name_or_id: String,
}

impl OpenRpcParams for GetAccountReferencesParams {
    const METHOD: &'static str = "get_account_references";
    type Response = Vec<String>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.account_name_or_id),
        ]
    }
}

/// This function has semantics identical to get_objects
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetAccountsParams {
    /// names or IDs of the accounts to retrieve
    pub account_names_or_ids: Vec<String>,
    /// true to subscribe to the queried account objects, false to not subscribe, null to subscribe or not subscribe according to current auto-subscription setting (see set_auto_subscription)
    pub subscribe: Option<bool>,
}

impl OpenRpcParams for GetAccountsParams {
    const METHOD: &'static str = "get_accounts";
    type Response = Vec<Option<AccountObject>>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.account_names_or_ids),
            serde_json::json!(self.subscribe),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetAllWorkersParams {
    /// null for all workers, true for expired workers only, false for non-expired workers only
    pub is_expired: Option<bool>,
}

impl OpenRpcParams for GetAllWorkersParams {
    const METHOD: &'static str = "get_all_workers";
    type Response = Vec<WorkerObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.is_expired),
        ]
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GetAssetCountParams;

impl OpenRpcParams for GetAssetCountParams {
    const METHOD: &'static str = "get_asset_count";
    type Response = ::graphene_rpc::GrapheneUInt64;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        Vec::new()
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetAssetIdFromStringParams {
    /// symbol name or ID of the asset
    pub symbol_or_id: String,
}

impl OpenRpcParams for GetAssetIdFromStringParams {
    const METHOD: &'static str = "get_asset_id_from_string";
    type Response = String;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.symbol_or_id),
        ]
    }
}

/// This function has semantics identical to get_objects
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetAssetsParams {
    /// symbol names or IDs of the assets to retrieve
    pub asset_symbols_or_ids: Vec<String>,
    /// true to subscribe to the queried asset objects, false to not subscribe, null to subscribe or not subscribe according to current auto-subscription setting (see set_auto_subscription)
    pub subscribe: Option<bool>,
}

impl OpenRpcParams for GetAssetsParams {
    const METHOD: &'static str = "get_assets";
    type Response = Vec<Option<ExtendedAssetObject>>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.asset_symbols_or_ids),
            serde_json::json!(self.subscribe),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetAssetsByIssuerParams {
    /// Account name or ID to get objects from
    pub issuer_name_or_id: String,
    /// Asset objects(1.3.X) before this ID will be skipped in results. Pagination purposes.
    pub start: String,
    /// Maximum number of assets to retrieve, must not exceed the configured value of api_limit_get_assets
    pub limit: u32,
}

impl OpenRpcParams for GetAssetsByIssuerParams {
    const METHOD: &'static str = "get_assets_by_issuer";
    type Response = Vec<ExtendedAssetObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.issuer_name_or_id),
            serde_json::json!(self.start),
            serde_json::json!(self.limit),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetBalanceObjectsParams {
    /// a list of addresses
    pub addrs: Vec<Address>,
}

impl OpenRpcParams for GetBalanceObjectsParams {
    const METHOD: &'static str = "get_balance_objects";
    type Response = Vec<BalanceObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.addrs),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetBlindedBalancesParams {
    /// a set of commitments to query for
    pub commitments: Vec<String>,
}

impl OpenRpcParams for GetBlindedBalancesParams {
    const METHOD: &'static str = "get_blinded_balances";
    type Response = Vec<BlindedBalanceObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.commitments),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetBlockParams {
    /// Height of the block to be returned
    pub block_num: u32,
}

impl OpenRpcParams for GetBlockParams {
    const METHOD: &'static str = "get_block";
    type Response = Option<SignedBlock>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.block_num),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetBlockHeaderParams {
    /// Height of the block whose header should be returned
    pub block_num: u32,
    /// Whether to return witness signature. Optional. If omitted or is false, will not return witness signature.
    pub with_witness_signature: Option<bool>,
}

impl OpenRpcParams for GetBlockHeaderParams {
    const METHOD: &'static str = "get_block_header";
    type Response = Option<MaybeSignedBlockHeader>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.block_num),
            serde_json::json!(self.with_witness_signature),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetBlockHeaderBatchParams {
    /// vector containing heights of the blocks whose headers should be returned
    pub block_nums: Vec<u32>,
    /// Whether to return witness signatures. Optional. If omitted or is false, will not return witness signatures.
    pub with_witness_signatures: Option<bool>,
}

impl OpenRpcParams for GetBlockHeaderBatchParams {
    const METHOD: &'static str = "get_block_header_batch";
    type Response = Vec<(u32, Option<MaybeSignedBlockHeader>)>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.block_nums),
            serde_json::json!(self.with_witness_signatures),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetCallOrdersParams {
    /// symbol name or ID of the debt asset
    pub a: String,
    /// Maximum number of orders to retrieve, must not exceed the configured value of api_limit_get_call_orders
    pub limit: u32,
}

impl OpenRpcParams for GetCallOrdersParams {
    const METHOD: &'static str = "get_call_orders";
    type Response = Vec<CallOrderObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.a),
            serde_json::json!(self.limit),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetCallOrdersByAccountParams {
    /// Account name or ID to get objects from
    pub account_name_or_id: String,
    /// Asset objects(1.3.X) before this ID will be skipped in results. Pagination purposes.
    pub start: String,
    /// Maximum number of orders to retrieve, must not exceed the configured value of api_limit_get_call_orders
    pub limit: u32,
}

impl OpenRpcParams for GetCallOrdersByAccountParams {
    const METHOD: &'static str = "get_call_orders_by_account";
    type Response = Vec<CallOrderObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.account_name_or_id),
            serde_json::json!(self.start),
            serde_json::json!(self.limit),
        ]
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GetChainIdParams;

impl OpenRpcParams for GetChainIdParams {
    const METHOD: &'static str = "get_chain_id";
    type Response = ChainIdType;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        Vec::new()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GetChainPropertiesParams;

impl OpenRpcParams for GetChainPropertiesParams {
    const METHOD: &'static str = "get_chain_properties";
    type Response = ChainPropertyObject;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        Vec::new()
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetCollateralBidsParams {
    /// Symbol or ID of asset
    pub a: String,
    /// Maximum number of objects to retrieve, must not exceed the configured value of api_limit_get_collateral_bids
    pub limit: u32,
    /// skip that many results
    pub start: u32,
}

impl OpenRpcParams for GetCollateralBidsParams {
    const METHOD: &'static str = "get_collateral_bids";
    type Response = Vec<CollateralBidObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.a),
            serde_json::json!(self.limit),
            serde_json::json!(self.start),
        ]
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GetCommitteeCountParams;

impl OpenRpcParams for GetCommitteeCountParams {
    const METHOD: &'static str = "get_committee_count";
    type Response = ::graphene_rpc::GrapheneUInt64;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        Vec::new()
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetCommitteeMemberByAccountParams {
    /// The name or ID of the account whose committee_member should be retrieved
    pub account_name_or_id: String,
}

impl OpenRpcParams for GetCommitteeMemberByAccountParams {
    const METHOD: &'static str = "get_committee_member_by_account";
    type Response = Option<CommitteeMemberObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.account_name_or_id),
        ]
    }
}

/// This function has semantics identical to get_objects, but doesn't subscribe
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetCommitteeMembersParams {
    /// IDs of the committee_members to retrieve
    pub committee_member_ids: Vec<String>,
}

impl OpenRpcParams for GetCommitteeMembersParams {
    const METHOD: &'static str = "get_committee_members";
    type Response = Vec<Option<CommitteeMemberObject>>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.committee_member_ids),
        ]
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GetConfigParams;

impl OpenRpcParams for GetConfigParams {
    const METHOD: &'static str = "get_config";
    type Response = std::collections::BTreeMap<String, serde_json::Value>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        Vec::new()
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetCreditDealsByBorrowerParams {
    /// name or ID of the borrower account
    pub account_name_or_id: String,
    /// The limitation of items each query can fetch, not greater than the configured value of api_limit_get_credit_offers
    pub limit: Option<u32>,
    /// Start credit deal id, fetch items whose IDs are greater than or equal to this ID
    pub start_id: Option<String>,
}

impl OpenRpcParams for GetCreditDealsByBorrowerParams {
    const METHOD: &'static str = "get_credit_deals_by_borrower";
    type Response = Vec<CreditDealObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.account_name_or_id),
            serde_json::json!(self.limit),
            serde_json::json!(self.start_id),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetCreditDealsByCollateralAssetParams {
    /// symbol or ID of the collateral asset type
    pub asset_symbol_or_id: String,
    /// The limitation of items each query can fetch, not greater than the configured value of api_limit_get_credit_offers
    pub limit: Option<u32>,
    /// Start credit deal id, fetch items whose IDs are greater than or equal to this ID
    pub start_id: Option<String>,
}

impl OpenRpcParams for GetCreditDealsByCollateralAssetParams {
    const METHOD: &'static str = "get_credit_deals_by_collateral_asset";
    type Response = Vec<CreditDealObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.asset_symbol_or_id),
            serde_json::json!(self.limit),
            serde_json::json!(self.start_id),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetCreditDealsByDebtAssetParams {
    /// symbol or ID of the debt asset type
    pub asset_symbol_or_id: String,
    /// The limitation of items each query can fetch, not greater than the configured value of api_limit_get_credit_offers
    pub limit: Option<u32>,
    /// Start credit deal id, fetch items whose IDs are greater than or equal to this ID
    pub start_id: Option<String>,
}

impl OpenRpcParams for GetCreditDealsByDebtAssetParams {
    const METHOD: &'static str = "get_credit_deals_by_debt_asset";
    type Response = Vec<CreditDealObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.asset_symbol_or_id),
            serde_json::json!(self.limit),
            serde_json::json!(self.start_id),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetCreditDealsByOfferIdParams {
    /// ID of the credit offer
    pub offer_id: String,
    /// The limitation of items each query can fetch, not greater than the configured value of api_limit_get_credit_offers
    pub limit: Option<u32>,
    /// Start credit deal id, fetch items whose IDs are greater than or equal to this ID
    pub start_id: Option<String>,
}

impl OpenRpcParams for GetCreditDealsByOfferIdParams {
    const METHOD: &'static str = "get_credit_deals_by_offer_id";
    type Response = Vec<CreditDealObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.offer_id),
            serde_json::json!(self.limit),
            serde_json::json!(self.start_id),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetCreditDealsByOfferOwnerParams {
    /// name or ID of the credit offer owner account
    pub account_name_or_id: String,
    /// The limitation of items each query can fetch, not greater than the configured value of api_limit_get_credit_offers
    pub limit: Option<u32>,
    /// Start credit deal id, fetch items whose IDs are greater than or equal to this ID
    pub start_id: Option<String>,
}

impl OpenRpcParams for GetCreditDealsByOfferOwnerParams {
    const METHOD: &'static str = "get_credit_deals_by_offer_owner";
    type Response = Vec<CreditDealObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.account_name_or_id),
            serde_json::json!(self.limit),
            serde_json::json!(self.start_id),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetCreditOffersByAssetParams {
    /// symbol or ID of the asset type
    pub asset_symbol_or_id: String,
    /// The limitation of items each query can fetch, not greater than the configured value of api_limit_get_credit_offers
    pub limit: Option<u32>,
    /// Start credit offer id, fetch items whose IDs are greater than or equal to this ID
    pub start_id: Option<String>,
}

impl OpenRpcParams for GetCreditOffersByAssetParams {
    const METHOD: &'static str = "get_credit_offers_by_asset";
    type Response = Vec<CreditOfferObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.asset_symbol_or_id),
            serde_json::json!(self.limit),
            serde_json::json!(self.start_id),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetCreditOffersByOwnerParams {
    /// name or ID of the owner account
    pub account_name_or_id: String,
    /// The limitation of items each query can fetch, not greater than the configured value of api_limit_get_credit_offers
    pub limit: Option<u32>,
    /// Start credit offer id, fetch items whose IDs are greater than or equal to this ID
    pub start_id: Option<String>,
}

impl OpenRpcParams for GetCreditOffersByOwnerParams {
    const METHOD: &'static str = "get_credit_offers_by_owner";
    type Response = Vec<CreditOfferObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.account_name_or_id),
            serde_json::json!(self.limit),
            serde_json::json!(self.start_id),
        ]
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GetDynamicGlobalPropertiesParams;

impl OpenRpcParams for GetDynamicGlobalPropertiesParams {
    const METHOD: &'static str = "get_dynamic_global_properties";
    type Response = DynamicGlobalPropertyObject;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        Vec::new()
    }
}

/// This function fetches relevant objects for the given accounts, and subscribes to updates to the given accounts. If any of the strings in names_or_ids cannot be tied to an account, that input will be ignored. Other accounts will be retrieved and subscribed.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetFullAccountsParams {
    /// Each item must be the name or ID of an account to retrieve, the quantity should not be greater than the configured value of api_limit_get_full_accounts
    pub names_or_ids: Vec<String>,
    /// true to subscribe to the queried full account objects, false to not subscribe, null to subscribe or not subscribe according to current auto-subscription setting (see set_auto_subscription)
    pub subscribe: Option<bool>,
}

impl OpenRpcParams for GetFullAccountsParams {
    const METHOD: &'static str = "get_full_accounts";
    type Response = Vec<(String, FullAccount)>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.names_or_ids),
            serde_json::json!(self.subscribe),
        ]
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GetGlobalPropertiesParams;

impl OpenRpcParams for GetGlobalPropertiesParams {
    const METHOD: &'static str = "get_global_properties";
    type Response = GlobalPropertyObject;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        Vec::new()
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetHtlcParams {
    /// HTLC contract id
    pub id: String,
    /// true to subscribe to the queried HTLC objects, false to not subscribe, null to subscribe or not subscribe according to current auto-subscription setting (see set_auto_subscription)
    pub subscribe: Option<bool>,
}

impl OpenRpcParams for GetHtlcParams {
    const METHOD: &'static str = "get_htlc";
    type Response = Option<HtlcObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.id),
            serde_json::json!(self.subscribe),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetHtlcByFromParams {
    /// Account name or ID to get objects from
    pub account_name_or_id: String,
    /// htlc objects before this ID will be skipped in results. Pagination purposes.
    pub start: String,
    /// Maximum number of objects to retrieve, must not exceed the configured value of api_limit_get_htlc_by
    pub limit: u32,
}

impl OpenRpcParams for GetHtlcByFromParams {
    const METHOD: &'static str = "get_htlc_by_from";
    type Response = Vec<HtlcObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.account_name_or_id),
            serde_json::json!(self.start),
            serde_json::json!(self.limit),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetHtlcByToParams {
    /// Account name or ID to get objects from
    pub account_name_or_id: String,
    /// htlc objects before this ID will be skipped in results. Pagination purposes.
    pub start: String,
    /// Maximum number of objects to retrieve, must not exceed the configured value of api_limit_get_htlc_by
    pub limit: u32,
}

impl OpenRpcParams for GetHtlcByToParams {
    const METHOD: &'static str = "get_htlc_by_to";
    type Response = Vec<HtlcObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.account_name_or_id),
            serde_json::json!(self.start),
            serde_json::json!(self.limit),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetKeyReferencesParams {
    /// a list of public keys to query, the quantity should not be greater than the configured value of api_limit_get_key_references
    pub keys: Vec<String>,
}

impl OpenRpcParams for GetKeyReferencesParams {
    const METHOD: &'static str = "get_key_references";
    type Response = Vec<Vec<String>>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.keys),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetLimitOrdersParams {
    /// symbol or ID of asset being sold
    pub a: String,
    /// symbol or ID of asset being purchased
    pub b: String,
    /// Maximum number of orders to retrieve, must not exceed the configured value of api_limit_get_limit_orders
    pub limit: u32,
}

impl OpenRpcParams for GetLimitOrdersParams {
    const METHOD: &'static str = "get_limit_orders";
    type Response = Vec<LimitOrderObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.a),
            serde_json::json!(self.b),
            serde_json::json!(self.limit),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetLimitOrdersByAccountParams {
    /// The name or ID of an account to retrieve
    pub account_name_or_id: String,
    /// The limitation of items each query can fetch, not greater than the configured value of api_limit_get_limit_orders_by_account
    pub limit: Option<u32>,
    /// Start order id, fetch orders whose IDs are greater than or equal to this order
    pub start_id: Option<String>,
}

impl OpenRpcParams for GetLimitOrdersByAccountParams {
    const METHOD: &'static str = "get_limit_orders_by_account";
    type Response = Vec<LimitOrderObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.account_name_or_id),
            serde_json::json!(self.limit),
            serde_json::json!(self.start_id),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetLiquidityPoolsParams {
    /// IDs of the liquidity pools, the quantity should not be greater than the configured value of api_limit_get_liquidity_pools
    pub ids: Vec<String>,
    /// true to subscribe to the queried objects, false to not subscribe, null to subscribe or not subscribe according to current auto-subscription setting (see set_auto_subscription)
    pub subscribe: Option<bool>,
    /// Whether to return statistics
    pub with_statistics: Option<bool>,
}

impl OpenRpcParams for GetLiquidityPoolsParams {
    const METHOD: &'static str = "get_liquidity_pools";
    type Response = Vec<Option<ExtendedLiquidityPoolObject>>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.ids),
            serde_json::json!(self.subscribe),
            serde_json::json!(self.with_statistics),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetLiquidityPoolsByAssetAParams {
    /// symbol name or ID of the asset
    pub asset_symbol_or_id: String,
    /// The limitation of items each query can fetch, not greater than the configured value of api_limit_get_liquidity_pools
    pub limit: Option<u32>,
    /// Start liquidity pool id, fetch pools whose IDs are greater than or equal to this ID
    pub start_id: Option<String>,
    /// Whether to return statistics
    pub with_statistics: Option<bool>,
}

impl OpenRpcParams for GetLiquidityPoolsByAssetAParams {
    const METHOD: &'static str = "get_liquidity_pools_by_asset_a";
    type Response = Vec<ExtendedLiquidityPoolObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.asset_symbol_or_id),
            serde_json::json!(self.limit),
            serde_json::json!(self.start_id),
            serde_json::json!(self.with_statistics),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetLiquidityPoolsByAssetBParams {
    /// symbol name or ID of the asset
    pub asset_symbol_or_id: String,
    /// The limitation of items each query can fetch, not greater than the configured value of api_limit_get_liquidity_pools
    pub limit: Option<u32>,
    /// Start liquidity pool id, fetch pools whose IDs are greater than or equal to this ID
    pub start_id: Option<String>,
    /// Whether to return statistics
    pub with_statistics: Option<bool>,
}

impl OpenRpcParams for GetLiquidityPoolsByAssetBParams {
    const METHOD: &'static str = "get_liquidity_pools_by_asset_b";
    type Response = Vec<ExtendedLiquidityPoolObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.asset_symbol_or_id),
            serde_json::json!(self.limit),
            serde_json::json!(self.start_id),
            serde_json::json!(self.with_statistics),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetLiquidityPoolsByBothAssetsParams {
    /// symbol name or ID of one asset
    pub asset_symbol_or_id_a: String,
    /// symbol name or ID of the other asset
    pub asset_symbol_or_id_b: String,
    /// The limitation of items each query can fetch, not greater than the configured value of api_limit_get_liquidity_pools
    pub limit: Option<u32>,
    /// Start liquidity pool id, fetch pools whose IDs are greater than or equal to this ID
    pub start_id: Option<String>,
    /// Whether to return statistics
    pub with_statistics: Option<bool>,
}

impl OpenRpcParams for GetLiquidityPoolsByBothAssetsParams {
    const METHOD: &'static str = "get_liquidity_pools_by_both_assets";
    type Response = Vec<ExtendedLiquidityPoolObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.asset_symbol_or_id_a),
            serde_json::json!(self.asset_symbol_or_id_b),
            serde_json::json!(self.limit),
            serde_json::json!(self.start_id),
            serde_json::json!(self.with_statistics),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetLiquidityPoolsByOneAssetParams {
    /// symbol name or ID of the asset
    pub asset_symbol_or_id: String,
    /// The limitation of items each query can fetch, not greater than the configured value of api_limit_get_liquidity_pools
    pub limit: Option<u32>,
    /// Start liquidity pool id, fetch pools whose IDs are greater than or equal to this ID
    pub start_id: Option<String>,
    /// Whether to return statistics
    pub with_statistics: Option<bool>,
}

impl OpenRpcParams for GetLiquidityPoolsByOneAssetParams {
    const METHOD: &'static str = "get_liquidity_pools_by_one_asset";
    type Response = Vec<ExtendedLiquidityPoolObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.asset_symbol_or_id),
            serde_json::json!(self.limit),
            serde_json::json!(self.start_id),
            serde_json::json!(self.with_statistics),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetLiquidityPoolsByOwnerParams {
    /// name or ID of the owner account
    pub account_name_or_id: String,
    /// The limitation of items each query can fetch, not greater than the configured value of api_limit_get_liquidity_pools
    pub limit: Option<u32>,
    /// Start share asset id, fetch pools whose share asset IDs are greater than or equal to this ID
    pub start_id: Option<String>,
    /// Whether to return statistics
    pub with_statistics: Option<bool>,
}

impl OpenRpcParams for GetLiquidityPoolsByOwnerParams {
    const METHOD: &'static str = "get_liquidity_pools_by_owner";
    type Response = Vec<ExtendedLiquidityPoolObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.account_name_or_id),
            serde_json::json!(self.limit),
            serde_json::json!(self.start_id),
            serde_json::json!(self.with_statistics),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetLiquidityPoolsByShareAssetParams {
    /// symbol names or IDs of the share assets, the quantity should not be greater than the configured value of api_limit_get_liquidity_pools
    pub asset_symbols_or_ids: Vec<String>,
    /// true to subscribe to the queried objects, false to not subscribe, null to subscribe or not subscribe according to current auto-subscription setting (see set_auto_subscription)
    pub subscribe: Option<bool>,
    /// Whether to return statistics
    pub with_statistics: Option<bool>,
}

impl OpenRpcParams for GetLiquidityPoolsByShareAssetParams {
    const METHOD: &'static str = "get_liquidity_pools_by_share_asset";
    type Response = Vec<Option<ExtendedLiquidityPoolObject>>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.asset_symbols_or_ids),
            serde_json::json!(self.subscribe),
            serde_json::json!(self.with_statistics),
        ]
    }
}

/// Similar to get_call_orders_by_account, but only the first page will be returned, the page size is the configured value of api_limit_get_call_orders.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetMarginPositionsParams {
    /// name or ID of an account
    pub account_name_or_id: String,
}

impl OpenRpcParams for GetMarginPositionsParams {
    const METHOD: &'static str = "get_margin_positions";
    type Response = Vec<CallOrderObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.account_name_or_id),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetNamedAccountBalancesParams {
    pub name: String,
    pub assets: Vec<String>,
}

impl OpenRpcParams for GetNamedAccountBalancesParams {
    const METHOD: &'static str = "get_named_account_balances";
    type Response = Vec<Asset>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.name),
            serde_json::json!(self.assets),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetNextObjectIdParams {
    /// The space ID
    pub space_id: u8,
    /// The type ID
    pub type_id: u8,
    /// Whether to include pending transactions
    pub with_pending_transactions: bool,
}

impl OpenRpcParams for GetNextObjectIdParams {
    const METHOD: &'static str = "get_next_object_id";
    type Response = String;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.space_id),
            serde_json::json!(self.type_id),
            serde_json::json!(self.with_pending_transactions),
        ]
    }
}

/// If any of the provided IDs does not map to an object, a null variant is returned in its position.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetObjectsParams {
    /// IDs of the objects to retrieve
    pub ids: Vec<String>,
    /// true to subscribe to the queried objects, false to not subscribe, null to subscribe or not subscribe according to current auto-subscription setting (see set_auto_subscription)
    pub subscribe: Option<bool>,
}

impl OpenRpcParams for GetObjectsParams {
    const METHOD: &'static str = "get_objects";
    type Response = Vec<GetObjectResult>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.ids),
            serde_json::json!(self.subscribe),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetOrderBookParams {
    /// symbol name or ID of the base asset
    pub base: String,
    /// symbol name or ID of the quote asset
    pub quote: String,
    /// depth of the order book to retrieve, for bids and asks each, capped at the configured value of api_limit_get_order_book and api_limit_get_limit_orders
    pub limit: u32,
}

impl OpenRpcParams for GetOrderBookParams {
    const METHOD: &'static str = "get_order_book";
    type Response = OrderBook;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.base),
            serde_json::json!(self.quote),
            serde_json::json!(self.limit),
        ]
    }
}

/// This method will return the set of all addresses that could possibly sign for a given transaction.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetPotentialAddressSignaturesParams {
    /// the transaction to be signed
    pub trx: SignedTransaction,
}

impl OpenRpcParams for GetPotentialAddressSignaturesParams {
    const METHOD: &'static str = "get_potential_address_signatures";
    type Response = Vec<Address>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.trx),
        ]
    }
}

/// This method will return the set of all public keys that could possibly sign for a given transaction. This call can be used by wallets to filter their set of public keys to just the relevant subset prior to calling get_required_signatures to get the minimum subset.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetPotentialSignaturesParams {
    /// the transaction to be signed
    pub trx: SignedTransaction,
}

impl OpenRpcParams for GetPotentialSignaturesParams {
    const METHOD: &'static str = "get_potential_signatures";
    type Response = Vec<String>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.trx),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetProposedTransactionsParams {
    /// The name or ID of an account
    pub account_name_or_id: String,
}

impl OpenRpcParams for GetProposedTransactionsParams {
    const METHOD: &'static str = "get_proposed_transactions";
    type Response = Vec<ProposalObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.account_name_or_id),
        ]
    }
}

/// If the transaction has not expired, this method will return the transaction for the given ID or it will return NULL if it is not known. Just because it is not known does not mean it wasn't included in the blockchain.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetRecentTransactionByIdParams {
    /// hash of the transaction
    pub txid: TransactionIdType,
}

impl OpenRpcParams for GetRecentTransactionByIdParams {
    const METHOD: &'static str = "get_recent_transaction_by_id";
    type Response = Option<SignedTransaction>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.txid),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetRequiredFeesParams {
    /// a list of operations to be query for required fees
    pub ops: Vec<Operation>,
    /// symbol name or ID of an asset that to be used to pay the fees
    pub asset_symbol_or_id: String,
}

impl OpenRpcParams for GetRequiredFeesParams {
    const METHOD: &'static str = "get_required_fees";
    type Response = Vec<RequiredFee>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.ops),
            serde_json::json!(self.asset_symbol_or_id),
        ]
    }
}

/// This API will take a partially signed transaction and a set of public keys that the owner has the ability to sign for and return the minimal subset of public keys that should add signatures to the transaction.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetRequiredSignaturesParams {
    /// the transaction to be signed
    pub trx: SignedTransaction,
    /// a set of public keys
    pub available_keys: Vec<String>,
}

impl OpenRpcParams for GetRequiredSignaturesParams {
    const METHOD: &'static str = "get_required_signatures";
    type Response = Vec<String>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.trx),
            serde_json::json!(self.available_keys),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetSametFundsByAssetParams {
    /// symbol or ID of the asset type
    pub asset_symbol_or_id: String,
    /// The limitation of items each query can fetch, not greater than the configured value of api_limit_get_samet_funds
    pub limit: Option<u32>,
    /// Start SameT Fund id, fetch items whose IDs are greater than or equal to this ID
    pub start_id: Option<String>,
}

impl OpenRpcParams for GetSametFundsByAssetParams {
    const METHOD: &'static str = "get_samet_funds_by_asset";
    type Response = Vec<SametFundObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.asset_symbol_or_id),
            serde_json::json!(self.limit),
            serde_json::json!(self.start_id),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetSametFundsByOwnerParams {
    /// name or ID of the owner account
    pub account_name_or_id: String,
    /// The limitation of items each query can fetch, not greater than the configured value of api_limit_get_samet_funds
    pub limit: Option<u32>,
    /// Start SameT Fund id, fetch items whose IDs are greater than or equal to this ID
    pub start_id: Option<String>,
}

impl OpenRpcParams for GetSametFundsByOwnerParams {
    const METHOD: &'static str = "get_samet_funds_by_owner";
    type Response = Vec<SametFundObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.account_name_or_id),
            serde_json::json!(self.limit),
            serde_json::json!(self.start_id),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetSettleOrdersParams {
    /// Symbol or ID of asset being settled
    pub a: String,
    /// Maximum number of orders to retrieve, must not exceed the configured value of api_limit_get_settle_orders
    pub limit: u32,
}

impl OpenRpcParams for GetSettleOrdersParams {
    const METHOD: &'static str = "get_settle_orders";
    type Response = Vec<ForceSettlementObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.a),
            serde_json::json!(self.limit),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetSettleOrdersByAccountParams {
    /// Account name or ID to get objects from
    pub account_name_or_id: String,
    /// Force settlement objects(1.4.X) before this ID will be skipped in results. Pagination purposes.
    pub start: String,
    /// Maximum number of orders to retrieve, must not exceed the configured value of api_limit_get_settle_orders
    pub limit: u32,
}

impl OpenRpcParams for GetSettleOrdersByAccountParams {
    const METHOD: &'static str = "get_settle_orders_by_account";
    type Response = Vec<ForceSettlementObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.account_name_or_id),
            serde_json::json!(self.start),
            serde_json::json!(self.limit),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetTickerParams {
    /// symbol name or ID of the base asset
    pub base: String,
    /// symbol name or ID of the quote asset
    pub quote: String,
}

impl OpenRpcParams for GetTickerParams {
    const METHOD: &'static str = "get_ticker";
    type Response = MarketTicker;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.base),
            serde_json::json!(self.quote),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetTicketsByAccountParams {
    /// name or ID of the owner account
    pub account_name_or_id: String,
    /// The limitation of items each query can fetch, not greater than the configured value of api_limit_get_tickets
    pub limit: Option<u32>,
    /// Start ticket id, fetch tickets whose IDs are greater than or equal to this ID
    pub start_id: Option<String>,
}

impl OpenRpcParams for GetTicketsByAccountParams {
    const METHOD: &'static str = "get_tickets_by_account";
    type Response = Vec<TicketObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.account_name_or_id),
            serde_json::json!(self.limit),
            serde_json::json!(self.start_id),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetTopMarketsParams {
    /// Max number of results, must not exceed the configured value of api_limit_get_top_markets
    pub limit: u32,
}

impl OpenRpcParams for GetTopMarketsParams {
    const METHOD: &'static str = "get_top_markets";
    type Response = Vec<MarketTicker>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.limit),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetTopVotersParams {
    /// Maximum number of accounts to retrieve, must not exceed the configured value of api_limit_get_top_voters
    pub limit: u32,
}

impl OpenRpcParams for GetTopVotersParams {
    const METHOD: &'static str = "get_top_voters";
    type Response = Vec<AccountStatisticsObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.limit),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetTradeHistoryParams {
    /// symbol or ID of the base asset
    pub base: String,
    /// symbol or ID of the quote asset
    pub quote: String,
    /// Start time as a UNIX timestamp, the latest transactions to retrieve
    pub start: ::graphene_rpc::GrapheneTimePointSec,
    /// Stop time as a UNIX timestamp, the earliest transactions to retrieve
    pub stop: ::graphene_rpc::GrapheneTimePointSec,
    /// Maximum quantity of transactions to retrieve, capped at the configured value of api_limit_get_trade_history
    pub limit: u32,
}

impl OpenRpcParams for GetTradeHistoryParams {
    const METHOD: &'static str = "get_trade_history";
    type Response = Vec<MarketTrade>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.base),
            serde_json::json!(self.quote),
            serde_json::json!(self.start),
            serde_json::json!(self.stop),
            serde_json::json!(self.limit),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetTradeHistoryBySequenceParams {
    /// symbol or ID of the base asset
    pub base: String,
    /// symbol or ID of the quote asset
    pub quote: String,
    /// Start sequence as an Integer, the latest transaction to retrieve
    pub start: ::graphene_rpc::GrapheneInt64,
    /// Stop time as a UNIX timestamp, the earliest transactions to retrieve
    pub stop: ::graphene_rpc::GrapheneTimePointSec,
    /// Maximum quantity of transactions to retrieve, capped at the configured value of api_limit_get_trade_history_by_sequence
    pub limit: u32,
}

impl OpenRpcParams for GetTradeHistoryBySequenceParams {
    const METHOD: &'static str = "get_trade_history_by_sequence";
    type Response = Vec<MarketTrade>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.base),
            serde_json::json!(self.quote),
            serde_json::json!(self.start),
            serde_json::json!(self.stop),
            serde_json::json!(self.limit),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetTransactionParams {
    /// height of the block to fetch
    pub block_num: u32,
    /// the index (sequence number) of the transaction in the block, starts from 0
    pub trx_in_block: u32,
}

impl OpenRpcParams for GetTransactionParams {
    const METHOD: &'static str = "get_transaction";
    type Response = ProcessedTransaction;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.block_num),
            serde_json::json!(self.trx_in_block),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetTransactionHexParams {
    /// a transaction to get hexdump from
    pub trx: SignedTransaction,
}

impl OpenRpcParams for GetTransactionHexParams {
    const METHOD: &'static str = "get_transaction_hex";
    type Response = String;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.trx),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetTransactionHexWithoutSigParams {
    /// a transaction to get hexdump from
    pub trx: Transaction,
}

impl OpenRpcParams for GetTransactionHexWithoutSigParams {
    const METHOD: &'static str = "get_transaction_hex_without_sig";
    type Response = String;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.trx),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetVestedBalancesParams {
    /// a list of balance object IDs
    pub objs: Vec<String>,
}

impl OpenRpcParams for GetVestedBalancesParams {
    const METHOD: &'static str = "get_vested_balances";
    type Response = Vec<Asset>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.objs),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetVestingBalancesParams {
    /// name or ID of an account
    pub account_name_or_id: String,
}

impl OpenRpcParams for GetVestingBalancesParams {
    const METHOD: &'static str = "get_vesting_balances";
    type Response = Vec<VestingBalanceObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.account_name_or_id),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetWithdrawPermissionsByGiverParams {
    /// Account name or ID to get objects from
    pub account_name_or_id: String,
    /// Withdraw permission objects(1.12.X) before this ID will be skipped in results. Pagination purposes.
    pub start: String,
    /// Maximum number of objects to retrieve, must not exceed the configured value of api_limit_get_withdraw_permissions_by_giver
    pub limit: u32,
}

impl OpenRpcParams for GetWithdrawPermissionsByGiverParams {
    const METHOD: &'static str = "get_withdraw_permissions_by_giver";
    type Response = Vec<WithdrawPermissionObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.account_name_or_id),
            serde_json::json!(self.start),
            serde_json::json!(self.limit),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetWithdrawPermissionsByRecipientParams {
    /// Account name or ID to get objects from
    pub account_name_or_id: String,
    /// Withdraw permission objects(1.12.X) before this ID will be skipped in results. Pagination purposes.
    pub start: String,
    /// Maximum number of objects to retrieve, must not exceed the configured value of api_limit_get_withdraw_permissions_by_recipient
    pub limit: u32,
}

impl OpenRpcParams for GetWithdrawPermissionsByRecipientParams {
    const METHOD: &'static str = "get_withdraw_permissions_by_recipient";
    type Response = Vec<WithdrawPermissionObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.account_name_or_id),
            serde_json::json!(self.start),
            serde_json::json!(self.limit),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetWitnessByAccountParams {
    /// The name or ID of the account whose witness should be retrieved
    pub account_name_or_id: String,
}

impl OpenRpcParams for GetWitnessByAccountParams {
    const METHOD: &'static str = "get_witness_by_account";
    type Response = Option<WitnessObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.account_name_or_id),
        ]
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GetWitnessCountParams;

impl OpenRpcParams for GetWitnessCountParams {
    const METHOD: &'static str = "get_witness_count";
    type Response = ::graphene_rpc::GrapheneUInt64;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        Vec::new()
    }
}

/// This function has semantics identical to get_objects, but doesn't subscribe
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetWitnessesParams {
    /// IDs of the witnesses to retrieve
    pub witness_ids: Vec<String>,
}

impl OpenRpcParams for GetWitnessesParams {
    const METHOD: &'static str = "get_witnesses";
    type Response = Vec<Option<WitnessObject>>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.witness_ids),
        ]
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GetWorkerCountParams;

impl OpenRpcParams for GetWorkerCountParams {
    const METHOD: &'static str = "get_worker_count";
    type Response = ::graphene_rpc::GrapheneUInt64;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        Vec::new()
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GetWorkersByAccountParams {
    /// The name or ID of the account whose worker should be retrieved
    pub account_name_or_id: String,
}

impl OpenRpcParams for GetWorkersByAccountParams {
    const METHOD: &'static str = "get_workers_by_account";
    type Response = Vec<WorkerObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.account_name_or_id),
        ]
    }
}

/// Determine whether a textual representation of a public key (in Base-58 format) is currently linked to any registered (i.e. non-stealth) account on the blockchain
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct IsPublicKeyRegisteredParams {
    /// Public key
    pub public_key: String,
}

impl OpenRpcParams for IsPublicKeyRegisteredParams {
    const METHOD: &'static str = "is_public_key_registered";
    type Response = bool;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.public_key),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ListAssetsParams {
    /// Lower bound of symbol names to retrieve
    pub lower_bound_symbol: String,
    /// Maximum number of assets to fetch, must not exceed the configured value of api_limit_get_assets
    pub limit: u32,
}

impl OpenRpcParams for ListAssetsParams {
    const METHOD: &'static str = "list_assets";
    type Response = Vec<ExtendedAssetObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.lower_bound_symbol),
            serde_json::json!(self.limit),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ListCreditDealsParams {
    /// The limitation of items each query can fetch, not greater than the configured value of api_limit_get_credit_offers
    pub limit: Option<u32>,
    /// Start credit deal id, fetch items whose IDs are greater than or equal to this ID
    pub start_id: Option<String>,
}

impl OpenRpcParams for ListCreditDealsParams {
    const METHOD: &'static str = "list_credit_deals";
    type Response = Vec<CreditDealObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.limit),
            serde_json::json!(self.start_id),
        ]
    }
}

/// Credit offers and credit deals
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ListCreditOffersParams {
    /// The limitation of items each query can fetch, not greater than the configured value of api_limit_get_credit_offers
    pub limit: Option<u32>,
    /// Start credit offer id, fetch items whose IDs are greater than or equal to this ID
    pub start_id: Option<String>,
}

impl OpenRpcParams for ListCreditOffersParams {
    const METHOD: &'static str = "list_credit_offers";
    type Response = Vec<CreditOfferObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.limit),
            serde_json::json!(self.start_id),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ListHtlcsParams {
    /// Lower bound of htlc id to start getting results
    pub start: String,
    /// Maximum number of htlc objects to fetch, must not exceed the configured value of api_limit_list_htlcs
    pub limit: u32,
}

impl OpenRpcParams for ListHtlcsParams {
    const METHOD: &'static str = "list_htlcs";
    type Response = Vec<HtlcObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.start),
            serde_json::json!(self.limit),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ListLiquidityPoolsParams {
    /// The limitation of items each query can fetch, not greater than the configured value of api_limit_get_liquidity_pools
    pub limit: Option<u32>,
    /// Start liquidity pool id, fetch pools whose IDs are greater than or equal to this ID
    pub start_id: Option<String>,
    /// Whether to return statistics
    pub with_statistics: Option<bool>,
}

impl OpenRpcParams for ListLiquidityPoolsParams {
    const METHOD: &'static str = "list_liquidity_pools";
    type Response = Vec<ExtendedLiquidityPoolObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.limit),
            serde_json::json!(self.start_id),
            serde_json::json!(self.with_statistics),
        ]
    }
}

/// SameT Funds
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ListSametFundsParams {
    /// The limitation of items each query can fetch, not greater than the configured value of api_limit_get_samet_funds
    pub limit: Option<u32>,
    /// Start SameT Fund id, fetch items whose IDs are greater than or equal to this ID
    pub start_id: Option<String>,
}

impl OpenRpcParams for ListSametFundsParams {
    const METHOD: &'static str = "list_samet_funds";
    type Response = Vec<SametFundObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.limit),
            serde_json::json!(self.start_id),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ListTicketsParams {
    /// The limitation of items each query can fetch, not greater than the configured value of api_limit_get_tickets
    pub limit: Option<u32>,
    /// Start ticket id, fetch tickets whose IDs are greater than or equal to this ID
    pub start_id: Option<String>,
}

impl OpenRpcParams for ListTicketsParams {
    const METHOD: &'static str = "list_tickets";
    type Response = Vec<TicketObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.limit),
            serde_json::json!(self.start_id),
        ]
    }
}

/// This function has semantics identical to get_objects, but doesn't subscribe.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LookupAccountNamesParams {
    /// Names of the accounts to retrieve
    pub account_names: Vec<String>,
}

impl OpenRpcParams for LookupAccountNamesParams {
    const METHOD: &'static str = "lookup_account_names";
    type Response = Vec<Option<AccountObject>>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.account_names),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LookupAccountsParams {
    /// Lower bound of the first name to return
    pub lower_bound_name: String,
    /// Maximum number of results to return, must not exceed the configured value of api_limit_lookup_accounts
    pub limit: u32,
    /// true to subscribe to the queried account objects, false to not subscribe, null to subscribe or not subscribe according to current auto-subscription setting (see set_auto_subscription)
    pub subscribe: Option<bool>,
}

impl OpenRpcParams for LookupAccountsParams {
    const METHOD: &'static str = "lookup_accounts";
    type Response = Vec<(String, String)>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.lower_bound_name),
            serde_json::json!(self.limit),
            serde_json::json!(self.subscribe),
        ]
    }
}

/// This function has semantics identical to get_objects, but doesn't subscribe
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LookupAssetSymbolsParams {
    /// symbol names or IDs of the assets to retrieve
    pub symbols_or_ids: Vec<String>,
}

impl OpenRpcParams for LookupAssetSymbolsParams {
    const METHOD: &'static str = "lookup_asset_symbols";
    type Response = Vec<Option<ExtendedAssetObject>>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.symbols_or_ids),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LookupCommitteeMemberAccountsParams {
    /// Lower bound of the first name to return
    pub lower_bound_name: String,
    /// Maximum number of results to return, must not exceed the configured value of api_limit_lookup_committee_member_accounts
    pub limit: u32,
}

impl OpenRpcParams for LookupCommitteeMemberAccountsParams {
    const METHOD: &'static str = "lookup_committee_member_accounts";
    type Response = Vec<(String, String)>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.lower_bound_name),
            serde_json::json!(self.limit),
        ]
    }
}

/// This will be a mixture of committee_member_objects, witness_objects, and worker_objects
///
/// The results will be in the same order as the votes. Null will be returned for any vote IDs that are not found.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LookupVoteIdsParams {
    /// a list of vote IDs, the quantity should not be greater than the configured value of api_limit_lookup_vote_ids
    pub votes: Vec<String>,
}

impl OpenRpcParams for LookupVoteIdsParams {
    const METHOD: &'static str = "lookup_vote_ids";
    type Response = Vec<LookupVoteIdObject>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.votes),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LookupWitnessAccountsParams {
    /// Lower bound of the first name to return
    pub lower_bound_name: String,
    /// Maximum number of results to return, must not exceed the configured value of api_limit_lookup_witness_accounts
    pub limit: u32,
}

impl OpenRpcParams for LookupWitnessAccountsParams {
    const METHOD: &'static str = "lookup_witness_accounts";
    type Response = Vec<(String, String)>;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.lower_bound_name),
            serde_json::json!(self.limit),
        ]
    }
}

/// Impacts behavior of these APIs: get_accounts get_assets get_objects lookup_accounts get_full_accounts get_htlc get_liquidity_pools get_liquidity_pools_by_share_asset
///
/// Note: auto-subscription is enabled by default
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SetAutoSubscriptionParams {
    /// whether follow-up API queries will automatically subscribe to queried objects
    pub enable: bool,
}

impl OpenRpcParams for SetAutoSubscriptionParams {
    const METHOD: &'static str = "set_auto_subscription";
    type Response = ();

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.enable),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct UnsubscribeFromMarketParams {
    /// symbol name or ID of the first asset
    pub a: String,
    /// symbol name or ID of the second asset
    pub b: String,
}

impl OpenRpcParams for UnsubscribeFromMarketParams {
    const METHOD: &'static str = "unsubscribe_from_market";
    type Response = ();

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.a),
            serde_json::json!(self.b),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ValidateTransactionParams {
    /// a transaction to be validated
    pub trx: SignedTransaction,
}

impl OpenRpcParams for ValidateTransactionParams {
    const METHOD: &'static str = "validate_transaction";
    type Response = ProcessedTransaction;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.trx),
        ]
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct VerifyAccountAuthorityParams {
    /// name or ID of an account to check
    pub account_name_or_id: String,
    /// the public keys
    pub signers: Vec<String>,
}

impl OpenRpcParams for VerifyAccountAuthorityParams {
    const METHOD: &'static str = "verify_account_authority";
    type Response = bool;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.account_name_or_id),
            serde_json::json!(self.signers),
        ]
    }
}

/// Check whether a transaction has all of the required signatures
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct VerifyAuthorityParams {
    /// a transaction to be verified
    pub trx: SignedTransaction,
}

impl OpenRpcParams for VerifyAuthorityParams {
    const METHOD: &'static str = "verify_authority";
    type Response = bool;

    fn into_positional_params(self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!(self.trx),
        ]
    }
}
