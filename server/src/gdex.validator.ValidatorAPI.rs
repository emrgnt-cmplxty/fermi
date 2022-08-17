/// Generated client implementations.
pub mod validator_a_p_i_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    ///The Validator API
    #[derive(Debug, Clone)]
    pub struct ValidatorAPIClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl ValidatorAPIClient<tonic::transport::Channel> {
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
    impl<T> ValidatorAPIClient<T>
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
        ) -> ValidatorAPIClient<InterceptedService<T, F>>
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
            ValidatorAPIClient::new(InterceptedService::new(inner, interceptor))
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
        pub async fn transaction(
            &mut self,
            request: impl tonic::IntoRequest<gdex_types::transaction::SignedTransaction>,
        ) -> Result<
            tonic::Response<gdex_types::node::TransactionResult>,
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
            let codec = crate::codec::BincodeCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/gdex.validator.ValidatorAPI/SignedTransaction",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn stream_transaction(
            &mut self,
            request: impl tonic::IntoStreamingRequest<
                Message = gdex_types::transaction::SignedTransaction,
            >,
        ) -> Result<
            tonic::Response<gdex_types::node::TransactionResult>,
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
            let codec = crate::codec::BincodeCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/gdex.validator.ValidatorAPI/SignedTransactionStream",
            );
            self.inner
                .client_streaming(request.into_streaming_request(), path, codec)
                .await
        }
    }
}
/// Generated server implementations.
pub mod validator_a_p_i_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    ///Generated trait containing gRPC methods that should be implemented for use with ValidatorAPIServer.
    #[async_trait]
    pub trait ValidatorAPI: Send + Sync + 'static {
        async fn transaction(
            &self,
            request: tonic::Request<gdex_types::transaction::SignedTransaction>,
        ) -> Result<tonic::Response<gdex_types::node::TransactionResult>, tonic::Status>;
        async fn stream_transaction(
            &self,
            request: tonic::Request<
                tonic::Streaming<gdex_types::transaction::SignedTransaction>,
            >,
        ) -> Result<tonic::Response<gdex_types::node::TransactionResult>, tonic::Status>;
    }
    ///The Validator API
    #[derive(Debug)]
    pub struct ValidatorAPIServer<T: ValidatorAPI> {
        inner: _Inner<T>,
        accept_compression_encodings: (),
        send_compression_encodings: (),
    }
    struct _Inner<T>(Arc<T>);
    impl<T: ValidatorAPI> ValidatorAPIServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for ValidatorAPIServer<T>
    where
        T: ValidatorAPI,
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
                "/gdex.validator.ValidatorAPI/SignedTransaction" => {
                    #[allow(non_camel_case_types)]
                    struct SignedTransactionSvc<T: ValidatorAPI>(pub Arc<T>);
                    impl<
                        T: ValidatorAPI,
                    > tonic::server::UnaryService<
                        gdex_types::transaction::SignedTransaction,
                    > for SignedTransactionSvc<T> {
                        type Response = gdex_types::node::TransactionResult;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                gdex_types::transaction::SignedTransaction,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).transaction(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SignedTransactionSvc(inner);
                        let codec = crate::codec::BincodeCodec::default();
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
                "/gdex.validator.ValidatorAPI/SignedTransactionStream" => {
                    #[allow(non_camel_case_types)]
                    struct SignedTransactionStreamSvc<T: ValidatorAPI>(pub Arc<T>);
                    impl<
                        T: ValidatorAPI,
                    > tonic::server::ClientStreamingService<
                        gdex_types::transaction::SignedTransaction,
                    > for SignedTransactionStreamSvc<T> {
                        type Response = gdex_types::node::TransactionResult;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                tonic::Streaming<gdex_types::transaction::SignedTransaction>,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).stream_transaction(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SignedTransactionStreamSvc(inner);
                        let codec = crate::codec::BincodeCodec::default();
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
    impl<T: ValidatorAPI> Clone for ValidatorAPIServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: ValidatorAPI> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: ValidatorAPI> tonic::transport::NamedService for ValidatorAPIServer<T> {
        const NAME: &'static str = "gdex.validator.ValidatorAPI";
    }
}
