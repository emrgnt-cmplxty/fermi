// narwhal imports
use fastcrypto::traits::KeyPair as _;
use narwhal_config::{Committee, Parameters, SharedWorkerCache};
use narwhal_crypto::KeyPair;
use narwhal_executor::{ExecutionState, ExecutorOutput};
use narwhal_network::{PrimaryToWorkerNetwork, ReliableNetwork, UnreliableNetwork, WorkerToPrimaryNetwork};
use narwhal_node::{Node, NodeStorage};
use narwhal_types::{PrimaryWorkerMessage, ReconfigureNotification, WorkerPrimaryMessage};

// external imports
use arc_swap::ArcSwap;
use futures::future::join_all;
use prometheus::Registry;
use std::{fmt::Debug, path::PathBuf, sync::Arc};
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::info;

/// Module to start a node (primary, workers and default consensus), keep it running, and restarting it
/// every time the committee changes.
pub struct NodeRestarter;

#[allow(clippy::too_many_arguments)]
impl NodeRestarter {
    pub async fn watch<State>(
        keypair: KeyPair,
        committee: &Committee,
        worker_cache: SharedWorkerCache,
        storage_base_path: PathBuf,
        execution_state: Arc<State>,
        parameters: Parameters,
        mut rx_reconfigure: Receiver<(KeyPair, Committee)>,
        tx_output: Sender<ExecutorOutput<State>>,
        registry: &Registry,
    ) where
        State: ExecutionState + Send + Sync + 'static,
        State::Outcome: Send + 'static,
        State::Error: Debug,
    {
        let mut keypair = keypair;
        let mut name = keypair.public().clone();
        let mut committee = committee.clone();

        let mut handles = Vec::new();
        let mut primary_network = WorkerToPrimaryNetwork::default();
        let mut worker_network = PrimaryToWorkerNetwork::default();
        info!("creating new node with committee={:?}", committee);
        // Listen for new committees.
        loop {
            tracing::info!("Starting epoch E{}", committee.epoch());

            // Get a fresh store for the new epoch.
            let mut store_path = storage_base_path.clone();
            store_path.push(format!("epoch{}", committee.epoch()));
            let store = NodeStorage::reopen(store_path);

            // Restart the relevant components.
            let primary = Node::spawn_primary(
                keypair,
                Arc::new(ArcSwap::new(Arc::new(committee.clone()))),
                worker_cache.clone(),
                &store,
                parameters.clone(),
                /* consensus */ true,
                execution_state.clone(),
                tx_output.clone(),
                registry,
            )
            .await
            .unwrap();

            let worker_ids = worker_cache
                .load()
                .workers
                .iter()
                .find(|(k, _)| *k == &name)
                .unwrap()
                .1
                 .0
                .keys()
                .cloned()
                .collect::<Vec<u32>>();

            let workers = Node::spawn_workers(
                name.clone(),
                worker_ids,
                Arc::new(ArcSwap::new(Arc::new(committee.clone()))),
                worker_cache.clone(),
                &store,
                parameters.clone(),
                registry,
            );

            handles.extend(primary);
            handles.extend(workers);

            // Wait for a committee change.
            let (new_keypair, new_committee) = match rx_reconfigure.recv().await {
                Some(x) => x,
                None => break,
            };
            tracing::info!("Starting reconfiguration with committee {committee}");

            // Shutdown all relevant components.
            let address = committee
                .primary(&name)
                .expect("Our key is not in the committee")
                .worker_to_primary;
            let message = WorkerPrimaryMessage::Reconfigure(ReconfigureNotification::Shutdown);
            let primary_cancel_handle = primary_network.send(address, &message).await;

            let addresses = worker_cache
                .load()
                .our_workers(&name)
                .expect("Our key is not in the committee")
                .into_iter()
                .map(|x| x.primary_to_worker)
                .collect();
            let message = PrimaryWorkerMessage::Reconfigure(ReconfigureNotification::Shutdown);
            let worker_cancel_handles = worker_network.unreliable_broadcast(addresses, &message).await;

            // Ensure the message has been received.
            primary_cancel_handle.await.expect("Failed to notify primary");
            join_all(worker_cancel_handles).await;
            tracing::debug!("Committee reconfiguration message successfully sent");

            // Cleanup the network.
            worker_network.cleanup(committee.network_diff(&new_committee));

            // Wait for the components to shut down.
            join_all(handles.drain(..)).await;
            tracing::debug!("All tasks successfully exited");

            // Give it an extra second in case the last task to exit is a network server. The OS
            // may need a moment to make the TCP ports available again.
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            tracing::debug!("Epoch E{} terminated", committee.epoch());

            // Update the settings for the next epoch.
            keypair = new_keypair;
            name = keypair.public().clone();
            committee = new_committee;
        }
    }
}