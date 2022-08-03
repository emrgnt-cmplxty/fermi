// Copyright (c) 2022, BTI
// SPDX-License-Identifier: Apache-2.0
use narwhal_crypto::ed25519::{Ed25519KeyPair, Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature};

pub type AccountPubKey = Ed25519PublicKey;
pub type AccountPrivKey = Ed25519PrivateKey;
pub type AccountSignature = Ed25519Signature;
pub type AccountKeyPair = Ed25519KeyPair;
pub type AccountBalance = u64;
