//! Hash functions and their outputs

use crate::error::{Error, Kind};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use std::convert::TryFrom;
use std::{
    fmt::{self, Debug, Display},
    str::FromStr,
};
use subtle_encoding::{Encoding, Hex};
use tendermint_proto::DomainType;

/// Output size for the SHA-256 hash function
pub const SHA256_HASH_SIZE: usize = 32;

/// Hash algorithms
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum Algorithm {
    /// SHA-256
    Sha256,
}

/// Hash digests
#[derive(Copy, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum Hash {
    /// SHA-256 hashes
    Sha256([u8; SHA256_HASH_SIZE]),
}

impl DomainType<Vec<u8>> for Hash {}

/// Default conversion from Vec<u8> is SHA256 Hash
impl TryFrom<Vec<u8>> for Hash {
    type Error = Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Hash::from_bytes(Algorithm::Sha256, &value)
    }
}

impl From<Hash> for Vec<u8> {
    fn from(value: Hash) -> Self {
        if value == Hash::default() {
            return vec![];
        }
        match value {
            Hash::Sha256(s) => s.to_vec(),
        }
    }
}

impl Hash {
    /// Create a new `Hash` with the given algorithm type
    pub fn from_bytes(alg: Algorithm, bytes: &[u8]) -> Result<Hash, Error> {
        match alg {
            Algorithm::Sha256 => {
                if bytes.len() == SHA256_HASH_SIZE {
                    let mut h = [0u8; SHA256_HASH_SIZE];
                    h.copy_from_slice(bytes);
                    Ok(Hash::Sha256(h))
                } else {
                    Err(Kind::Parse.into())
                }
            }
        }
    }

    /// Decode a `Hash` from upper-case hexadecimal
    pub fn from_hex_upper(alg: Algorithm, s: &str) -> Result<Hash, Error> {
        match alg {
            Algorithm::Sha256 => {
                let mut h = [0u8; SHA256_HASH_SIZE];
                Hex::upper_case().decode_to_slice(s.as_bytes(), &mut h)?;
                Ok(Hash::Sha256(h))
            }
        }
    }

    /// Return the digest algorithm used to produce this hash
    pub fn algorithm(self) -> Algorithm {
        match self {
            Hash::Sha256(_) => Algorithm::Sha256,
        }
    }

    /// Borrow the `Hash` as a byte slice
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Hash::Sha256(ref h) => h.as_ref(),
        }
    }
}

impl Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Hash::Sha256(_) => write!(f, "Hash::Sha256({})", self),
        }
    }
}

impl Default for Hash {
    fn default() -> Self {
        Hash::from_bytes(Algorithm::Sha256, &[0; SHA256_HASH_SIZE]).unwrap()
    }
}

impl Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hex = match self {
            Hash::Sha256(ref h) => Hex::upper_case().encode_to_string(h).unwrap(),
        };

        write!(f, "{}", hex)
    }
}

impl FromStr for Hash {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        Self::from_hex_upper(Algorithm::Sha256, s)
    }
}

impl<'de> Deserialize<'de> for Hash {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let hex = String::deserialize(deserializer)?;

        if hex.is_empty() {
            Err(D::Error::custom("empty hash"))
        } else {
            Ok(Self::from_str(&hex).map_err(|e| D::Error::custom(format!("{}", e)))?)
        }
    }
}

impl Serialize for Hash {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}

/// AppHash is usually a SHA256 hash, but in reality it can be any kind of data
#[derive(Clone)]
pub struct AppHash(Vec<u8>);

impl DomainType<Vec<u8>> for AppHash {}

impl TryFrom<Vec<u8>> for AppHash {
    type Error = Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(AppHash(value))
    }
}
impl From<AppHash> for Vec<u8> {
    fn from(value: AppHash) -> Self {
        value.0
    }
}

impl AppHash {
    /// Return AppHash value as vec<u8>
    pub fn value(&self) -> Vec<u8> {
        self.0.clone()
    }

    /// Decode a `Hash` from upper-case hexadecimal
    pub fn from_hex_upper(s: &str) -> Result<Self, Error> {
        if s.len() % 2 != 0 {
            return Err(Kind::InvalidAppHashLength.into());
        }
        let mut h = vec![0; s.len() / 2];
        Hex::upper_case().decode_to_slice(s.as_bytes(), &mut h)?;
        Ok(AppHash(h))
    }
}

impl AsRef<[u8]> for AppHash {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl Debug for AppHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "hash::AppHash({:?})", self.0)
    }
}

impl Display for AppHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            Hex::upper_case().encode_to_string(&self.0).unwrap()
        )
    }
}

impl PartialEq for AppHash {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl FromStr for AppHash {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        Self::from_hex_upper(s)
    }
}

impl<'de> Deserialize<'de> for AppHash {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let hex = String::deserialize(deserializer)?;
        Ok(Self::from_str(&hex).map_err(|e| D::Error::custom(format!("{}", e)))?)
    }
}

impl Serialize for AppHash {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}
