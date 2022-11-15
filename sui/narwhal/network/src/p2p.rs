// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::anemo_ext::NetworkExt;
use crate::traits::{PrimaryToPrimaryRpc, PrimaryToWorkerRpc, WorkerRpc};
use crate::{
    traits::{Lucky, ReliableNetwork, UnreliableNetwork},
    BoundedExecutor, CancelOnDropHandler, RetryConfig, MAX_TASK_CONCURRENCY,
};
use anemo::PeerId;
use anyhow::format_err;
use anyhow::Result;
use async_trait::async_trait;
use crypto::{traits::KeyPair, NetworkPublicKey};
use rand::{rngs::SmallRng, SeedableRng as _};
use std::collections::HashMap;
use std::time::Duration;
use tokio::{runtime::Handle, task::JoinHandle};
use types::{
    Batch, BatchDigest, FetchCertificatesRequest, FetchCertificatesResponse,
    GetCertificatesRequest, GetCertificatesResponse, LatestHeaderRequest, LatestHeaderResponse,
    PrimaryMessage, PrimaryToPrimaryClient, PrimaryToWorkerClient, RequestBatchRequest,
    WorkerBatchMessage, WorkerDeleteBatchesMessage, WorkerOthersBatchMessage,
    WorkerOurBatchMessage, WorkerReconfigureMessage, WorkerSynchronizeMessage,
    WorkerToPrimaryClient, WorkerToWorkerClient,
};

fn default_executor() -> BoundedExecutor {
    BoundedExecutor::new(MAX_TASK_CONCURRENCY, Handle::current())
}

pub struct P2pNetwork {
    network: anemo::Network,
    retry_config: RetryConfig,
    /// Small RNG just used to shuffle nodes and randomize connections (not crypto related).
    rng: SmallRng,
    // One bounded executor per address
    executors: HashMap<PeerId, BoundedExecutor>,
}

impl P2pNetwork {
    pub fn new(network: anemo::Network) -> Self {
        let retry_config = RetryConfig {
            // Retry forever
            retrying_max_elapsed_time: None,
            ..Default::default()
        };

        Self {
            network,
            retry_config,
            rng: SmallRng::from_entropy(),
            executors: HashMap::new(),
        }
    }

    // Creates a new single-use anemo::Network to connect outbound to a single
    // address. This is for tests and should not be used from worker code.
    pub async fn new_for_single_address(
        name: NetworkPublicKey,
        address: anemo::types::Address,
    ) -> Self {
        let routes = anemo::Router::new();
        let network = anemo::Network::bind("127.0.0.1:0")
            .server_name("narwhal")
            .private_key(
                crypto::NetworkKeyPair::generate(&mut rand::thread_rng())
                    .private()
                    .0
                    .to_bytes(),
            )
            .start(routes)
            .unwrap();
        network
            .connect_with_peer_id(address, PeerId(name.0.to_bytes()))
            .await
            .unwrap();
        Self::new(network)
    }

    pub fn network(&self) -> anemo::Network {
        self.network.clone()
    }

    fn unreliable_send<F, R, Fut>(
        &mut self,
        peer: NetworkPublicKey,
        f: F,
    ) -> Result<JoinHandle<Result<anemo::Response<R>>>>
    where
        F: FnOnce(anemo::Peer) -> Fut + Send + Sync + 'static,
        R: Send + Sync + 'static + Clone,
        Fut: std::future::Future<Output = Result<anemo::Response<R>, anemo::rpc::Status>> + Send,
    {
        let peer_id = PeerId(peer.0.to_bytes());
        let peer = self.network.peer(peer_id).ok_or_else(|| {
            anemo::Error::msg(format!("Network has no connection with peer {peer_id}"))
        })?;

        self.executors
            .entry(peer_id)
            .or_insert_with(default_executor)
            .try_spawn(async move {
                f(peer)
                    .await
                    .map_err(|e| anyhow::anyhow!("RPC error: {e:?}"))
            })
            .map_err(|e| anemo::Error::msg(e.to_string()))
    }

    // TODO: remove async in a cleanup, this doesn't need it anymore.
    async fn send<F, R, Fut>(
        &mut self,
        peer: NetworkPublicKey,
        f: F,
    ) -> CancelOnDropHandler<Result<anemo::Response<R>>>
    where
        F: Fn(anemo::Peer) -> Fut + Send + Sync + 'static + Clone,
        R: Send + Sync + 'static + Clone,
        Fut: std::future::Future<Output = Result<anemo::Response<R>, anemo::rpc::Status>> + Send,
    {
        // Safety
        // Since this spawns an unbounded task, this should be called in a time-restricted fashion.
        // Here the callers are [`PrimaryNetwork::broadcast`] and [`PrimaryNetwork::send`],
        // at respectively N and K calls per round.
        //  (where N is the number of primaries, K the number of workers for this primary)
        // See the TODO on spawn_with_retries for lifting this restriction.

        let network = self.network.clone();
        let peer_id = PeerId(peer.0.to_bytes());
        let message_send = move || {
            let network = network.clone();
            let f = f.clone();

            async move {
                if let Some(peer) = network.peer(peer_id) {
                    f(peer).await.map_err(|e| {
                        // this returns a backoff::Error::Transient
                        // so that if anemo::Status is returned, we retry
                        backoff::Error::transient(anyhow::anyhow!("RPC error: {e:?}"))
                    })
                } else {
                    Err(backoff::Error::transient(anyhow::anyhow!(
                        "not connected to peer {peer_id}"
                    )))
                }
            }
        };

        let handle = self
            .executors
            .entry(peer_id)
            .or_insert_with(default_executor)
            .spawn_with_retries(self.retry_config, message_send);

        CancelOnDropHandler(handle)
    }
}

