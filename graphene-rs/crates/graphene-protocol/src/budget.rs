/// Chain maintenance budget accounting snapshot.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct BudgetRecord {
    pub time_since_last_budget: u64,
    pub from_initial_reserve: i64,
    pub from_accumulated_fees: i64,
    pub from_unused_witness_budget: i64,
    pub requested_witness_budget: i64,
    pub total_budget: i64,
    pub witness_budget: i64,
    pub worker_budget: i64,
    pub leftover_worker_funds: i64,
    pub supply_delta: i64,
    pub max_supply: i64,
    pub current_supply: i64,
}
