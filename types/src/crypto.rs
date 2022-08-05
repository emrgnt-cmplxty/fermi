use crate::error::SuiError;
use crate::utils;
use digest::Digest;
use rand::rngs::OsRng;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use sha3::Sha3_256;
use sui_types::sui_serde::{Hex, Readable};

// declare traits
pub use signature::Signer;
pub use signature::Verifier;
pub use narwhal_crypto::traits::VerifyingKey;

pub use sui_types::crypto::KeypairTraits;
pub use sui_types::crypto::ToFromBytes;

/// The number of bytes in an address.
/// Default to 16 bytes, can be set to 20 bytes with address20 feature.
pub const ADDRESS_LENGTH: usize = if cfg!(feature = "address20") {
    20
} else if cfg!(feature = "address32") {
    32
} else {
    16
};

pub trait GDEXPublicKey: VerifyingKey {
    const FLAG: u8;
}

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
    type Error = SuiError;

    fn try_from(bytes: &[u8]) -> Result<Self, SuiError> {
        let arr: [u8; ADDRESS_LENGTH] = bytes.try_into().map_err(|_| SuiError::InvalidAddress)?;
        Ok(Self(arr))
    }
}

/// AsRef trait is necessary for utils::encode_bytes_hex in optional_address_as_hex
impl AsRef<[u8]> for GDEXAddress {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}

// TODO: get_key_pair() and get_key_pair_from_bytes() should return KeyPair only.
// TODO: rename to random_key_pair
pub fn get_key_pair<KP: KeypairTraits>() -> (GDEXAddress, KP)
where
    <KP as KeypairTraits>::PubKey: GDEXPublicKey,
{
    get_key_pair_from_rng(&mut OsRng)
}

/// Generate a keypair from the specified RNG (useful for testing with seedable rngs).
pub fn get_key_pair_from_rng<KP: KeypairTraits, R>(csprng: &mut R) -> (GDEXAddress, KP)
where
    R: rand::CryptoRng + rand::RngCore,
    <KP as KeypairTraits>::PubKey: GDEXPublicKey,
{
    let kp = KP::generate(csprng);
    (kp.public().into(), kp)
}

pub fn write_keypair_to_file<K: KeypairTraits, P: AsRef<std::path::Path>>(keypair: &K, path: P) -> anyhow::Result<()> {
    let contents = keypair.encode_base64();
    std::fs::write(path, contents)?;
    Ok(())
}
