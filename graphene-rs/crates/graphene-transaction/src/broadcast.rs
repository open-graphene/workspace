use std::fmt;

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
            Self::MissingField(_) | Self::UnexpectedShape(_) | Self::FieldOutOfRange { .. } => None,
        }
    }
}

impl From<RpcError> for BroadcastResultError {
    fn from(error: RpcError) -> Self {
        Self::Rpc(error)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BroadcastResultError, BroadcastTransactionWithCallbackParams, SynchronousBroadcastResult,
    };
    use graphene_rpc::OpenRpcCallbackParams;

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
