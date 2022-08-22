use crate::{relay::server::RelayService, validator::state::ValidatorState};
use gdex_types::{proto::RelayServer, utils};
use std::{net::SocketAddr, sync::Arc};
use tonic::transport::Server;
use tracing::info;

pub struct RelaySpawner {
    validator_state: Option<Arc<ValidatorState>>,
}

impl RelaySpawner {
    pub async fn spawn_relay_server(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Putting the port to 8000
        let addr = "127.0.0.1:8000";

        // Parsing it into an address
        let addr = addr.parse::<SocketAddr>()?;

        // Instantiating the faucet service
        let relay_service = RelayService {
            state: Arc::clone(self.validator_state.as_ref().unwrap()),
        };

        // Start the faucet service
        Server::builder()
            .add_service(RelayServer::new(relay_service))
            .serve(addr)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
pub mod suite_spawn_tests {
    use crate::relay::spawner::RelaySpawner;
    use crate::validator::spawner::ValidatorSpawner;
    use gdex_types::utils;
    use std::path::Path;

    #[tokio::test]
    pub async fn spawn_node_and_reconfigure() {
        let dir = "../.proto";
        let path = Path::new(dir).to_path_buf();

        let address = utils::new_network_address();
        let mut validator_spawner = ValidatorSpawner::new(
            /* db_path */ path.clone(),
            /* key_path */ path.clone(),
            /* genesis_path */ path.clone(),
            /* validator_port */ address.clone(),
            /* validator_name */ "validator-0".to_string(),
        );

        let handles = validator_spawner.spawn_validator_with_reconfigure().await;

        let validator_state = validator_spawner.get_validator_state();

        let mut relay_spawner = RelaySpawner {
            validator_state: validator_state.clone(),
        };

        let result = relay_spawner.spawn_relay_server().await;
    }
}
