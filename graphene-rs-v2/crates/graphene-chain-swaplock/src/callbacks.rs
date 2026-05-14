use std::error::Error;
use std::fmt;

use graphene_rpc::{ApiHandle, CallbackSubscription, GrapheneWebSocketSession, RpcError};
use serde_json::Value;

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

pub fn set_block_applied_callback<F>(
    session: &GrapheneWebSocketSession,
    database_api: &ApiHandle,
    mut callback: F,
) -> Result<CallbackSubscription, RpcError>
where
    F: FnMut(BlockAppliedNotice) + Send + 'static,
{
    let (subscription, response) = session.call_with_callback_raw(
        database_api,
        "set_block_applied_callback",
        Vec::new(),
        move |payload| {
            if let Ok(notice) = BlockAppliedNotice::try_from(payload) {
                callback(notice);
            }
        },
    )?;

    if !response.is_null() {
        subscription.unsubscribe()?;
        return Err(RpcError::protocol(
            "set_block_applied_callback",
            format!("expected null acknowledgement, got {response}"),
        ));
    }

    Ok(subscription)
}

#[cfg(test)]
mod tests {
    use super::{BlockAppliedNotice, BlockAppliedNoticeError};
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
}
