crate::types::prefixed_string::define_prefixed_string_type! {
    /// A BitShares address in its chain-prefixed string form.
    type: Address,
    error: Error,
    prefix: "BTS",
    empty_doc: "The address string was empty.",
    wrong_prefix_doc: "The address does not use the BitShares `BTS` prefix.",
    empty_message: "BitShares address must not be empty",
    wrong_prefix_message: "BitShares address must start with BTS",
}

#[cfg(test)]
mod tests {
    use super::{Address, Error};

    #[test]
    fn parses_bitshares_addresses() {
        let address: Address = "BTS1111111111111111111111111111111114T1Anm"
            .parse()
            .expect("valid prefixed address");

        assert_eq!(
            address.as_str(),
            "BTS1111111111111111111111111111111114T1Anm"
        );
        assert_eq!(
            address.to_string(),
            "BTS1111111111111111111111111111111114T1Anm"
        );
    }

    #[test]
    fn rejects_empty_addresses() {
        assert_eq!("".parse::<Address>(), Err(Error::Empty));
    }

    #[test]
    fn rejects_addresses_with_wrong_prefix() {
        assert_eq!(
            "R2S1111111111111111111111111111111114T1Anm".parse::<Address>(),
            Err(Error::WrongPrefix)
        );
    }
}
