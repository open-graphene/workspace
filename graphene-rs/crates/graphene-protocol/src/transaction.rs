use crate::RestrictionArgument;

/// Raw Graphene operation static-variant JSON payload.
///
/// BitShares currently has dozens of operation variants. This wrapper preserves the exact JSON
/// shape while leaving room to replace it with a typed enum once operation modeling is tackled.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
#[serde(transparent)]
pub struct Operation(pub RestrictionArgument);

/// Raw Graphene operation-result static-variant JSON payload.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
#[serde(transparent)]
pub struct OperationResult(pub RestrictionArgument);

/// Raw Graphene transaction JSON payload.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
#[serde(transparent)]
pub struct Transaction(pub RestrictionArgument);

/// Raw Graphene signed-transaction JSON payload.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
#[serde(transparent)]
pub struct SignedTransaction(pub RestrictionArgument);
