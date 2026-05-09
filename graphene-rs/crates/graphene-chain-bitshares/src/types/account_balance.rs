graphene_protocol::define_object_id_type! {
    chain: "BitShares",
    chain_article: "a",
    /// A BitShares account balance object identifier (`2.5.x`).
    id: "account balance",
    object_space: 2,
    type_id: 5,
    instance_doc: "Returns the account balance object instance component.",
    wrong_type_doc: "The object id is not in the BitShares account balance object range (`2.5.x`).",
    wrong_type_message: "object id {actual} is not a BitShares account balance id",
}

#[cfg(test)]
mod tests {
    use super::{Error, Id};
    use graphene_protocol::{ObjectId, ObjectIdParseError};

    #[test]
    fn constructs_account_balance_ids_from_instances() {
        let balance = Id::new(345);

        assert_eq!(balance.object_id(), ObjectId::new(2, 5, 345));
        assert_eq!(ObjectId::from(balance), ObjectId::new(2, 5, 345));
        assert_eq!(balance.instance(), 345);
        assert_eq!(balance.to_string(), "2.5.345");
        assert_eq!("2.5.345".parse::<Id>(), Ok(balance));
    }

    #[test]
    fn rejects_non_account_balance_ids() {
        assert_eq!(
            "2.6.345".parse::<Id>(),
            Err(Error::WrongType {
                actual: ObjectId::new(2, 6, 345),
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
