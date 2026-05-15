use std::error::Error;
use std::fmt;

use serde_json::Value;

#[derive(Debug)]
pub enum RpcError {
    Transport {
        message: String,
    },
    Http {
        method: String,
        message: String,
    },
    JsonRpc {
        method: String,
        code: i64,
        message: String,
        data: Option<Value>,
    },
    Protocol {
        method: String,
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

    pub fn http(method: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Http {
            method: method.into(),
            message: message.into(),
        }
    }

    pub fn json_rpc(
        method: impl Into<String>,
        code: i64,
        message: impl Into<String>,
        data: Option<Value>,
    ) -> Self {
        Self::JsonRpc {
            method: method.into(),
            code,
            message: message.into(),
            data,
        }
    }

    pub fn protocol(method: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Protocol {
            method: method.into(),
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
            Self::Http { method, message } => {
                write!(formatter, "HTTP RPC error for {method}: {message}")
            }
            Self::JsonRpc {
                method,
                code,
                message,
                data,
            } => {
                if let Some(data) = data {
                    write!(
                        formatter,
                        "JSON-RPC error for {method}: {code} {message}; data={data}"
                    )
                } else {
                    write!(formatter, "JSON-RPC error for {method}: {code} {message}")
                }
            }
            Self::Protocol { method, message } => {
                write!(formatter, "JSON-RPC protocol error for {method}: {message}")
            }
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
            Self::Decode { source, .. } => Some(source),
            Self::Transport { .. }
            | Self::Http { .. }
            | Self::JsonRpc { .. }
            | Self::Protocol { .. } => None,
        }
    }
}
