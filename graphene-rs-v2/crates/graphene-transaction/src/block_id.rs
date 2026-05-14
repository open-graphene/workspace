use std::fmt;

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

#[cfg(test)]
mod tests {
    use super::ref_block_prefix;

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
}
