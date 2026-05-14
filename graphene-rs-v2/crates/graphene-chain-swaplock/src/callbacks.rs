use graphene_rpc::{ApiHandle, CallbackSubscription, GrapheneWebSocketSession, RpcError};

pub use graphene_rpc::database_callbacks::{
    BlockAppliedNotice, BlockAppliedNoticeError, SetBlockAppliedCallbackParams,
};

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
