use std::fmt;

use graphene_rpc::RpcError;

pub fn exact_account_id_from_lookup(
    account_name: &str,
    accounts: Vec<(String, String)>,
) -> Result<String, LookupAccountError> {
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
            Self::Rpc(error) => {
                write!(formatter, "RPC error while looking up account: {error}")
            }
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

#[cfg(test)]
mod tests {
    use super::{exact_account_id_from_lookup, LookupAccountError};

    #[test]
    fn exact_account_lookup_returns_matching_id() {
        let id =
            exact_account_id_from_lookup("alice", vec![("alice".to_owned(), "1.2.100".to_owned())])
                .unwrap();

        assert_eq!(id, "1.2.100");
    }

    #[test]
    fn exact_account_lookup_rejects_empty_response() {
        let error =
            exact_account_id_from_lookup("alice", vec![]).expect_err("empty response should fail");

        assert!(matches!(error, LookupAccountError::NotFound { .. }));
    }

    #[test]
    fn exact_account_lookup_rejects_lower_bound_mismatch() {
        let error = exact_account_id_from_lookup(
            "alice",
            vec![("alice2".to_owned(), "1.2.101".to_owned())],
        )
        .expect_err("lower-bound mismatch should fail");

        assert!(matches!(
            error,
            LookupAccountError::NameMismatch {
                expected,
                actual,
            } if expected == "alice" && actual == "alice2"
        ));
    }
}
