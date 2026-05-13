use std::fmt;
use std::io::{self, Read, Write};
use std::str::FromStr;

pub trait GrapheneEncode {
    fn encode_graphene<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError>;

    fn to_graphene_bytes(&self) -> Result<Vec<u8>, EncodeError> {
        let mut bytes = Vec::new();
        self.encode_graphene(&mut bytes)?;
        Ok(bytes)
    }
}

pub trait GrapheneDecode: Sized {
    fn decode_graphene<R: Read>(reader: &mut R) -> Result<Self, DecodeError>;
}

#[derive(Debug)]
pub enum EncodeError {
    Io(io::Error),
    LengthOverflow { len: usize },
    ObjectIdInstanceOverflow { instance: u64 },
}

impl fmt::Display for EncodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(
                formatter,
                "I/O error while encoding Graphene bytes: {error}"
            ),
            Self::LengthOverflow { len } => {
                write!(
                    formatter,
                    "container length {len} does not fit in Graphene varint"
                )
            }
            Self::ObjectIdInstanceOverflow { instance } => write!(
                formatter,
                "object id instance {instance} does not fit in Graphene typed object id encoding",
            ),
        }
    }
}

impl std::error::Error for EncodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::LengthOverflow { .. } | Self::ObjectIdInstanceOverflow { .. } => None,
        }
    }
}

impl From<io::Error> for EncodeError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Debug)]
pub enum DecodeError {
    Io(io::Error),
    InvalidBool(u8),
    VarintOverflow,
    Utf8(std::string::FromUtf8Error),
    InvalidOptionTag(u8),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(
                formatter,
                "I/O error while decoding Graphene bytes: {error}"
            ),
            Self::InvalidBool(value) => write!(formatter, "invalid Graphene bool byte {value}"),
            Self::VarintOverflow => write!(formatter, "Graphene varint exceeds u64"),
            Self::Utf8(error) => {
                write!(formatter, "invalid UTF-8 string in Graphene bytes: {error}")
            }
            Self::InvalidOptionTag(value) => {
                write!(formatter, "invalid Graphene optional tag {value}")
            }
        }
    }
}

impl std::error::Error for DecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Utf8(error) => Some(error),
            Self::InvalidBool(_) | Self::VarintOverflow | Self::InvalidOptionTag(_) => None,
        }
    }
}

impl From<io::Error> for DecodeError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<std::string::FromUtf8Error> for DecodeError {
    fn from(error: std::string::FromUtf8Error) -> Self {
        Self::Utf8(error)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UnsignedVarint(pub u64);

impl GrapheneEncode for UnsignedVarint {
    fn encode_graphene<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        let mut value = self.0;
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            writer.write_all(&[byte])?;
            if value == 0 {
                return Ok(());
            }
        }
    }
}

impl GrapheneDecode for UnsignedVarint {
    fn decode_graphene<R: Read>(reader: &mut R) -> Result<Self, DecodeError> {
        let mut value = 0u64;
        for shift in (0..=63).step_by(7) {
            let byte = read_u8(reader)?;
            let low = u64::from(byte & 0x7f);
            if shift == 63 && low > 1 {
                return Err(DecodeError::VarintOverflow);
            }
            value |= low << shift;
            if byte & 0x80 == 0 {
                return Ok(Self(value));
            }
        }
        Err(DecodeError::VarintOverflow)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ObjectId {
    pub space: u8,
    pub type_id: u8,
    pub instance: u64,
}

impl ObjectId {
    pub fn new(space: u8, type_id: u8, instance: u64) -> Self {
        Self {
            space,
            type_id,
            instance,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseObjectIdError {
    WrongPartCount,
    InvalidSpace,
    InvalidType,
    InvalidInstance,
}

impl fmt::Display for ParseObjectIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongPartCount => {
                write!(formatter, "object id must have shape space.type.instance")
            }
            Self::InvalidSpace => write!(formatter, "object id space is not a valid u8"),
            Self::InvalidType => write!(formatter, "object id type is not a valid u8"),
            Self::InvalidInstance => write!(formatter, "object id instance is not a valid u64"),
        }
    }
}

impl std::error::Error for ParseObjectIdError {}

impl FromStr for ObjectId {
    type Err = ParseObjectIdError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut parts = value.split('.');
        let space = parts
            .next()
            .ok_or(ParseObjectIdError::WrongPartCount)?
            .parse::<u8>()
            .map_err(|_| ParseObjectIdError::InvalidSpace)?;
        let type_id = parts
            .next()
            .ok_or(ParseObjectIdError::WrongPartCount)?
            .parse::<u8>()
            .map_err(|_| ParseObjectIdError::InvalidType)?;
        let instance = parts
            .next()
            .ok_or(ParseObjectIdError::WrongPartCount)?
            .parse::<u64>()
            .map_err(|_| ParseObjectIdError::InvalidInstance)?;
        if parts.next().is_some() {
            return Err(ParseObjectIdError::WrongPartCount);
        }
        Ok(Self::new(space, type_id, instance))
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}.{}.{}",
            self.space, self.type_id, self.instance
        )
    }
}

