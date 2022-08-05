//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0

use anyhow::anyhow;
use base64ct::Encoding as _;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

/// This class is taken directly from https://github.com/MystenLabs/sui/blob/main/crates/sui-types/src/sui_serde.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bc
pub type KeyPairBase64 = sui_types::sui_serde::KeyPairBase64;

impl Encoding for Base64 {
    fn decode(s: &str) -> Result<Vec<u8>, anyhow::Error> {
        base64ct::Base64::decode_vec(s).map_err(|e| anyhow!(e))
    }

    fn encode<T: AsRef<[u8]>>(data: T) -> String {
        base64ct::Base64::encode_string(data.as_ref())
    }
}

/// This class is taken directly from https://github.com/MystenLabs/sui/blob/main/crates/sui-types/src/sui_serde.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bc
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, JsonSchema)]
#[serde(try_from = "String")]
pub struct Base64(String);

impl TryFrom<String> for Base64 {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        // Make sure the value is valid base64 string.
        Base64::decode(&value)?;
        Ok(Self(value))
    }
}

/// This trait is taken directly from https://github.com/MystenLabs/sui/blob/main/crates/sui-types/src/sui_serde.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bc
pub trait Encoding {
    fn decode(s: &str) -> Result<Vec<u8>, anyhow::Error>;
    fn encode<T: AsRef<[u8]>>(data: T) -> String;
}
