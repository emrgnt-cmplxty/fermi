// INTERFACE

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Empty {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FaucetAirdropRequest {
    #[prost(string, tag="1")]
    pub airdrop_to: ::prost::alloc::string::String,
    #[prost(uint64, tag="2")]
    pub amount: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FaucetAirdropResponse {
    #[prost(bool, tag="1")]
    pub successful: bool,
}
// block info

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RelayerGetLatestBlockInfoRequest {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RelayerGetBlockInfoRequest {
    #[prost(uint64, tag="1")]
    pub block_number: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RelayerBlockInfoResponse {
    #[prost(bool, tag="1")]
    pub successful: bool,
    #[prost(message, optional, tag="2")]
    pub block_info: ::core::option::Option<super::block::BlockInfo>,
}
// block

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RelayerGetBlockRequest {
    #[prost(uint64, tag="1")]
    pub block_number: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RelayerBlock {
    #[prost(bytes="bytes", tag="1")]
    pub block: ::prost::bytes::Bytes,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RelayerBlockResponse {
    #[prost(bool, tag="1")]
    pub successful: bool,
    #[prost(message, optional, tag="2")]
    pub block: ::core::option::Option<RelayerBlock>,
}
// orderbook depth

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RelayerGetLatestOrderbookDepthRequest {
    #[prost(uint64, tag="1")]
    pub base_asset_id: u64,
    #[prost(uint64, tag="2")]
    pub quote_asset_id: u64,
    #[prost(uint64, tag="3")]
    pub depth: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Depth {
    #[prost(uint64, tag="1")]
    pub price: u64,
    #[prost(uint64, tag="2")]
    pub quantity: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RelayerLatestOrderbookDepthResponse {
    #[prost(message, repeated, tag="1")]
    pub bids: ::prost::alloc::vec::Vec<Depth>,
    #[prost(message, repeated, tag="2")]
    pub asks: ::prost::alloc::vec::Vec<Depth>,
}
// fast catchup state

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RelayerGetLatestCatchupStateRequest {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RelayerLatestCatchupStateResponse {
    #[prost(bytes="bytes", tag="1")]
    pub state: ::prost::bytes::Bytes,
    #[prost(uint64, tag="2")]
    pub block_number: u64,
}
// futures orders

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RelayerGetFuturesUserRequest {
    #[prost(bytes="bytes", tag="1")]
    pub user: ::prost::bytes::Bytes,
    #[prost(bytes="bytes", tag="2")]
    pub market_admin: ::prost::bytes::Bytes,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FuturesPosition {
    #[prost(uint64, tag="1")]
    pub quantity: u64,
    #[prost(uint64, tag="2")]
    pub side: u64,
    #[prost(uint64, tag="3")]
    pub average_price: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FuturesOrder {
    #[prost(uint64, tag="1")]
    pub order_id: u64,
    #[prost(uint64, tag="2")]
    pub side: u64,
    #[prost(uint64, tag="3")]
    pub quantity: u64,
    #[prost(uint64, tag="4")]
    pub price: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FuturesUserByMarket {
    #[prost(message, repeated, tag="1")]
    pub orders: ::prost::alloc::vec::Vec<FuturesOrder>,
    #[prost(message, optional, tag="2")]
    pub position: ::core::option::Option<FuturesPosition>,
    #[prost(uint64, tag="3")]
    pub quote_asset_id: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RelayerFuturesUserResponse {
    #[prost(message, repeated, tag="1")]
    pub market_state: ::prost::alloc::vec::Vec<FuturesUserByMarket>,
    #[prost(uint64, tag="2")]
    pub total_collateral_req: u64,
    #[prost(int64, tag="3")]
    pub unrealized_pnl: i64,
    #[prost(int64, tag="4")]
    pub deposit: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RelayerGetFuturesMarketsRequest {
    #[prost(bytes="bytes", tag="1")]
    pub market_admin: ::prost::bytes::Bytes,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FuturesMarket {
    #[prost(uint64, tag="1")]
    pub oracle_price: u64,
    #[prost(uint64, tag="2")]
    pub last_traded_price: u64,
    #[prost(uint64, tag="3")]
    pub open_interest: u64,
    #[prost(uint64, tag="4")]
    pub max_leverage: u64,
    #[prost(uint64, tag="5")]
    pub base_asset_id: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RelayerFuturesMarketsResponse {
    #[prost(message, repeated, tag="1")]
    pub market_data: ::prost::alloc::vec::Vec<FuturesMarket>,
    #[prost(uint64, tag="2")]
    pub quote_asset_id: u64,
}
// metrics

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RelayerMetricsRequest {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RelayerMetricsResponse {
    #[prost(uint64, tag="1")]
    pub average_latency: u64,
    #[prost(double, tag="2")]
    pub average_tps: f64,
}
/// Generated client implementations.
pub mod transaction_submitter_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct TransactionSubmitterClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl TransactionSubmitterClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> TransactionSubmitterClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> TransactionSubmitterClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            TransactionSubmitterClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with `gzip`.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_gzip(mut self) -> Self {
            self.inner = self.inner.send_gzip();
            self
        }
        /// Enable decompressing responses with `gzip`.
        #[must_use]
        pub fn accept_gzip(mut self) -> Self {
            self.inner = self.inner.accept_gzip();
            self
        }
        /// submit a transaction
        pub async fn submit_transaction(
            &mut self,
            request: impl tonic::IntoRequest<
                super::super::transaction::SignedTransaction,
            >,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/services.TransactionSubmitter/SubmitTransaction",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// submit a transaction via stream
        pub async fn submit_transaction_stream(
            &mut self,
            request: impl tonic::IntoStreamingRequest<
                Message = super::super::transaction::SignedTransaction,
            >,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/services.TransactionSubmitter/SubmitTransactionStream",
            );
            self.inner
                .client_streaming(request.into_streaming_request(), path, codec)
                .await
        }
    }
}
/// Generated client implementations.
pub mod faucet_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct FaucetClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl FaucetClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> FaucetClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> FaucetClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            FaucetClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with `gzip`.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_gzip(mut self) -> Self {
            self.inner = self.inner.send_gzip();
            self
        }
        /// Enable decompressing responses with `gzip`.
        #[must_use]
        pub fn accept_gzip(mut self) -> Self {
            self.inner = self.inner.accept_gzip();
            self
        }
        /// request airdrop
        pub async fn airdrop(
            &mut self,
            request: impl tonic::IntoRequest<super::FaucetAirdropRequest>,
        ) -> Result<tonic::Response<super::FaucetAirdropResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/services.Faucet/Airdrop");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated client implementations.
pub mod relayer_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct RelayerClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl RelayerClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> RelayerClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> RelayerClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            RelayerClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with `gzip`.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_gzip(mut self) -> Self {
            self.inner = self.inner.send_gzip();
            self
        }
        /// Enable decompressing responses with `gzip`.
        #[must_use]
        pub fn accept_gzip(mut self) -> Self {
            self.inner = self.inner.accept_gzip();
            self
        }
        /// request latest block info
        pub async fn get_latest_block_info(
            &mut self,
            request: impl tonic::IntoRequest<super::RelayerGetLatestBlockInfoRequest>,
        ) -> Result<tonic::Response<super::RelayerBlockInfoResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/services.Relayer/GetLatestBlockInfo",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// request block info by block number
        pub async fn get_block_info(
            &mut self,
            request: impl tonic::IntoRequest<super::RelayerGetBlockInfoRequest>,
        ) -> Result<tonic::Response<super::RelayerBlockInfoResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/services.Relayer/GetBlockInfo",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// request block (includes transactions)
        pub async fn get_block(
            &mut self,
            request: impl tonic::IntoRequest<super::RelayerGetBlockRequest>,
        ) -> Result<tonic::Response<super::RelayerBlockResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/services.Relayer/GetBlock",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// request orderbook depth snapshot
        pub async fn get_latest_orderbook_depth(
            &mut self,
            request: impl tonic::IntoRequest<
                super::RelayerGetLatestOrderbookDepthRequest,
            >,
        ) -> Result<
            tonic::Response<super::RelayerLatestOrderbookDepthResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/services.Relayer/GetLatestOrderbookDepth",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// request catchup state
        pub async fn get_latest_catchup_state(
            &mut self,
            request: impl tonic::IntoRequest<super::RelayerGetLatestCatchupStateRequest>,
        ) -> Result<
            tonic::Response<super::RelayerLatestCatchupStateResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/services.Relayer/GetLatestCatchupState",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// request futures positions
        pub async fn get_futures_user(
            &mut self,
            request: impl tonic::IntoRequest<super::RelayerGetFuturesUserRequest>,
        ) -> Result<tonic::Response<super::RelayerFuturesUserResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/services.Relayer/GetFuturesUser",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// request orderbook metrics
        pub async fn get_futures_markets(
            &mut self,
            request: impl tonic::IntoRequest<super::RelayerGetFuturesMarketsRequest>,
        ) -> Result<
            tonic::Response<super::RelayerFuturesMarketsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/services.Relayer/GetFuturesMarkets",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// request metrics
        pub async fn get_latest_metrics(
            &mut self,
            request: impl tonic::IntoRequest<super::RelayerMetricsRequest>,
        ) -> Result<tonic::Response<super::RelayerMetricsResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/services.Relayer/GetLatestMetrics",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod transaction_submitter_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    ///Generated trait containing gRPC methods that should be implemented for use with TransactionSubmitterServer.
    #[async_trait]
    pub trait TransactionSubmitter: Send + Sync + 'static {
        /// submit a transaction
        async fn submit_transaction(
            &self,
            request: tonic::Request<super::super::transaction::SignedTransaction>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status>;
        /// submit a transaction via stream
        async fn submit_transaction_stream(
            &self,
            request: tonic::Request<
                tonic::Streaming<super::super::transaction::SignedTransaction>,
            >,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct TransactionSubmitterServer<T: TransactionSubmitter> {
        inner: _Inner<T>,
        accept_compression_encodings: (),
        send_compression_encodings: (),
    }
    struct _Inner<T>(Arc<T>);
    impl<T: TransactionSubmitter> TransactionSubmitterServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>>
    for TransactionSubmitterServer<T>
    where
        T: TransactionSubmitter,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/services.TransactionSubmitter/SubmitTransaction" => {
                    #[allow(non_camel_case_types)]
                    struct SubmitTransactionSvc<T: TransactionSubmitter>(pub Arc<T>);
                    impl<
                        T: TransactionSubmitter,
                    > tonic::server::UnaryService<
                        super::super::transaction::SignedTransaction,
                    > for SubmitTransactionSvc<T> {
                        type Response = super::Empty;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::transaction::SignedTransaction,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).submit_transaction(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SubmitTransactionSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/services.TransactionSubmitter/SubmitTransactionStream" => {
                    #[allow(non_camel_case_types)]
                    struct SubmitTransactionStreamSvc<T: TransactionSubmitter>(
                        pub Arc<T>,
                    );
                    impl<
                        T: TransactionSubmitter,
                    > tonic::server::ClientStreamingService<
                        super::super::transaction::SignedTransaction,
                    > for SubmitTransactionStreamSvc<T> {
                        type Response = super::Empty;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                tonic::Streaming<
                                    super::super::transaction::SignedTransaction,
                                >,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).submit_transaction_stream(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SubmitTransactionStreamSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.client_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: TransactionSubmitter> Clone for TransactionSubmitterServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: TransactionSubmitter> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: TransactionSubmitter> tonic::transport::NamedService
    for TransactionSubmitterServer<T> {
        const NAME: &'static str = "services.TransactionSubmitter";
    }
}
/// Generated server implementations.
pub mod faucet_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    ///Generated trait containing gRPC methods that should be implemented for use with FaucetServer.
    #[async_trait]
    pub trait Faucet: Send + Sync + 'static {
        /// request airdrop
        async fn airdrop(
            &self,
            request: tonic::Request<super::FaucetAirdropRequest>,
        ) -> Result<tonic::Response<super::FaucetAirdropResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct FaucetServer<T: Faucet> {
        inner: _Inner<T>,
        accept_compression_encodings: (),
        send_compression_encodings: (),
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Faucet> FaucetServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for FaucetServer<T>
    where
        T: Faucet,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/services.Faucet/Airdrop" => {
                    #[allow(non_camel_case_types)]
                    struct AirdropSvc<T: Faucet>(pub Arc<T>);
                    impl<
                        T: Faucet,
                    > tonic::server::UnaryService<super::FaucetAirdropRequest>
                    for AirdropSvc<T> {
                        type Response = super::FaucetAirdropResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::FaucetAirdropRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).airdrop(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = AirdropSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: Faucet> Clone for FaucetServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: Faucet> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Faucet> tonic::transport::NamedService for FaucetServer<T> {
        const NAME: &'static str = "services.Faucet";
    }
}
/// Generated server implementations.
pub mod relayer_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    ///Generated trait containing gRPC methods that should be implemented for use with RelayerServer.
    #[async_trait]
    pub trait Relayer: Send + Sync + 'static {
        /// request latest block info
        async fn get_latest_block_info(
            &self,
            request: tonic::Request<super::RelayerGetLatestBlockInfoRequest>,
        ) -> Result<tonic::Response<super::RelayerBlockInfoResponse>, tonic::Status>;
        /// request block info by block number
        async fn get_block_info(
            &self,
            request: tonic::Request<super::RelayerGetBlockInfoRequest>,
        ) -> Result<tonic::Response<super::RelayerBlockInfoResponse>, tonic::Status>;
        /// request block (includes transactions)
        async fn get_block(
            &self,
            request: tonic::Request<super::RelayerGetBlockRequest>,
        ) -> Result<tonic::Response<super::RelayerBlockResponse>, tonic::Status>;
        /// request orderbook depth snapshot
        async fn get_latest_orderbook_depth(
            &self,
            request: tonic::Request<super::RelayerGetLatestOrderbookDepthRequest>,
        ) -> Result<
            tonic::Response<super::RelayerLatestOrderbookDepthResponse>,
            tonic::Status,
        >;
        /// request catchup state
        async fn get_latest_catchup_state(
            &self,
            request: tonic::Request<super::RelayerGetLatestCatchupStateRequest>,
        ) -> Result<
            tonic::Response<super::RelayerLatestCatchupStateResponse>,
            tonic::Status,
        >;
        /// request futures positions
        async fn get_futures_user(
            &self,
            request: tonic::Request<super::RelayerGetFuturesUserRequest>,
        ) -> Result<tonic::Response<super::RelayerFuturesUserResponse>, tonic::Status>;
        /// request orderbook metrics
        async fn get_futures_markets(
            &self,
            request: tonic::Request<super::RelayerGetFuturesMarketsRequest>,
        ) -> Result<
            tonic::Response<super::RelayerFuturesMarketsResponse>,
            tonic::Status,
        >;
        /// request metrics
        async fn get_latest_metrics(
            &self,
            request: tonic::Request<super::RelayerMetricsRequest>,
        ) -> Result<tonic::Response<super::RelayerMetricsResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct RelayerServer<T: Relayer> {
        inner: _Inner<T>,
        accept_compression_encodings: (),
        send_compression_encodings: (),
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Relayer> RelayerServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for RelayerServer<T>
    where
        T: Relayer,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/services.Relayer/GetLatestBlockInfo" => {
                    #[allow(non_camel_case_types)]
                    struct GetLatestBlockInfoSvc<T: Relayer>(pub Arc<T>);
                    impl<
                        T: Relayer,
                    > tonic::server::UnaryService<
                        super::RelayerGetLatestBlockInfoRequest,
                    > for GetLatestBlockInfoSvc<T> {
                        type Response = super::RelayerBlockInfoResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::RelayerGetLatestBlockInfoRequest,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).get_latest_block_info(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetLatestBlockInfoSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/services.Relayer/GetBlockInfo" => {
                    #[allow(non_camel_case_types)]
                    struct GetBlockInfoSvc<T: Relayer>(pub Arc<T>);
                    impl<
                        T: Relayer,
                    > tonic::server::UnaryService<super::RelayerGetBlockInfoRequest>
                    for GetBlockInfoSvc<T> {
                        type Response = super::RelayerBlockInfoResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RelayerGetBlockInfoRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).get_block_info(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetBlockInfoSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/services.Relayer/GetBlock" => {
                    #[allow(non_camel_case_types)]
                    struct GetBlockSvc<T: Relayer>(pub Arc<T>);
                    impl<
                        T: Relayer,
                    > tonic::server::UnaryService<super::RelayerGetBlockRequest>
                    for GetBlockSvc<T> {
                        type Response = super::RelayerBlockResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RelayerGetBlockRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_block(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetBlockSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/services.Relayer/GetLatestOrderbookDepth" => {
                    #[allow(non_camel_case_types)]
                    struct GetLatestOrderbookDepthSvc<T: Relayer>(pub Arc<T>);
                    impl<
                        T: Relayer,
                    > tonic::server::UnaryService<
                        super::RelayerGetLatestOrderbookDepthRequest,
                    > for GetLatestOrderbookDepthSvc<T> {
                        type Response = super::RelayerLatestOrderbookDepthResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::RelayerGetLatestOrderbookDepthRequest,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).get_latest_orderbook_depth(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetLatestOrderbookDepthSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/services.Relayer/GetLatestCatchupState" => {
                    #[allow(non_camel_case_types)]
                    struct GetLatestCatchupStateSvc<T: Relayer>(pub Arc<T>);
                    impl<
                        T: Relayer,
                    > tonic::server::UnaryService<
                        super::RelayerGetLatestCatchupStateRequest,
                    > for GetLatestCatchupStateSvc<T> {
                        type Response = super::RelayerLatestCatchupStateResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::RelayerGetLatestCatchupStateRequest,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).get_latest_catchup_state(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetLatestCatchupStateSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/services.Relayer/GetFuturesUser" => {
                    #[allow(non_camel_case_types)]
                    struct GetFuturesUserSvc<T: Relayer>(pub Arc<T>);
                    impl<
                        T: Relayer,
                    > tonic::server::UnaryService<super::RelayerGetFuturesUserRequest>
                    for GetFuturesUserSvc<T> {
                        type Response = super::RelayerFuturesUserResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RelayerGetFuturesUserRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).get_futures_user(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetFuturesUserSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/services.Relayer/GetFuturesMarkets" => {
                    #[allow(non_camel_case_types)]
                    struct GetFuturesMarketsSvc<T: Relayer>(pub Arc<T>);
                    impl<
                        T: Relayer,
                    > tonic::server::UnaryService<super::RelayerGetFuturesMarketsRequest>
                    for GetFuturesMarketsSvc<T> {
                        type Response = super::RelayerFuturesMarketsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::RelayerGetFuturesMarketsRequest,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).get_futures_markets(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetFuturesMarketsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/services.Relayer/GetLatestMetrics" => {
                    #[allow(non_camel_case_types)]
                    struct GetLatestMetricsSvc<T: Relayer>(pub Arc<T>);
                    impl<
                        T: Relayer,
                    > tonic::server::UnaryService<super::RelayerMetricsRequest>
                    for GetLatestMetricsSvc<T> {
                        type Response = super::RelayerMetricsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RelayerMetricsRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).get_latest_metrics(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetLatestMetricsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: Relayer> Clone for RelayerServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: Relayer> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Relayer> tonic::transport::NamedService for RelayerServer<T> {
        const NAME: &'static str = "services.Relayer";
    }
}
