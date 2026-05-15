use graphene_rpc::GrapheneTimePointSec;

use crate::block_id::{ref_block_prefix, BlockIdError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransactionHeaderFields {
    pub ref_block_num: u16,
    pub ref_block_prefix: u32,
    pub expiration: GrapheneTimePointSec,
}

pub fn compute_transaction_header_fields(
    head_block_id: &str,
    head_block_number: u32,
    expiration: GrapheneTimePointSec,
) -> Result<TransactionHeaderFields, BlockIdError> {
    Ok(TransactionHeaderFields {
        ref_block_num: (head_block_number & 0xffff) as u16,
        ref_block_prefix: ref_block_prefix(head_block_id)?,
        expiration,
    })
}

#[cfg(test)]
mod tests {
    use super::compute_transaction_header_fields;

    #[test]
    fn transaction_header_fields_derive_ref_block_values() {
        let block_id = "00012345a1b2c3d4ffffffffffffffffffffffffffffffffffffffffffffffff";
        let expiration = "2026-05-13T20:05:00".parse().expect("valid timestamp");

        let fields = compute_transaction_header_fields(block_id, 0x12345, expiration).unwrap();

        assert_eq!(fields.ref_block_num, 0x2345);
        assert_eq!(fields.ref_block_prefix, 0xd4c3b2a1);
        assert_eq!(fields.expiration, expiration);
    }
}
