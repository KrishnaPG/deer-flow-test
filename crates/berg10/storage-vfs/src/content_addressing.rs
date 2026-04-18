use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContentHashError {
    #[error("invalid base58 hash: {0}")]
    InvalidHash(String),
    #[error("hash length mismatch: expected 32 bytes, got {0}")]
    LengthMismatch(usize),
}

/// A content-addressed hash: base58(blake3(payload)).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ContentHash {
    b58: String,
    bytes: [u8; 32],
}

impl ContentHash {
    pub fn new(data: &[u8]) -> Self {
        let bytes = blake3::hash(data).into();
        let b58 = bs58::encode(&bytes).into_string();
        Self { b58, bytes }
    }

    pub fn from_b58(b58: &str) -> Result<Self, ContentHashError> {
        let bytes = bs58::decode(b58)
            .into_vec()
            .map_err(|e| ContentHashError::InvalidHash(e.to_string()))?;

        if bytes.len() != 32 {
            return Err(ContentHashError::LengthMismatch(bytes.len()));
        }

        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Self {
            b58: b58.to_string(),
            bytes: arr,
        })
    }

    pub fn as_str(&self) -> &str {
        &self.b58
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }
}

impl FromStr for ContentHash {
    type Err = ContentHashError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_b58(s)
    }
}

impl TryFrom<&str> for ContentHash {
    type Error = ContentHashError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_b58(value)
    }
}

impl TryFrom<String> for ContentHash {
    type Error = ContentHashError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_b58(&value)
    }
}

impl From<ContentHash> for String {
    fn from(value: ContentHash) -> Self {
        value.b58
    }
}

impl std::fmt::Display for ContentHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.b58)
    }
}

impl Serialize for ContentHash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ContentHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_b58(&value).map_err(serde::de::Error::custom)
    }
}

/// Hash file payload with blake3 and encode as base58.
pub fn hash_content(data: &[u8]) -> ContentHash {
    ContentHash::new(data)
}

/// Decode a base58-encoded hash back to its 32-byte blake3 digest.
pub fn decode_content_hash(b58: &str) -> Result<[u8; 32], ContentHashError> {
    let ch = ContentHash::from_b58(b58)?;
    Ok(ch.bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_content_is_deterministic() {
        let data = b"hello world";
        let h1 = hash_content(data);
        let h2 = hash_content(data);
        assert_eq!(h1, h2);
    }

    #[test]
    fn hash_content_is_unique_for_different_input() {
        let h1 = hash_content(b"hello");
        let h2 = hash_content(b"world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn base58_round_trip() {
        let data = b"test content for round trip";
        let ch = hash_content(data);
        let decoded = decode_content_hash(ch.as_str()).unwrap();
        assert_eq!(decoded, *ch.as_bytes());
    }

    #[test]
    fn identical_content_produces_identical_hash() {
        let data = b"identical content";
        let h1 = hash_content(data);
        let h2 = hash_content(data);
        assert_eq!(h1.as_str(), h2.as_str());
        assert_eq!(h1.as_bytes(), h2.as_bytes());
    }

    #[test]
    fn invalid_base58_rejected() {
        assert!(decode_content_hash("0OIl").is_err());
    }

    #[test]
    fn wrong_length_rejected() {
        let short = bs58::encode(b"short").into_string();
        assert!(decode_content_hash(&short).is_err());
    }
}
