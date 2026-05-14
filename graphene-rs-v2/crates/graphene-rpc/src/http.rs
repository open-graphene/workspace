use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::Value;

use crate::jsonrpc::{build_json_rpc_request, parse_json_rpc_response};
use crate::{RpcError, RpcTransport};

#[derive(Debug)]
pub struct HttpTransport {
    endpoint: String,
    next_id: AtomicU64,
}

impl HttpTransport {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            next_id: AtomicU64::new(1),
        }
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    fn next_request_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }
}

impl RpcTransport for HttpTransport {
    fn call_raw(&self, method: &str, params: Vec<Value>) -> Result<Value, RpcError> {
        let request = build_json_rpc_request(self.next_request_id(), method, params);
        let response = ureq::post(&self.endpoint)
            .set("content-type", "application/json")
            .send_json(request)
            .map_err(|source| http_error_with_body(method, source))?;
        let response = response
            .into_json::<Value>()
            .map_err(|source| RpcError::http(method, source.to_string()))?;
        parse_json_rpc_response(method, response)
    }
}

fn http_error_with_body(method: &str, source: ureq::Error) -> RpcError {
    match source {
        ureq::Error::Status(code, response) => {
            let body = response
                .into_string()
                .unwrap_or_else(|error| format!("<failed to read response body: {error}>"));
            RpcError::http(method, format!("status code {code}: {body}"))
        }
        other => RpcError::http(method, other.to_string()),
    }
}
