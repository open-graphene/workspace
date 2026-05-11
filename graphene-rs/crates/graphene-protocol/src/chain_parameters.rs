/// Graphene genesis-time chain invariants.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct ImmutableChainParameters {
    pub min_committee_member_count: u16,
    pub min_witness_count: u16,
    pub num_special_accounts: u32,
    pub num_special_assets: u32,
}
