graphene_protocol::define_object_id_type! {
    chain: "RSquared",
    chain_article: "an",
    /// A RSquared committee member object identifier (`1.5.x`).
    id: "committee member",
    type_id: 5,
    instance_doc: "Returns the committee member object instance component.",
    wrong_type_doc: "The object id is not in the RSquared committee member object range (`1.5.x`).",
    wrong_type_message: "object id {actual} is not a RSquared committee member id",
}

#[cfg(test)]
mod tests {
    use super::{Error, Id};
    use graphene_protocol::{ObjectId, ObjectIdParseError};

    #[test]
    fn constructs_committee_member_ids_from_instances() {
        let committee_member = Id::new(9);

        assert_eq!(committee_member.object_id(), ObjectId::new(1, 5, 9));
        assert_eq!(ObjectId::from(committee_member), ObjectId::new(1, 5, 9));
        assert_eq!(Id::try_from(ObjectId::new(1, 5, 9)), Ok(committee_member));
    }

    #[test]
    fn rejects_non_committee_member_object_ids() {
        assert_eq!(
            "1.6.9".parse::<Id>(),
            Err(Error::WrongType {
                actual: ObjectId::new(1, 6, 9),
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
