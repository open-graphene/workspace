use std::fmt;

pub mod block_id {
    use super::fmt;

    pub fn ref_block_prefix(block_id: &str) -> Result<u32, BlockIdError> {
        let bytes = hex_to_bytes(block_id)?;
        if bytes.len() < 8 {
            return Err(BlockIdError {
                value: block_id.to_owned(),
                reason: format!("expected at least 8 bytes, got {}", bytes.len()),
            });
        }
        Ok(u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]))
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct BlockIdError {
        pub value: String,
        pub reason: String,
    }

    impl fmt::Display for BlockIdError {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                formatter,
                "invalid block id {:?}: {}",
                self.value, self.reason
            )
        }
    }

    impl std::error::Error for BlockIdError {}

    fn hex_to_bytes(value: &str) -> Result<Vec<u8>, BlockIdError> {
        if value.len() % 2 != 0 {
            return Err(BlockIdError {
                value: value.to_owned(),
                reason: "hex string has odd length".to_owned(),
            });
        }
        value
            .as_bytes()
            .chunks_exact(2)
            .map(|chunk| Ok((hex_nibble(chunk[0], value)? << 4) | hex_nibble(chunk[1], value)?))
            .collect()
    }

    fn hex_nibble(byte: u8, value: &str) -> Result<u8, BlockIdError> {
        match byte {
            b'0'..=b'9' => Ok(byte - b'0'),
            b'a'..=b'f' => Ok(byte - b'a' + 10),
            b'A'..=b'F' => Ok(byte - b'A' + 10),
            _ => Err(BlockIdError {
                value: value.to_owned(),
                reason: format!("invalid hex byte 0x{byte:02x}"),
            }),
        }
    }
}

pub mod account {
    use super::fmt;
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
}

pub mod transaction {
    use graphene_rpc::GrapheneTimePointSec;

    use crate::block_id::{ref_block_prefix, BlockIdError};

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct TransactionHeaderFields {
        pub ref_block_num: u16,
        pub ref_block_prefix: u32,
        pub expiration: GrapheneTimePointSec,
    }

    pub fn compute_transaction_header_fields(
        head_block_id: &str,
        head_block_number: u32,
        expiration: GrapheneTimePointSec,
    ) -> Result<TransactionHeaderFields, BlockIdError> {
        Ok(TransactionHeaderFields {
            ref_block_num: (head_block_number & 0xffff) as u16,
            ref_block_prefix: ref_block_prefix(head_block_id)?,
            expiration,
        })
    }
}

pub mod transfer {
    use super::fmt;
    use crate::block_id::BlockIdError;
    use graphene_rpc::RpcError;

    #[derive(Clone, Debug)]
    pub struct TransferDraft {
        pub from: String,
        pub to: String,
        pub amount: i64,
        pub asset_id: String,
    }

    #[derive(Debug)]
    pub enum BuildTransactionError {
        InvalidObjectId {
            field: &'static str,
            value: String,
            source: String,
        },
        InvalidBlockId {
            value: String,
            reason: String,
        },
        MissingRequiredFee,
        UnsupportedFeeShape(String),
        Rpc(RpcError),
    }

    pub fn parse_object_id<T>(field: &'static str, value: &str) -> Result<T, BuildTransactionError>
    where
        for<'a> T: TryFrom<&'a str>,
        for<'a> <T as TryFrom<&'a str>>::Error: fmt::Display,
    {
        T::try_from(value).map_err(|source| BuildTransactionError::InvalidObjectId {
            field,
            value: value.to_owned(),
            source: source.to_string(),
        })
    }

    impl fmt::Display for BuildTransactionError {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::InvalidObjectId {
                    field,
                    value,
                    source,
                } => {
                    write!(formatter, "invalid {field} object id {value:?}: {source}")
                }
                Self::InvalidBlockId { value, reason } => {
                    write!(formatter, "invalid block id {value:?}: {reason}")
                }
                Self::Rpc(error) => {
                    write!(formatter, "RPC error while building transaction: {error}")
                }
                Self::MissingRequiredFee => write!(formatter, "required fee response was empty"),
                Self::UnsupportedFeeShape(shape) => {
                    write!(
                        formatter,
                        "unsupported required fee response shape: {shape}"
                    )
                }
            }
        }
    }

    impl std::error::Error for BuildTransactionError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            match self {
                Self::Rpc(error) => Some(error),
                Self::InvalidObjectId { .. }
                | Self::InvalidBlockId { .. }
                | Self::MissingRequiredFee
                | Self::UnsupportedFeeShape(_) => None,
            }
        }
    }

    impl From<BlockIdError> for BuildTransactionError {
        fn from(error: BlockIdError) -> Self {
            Self::InvalidBlockId {
                value: error.value,
                reason: error.reason,
            }
        }
    }

    impl From<RpcError> for BuildTransactionError {
        fn from(error: RpcError) -> Self {
            Self::Rpc(error)
        }
    }
}

