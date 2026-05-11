use crate::Asset;

/// Graphene price represented as a base/quote asset pair.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct Price<AssetId> {
    pub base: Asset<AssetId>,
    pub quote: Asset<AssetId>,
}
