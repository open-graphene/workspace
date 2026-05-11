/// Graphene asset amount paired with a typed asset id.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct Asset<AssetId> {
    pub amount: i64,
    pub asset_id: AssetId,
}
