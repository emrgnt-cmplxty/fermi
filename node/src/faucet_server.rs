use std::{env, fs, io, net::SocketAddr, path::Path};
use tonic::{transport::Server, Request, Response, Status};

use faucet::faucet_server::{Faucet, FaucetServer};
use faucet::{FaucetAirdropRequest, FaucetAirdropResponse};

use gdex_core::client;
use gdex_core::validator::server::ValidatorServer;
use gdex_types::{
    account::{account_test_functions::generate_keypair_vec, AccountKeyPair},
    crypto::{KeypairTraits, Signer},
    proto::{TransactionProto, TransactionsClient},
    transaction::{PaymentRequest, SignedTransaction, Transaction, TransactionVariant},
    utils,
};
use narwhal_crypto::{Hash, DIGEST_LEN};
use narwhal_types::BatchDigest;

pub const PRIMARY_ASSET_ID: u64 = 0;

pub mod faucet {
    tonic::include_proto!("faucet");
}

#[derive(Debug, Default)]
pub struct FaucetService {}

pub fn generate_signed_airdrop_transaction_from_faucet(
    kp_sender: &AccountKeyPair,
    kp_receiver: &AccountKeyPair,
    amount: u64,
) -> SignedTransaction {
    let dummy_batch_digest = BatchDigest::new([0; DIGEST_LEN]);
    let transaction_variant = TransactionVariant::PaymentTransaction(PaymentRequest::new(
        kp_receiver.public().clone(),
        PRIMARY_ASSET_ID,
        amount,
    ));

    let transaction = Transaction::new(kp_sender.public().clone(), dummy_batch_digest, transaction_variant);

    let signed_digest = kp_sender.sign(&transaction.digest().get_array()[..]);

    SignedTransaction::new(kp_sender.public().clone(), transaction, signed_digest)
}

#[tonic::async_trait]
impl Faucet for FaucetService {
    async fn airdrop(&self, request: Request<FaucetAirdropRequest>) -> Result<Response<FaucetAirdropResponse>, Status> {
        let key_dir = ".proto/";
        let key_path = Path::new(key_dir).to_path_buf();
        let primary_validator_index = 0;
        let key_file = key_path.join(format!("validator-{}.key", primary_validator_index));

        // Treating the validator as an account for now
        let kp_sender: AccountKeyPair = utils::read_keypair_from_file(&key_file).unwrap();
        let kp_receiver = generate_keypair_vec([1; 32]).pop().unwrap();

        let signed_transaction = generate_signed_airdrop_transaction_from_faucet(&kp_sender, &kp_receiver, 100);
        let transaction_proto = TransactionProto {
            transaction: signed_transaction.serialize().unwrap().into(),
        };

        let primary_validator_addr = "/dns/localhost/tcp/62276/http";
        let primary_validator_multiaddr = primary_validator_addr.parse().unwrap();
        let mut client = TransactionsClient::new(
            client::connect_lazy(&primary_validator_multiaddr).expect("Failed to connect to consensus"),
        );

        let _resp1 = client
            .submit_transaction(transaction_proto)
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
            .unwrap();

        println!("{:?}", _resp1);

        // let req = request.into_inner();

        let reply = FaucetAirdropResponse { successful: true };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Getting the address that is passed in
    let addr = env::args().nth(1).unwrap();
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
