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
                        block_info: Some(BlockInfoProto {
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
                        block_info: Some(BlockInfoProto {
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
                        block: Some(BlockProto { block: block_bytes }),
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
    async fn get_latest_orderbook_snap(
        &self,
        request: Request<RelayerGetLatestOrderbookSnapRequest>,
    ) -> Result<Response<RelayerLatestOrderbookSnapResponse>, Status> {
        let validator_state = &self.state;
        let req = request.into_inner();

        // request params
        let depth = req.depth;
        let base_asset_id = req.base_asset_id;
        let quote_asset_id = req.quote_asset_id;
        let orderbook_snap_key = format!("{}_{}", base_asset_id, quote_asset_id);

        let returned_value = validator_state
            .validator_store
            .latest_orderbook_snap_store
            .read(orderbook_snap_key)
            .await;
        println!("{:?}", returned_value.as_ref().unwrap());

        match returned_value {
            Ok(opt) => {
                if let Some(orderbook_snap) = opt {
                    let mut bids: Vec<DepthProto> = Vec::new();
                    let mut asks: Vec<DepthProto> = Vec::new();

                    let mut counter: u64 = 0;
                    for i in 0..orderbook_snap.bids.len() {
                        if counter >= depth {
                            break;
                        }
                        let bid = &orderbook_snap.bids[orderbook_snap.bids.len() - 1 - i];
                        bids.push(DepthProto {
                            price: bid.price,
                            quantity: bid.quantity,
                        });
                        counter += 1;
                    }
                    for i in 0..orderbook_snap.asks.len() {
                        if counter >= depth {
                            break;
                        }
                        let ask = &orderbook_snap.asks[orderbook_snap.asks.len() - 1 - i];
                        asks.push(DepthProto {
                            price: ask.price,
                            quantity: ask.quantity,
                        });
                        counter += 1;
                    }
                    return Ok(Response::new(RelayerLatestOrderbookSnapResponse { bids, asks }));
                } else {
                    let bids: Vec<Depth> = Vec::new();
                    let asks: Vec<Depth> = Vec::new();
                    return Ok(Response::new(RelayerLatestOrderbookSnapResponse { bids, asks }));
                }
            }
            // TODO propagate error message to client
            Err(_) => {
                let bids: Vec<Depth> = Vec::new();
                let asks: Vec<Depth> = Vec::new();
                return Ok(Response::new(RelayerLatestOrderbookSnapResponse { bids, asks }));
            }
        }
    }
    async fn get_latest_metrics(
        &self,
        _request: Request<RelayerMetricsRequest>,
    ) -> Result<Response<RelayerMetricsResponse>, Status> {
        let validator_state = &self.state;
        let metrics = &validator_state.metrics;

        Ok(Response::new(RelayerMetricsResponse {
            average_latency: metrics.get_average_latency_in_milis(),
            average_tps: metrics.get_average_tps(),
        }))
    }
}
