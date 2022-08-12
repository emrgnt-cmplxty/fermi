//! consensus controller contains all relevant consensus params
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConsensusController {
    pub min_batch_size: usize,
    pub max_batch_delay: Duration,
}
