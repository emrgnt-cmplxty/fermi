//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
//! This file is largely inspired by https://github.com/MystenLabs/sui/blob/main/crates/sui-core/src/authority.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bcuse super::state::ValidatorState;
use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

/// Track end to end transaction pipeline metrics
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
        }
    }
}
