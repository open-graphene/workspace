graphene_protocol::define_object_id_type! {
    chain: "RSquared",
    chain_article: "an",
    /// A RSquared witness object identifier (`1.6.x`).
    id: "witness",
    type_id: 6,
    instance_doc: "Returns the witness object instance component.",
    wrong_type_doc: "The object id is not in the RSquared witness object range (`1.6.x`).",
    wrong_type_message: "object id {actual} is not a RSquared witness id",
}

#[cfg(test)]
mod tests {
    use super::{Error, Id};
    use graphene_protocol::{ObjectId, ObjectIdParseError};

    #[test]
    fn constructs_witness_ids_from_instances() {
        let witness = Id::new(11);

        assert_eq!(witness.object_id(), ObjectId::new(1, 6, 11));
        assert_eq!(ObjectId::from(witness), ObjectId::new(1, 6, 11));
        assert_eq!(Id::try_from(ObjectId::new(1, 6, 11)), Ok(witness));
    }

    #[test]
    fn rejects_non_witness_object_ids() {
        assert_eq!(
            "1.5.11".parse::<Id>(),
            Err(Error::WrongType {
                actual: ObjectId::new(1, 5, 11),
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
