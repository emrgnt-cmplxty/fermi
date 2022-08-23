use crate::validator::state::ValidatorState;
use gdex_types::proto::{Relayer, RelayerRequest, RelayerResponse};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct RelayerService {
    pub state: Arc<ValidatorState>,
}

#[tonic::async_trait]
impl Relayer for RelayerService {
    async fn read_latest_block_info(
        &self,
        _request: Request<RelayerRequest>,
    ) -> Result<Response<RelayerResponse>, Status> {
        let _validator_state = &self.state;
        println!("Returned succesfully!");

        // We can now return true because errors will have been caught above
        let reply = RelayerResponse { successful: true };

        // Sending back a response
        Ok(Response::new(reply))
    }
}
