use crate::validator::state::ValidatorState;
use bytes::Bytes;
use gdex_types::account::AccountPubKey;
use gdex_types::crypto::ToFromBytes;
use gdex_types::proto::*;
use gdex_types::transaction::ConsensusTransaction;
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
        let returned_value = validator_state
            .validator_store
            .post_process_store
            .last_block_info_store
            .read(0)
            .await;

        match returned_value {
            Ok(opt) => {
                if let Some(block_info) = opt {
                    Ok(Response::new(RelayerBlockInfoResponse {
                        successful: true,
                        block_info: Some(BlockInfo {
                            block_number: block_info.block_number,
                            digest: CertificateDigestProto::from(block_info.block_digest).digest, // TODO egregious hack (MI)
                        }),
                    }))
                } else {
                    Err(Status::not_found("Latest block info was not found."))
                }
            }
            Err(err) => Err(Status::unknown(err.to_string())),
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
            .post_process_store
            .block_info_store
            .read(block_number)
            .await
        {
            Ok(opt) => {
                if let Some(block_info) = opt {
                    Ok(Response::new(RelayerBlockInfoResponse {
                        successful: true,
                        block_info: Some(BlockInfo {
                            block_number: block_info.block_number,
                            // TODO clean up this egregious hack (MI)
                            digest: CertificateDigestProto::from(block_info.block_digest).digest,
                        }),
                    }))
                } else {
                    Err(Status::not_found("Block info was not found."))
                }
            }
            Err(err) => Err(Status::unknown(err.to_string())),
        }
    }
    async fn get_block(
        &self,
        request: Request<RelayerGetBlockRequest>,
    ) -> Result<Response<RelayerBlockResponse>, Status> {
        let validator_state = &self.state;
        let req = request.into_inner();
        let block_number = req.block_number;

        match validator_state
            .validator_store
            .post_process_store
            .block_store
            .read(block_number)
            .await
        {
            Ok(opt) => {
                if let Some(block) = opt {
                    let mut executed_transactions = Vec::new();
                    for (serialized_transaction, exec_result) in block.transactions.iter() {
                        let consensus_transaction: ConsensusTransaction =
                            ConsensusTransaction::deserialize(serialized_transaction.clone())
                                .map_err(|_| Status::unknown("Error deserializing consensus transaction"))?;

                        let signed_transaction: SignedTransaction = consensus_transaction
                            .get_payload()
                            .map_err(|_| Status::unknown("Error deserializing transactions"))?;

                        let transaction_digest = Bytes::from(
                            signed_transaction
                                .get_transaction_digest()
                                .map_err(|_| Status::unknown("Error calculating transaction digest"))?
                                .get_array()
                                .to_vec(),
                        );

                        let exec_string = match exec_result {
                            Ok(_) => "Success".to_string(),
                            Err(err) => format!("Error: {}", err.to_string()),
                        };

                        let executed_transaction = ExecutedTransaction {
                            digest: transaction_digest,
                            result: exec_string,
                            signed_transaction: Some(signed_transaction),
                        };
                        executed_transactions.push(executed_transaction);
                    }

                    Ok(Response::new(RelayerBlockResponse {
                        successful: true,
                        block: Some(Block { executed_transactions }),
                    }))
                } else {
                    Err(Status::not_found("Block was not found."))
                }
            }
            Err(err) => Err(Status::unknown(err.to_string())),
        }
    }
    async fn get_latest_orderbook_depth(
        &self,
        request: Request<RelayerGetLatestOrderbookDepthRequest>,
    ) -> Result<Response<RelayerLatestOrderbookDepthResponse>, Status> {
        let validator_state = &self.state;
        let req = request.into_inner();

        // request params
        let depth = req.depth;
        let base_asset_id = req.base_asset_id;
        let quote_asset_id = req.quote_asset_id;
        let orderbook_depth_key = validator_state
            .controller_router
            .spot_controller
            .lock()
            .unwrap()
            .get_orderbook_key(base_asset_id, quote_asset_id);

        let returned_value = validator_state
            .validator_store
            .post_process_store
            .latest_orderbook_depth_store
            .read(orderbook_depth_key)
            .await;

        match returned_value {
            Ok(opt) => {
                if let Some(orderbook_depth) = opt {
                    let mut bids: Vec<Depth> = Vec::new();
                    let mut asks: Vec<Depth> = Vec::new();

                    let mut counter: u64 = 0;
                    for i in 0..orderbook_depth.bids.len() {
                        if counter >= depth {
                            break;
                        }
                        let bid = &orderbook_depth.bids[orderbook_depth.bids.len() - 1 - i];
                        bids.push(Depth {
                            price: bid.price,
                            quantity: bid.quantity,
                        });
                        counter += 1;
                    }
                    for i in 0..orderbook_depth.asks.len() {
                        if counter >= depth {
                            break;
                        }
                        let ask = &orderbook_depth.asks[orderbook_depth.asks.len() - 1 - i];
                        asks.push(Depth {
                            price: ask.price,
                            quantity: ask.quantity,
                        });
                        counter += 1;
                    }
                    return Ok(Response::new(RelayerLatestOrderbookDepthResponse { bids, asks }));
                } else {
                    Err(Status::not_found("Orderbook depth was not found."))
                }
            }
            // Propogate a tonic error to client
            Err(err) => Err(Status::unknown(err.to_string())),
        }
    }
    async fn get_latest_catchup_state(
        &self,
        _request: Request<RelayerGetLatestCatchupStateRequest>,
    ) -> Result<Response<RelayerLatestCatchupStateResponse>, Status> {
        let validator_state = &self.state;
        let returned_value = validator_state
            .validator_store
            .post_process_store
            .catchup_state_store
            .read(0)
            .await;

        match returned_value {
            Ok(opt) => {
                if let Some(catchup_state) = opt {
                    let catchup_state_bytes = bincode::serialize(&catchup_state).unwrap().into();
                    return Ok(Response::new(RelayerLatestCatchupStateResponse {
                        block_number: catchup_state.block_number,
                        state: catchup_state_bytes,
                    }));
                }
                Err(Status::not_found("Catchup state was not found."))
            }
            Err(err) => Err(Status::unknown(err.to_string())),
        }
    }
    async fn get_futures_user(
        &self,
        request: Request<RelayerGetFuturesUserRequest>,
    ) -> Result<Response<RelayerFuturesUserResponse>, Status> {
        let validator_state = &self.state;
        let req = request.into_inner();
        let futures_controller = validator_state.controller_router.futures_controller.lock().unwrap();

        let market_admin = AccountPubKey::from_bytes(&req.market_admin)
            .map_err(|_| Status::unknown("Could not load market admin address"))?;
        let user =
            AccountPubKey::from_bytes(&req.user).map_err(|_| Status::unknown("Could not load requester address"))?;

        let user_unrealized_pnl = futures_controller
            .get_account_unrealized_pnl(&market_admin, &user)
            .map_err(|_| Status::unknown("Could not load user pnl"))?;
        let user_deposit = futures_controller
            .get_account_available_deposit(&market_admin, &user)
            .map_err(|_| Status::unknown("Could not load user deposit"))?;
        let user_total_collateral_req = futures_controller
            .get_account_total_req_collateral(&market_admin, &user)
            .map_err(|_| Status::unknown("Could not load user total collateral required"))?;

        let user_market_state = futures_controller
            .get_account_state_by_market(&market_admin, &user)
            .map_err(|_| Status::unknown("Could not load position data"))?
            .iter()
            .map(|account_state| FuturesUserByMarket {
                quote_asset_id: account_state.0,
                orders: account_state.1.clone(),
                position: account_state.2.clone(),
            })
            .collect();

        Ok(Response::new(RelayerFuturesUserResponse {
            market_state: user_market_state,
            unrealized_pnl: user_unrealized_pnl,
            deposit: user_deposit,
            total_collateral_req: user_total_collateral_req,
        }))
    }
    async fn get_futures_markets(
        &self,
        request: Request<RelayerGetFuturesMarketsRequest>,
    ) -> Result<Response<RelayerFuturesMarketsResponse>, Status> {
        let validator_state = &self.state;
        let req = request.into_inner();
        let futures_controller = validator_state.controller_router.futures_controller.lock().unwrap();

        let market_admin = AccountPubKey::from_bytes(&req.market_admin)
            .map_err(|_| Status::unknown("Could not load market admin address"))?;

        let marketplace = futures_controller
            .get_marketplace_state(&market_admin)
            .map_err(|_| Status::unknown("Could not load marketplace"))?;

        let market_data: Vec<FuturesMarket> = marketplace
            .1
            .iter()
            .map(|market| FuturesMarket {
                oracle_price: market.oracle_price,
                last_traded_price: market.last_traded_price,
                open_interest: market.open_interest,
                max_leverage: market.max_leverage,
                base_asset_id: market.base_asset_id,
            })
            .collect();

        Ok(Response::new(RelayerFuturesMarketsResponse {
            market_data,
            quote_asset_id: marketplace.0,
        }))
    }
    async fn get_latest_metrics(
        &self,
        _request: Request<RelayerMetricsRequest>,
    ) -> Result<Response<RelayerMetricsResponse>, Status> {
        let validator_state = &self.state;
        let metrics = &validator_state.metrics;

        Ok(Response::new(RelayerMetricsResponse {
            average_latency: metrics.get_average_latency_in_micros(),
            average_tps: metrics.get_average_tps(),
        }))
    }
}
