graphene_protocol::define_object_id_type! {
    chain: "RSquared",
    chain_article: "an",
    /// A RSquared asset object identifier (`1.3.x`).
    id: "asset",
    type_id: 3,
    instance_doc: "Returns the asset object instance component.",
    wrong_type_doc: "The object id is not in the RSquared asset object range (`1.3.x`).",
    wrong_type_message: "object id {actual} is not a RSquared asset id",
}

#[cfg(test)]
mod tests {
    use super::{Error, Id};
    use graphene_protocol::{ObjectId, ObjectIdParseError};

    #[test]
    fn constructs_asset_ids_from_instances() {
        let asset = Id::new(7);

        assert_eq!(asset.object_id(), ObjectId::new(1, 3, 7));
        assert_eq!(ObjectId::from(asset), ObjectId::new(1, 3, 7));
        assert_eq!(Id::try_from(ObjectId::new(1, 3, 7)), Ok(asset));
    }

    #[test]
    fn rejects_non_asset_object_ids() {
        assert_eq!(
            "1.2.7".parse::<Id>(),
            Err(Error::WrongType {
                actual: ObjectId::new(1, 2, 7),
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
