use crate::{relayer::server::RelayerService, validator::state::ValidatorState};
use gdex_types::proto::RelayerServer;
use std::{net::SocketAddr, sync::Arc};
use tonic::transport::Server;

pub struct RelayerSpawner {
    validator_state: Option<Arc<ValidatorState>>,
}

impl RelayerSpawner {
    pub async fn spawn_relay_server(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Putting the port to 8000
        let addr = "127.0.0.1:8000";

        // Parsing it into an address
        let addr = addr.parse::<SocketAddr>()?;

        // Instantiating the faucet service
        let relay_service = RelayerService {
            state: Arc::clone(self.validator_state.as_ref().unwrap()),
        };

        // Start the faucet service
        Server::builder()
            .add_service(RelayerServer::new(relay_service))
            .serve(addr)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
pub mod suite_spawn_tests {
    use crate::relayer::spawner::RelayerSpawner;
    use crate::validator::spawner::ValidatorSpawner;
    use gdex_types::{
        proto::{RelayerClient, RelayerGetBlockInfoRequest, RelayerGetLatestBlockInfoRequest},
        utils,
    };
    use std::path::Path;

    #[tokio::test]
    #[ignore]
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

        let validator_state = validator_spawner.get_validator_state().clone();

        let mut relay_spawner = RelayerSpawner {
            validator_state: validator_state,
        };

        let _result = relay_spawner.spawn_relay_server().await;
    }

    #[tokio::test]
    #[ignore]
    pub async fn ping_relay_server() {
        let addr = "http://127.0.0.1:8000";
        let mut client = RelayerClient::connect(addr.to_string()).await.unwrap();

        let latest_block_request = tonic::Request::new(RelayerGetLatestBlockInfoRequest {
            dummy_request: "hello world".to_string(),
        });

        let latest_block_response = client.read_latest_block_info(latest_block_request).await;

        println!("Response from latest block={:?}", latest_block_response);

        let specific_block_request = tonic::Request::new(RelayerGetBlockInfoRequest { block_number: 0 });

        let specific_block_response = client.get_block_info(specific_block_request).await;

        println!("Response from specific block request = {:?}", specific_block_response);
    }
}
