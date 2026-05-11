/// Linear vesting policy used by legacy balance objects.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct LinearVestingPolicy {
    pub begin_timestamp: String,
    pub vesting_cliff_seconds: u32,
    pub vesting_duration_seconds: u32,
    pub begin_balance: i64,
}
