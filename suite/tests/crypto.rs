
#[cfg(test)]
mod tests {
    use gdex_crypto::{
        ed25519::{Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature},
        hash::CryptoHash,
        traits::{Signature, SigningKey, Uniform},
    };
    use gdex_crypto_derive::{BCSCryptoHash, CryptoHasher};
    use rand::{prelude::ThreadRng, thread_rng};
    use serde::{Deserialize, Serialize};
    use types::spot::{DiemCryptoMessage};
    
    // make a new struct for an order that we have to hash
    #[derive(Serialize, Deserialize, CryptoHasher, BCSCryptoHash)]
    struct Order {
        quantity: i32,
        side: String,
    }
    
    // testing basic verification
    #[test]
    fn test_basic_verification() {
        let mut csprng: ThreadRng = thread_rng();
        let priv_key = Ed25519PrivateKey::generate(&mut csprng);
        let pub_key: Ed25519PublicKey = (&priv_key).into();
        let msg = DiemCryptoMessage("".to_string());
        let sig: Ed25519Signature = priv_key.sign(&msg);
        // gives us 1 if it was verified and 0 if it wasn't
        let result = match sig.verify(&msg, &pub_key) {
            Ok(_) => 1,
            Err(_) => 0,
        };
        // it should be verified in this case
        assert_eq!(result, 1);
    }

    // testing batch verification
    #[test]
    fn test_batch_verification() {
        let mut csprng: ThreadRng = thread_rng();
        let priv_key = Ed25519PrivateKey::generate(&mut csprng);
        let pub_key: Ed25519PublicKey = (&priv_key).into();
        let msg = DiemCryptoMessage("".to_string());
        let sig: Ed25519Signature = priv_key.sign(&msg);
        let mut keys_and_signatures: Vec<(Ed25519PublicKey, Ed25519Signature)> = Vec::new();
        keys_and_signatures.push((pub_key, sig));

        // gives us 1 if it was verified and 0 if it wasn't
        let result = match Signature::batch_verify(&msg, keys_and_signatures) {
            Ok(_) => 1,
            Err(_) => 0,
        };
        // it should be verified in this case
        assert_eq!(result, 1);
    }

    // testing incorrect message verification fail
    #[test]
    fn test_incorrect_message_verification_fail() {
        let mut csprng: ThreadRng = thread_rng();
        let priv_key = Ed25519PrivateKey::generate(&mut csprng);
        let pub_key: Ed25519PublicKey = (&priv_key).into();
        let msg = DiemCryptoMessage("".to_string());
        // making a different message than what was encoded
        let faulty_message = DiemCryptoMessage("a".to_string());
        // signing correct message but verifying with faulty message
        let sig: Ed25519Signature = priv_key.sign(&msg);
        let result = match sig.verify(&faulty_message, &pub_key) {
            Ok(_) => 1,
            Err(_) => 0,
        };
        // making sure it failed
        assert_eq!(result, 0);
    }

    // testing incorrect keys fail
    #[test]
    fn test_incorrect_key_failure() {
        // first set of private and public keys that it will sign with
        let mut csprng: ThreadRng = thread_rng();
        let priv_key = Ed25519PrivateKey::generate(&mut csprng);
        // making a new public key
        let mut csprng_2: ThreadRng = thread_rng();
        let priv_key_2 = Ed25519PrivateKey::generate(&mut csprng_2);
        let pub_key_2: Ed25519PublicKey = (&priv_key_2).into();
        // creating and signing the msg
        let msg = DiemCryptoMessage("".to_string());
        // making a different message than what was encoded
        let sig: Ed25519Signature = priv_key.sign(&msg);
        let result = match sig.verify(&msg, &pub_key_2) {
            Ok(_) => 1,
            Err(_) => 0,
        };
        // making sure it failed
        assert_eq!(result, 0);
    }

    // testing that if we have an order, and we hash it twice we receive the same hash
    #[test]
    fn test_hashing_basic() {
        // instnatiating an order
        let order = Order {
            quantity: 10,
            side: String::from("Buy"),
        };
        // getting the hash the first time
        let hash1 = order.hash();
        // hashing it a second time
        let hash2 = order.hash();
        assert_eq!(hash1, hash2);
    }

    // here we will hash two different orders and make sure that the verification fails
    #[test]
    fn test_hashing_verification_fail() {
        // instnatiating an order
        let order1 = Order {
            quantity: 10,
            side: String::from("Buy"),
        };
        // getting the hash the first time
        let hash1 = order1.hash();
        // instnatiating an order
        let order2 = Order {
            quantity: 2,
            side: String::from("Sell"),
        };
        // getting the hash the first time
        let hash2 = order2.hash();
        assert_ne!(hash1, hash2);
    }
}
