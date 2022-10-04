// IMPORTS

// crate
use crate::client::endpoint_from_multiaddr;

// local
use gdex_controller::router::ControllerRouter;
use gdex_types::block::{Block, BlockInfo};
use gdex_types::proto::{BlockRequest, LatestBlockInfoRequest, ValidatorGrpcClient};
use gdex_types::store::RPCStoreHandle;

use gdex_types::transaction::SignedTransaction;
// external
use multiaddr::Multiaddr;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};
use tokio::task::JoinHandle;
use tracing::info;

// The BlockProcessor is responsible for listening for new blocks from the
// ValidatorGRPC and updating the JSON RPC state accordingly
pub struct BlockProcessor {
    controller_router: Arc<Mutex<ControllerRouter>>,
    last_block_number: Arc<AtomicU64>,
    grpc_addr: Multiaddr,
    rpc_store_handle: Arc<RPCStoreHandle>,
}

impl BlockProcessor {
    pub fn spawn(
        controller_router: Arc<Mutex<ControllerRouter>>,
        last_block_number: Arc<AtomicU64>,
        rpc_store_handle: Arc<RPCStoreHandle>,
        grpc_addr: Multiaddr,
    ) -> Vec<JoinHandle<()>> {
        let controller_router_clone = Arc::clone(&controller_router);
        let last_block_number_clone = Arc::clone(&last_block_number);

        let block_listener_handle = tokio::spawn(async move {
            BlockProcessor {
                controller_router: controller_router_clone,
                last_block_number: last_block_number_clone,
                grpc_addr,
                rpc_store_handle,
            }
            .run()
            .await
        });

        vec![block_listener_handle]
    }

