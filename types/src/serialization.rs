//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
//! This package appears to give us a flexible and easy to implement serialization standard for conversion into bytes and back

use anyhow::anyhow;
use base64ct::Encoding as _;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

/// This struct is taken directly from https://github.com/MystenLabs/sui/blob/main/crates/sui-types/src/sui_serde.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bc
pub type KeyPairBase64 = sui_types::sui_serde::KeyPairBase64;

/// This struct is taken directly from https://github.com/MystenLabs/sui/blob/main/crates/sui-types/src/sui_serde.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bc
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, JsonSchema)]
#[serde(try_from = "String")]
pub struct Base64(String);

/// This trait is taken directly from https://github.com/MystenLabs/sui/blob/main/crates/sui-types/src/sui_serde.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bc
pub trait Encoding {
    fn decode(s: &str) -> Result<Vec<u8>, anyhow::Error>;
    fn encode<T: AsRef<[u8]>>(data: T) -> String;
}

/// This impl is taken directly from https://github.com/MystenLabs/sui/blob/main/crates/sui-types/src/sui_serde.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bc
impl Encoding for Base64 {
    fn decode(s: &str) -> Result<Vec<u8>, anyhow::Error> {
        base64ct::Base64::decode_vec(s).map_err(|e| anyhow!(e))
    }

    fn encode<T: AsRef<[u8]>>(data: T) -> String {
        base64ct::Base64::encode_string(data.as_ref())
    }
}

/// allow conversions of bytes like
/// let s: String = Base64::encode(&bytes);
impl TryFrom<String> for Base64 {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        // Make sure the value is valid base64 string.
        Base64::decode(&value)?;
        Ok(Self(value))
    }
}

/// Begin the testing suite for serialization
#[cfg(test)]
pub mod serialization_tests {
    use super::*;

    #[test]
    pub fn serialize_deserialize_string() {
        let input_string = String::from("input");
        let bytes = input_string.as_bytes();
        let _encoded_string: String = Base64::encode(&bytes);
    }

    #[test]
    pub fn serialize_deserialize_vec() {
        let vec = vec![0, 1, 2, 3, 4, 5];
        let bytes = Base64::encode(&vec);
        let deserialized_vec = Base64::decode(&bytes).unwrap();
        assert!(vec == deserialized_vec);
    }
}
