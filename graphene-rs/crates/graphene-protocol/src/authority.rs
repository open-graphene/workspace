/// Graphene weighted authority structure.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct Authority<AccountId> {
    pub weight_threshold: u32,
    pub account_auths: Vec<(AccountId, u16)>,
    pub key_auths: Vec<(String, u16)>,
    pub address_auths: Vec<(String, u16)>,
}
