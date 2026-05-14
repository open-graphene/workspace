use graphene_rpc::{RpcClient, RpcTransport};
use graphene_transaction::account::exact_account_id_from_lookup;
pub use graphene_transaction::account::LookupAccountError;

use crate::generated::LookupAccountsParams;

pub fn lookup_exact_account_id<T>(
    client: &RpcClient<T>,
    account_name: &str,
) -> Result<String, LookupAccountError>
where
    T: RpcTransport,
{
    let accounts = client.call(LookupAccountsParams {
        lower_bound_name: account_name.to_owned(),
        limit: 1,
        subscribe: Some(false),
    })?;

    exact_account_id_from_lookup(account_name, accounts)
}
