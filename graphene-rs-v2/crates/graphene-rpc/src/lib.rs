mod error;
mod http;
mod jsonrpc;
mod params;
mod scalar;
mod ws;

pub use error::RpcError;
pub use http::HttpTransport;
pub use params::{OpenRpcCallbackParams, OpenRpcParams, RpcClient, RpcTransport};
pub use scalar::{GrapheneInt64, GrapheneTimePointSec, GrapheneUInt64};
pub use ws::{
    ApiHandle, CallbackHandle, CallbackSubscription, GrapheneNotice, GrapheneWebSocketApiTransport,
    GrapheneWebSocketSession, GrapheneWebSocketTransport,
};

#[cfg(test)]
pub(crate) use jsonrpc::{build_json_rpc_request, parse_json_rpc_response};
#[cfg(test)]
pub(crate) use ws::{build_graphene_ws_call_request, callback_params, parse_graphene_notice};

#[cfg(test)]
mod tests {
    use super::{
        build_graphene_ws_call_request, build_json_rpc_request, callback_params,
        parse_graphene_notice, parse_json_rpc_response, GrapheneInt64, GrapheneNotice,
        GrapheneTimePointSec, GrapheneUInt64, HttpTransport, OpenRpcParams, RpcClient, RpcError,
        RpcTransport,
    };
    use chrono::NaiveDate;
    use serde::Deserialize;
    use serde_json::{json, Value};
    use std::io::{BufRead, BufReader, Read, Write};
    use std::net::TcpListener;
    use std::thread::{self, JoinHandle};

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

    fn spawn_one_request_server(
        status_line: &str,
        response_body: &str,
    ) -> (String, JoinHandle<Value>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("test server should bind");
        let endpoint = format!("http://{}", listener.local_addr().unwrap());
        let status_line = status_line.to_owned();
        let response_body = response_body.to_owned();

        let handle = thread::spawn(move || {
            let (mut stream, _) = listener
                .accept()
                .expect("test server should accept one request");
            let mut reader = BufReader::new(stream.try_clone().expect("stream clone should work"));

            let mut request_line = String::new();
            reader
                .read_line(&mut request_line)
                .expect("request line should be readable");
            assert!(
                request_line.starts_with("POST / HTTP/1.1"),
                "unexpected request line: {request_line:?}"
            );

            let mut content_length = None;
            loop {
                let mut line = String::new();
                reader
                    .read_line(&mut line)
                    .expect("headers should be readable");
                if line == "\r\n" {
                    break;
                }
                let lower = line.to_ascii_lowercase();
                if let Some(value) = lower.strip_prefix("content-length:") {
                    content_length = Some(
                        value
                            .trim()
                            .parse::<usize>()
                            .expect("content-length should be numeric"),
                    );
                }
            }

            let content_length = content_length.expect("request should include content-length");
            let mut body = vec![0; content_length];
            reader
                .read_exact(&mut body)
                .expect("request body should be readable");
            let request_body = serde_json::from_slice::<Value>(&body)
                .expect("request body should be JSON-RPC JSON");

            let response = format!(
                "{status_line}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{response_body}",
                response_body.len()
            );
            stream
                .write_all(response.as_bytes())
                .expect("response should be writable");
            request_body
        });

        (endpoint, handle)
    }

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

    #[test]
    fn builds_graphene_websocket_call_envelope() {
        assert_eq!(
            build_graphene_ws_call_request(9, 2, "get_objects", vec![json!(["2.1.0"])]),
            json!({
                "id": 9,
                "method": "call",
                "params": [2, "get_objects", [["2.1.0"]]],
            })
        );
    }

    #[test]
    fn callback_params_prepend_local_callback_id() {
        assert_eq!(
            callback_params(77, vec![json!({ "trx": true })]),
            vec![json!(77), json!({ "trx": true })]
        );
    }

    #[test]
    fn parses_graphene_notice_shape() {
        let notice = parse_graphene_notice(&json!({
            "method": "notice",
            "params": [42, [{"id": "2.1.0"}]],
        }))
        .unwrap()
        .expect("notice should parse");

        assert_eq!(
            notice,
            GrapheneNotice {
                callback_id: 42,
                payload: json!([{ "id": "2.1.0" }]),
            }
        );
    }

    #[test]
    fn ignores_non_notice_messages_when_parsing_graphene_notice() {
        let notice = parse_graphene_notice(&json!({
            "id": 1,
            "method": "call",
            "result": true,
        }))
        .unwrap();

        assert!(notice.is_none());
    }

