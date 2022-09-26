// IMPORTS

// local
use crate::validator::{server::HandledTransaction, state::ValidatorState};

// gdex
use gdex_types::transaction::{ConsensusTransaction, ExecutionResultBody};

// external
use narwhal_executor::SerializedTransaction;
use std::sync::Arc;
use tokio::{
    sync::{broadcast, mpsc},
    task::JoinHandle,
};
use tracing::{error, info};

// INTERFACE

pub struct ValidatorPostProcessService {}

impl ValidatorPostProcessService {
    pub fn spawn(
        rx_narwhal_to_post_process: mpsc::Receiver<(HandledTransaction, SerializedTransaction)>,
        validator_state: Arc<ValidatorState>,
    ) -> anyhow::Result<Vec<JoinHandle<()>>> {
        // channel to communicate from txn processor to block, catchup processors
        let (tx_txn_to_processors, rx_txn_to_block_processor) = broadcast::channel::<u64>(1_000);
        let rx_txn_to_catchup_processor = tx_txn_to_processors.subscribe();

        let transaction_processor_handle = TransactionProcessor::spawn(
            rx_narwhal_to_post_process,
            Arc::clone(&validator_state),
            tx_txn_to_processors,
        );
        let block_processor_handle = BlockProcessor::spawn(Arc::clone(&validator_state), rx_txn_to_block_processor);
        let catchup_processor_handle =
            CatchupProcessor::spawn(Arc::clone(&validator_state), rx_txn_to_catchup_processor);

        Ok(vec![
            transaction_processor_handle,
            block_processor_handle,
            catchup_processor_handle,
        ])
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
        let mut serialized_txns_buf = Vec::new();
        // unpack validator store, metrics and controllers
        let store = &self.validator_state.validator_store;

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

                            // write txns to block store for use in block processor
                            store
                                .write_latest_block(consensus_output.certificate, serialized_txns_buf.clone())
                                .await;

                            let block_number = store.block_number.load(std::sync::atomic::Ordering::SeqCst);

                            // broadcast block number to catchup + block processors
                            self.tx_txn_to_processors.send(block_number).unwrap();

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
        let controller_router = &self.validator_state.controller_router;

        loop {
            while let Ok(block_number) = self.rx_txn_to_block_processor.recv().await {
                // load block and block info
                let block = store.post_process_store.block_store.read(block_number).await.unwrap();

                let block_info = store
                    .post_process_store
                    .block_info_store
                    .read(block_number)
                    .await
                    .unwrap();

                if block.is_some() && block_info.is_some() {
                    // metrics process end of block
                    metrics.process_end_of_block(block.unwrap(), block_info.unwrap());
                }

                // controller logic process end of block
                controller_router
                    .process_end_of_block(&store.post_process_store, block_number)
                    .await;
            }
        }
    }
}

pub struct CatchupProcessor {
    validator_state: Arc<ValidatorState>,
    rx_txn_to_catchup_processor: broadcast::Receiver<u64>,
}

impl CatchupProcessor {
    pub fn spawn(
        validator_state: Arc<ValidatorState>,
        rx_txn_to_catchup_processor: broadcast::Receiver<u64>,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            Self {
                validator_state,
                rx_txn_to_catchup_processor,
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
            while let Ok(block_number) = self.rx_txn_to_catchup_processor.recv().await {
                // run transactions through catchup router to sync catchup router
                if let Ok(Some(block)) = store.post_process_store.block_store.read(block_number).await {
                    let transactions = block.transactions;
                    for (serialized_transaction, _) in &transactions {
                        let consensus_transaction: ConsensusTransaction =
                            bincode::deserialize(serialized_transaction).unwrap();
                        let transaction = consensus_transaction.get_payload().unwrap().transaction.unwrap();
                        catchup_router
                            .handle_consensus_transaction(&transaction)
                            .unwrap_or_else(|_| ExecutionResultBody::new());
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
