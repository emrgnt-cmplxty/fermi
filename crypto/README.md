---
id: crypto
title: Crypto
custom_edit_url: https://github.com/diem/diem/edit/main/crypto/crypto/README.md
---

The crypto component hosts all the implementations of cryptographic primitives we use in GDEX: hashing, signing, and key derivation/generation. The parts of the library using traits.rs contains the crypto API enforcing type safety, verifiable random functions, EdDSA & MultiEdDSA signatures.

## Overview

GDEX makes use of several cryptographic algorithms:

* SHA-3 as the main hash function. It is standardized in [FIPS 202](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.202.pdf). It is based on the [tiny_keccak](https://docs.rs/tiny-keccak/1.4.2/tiny_keccak/) library.
* HKDF: HMAC-based Extract-and-Expand Key Derivation Function (HKDF) based on [RFC 5869](https://tools.ietf.org/html/rfc5869). It is used to generate keys from a salt (optional), seed, and application-info (optional).
* traits.rs introduces new abstractions for the crypto API.
* Ed25519 performs signatures using the new API design based on [ed25519-dalek](https://docs.rs/ed25519-dalek/1.0.0-pre.1/ed25519_dalek/) library with additional security checks (e.g. for malleability).
* X25519 to perform key exchanges. It is used to secure communications between validators via the [Noise Protocol Framework](http://www.noiseprotocol.org/noise.html). It is based on the x25519-dalek library.

## How is this module organized?
```
    crypto/src
    ├── compat.rs
    ├── ed25519.rs          # Ed25519 implementation of the signing/verification API in traits.rs
    ├── error.rs
    ├── hash.rs             # Hash function (SHA-3)
    ├── lib.rs
    ├── multi_ed25519.rs    # MultiEd25519 implementation of the signing/verification API in traits.rs
    ├── test_utils.rs
    ├── traits.rs           # New API design and the necessary abstractions
    ├── x25519.rs           # X25519 wrapper
```

Note: This crate historically had support for BLS12381, ECVRF, and SlIP-0010, though were removed due to lack of use. The last git revision before there removal is 00301524.
