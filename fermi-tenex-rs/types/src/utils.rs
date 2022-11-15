// fermi
use crate::crypto::KeypairTraits;
// mysten
use fastcrypto::traits::ToFromBytes;
// external
use anyhow::anyhow;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::net::{TcpListener, TcpStream};

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

pub fn new_network_address() -> multiaddr::Multiaddr {
    format!("/ip4/0.0.0.0/tcp/{}/http", get_available_port())
        .parse()
        .unwrap()
}

pub fn available_local_socket_address() -> std::net::SocketAddr {
    format!("0.0.0.0:{}", get_available_port()).parse().unwrap()
}

pub fn encode_bytes_hex<B: AsRef<[u8]>>(bytes: B) -> String {
    format!("0x{}", hex::encode(bytes.as_ref()))
}

pub fn decode_bytes_hex<T: for<'a> TryFrom<&'a [u8]>>(s: &str) -> Result<T, anyhow::Error> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    let value = hex::decode(s)?;
    T::try_from(&value[..]).map_err(|_| anyhow::anyhow!("byte deserialization failed"))
}

pub fn read_keypair_from_file<K: KeypairTraits + DeserializeOwned, P: AsRef<std::path::Path>>(
    path: P,
) -> anyhow::Result<K> {
    let contents = std::fs::read_to_string(path)?;
    serde_json::from_str(contents.as_str().trim()).map_err(|e| anyhow!(e))
}

pub fn write_keypair_to_file<K: KeypairTraits + Serialize, P: AsRef<std::path::Path>>(
    keypair: &K,
    path: P,
) -> anyhow::Result<()> {
    let contents = serde_json::to_string_pretty(keypair)?;
    std::fs::write(path, contents)?;

    Ok(())
}

pub fn write_keypair_to_file_raw<K: KeypairTraits + Serialize, P: AsRef<std::path::Path>>(
    keypair: &K,
    path: P,
) -> anyhow::Result<()> {
    let keypair_copy = keypair.copy();
    let private = keypair_copy.private();
    let contents = encode_bytes_hex(private.as_bytes());
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