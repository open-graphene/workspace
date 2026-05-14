use std::fmt;

use graphene_rpc::{RpcClient, RpcError, RpcTransport};

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

    let Some((name, id)) = accounts.into_iter().next() else {
        return Err(LookupAccountError::NotFound {
            name: account_name.to_owned(),
        });
    };

    if name != account_name {
        return Err(LookupAccountError::NameMismatch {
            expected: account_name.to_owned(),
            actual: name,
        });
    }

    Ok(id)
}

#[derive(Debug)]
pub enum LookupAccountError {
    Rpc(RpcError),
    NotFound { name: String },
    NameMismatch { expected: String, actual: String },
}

impl fmt::Display for LookupAccountError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rpc(error) => write!(formatter, "RPC error while looking up account: {error}"),
            Self::NotFound { name } => write!(formatter, "account {name:?} was not found"),
            Self::NameMismatch { expected, actual } => write!(
                formatter,
                "expected account {expected:?}, but lookup returned {actual:?}"
            ),
        }
    }
}

impl std::error::Error for LookupAccountError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Rpc(error) => Some(error),
            Self::NotFound { .. } | Self::NameMismatch { .. } => None,
        }
    }
}

impl From<RpcError> for LookupAccountError {
    fn from(error: RpcError) -> Self {
        Self::Rpc(error)
    }
}
