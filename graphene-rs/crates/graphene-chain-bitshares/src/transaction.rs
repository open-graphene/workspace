use crate::operations::Operation;

#[derive(Clone, Debug, PartialEq)]
pub enum OperationResult {
    Void,
    Unsupported {
        tag: u16,
        payload: serde_json::Value,
    },
}

impl OperationResult {
    pub fn tag(&self) -> u16 {
        match self {
            Self::Void => 0,
            Self::Unsupported { tag, .. } => *tag,
        }
    }

    pub fn is_typed(&self) -> bool {
        !matches!(self, Self::Unsupported { .. })
    }
}

impl<'de> serde::Deserialize<'de> for OperationResult {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let serde_json::Value::Array(mut elements) = value else {
            return Err(serde::de::Error::custom(
                "operation result static_variant must be a two-element array",
            ));
        };
        if elements.len() != 2 {
            return Err(serde::de::Error::custom(
                "operation result static_variant must be a two-element array",
            ));
        }
        let payload = elements.pop().expect("length checked");
        let tag_value = elements.pop().expect("length checked");
        let tag_u64 = tag_value.as_u64().ok_or_else(|| {
            serde::de::Error::custom(
                "operation result static_variant tag must be an unsigned integer",
            )
        })?;
        let tag = u16::try_from(tag_u64).map_err(|_| {
            serde::de::Error::custom("operation result static_variant tag must fit in u16")
        })?;

        match tag {
            0 => {
                let serde_json::Value::Object(map) = payload else {
                    return Err(serde::de::Error::custom(
                        "void operation result payload must be an empty object",
                    ));
                };
                if !map.is_empty() {
                    return Err(serde::de::Error::custom(
                        "void operation result payload must be an empty object",
                    ));
                }
                Ok(Self::Void)
            }
            _ => Ok(Self::Unsupported { tag, payload }),
        }
    }
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
pub struct Transaction {
    pub ref_block_num: u16,
    pub ref_block_prefix: u32,
    pub expiration: String,
    pub operations: Vec<Operation>,
    pub extensions: Vec<graphene_protocol::RestrictionArgument>,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
pub struct SignedTransaction {
    #[serde(flatten)]
    pub transaction: Transaction,
    pub signatures: Vec<String>,
}
