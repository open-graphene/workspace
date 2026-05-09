graphene_protocol::define_object_id_type! {
    chain: "BitShares",
    chain_article: "a",
    /// A BitShares account statistics object identifier (`2.6.x`).
    id: "account statistics",
    object_space: 2,
    type_id: 6,
    instance_doc: "Returns the account statistics object instance component.",
    wrong_type_doc: "The object id is not in the BitShares account statistics object range (`2.6.x`).",
    wrong_type_message: "object id {actual} is not a BitShares account statistics id",
}

#[cfg(test)]
mod tests {
    use super::{Error, Id};
    use graphene_protocol::{ObjectId, ObjectIdParseError};

    #[test]
    fn constructs_account_statistics_ids_from_instances() {
        let statistics = Id::new(345);

        assert_eq!(statistics.object_id(), ObjectId::new(2, 6, 345));
        assert_eq!(ObjectId::from(statistics), ObjectId::new(2, 6, 345));
        assert_eq!(statistics.instance(), 345);
        assert_eq!(statistics.to_string(), "2.6.345");
        assert_eq!("2.6.345".parse::<Id>(), Ok(statistics));
    }

    #[test]
    fn rejects_non_account_statistics_ids() {
        assert_eq!(
            "2.5.345".parse::<Id>(),
            Err(Error::WrongType {
                actual: ObjectId::new(2, 5, 345),
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
