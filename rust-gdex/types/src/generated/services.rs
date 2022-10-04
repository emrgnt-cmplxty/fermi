// INTERFACE

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Empty {
}
// block info

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LatestBlockInfoRequest {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockInfoRequest {
    #[prost(uint64, tag="1")]
    pub block_number: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockInfoResponse {
    #[prost(bool, tag="1")]
    pub successful: bool,
    #[prost(bytes="bytes", tag="2")]
    pub serialized_block_info: ::prost::bytes::Bytes,
}
// block

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockRequest {
    #[prost(uint64, tag="1")]
    pub block_number: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockResponse {
    #[prost(bool, tag="1")]
    pub successful: bool,
    #[prost(bytes="bytes", tag="2")]
    pub serialized_block: ::prost::bytes::Bytes,
}
// metrics

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MetricsRequest {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MetricsResponse {
    #[prost(uint64, tag="1")]
    pub average_latency: u64,
    #[prost(double, tag="2")]
    pub average_tps: f64,
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
/// Generated client implementations.
pub mod validator_grpc_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct ValidatorGrpcClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl ValidatorGrpcClient<tonic::transport::Channel> {
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
    impl<T> ValidatorGrpcClient<T>
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
        ) -> ValidatorGrpcClient<InterceptedService<T, F>>
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
            ValidatorGrpcClient::new(InterceptedService::new(inner, interceptor))
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
            request: impl tonic::IntoRequest<super::LatestBlockInfoRequest>,
        ) -> Result<tonic::Response<super::BlockInfoResponse>, tonic::Status> {
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
                "/services.ValidatorGRPC/GetLatestBlockInfo",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// request block info by block number
        pub async fn get_block_info(
            &mut self,
            request: impl tonic::IntoRequest<super::BlockInfoRequest>,
        ) -> Result<tonic::Response<super::BlockInfoResponse>, tonic::Status> {
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
                "/services.ValidatorGRPC/GetBlockInfo",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// request block (includes transactions)
        pub async fn get_block(
            &mut self,
            request: impl tonic::IntoRequest<super::BlockRequest>,
        ) -> Result<tonic::Response<super::BlockResponse>, tonic::Status> {
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
                "/services.ValidatorGRPC/GetBlock",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// request metrics
        pub async fn get_latest_metrics(
            &mut self,
            request: impl tonic::IntoRequest<super::MetricsRequest>,
        ) -> Result<tonic::Response<super::MetricsResponse>, tonic::Status> {
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
                "/services.ValidatorGRPC/GetLatestMetrics",
            );
            self.inner.unary(request.into_request(), path, codec).await
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
                "/services.ValidatorGRPC/SubmitTransaction",
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
                "/services.ValidatorGRPC/SubmitTransactionStream",
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
/// Generated server implementations.
pub mod validator_grpc_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    ///Generated trait containing gRPC methods that should be implemented for use with ValidatorGrpcServer.
    #[async_trait]
    pub trait ValidatorGrpc: Send + Sync + 'static {
        /// request latest block info
        async fn get_latest_block_info(
            &self,
            request: tonic::Request<super::LatestBlockInfoRequest>,
        ) -> Result<tonic::Response<super::BlockInfoResponse>, tonic::Status>;
        /// request block info by block number
        async fn get_block_info(
            &self,
            request: tonic::Request<super::BlockInfoRequest>,
        ) -> Result<tonic::Response<super::BlockInfoResponse>, tonic::Status>;
        /// request block (includes transactions)
        async fn get_block(
            &self,
            request: tonic::Request<super::BlockRequest>,
        ) -> Result<tonic::Response<super::BlockResponse>, tonic::Status>;
        /// request metrics
        async fn get_latest_metrics(
            &self,
            request: tonic::Request<super::MetricsRequest>,
        ) -> Result<tonic::Response<super::MetricsResponse>, tonic::Status>;
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
    pub struct ValidatorGrpcServer<T: ValidatorGrpc> {
        inner: _Inner<T>,
        accept_compression_encodings: (),
        send_compression_encodings: (),
    }
    struct _Inner<T>(Arc<T>);
    impl<T: ValidatorGrpc> ValidatorGrpcServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for ValidatorGrpcServer<T>
    where
        T: ValidatorGrpc,
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
                "/services.ValidatorGRPC/GetLatestBlockInfo" => {
                    #[allow(non_camel_case_types)]
                    struct GetLatestBlockInfoSvc<T: ValidatorGrpc>(pub Arc<T>);
                    impl<
                        T: ValidatorGrpc,
                    > tonic::server::UnaryService<super::LatestBlockInfoRequest>
                    for GetLatestBlockInfoSvc<T> {
                        type Response = super::BlockInfoResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::LatestBlockInfoRequest>,
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
                "/services.ValidatorGRPC/GetBlockInfo" => {
                    #[allow(non_camel_case_types)]
                    struct GetBlockInfoSvc<T: ValidatorGrpc>(pub Arc<T>);
                    impl<
                        T: ValidatorGrpc,
                    > tonic::server::UnaryService<super::BlockInfoRequest>
                    for GetBlockInfoSvc<T> {
                        type Response = super::BlockInfoResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::BlockInfoRequest>,
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
                "/services.ValidatorGRPC/GetBlock" => {
                    #[allow(non_camel_case_types)]
                    struct GetBlockSvc<T: ValidatorGrpc>(pub Arc<T>);
                    impl<
                        T: ValidatorGrpc,
                    > tonic::server::UnaryService<super::BlockRequest>
                    for GetBlockSvc<T> {
                        type Response = super::BlockResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::BlockRequest>,
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
                "/services.ValidatorGRPC/GetLatestMetrics" => {
                    #[allow(non_camel_case_types)]
                    struct GetLatestMetricsSvc<T: ValidatorGrpc>(pub Arc<T>);
                    impl<
                        T: ValidatorGrpc,
                    > tonic::server::UnaryService<super::MetricsRequest>
                    for GetLatestMetricsSvc<T> {
                        type Response = super::MetricsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MetricsRequest>,
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
                "/services.ValidatorGRPC/SubmitTransaction" => {
                    #[allow(non_camel_case_types)]
                    struct SubmitTransactionSvc<T: ValidatorGrpc>(pub Arc<T>);
                    impl<
                        T: ValidatorGrpc,
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
                "/services.ValidatorGRPC/SubmitTransactionStream" => {
                    #[allow(non_camel_case_types)]
                    struct SubmitTransactionStreamSvc<T: ValidatorGrpc>(pub Arc<T>);
                    impl<
                        T: ValidatorGrpc,
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
    impl<T: ValidatorGrpc> Clone for ValidatorGrpcServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: ValidatorGrpc> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: ValidatorGrpc> tonic::transport::NamedService for ValidatorGrpcServer<T> {
        const NAME: &'static str = "services.ValidatorGRPC";
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
