//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
//! This file is largely inspired by https://github.com/MystenLabs/sui/blob/main/crates/sui-core/src/authority.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bcuse super::state::ValidatorState;
use gdex_types::block::{Block, BlockInfo};
use ringbuffer::{AllocRingBuffer, RingBufferExt, RingBufferWrite};
use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::{SystemTime, UNIX_EPOCH},
};

/// Capacitate must be of form 2^n
const TPX_CAPACITY: usize = 128;

type ClusterTPS = f64;
type BlockLatencyInMilis = u64;

/// Track end to end transaction pipeline metrics
// TODO - think about how to keep num_transactions_consensus up to date after catch up
pub struct ValidatorMetrics {
    // Continuously updated information
    /// The number of transactions submitted to the validator
    pub num_transactions_rec: AtomicU64,
    /// The number of transactions submitted to the validator that were not executed
    pub num_transactions_rec_failed: AtomicU64,
    /// The number of transactions submitted from consensus
    pub num_transactions_consensus: AtomicU64,
    /// The number of transactions submitted from consensus that failed state execution
    pub num_transactions_consensus_failed: AtomicU64,
    /// Latest system epoch time in ms
    pub latest_system_epoch_time_in_ms: AtomicU64,
    /// Facilitators in calculating recent TPS and latency
    tps_ring_buffer: Arc<Mutex<AllocRingBuffer<(ClusterTPS, BlockLatencyInMilis)>>>,
    prev_block_info: Arc<Mutex<BlockInfo>>,
}

impl ValidatorMetrics {
    // incrementers
    pub fn increment_num_transactions_rec(&self) {
        self.update_latest_time();
        self.num_transactions_rec.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_num_transactions_rec_failed(&self) {
        self.update_latest_time();
        self.num_transactions_rec_failed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_num_transactions_consensus(&self) {
        self.update_latest_time();
        self.num_transactions_consensus.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_num_transactions_consensus_failed(&self) {
        self.update_latest_time();
        self.num_transactions_consensus_failed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn update_latest_time(&self) {
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        self.latest_system_epoch_time_in_ms
            .store(since_the_epoch.as_millis().try_into().unwrap(), Ordering::Relaxed);
    }

    pub fn process_new_block(&self, block: Block, block_info: BlockInfo) {
        let mut prev_block_info = self.prev_block_info.lock().unwrap();

        // Check that default block info is not stored in prev_block_info
        if prev_block_info.validator_system_epoch_time_in_ms != 0 {
            let num_transactions = block.transactions.len();
            let time_delta =
                block_info.validator_system_epoch_time_in_ms - prev_block_info.validator_system_epoch_time_in_ms;
            let calculated_tps = num_transactions as f64 / (time_delta as f64 / 1000.0);
            self.tps_ring_buffer.lock().unwrap().push((calculated_tps, time_delta));
        }

        *prev_block_info = block_info;
    }

    pub fn get_average_tps(&self) -> f64 {
        let mut sum = 0.0;
        let mut count = 0;
        for (tps, _latency) in self.tps_ring_buffer.lock().unwrap().iter() {
            sum += tps;
            count += 1;
        }
        sum / count as f64
    }

    pub fn get_average_latency_in_milis(&self) -> u64 {
        let mut sum = 0;
        let mut count = 0;
        for (_tps, time_delta) in self.tps_ring_buffer.lock().unwrap().iter() {
            sum += time_delta;
            count += 1;
        }
        sum / count
    }
}

impl Default for ValidatorMetrics {
    fn default() -> Self {
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();

        ValidatorMetrics {
            num_transactions_rec: AtomicU64::new(0),
            num_transactions_rec_failed: AtomicU64::new(0),
            num_transactions_consensus: AtomicU64::new(0),
            num_transactions_consensus_failed: AtomicU64::new(0),
            latest_system_epoch_time_in_ms: AtomicU64::new(since_the_epoch.as_millis().try_into().unwrap()),
            tps_ring_buffer: Arc::new(Mutex::new(AllocRingBuffer::with_capacity(TPX_CAPACITY))),
            prev_block_info: Arc::new(Mutex::new(BlockInfo::default())),
        }
    }
}
