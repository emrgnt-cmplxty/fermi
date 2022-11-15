// fermi
use crate::{
    account::{ValidatorPubKey, ValidatorPubKeyBytes},
    error::GDEXError,
    utils,
};
// external
use digest::Digest;
use rand::rngs::OsRng;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use sha3::Sha3_256;
use sui_types::sui_serde::{Hex, Readable};

/// Public traits that are externally consumed
pub use fastcrypto::traits::{Error, VerifyingKey};
pub use signature::{Signer, Verifier};
pub use sui_types::crypto::{KeypairTraits, SigningKey, ToFromBytes};

/// The number of bytes in an address.
/// Default to 16 bytes, can be set to 20 bytes with address20 feature.
pub const ADDRESS_LENGTH: usize = if cfg!(feature = "address20") {
    20
} else if cfg!(feature = "address16") {
    16
} else {
    32
};

/// This is a reduced scope use of the SuiPublicKey
pub trait GDEXPublicKey: VerifyingKey {
    const FLAG: u8;
}

/// This is a reduced scope use of the SuiAddress
#[serde_as]
#[derive(Eq, Debug, Default, PartialEq, Ord, PartialOrd, Copy, Clone, Hash, Serialize, Deserialize, JsonSchema)]
pub struct GDEXAddress(
    #[schemars(with = "Hex")]
    #[serde_as(as = "Readable<Hex, _>")]
    [u8; ADDRESS_LENGTH],
);

impl GDEXAddress {
    pub fn optional_address_as_hex<S>(key: &Option<GDEXAddress>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(
            &key.map(|addr| utils::encode_bytes_hex(&addr))
                .unwrap_or_else(|| "".to_string()),
        )
    }

    pub fn optional_address_from_hex<'de, D>(deserializer: D) -> Result<Option<GDEXAddress>, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let value = utils::decode_bytes_hex(&s).map_err(serde::de::Error::custom)?;
        Ok(Some(value))
    }

    pub fn to_inner(self) -> [u8; ADDRESS_LENGTH] {
        self.0
    }
}

impl<T: GDEXPublicKey> From<&T> for GDEXAddress {
    fn from(pk: &T) -> Self {
        let mut hasher = Sha3_256::default();
        hasher.update(&[T::FLAG]);
        hasher.update(pk);
        let g_arr = hasher.finalize();

        let mut res = [0u8; ADDRESS_LENGTH];
        res.copy_from_slice(&AsRef::<[u8]>::as_ref(&g_arr)[..ADDRESS_LENGTH]);
        GDEXAddress(res)
    }
}
/// TryFrom trait is necessary for utils::decode_bytes_hex in optional_address_from_hex
impl TryFrom<&[u8]> for GDEXAddress {
    type Error = GDEXError;

    fn try_from(bytes: &[u8]) -> Result<Self, GDEXError> {
        let arr: [u8; ADDRESS_LENGTH] = bytes.try_into().map_err(|_| GDEXError::InvalidAddress)?;
        Ok(Self(arr))
    }
}

/// AsRef trait is necessary for utils::encode_bytes_hex in optional_address_as_hex
impl AsRef<[u8]> for GDEXAddress {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}

/// From trait is necessary to convert ValidatorPubKeyBytes into a GDEXAddress
impl From<&ValidatorPubKeyBytes> for GDEXAddress {
    fn from(pkb: &ValidatorPubKeyBytes) -> Self {
        let mut hasher = Sha3_256::default();
        hasher.update(&[ValidatorPubKey::FLAG]);
        hasher.update(pkb);
        let g_arr = hasher.finalize();

        let mut res = [0u8; ADDRESS_LENGTH];
        res.copy_from_slice(&AsRef::<[u8]>::as_ref(&g_arr)[..ADDRESS_LENGTH]);
        GDEXAddress(res)
    }
}

// TODO: get_random_key_pair() and get_key_pair_from_bytes() should return KeyPair only.
pub fn get_random_key_pair<KP: KeypairTraits>() -> KP
where
    <KP as KeypairTraits>::PubKey: GDEXPublicKey,
{
    get_key_pair_from_rng(&mut OsRng)
}

/// Generate a keypair from the specified RNG (useful for testing with seedable rngs).
pub fn get_key_pair_from_rng<KP: KeypairTraits, R>(csprng: &mut R) -> KP
where
    R: rand::CryptoRng + rand::RngCore,
    <KP as KeypairTraits>::PubKey: GDEXPublicKey,
{
    KP::generate(csprng)
}

/// Begin the testing suite for serialization
#[cfg(test)]
pub mod crypto_tests {
    use super::*;
    use crate::account::ValidatorKeyPair;

    #[test]
    pub fn get_keypairs() {
        let _key1: ValidatorKeyPair =
            get_key_pair_from_rng::<ValidatorKeyPair, rand::rngs::OsRng>(&mut rand::rngs::OsRng);
        let _key2: ValidatorKeyPair = get_random_key_pair();
    }

    #[test]
    pub fn to_and_from_bytes() {
        let key: ValidatorKeyPair =
            get_key_pair_from_rng::<ValidatorKeyPair, rand::rngs::OsRng>(&mut rand::rngs::OsRng);
        let fermi_addr = GDEXAddress::from(key.public());
        let key_bytes = fermi_addr.as_ref();
        let fermi_addr_from_bytes: GDEXAddress = GDEXAddress::try_from(key_bytes).unwrap();
        assert!(fermi_addr == fermi_addr_from_bytes);
    }
}