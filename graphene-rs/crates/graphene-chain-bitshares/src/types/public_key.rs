crate::types::prefixed_string::define_prefixed_string_type! {
    /// A BitShares public key in its chain-prefixed string form.
    type: PublicKey,
    error: Error,
    prefix: "BTS",
    empty_doc: "The public key string was empty.",
    wrong_prefix_doc: "The public key does not use the BitShares `BTS` prefix.",
    empty_message: "BitShares public key must not be empty",
    wrong_prefix_message: "BitShares public key must start with BTS",
}

#[cfg(test)]
mod tests {
    use super::{Error, PublicKey};

    #[test]
    fn parses_bitshares_public_keys() {
        let key: PublicKey = "BTS1111111111111111111111111111111114T1Anm"
            .parse()
            .expect("valid prefixed public key");

        assert_eq!(key.as_str(), "BTS1111111111111111111111111111111114T1Anm");
        assert_eq!(
            key.to_string(),
            "BTS1111111111111111111111111111111114T1Anm"
        );
    }

    #[test]
    fn rejects_empty_public_keys() {
        assert_eq!("".parse::<PublicKey>(), Err(Error::Empty));
    }

    #[test]
    fn rejects_public_keys_with_wrong_prefix() {
        assert_eq!(
            "R2S1111111111111111111111111111111114T1Anm".parse::<PublicKey>(),
            Err(Error::WrongPrefix)
        );
    }
}