    pub async fn run(&self) {
        // initialize grpc_client
        let grpc_endpoint = endpoint_from_multiaddr(&self.grpc_addr).unwrap();
        let mut grpc_client = ValidatorGrpcClient::connect(grpc_endpoint.endpoint().clone())
            .await
            .unwrap();

        loop {
            let latest_block_info_request = LatestBlockInfoRequest {};
            let latest_block_info_response = grpc_client.get_latest_block_info(latest_block_info_request).await;

            let latest_block_number: u64;
            match latest_block_info_response {
                Ok(response) => {
                    let latest_block_payload = response.into_inner();
                    if !latest_block_payload.successful {
                        info!("Latest block info request returned unsuccessful. Retrying");
                        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                        continue;
                    }
                    let latest_block_info: BlockInfo =
                        bincode::deserialize(&latest_block_payload.serialized_block_info).unwrap();
                    latest_block_number = latest_block_info.block_number;
                }
                Err(_) => {
                    info!("Failed to fetch latest block info. Retrying");
                    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                    continue;
                }
            }

            if latest_block_number <= self.last_block_number.load(Ordering::SeqCst) {
                info!("No new blocks. Waiting and checking again");
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                continue;
            }

            while latest_block_number > self.last_block_number.load(Ordering::SeqCst) {
                let target_block_number: u64 = self.last_block_number.load(Ordering::SeqCst) + 1;
                let get_block_request = BlockRequest {
                    block_number: target_block_number,
                };

                let block_response = grpc_client.get_block(get_block_request).await;
                match block_response {
                    Ok(response) => {
                        let block_payload = response.into_inner();
                        if !block_payload.successful {
                            info!(
                                "Request for block with block number: {}, returned unsuccessful. Resetting",
                                target_block_number
                            );
                            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                            continue;
                        }
                        info!("Processing transactions for block {}.", target_block_number);
                        let block: Block = bincode::deserialize(&block_payload.serialized_block).unwrap();

                        for executed_transaction in &block.transactions {
                            let signed_transaction: &SignedTransaction = &executed_transaction.signed_transaction;
                            if let Ok(transaction) = signed_transaction.get_transaction() {
                                let _result = self
                                    .controller_router
                                    .lock()
                                    .unwrap()
                                    .handle_consensus_transaction(transaction);
                            }
                        }
                        self.last_block_number.store(target_block_number, Ordering::SeqCst);
                        self.controller_router
                            .lock()
                            .unwrap()
                            .non_critical_process_end_of_block(&self.rpc_store_handle.rpc_store, target_block_number)
                            .unwrap();
                    }
                    Err(_) => {
                        info!(
                            "Failed to fetch block with block number: {}. Resetting",
                            target_block_number
                        );
                        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test_json_rpc_listener {
    use super::*;
    // use crate::relayer::spawner::RelayerSpawner;
    use crate::config::JSONRPC_DB_NAME;
    use crate::validator::post_processor::ValidatorPostProcessor;
    use crate::validator::server::ValidatorServer;
    use crate::validator::state::test_validator_state::get_test_validator_state;
    use crate::validator::{server::HandledTransaction, state::ValidatorState};
    use gdex_types::store::RPCStore;
    use gdex_types::utils;
    use gdex_types::{account::AccountKeyPair, crypto::KeypairTraits, transaction::SignedTransaction};
    // mysten
    use fastcrypto::DIGEST_LEN;
    use narwhal_types::{Certificate, CertificateDigest};

    use super::BlockProcessor;
    use gdex_controller::bank::proto::create_payment_transaction;
    use narwhal_consensus::ConsensusOutput;
    use narwhal_executor::SerializedTransaction;
    use narwhal_executor::{ExecutionIndices, ExecutionState};
    use rand::{rngs::StdRng, SeedableRng};
    use std::sync::atomic::AtomicU64;
    use tokio::sync::mpsc;
    use tokio::time::{sleep, Duration};

    pub fn keys(seed: [u8; 32]) -> Vec<AccountKeyPair> {
        let mut rng = StdRng::from_seed(seed);
        (0..4).map(|_| AccountKeyPair::generate(&mut rng)).collect()
    }

    fn get_signed_transaction(sender_seed: [u8; 32], receiver_seed: [u8; 32], amount: u64) -> SignedTransaction {
        let kp_sender = keys(sender_seed).pop().unwrap();
        let kp_receiver = keys(receiver_seed).pop().unwrap();
        let certificate_digest = CertificateDigest::new([0; DIGEST_LEN]);
        let transaction =
            create_payment_transaction(kp_sender.public(), certificate_digest, kp_receiver.public(), 0, amount);
        transaction.sign(&kp_sender).unwrap()
    }

    // A test function that ticks a block by injecting a "final transaction" into the handle_consensus_tranasction workflow
    async fn tick_block(
        validator_state: Arc<ValidatorState>,
        tx_narwhal_to_post_process: &mpsc::Sender<(HandledTransaction, SerializedTransaction)>,
    ) {
        let signed_transaction = get_signed_transaction([0; 32], [1; 32], 100);
        let dummy_output = ConsensusOutput {
            certificate: Certificate::default(),
            consensus_index: 1,
        };
        let dummy_transaction_index = ExecutionIndices {
            /// The index of the latest consensus message we processed (used for crash-recovery).
            next_certificate_index: 0,
            /// The index of the last batch we executed (used for crash-recovery).
            next_batch_index: 0,
            /// The index of the last transaction we executed (used for crash-recovery).
            next_transaction_index: 0,
        };

        // tick a block!
        let result = validator_state
            .handle_consensus_transaction(
                &dummy_output,
                dummy_transaction_index.clone(),
                signed_transaction.clone(),
            )
            .await
            .unwrap();

        let serialized_transaction = bincode::serialize(&signed_transaction).unwrap();
        tx_narwhal_to_post_process
            .send((Ok(result), serialized_transaction))
            .await
            .unwrap();
    }

    #[ignore]
    #[tokio::test]
    async fn test_block_listener() {
        let temp_dir = tempfile::tempdir().unwrap();
        let working_dir = temp_dir.path();
        let json_rpc_db_dir = working_dir.join(JSONRPC_DB_NAME);

        let validator_state = Arc::new(get_test_validator_state());
        let relayer_address = utils::new_network_address();

        // let mut relayer_spawner = RelayerSpawner::new(Arc::clone(&validator_state), relayer_address.clone());
        // relayer_spawner.spawn_relayer().await.unwrap();

        let (tx_reconfigure_consensus, _rx_reconfigure_consensus) = mpsc::channel(1);
        let validator_server = ValidatorServer::new(
            relayer_address.clone(),
            Arc::clone(&validator_state),
            vec![utils::new_network_address()],
            tx_reconfigure_consensus,
        );

        validator_server.spawn().await.unwrap();

        let listener_router = Arc::new(Mutex::new(ControllerRouter::default()));
        let last_block_number = Arc::new(AtomicU64::new(0));

        let (tx_narwhal_to_post_process, rx_narwhal_to_post_process) = mpsc::channel(1_000);
        let _post_process_handles =
            ValidatorPostProcessor::spawn(rx_narwhal_to_post_process, Arc::clone(&validator_state)).unwrap();

        let rpc_store_handle = Arc::new(RPCStoreHandle {
            rpc_store: RPCStore::reopen(json_rpc_db_dir),
        });

        let _block_listener_handle = BlockProcessor::spawn(
            listener_router,
            Arc::clone(&last_block_number),
            rpc_store_handle,
            relayer_address,
        );

        let n_blocks = 200;
        for _ in 0..n_blocks {
            tick_block(Arc::clone(&validator_state), &tx_narwhal_to_post_process).await;
            sleep(Duration::from_millis(10)).await;
        }
        sleep(Duration::from_millis(10000)).await;
        println!(
            "validator_state.validator_store.block_number.load(Ordering::SeqCst)= {}",
            validator_state.validator_store.block_number.load(Ordering::SeqCst)
        );
        println!(
            "last_block_number.load(Ordering::SeqCst)= {}",
            last_block_number.load(Ordering::SeqCst)
        );
        assert!(validator_state.validator_store.block_number.load(Ordering::SeqCst) == n_blocks);
        assert!(last_block_number.load(Ordering::SeqCst) == n_blocks);
    }
}
