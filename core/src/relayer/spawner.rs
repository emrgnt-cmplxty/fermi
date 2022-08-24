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
        let addr = "127.0.0.1:8001";

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
    // use executor::ExecutionState;
    use gdex_types::block::BlockDigest;
    use gdex_types::crypto::KeypairTraits;
    use gdex_types::crypto::Signer;
    use gdex_types::transaction::create_asset_creation_transaction;
    use gdex_types::transaction::SignedTransaction;
    use gdex_types::{
        proto::{RelayerClient, RelayerGetBlockInfoRequest, RelayerGetBlockRequest, RelayerGetLatestBlockInfoRequest},
        utils,
    };
    use narwhal_consensus::ConsensusOutput;
    use narwhal_crypto::generate_production_keypair;
    use narwhal_crypto::Hash;
    use narwhal_crypto::KeyPair;
    use narwhal_crypto::DIGEST_LEN;
    use narwhal_types::Certificate;
    use narwhal_types::Header;
    use std::path::Path;

    pub fn create_test_consensus_output() -> ConsensusOutput {
        let dummy_header = Header::default();
        let dummy_certificate = Certificate {
            header: dummy_header,
            votes: Vec::new(),
        };
        ConsensusOutput {
            certificate: dummy_certificate,
            consensus_index: 1,
        }
    }

    #[tokio::test]
    pub async fn spawn_and_ping_relay_server() {
        // SPAWNING THE SERVER
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

        let validator_state = validator_spawner.get_validator_state().clone().unwrap();

        let dummy_consensus_output = create_test_consensus_output();
        // create asset transaction
        let sender_kp = generate_production_keypair::<KeyPair>();
        let recent_block_hash = BlockDigest::new([0; DIGEST_LEN]);
        let create_asset_txn = create_asset_creation_transaction(&sender_kp, recent_block_hash);
        let signed_digest = sender_kp.sign(&create_asset_txn.digest().get_array()[..]);
        let signed_create_asset_txn =
            SignedTransaction::new(sender_kp.public().clone(), create_asset_txn, signed_digest);

        // preparing serialized txn buf
        let mut serialized_txns_buf: Vec<Vec<u8>> = Vec::new();
        for _ in 1..10 {
            let serialized_txn = signed_create_asset_txn.serialize().unwrap();
            serialized_txns_buf.push(serialized_txn)
        }
        // writing the latest block
        validator_state
            .validator_store
            .write_latest_block(dummy_consensus_output.certificate, serialized_txns_buf)
            .await;

        let mut relay_spawner = RelayerSpawner {
            validator_state: Some(validator_state),
        };

        tokio::spawn(async move {
            relay_spawner.spawn_relay_server().await.unwrap();
        });

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        // PINGING THE SERVER
        let addr = "http://127.0.0.1:8001";
        let mut client = RelayerClient::connect(addr.to_string()).await.unwrap();

        let latest_block_info_request = tonic::Request::new(RelayerGetLatestBlockInfoRequest {});
        let latest_block_info_response = client.read_latest_block_info(latest_block_info_request).await;
        println!("Response from latest block={:?}", latest_block_info_response);

        let specific_block_info_request = tonic::Request::new(RelayerGetBlockInfoRequest { block_number: 0 });
        let specific_block_info_response = client.get_block_info(specific_block_info_request).await;
        println!(
            "Response from specific block request = {:?}",
            specific_block_info_response
        );

        let specific_block_request = tonic::Request::new(RelayerGetBlockRequest { block_number: 0 });
        let specific_block_response = client.get_block(specific_block_request).await;
        println!("Response from specific block request = {:?}", specific_block_response);
    }
}
