use std::fmt;

pub mod block_id {
    use super::fmt;

    pub fn ref_block_prefix(block_id: &str) -> Result<u32, BlockIdError> {
        let bytes = hex_to_bytes(block_id)?;
        if bytes.len() < 8 {
            return Err(BlockIdError {
                value: block_id.to_owned(),
                reason: format!("expected at least 8 bytes, got {}", bytes.len()),
            });
        }
        Ok(u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]))
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct BlockIdError {
        pub value: String,
        pub reason: String,
    }

    impl fmt::Display for BlockIdError {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                formatter,
                "invalid block id {:?}: {}",
                self.value, self.reason
            )
        }
    }

    impl std::error::Error for BlockIdError {}

    fn hex_to_bytes(value: &str) -> Result<Vec<u8>, BlockIdError> {
        if value.len() % 2 != 0 {
            return Err(BlockIdError {
                value: value.to_owned(),
                reason: "hex string has odd length".to_owned(),
            });
        }
        value
            .as_bytes()
            .chunks_exact(2)
            .map(|chunk| Ok((hex_nibble(chunk[0], value)? << 4) | hex_nibble(chunk[1], value)?))
            .collect()
    }

    fn hex_nibble(byte: u8, value: &str) -> Result<u8, BlockIdError> {
        match byte {
            b'0'..=b'9' => Ok(byte - b'0'),
            b'a'..=b'f' => Ok(byte - b'a' + 10),
            b'A'..=b'F' => Ok(byte - b'A' + 10),
            _ => Err(BlockIdError {
                value: value.to_owned(),
                reason: format!("invalid hex byte 0x{byte:02x}"),
            }),
        }
    }
}

pub mod broadcast {
    use super::fmt;
    use graphene_rpc::RpcError;

    #[derive(Clone, Debug, PartialEq)]
    pub struct SynchronousBroadcastResult {
        pub id: String,
        pub block_num: u32,
        pub trx_num: u32,
        pub raw: serde_json::Value,
    }

    impl TryFrom<serde_json::Value> for SynchronousBroadcastResult {
        type Error = BroadcastResultError;

        fn try_from(raw: serde_json::Value) -> Result<Self, Self::Error> {
            let id = raw
                .get("id")
                .and_then(serde_json::Value::as_str)
                .ok_or(BroadcastResultError::MissingField("id"))?
                .to_owned();
            let block_num = u64_to_u32_field(
                raw.get("block_num")
                    .and_then(serde_json::Value::as_u64)
                    .ok_or(BroadcastResultError::MissingField("block_num"))?,
                "block_num",
            )?;
            let trx_num = u64_to_u32_field(
                raw.get("trx_num")
                    .and_then(serde_json::Value::as_u64)
                    .ok_or(BroadcastResultError::MissingField("trx_num"))?,
                "trx_num",
            )?;

            Ok(Self {
                id,
                block_num,
                trx_num,
                raw,
            })
        }
    }

    fn u64_to_u32_field(value: u64, field: &'static str) -> Result<u32, BroadcastResultError> {
        u32::try_from(value).map_err(|_| BroadcastResultError::FieldOutOfRange { field, value })
    }

    #[derive(Debug)]
    pub enum BroadcastResultError {
        Rpc(RpcError),
        MissingField(&'static str),
        FieldOutOfRange { field: &'static str, value: u64 },
    }

    impl fmt::Display for BroadcastResultError {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Rpc(error) => write!(
                    formatter,
                    "RPC error while broadcasting transaction: {error}"
                ),
                Self::MissingField(field) => {
                    write!(
                        formatter,
                        "synchronous broadcast result is missing field {field}"
                    )
                }
                Self::FieldOutOfRange { field, value } => write!(
                    formatter,
                    "synchronous broadcast result field {field} is out of u32 range: {value}"
                ),
            }
        }
    }

    impl std::error::Error for BroadcastResultError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            match self {
                Self::Rpc(error) => Some(error),
                Self::MissingField(_) | Self::FieldOutOfRange { .. } => None,
            }
        }
    }

    impl From<RpcError> for BroadcastResultError {
        fn from(error: RpcError) -> Self {
            Self::Rpc(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::block_id::ref_block_prefix;
    use super::broadcast::{BroadcastResultError, SynchronousBroadcastResult};

    #[test]
    fn ref_block_prefix_reads_bytes_four_through_seven_as_little_endian_u32() {
        let block_id = "00012345a1b2c3d4ffffffffffffffffffffffffffffffffffffffffffffffff";

        assert_eq!(ref_block_prefix(block_id).unwrap(), 0xd4c3b2a1);
    }

    #[test]
    fn ref_block_prefix_rejects_invalid_hex() {
        let error = ref_block_prefix("zz").expect_err("bad hex should fail");

        assert_eq!(error.value, "zz");
        assert!(error.reason.contains("invalid hex byte"));
    }

    #[test]
    fn synchronous_broadcast_result_parses_raw_shape() {
        let raw = serde_json::json!({
            "id": "abc123",
            "block_num": 99,
            "trx_num": 7,
            "trx": { "signatures": [] }
        });

        let result = SynchronousBroadcastResult::try_from(raw.clone()).unwrap();

        assert_eq!(result.id, "abc123");
        assert_eq!(result.block_num, 99);
        assert_eq!(result.trx_num, 7);
        assert_eq!(result.raw, raw);
    }

    #[test]
    fn synchronous_broadcast_result_rejects_out_of_range_block_num() {
        let raw = serde_json::json!({
            "id": "abc123",
            "block_num": u64::from(u32::MAX) + 1,
            "trx_num": 7
        });

        let error = SynchronousBroadcastResult::try_from(raw).unwrap_err();

        assert!(matches!(
            error,
            BroadcastResultError::FieldOutOfRange {
                field: "block_num",
                ..
            }
        ));
    }
}