impl GrapheneEncode for ObjectId {
    fn encode_graphene<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        if self.instance > u64::from(u32::MAX) {
            return Err(EncodeError::ObjectIdInstanceOverflow {
                instance: self.instance,
            });
        }
        UnsignedVarint(self.instance).encode_graphene(writer)
    }
}

impl GrapheneDecode for ObjectId {
    fn decode_graphene<R: Read>(reader: &mut R) -> Result<Self, DecodeError> {
        let instance = UnsignedVarint::decode_graphene(reader)?.0;
        Ok(Self::new(0, 0, instance))
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TimePointSec(pub u32);

impl GrapheneEncode for TimePointSec {
    fn encode_graphene<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        self.0.encode_graphene(writer)
    }
}

impl GrapheneDecode for TimePointSec {
    fn decode_graphene<R: Read>(reader: &mut R) -> Result<Self, DecodeError> {
        u32::decode_graphene(reader).map(Self)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StaticVariantTag(pub u64);

impl GrapheneEncode for StaticVariantTag {
    fn encode_graphene<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        UnsignedVarint(self.0).encode_graphene(writer)
    }
}

impl GrapheneDecode for StaticVariantTag {
    fn decode_graphene<R: Read>(reader: &mut R) -> Result<Self, DecodeError> {
        UnsignedVarint::decode_graphene(reader).map(|value| Self(value.0))
    }
}

impl GrapheneEncode for bool {
    fn encode_graphene<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        writer.write_all(&[u8::from(*self)])?;
        Ok(())
    }
}

impl GrapheneDecode for bool {
    fn decode_graphene<R: Read>(reader: &mut R) -> Result<Self, DecodeError> {
        match read_u8(reader)? {
            0 => Ok(false),
            1 => Ok(true),
            value => Err(DecodeError::InvalidBool(value)),
        }
    }
}

macro_rules! fixed_int_codec {
    ($($ty:ty),* $(,)?) => {
        $(
            impl GrapheneEncode for $ty {
                fn encode_graphene<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
                    writer.write_all(&self.to_le_bytes())?;
                    Ok(())
                }
            }

            impl GrapheneDecode for $ty {
                fn decode_graphene<R: Read>(reader: &mut R) -> Result<Self, DecodeError> {
                    let mut bytes = [0u8; std::mem::size_of::<$ty>()];
                    reader.read_exact(&mut bytes)?;
                    Ok(<$ty>::from_le_bytes(bytes))
                }
            }
        )*
    };
}

fixed_int_codec!(u8, u16, u32, u64, i8, i16, i32, i64);

impl GrapheneEncode for String {
    fn encode_graphene<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        self.as_str().encode_graphene(writer)
    }
}

impl GrapheneEncode for str {
    fn encode_graphene<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        UnsignedVarint(
            u64::try_from(self.len())
                .map_err(|_| EncodeError::LengthOverflow { len: self.len() })?,
        )
        .encode_graphene(writer)?;
        writer.write_all(self.as_bytes())?;
        Ok(())
    }
}

impl GrapheneDecode for String {
    fn decode_graphene<R: Read>(reader: &mut R) -> Result<Self, DecodeError> {
        let len = UnsignedVarint::decode_graphene(reader)?.0;
        let mut bytes = vec![0u8; usize::try_from(len).map_err(|_| DecodeError::VarintOverflow)?];
        reader.read_exact(&mut bytes)?;
        Ok(String::from_utf8(bytes)?)
    }
}

impl<T: GrapheneEncode> GrapheneEncode for Vec<T> {
    fn encode_graphene<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        UnsignedVarint(
            u64::try_from(self.len())
                .map_err(|_| EncodeError::LengthOverflow { len: self.len() })?,
        )
        .encode_graphene(writer)?;
        for item in self {
            item.encode_graphene(writer)?;
        }
        Ok(())
    }
}

impl<T: GrapheneDecode> GrapheneDecode for Vec<T> {
    fn decode_graphene<R: Read>(reader: &mut R) -> Result<Self, DecodeError> {
        let len = UnsignedVarint::decode_graphene(reader)?.0;
        let len = usize::try_from(len).map_err(|_| DecodeError::VarintOverflow)?;
        let mut items = Vec::with_capacity(len);
        for _ in 0..len {
            items.push(T::decode_graphene(reader)?);
        }
        Ok(items)
    }
}

