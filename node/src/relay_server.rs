use gdex_core::builder::genesis_state::GenesisStateBuilder;
use gdex_types::account::{AccountKeyPair, ValidatorKeyPair, ValidatorPubKeyBytes};
// use relay::relay_server::{Relay, RelayServer};
// use relay::{RelayRequest, RelayResponse};
use std::{net::SocketAddr, path::Path, sync::Arc};
use tonic::{transport::Server, Request, Response, Status};
// pub mod relay {
//     tonic::include_proto!("relay");
// }
use gdex_controller::master::MasterController;
use gdex_core::{genesis_ceremony::VALIDATOR_FUNDING_AMOUNT, validator::state::ValidatorState};
use gdex_types::{
    crypto::KeypairTraits,
    node::ValidatorInfo,
    proto::{Relay, RelayRequest, RelayResponse, RelayServer},
    utils,
};

#[derive(Debug, Default)]
pub struct RelayService {}

#[tonic::async_trait]
impl Relay for RelayService {
    async fn read_data(&self, request: Request<RelayRequest>) -> Result<Response<RelayResponse>, Status> {
        let master_controller = MasterController::default();

        // Getting the file path for the key of the faucet
        let key_dir = ".proto/";
        let key_path = Path::new(key_dir).to_path_buf();

        // For now the faucet is the primary validator
        let primary_validator_index = 0;
        let key_file = key_path.join(format!("validator-{}.key", primary_validator_index));

        let kp: AccountKeyPair = utils::read_keypair_from_file(&key_file).unwrap();
        let public_key = ValidatorPubKeyBytes::from(kp.public());

        let validator = ValidatorInfo {
            name: "0".into(),
            public_key: public_key.clone(),
            stake: VALIDATOR_FUNDING_AMOUNT,
            delegation: 0,
            network_address: utils::new_network_address(),
            narwhal_primary_to_primary: utils::new_network_address(),
            narwhal_worker_to_primary: utils::new_network_address(),
            narwhal_primary_to_worker: utils::new_network_address(),
            narwhal_worker_to_worker: utils::new_network_address(),
            narwhal_consensus_address: utils::new_network_address(),
        };

        let builder = GenesisStateBuilder::new()
            .set_master_controller(master_controller)
            .add_validator(validator);

        let genesis = builder.build();

        let secret = Arc::pin(kp);

        let validator_state = Arc::new(ValidatorState::new(public_key, secret, &genesis));
        validator_state.validator_store.prune();
        let returned_value = validator_state.validator_store.transaction_store.read(1).await.unwrap();
        println!("{:?}", returned_value);

        println!("Succesfully loaded validator state!");

        // We can now return true because errors will have been caught above
        let reply = RelayResponse { successful: true };

        // Sending back a response
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Putting the port to 8000
    let addr = "127.0.0.1:8000";

    // Parsing it into an address
    let addr = addr.parse::<SocketAddr>()?;

    // Instantiating the faucet service
    let relay_service = RelayService::default();

    // Start the faucet service
    Server::builder()
        .add_service(RelayServer::new(relay_service))
        .serve(addr)
        .await?;

    Ok(())
}
