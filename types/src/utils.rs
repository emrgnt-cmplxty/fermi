//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
use crate::crypto::KeypairTraits;
use anyhow::anyhow;
use std::net::{TcpListener, TcpStream};

/// This class is taken directly from https://github.com/MystenLabs/sui/blob/main/crates/sui-config/src/utils.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bc
/// Return an ephemeral, available port. On unix systems, the port returned will be in the
/// TIME_WAIT state ensuring that the OS won't hand out this port for some grace period.
/// Callers should be able to bind to this port given they use SO_REUSEADDR.
pub fn get_available_port() -> u16 {
    const MAX_PORT_RETRIES: u32 = 1000;

    for _ in 0..MAX_PORT_RETRIES {
        if let Ok(port) = get_ephemeral_port() {
            return port;
        }
    }

    panic!("Error: could not find an available port");
}

/// This function is taken directly from https://github.com/MystenLabs/sui/blob/main/crates/sui-config/src/utils.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bc
fn get_ephemeral_port() -> ::std::io::Result<u16> {
    // Request a random available port from the OS
    let listener = TcpListener::bind(("localhost", 0))?;
    let addr = listener.local_addr()?;

    // Create and accept a connection (which we'll promptly drop) in order to force the port
    // into the TIME_WAIT state, ensuring that the port will be reserved from some limited
    // amount of time (roughly 60s on some Linux systems)
    let _sender = TcpStream::connect(addr)?;
    let _incoming = listener.accept()?;

    Ok(addr.port())
}

/// This function is taken directly from https://github.com/MystenLabs/sui/blob/main/crates/sui-config/src/utils.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bc
pub fn new_network_address() -> multiaddr::Multiaddr {
    format!("/dns/localhost/tcp/{}/http", get_available_port())
        .parse()
        .unwrap()
}

/// This function is taken directly from https://github.com/MystenLabs/sui/blob/main/crates/sui-config/src/utils.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bc
pub fn available_local_socket_address() -> std::net::SocketAddr {
    format!("127.0.0.1:{}", get_available_port()).parse().unwrap()
}

/// This function is taken directly from https://github.com/MystenLabs/sui/blob/main/crates/sui-types/src/base_types.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bc
pub fn encode_bytes_hex<B: AsRef<[u8]>>(bytes: B) -> String {
    hex::encode(bytes.as_ref())
}

/// This function is taken directly from https://github.com/MystenLabs/sui/blob/main/crates/sui-types/src/base_types.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bc
pub fn decode_bytes_hex<T: for<'a> TryFrom<&'a [u8]>>(s: &str) -> Result<T, anyhow::Error> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    let value = hex::decode(s)?;
    T::try_from(&value[..]).map_err(|_| anyhow::anyhow!("byte deserialization failed"))
}

/// This function is taken directly from https://github.com/MystenLabs/sui/blob/main/crates/sui/src/keytool.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bc
pub fn read_keypair_from_file<K: KeypairTraits, P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<K> {
    let contents = std::fs::read_to_string(path)?;
    K::decode_base64(contents.as_str().trim()).map_err(|e| anyhow!(e))
}

/// This function is taken directly from https://github.com/MystenLabs/sui/blob/main/crates/sui/src/keytool.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bc
pub fn write_keypair_to_file<K: KeypairTraits, P: AsRef<std::path::Path>>(keypair: &K, path: P) -> anyhow::Result<()> {
    let contents = keypair.encode_base64();
    std::fs::write(path, contents)?;
    Ok(())
}

#[allow(unused_must_use)]
pub fn set_testing_telemetry(filter: &str) {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(filter)
        .finish();
    // unwrapping causes failure in tests, but is not a problem in production
    tracing::subscriber::set_global_default(subscriber);
}
