use tonic::{transport::Server, Request, Response, Status};
use std::{env, net::SocketAddr};

use faucet::faucet_server::{Faucet, FaucetServer};
use faucet::{FaucetAirdropRequest, FaucetAirdropResponse};

use gdex_types::proto::TransactionsClient;

pub mod faucet {
    tonic::include_proto!("faucet");
}

#[derive(Debug, Default)]
pub struct FaucetService {}

#[tonic::async_trait]
impl Faucet for FaucetService {
    async fn airdrop(
        &self,
        request: Request<FaucetAirdropRequest>,
    ) -> Result<Response<FaucetAirdropResponse>, Status> {
        println!("Got a request {:?}", request);

        let req = request.into_inner();

        let reply = FaucetAirdropResponse {
            successful: true,
        };

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