pub mod broadcast {
    use super::fmt;
    use graphene_rpc::{OpenRpcCallbackParams, RpcError};

    #[derive(Clone, Debug)]
    pub struct BroadcastTransactionWithCallbackParams<TSigned> {
        pub trx: TSigned,
    }

    impl<TSigned> OpenRpcCallbackParams for BroadcastTransactionWithCallbackParams<TSigned>
    where
        TSigned: serde::Serialize,
    {
        const METHOD: &'static str = "broadcast_transaction_with_callback";
        type Response = ();
        type Callback = serde_json::Value;

        fn into_positional_params_after_callback(self) -> Vec<serde_json::Value> {
            vec![serde_json::json!(self.trx)]
        }

        fn decode_response(value: serde_json::Value) -> Result<Self::Response, RpcError> {
            if value.is_null() {
                Ok(())
            } else {
                Err(RpcError::protocol(
                    Self::METHOD,
                    format!("expected null acknowledgement, got {value}"),
                ))
            }
        }

        fn decode_callback(value: serde_json::Value) -> Result<Self::Callback, RpcError> {
            Ok(value)
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct SynchronousBroadcastResult {
        pub id: String,
        pub block_num: u32,
        pub trx_num: u32,
        pub raw: serde_json::Value,
    }

    impl TryFrom<serde_json::Value> for SynchronousBroadcastResult {
        type Error = BroadcastResultError;

        fn try_from(raw: serde_json::Value) -> Result<Self, Self::Error> {
            let result = broadcast_result_object(&raw)?;
            let id = result
                .get("id")
                .and_then(serde_json::Value::as_str)
                .ok_or(BroadcastResultError::MissingField("id"))?
                .to_owned();
            let block_num = u64_to_u32_field(
                result
                    .get("block_num")
                    .and_then(serde_json::Value::as_u64)
                    .ok_or(BroadcastResultError::MissingField("block_num"))?,
                "block_num",
            )?;
            let trx_num = u64_to_u32_field(
                result
                    .get("trx_num")
                    .and_then(serde_json::Value::as_u64)
                    .ok_or(BroadcastResultError::MissingField("trx_num"))?,
                "trx_num",
            )?;

            Ok(Self {
                id,
                block_num,
                trx_num,
                raw,
            })
        }
    }

    fn broadcast_result_object(
        raw: &serde_json::Value,
    ) -> Result<&serde_json::Map<String, serde_json::Value>, BroadcastResultError> {
        if let Some(object) = raw.as_object() {
            return Ok(object);
        }
        if let Some(values) = raw.as_array() {
            if values.len() == 1 {
                if let Some(object) = values[0].as_object() {
                    return Ok(object);
                }
            }
            return Err(BroadcastResultError::UnexpectedShape(
                "expected object or single callback-result object".to_owned(),
            ));
        }
        Err(BroadcastResultError::UnexpectedShape(
            "expected object broadcast result".to_owned(),
        ))
    }

    fn u64_to_u32_field(value: u64, field: &'static str) -> Result<u32, BroadcastResultError> {
        u32::try_from(value).map_err(|_| BroadcastResultError::FieldOutOfRange { field, value })
    }

    #[derive(Debug)]
    pub enum BroadcastResultError {
        Rpc(RpcError),
        MissingField(&'static str),
        UnexpectedShape(String),
        FieldOutOfRange { field: &'static str, value: u64 },
    }

    impl fmt::Display for BroadcastResultError {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Rpc(error) => write!(
                    formatter,
                    "RPC error while broadcasting transaction: {error}"
                ),
                Self::MissingField(field) => {
                    write!(
                        formatter,
                        "synchronous broadcast result is missing field {field}"
                    )
                }
                Self::UnexpectedShape(message) => {
                    write!(formatter, "unexpected broadcast result shape: {message}")
                }
                Self::FieldOutOfRange { field, value } => write!(
                    formatter,
                    "synchronous broadcast result field {field} is out of u32 range: {value}"
                ),
            }
        }
    }

    impl std::error::Error for BroadcastResultError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            match self {
                Self::Rpc(error) => Some(error),
                Self::MissingField(_) | Self::UnexpectedShape(_) | Self::FieldOutOfRange { .. } => {
                    None
                }
            }
        }
    }

