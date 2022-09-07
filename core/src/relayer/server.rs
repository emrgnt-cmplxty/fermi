use crate::validator::state::ValidatorState;
use gdex_types::proto::*;
use narwhal_types::CertificateDigestProto;

use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct RelayerService {
    pub state: Arc<ValidatorState>,
}

#[tonic::async_trait]
impl Relayer for RelayerService {
    async fn get_latest_block_info(
        &self,
        _request: Request<RelayerGetLatestBlockInfoRequest>,
    ) -> Result<Response<RelayerBlockInfoResponse>, Status> {
        let validator_state = &self.state;
        let returned_value = validator_state.validator_store.last_block_info_store.read(0).await;
        println!("{:?}", returned_value.as_ref().unwrap());

        match returned_value {
            Ok(opt) => {
                if let Some(block_info) = opt {
                    Ok(Response::new(RelayerBlockInfoResponse {
                        successful: true,
                        block_info: Some(RelayerBlockInfo {
                            block_number: block_info.block_number,
                            digest: CertificateDigestProto::from(block_info.block_digest).digest, // TODO egregious hack (MI)
                        }),
                    }))
                } else {
                    Ok(Response::new(RelayerBlockInfoResponse {
                        successful: true,
                        block_info: None,
                    }))
                }
            }
            // TODO propagate error message to client
            Err(_) => Ok(Response::new(RelayerBlockInfoResponse {
                successful: false,
                block_info: None,
            })),
        }
    }
    async fn get_block_info(
        &self,
        request: Request<RelayerGetBlockInfoRequest>,
    ) -> Result<Response<RelayerBlockInfoResponse>, Status> {
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
                if let Some(block_info) = opt {
                    Ok(Response::new(RelayerBlockInfoResponse {
                        successful: true,
                        block_info: Some(RelayerBlockInfo {
                            block_number: block_info.block_number,
                            digest: CertificateDigestProto::from(block_info.block_digest).digest, // TODO egregious hack (MI)
                        }),
                    }))
                } else {
                    Ok(Response::new(RelayerBlockInfoResponse {
                        successful: true,
                        block_info: None,
                    }))
                }
            }
            // TODO propagate error message to client
            Err(_) => Ok(Response::new(RelayerBlockInfoResponse {
                successful: false,
                block_info: None,
            })),
        }
    }
    async fn get_block(
        &self,
        request: Request<RelayerGetBlockRequest>,
    ) -> Result<Response<RelayerBlockResponse>, Status> {
        let validator_state = &self.state;
        let req = request.into_inner();
        let block_number = req.block_number;

        match validator_state.validator_store.block_store.read(block_number).await {
            Ok(opt) => {
                if let Some(block) = opt {
                    let block_bytes = bincode::serialize(&block).unwrap().into();
                    Ok(Response::new(RelayerBlockResponse {
                        successful: true,
                        block: Some(RelayerBlock { block: block_bytes }),
                    }))
                } else {
                    Ok(Response::new(RelayerBlockResponse {
                        successful: true,
                        block: None,
                    }))
                }
            }
            // TODO propagate error message to client
            Err(_) => Ok(Response::new(RelayerBlockResponse {
                successful: false,
                block: None,
            })),
        }
    }
}
