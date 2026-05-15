use std::fmt;
use std::str::FromStr;

use secp256k1::{Message, Secp256k1, SecretKey};
use sha2::{Digest as _, Sha256};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ChainId([u8; 32]);

impl ChainId {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_hex(self) -> String {
        bytes_to_hex(&self.0)
    }
}

impl From<[u8; 32]> for ChainId {
    fn from(bytes: [u8; 32]) -> Self {
        Self::new(bytes)
    }
}

impl From<ChainId> for [u8; 32] {
    fn from(value: ChainId) -> Self {
        value.0
    }
}

impl fmt::Display for ChainId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&bytes_to_hex(&self.0))
    }
}

impl FromStr for ChainId {
    type Err = ParseHexBytesError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        parse_hex_array(value).map(Self)
    }
}

impl TryFrom<&str> for ChainId {
    type Error = ParseHexBytesError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl TryFrom<String> for ChainId {
    type Error = ParseHexBytesError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SigningDigest([u8; 32]);

impl SigningDigest {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_hex(self) -> String {
        bytes_to_hex(&self.0)
    }
}

impl From<[u8; 32]> for SigningDigest {
    fn from(bytes: [u8; 32]) -> Self {
        Self::new(bytes)
    }
}

impl From<SigningDigest> for [u8; 32] {
    fn from(value: SigningDigest) -> Self {
        value.0
    }
}

impl fmt::Display for SigningDigest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&bytes_to_hex(&self.0))
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PublicKey([u8; 33]);

impl PublicKey {
    pub fn new_compressed(bytes: [u8; 33]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 33] {
        &self.0
    }

    pub fn to_hex(self) -> String {
        bytes_to_hex(&self.0)
    }
}

impl From<[u8; 33]> for PublicKey {
    fn from(bytes: [u8; 33]) -> Self {
        Self::new_compressed(bytes)
    }
}

impl From<PublicKey> for [u8; 33] {
    fn from(value: PublicKey) -> Self {
        value.0
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&bytes_to_hex(&self.0))
    }
}

impl FromStr for PublicKey {
    type Err = ParseHexBytesError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        parse_hex_array(value).map(Self)
    }
}

impl TryFrom<&str> for PublicKey {
    type Error = ParseHexBytesError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl TryFrom<String> for PublicKey {
    type Error = ParseHexBytesError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Signature([u8; 65]);

impl Signature {
    pub fn new_compact(bytes: [u8; 65]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 65] {
        &self.0
    }

    pub fn to_hex(self) -> String {
        bytes_to_hex(&self.0)
    }
}

impl From<[u8; 65]> for Signature {
    fn from(bytes: [u8; 65]) -> Self {
        Self::new_compact(bytes)
    }
}

impl From<Signature> for [u8; 65] {
    fn from(value: Signature) -> Self {
        value.0
    }
}

impl fmt::Display for Signature {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&bytes_to_hex(&self.0))
    }
}

impl FromStr for Signature {
    type Err = ParseHexBytesError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        parse_hex_array(value).map(Self)
    }
}

impl TryFrom<&str> for Signature {
    type Error = ParseHexBytesError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl TryFrom<String> for Signature {
    type Error = ParseHexBytesError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

pub trait Signer {
    fn public_key(&self) -> PublicKey;
    fn sign_digest(&self, digest: &SigningDigest) -> Result<Signature, SignError>;
}

#[derive(Clone)]
pub struct WifSigner {
    secret_key: SecretKey,
    public_key: PublicKey,
}

impl WifSigner {
    pub fn from_wif(wif: &str) -> Result<Self, WifError> {
        let secret_key = decode_wif_secret_key(wif)?;
        let secp = Secp256k1::new();
        let public_key = secp256k1::PublicKey::from_secret_key(&secp, &secret_key).serialize();

        Ok(Self {
            secret_key,
            public_key: PublicKey::new_compressed(public_key),
        })
    }
}

impl Signer for WifSigner {
    fn public_key(&self) -> PublicKey {
        self.public_key
    }

    fn sign_digest(&self, digest: &SigningDigest) -> Result<Signature, SignError> {
        let message = Message::from_digest_slice(digest.as_bytes())
            .map_err(|error| SignError::Backend(error.to_string()))?;
        let secp = Secp256k1::new();
        for attempt in 0u8..=255 {
            let signature = if attempt == 0 {
                secp.sign_ecdsa_recoverable(&message, &self.secret_key)
            } else {
                let mut noncedata = [0u8; 32];
                noncedata[0] = attempt;
                secp.sign_ecdsa_recoverable_with_noncedata(&message, &self.secret_key, &noncedata)
            };
            let (recovery_id, compact) = signature.serialize_compact();

            let mut bytes = [0u8; 65];
            bytes[0] = 27 + 4 + recovery_id.to_i32() as u8;
            bytes[1..].copy_from_slice(&compact);
            if is_canonical_graphene_signature(&bytes) {
                return Ok(Signature::new_compact(bytes));
            }
        }

        Err(SignError::Unsupported(
            "canonical Graphene compact signature after 256 nonce attempts",
        ))
    }
}

fn is_canonical_graphene_signature(bytes: &[u8; 65]) -> bool {
    (bytes[1] & 0x80) == 0
        && !(bytes[1] == 0 && (bytes[2] & 0x80) == 0)
        && (bytes[33] & 0x80) == 0
        && !(bytes[33] == 0 && (bytes[34] & 0x80) == 0)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WifError {
    Base58(String),
    WrongPayloadLength { actual: usize },
    InvalidVersion { actual: u8 },
    InvalidChecksum,
    InvalidSecretKey(String),
}

impl fmt::Display for WifError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Base58(message) => write!(formatter, "invalid WIF base58: {message}"),
            Self::WrongPayloadLength { actual } => {
                write!(formatter, "invalid WIF payload length: {actual}")
            }
            Self::InvalidVersion { actual } => write!(
                formatter,
                "invalid WIF version byte: expected 0x80, got 0x{actual:02x}"
            ),
            Self::InvalidChecksum => write!(formatter, "invalid WIF checksum"),
            Self::InvalidSecretKey(message) => {
                write!(formatter, "invalid WIF secret key: {message}")
            }
        }
    }
}

