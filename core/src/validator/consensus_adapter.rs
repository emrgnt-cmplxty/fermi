//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
use crate::client;
use multiaddr::Multiaddr;
use narwhal_config::Committee as ConsensusCommittee;
use narwhal_crypto::KeyPair as ConsensusKeyPair;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use tokio::sync::{mpsc::Sender, Mutex};
/// TODO - Add batch timer to force batches to be sent after a certain amount of time
/// TODO - move timer and batch size to a config file which specifies gdex params
const BATCH_SIZE: usize = 100;

/// Submit transactions to the Narwhal consensus.
pub struct ConsensusAdapter {
    /// A network client connecting to the consensus node of this authority.
    consensus_clients: Vec<Mutex<narwhal_types::TransactionsClient<tonic::transport::Channel>>>,
    // /// A transaction counter used for worker selection
    batch_counter: AtomicU64,
    // /// The address of consensus
    pub consensus_addresses: Vec<Multiaddr>,
    // /// A channel to tell consensus to reconfigure.
    pub tx_reconfigure_consensus: Sender<(ConsensusKeyPair, ConsensusCommittee)>,
    batch_size: AtomicUsize,
    submitted_transactions: Mutex<Vec<narwhal_types::TransactionProto>>,
}

impl ConsensusAdapter {
    pub fn new(
        consensus_addresses: Vec<Multiaddr>,
        tx_reconfigure_consensus: Sender<(ConsensusKeyPair, ConsensusCommittee)>,
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
            batch_counter: AtomicU64::new(0),
            consensus_addresses,
            tx_reconfigure_consensus,
            batch_size: AtomicUsize::new(BATCH_SIZE),
            submitted_transactions: Mutex::new(Vec::new()),
        }
    }

    pub fn update_batch_size(&self, batch_size: usize) {
        self.batch_size.store(batch_size, Ordering::SeqCst);
    }

    pub async fn submit_transaction(
        &self,
        transaction_proto: narwhal_types::TransactionProto,
    ) -> Result<(), tonic::Status> {
        let mut transaction_buffer_lock = self.submitted_transactions.lock().await;
        transaction_buffer_lock.push(transaction_proto);

        if transaction_buffer_lock.len() >= self.batch_size.load(Ordering::SeqCst) {
            let worker_index = self.batch_counter.load(Ordering::SeqCst) % (self.consensus_clients.len() as u64);
            let transactions_copy = tokio_stream::iter(transaction_buffer_lock.clone());
            *transaction_buffer_lock = Vec::new();
            // drop the transaction buffer so that other threads do not remain blocked while we submit our batch
            drop(transaction_buffer_lock);
            // increment the batch counter so that next batch is sent to a different worker
            self.batch_counter.fetch_add(1, Ordering::SeqCst);

            self.consensus_clients
                .get(worker_index as usize) //worker_index)
                // safe to unwrap as the client counter is bounded by number of clients
                .unwrap()
                .lock()
                .await
                .submit_transaction_stream(transactions_copy)
                .await?;
        }
        Ok(())
    }
}
