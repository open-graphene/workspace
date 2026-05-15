use std::fmt;

use graphene_codec::{to_graphene_bytes, EncodeError, GrapheneEncode};
use graphene_signing::{signing_digest, ChainId, SignError, Signature, Signer};

#[derive(Clone, Debug)]
pub struct PreparedTransaction<TTransaction> {
    pub transaction: TTransaction,
}

impl<TTransaction> PreparedTransaction<TTransaction> {
    pub fn new(transaction: TTransaction) -> Self {
        Self { transaction }
    }

    pub fn into_transaction(self) -> TTransaction {
        self.transaction
    }

    pub fn sign<S: Signer>(
        self,
        chain_id: &ChainId,
        signer: &S,
    ) -> Result<SignedTransactionEnvelope<TTransaction>, SignTransactionError>
    where
        TTransaction: GrapheneEncode,
    {
        sign_transaction(chain_id, self.transaction, signer)
    }
}

#[derive(Clone, Debug)]
pub struct SignedTransactionEnvelope<TTransaction> {
    pub transaction: TTransaction,
    pub signatures: Vec<Signature>,
}

pub fn sign_transaction<T, S>(
    chain_id: &ChainId,
    transaction: T,
    signer: &S,
) -> Result<SignedTransactionEnvelope<T>, SignTransactionError>
where
    T: GrapheneEncode,
    S: Signer,
{
    let signatures = sign_transaction_bytes(chain_id, &transaction, signer)?;
    Ok(SignedTransactionEnvelope {
        transaction,
        signatures,
    })
}

pub fn sign_transaction_bytes<T, S>(
    chain_id: &ChainId,
    transaction: &T,
    signer: &S,
) -> Result<Vec<Signature>, SignTransactionError>
where
    T: GrapheneEncode,
    S: Signer,
{
    let transaction_bytes = to_graphene_bytes(transaction)?;
    let digest = signing_digest(chain_id, &transaction_bytes);
    let signature = signer.sign_digest(&digest)?;
    Ok(vec![signature])
}

#[derive(Debug)]
pub enum SignTransactionError {
    Encode(EncodeError),
    Sign(SignError),
}

impl fmt::Display for SignTransactionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Encode(error) => write!(formatter, "failed to encode transaction: {error}"),
            Self::Sign(error) => {
                write!(formatter, "failed to sign transaction digest: {error}")
            }
        }
    }
}

impl std::error::Error for SignTransactionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Encode(error) => Some(error),
            Self::Sign(error) => Some(error),
        }
    }
}

impl From<EncodeError> for SignTransactionError {
    fn from(error: EncodeError) -> Self {
        Self::Encode(error)
    }
}

impl From<SignError> for SignTransactionError {
    fn from(error: SignError) -> Self {
        Self::Sign(error)
    }
}
