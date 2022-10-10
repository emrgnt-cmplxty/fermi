// gdex
use crate::crypto::GDEXPublicKey;

/// The account key logic is fully configurable to allow for agile changes throughout the
/// codebase, by simply changing the specified type here
/// for now we are leveraging the Sui crypto library and only form a local implementations
/// when it gives a clear reduction in overhead and enhanced consistency
pub type ValidatorPubKey = Ed25519PublicKeyLocal;
pub type ValidatorPrivKey = sui_types::crypto::AuthorityPrivateKey;
pub type ValidatorSignature = sui_types::crypto::AuthoritySignature;
pub type ValidatorKeyPair = ValidatorKeyPairLocal;
pub type ValidatorPubKeyBytes = sui_types::crypto::AuthorityPublicKeyBytes;

pub type AccountPubKey = sui_types::crypto::AccountPublicKey;
pub type AccountPrivKey = sui_types::crypto::AccountPrivateKey;
pub type AccountSignature = sui_types::crypto::AccountSignature;
pub type AccountKeyPair = sui_types::crypto::AccountKeyPair;
pub type AccountBalance = u64;

/// create a local representation of the Ed25519PublicKey in order to implement necessary traits
/// such a change is necessary in order to implement the GDEXPublicKey locally, rather than utilize
/// the exposed SuiPublicKey
pub type Ed25519PublicKeyLocal = sui_types::crypto::AuthorityPublicKey;

impl GDEXPublicKey for Ed25519PublicKeyLocal {
    const FLAG: u8 = 0x00;
}

pub type ValidatorKeyPairLocal = sui_types::crypto::AuthorityKeyPair;

/// Begin externally available testing functions
#[cfg(any(test, feature = "testing"))]
pub mod account_test_functions {
    use super::*;
    use crate::crypto::KeypairTraits;
    use rand::{rngs::StdRng, SeedableRng};

    pub fn generate_keypair_vec(seed: [u8; 32]) -> Vec<AccountKeyPair> {
        let mut rng = StdRng::from_seed(seed);
        (0..4).map(|_| AccountKeyPair::generate(&mut rng)).collect()
    }
}
