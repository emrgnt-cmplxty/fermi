use gdex_core::client;
use gdex_types::{
    account::AccountKeyPair,
    crypto::{KeypairTraits, Signer},
    proto::{Faucet, FaucetAirdropRequest, FaucetAirdropResponse, FaucetServer, TransactionSubmitterClient},
    new_transaction::{NewSignedTransaction, NewTransaction, new_create_payment_transaction, sign_transaction},
    utils,
};
use multiaddr::Multiaddr;
use narwhal_crypto::{ed25519::Ed25519PublicKey, traits::ToFromBytes, Hash, DIGEST_LEN};
use narwhal_types::CertificateDigest;
use std::{
    env, io,
    net::SocketAddr,
    path::{Path, PathBuf},
};
use tonic::{transport::Server, Request, Response, Status};
pub const PRIMARY_ASSET_ID: u64 = 0;
pub const FAUCET_PORT: u32 = 8080;

#[derive(Debug)]
pub struct FaucetService {
    pub validator_index: u64,
    pub key_path: PathBuf,
    pub validator_addr: Multiaddr,
}

fn generate_signed_airdrop_transaction_for_faucet(
    kp_sender: &AccountKeyPair,
    kp_receiver_public_key: &Ed25519PublicKey,
    amount: u64,
) -> NewSignedTransaction {
    // Setting a certificate_digest
    let recent_certificate_digest = CertificateDigest::new([0; DIGEST_LEN]);
    let gas: u64 = 1000;
    let new_transaction = new_create_payment_transaction(kp_sender.public().clone(), kp_receiver_public_key, PRIMARY_ASSET_ID, amount, gas, recent_certificate_digest);
    let new_signed_transaction = match sign_transaction(kp_sender, new_transaction) {
        Ok(t) => t,
        _ => panic!("Error signing transaction"),
    };

    new_signed_transaction
}

#[tonic::async_trait]
impl Faucet for FaucetService {
    async fn airdrop(&self, request: Request<FaucetAirdropRequest>) -> Result<Response<FaucetAirdropResponse>, Status> {
        // Getting the file path for the key of the faucet

        // For now the faucet is the primary validator
        let key_file = self.key_path.join(format!("validator-{}.key", self.validator_index));
        // Getting request, parsing it, and changing the hex public key passed in into a public key object
        let req = request.into_inner();
        let bytes_airdrop_to = hex::decode(req.airdrop_to).unwrap();
        let kp_public_airdrop_to = Ed25519PublicKey::from_bytes(&bytes_airdrop_to).unwrap();

        // Sender is validator, receiver is the person passed in
        let kp_sender: AccountKeyPair = utils::read_keypair_from_file(&key_file).unwrap();
        let kp_receiver_public_key = kp_public_airdrop_to;

        // Creating signed transaction and proto
        let signed_transaction =
            generate_signed_airdrop_transaction_for_faucet(&kp_sender, &kp_receiver_public_key, 100);
        
        // Getting the validator port from the second cli argument
        // The port for the validator that we will send the transaction to is passed in as the second cli argument when the server is starting
        let mut client = TransactionSubmitterClient::new(
            client::connect_lazy(&self.validator_addr).expect("Failed to connect to consensus"),
        );

        // If there is an error we will get a panic because of the unwrap
        let _response = client
            .submit_transaction(signed_transaction)
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
            .unwrap();

        // We can now return true because errors will have been caught above
        let reply = FaucetAirdropResponse { successful: true };

        // Sending back a response
        Ok(Response::new(reply))
    }
}

impl Default for FaucetService {
    fn default() -> Self {
        let key_dir = ".proto/";
        let key_path = Path::new(key_dir).to_path_buf();

        // TODO - take validator addr directly as multiaddr as in spawn node
        let validator_port = env::args().nth(1).unwrap();
        let validator_addr = format!("/dns/localhost/tcp/{}/http", validator_port).parse().unwrap();

        Self {
            validator_index: 0,
            key_path,
            validator_addr,
        }
    }
}

#[tokio::main]
#[allow(dead_code)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Getting the address that is passed in
    let addr = format!("127.0.0.1:{}", FAUCET_PORT);

    // Parsing it into an address
    let addr = addr.parse::<SocketAddr>()?;

    // Instantiating the faucet service
    let faucet_service = FaucetService::default();

    // Start the faucet service
    Server::builder()
        .add_service(FaucetServer::new(faucet_service))
        .serve(addr)
        .await?;

    Ok(())
}
