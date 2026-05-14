use std::str::FromStr;

use graphene_codec::{EncodeError, GrapheneEncode, ObjectId, StaticVariantTag};
use graphene_rpc::GrapheneTimePointSec;

use crate::generated::{
    Asset, AssetAssetId, ExtensionsType, FutureExtensions, Operation, Transaction,
    TransferOperation, TransferOperationFrom, TransferOperationTo,
};

impl GrapheneEncode for Asset {
    fn encode_graphene<W: std::io::Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        self.amount.as_i64().encode_graphene(writer)?;
        self.asset_id.encode_graphene(writer)
    }
}

impl GrapheneEncode for AssetAssetId {
    fn encode_graphene<W: std::io::Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        encode_object_id(self, writer)
    }
}

impl GrapheneEncode for TransferOperationFrom {
    fn encode_graphene<W: std::io::Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        encode_object_id(self, writer)
    }
}

impl GrapheneEncode for TransferOperationTo {
    fn encode_graphene<W: std::io::Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        encode_object_id(self, writer)
    }
}

impl GrapheneEncode for ExtensionsType {
    fn encode_graphene<W: std::io::Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        self.0.encode_graphene(writer)
    }
}

impl GrapheneEncode for FutureExtensions {
    fn encode_graphene<W: std::io::Write>(&self, _writer: &mut W) -> Result<(), EncodeError> {
        Err(EncodeError::Unsupported {
            feature: "future extension payloads",
        })
    }
}

impl GrapheneEncode for TransferOperation {
    fn encode_graphene<W: std::io::Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        // FC_REFLECT(graphene::protocol::transfer_operation,
        //            (fee)(from)(to)(amount)(memo)(extensions))
        self.fee.encode_graphene(writer)?;
        self.from.encode_graphene(writer)?;
        self.to.encode_graphene(writer)?;
        self.amount.encode_graphene(writer)?;
        match self.memo {
            Some(_) => {
                return Err(EncodeError::Unsupported {
                    feature: "transfer_operation memo_data",
                });
            }
            None => 0u8.encode_graphene(writer)?,
        }
        self.extensions.encode_graphene(writer)
    }
}

impl GrapheneEncode for Operation {
    fn encode_graphene<W: std::io::Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        match self {
            Operation::Transfer(operation) => {
                StaticVariantTag(0).encode_graphene(writer)?;
                operation.encode_graphene(writer)
            }
            _ => unimplemented!(
                "Graphene binary encoding is currently implemented only for transfer_operation"
            ),
        }
    }
}

impl GrapheneEncode for Transaction {
    fn encode_graphene<W: std::io::Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        // FC_REFLECT(graphene::protocol::transaction,
        //            (ref_block_num)(ref_block_prefix)(expiration)(operations)(extensions))
        self.ref_block_num.encode_graphene(writer)?;
        self.ref_block_prefix.encode_graphene(writer)?;
        encode_time_point_sec(&self.expiration, writer)?;
        self.operations.encode_graphene(writer)?;
        self.extensions.encode_graphene(writer)
    }
}

fn encode_time_point_sec<W>(value: &GrapheneTimePointSec, writer: &mut W) -> Result<(), EncodeError>
where
    W: std::io::Write,
{
    let seconds = value.naive_utc().and_utc().timestamp();
    let seconds =
        u32::try_from(seconds).map_err(|_| EncodeError::TimestampOutOfRange { seconds })?;
    seconds.encode_graphene(writer)
}

fn encode_object_id<W, T>(value: &T, writer: &mut W) -> Result<(), EncodeError>
where
    W: std::io::Write,
    T: std::ops::Deref<Target = String>,
{
    ObjectId::from_str(value)
        .map_err(|_| EncodeError::InvalidObjectId {
            value: value.to_string(),
        })?
        .encode_graphene(writer)
}
