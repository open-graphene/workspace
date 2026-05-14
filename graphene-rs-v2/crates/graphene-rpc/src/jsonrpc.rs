use serde_json::{json, Value};

use crate::RpcError;

pub(crate) fn build_json_rpc_request(id: u64, method: &str, params: Vec<Value>) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params,
    })
}

pub(crate) fn parse_json_rpc_response(method: &str, response: Value) -> Result<Value, RpcError> {
    let Some(object) = response.as_object() else {
        return Err(RpcError::protocol(
            method,
            "JSON-RPC response must be an object",
        ));
    };

    if let Some(error) = object.get("error") {
        let Some(error_object) = error.as_object() else {
            return Err(RpcError::protocol(
                method,
                "JSON-RPC error must be an object",
            ));
        };
        let code = error_object
            .get("code")
            .and_then(Value::as_i64)
            .ok_or_else(|| RpcError::protocol(method, "JSON-RPC error is missing numeric code"))?;
        let message = error_object
            .get("message")
            .and_then(Value::as_str)
            .ok_or_else(|| RpcError::protocol(method, "JSON-RPC error is missing message"))?
            .to_owned();
        return Err(RpcError::json_rpc(
            method,
            code,
            message,
            error_object.get("data").cloned(),
        ));
    }

    object
        .get("result")
        .cloned()
        .ok_or_else(|| RpcError::protocol(method, "JSON-RPC response is missing result"))
}
