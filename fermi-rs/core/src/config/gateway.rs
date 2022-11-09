// fermi
use crate::config::Config;
use fermi_types::{committee::EpochId, node::ValidatorInfo};
// external
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

const DEFAULT_SEND_TIMEOUT: u64 = 4000000;
const DEFAULT_RECEIVE_TIMEOUT: u64 = 4000000;
const DEFAULT_BUFFER_SIZE: usize = 650000;

/// Configures the network gateway for the local validator
#[derive(Serialize, Deserialize)]
pub struct GatewayConfig {
    /// Latest epoch of consensus
    pub epoch: EpochId,
    /// Current validators
    pub validator_set: Vec<ValidatorInfo>,
    /// Timeout for sending transactions
    pub send_timeout: Duration,
    /// Timeout for receiving transactions
    pub recv_timeout: Duration,
    /// Timeout for receiving transactions
    pub buffer_size: usize,
    pub db_folder_path: PathBuf,
}

impl Config for GatewayConfig {}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            epoch: 0,
            validator_set: vec![],
            send_timeout: Duration::from_micros(DEFAULT_SEND_TIMEOUT),
            recv_timeout: Duration::from_micros(DEFAULT_RECEIVE_TIMEOUT),
            buffer_size: DEFAULT_BUFFER_SIZE,
            db_folder_path: Default::default(),
        }
    }
}
