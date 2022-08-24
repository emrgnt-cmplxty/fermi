use gdex_core::client;
use gdex_types::{
    account::AccountKeyPair,
    crypto::{KeypairTraits, Signer},
    proto::{Faucet, FaucetAirdropRequest, FaucetAirdropResponse, FaucetServer, TransactionProto, TransactionsClient},
    transaction::{PaymentRequest, SignedTransaction, Transaction, TransactionVariant},
    utils,
};
use narwhal_crypto::{ed25519::Ed25519PublicKey, traits::ToFromBytes, Hash, DIGEST_LEN};
use narwhal_types::CertificateDigest;
use std::{env, io, net::SocketAddr, path::Path};
use tonic::{transport::Server, Request, Response, Status};
pub const PRIMARY_ASSET_ID: u64 = 0;
pub const FAUCET_PORT: u32 = 8080;

#[derive(Debug, Default)]
pub struct FaucetService {}

pub fn generate_signed_airdrop_transaction_for_faucet(
    kp_sender: &AccountKeyPair,
    kp_receiver_public_key: &Ed25519PublicKey,
    amount: u64,
) -> SignedTransaction {
    // Setting a certificate_digest
    let recent_certificate_digest = CertificateDigest::new([0; DIGEST_LEN]);

    // Creating the transaction variant
    let transaction_variant = TransactionVariant::PaymentTransaction(PaymentRequest::new(
        kp_receiver_public_key.clone(),
        PRIMARY_ASSET_ID,
        amount,
    ));

    // Creating the transaction itself, signing it, and returning it
    let transaction = Transaction::new(
        kp_sender.public().clone(),
        recent_certificate_digest,
        transaction_variant,
    );
    let signed_digest = kp_sender.sign(&transaction.digest().get_array()[..]);
    SignedTransaction::new(kp_sender.public().clone(), transaction, signed_digest)
}

#[tonic::async_trait]
impl Faucet for FaucetService {
    async fn airdrop(&self, request: Request<FaucetAirdropRequest>) -> Result<Response<FaucetAirdropResponse>, Status> {
        // Getting the file path for the key of the faucet
        let key_dir = ".proto/";
        let key_path = Path::new(key_dir).to_path_buf();

        // For now the faucet is the primary validator
        let primary_validator_index = 0;
        let key_file = key_path.join(format!("validator-{}.key", primary_validator_index));

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
        let transaction_proto = TransactionProto {
            transaction: signed_transaction.serialize().unwrap().into(),
        };

        // Getting the validator port from the second cli argument
        let validator_port = env::args().nth(1).unwrap();

        // The port for the validator that we will send the transaction to is passed in as the second cli argument when the server is starting
        let primary_validator_addr = format!("/dns/localhost/tcp/{}/http", validator_port);
        let primary_validator_multiaddr = primary_validator_addr.parse().unwrap();
        let mut client = TransactionsClient::new(
            client::connect_lazy(&primary_validator_multiaddr).expect("Failed to connect to consensus"),
        );

        // If there is an error we will get a panic because of the unwrap
        let response = client
            .submit_transaction(transaction_proto)
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
            .unwrap();

        // Printing the response
        println!("{:?}", response);

        // We can now return true because errors will have been caught above
        let reply = FaucetAirdropResponse { successful: true };

        // Sending back a response
        Ok(Response::new(reply))
    }
}

#[allow(dead_code)]
#[tokio::main]
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
