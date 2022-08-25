use crate::{relayer::server::RelayerService, validator::state::ValidatorState};
use gdex_types::proto::*;
use multiaddr::Multiaddr;
use std::sync::Arc;
use tokio::task::JoinHandle;

pub struct RelayerSpawner {
    validator_state: Arc<ValidatorState>,
    address: Multiaddr,
    /// Handle for the server related tasks
    server_handles: Option<JoinHandle<()>>,
}

impl RelayerSpawner {
    pub fn new(state: Arc<ValidatorState>, address: Multiaddr) -> Self {
        RelayerSpawner {
            validator_state: state,
            address,
            server_handles: None,
        }
    }

    pub async fn spawn_relayer(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Instantiating the faucet service
        let relay_service = RelayerService {
            state: self.validator_state.clone(),
        };

        // Start the relay service

        let server = crate::config::server::ServerConfig::new()
            .server_builder()
            .add_service(RelayerServer::new(relay_service))
            .bind(&self.address)
            .await
            .unwrap();

        let handle = tokio::spawn(async move { server.serve().await });
        self.server_handles = Some(handle);

        Ok(())
    }

    pub async fn await_handles(&mut self) {
        self.server_handles
            .as_mut()
            .expect("Attempting to await a non-existent handle")
            .await
            .unwrap();
    }
}