impl Lucky for P2pNetwork {
    fn rng(&mut self) -> &mut SmallRng {
        &mut self.rng
    }
}

//
// Primary-to-Primary
//

impl UnreliableNetwork<PrimaryMessage> for P2pNetwork {
    type Response = ();
    fn unreliable_send(
        &mut self,
        peer: NetworkPublicKey,
        message: &PrimaryMessage,
    ) -> Result<JoinHandle<Result<anemo::Response<()>>>> {
        let message = message.to_owned();
        let f = move |peer| async move {
            PrimaryToPrimaryClient::new(peer)
                .send_message(message)
                .await
        };
        self.unreliable_send(peer, f)
    }
}

#[async_trait]
impl ReliableNetwork<PrimaryMessage> for P2pNetwork {
    type Response = ();
    async fn send(
        &mut self,
        peer: NetworkPublicKey,
        message: &PrimaryMessage,
    ) -> CancelOnDropHandler<Result<anemo::Response<()>>> {
        let message = message.to_owned();
        let f = move |peer| {
            let message = message.clone();
            async move {
                PrimaryToPrimaryClient::new(peer)
                    .send_message(message)
                    .await
            }
        };

        self.send(peer, f).await
    }
}

#[async_trait]
impl PrimaryToPrimaryRpc for anemo::Network {
    async fn get_certificates(
        &self,
        peer: &NetworkPublicKey,
        request: impl anemo::types::request::IntoRequest<GetCertificatesRequest> + Send,
    ) -> Result<GetCertificatesResponse> {
        let peer_id = PeerId(peer.0.to_bytes());
        let peer = self
            .peer(peer_id)
            .ok_or_else(|| format_err!("Network has no connection with peer {peer_id}"))?;
        let response = PrimaryToPrimaryClient::new(peer)
            .get_certificates(request)
            .await
            .map_err(|e| format_err!("Network error {:?}", e))?;
        Ok(response.into_body())
    }
    async fn fetch_certificates(
        &self,
        peer: &NetworkPublicKey,
        request: FetchCertificatesRequest,
    ) -> Result<FetchCertificatesResponse> {
        let peer_id = PeerId(peer.0.to_bytes());
        let peer = self
            .peer(peer_id)
            .ok_or_else(|| format_err!("Network has no connection with peer {peer_id}"))?;
        let response = PrimaryToPrimaryClient::new(peer)
            .fetch_certificates(request)
            .await
            .map_err(|e| format_err!("Network error {:?}", e))?;
        Ok(response.into_body())
    }

    async fn get_latest_header(
        &self,
        peer: &NetworkPublicKey,
        request: LatestHeaderRequest,
    ) -> Result<LatestHeaderResponse> {
        const LATEST_HEADER_REQUEST_TIMEOUT: Duration = Duration::from_secs(15);
        let request = anemo::Request::new(request).with_timeout(LATEST_HEADER_REQUEST_TIMEOUT);
        let peer_id = PeerId(peer.0.to_bytes());
        let peer = self.waiting_peer(peer_id);
        let response = PrimaryToPrimaryClient::new(peer)
            .get_latest_header(request)
            .await
            .map_err(|e| format_err!("Network error {:?}", e))?;
        Ok(response.into_body())
    }
}

//
// Primary-to-Worker
//

impl UnreliableNetwork<WorkerReconfigureMessage> for P2pNetwork {
    type Response = ();
    fn unreliable_send(
        &mut self,
        peer: NetworkPublicKey,
        message: &WorkerReconfigureMessage,
    ) -> Result<JoinHandle<Result<anemo::Response<()>>>> {
        let message = message.to_owned();
        let f =
            move |peer| async move { PrimaryToWorkerClient::new(peer).reconfigure(message).await };
        self.unreliable_send(peer, f)
    }
}
#[async_trait]
impl ReliableNetwork<WorkerReconfigureMessage> for P2pNetwork {
    type Response = ();
    async fn send(
        &mut self,
        peer: NetworkPublicKey,
        message: &WorkerReconfigureMessage,
    ) -> CancelOnDropHandler<Result<anemo::Response<()>>> {
        let message = message.to_owned();
        let f = move |peer| {
            let message = message.clone();
            async move { PrimaryToWorkerClient::new(peer).reconfigure(message).await }
        };

        self.send(peer, f).await
    }
}

