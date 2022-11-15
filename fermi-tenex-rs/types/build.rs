type Result<T> = ::std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn main() -> Result<()> {
    let proto_files = &["proto/services.proto", "proto/transaction.proto"];
    let dirs = &["proto"];

    // Use `Bytes` instead of `Vec<u8>` for bytes fields
    let mut config = prost_build::Config::new();
    config.bytes(&["."]);

    tonic_build::configure()
        .out_dir("./src/generated/")
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