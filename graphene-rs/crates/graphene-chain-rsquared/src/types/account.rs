graphene_protocol::define_object_id_type! {
    chain: "RSquared",
    chain_article: "an",
    /// A RSquared account object identifier (`1.2.x`).
    id: "account",
    type_id: 2,
    instance_doc: "Returns the account object instance component.",
    wrong_type_doc: "The object id is not in the RSquared account object range (`1.2.x`).",
    wrong_type_message: "object id {actual} is not a RSquared account id",
}

#[cfg(test)]
mod tests {
    use super::{Error, Id};
    use graphene_protocol::{ObjectId, ObjectIdParseError};

    #[test]
    fn constructs_account_ids_from_instances() {
        let account = Id::new(345);

        assert_eq!(account.object_id(), ObjectId::new(1, 2, 345));
        assert_eq!(ObjectId::from(account), ObjectId::new(1, 2, 345));
        assert_eq!(Id::try_from(ObjectId::new(1, 2, 345)), Ok(account));
    }

    #[test]
    fn rejects_non_account_object_ids() {
        assert_eq!(
            "1.3.345".parse::<Id>(),
            Err(Error::WrongType {
                actual: ObjectId::new(1, 3, 345),
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
