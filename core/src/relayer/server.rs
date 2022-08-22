use crate::validator::state::ValidatorState;
use gdex_types::proto::{
    BlockInfoProto, Relayer, RelayerGetBlockInfoRequest, RelayerGetLatestBlockInfoRequest, RelayerResponse,
};
use narwhal_types::CertificateDigestProto;
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct RelayerService {
    pub state: Arc<ValidatorState>,
}

#[tonic::async_trait]
impl Relayer for RelayerService {
    async fn read_latest_block_info(
        &self,
        _request: Request<RelayerGetLatestBlockInfoRequest>,
    ) -> Result<Response<RelayerResponse>, Status> {
        let validator_state = &self.state;

        match validator_state.validator_store.last_block_store.read(0).await {
            Ok(opt) => {
                if opt.is_some() {
                    let block_info = opt.unwrap();
                    Ok(Response::new(RelayerResponse {
                        successful: true,
                        block_info: Some(BlockInfoProto {
                            block_number: block_info.block_number,
                            digest: CertificateDigestProto::from(block_info.block_digest).digest, // TODO egregious hack (MI)
                        }),
                    }))
                } else {
                    Ok(Response::new(RelayerResponse {
                        successful: true,
                        block_info: None,
                    }))
                }
            }
            // TODO propagate error message to client
            Err(_) => Ok(Response::new(RelayerResponse {
                successful: false,
                block_info: None,
            })),
        }
    }
    async fn get_block_info(
        &self,
        request: Request<RelayerGetBlockInfoRequest>,
    ) -> Result<Response<RelayerResponse>, Status> {
        let validator_state = &self.state;
        let req = request.into_inner();
        let block_number = req.block_number;

        match validator_state
            .validator_store
            .block_info_store
            .read(block_number)
            .await
        {
            Ok(opt) => {
                if opt.is_some() {
                    let block_info = opt.unwrap();
                    Ok(Response::new(RelayerResponse {
                        successful: true,
                        block_info: Some(BlockInfoProto {
                            block_number: block_info.block_number,
                            digest: CertificateDigestProto::from(block_info.block_digest).digest, // TODO egregious hack (MI)
                        }),
                    }))
                } else {
                    Ok(Response::new(RelayerResponse {
                        successful: true,
                        block_info: None,
                    }))
                }
            }
            // TODO propagate error message to client
            Err(_) => Ok(Response::new(RelayerResponse {
                successful: false,
                block_info: None,
            })),
        }
    }
}
