use std::error::Error;
use std::fmt;

use serde_json::Value;

use crate::{
    ApiHandle, CallbackSubscription, GrapheneWebSocketSession, OpenRpcCallbackParams, RpcError,
};

#[derive(Clone, Debug, PartialEq)]
pub struct BlockAppliedNotice {
    pub block_id: String,
    pub raw: Value,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BlockAppliedNoticeError {
    UnexpectedShape(Value),
}

impl fmt::Display for BlockAppliedNoticeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedShape(value) => write!(
                formatter,
                "expected block applied callback payload to contain a block id string, got {value}"
            ),
        }
    }
}

impl Error for BlockAppliedNoticeError {}

impl TryFrom<Value> for BlockAppliedNotice {
    type Error = BlockAppliedNoticeError;

    fn try_from(raw: Value) -> Result<Self, Self::Error> {
        let block_id = raw
            .as_array()
            .and_then(|values| values.first())
            .and_then(Value::as_str)
            .or_else(|| raw.as_str())
            .ok_or_else(|| BlockAppliedNoticeError::UnexpectedShape(raw.clone()))?
            .to_owned();

        Ok(Self { block_id, raw })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SetBlockAppliedCallbackParams;

impl OpenRpcCallbackParams for SetBlockAppliedCallbackParams {
    const METHOD: &'static str = "set_block_applied_callback";
    type Response = ();
    type Callback = BlockAppliedNotice;

    fn into_positional_params_after_callback(self) -> Vec<Value> {
        Vec::new()
    }

    fn decode_response(value: Value) -> Result<Self::Response, RpcError> {
        if value.is_null() {
            Ok(())
        } else {
            Err(RpcError::protocol(
                Self::METHOD,
                format!("expected null acknowledgement, got {value}"),
            ))
        }
    }

    fn decode_callback(value: Value) -> Result<Self::Callback, RpcError> {
        BlockAppliedNotice::try_from(value)
            .map_err(|source| RpcError::protocol(Self::METHOD, source.to_string()))
    }
}

pub fn set_block_applied_callback<F>(
    session: &GrapheneWebSocketSession,
    database_api: &ApiHandle,
    callback: F,
) -> Result<CallbackSubscription, RpcError>
where
    F: FnMut(BlockAppliedNotice) + Send + 'static,
{
    let (subscription, ()) =
        session.subscribe(database_api, SetBlockAppliedCallbackParams, callback)?;
    Ok(subscription)
}

#[cfg(test)]
mod tests {
    use super::{BlockAppliedNotice, BlockAppliedNoticeError, SetBlockAppliedCallbackParams};
    use crate::OpenRpcCallbackParams;
    use serde_json::json;

    #[test]
    fn block_applied_notice_decodes_array_payload() {
        let raw = json!(["000555a5d6db937df830ed2d8449d4f1706269de"]);
        let notice = BlockAppliedNotice::try_from(raw.clone()).unwrap();

        assert_eq!(notice.block_id, "000555a5d6db937df830ed2d8449d4f1706269de");
        assert_eq!(notice.raw, raw);
    }

    #[test]
    fn block_applied_notice_decodes_direct_string_payload() {
        let raw = json!("000555a5d6db937df830ed2d8449d4f1706269de");
        let notice = BlockAppliedNotice::try_from(raw.clone()).unwrap();

        assert_eq!(notice.block_id, "000555a5d6db937df830ed2d8449d4f1706269de");
        assert_eq!(notice.raw, raw);
    }

    #[test]
    fn block_applied_notice_rejects_unexpected_payload() {
        let raw = json!({ "block_id": "000555a5d6db937df830ed2d8449d4f1706269de" });
        let error = BlockAppliedNotice::try_from(raw.clone()).unwrap_err();

        assert_eq!(error, BlockAppliedNoticeError::UnexpectedShape(raw));
    }

    #[test]
    fn set_block_applied_callback_params_decode_null_ack() {
        assert_eq!(
            SetBlockAppliedCallbackParams::decode_response(json!(null)).unwrap(),
            ()
        );
    }

    #[test]
    fn set_block_applied_callback_params_reject_non_null_ack() {
        let error = SetBlockAppliedCallbackParams::decode_response(json!(true)).unwrap_err();

        assert!(error.to_string().contains("expected null acknowledgement"));
    }
}