impl std::error::Error for WifError {}

fn decode_wif_secret_key(wif: &str) -> Result<SecretKey, WifError> {
    let decoded = bs58::decode(wif)
        .into_vec()
        .map_err(|error| WifError::Base58(error.to_string()))?;

    if decoded.len() != 37 && decoded.len() != 38 {
        return Err(WifError::WrongPayloadLength {
            actual: decoded.len(),
        });
    }
    if decoded[0] != 0x80 {
        return Err(WifError::InvalidVersion { actual: decoded[0] });
    }

    let payload_len = if decoded.len() == 38 {
        if decoded[33] != 0x01 {
            return Err(WifError::WrongPayloadLength {
                actual: decoded.len(),
            });
        }
        34
    } else {
        33
    };

    let checksum = double_sha256(&decoded[..payload_len]);
    if decoded[payload_len..] != checksum[..4] {
        return Err(WifError::InvalidChecksum);
    }

    SecretKey::from_slice(&decoded[1..33])
        .map_err(|error| WifError::InvalidSecretKey(error.to_string()))
}

fn double_sha256(bytes: &[u8]) -> [u8; 32] {
    let first = Sha256::digest(bytes);
    let second = Sha256::digest(first);
    let mut out = [0u8; 32];
    out.copy_from_slice(&second);
    out
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SignError {
    Backend(String),
    Unsupported(&'static str),
}

impl fmt::Display for SignError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Backend(message) => write!(formatter, "signing backend error: {message}"),
            Self::Unsupported(feature) => {
                write!(formatter, "signing feature is unsupported: {feature}")
            }
        }
    }
}

impl std::error::Error for SignError {}

pub fn signing_digest(chain_id: &ChainId, transaction_bytes: &[u8]) -> SigningDigest {
    let mut hasher = Sha256::new();
    hasher.update(chain_id.as_bytes());
    hasher.update(transaction_bytes);
    let digest = hasher.finalize();
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&digest);
    SigningDigest(bytes)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseHexBytesError {
    WrongLength { expected: usize, actual: usize },
    InvalidHex { index: usize, byte: u8 },
}

impl fmt::Display for ParseHexBytesError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongLength { expected, actual } => write!(
                formatter,
                "expected {expected} hex characters, got {actual}"
            ),
            Self::InvalidHex { index, byte } => write!(
                formatter,
                "invalid hex byte 0x{byte:02x} at character index {index}"
            ),
        }
    }
}

impl std::error::Error for ParseHexBytesError {}

