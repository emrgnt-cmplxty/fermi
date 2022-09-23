// Copyright (c) 2022, Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

type Result<T> = ::std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn main() -> Result<()> {
    build_proto("bank")?;
    build_proto("spot")?;
    build_proto("futures")?;

    Ok(())
}

fn build_proto(controller_name: &str) -> Result<()> {
    let proto_dir = format!("./src/{}/proto", controller_name);
    let requests_proto_file = format!("./src/{}/proto/{}_proto.proto", controller_name, controller_name);
    let generated_dir = format!("./src/{}/generated/", controller_name);

    let proto_files = &[requests_proto_file];
    let dirs = &[proto_dir];

    // Use `Bytes` instead of `Vec<u8>` for bytes fields
    let mut config = prost_build::Config::new();
    config.bytes(&["."]);

    tonic_build::configure()
        .out_dir(generated_dir)
        .compile_with_config(config, proto_files, dirs)?;

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=proto");
    println!("cargo:rerun-if-env-changed=DUMP_GENERATED_GRPC");

    nightly();
    beta();
    stable();

    Ok(())
}

#[rustversion::nightly]
fn nightly() {
    println!("cargo:rustc-cfg=nightly");
}

#[rustversion::not(nightly)]
fn nightly() {}

#[rustversion::beta]
fn beta() {
    println!("cargo:rustc-cfg=beta");
}

#[rustversion::not(beta)]
fn beta() {}

#[rustversion::stable]
fn stable() {
    println!("cargo:rustc-cfg=stable");
}

#[rustversion::not(stable)]
fn stable() {}
