use crate::RpcError;

/// Generated OpenRPC parameter structs implement this trait.
///
/// The trait is hand-written runtime API. Chain crates generate only the
/// method-specific structs and impls.
pub trait OpenRpcParams {
    const METHOD: &'static str;
    type Response;

    fn into_positional_params(self) -> Vec<serde_json::Value>;
}

/// Callback-aware Graphene RPC parameter structs implement this trait.
///
/// Graphene callback methods are not ordinary request/response calls: the first
/// wire parameter is a local callback id allocated by the WebSocket session.
/// Implementations return only the parameters that come after that callback id.
pub trait OpenRpcCallbackParams {
    const METHOD: &'static str;
    type Response;
    type Callback;

    fn into_positional_params_after_callback(self) -> Vec<serde_json::Value>;

    fn decode_response(value: serde_json::Value) -> Result<Self::Response, RpcError>;

    fn decode_callback(value: serde_json::Value) -> Result<Self::Callback, RpcError>;
}

/// Minimal transport abstraction for typed JSON-RPC calls.
///
/// Concrete HTTP/WebSocket transports can be added without changing the
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
