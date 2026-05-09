graphene_protocol::define_object_id_type! {
    chain: "BitShares",
    chain_article: "a",
    /// A BitShares asset bitasset data object identifier (`2.4.x`).
    id: "asset bitasset data",
    object_space: 2,
    type_id: 4,
    instance_doc: "Returns the asset bitasset data object instance component.",
    wrong_type_doc: "The object id is not in the BitShares asset bitasset data object range (`2.4.x`).",
    wrong_type_message: "object id {actual} is not a BitShares asset bitasset data id",
}

#[cfg(test)]
mod tests {
    use super::{Error, Id};
    use graphene_protocol::{ObjectId, ObjectIdParseError};

    #[test]
    fn constructs_asset_bitasset_data_ids_from_instances() {
        let data = Id::new(345);

        assert_eq!(data.object_id(), ObjectId::new(2, 4, 345));
        assert_eq!(ObjectId::from(data), ObjectId::new(2, 4, 345));
        assert_eq!(data.instance(), 345);
        assert_eq!(data.to_string(), "2.4.345");
        assert_eq!("2.4.345".parse::<Id>(), Ok(data));
    }

    #[test]
    fn rejects_non_asset_bitasset_data_ids() {
        assert_eq!(
            "2.3.345".parse::<Id>(),
            Err(Error::WrongType {
                actual: ObjectId::new(2, 3, 345),
            })
        );
    }

    #[test]
    fn propagates_object_id_parse_errors() {
        assert_eq!(
            "not-an-id".parse::<Id>(),
            Err(Error::Parse(ObjectIdParseError::WrongPartCount))
        );
    }
}
