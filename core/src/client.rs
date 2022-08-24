//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
//! This file is largely inspired by https://github.com/MystenLabs/mysten-infra/blob/main/crates/mysten-network/src/client.rs, commit #0f0f01b87f2c8ebbfdbab575070d4c5abfbaa7f8
use crate::{
    config::server::ServerConfig,
    multiaddr::{parse_dns, parse_ip4, parse_ip6},
};
use anyhow::{anyhow, Context, Result};
use multiaddr::{Multiaddr, Protocol};
use tonic::transport::{Channel, Endpoint, Uri};

pub async fn connect(address: &Multiaddr) -> Result<Channel> {
    let channel = endpoint_from_multiaddr(address)?.connect().await?;
    Ok(channel)
}

pub fn connect_lazy(address: &Multiaddr) -> Result<Channel> {
    let channel = endpoint_from_multiaddr(address)?.connect_lazy();
    Ok(channel)
}

pub(crate) async fn connect_with_config(address: &Multiaddr, config: &ServerConfig) -> Result<Channel> {
    let channel = endpoint_from_multiaddr(address)?.apply_config(config).connect().await?;
    Ok(channel)
}

pub(crate) fn connect_lazy_with_config(address: &Multiaddr, config: &ServerConfig) -> Result<Channel> {
    let channel = endpoint_from_multiaddr(address)?.apply_config(config).connect_lazy();
    Ok(channel)
}

pub fn endpoint_from_multiaddr(addr: &Multiaddr) -> Result<TargetEndpoint> {
    let mut iter = addr.iter();

    let channel = match iter.next().ok_or_else(|| anyhow!("address is empty"))? {
        Protocol::Dns(..) => {
            let (dns_name, tcp_port, http_or_https) = parse_dns(addr)?;
            let uri = format!("{http_or_https}://{dns_name}:{tcp_port}");
            TargetEndpoint::try_from_uri(uri)?
        }
        Protocol::Ip4(..) => {
            let (socket_addr, http_or_https) = parse_ip4(addr)?;
            let uri = format!("{http_or_https}://{socket_addr}");
            TargetEndpoint::try_from_uri(uri)?
        }
        Protocol::Ip6(..) => {
            let (socket_addr, http_or_https) = parse_ip6(addr)?;
            let uri = format!("{http_or_https}://{socket_addr}");
            TargetEndpoint::try_from_uri(uri)?
        }
        // Protocol::Memory(..) => todo!(),
        #[cfg(unix)]
        Protocol::Unix(..) => {
            let (path, http_or_https) = crate::multiaddr::parse_unix(addr)?;
            let uri = format!("{http_or_https}://localhost");
            TargetEndpoint::try_from_uri(uri)?.with_uds_connector(path.as_ref().into())
        }
        unsupported => return Err(anyhow!("unsupported protocol {unsupported}")),
    };

    Ok(channel)
}

/// Creates a new endpoint and facilitates connectivity
pub struct TargetEndpoint {
    endpoint: Endpoint,
    #[cfg(unix)]
    uds_connector: Option<std::path::PathBuf>,
}

impl TargetEndpoint {
    fn new(endpoint: Endpoint) -> Self {
        Self {
            endpoint,
            #[cfg(unix)]
            uds_connector: None,
        }
    }

    pub fn endpoint(&self) -> &Endpoint {
        return &self.endpoint;
    }

    fn try_from_uri(uri: String) -> Result<Self> {
        let uri: Uri = uri
            .parse()
            .with_context(|| format!("unable to create Uri from '{uri}'"))?;
        let endpoint = Endpoint::from(uri);
        Ok(Self::new(endpoint))
    }

    #[cfg(unix)]
    fn with_uds_connector(self, path: std::path::PathBuf) -> Self {
        Self {
            endpoint: self.endpoint,
            uds_connector: Some(path),
        }
    }

    fn apply_config(mut self, config: &ServerConfig) -> Self {
        self.endpoint = apply_config_to_endpoint(config, self.endpoint);
        self
    }

    fn connect_lazy(self) -> Channel {
        #[cfg(unix)]
        if let Some(path) = self.uds_connector {
            return self
                .endpoint
                .connect_with_connector_lazy(tower::service_fn(move |_: Uri| {
                    let path = path.clone();

                    // Connect to a Uds socket
                    tokio::net::UnixStream::connect(path)
                }));
        }

        self.endpoint.connect_lazy()
    }

    async fn connect(self) -> Result<Channel> {
        #[cfg(unix)]
        if let Some(path) = self.uds_connector {
            return self
                .endpoint
                .connect_with_connector(tower::service_fn(move |_: Uri| {
                    let path = path.clone();

                    // Connect to a Uds socket
                    tokio::net::UnixStream::connect(path)
                }))
                .await
                .map_err(Into::into);
        }

        self.endpoint.connect().await.map_err(Into::into)
    }
}

fn apply_config_to_endpoint(config: &ServerConfig, mut endpoint: Endpoint) -> Endpoint {
    if let Some(limit) = config.concurrency_limit_per_connection {
        endpoint = endpoint.concurrency_limit(limit);
    }

    if let Some(timeout) = config.request_timeout {
        endpoint = endpoint.timeout(timeout);
    }

    if let Some(timeout) = config.connect_timeout {
        endpoint = endpoint.connect_timeout(timeout);
    }

    if let Some(tcp_nodelay) = config.tcp_nodelay {
        endpoint = endpoint.tcp_nodelay(tcp_nodelay);
    }

    if let Some(http2_keepalive_interval) = config.http2_keepalive_interval {
        endpoint = endpoint.http2_keep_alive_interval(http2_keepalive_interval);
    }

    if let Some(http2_keepalive_timeout) = config.http2_keepalive_timeout {
        endpoint = endpoint.keep_alive_timeout(http2_keepalive_timeout);
    }

    if let Some((limit, duration)) = config.rate_limit {
        endpoint = endpoint.rate_limit(limit, duration);
    }

    endpoint
        .initial_stream_window_size(config.http2_initial_stream_window_size)
        .initial_connection_window_size(config.http2_initial_connection_window_size)
        .tcp_keepalive(config.tcp_keepalive)
}

#[cfg(test)]
mod client_tests {
    use super::*;
    use crate::config::server::ServerConfig;
    use gdex_types::utils;

    // TODO - how can we make local unit tests more robust?
    #[tokio::test]
    async fn basic_functionality() {
        let address = utils::new_network_address();
        let config = ServerConfig::new();
        let _server = config.server_builder().bind(&address).await.unwrap();
        let _basic_conn = connect(&address).await.unwrap();
        let _basic_lazy_conn = connect_lazy(&address).unwrap();
        let _basic_lazy_conn_with_config = connect_lazy_with_config(&address, &config).unwrap();
    }
}
