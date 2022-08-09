//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
//! TODO - How do we test this codec? It seems impossible to test directly given that EncodeBuf does not have a public constructor.
//! further, I could not find any peripheral objects that publicly exposed this struct in the codebase https://github.com/hyperium/tonic/search?q=EncodeBuf
//! Note, the code in this file is inspired by https://github.com/MystenLabs/mysten-infra/blob/main/crates/mysten-network/src/codec.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bc
#[cfg(not(tarpaulin))]
use bytes::{Buf, BufMut};
use std::marker::PhantomData;
use tonic::{
    codec::{Codec, DecodeBuf, Decoder, EncodeBuf, Encoder},
    Status,
};

#[derive(Debug)]
pub struct BincodeEncoder<T>(PhantomData<T>);

impl<T: serde::Serialize> Encoder for BincodeEncoder<T> {
    type Item = T;
    type Error = Status;

    fn encode(&mut self, item: Self::Item, buf: &mut EncodeBuf<'_>) -> Result<(), Self::Error> {
        bincode::serialize_into(buf.writer(), &item).map_err(|e| Status::internal(e.to_string()))
    }
}

#[derive(Debug)]
pub struct BincodeDecoder<U>(PhantomData<U>);

impl<U: serde::de::DeserializeOwned> Decoder for BincodeDecoder<U> {
    type Item = U;
    type Error = Status;

    fn decode(&mut self, buf: &mut DecodeBuf<'_>) -> Result<Option<Self::Item>, Self::Error> {
        if !buf.has_remaining() {
            return Ok(None);
        }

        let item: Self::Item = bincode::deserialize_from(buf.reader()).map_err(|e| Status::internal(e.to_string()))?;
        Ok(Some(item))
    }
}

/// A [`Codec`] that implements `application/grpc+bincode` via the serde library.
#[derive(Debug, Clone)]
pub struct BincodeCodec<T, U>(PhantomData<(T, U)>);

impl<T, U> Default for BincodeCodec<T, U> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T, U> Codec for BincodeCodec<T, U>
where
    T: serde::Serialize + Send + 'static,
    U: serde::de::DeserializeOwned + Send + 'static,
{
    type Encode = T;
    type Decode = U;
    type Encoder = BincodeEncoder<T>;
    type Decoder = BincodeDecoder<U>;

    fn encoder(&mut self) -> Self::Encoder {
        BincodeEncoder(PhantomData)
    }

    fn decoder(&mut self) -> Self::Decoder {
        BincodeDecoder(PhantomData)
    }
}
