// IMPORTS

// local
use crate::validator::state::ValidatorState;

// gdex
use gdex_types::{error::GDEXError, transaction::ConsensusTransaction};

// external
use anyhow::Result;
use narwhal_consensus::ConsensusOutput;
use narwhal_executor::{ExecutionIndices, SerializedTransaction, SubscriberError};
use std::sync::Arc;
use tokio::{
    sync::mpsc::{channel, Receiver, Sender},
    task::JoinHandle,
};
use tracing::{error, info};

// INTERFACE
type ExecutionResult = Result<(), GDEXError>;
type HandledTransaction = Result<(ConsensusOutput, ExecutionIndices, ExecutionResult), SubscriberError>;

pub struct PostProcessService {}

impl PostProcessService {
    pub fn spawn(
        rx_narwhal_to_post_process: Receiver<(HandledTransaction, SerializedTransaction)>,
        validator_state: Arc<ValidatorState>,
    ) -> anyhow::Result<Vec<JoinHandle<()>>> {
        let (tx_pp_to_catchup, rx_pp_to_catchup) = channel::<u64>(1_000);
        let validator_state_clone = validator_state.clone();

        let transaction_processor_handle =
            TransactionProcessor::spawn(rx_narwhal_to_post_process, validator_state, tx_pp_to_catchup);
        let block_processor_handle = BlockProcessor::spawn(validator_state_clone, rx_pp_to_catchup);

        Ok(vec![transaction_processor_handle, block_processor_handle])
    }
}

pub struct TransactionProcessor {
    rx_narwhal_to_post_process: Receiver<(HandledTransaction, SerializedTransaction)>,
    validator_state: Arc<ValidatorState>,
    tx_pp_to_catchup: Sender<u64>,
}

impl TransactionProcessor {
    pub fn spawn(
        rx_narwhal_to_post_process: Receiver<(HandledTransaction, SerializedTransaction)>,
        validator_state: Arc<ValidatorState>,
        tx_pp_to_catchup: Sender<u64>,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            Self {
                rx_narwhal_to_post_process,
                validator_state,
                tx_pp_to_catchup,
            }
            .run()
            .await
        })
    }

    /// Main loop listening to new certificates and execute them.
    async fn run(&mut self) {
        // create vec of transactions to store in blocks on disk
        let mut serialized_txns_buf = Vec::new();
        // unpack validator store, metrics and controllers
        let store = &self.validator_state.validator_store;
        let metrics = &self.validator_state.metrics;
        let controller_router = &self.validator_state.controller_router;

        loop {
            while let Some(message) = self.rx_narwhal_to_post_process.recv().await {
                let (result, serialized_txn) = message;
                match result {
                    Ok((consensus_output, execution_indices, execution_result)) => {
                        serialized_txns_buf.push((serialized_txn, execution_result));

                        // if next_transaction_index == 0 then the block is complete and we may write-out
                        if execution_indices.next_transaction_index == 0 {
                            // subtract round look-back from the latest round to get block number
                            let num_txns = serialized_txns_buf.len();

                            // prune transaction cache on validator store + write out latest block
                            store.prune();
                            let (block, block_info) = store
                                .write_latest_block(consensus_output.certificate, serialized_txns_buf.clone())
                                .await;

                            let block_number = store.block_number.load(std::sync::atomic::Ordering::SeqCst);

                            self.tx_pp_to_catchup
                                .send(block_number)
                                .await
                                .expect("Failed to send block number to catchup process");

                            // metrics process end of block
                            metrics.process_end_of_block(block, block_info);
                            // controller logic process end of block
                            controller_router
                                .process_end_of_block(&store.post_process_store, block_number)
                                .await;

                            // catchup generate
                            controller_router
                                .create_catchup_state(&store.post_process_store, block_number)
                                .await;

                            serialized_txns_buf.clear();
                            // This log is used in benchmarking
                            info!("Finalized block {block_number} contains {num_txns} transactions");
                        }
                    }
                    Err(e) => error!("{:?}", e), // TODO
                }
            }
        }
    }
}

pub struct BlockProcessor {
    validator_state: Arc<ValidatorState>,
    rx_pp_to_catchup: Receiver<u64>,
}

impl BlockProcessor {
    pub fn spawn(validator_state: Arc<ValidatorState>, rx_pp_to_catchup: Receiver<u64>) -> JoinHandle<()> {
        tokio::spawn(async move {
            Self {
                validator_state,
                rx_pp_to_catchup,
            }
            .run()
            .await
        })
    }

    async fn run(&mut self) {
        // get references to catchup router and store
        let catchup_router = &self.validator_state.catchup_router;
        let store = &self.validator_state.validator_store;

        loop {
            while let Some(block_number) = self.rx_pp_to_catchup.recv().await {
                // run transactions through catchup router to sync catchup router
                if let Ok(Some(block)) = store.post_process_store.block_store.read(block_number).await {
                    let transactions = block.transactions;
                    for (serialized_transaction, _) in &transactions {
                        let consensus_transaction: ConsensusTransaction =
                            bincode::deserialize(serialized_transaction).unwrap();
                        let transaction = consensus_transaction.get_payload().unwrap().transaction.unwrap();
                        catchup_router.handle_consensus_transaction(&transaction).unwrap_or(());
                    }
                }

                // catchup generate
                catchup_router
                    .create_catchup_state(&store.post_process_store, block_number)
                    .await;
            }
        }
    }
}
