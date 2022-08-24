use crate::{relayer::server::RelayerService, validator::state::ValidatorState};
use gdex_types::proto::{*};
use gdex_types::utils;

use std::sync::Arc;

use crate::relayer::server::RelayerServerHandle;

pub struct RelayerSpawner {
    validator_state: Arc<ValidatorState>,
}

impl RelayerSpawner {
    pub fn new(state: Arc<ValidatorState>) -> Self {
        RelayerSpawner { validator_state: state }
    }

    pub async fn spawn_relay_server(&mut self) -> Result<RelayerServerHandle, Box<dyn std::error::Error>> {
        // Putting the port to 8000
        let addr = utils::new_network_address();
        // let multiaddr = Multiaddr::from(addr);

        // Instantiating the faucet service
        let relay_service = RelayerService {
            state: self.validator_state.clone(),
        };

        // Start the relay service

        let server = crate::config::server::ServerConfig::new()
            .server_builder()
            .add_service(RelayerServer::new(relay_service))
            .bind(&addr)
            .await
            .unwrap();

        // let server = Server::builder().add_service(RelayerServer::new(relay_service));
        let handle = tokio::spawn(async move { server.serve().await });
        let server_handle = RelayerServerHandle::new(addr, handle);
        Ok(server_handle)
    }
}

#[cfg(test)]
pub mod suite_spawn_tests {
    use crate::relayer::spawner::RelayerSpawner;
    use crate::validator::spawner::ValidatorSpawner;
    use gdex_types::{
        proto::{*},
        utils,
    };
    use std::path::Path;

    use crate::client::endpoint_from_multiaddr;

    #[tokio::test]
    pub async fn spawn_relay_server() {
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

        let _handles = validator_spawner.spawn_validator_with_reconfigure().await;

        let validator_state = validator_spawner.get_validator_state();

        let mut relay_spawner = RelayerSpawner::new(validator_state.clone().unwrap());

        // TODO clean
        let address = validator_spawner.get_relayer_address();
        let address_ref= &address.clone().unwrap();
        let target_endpoint = endpoint_from_multiaddr(address_ref).unwrap();
        let endpoint = target_endpoint.endpoint();
        let mut client = RelayerClient::connect(endpoint.clone()).await.unwrap();

        let request = tonic::Request::new(RelayerRequest {
            dummy_request: "hello world".to_string(),
        });

        let response = client.read_latest_block_info(request).await;

        assert!(response.unwrap().into_inner().successful)
    }
}
