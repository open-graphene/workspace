pub mod types;

#[cfg(test)]
mod tests {
    use graphene_protocol::ObjectId;

    use crate::types::{account, asset, committee_member, witness};

    #[test]
    fn parses_account_ids_from_bitshares_object_ids() {
        let account: account::Id = "1.2.345".parse().expect("valid BitShares account id");

        assert_eq!(account.object_id(), ObjectId::new(1, 2, 345));
        assert_eq!(account.instance(), 345);
        assert_eq!(account.to_string(), "1.2.345");
    }

    #[test]
    fn parses_asset_ids_from_bitshares_object_ids() {
        let asset: asset::Id = "1.3.7".parse().expect("valid BitShares asset id");

        assert_eq!(asset.object_id(), ObjectId::new(1, 3, 7));
        assert_eq!(asset.instance(), 7);
        assert_eq!(asset.to_string(), "1.3.7");
    }

    #[test]
    fn parses_committee_member_ids_from_bitshares_object_ids() {
        let committee_member: committee_member::Id = "1.5.9"
            .parse()
            .expect("valid BitShares committee member id");

        assert_eq!(committee_member.object_id(), ObjectId::new(1, 5, 9));
        assert_eq!(committee_member.instance(), 9);
        assert_eq!(committee_member.to_string(), "1.5.9");
    }

    #[test]
    fn parses_witness_ids_from_bitshares_object_ids() {
        let witness: witness::Id = "1.6.11".parse().expect("valid BitShares witness id");

        assert_eq!(witness.object_id(), ObjectId::new(1, 6, 11));
        assert_eq!(witness.instance(), 11);
        assert_eq!(witness.to_string(), "1.6.11");
    }
}