    #[test]
    fn rejects_malformed_graphene_notice() {
        let error = parse_graphene_notice(&json!({
            "method": "notice",
            "params": [],
        }))
        .unwrap_err();

        match error {
            RpcError::Protocol { method, message } => {
                assert_eq!(method, "notice");
                assert!(message.contains("callback id"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn builds_json_rpc_request_envelope() {
        assert_eq!(
            build_json_rpc_request(7, "about", vec![json!("x")]),
            json!({
                "jsonrpc": "2.0",
                "id": 7,
                "method": "about",
                "params": ["x"],
            })
        );
    }

    #[test]
    fn http_transport_posts_json_rpc_request_and_decodes_success() {
        let (endpoint, handle) = spawn_one_request_server(
            "HTTP/1.1 200 OK",
            r#"{"jsonrpc":"2.0","id":1,"result":{"ok":true}}"#,
        );
        let transport = HttpTransport::new(endpoint);

        let result = transport
            .call_raw("ping", vec![json!("payload")])
            .expect("HTTP transport should decode success response");

        assert_eq!(result, json!({ "ok": true }));
        assert_eq!(
            handle.join().expect("server thread should finish"),
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "ping",
                "params": ["payload"],
            })
        );
    }

    #[test]
    fn http_transport_surfaces_json_rpc_error_response() {
        let (endpoint, handle) = spawn_one_request_server(
            "HTTP/1.1 200 OK",
            r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32000,"message":"boom","data":{"detail":"x"}}}"#,
        );
        let transport = HttpTransport::new(endpoint);

        let error = transport.call_raw("explode", Vec::new()).unwrap_err();

        handle.join().expect("server thread should finish");
        match error {
            RpcError::JsonRpc {
                method,
                code,
                message,
                data,
            } => {
                assert_eq!(method, "explode");
                assert_eq!(code, -32000);
                assert_eq!(message, "boom");
                assert_eq!(data, Some(json!({ "detail": "x" })));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn http_transport_reports_malformed_json_body() {
        let (endpoint, handle) =
            spawn_one_request_server("HTTP/1.1 200 OK", r#"{"jsonrpc":"2.0","result":"#);
        let transport = HttpTransport::new(endpoint);

        let error = transport.call_raw("broken", Vec::new()).unwrap_err();

        handle.join().expect("server thread should finish");
        match error {
            RpcError::Http { method, message } => {
                assert_eq!(method, "broken");
                assert!(
                    message.contains("EOF") || message.contains("expected"),
                    "unexpected malformed JSON error: {message}"
                );
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn http_transport_reports_non_success_http_status() {
        let (endpoint, handle) = spawn_one_request_server(
            "HTTP/1.1 500 Internal Server Error",
            r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32000,"message":"server exploded"}}"#,
        );
        let transport = HttpTransport::new(endpoint);

        let error = transport.call_raw("server_error", Vec::new()).unwrap_err();

        handle.join().expect("server thread should finish");
        match error {
            RpcError::Http { method, message } => {
                assert_eq!(method, "server_error");
                assert!(
                    message.contains("500") && message.contains("server exploded"),
                    "unexpected HTTP status error: {message}"
                );
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn parses_json_rpc_success_response() {
        let result = parse_json_rpc_response(
            "about",
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": { "name": "swaplock" },
            }),
        )
        .unwrap();

        assert_eq!(result, json!({ "name": "swaplock" }));
    }

    #[test]
    fn parses_json_rpc_error_response() {
        let error = parse_json_rpc_response(
            "about",
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "error": {
                    "code": -32601,
                    "message": "method not found",
                    "data": { "method": "about" },
                },
            }),
        )
        .unwrap_err();

        match error {
            RpcError::JsonRpc {
                method,
                code,
                message,
                data,
            } => {
                assert_eq!(method, "about");
                assert_eq!(code, -32601);
                assert_eq!(message, "method not found");
                assert_eq!(data, Some(json!({ "method": "about" })));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn graphene_time_point_sec_deserializes_graphene_wire_format() {
        let value: GrapheneTimePointSec = serde_json::from_value(json!("2026-05-13T14:44:57"))
            .expect("Graphene timestamp should parse without timezone suffix");

        let expected = NaiveDate::from_ymd_opt(2026, 5, 13)
            .unwrap()
            .and_hms_opt(14, 44, 57)
            .unwrap();
        assert_eq!(value.naive_utc(), expected);
        assert_eq!(
            serde_json::to_value(value).unwrap(),
            json!("2026-05-13T14:44:57")
        );
    }

    #[test]
    fn graphene_uint64_deserializes_number_and_decimal_string() {
        let from_number: GrapheneUInt64 = serde_json::from_value(json!(50_000_000_000_u64))
            .expect("Graphene uint64 should parse from JSON number");
        let from_string: GrapheneUInt64 = serde_json::from_value(json!("50000000000"))
            .expect("Graphene uint64 should parse from decimal string");

        assert_eq!(from_number.as_u64(), 50_000_000_000);
        assert_eq!(from_string.as_u64(), 50_000_000_000);
        assert_eq!(
            serde_json::to_value(from_string).unwrap(),
            json!(50_000_000_000_u64)
        );
    }

    #[test]
    fn graphene_int64_deserializes_number_and_decimal_string() {
        let from_number: GrapheneInt64 = serde_json::from_value(json!(-50_000_000_000_i64))
            .expect("Graphene int64 should parse from JSON number");
        let from_string: GrapheneInt64 = serde_json::from_value(json!("1000000000000000"))
            .expect("Graphene int64 should parse from decimal string");

        assert_eq!(from_number.as_i64(), -50_000_000_000);
        assert_eq!(from_string.as_i64(), 1_000_000_000_000_000);
        assert_eq!(
            serde_json::to_value(from_string).unwrap(),
            json!(1_000_000_000_000_000_i64)
        );
    }

    #[test]
    fn rejects_malformed_json_rpc_response() {
        let error =
            parse_json_rpc_response("about", json!({ "jsonrpc": "2.0", "id": 1 })).unwrap_err();

        match error {
            RpcError::Protocol { method, message } => {
                assert_eq!(method, "about");
                assert!(message.contains("missing result"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
