// Copyright (c) 2022, Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::{env, path::PathBuf};
use tonic_build::manual::{Builder, Method, Service};
// use crate::codec;

type Result<T> = ::std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn main() -> Result<()> {
    let out_dir = if env::var("DUMP_GENERATED_GRPC").is_ok() {
        PathBuf::from("")
    } else {
        PathBuf::from(env::var("OUT_DIR")?)
    };

    let codec_path = "crate::codec::BincodeCodec";

    let validator_service = Service::builder()
        .name("ValidatorAPI")
        .package("gdex.validator")
        .comment("The Validator interface")
        .method(
            Method::builder()
                .name("transaction")
                .route_name("SignedTransaction")
                .input_type("gdex_types::transaction::SignedTransaction")
                .output_type("gdex_types::transaction::SignedTransaction")
                .codec_path(codec_path)
                .build(),
        )
        .build();

    Builder::new().out_dir(&out_dir).compile(&[validator_service]);

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=DUMP_GENERATED_GRPC");

    Ok(())
}