impl<T: GrapheneEncode> GrapheneEncode for Option<T> {
    fn encode_graphene<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        match self {
            Some(value) => {
                writer.write_all(&[1])?;
                value.encode_graphene(writer)
            }
            None => {
                writer.write_all(&[0])?;
                Ok(())
            }
        }
    }
}

impl<T: GrapheneDecode> GrapheneDecode for Option<T> {
    fn decode_graphene<R: Read>(reader: &mut R) -> Result<Self, DecodeError> {
        match read_u8(reader)? {
            0 => Ok(None),
            1 => T::decode_graphene(reader).map(Some),
            value => Err(DecodeError::InvalidOptionTag(value)),
        }
    }
}

impl<A: GrapheneEncode, B: GrapheneEncode> GrapheneEncode for (A, B) {
    fn encode_graphene<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        self.0.encode_graphene(writer)?;
        self.1.encode_graphene(writer)
    }
}

impl<A: GrapheneDecode, B: GrapheneDecode> GrapheneDecode for (A, B) {
    fn decode_graphene<R: Read>(reader: &mut R) -> Result<Self, DecodeError> {
        Ok((A::decode_graphene(reader)?, B::decode_graphene(reader)?))
    }
}

fn read_u8<R: Read>(reader: &mut R) -> Result<u8, DecodeError> {
    let mut byte = [0u8; 1];
    reader.read_exact(&mut byte)?;
    Ok(byte[0])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encode<T: GrapheneEncode + ?Sized>(value: &T) -> Vec<u8> {
        value.to_graphene_bytes().expect("encoding should succeed")
    }

    fn decode<T: GrapheneDecode>(bytes: &[u8]) -> T {
        T::decode_graphene(&mut &bytes[..]).expect("decoding should succeed")
    }

    #[test]
    fn fixed_width_integers_encode_little_endian() {
        assert_eq!(encode(&0x1234u16), vec![0x34, 0x12]);
        assert_eq!(encode(&0x1234_5678u32), vec![0x78, 0x56, 0x34, 0x12]);
        assert_eq!(decode::<u32>(&[0x78, 0x56, 0x34, 0x12]), 0x1234_5678);
    }

    #[test]
    fn unsigned_varint_uses_leb128_shape() {
        assert_eq!(encode(&UnsignedVarint(0)), vec![0x00]);
        assert_eq!(encode(&UnsignedVarint(127)), vec![0x7f]);
        assert_eq!(encode(&UnsignedVarint(128)), vec![0x80, 0x01]);
        assert_eq!(encode(&UnsignedVarint(345)), vec![0xd9, 0x02]);
        assert_eq!(decode::<UnsignedVarint>(&[0xd9, 0x02]), UnsignedVarint(345));
    }

    #[test]
    fn object_id_parses_text_and_encodes_typed_instance_only() {
        let id = ObjectId::from_str("1.2.345").expect("valid object id");
        assert_eq!(id.space, 1);
        assert_eq!(id.type_id, 2);
        assert_eq!(id.instance, 345);
        assert_eq!(id.to_string(), "1.2.345");
        assert_eq!(encode(&id), vec![0xd9, 0x02]);
    }

    #[test]
    fn string_vec_option_and_tuple_have_fc_container_shape() {
        assert_eq!(encode("BTS"), vec![3, b'B', b'T', b'S']);
        assert_eq!(decode::<String>(&[3, b'B', b'T', b'S']), "BTS");
        assert_eq!(encode(&vec![1u16, 2u16]), vec![2, 1, 0, 2, 0]);
        assert_eq!(encode(&Some(7u8)), vec![1, 7]);
        assert_eq!(encode(&Option::<u8>::None), vec![0]);
        assert_eq!(encode(&(1u8, 0x0203u16)), vec![1, 3, 2]);
    }

    #[test]
    fn time_point_sec_is_uint32_seconds_since_epoch() {
        assert_eq!(
            encode(&TimePointSec(1_700_000_000)),
            vec![0x00, 0xf1, 0x53, 0x65]
        );
        assert_eq!(
            decode::<TimePointSec>(&[0x00, 0xf1, 0x53, 0x65]),
            TimePointSec(1_700_000_000)
        );
    }

    #[test]
    fn static_variant_tag_is_varint() {
        assert_eq!(encode(&StaticVariantTag(40)), vec![40]);
        assert_eq!(decode::<StaticVariantTag>(&[40]), StaticVariantTag(40));
    }
}