impl UnreliableNetwork<WorkerSynchronizeMessage> for P2pNetwork {
    type Response = ();
    fn unreliable_send(
        &mut self,
        peer: NetworkPublicKey,
        message: &WorkerSynchronizeMessage,
    ) -> Result<JoinHandle<Result<anemo::Response<()>>>> {
        let message = message.to_owned();
        let f = move |peer| async move {
            // Set a timeout on unreliable sends of synchronize, so it doesn't run forever.
            const UNRELIABLE_SYNCHRONIZE_TIMEOUT: Duration = Duration::from_secs(30);
            PrimaryToWorkerClient::new(peer)
                .synchronize(
                    anemo::Request::new(message).with_timeout(UNRELIABLE_SYNCHRONIZE_TIMEOUT),
                )
                .await
        };
        self.unreliable_send(peer, f)
    }
}

#[async_trait]
impl ReliableNetwork<WorkerSynchronizeMessage> for P2pNetwork {
    type Response = ();
    async fn send(
        &mut self,
        peer: NetworkPublicKey,
        message: &WorkerSynchronizeMessage,
    ) -> CancelOnDropHandler<Result<anemo::Response<()>>> {
        let message = message.to_owned();
        let f = move |peer| {
            let message = message.clone();
            async move { PrimaryToWorkerClient::new(peer).synchronize(message).await }
        };

        self.send(peer, f).await
    }
}

//
// Worker-to-Primary
//

#[async_trait]
impl ReliableNetwork<WorkerOurBatchMessage> for P2pNetwork {
    type Response = ();
    async fn send(
        &mut self,
        peer: NetworkPublicKey,
        message: &WorkerOurBatchMessage,
    ) -> CancelOnDropHandler<Result<anemo::Response<()>>> {
        let message = message.to_owned();
        let f = move |peer| {
            let message = message.clone();
            async move {
                WorkerToPrimaryClient::new(peer)
                    .report_our_batch(message)
                    .await
            }
        };

        self.send(peer, f).await
    }
}

#[async_trait]
impl ReliableNetwork<WorkerOthersBatchMessage> for P2pNetwork {
    type Response = ();
    async fn send(
        &mut self,
        peer: NetworkPublicKey,
        message: &WorkerOthersBatchMessage,
    ) -> CancelOnDropHandler<Result<anemo::Response<()>>> {
        let message = message.to_owned();
        let f = move |peer| {
            let message = message.clone();
            async move {
                WorkerToPrimaryClient::new(peer)
                    .report_others_batch(message)
                    .await
            }
        };

        self.send(peer, f).await
    }
}

//
// Worker-to-Worker
//

impl UnreliableNetwork<WorkerBatchMessage> for P2pNetwork {
    type Response = ();
    fn unreliable_send(
        &mut self,
        peer: NetworkPublicKey,
        message: &WorkerBatchMessage,
    ) -> Result<JoinHandle<Result<anemo::Response<()>>>> {
        let message = message.to_owned();
        let f =
            move |peer| async move { WorkerToWorkerClient::new(peer).report_batch(message).await };
        self.unreliable_send(peer, f)
    }
}

#[async_trait]
impl ReliableNetwork<WorkerBatchMessage> for P2pNetwork {
    type Response = ();
    async fn send(
        &mut self,
        peer: NetworkPublicKey,
        message: &WorkerBatchMessage,
    ) -> CancelOnDropHandler<Result<anemo::Response<()>>> {
        let message = message.to_owned();
        let f = move |peer| {
            let message = message.clone();
            async move { WorkerToWorkerClient::new(peer).report_batch(message).await }
        };

        self.send(peer, f).await
    }
}

#[async_trait]
impl PrimaryToWorkerRpc for P2pNetwork {
    async fn delete_batches(
        &self,
        peer: NetworkPublicKey,
        digests: Vec<BatchDigest>,
    ) -> Result<()> {
        const BATCH_DELETE_TIMEOUT: Duration = Duration::from_secs(2);

        let peer_id = PeerId(peer.0.to_bytes());
        let peer = self
            .network
            .peer(peer_id)
            .ok_or_else(|| format_err!("Network has no connection with peer {peer_id}"))?;
        let request = anemo::Request::new(WorkerDeleteBatchesMessage { digests })
            .with_timeout(BATCH_DELETE_TIMEOUT);
        PrimaryToWorkerClient::new(peer)
            .delete_batches(request)
            .await
            .map(|_| ())
            .map_err(|e| format_err!("DeleteBatches error: {e:?}"))
    }
}

#[async_trait]
impl WorkerRpc for P2pNetwork {
    async fn request_batch(
        &self,
        peer: NetworkPublicKey,
        batch: BatchDigest,
    ) -> Result<Option<Batch>> {
        const BATCH_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

        let peer_id = PeerId(peer.0.to_bytes());
        let peer = self
            .network
            .peer(peer_id)
            .ok_or_else(|| format_err!("Network has no connection with peer {peer_id}"))?;
        let request =
            anemo::Request::new(RequestBatchRequest { batch }).with_timeout(BATCH_REQUEST_TIMEOUT);
        let response = WorkerToWorkerClient::new(peer)
            .request_batch(request)
            .await
            .map_err(|e| format_err!("Network error {:?}", e))?;
        Ok(response.into_body().batch)
    }
}
