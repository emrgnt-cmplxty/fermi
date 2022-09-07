//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
use crate::client;
use multiaddr::Multiaddr;
use narwhal_config::Committee as ConsensusCommittee;
use narwhal_crypto::KeyPair as ConsensusKeyPair;
use std::sync::{
    atomic::{AtomicU64, Ordering},
};
use tokio::sync::{Mutex, mpsc::Sender};
/// Default transaction rate at which to rotate target worker
const CLIENTS_PER_WORKER: usize = 4;

/// Submit transactions to the Narwhal consensus.
pub struct ConsensusAdapter {
    /// A network client connecting to the consensus node of this authority.
    consensus_clients: Vec<Mutex<narwhal_types::TransactionsClient<tonic::transport::Channel>>>,
    // /// A transaction counter used for worker selection
    transaction_counter: AtomicU64,
    // /// The address of consensus
    pub consensus_addresses: Vec<Multiaddr>,
    // /// A channel to tell consensus to reconfigure.
    pub tx_reconfigure_consensus: Sender<(ConsensusKeyPair, ConsensusCommittee)>,
}

impl ConsensusAdapter {
    pub fn new(
        consensus_addresses: Vec<Multiaddr>,
        tx_reconfigure_consensus: Sender<(ConsensusKeyPair, ConsensusCommittee)>,
    ) -> Self {
        let mut consensus_clients: Vec<Mutex<narwhal_types::TransactionsClient<tonic::transport::Channel>>> =
            Vec::new();
        for _i in 0..CLIENTS_PER_WORKER {
            let iter_clients: Vec<Mutex<narwhal_types::TransactionsClient<tonic::transport::Channel>>> =
                consensus_addresses
                    .iter()
                    .map(|addr| {
                        let client = client::connect_lazy(addr).unwrap();
                        Mutex::new(narwhal_types::TransactionsClient::new(client))
                    })
                    .collect();
            consensus_clients.extend(iter_clients);
        }

        Self {
            consensus_clients,
            transaction_counter: AtomicU64::new(0),
            consensus_addresses,
            tx_reconfigure_consensus,
        }
    }

    pub async fn submit_transaction(
        &self,
        transaction_proto: narwhal_types::TransactionProto,
    ) -> Result<tonic::Response<narwhal_types::Empty>, tonic::Status> {
        let worker_index = self.transaction_counter.load(Ordering::SeqCst) % (self.consensus_clients.len() as u64);
        self.transaction_counter.fetch_add(1, Ordering::SeqCst);

        self.consensus_clients
            .get(worker_index as usize) //worker_index)
            // safe to unwrap as the client counter is bounded by number of clients
            .unwrap()
            .lock()
            .await
            .submit_transaction(transaction_proto)
            .await
    }
}
