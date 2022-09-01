//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
use crate::client;
use multiaddr::Multiaddr;
use narwhal_config::Committee as ConsensusCommittee;
use narwhal_crypto::KeyPair as ConsensusKeyPair;
use tokio::sync::mpsc::Sender;

/// Default transaction rate at which to rotate target worker
const TRANSACTIONS_PER_WORKER: usize = 50;

/// Submit transactions to the Narwhal consensus.
pub struct ConsensusAdapter {
    /// A network client connecting to the consensus node of this authority.
    consensus_clients: Vec<narwhal_types::TransactionsClient<tonic::transport::Channel>>,
    /// A transaction counter used for worker selection
    transaction_counter: usize,
    /// The number of transactions to send per rotation
    transactions_per_rotation: usize,
    /// The address of consensus
    pub consensus_addresses: Vec<Multiaddr>,
    /// A channel to tell consensus to reconfigure.
    pub tx_reconfigure_consensus: Sender<(ConsensusKeyPair, ConsensusCommittee)>,
}

impl ConsensusAdapter {
    pub fn new(
        consensus_addresses: Vec<Multiaddr>,
        transactions_per_rotation: Option<usize>,
        tx_reconfigure_consensus: Sender<(ConsensusKeyPair, ConsensusCommittee)>,
    ) -> Self {
        let consensus_clients = consensus_addresses
            .iter()
            .map(|addr| {
                let client = client::connect_lazy(addr).unwrap();
                narwhal_types::TransactionsClient::new(client)
            })
            .collect();

        Self {
            consensus_clients,
            transaction_counter: 0,
            transactions_per_rotation: transactions_per_rotation.unwrap_or(TRANSACTIONS_PER_WORKER),
            consensus_addresses,
            tx_reconfigure_consensus,
        }
    }

    pub async fn submit_transaction(
        &mut self,
        transaction_proto: narwhal_types::TransactionProto,
    ) -> Result<tonic::Response<narwhal_types::Empty>, tonic::Status> {
        let worker_index = (self.transaction_counter / self.transactions_per_rotation) % self.consensus_clients.len();
        self.transaction_counter += 1;
        self.consensus_clients
            .get_mut(worker_index) //worker_index)
            // safe to unwrap as the client counter is bounded by number of clients
            .unwrap()
            .submit_transaction(transaction_proto)
            .await
    }
}
