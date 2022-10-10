// gdex
use gdex_types::block::{Block, BlockInfo};
// external
use prometheus::{
    register_histogram_with_registry, register_int_counter_with_registry, register_int_gauge_with_registry, Histogram,
    IntCounter, IntGauge, Registry,
};
use ringbuffer::{AllocRingBuffer, RingBufferExt, RingBufferWrite};
use std::sync::{Arc, Mutex};

/// Capacity must be of form 2^n
const TPS_CAPACITY: usize = 128;

type ClusterTPS = f64;
type BlockSize = usize;
type BlockLatencyInMilis = u64;

/// Track end to end transaction pipeline metrics
pub struct ValidatorMetrics {
    // Continuously updated information
    /// The number of transactions submitted to the validator
    pub transactions_received: IntCounter,
    /// The number of transactions submitted to the validator that were not executed
    pub transactions_received_failed: IntCounter,
    /// The number of transactions submitted from consensus
    pub transactions_executed: IntCounter,
    /// The number of transactions submitted from consensus that failed state execution
    pub transactions_executed_failed: IntCounter,
    /// The number of blocks processed
    pub block_number: IntCounter,
    /// The validator system epoch time
    pub validator_system_epoch_time_in_micros: IntGauge,
    /// The block latency in miliseconds
    pub block_latency_micros: Histogram,
    /// The transactions per second of the cluster
    pub cluster_tps: Histogram,
    /// The validator transaction processing time
    pub transaction_rec_latency_in_micros: Histogram,
    /// Facilitators in calculating recent TPS and latency
    tps_ring_buffer: Arc<Mutex<AllocRingBuffer<(ClusterTPS, BlockSize, BlockLatencyInMilis)>>>,
    prev_block_info: Arc<Mutex<BlockInfo>>,
}

impl ValidatorMetrics {
    pub fn new(registry: &Registry) -> Self {
        // step from 0 to 3M micros
        let block_latency_buckets: Vec<f64> = (0..3_000).map(|i| i as f64 * 1_000.).collect();

        // step from 0 to 20k micros
        let transaction_latency_buckets: Vec<f64> = (0..2_000).map(|i| i as f64 * 10.).collect();

        // cluster TPS from 0 to 200k
        let cluster_tps_buckets: Vec<f64> = (0..2_000).map(|i| i as f64 * 100.).collect();

        Self {
            transactions_received: register_int_counter_with_registry!(
                "transactions_received",
                "The number of transactions sent to this validator.",
                registry
            )
            .unwrap(),
            transactions_received_failed: register_int_counter_with_registry!(
                "transactions_received_failed",
                "The number of transactions sent to this validator that failed execution.",
                registry
            )
            .unwrap(),
            transactions_executed: register_int_counter_with_registry!(
                "transactions_executed",
                "The number of transactions processed by this validator through consensus.",
                registry
            )
            .unwrap(),
            transactions_executed_failed: register_int_counter_with_registry!(
                "transactions_executed_failed",
                "The number of transactions processed by this validator through consensus which failed execution.",
                registry
            )
            .unwrap(),
            block_number: register_int_counter_with_registry!(
                "block_number",
                "The number of blocks created from consensus.",
                registry
            )
            .unwrap(),
            validator_system_epoch_time_in_micros: register_int_gauge_with_registry!(
                "validator_system_epoch_time_in_micros",
                "The system epoch time of the validator in micro seconds.",
                registry
            )
            .unwrap(),
            block_latency_micros: register_histogram_with_registry!(
                "block_latency_micros",
                "The latency between blocks in microseconds",
                block_latency_buckets,
                registry,
            )
            .unwrap(),
            transaction_rec_latency_in_micros: register_histogram_with_registry!(
                "transaction_rec_latency_in_micros",
                "The latency between receiving and processing a transaction",
                transaction_latency_buckets,
                registry,
            )
            .unwrap(),
            cluster_tps: register_histogram_with_registry!(
                "cluster_tps",
                "The transactions per second of the cluster",
                cluster_tps_buckets,
                registry,
            )
            .unwrap(),
            tps_ring_buffer: Arc::new(Mutex::new(AllocRingBuffer::with_capacity(TPS_CAPACITY))),
            prev_block_info: Arc::new(Mutex::new(BlockInfo::default())),
        }
    }

    pub fn process_end_of_block(&self, block: Block, block_info: BlockInfo) {
        self.block_number.inc();
        let mut prev_block_info = self.prev_block_info.lock().unwrap();

        // Check that default block info is not stored in prev_block_info
        if prev_block_info.validator_system_epoch_time_in_micros != 0 {
            let num_transactions = block.transactions.len();
            let time_delta_in_micros = block_info.validator_system_epoch_time_in_micros
                - prev_block_info.validator_system_epoch_time_in_micros;
            let calculated_tps = num_transactions as f64 / (time_delta_in_micros as f64 / 1_000_000.0);
            self.tps_ring_buffer
                .lock()
                .unwrap()
                .push((calculated_tps, num_transactions, time_delta_in_micros));
            self.validator_system_epoch_time_in_micros.set(
                (block_info.validator_system_epoch_time_in_micros as u64)
                    .try_into()
                    .unwrap(),
            );
            self.block_latency_micros
                .observe(self.get_average_latency_in_micros() as f64);
            self.cluster_tps.observe(self.get_average_tps());
        }

        *prev_block_info = block_info;
    }

    pub fn get_average_tps(&self) -> f64 {
        let buffer = self.tps_ring_buffer.lock().unwrap();
        let (sum, count) = buffer
            .iter()
            .fold((0 as f64, 0 as f64), |acc, x| (acc.0 + x.1 as f64, acc.1 + x.2 as f64));

        sum * 1_000_000.0 / count
    }

    pub fn get_average_latency_in_micros(&self) -> u64 {
        let buffer = self.tps_ring_buffer.lock().unwrap();
        let sum: u64 = buffer.iter().map(|x| x.2).sum();
        let count: u64 = buffer.iter().len().try_into().unwrap();

        sum / count
    }
}
