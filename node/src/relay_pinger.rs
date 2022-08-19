use relay::relay_client::RelayClient;
use std::error::Error;

use relay::RelayRequest;

pub mod relay {
    tonic::include_proto!("relay");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = "http://127.0.0.1:8000";
    let mut client = RelayClient::connect(addr.to_string()).await?;

    let request = tonic::Request::new(RelayRequest {
        dummy_request: "hello world".to_string(),
    });

    let response = client.read_data(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
