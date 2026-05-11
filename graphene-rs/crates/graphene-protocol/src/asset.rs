/// Graphene asset amount paired with a typed asset id.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct Asset<AssetId> {
    #[serde(deserialize_with = "crate::i64_from_number_or_string")]
    pub amount: i64,
    pub asset_id: AssetId,
}