fn parse_hex_array<const N: usize>(value: &str) -> Result<[u8; N], ParseHexBytesError> {
    let bytes = value.as_bytes();
    let expected = N * 2;
    if bytes.len() != expected {
        return Err(ParseHexBytesError::WrongLength {
            expected,
            actual: bytes.len(),
        });
    }

    let mut out = [0u8; N];
    for (byte_index, chunk) in bytes.chunks_exact(2).enumerate() {
        let high = hex_nibble(chunk[0]).ok_or(ParseHexBytesError::InvalidHex {
            index: byte_index * 2,
            byte: chunk[0],
        })?;
        let low = hex_nibble(chunk[1]).ok_or(ParseHexBytesError::InvalidHex {
            index: byte_index * 2 + 1,
            byte: chunk[1],
        })?;
        out[byte_index] = (high << 4) | low;
    }
    Ok(out)
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SWAPLOCK_CHAIN_ID: &str =
        "2267f694d96b7ffdcba1a98c63c09e720a18a85ad34954e299c66d5a42234098";

    #[derive(Clone, Copy, Debug)]
    struct FixedSigner {
        public_key: PublicKey,
        signature: Signature,
    }

    impl Signer for FixedSigner {
        fn public_key(&self) -> PublicKey {
            self.public_key
        }

        fn sign_digest(&self, _digest: &SigningDigest) -> Result<Signature, SignError> {
            Ok(self.signature)
        }
    }

    #[test]
    fn chain_id_parses_and_formats_hex() {
        let chain_id = ChainId::from_str(SWAPLOCK_CHAIN_ID).expect("valid chain id");

        assert_eq!(chain_id.to_string(), SWAPLOCK_CHAIN_ID);
        assert_eq!(chain_id.to_hex(), SWAPLOCK_CHAIN_ID);
        assert_eq!(chain_id.as_bytes()[0], 0x22);
        assert_eq!(chain_id.as_bytes()[31], 0x98);
    }

    #[test]
    fn fixed_size_hex_wrappers_parse_and_format() {
        let public_key_hex = concat!(
            "02",
            "1111111111111111111111111111111111111111111111111111111111111111"
        );
        let signature_hex = concat!(
            "1f",
            "2222222222222222222222222222222222222222222222222222222222222222",
            "3333333333333333333333333333333333333333333333333333333333333333"
        );

        let public_key = PublicKey::from_str(public_key_hex).expect("valid compressed key");
        let signature = Signature::from_str(signature_hex).expect("valid compact signature");

        assert_eq!(public_key.as_bytes().len(), 33);
        assert_eq!(signature.as_bytes().len(), 65);
        assert_eq!(public_key.to_hex(), public_key_hex);
        assert_eq!(signature.to_hex(), signature_hex);
    }

    #[test]
    fn hex_wrappers_reject_wrong_lengths_and_invalid_hex() {
        assert_eq!(
            ChainId::from_str("abc").expect_err("short chain id should fail"),
            ParseHexBytesError::WrongLength {
                expected: 64,
                actual: 3
            }
        );
        assert_eq!(
            ChainId::from_str("2267f694d96b7ffdcba1a98c63c09e720a18a85ad34954e299c66d5a4223409z",)
                .expect_err("bad hex should fail"),
            ParseHexBytesError::InvalidHex {
                index: 63,
                byte: b'z'
            }
        );
        assert_eq!(
            PublicKey::from_str(SWAPLOCK_CHAIN_ID).expect_err("chain id is not a public key"),
            ParseHexBytesError::WrongLength {
                expected: 66,
                actual: 64
            }
        );
    }

    #[test]
    fn signing_digest_hashes_chain_id_then_transaction_bytes() {
        let chain_id = ChainId::from_str(SWAPLOCK_CHAIN_ID).expect("valid chain id");
        let transaction_bytes = [0x34, 0x12, 0xef, 0xcd, 0xab, 0x89];

        let digest = signing_digest(&chain_id, &transaction_bytes);

        assert_eq!(
            digest.to_hex(),
            "71c2da649305e692a4fb5432f81df2571572d909698e73766cc80dc53eb78c65"
        );
    }

    #[test]
    fn signer_trait_returns_public_key_and_signature_without_backend_choice() {
        let public_key = PublicKey::from_str(concat!(
            "02",
            "1111111111111111111111111111111111111111111111111111111111111111"
        ))
        .expect("valid public key");
        let signature = Signature::from_str(concat!(
            "1f",
            "2222222222222222222222222222222222222222222222222222222222222222",
            "3333333333333333333333333333333333333333333333333333333333333333"
        ))
        .expect("valid signature");
        let digest = SigningDigest::new([7u8; 32]);
        let signer = FixedSigner {
            public_key,
            signature,
        };

        assert_eq!(signer.public_key(), public_key);
        assert_eq!(
            signer.sign_digest(&digest).expect("fixed signer"),
            signature
        );
    }

    #[test]
    fn wif_signer_derives_compressed_public_key_from_compressed_wif() {
        let signer = WifSigner::from_wif("KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn")
            .expect("known compressed WIF fixture should parse");

        assert_eq!(
            signer.public_key().to_hex(),
            "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798"
        );
    }

    #[test]
    fn wif_signer_produces_recoverable_graphene_compact_signature() {
        let signer = WifSigner::from_wif("KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn")
            .expect("known compressed WIF fixture should parse");
        let digest = SigningDigest::new([7u8; 32]);

        let signature = signer.sign_digest(&digest).expect("digest should sign");

        assert!((31..=34).contains(&signature.as_bytes()[0]));
        assert_eq!(signature.as_bytes().len(), 65);
        assert!(is_canonical_graphene_signature(signature.as_bytes()));
    }

    #[test]
    fn wif_signer_accepts_uncompressed_wif_and_derives_compressed_public_key() {
        let signer = WifSigner::from_wif("5HpHagT65TZzG1PH3CSu63k8DbpvD8s5ip4nEB3kEsreAnchuDf")
            .expect("known uncompressed WIF fixture should parse");

        assert_eq!(
            signer.public_key().to_hex(),
            "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798"
        );
    }
}
