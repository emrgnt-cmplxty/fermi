fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/faucet.proto")?;
    tonic_build::compile_protos("../types/proto/relay.proto")?;
    Ok(())
}
