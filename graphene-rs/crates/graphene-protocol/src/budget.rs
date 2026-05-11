/// Chain maintenance budget accounting snapshot.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct BudgetRecord {
    #[serde(deserialize_with = "crate::u64_from_number_or_string")]
    pub time_since_last_budget: u64,
    #[serde(deserialize_with = "crate::i64_from_number_or_string")]
    pub from_initial_reserve: i64,
    #[serde(deserialize_with = "crate::i64_from_number_or_string")]
    pub from_accumulated_fees: i64,
    #[serde(deserialize_with = "crate::i64_from_number_or_string")]
    pub from_unused_witness_budget: i64,
    #[serde(deserialize_with = "crate::i64_from_number_or_string")]
    pub requested_witness_budget: i64,
    #[serde(deserialize_with = "crate::i64_from_number_or_string")]
    pub total_budget: i64,
    #[serde(deserialize_with = "crate::i64_from_number_or_string")]
    pub witness_budget: i64,
    #[serde(deserialize_with = "crate::i64_from_number_or_string")]
    pub worker_budget: i64,
    #[serde(deserialize_with = "crate::i64_from_number_or_string")]
    pub leftover_worker_funds: i64,
    #[serde(deserialize_with = "crate::i64_from_number_or_string")]
    pub supply_delta: i64,
    #[serde(deserialize_with = "crate::i64_from_number_or_string")]
    pub max_supply: i64,
    #[serde(deserialize_with = "crate::i64_from_number_or_string")]
    pub current_supply: i64,
}
