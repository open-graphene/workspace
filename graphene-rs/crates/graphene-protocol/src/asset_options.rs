use crate::Price;

/// Optional BitShares asset settings carried in FC extension JSON.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct AdditionalAssetOptions<AccountId> {
    pub reward_percent: Option<u16>,
    pub whitelist_market_fee_sharing: Option<Vec<AccountId>>,
    pub taker_fee_percent: Option<u16>,
}

/// Options common to all Graphene assets.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct AssetOptions<AccountId, AssetId> {
    #[serde(deserialize_with = "crate::i64_from_number_or_string")]
    pub max_supply: i64,
    pub market_fee_percent: u16,
    #[serde(deserialize_with = "crate::i64_from_number_or_string")]
    pub max_market_fee: i64,
    pub issuer_permissions: u16,
    pub flags: u16,
    pub core_exchange_rate: Price<AssetId>,
    pub whitelist_authorities: Vec<AccountId>,
    pub blacklist_authorities: Vec<AccountId>,
    pub whitelist_markets: Vec<AssetId>,
    pub blacklist_markets: Vec<AssetId>,
    pub description: String,
    pub extensions: AdditionalAssetOptions<AccountId>,
}

/// Optional BitAsset settings carried in FC extension JSON.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct BitAssetOptionExtensions {
    pub initial_collateral_ratio: Option<u16>,
    pub maintenance_collateral_ratio: Option<u16>,
    pub maximum_short_squeeze_ratio: Option<u16>,
    pub margin_call_fee_ratio: Option<u16>,
    pub force_settle_fee_percent: Option<u16>,
    pub black_swan_response_method: Option<u8>,
}

/// Options specific to market-issued Graphene assets.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct BitAssetOptions<AssetId> {
    pub feed_lifetime_sec: u32,
    pub minimum_feeds: u8,
    pub force_settlement_delay_sec: u32,
    pub force_settlement_offset_percent: u16,
    pub maximum_force_settlement_volume: u16,
    pub short_backing_asset: AssetId,
    pub extensions: BitAssetOptionExtensions,
}

/// Market price feed values used by BitAssets.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct PriceFeed<AssetId> {
    pub settlement_price: Price<AssetId>,
    pub maintenance_collateral_ratio: u16,
    pub maximum_short_squeeze_ratio: u16,
    pub core_exchange_rate: Price<AssetId>,
}

/// BitShares price feed extended with BSIP77 initial collateral ratio.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct PriceFeedWithIcr<AssetId> {
    pub settlement_price: Price<AssetId>,
    pub maintenance_collateral_ratio: u16,
    pub maximum_short_squeeze_ratio: u16,
    pub core_exchange_rate: Price<AssetId>,
    pub initial_collateral_ratio: u16,
}
