use crate::{relayer::server::RelayerService, validator::state::ValidatorState};
use gdex_types::proto::*;
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
    
    use crate::validator::spawner::ValidatorSpawner;
    use gdex_types::block::Block;
    use gdex_types::block::BlockDigest;
    use gdex_types::crypto::KeypairTraits;
    use gdex_types::crypto::Signer;
    use gdex_types::transaction::create_asset_creation_transaction;
    use gdex_types::transaction::SignedTransaction;
    use gdex_types::{proto::*, utils};
    use narwhal_consensus::ConsensusOutput;
    use narwhal_crypto::generate_production_keypair;
    use narwhal_crypto::Hash;
    use narwhal_crypto::KeyPair;
    use narwhal_crypto::DIGEST_LEN;
    use narwhal_types::Certificate;
    use narwhal_types::Header;
    use std::path::Path;

    use crate::client::endpoint_from_multiaddr;

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

    // cargo test --package gdex-core --lib -- relayer::spawner::suite_spawn_tests::spawn_and_ping_relay_server --exact --nocapture
    #[tokio::test]
    pub async fn spawn_relay_server() {
        // Arrange
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

        // Create txns
        let dummy_consensus_output = create_test_consensus_output();
        let sender_kp = generate_production_keypair::<KeyPair>();
        let recent_block_hash = BlockDigest::new([0; DIGEST_LEN]);
        let create_asset_txn = create_asset_creation_transaction(&sender_kp, recent_block_hash);
        let signed_digest = sender_kp.sign(&create_asset_txn.digest().get_array()[..]);
        let signed_create_asset_txn =
            SignedTransaction::new(sender_kp.public().clone(), create_asset_txn, signed_digest);

        // Preparing serialized buf for transactions
        let mut serialized_txns_buf: Vec<Vec<u8>> = Vec::new();
        let serialized_txn = signed_create_asset_txn.serialize().unwrap();
        serialized_txns_buf.push(serialized_txn);
        let certificate = dummy_consensus_output.certificate;

        let initial_certificate = certificate.clone();
        let initial_serialized_txns_buf = serialized_txns_buf.clone();

        // Write the block
        validator_state
            .validator_store
            .write_latest_block(initial_certificate, initial_serialized_txns_buf)
            .await;

        // TODO clean
        // Connect client
        let address = validator_spawner.get_relayer_address();
        let address_ref = &address.clone().unwrap();
        let target_endpoint = endpoint_from_multiaddr(address_ref).unwrap();
        let endpoint = target_endpoint.endpoint();
        let mut client = RelayerClient::connect(endpoint.clone()).await.unwrap();

        let specific_block_request = tonic::Request::new(RelayerGetBlockRequest { block_number: 0 });
        let latest_block_info_request = tonic::Request::new(RelayerGetLatestBlockInfoRequest {});

        // Act
        let specific_block_response = client.get_block(specific_block_request).await;
        let latest_block_info_response = client.get_latest_block_info(latest_block_info_request).await;
        let block_bytes_returned = specific_block_response.unwrap().into_inner().block.unwrap().block;

        // Assert
        let deserialized_block: Block = bincode::deserialize(&block_bytes_returned).unwrap();
        let final_certificate = certificate.clone();
        let final_serialized_txns_buf = serialized_txns_buf.clone();
        let block_to_check_against = Block {
            block_certificate: final_certificate,
            transactions: final_serialized_txns_buf,
        };
        assert!(block_to_check_against.block_certificate == deserialized_block.block_certificate);
        assert!(block_to_check_against.transactions == deserialized_block.transactions);
        // TODO TESTS FOR BLOCK INFO, CURRENTLY WE JUST PRINT
        assert!(latest_block_info_response.unwrap().into_inner().successful)
    }
}
