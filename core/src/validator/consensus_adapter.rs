//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
use crate::client;
use multiaddr::Multiaddr;
use narwhal_config::Committee as ConsensusCommittee;
use narwhal_crypto::KeyPair as ConsensusKeyPair;
use std::sync::{Arc, atomic::{AtomicU64, Ordering}};
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

/// Default transaction rate at which to rotate target worker
const TRANSACTIONS_PER_WORKER: usize = 1;

/// Submit transactions to the Narwhal consensus.
pub struct ConsensusAdapter {
    /// A network client connecting to the consensus node of this authority.
    consensus_clients: Vec<Mutex<narwhal_types::TransactionsClient<tonic::transport::Channel>>>,
    // /// A transaction counter used for worker selection
    transaction_counter: AtomicU64,
    // /// The number of transactions to send per rotation
    // transactions_per_rotation: usize,
    // /// The address of consensus
    // pub consensus_addresses: Vec<Multiaddr>,
    // /// A channel to tell consensus to reconfigure.
    // pub tx_reconfigure_consensus: Sender<(ConsensusKeyPair, ConsensusCommittee)>,
}

impl ConsensusAdapter {
    pub fn new(
        consensus_addresses: Vec<Multiaddr>,
        _transactions_per_rotation: Option<usize>,
        _tx_reconfigure_consensus: Sender<(ConsensusKeyPair, ConsensusCommittee)>,
    ) -> Self {
        let consensus_clients = consensus_addresses
            .iter()
            .map(|addr| {
                let client = client::connect_lazy(addr).unwrap();
                Mutex::new(narwhal_types::TransactionsClient::new(client))
            })
            .collect();

        Self {
            consensus_clients,
            transaction_counter: AtomicU64::new(0),
            // transactions_per_rotation: transactions_per_rotation.unwrap_or(TRANSACTIONS_PER_WORKER),
            // consensus_addresses,
            // tx_reconfigure_consensus,
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