    impl From<RpcError> for BroadcastResultError {
        fn from(error: RpcError) -> Self {
            Self::Rpc(error)
        }
    }
}

pub mod signing {
    use super::fmt;
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
}

#[cfg(test)]
mod tests {
    use super::account::{exact_account_id_from_lookup, LookupAccountError};
    use super::block_id::ref_block_prefix;
    use super::broadcast::{
        BroadcastResultError, BroadcastTransactionWithCallbackParams, SynchronousBroadcastResult,
    };
    use super::transaction::compute_transaction_header_fields;
    use graphene_rpc::OpenRpcCallbackParams;

    #[test]
    fn ref_block_prefix_reads_bytes_four_through_seven_as_little_endian_u32() {
        let block_id = "00012345a1b2c3d4ffffffffffffffffffffffffffffffffffffffffffffffff";

        assert_eq!(ref_block_prefix(block_id).unwrap(), 0xd4c3b2a1);
    }

    #[test]
    fn ref_block_prefix_rejects_invalid_hex() {
        let error = ref_block_prefix("zz").expect_err("bad hex should fail");

        assert_eq!(error.value, "zz");
        assert!(error.reason.contains("invalid hex byte"));
    }

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

    #[test]
    fn transaction_header_fields_derive_ref_block_values() {
        let block_id = "00012345a1b2c3d4ffffffffffffffffffffffffffffffffffffffffffffffff";
        let expiration = "2026-05-13T20:05:00".parse().expect("valid timestamp");

        let fields = compute_transaction_header_fields(block_id, 0x12345, expiration).unwrap();

        assert_eq!(fields.ref_block_num, 0x2345);
        assert_eq!(fields.ref_block_prefix, 0xd4c3b2a1);
        assert_eq!(fields.expiration, expiration);
    }

    #[test]
    fn broadcast_transaction_with_callback_params_accept_null_ack() {
        let response =
            BroadcastTransactionWithCallbackParams::<serde_json::Value>::decode_response(
                serde_json::Value::Null,
            )
            .unwrap();

        assert_eq!(response, ());
    }

    #[test]
    fn broadcast_transaction_with_callback_params_reject_non_null_ack() {
        let error = BroadcastTransactionWithCallbackParams::<serde_json::Value>::decode_response(
            serde_json::json!(true),
        )
        .unwrap_err();

        assert!(error.to_string().contains("expected null acknowledgement"));
    }

    #[test]
    fn broadcast_transaction_with_callback_params_pass_callback_payload_through() {
        let raw = serde_json::json!([{ "id": "abc123" }]);
        let decoded = BroadcastTransactionWithCallbackParams::<serde_json::Value>::decode_callback(
            raw.clone(),
        )
        .unwrap();

        assert_eq!(decoded, raw);
    }

    #[test]
    fn synchronous_broadcast_result_parses_raw_shape() {
        let raw = serde_json::json!({
            "id": "abc123",
            "block_num": 99,
            "trx_num": 7,
            "trx": { "signatures": [] }
        });

        let result = SynchronousBroadcastResult::try_from(raw.clone()).unwrap();

        assert_eq!(result.id, "abc123");
        assert_eq!(result.block_num, 99);
        assert_eq!(result.trx_num, 7);
        assert_eq!(result.raw, raw);
    }

    #[test]
    fn synchronous_broadcast_result_parses_callback_argument_shape() {
        let raw = serde_json::json!([{
            "id": "abc123",
            "block_num": 99,
            "trx_num": 7,
            "trx": { "signatures": [] }
        }]);

        let result = SynchronousBroadcastResult::try_from(raw.clone()).unwrap();

        assert_eq!(result.id, "abc123");
        assert_eq!(result.block_num, 99);
        assert_eq!(result.trx_num, 7);
        assert_eq!(result.raw, raw);
    }

    #[test]
    fn synchronous_broadcast_result_rejects_out_of_range_block_num() {
        let raw = serde_json::json!({
            "id": "abc123",
            "block_num": u64::from(u32::MAX) + 1,
            "trx_num": 7
        });

        let error = SynchronousBroadcastResult::try_from(raw).unwrap_err();

        assert!(matches!(
            error,
            BroadcastResultError::FieldOutOfRange {
                field: "block_num",
                ..
            }
        ));
    }
}
