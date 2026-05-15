use serde_json::Value;

use crate::generated::{DynamicGlobalPropertyObject, GetObjectResult, GetObjectsParams};
use graphene_rpc::{
    ApiHandle, CallbackSubscription, GrapheneWebSocketSession, OpenRpcCallbackParams, RpcError,
};

const DYNAMIC_GLOBAL_PROPERTIES_ID: &str = "2.1.0";

/// Parameters for Graphene's object subscription callback registration.
///
/// The first wire parameter is the local callback id allocated by
/// [`GrapheneWebSocketSession`]. `notify_remove_create` maps to Graphene's
/// second `set_subscribe_callback` argument and should normally stay `false`
/// for object-specific subscriptions such as dynamic global properties.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SetSubscribeCallbackParams {
    pub notify_remove_create: bool,
}

impl OpenRpcCallbackParams for SetSubscribeCallbackParams {
    const METHOD: &'static str = "set_subscribe_callback";
    type Response = ();
    type Callback = DynamicGlobalPropertyObject;

    fn into_positional_params_after_callback(self) -> Vec<Value> {
        vec![serde_json::json!(self.notify_remove_create)]
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
        decode_dynamic_global_properties_notice(value)
    }
}

/// Subscribes to dynamic global properties (`2.1.0`) updates on a WebSocket database API.
///
/// The helper registers Graphene's object subscription callback, fetches the
/// initial `2.1.0` object with `subscribe=true`, and delivers later notices as
/// typed [`DynamicGlobalPropertyObject`] values. The returned
/// [`CallbackSubscription`] only removes local callback routing when
/// unsubscribed; it does not add reconnect or remote resubscribe semantics.
pub fn subscribe_dynamic_global_properties<F>(
    session: &GrapheneWebSocketSession,
    database_api: &ApiHandle,
    callback: F,
) -> Result<(CallbackSubscription, DynamicGlobalPropertyObject), RpcError>
where
    F: FnMut(DynamicGlobalPropertyObject) + Send + 'static,
{
    let (subscription, ()) = session.subscribe(
        database_api,
        SetSubscribeCallbackParams {
            notify_remove_create: false,
        },
        callback,
    )?;

    let initial = match session.call(
        database_api,
        GetObjectsParams {
            ids: vec![DYNAMIC_GLOBAL_PROPERTIES_ID.to_owned()],
            subscribe: Some(true),
        },
    ) {
        Ok(objects) => extract_dynamic_global_properties(objects),
        Err(error) => Err(error),
    };

    match initial {
        Ok(dynamic) => Ok((subscription, dynamic)),
        Err(error) => {
            let _ = subscription.unsubscribe();
            Err(error)
        }
    }
}

fn decode_dynamic_global_properties_notice(
    value: Value,
) -> Result<DynamicGlobalPropertyObject, RpcError> {
    let objects_value = match value.as_array() {
        Some(values) if values.len() == 1 && values[0].is_array() => values[0].clone(),
        Some(_) => value,
        None => {
            return Err(RpcError::protocol(
                SetSubscribeCallbackParams::METHOD,
                format!("expected object subscription notice array, got {value}"),
            ))
        }
    };

    let objects: Vec<GetObjectResult> = serde_json::from_value(objects_value)
        .map_err(|source| RpcError::decode(SetSubscribeCallbackParams::METHOD, source))?;
    extract_dynamic_global_properties(objects)
}

fn extract_dynamic_global_properties(
    objects: Vec<GetObjectResult>,
) -> Result<DynamicGlobalPropertyObject, RpcError> {
    objects
        .into_iter()
        .find_map(|object| match object {
            GetObjectResult::DynamicGlobalPropertyObject(dynamic) => Some(dynamic),
            _ => None,
        })
        .ok_or_else(|| {
            RpcError::protocol(
                SetSubscribeCallbackParams::METHOD,
                "dynamic global properties object 2.1.0 was not present in subscription payload",
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use graphene_rpc::OpenRpcCallbackParams;
    use serde_json::json;

    fn dynamic_json(head_block_number: u32) -> Value {
        json!({
            "id": "2.1.0",
            "accounts_registered_this_interval": 0,
            "current_aslot": 12345,
            "current_witness": "1.6.1",
            "dynamic_flags": 0,
            "head_block_id": "00012345a1b2c3d4ffffffffffffffffffffffffffffffffffffffffffffffff",
            "head_block_number": head_block_number,
            "last_budget_time": "2026-05-13T20:00:00",
            "last_irreversible_block_num": head_block_number.saturating_sub(1),
            "last_vote_tally_time": "2026-05-13T20:00:00",
            "next_maintenance_time": "2026-05-13T21:00:00",
            "recent_slots_filled": "340282366920938463463374607431768211455",
            "recently_missed_count": 0,
            "time": "2026-05-13T20:00:00",
            "total_inactive": 0,
            "total_pob": 0,
            "witness_budget": 0
        })
    }

    #[test]
    fn set_subscribe_callback_params_append_notify_remove_create_flag() {
        assert_eq!(
            SetSubscribeCallbackParams {
                notify_remove_create: false,
            }
            .into_positional_params_after_callback(),
            vec![json!(false)]
        );
    }

    #[test]
    fn set_subscribe_callback_params_decode_null_ack() {
        assert_eq!(
            SetSubscribeCallbackParams::decode_response(json!(null)).unwrap(),
            ()
        );
    }

    #[test]
    fn set_subscribe_callback_params_reject_non_null_ack() {
        let error = SetSubscribeCallbackParams::decode_response(json!(true)).unwrap_err();

        assert!(error.to_string().contains("expected null acknowledgement"));
    }

    #[test]
    fn dynamic_notice_decodes_single_argument_object_list_shape() {
        let dynamic =
            SetSubscribeCallbackParams::decode_callback(json!([[dynamic_json(42)]])).unwrap();

        assert_eq!(dynamic.head_block_number, 42);
    }

    #[test]
    fn dynamic_notice_decodes_direct_object_list_shape() {
        let dynamic =
            SetSubscribeCallbackParams::decode_callback(json!([dynamic_json(43)])).unwrap();

        assert_eq!(dynamic.head_block_number, 43);
    }

    #[test]
    fn dynamic_notice_rejects_missing_dynamic_object() {
        let error = SetSubscribeCallbackParams::decode_callback(json!([null])).unwrap_err();

        assert!(error.to_string().contains("2.1.0 was not present"));
    }
}
