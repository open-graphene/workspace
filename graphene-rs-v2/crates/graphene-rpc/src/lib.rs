use std::error::Error;
use std::fmt;

/// Generated OpenRPC parameter structs implement this trait.
///
/// The trait is hand-written runtime API. Chain crates generate only the
/// method-specific structs and impls.
pub trait OpenRpcParams {
    const METHOD: &'static str;
    type Response;

    fn into_positional_params(self) -> Vec<serde_json::Value>;
}

/// Minimal transport abstraction for typed JSON-RPC calls.
///
/// Concrete HTTP/WebSocket transports can be added later without changing the
/// generated chain bindings.
pub trait RpcTransport {
    fn call_raw(
        &self,
        method: &str,
        params: Vec<serde_json::Value>,
    ) -> Result<serde_json::Value, RpcError>;
}

#[derive(Debug)]
pub struct RpcClient<T> {
    transport: T,
}

impl<T> RpcClient<T> {
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    pub fn transport(&self) -> &T {
        &self.transport
    }

    pub fn into_transport(self) -> T {
        self.transport
    }
}

impl<T> RpcClient<T>
where
    T: RpcTransport,
{
    pub fn call<P>(&self, params: P) -> Result<P::Response, RpcError>
    where
        P: OpenRpcParams,
        P::Response: serde::de::DeserializeOwned,
    {
        let raw = self
            .transport
            .call_raw(P::METHOD, params.into_positional_params())?;
        serde_json::from_value(raw).map_err(|source| RpcError::decode(P::METHOD, source))
    }
}

#[derive(Debug)]
pub enum RpcError {
    Transport {
        message: String,
    },
    Decode {
        method: &'static str,
        source: serde_json::Error,
    },
}

impl RpcError {
    pub fn transport(message: impl Into<String>) -> Self {
        Self::Transport {
            message: message.into(),
        }
    }

    pub fn decode(method: &'static str, source: serde_json::Error) -> Self {
        Self::Decode { method, source }
    }
}

impl fmt::Display for RpcError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transport { message } => write!(formatter, "RPC transport error: {message}"),
            Self::Decode { method, source } => {
                write!(
                    formatter,
                    "failed to decode RPC response for {method}: {source}"
                )
            }
        }
    }
}

impl Error for RpcError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Transport { .. } => None,
            Self::Decode { source, .. } => Some(source),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{OpenRpcParams, RpcClient, RpcError, RpcTransport};
    use serde::Deserialize;
    use serde_json::{json, Value};

    #[derive(Debug, Deserialize, PartialEq)]
    struct Pong {
        ok: bool,
    }

    struct PingParams;

    impl OpenRpcParams for PingParams {
        const METHOD: &'static str = "ping";
        type Response = Pong;

        fn into_positional_params(self) -> Vec<Value> {
            vec![json!("payload")]
        }
    }

    struct MockTransport;

    impl RpcTransport for MockTransport {
        fn call_raw(&self, method: &str, params: Vec<Value>) -> Result<Value, RpcError> {
            assert_eq!(method, "ping");
            assert_eq!(params, vec![json!("payload")]);
            Ok(json!({ "ok": true }))
        }
    }

    #[test]
    fn typed_call_decodes_response() {
        let client = RpcClient::new(MockTransport);
        let response = client.call(PingParams).unwrap();
        assert_eq!(response, Pong { ok: true });
    }
}
