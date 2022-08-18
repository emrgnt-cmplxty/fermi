use std::{env, error::Error, net::SocketAddr};
use faucet::faucet_client::FaucetClient;

use faucet::FaucetAirdropRequest;

pub mod faucet {
    tonic::include_proto!("faucet");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = env::args().nth(1).unwrap();
    let mut client = FaucetClient::connect(addr.to_string()).await?;

    let request = tonic::Request::new(FaucetAirdropRequest{
        airdrop_to: "Armstrong".to_owned(),
    });

    let response = client.airdrop(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}