// IMPORTS

// local
use crate::validator::{server::HandledTransaction, state::ValidatorState};

// external
use narwhal_executor::SerializedTransaction;
use std::sync::Arc;
use tokio::{
    sync::{broadcast, mpsc},
    task::JoinHandle,
};
use tracing::{error, info};

// INTERFACE

pub struct ValidatorPostProcessor;

impl ValidatorPostProcessor {
    pub fn spawn(
        rx_narwhal_to_post_process: mpsc::Receiver<(HandledTransaction, SerializedTransaction)>,
        validator_state: Arc<ValidatorState>,
    ) -> anyhow::Result<Vec<JoinHandle<()>>> {
        // channel to communicate from txn processor to block, catchup processors
        let (tx_txn_to_processors, rx_txn_to_block_processor) = broadcast::channel::<u64>(1_000);

        let transaction_processor_handle = TransactionProcessor::spawn(
            rx_narwhal_to_post_process,
            Arc::clone(&validator_state),
            tx_txn_to_processors,
        );
        let block_processor_handle = BlockProcessor::spawn(Arc::clone(&validator_state), rx_txn_to_block_processor);
        Ok(vec![transaction_processor_handle, block_processor_handle])
    }
}

pub struct TransactionProcessor {
    rx_narwhal_to_post_process: mpsc::Receiver<(HandledTransaction, SerializedTransaction)>,
    validator_state: Arc<ValidatorState>,
    tx_txn_to_processors: broadcast::Sender<u64>,
}

impl TransactionProcessor {
    pub fn spawn(
        rx_narwhal_to_post_process: mpsc::Receiver<(HandledTransaction, SerializedTransaction)>,
        validator_state: Arc<ValidatorState>,
        tx_txn_to_processors: broadcast::Sender<u64>,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            Self {
                rx_narwhal_to_post_process,
                validator_state,
                tx_txn_to_processors,
            }
            .run()
            .await
        })
    }

    /// Main loop listening to new certificates and execute them.
    async fn run(&mut self) {
        // create vec of transactions to store in blocks on disk
        let mut executed_transactions = Vec::new();
        // unpack validator store, metrics and controllers
        let store = &self.validator_state.validator_store;

        loop {
            while let Some(message) = self.rx_narwhal_to_post_process.recv().await {
                let (result, _serialized_txn) = message;

                match result {
                    Ok((consensus_output, execution_indices, execution_result)) => {
                        executed_transactions.push(execution_result);

                        // if next_transaction_index == 0 then the block is complete and we may write-out
                        if execution_indices.next_transaction_index == 0 {
                            // subtract round look-back from the latest round to get block number
                            let num_txns = executed_transactions.len();

                            // prune transaction cache on validator store + write out latest block
                            store.prune();

                            // write txns to block store for use in block processor
                            store
                                .write_latest_block(consensus_output.certificate, executed_transactions.clone())
                                .await;

                            let block_number = store.block_number.load(std::sync::atomic::Ordering::SeqCst);

                            // broadcast block number to catchup + block processors
                            self.tx_txn_to_processors.send(block_number).unwrap();

                            executed_transactions.clear();
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
    rx_txn_to_block_processor: broadcast::Receiver<u64>,
}

impl BlockProcessor {
    pub fn spawn(
        validator_state: Arc<ValidatorState>,
        rx_txn_to_block_processor: broadcast::Receiver<u64>,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            Self {
                validator_state,
                rx_txn_to_block_processor,
            }
            .run()
            .await
        })
    }

    async fn run(&mut self) {
        // get references to catchup router and store
        let store = &self.validator_state.validator_store;
        let metrics = &self.validator_state.metrics;

        loop {
            while let Ok(block_number) = self.rx_txn_to_block_processor.recv().await {
                // load block and block info
                let block = store.critical_path_store.block_store.read(block_number).await.unwrap();

                let block_info = store
                    .critical_path_store
                    .block_info_store
                    .read(block_number)
                    .await
                    .unwrap();

                if block.is_some() && block_info.is_some() {
                    // metrics process end of block
                    metrics.process_end_of_block(block.unwrap(), block_info.unwrap());
                }
                // controller logic process end of block
                let controller_router = &self.validator_state.controller_router;

                controller_router
                    .critical_process_end_of_block(&store.critical_path_store, block_number)
                    .unwrap();
            }
        }
    }
}
