use crate::validator::state::ValidatorState;
use gdex_types::proto::{Relay, RelayRequest, RelayResponse};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct RelayService {
    pub state: Arc<ValidatorState>,
}

#[tonic::async_trait]
impl Relay for RelayService {
    async fn read_data(&self, request: Request<RelayRequest>) -> Result<Response<RelayResponse>, Status> {
        let validator_state = &self.state;
        validator_state.validator_store.prune();
        println!("Returned succesfully!");

        // We can now return true because errors will have been caught above
        let reply = RelayResponse { successful: true };

        // Sending back a response
        Ok(Response::new(reply))
    }
}
