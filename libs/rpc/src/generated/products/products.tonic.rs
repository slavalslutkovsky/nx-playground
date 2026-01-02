// @generated
/// Generated client implementations.
pub mod products_service_client {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value,
    )]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct ProductsServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl ProductsServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> ProductsServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::Body>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + std::marker::Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + std::marker::Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> ProductsServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::Body>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::Body>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::Body>,
            >>::Error: Into<StdError> + std::marker::Send + std::marker::Sync,
        {
            ProductsServiceClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        pub async fn create(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateRequest>,
        ) -> std::result::Result<tonic::Response<super::CreateResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/products.ProductsService/Create",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("products.ProductsService", "Create"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_by_id(
            &mut self,
            request: impl tonic::IntoRequest<super::GetByIdRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetByIdResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/products.ProductsService/GetById",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("products.ProductsService", "GetById"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_by_sku(
            &mut self,
            request: impl tonic::IntoRequest<super::GetBySkuRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetBySkuResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/products.ProductsService/GetBySku",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("products.ProductsService", "GetBySku"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_by_barcode(
            &mut self,
            request: impl tonic::IntoRequest<super::GetByBarcodeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetByBarcodeResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/products.ProductsService/GetByBarcode",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("products.ProductsService", "GetByBarcode"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_by_id(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateByIdRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpdateByIdResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/products.ProductsService/UpdateById",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("products.ProductsService", "UpdateById"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_by_id(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteByIdRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeleteByIdResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/products.ProductsService/DeleteById",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("products.ProductsService", "DeleteById"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn list(
            &mut self,
            request: impl tonic::IntoRequest<super::ListRequest>,
        ) -> std::result::Result<tonic::Response<super::ListResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/products.ProductsService/List",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("products.ProductsService", "List"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn list_stream(
            &mut self,
            request: impl tonic::IntoRequest<super::ListStreamRequest>,
        ) -> std::result::Result<
            tonic::Response<tonic::codec::Streaming<super::ListStreamResponse>>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/products.ProductsService/ListStream",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("products.ProductsService", "ListStream"));
            self.inner.server_streaming(req, path, codec).await
        }
        pub async fn search(
            &mut self,
            request: impl tonic::IntoRequest<super::SearchRequest>,
        ) -> std::result::Result<tonic::Response<super::SearchResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/products.ProductsService/Search",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("products.ProductsService", "Search"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn count(
            &mut self,
            request: impl tonic::IntoRequest<super::CountRequest>,
        ) -> std::result::Result<tonic::Response<super::CountResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/products.ProductsService/Count",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("products.ProductsService", "Count"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_by_category(
            &mut self,
            request: impl tonic::IntoRequest<super::GetByCategoryRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetByCategoryResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/products.ProductsService/GetByCategory",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("products.ProductsService", "GetByCategory"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_stock(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateStockRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpdateStockResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/products.ProductsService/UpdateStock",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("products.ProductsService", "UpdateStock"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn reserve_stock(
            &mut self,
            request: impl tonic::IntoRequest<super::ReserveStockRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ReservationResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/products.ProductsService/ReserveStock",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("products.ProductsService", "ReserveStock"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn release_stock(
            &mut self,
            request: impl tonic::IntoRequest<super::ReleaseStockRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ReleaseStockResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/products.ProductsService/ReleaseStock",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("products.ProductsService", "ReleaseStock"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn commit_stock(
            &mut self,
            request: impl tonic::IntoRequest<super::CommitStockRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CommitStockResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/products.ProductsService/CommitStock",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("products.ProductsService", "CommitStock"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_low_stock(
            &mut self,
            request: impl tonic::IntoRequest<super::GetLowStockRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetLowStockResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/products.ProductsService/GetLowStock",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("products.ProductsService", "GetLowStock"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn activate(
            &mut self,
            request: impl tonic::IntoRequest<super::ActivateRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ActivateResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/products.ProductsService/Activate",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("products.ProductsService", "Activate"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn deactivate(
            &mut self,
            request: impl tonic::IntoRequest<super::DeactivateRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeactivateResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/products.ProductsService/Deactivate",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("products.ProductsService", "Deactivate"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn discontinue(
            &mut self,
            request: impl tonic::IntoRequest<super::DiscontinueRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DiscontinueResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/products.ProductsService/Discontinue",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("products.ProductsService", "Discontinue"));
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod products_service_server {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value,
    )]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with ProductsServiceServer.
    #[async_trait]
    pub trait ProductsService: std::marker::Send + std::marker::Sync + 'static {
        async fn create(
            &self,
            request: tonic::Request<super::CreateRequest>,
        ) -> std::result::Result<tonic::Response<super::CreateResponse>, tonic::Status>;
        async fn get_by_id(
            &self,
            request: tonic::Request<super::GetByIdRequest>,
        ) -> std::result::Result<tonic::Response<super::GetByIdResponse>, tonic::Status>;
        async fn get_by_sku(
            &self,
            request: tonic::Request<super::GetBySkuRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetBySkuResponse>,
            tonic::Status,
        >;
        async fn get_by_barcode(
            &self,
            request: tonic::Request<super::GetByBarcodeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetByBarcodeResponse>,
            tonic::Status,
        >;
        async fn update_by_id(
            &self,
            request: tonic::Request<super::UpdateByIdRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpdateByIdResponse>,
            tonic::Status,
        >;
        async fn delete_by_id(
            &self,
            request: tonic::Request<super::DeleteByIdRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeleteByIdResponse>,
            tonic::Status,
        >;
        async fn list(
            &self,
            request: tonic::Request<super::ListRequest>,
        ) -> std::result::Result<tonic::Response<super::ListResponse>, tonic::Status>;
        /// Server streaming response type for the ListStream method.
        type ListStreamStream: tonic::codegen::tokio_stream::Stream<
                Item = std::result::Result<super::ListStreamResponse, tonic::Status>,
            >
            + std::marker::Send
            + 'static;
        async fn list_stream(
            &self,
            request: tonic::Request<super::ListStreamRequest>,
        ) -> std::result::Result<tonic::Response<Self::ListStreamStream>, tonic::Status>;
        async fn search(
            &self,
            request: tonic::Request<super::SearchRequest>,
        ) -> std::result::Result<tonic::Response<super::SearchResponse>, tonic::Status>;
        async fn count(
            &self,
            request: tonic::Request<super::CountRequest>,
        ) -> std::result::Result<tonic::Response<super::CountResponse>, tonic::Status>;
        async fn get_by_category(
            &self,
            request: tonic::Request<super::GetByCategoryRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetByCategoryResponse>,
            tonic::Status,
        >;
        async fn update_stock(
            &self,
            request: tonic::Request<super::UpdateStockRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpdateStockResponse>,
            tonic::Status,
        >;
        async fn reserve_stock(
            &self,
            request: tonic::Request<super::ReserveStockRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ReservationResponse>,
            tonic::Status,
        >;
        async fn release_stock(
            &self,
            request: tonic::Request<super::ReleaseStockRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ReleaseStockResponse>,
            tonic::Status,
        >;
        async fn commit_stock(
            &self,
            request: tonic::Request<super::CommitStockRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CommitStockResponse>,
            tonic::Status,
        >;
        async fn get_low_stock(
            &self,
            request: tonic::Request<super::GetLowStockRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetLowStockResponse>,
            tonic::Status,
        >;
        async fn activate(
            &self,
            request: tonic::Request<super::ActivateRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ActivateResponse>,
            tonic::Status,
        >;
        async fn deactivate(
            &self,
            request: tonic::Request<super::DeactivateRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeactivateResponse>,
            tonic::Status,
        >;
        async fn discontinue(
            &self,
            request: tonic::Request<super::DiscontinueRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DiscontinueResponse>,
            tonic::Status,
        >;
    }
    #[derive(Debug)]
    pub struct ProductsServiceServer<T> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    impl<T> ProductsServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
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
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for ProductsServiceServer<T>
    where
        T: ProductsService,
        B: Body + std::marker::Send + 'static,
        B::Error: Into<StdError> + std::marker::Send + 'static,
    {
        type Response = http::Response<tonic::body::Body>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            match req.uri().path() {
                "/products.ProductsService/Create" => {
                    #[allow(non_camel_case_types)]
                    struct CreateSvc<T: ProductsService>(pub Arc<T>);
                    impl<
                        T: ProductsService,
                    > tonic::server::UnaryService<super::CreateRequest>
                    for CreateSvc<T> {
                        type Response = super::CreateResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ProductsService>::create(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = CreateSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/products.ProductsService/GetById" => {
                    #[allow(non_camel_case_types)]
                    struct GetByIdSvc<T: ProductsService>(pub Arc<T>);
                    impl<
                        T: ProductsService,
                    > tonic::server::UnaryService<super::GetByIdRequest>
                    for GetByIdSvc<T> {
                        type Response = super::GetByIdResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetByIdRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ProductsService>::get_by_id(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = GetByIdSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/products.ProductsService/GetBySku" => {
                    #[allow(non_camel_case_types)]
                    struct GetBySkuSvc<T: ProductsService>(pub Arc<T>);
                    impl<
                        T: ProductsService,
                    > tonic::server::UnaryService<super::GetBySkuRequest>
                    for GetBySkuSvc<T> {
                        type Response = super::GetBySkuResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetBySkuRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ProductsService>::get_by_sku(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = GetBySkuSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/products.ProductsService/GetByBarcode" => {
                    #[allow(non_camel_case_types)]
                    struct GetByBarcodeSvc<T: ProductsService>(pub Arc<T>);
                    impl<
                        T: ProductsService,
                    > tonic::server::UnaryService<super::GetByBarcodeRequest>
                    for GetByBarcodeSvc<T> {
                        type Response = super::GetByBarcodeResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetByBarcodeRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ProductsService>::get_by_barcode(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = GetByBarcodeSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/products.ProductsService/UpdateById" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateByIdSvc<T: ProductsService>(pub Arc<T>);
                    impl<
                        T: ProductsService,
                    > tonic::server::UnaryService<super::UpdateByIdRequest>
                    for UpdateByIdSvc<T> {
                        type Response = super::UpdateByIdResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateByIdRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ProductsService>::update_by_id(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = UpdateByIdSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/products.ProductsService/DeleteById" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteByIdSvc<T: ProductsService>(pub Arc<T>);
                    impl<
                        T: ProductsService,
                    > tonic::server::UnaryService<super::DeleteByIdRequest>
                    for DeleteByIdSvc<T> {
                        type Response = super::DeleteByIdResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteByIdRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ProductsService>::delete_by_id(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = DeleteByIdSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/products.ProductsService/List" => {
                    #[allow(non_camel_case_types)]
                    struct ListSvc<T: ProductsService>(pub Arc<T>);
                    impl<
                        T: ProductsService,
                    > tonic::server::UnaryService<super::ListRequest> for ListSvc<T> {
                        type Response = super::ListResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ProductsService>::list(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = ListSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/products.ProductsService/ListStream" => {
                    #[allow(non_camel_case_types)]
                    struct ListStreamSvc<T: ProductsService>(pub Arc<T>);
                    impl<
                        T: ProductsService,
                    > tonic::server::ServerStreamingService<super::ListStreamRequest>
                    for ListStreamSvc<T> {
                        type Response = super::ListStreamResponse;
                        type ResponseStream = T::ListStreamStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListStreamRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ProductsService>::list_stream(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = ListStreamSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/products.ProductsService/Search" => {
                    #[allow(non_camel_case_types)]
                    struct SearchSvc<T: ProductsService>(pub Arc<T>);
                    impl<
                        T: ProductsService,
                    > tonic::server::UnaryService<super::SearchRequest>
                    for SearchSvc<T> {
                        type Response = super::SearchResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SearchRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ProductsService>::search(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = SearchSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/products.ProductsService/Count" => {
                    #[allow(non_camel_case_types)]
                    struct CountSvc<T: ProductsService>(pub Arc<T>);
                    impl<
                        T: ProductsService,
                    > tonic::server::UnaryService<super::CountRequest> for CountSvc<T> {
                        type Response = super::CountResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CountRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ProductsService>::count(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = CountSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/products.ProductsService/GetByCategory" => {
                    #[allow(non_camel_case_types)]
                    struct GetByCategorySvc<T: ProductsService>(pub Arc<T>);
                    impl<
                        T: ProductsService,
                    > tonic::server::UnaryService<super::GetByCategoryRequest>
                    for GetByCategorySvc<T> {
                        type Response = super::GetByCategoryResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetByCategoryRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ProductsService>::get_by_category(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = GetByCategorySvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/products.ProductsService/UpdateStock" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateStockSvc<T: ProductsService>(pub Arc<T>);
                    impl<
                        T: ProductsService,
                    > tonic::server::UnaryService<super::UpdateStockRequest>
                    for UpdateStockSvc<T> {
                        type Response = super::UpdateStockResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateStockRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ProductsService>::update_stock(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = UpdateStockSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/products.ProductsService/ReserveStock" => {
                    #[allow(non_camel_case_types)]
                    struct ReserveStockSvc<T: ProductsService>(pub Arc<T>);
                    impl<
                        T: ProductsService,
                    > tonic::server::UnaryService<super::ReserveStockRequest>
                    for ReserveStockSvc<T> {
                        type Response = super::ReservationResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ReserveStockRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ProductsService>::reserve_stock(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = ReserveStockSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/products.ProductsService/ReleaseStock" => {
                    #[allow(non_camel_case_types)]
                    struct ReleaseStockSvc<T: ProductsService>(pub Arc<T>);
                    impl<
                        T: ProductsService,
                    > tonic::server::UnaryService<super::ReleaseStockRequest>
                    for ReleaseStockSvc<T> {
                        type Response = super::ReleaseStockResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ReleaseStockRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ProductsService>::release_stock(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = ReleaseStockSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/products.ProductsService/CommitStock" => {
                    #[allow(non_camel_case_types)]
                    struct CommitStockSvc<T: ProductsService>(pub Arc<T>);
                    impl<
                        T: ProductsService,
                    > tonic::server::UnaryService<super::CommitStockRequest>
                    for CommitStockSvc<T> {
                        type Response = super::CommitStockResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CommitStockRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ProductsService>::commit_stock(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = CommitStockSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/products.ProductsService/GetLowStock" => {
                    #[allow(non_camel_case_types)]
                    struct GetLowStockSvc<T: ProductsService>(pub Arc<T>);
                    impl<
                        T: ProductsService,
                    > tonic::server::UnaryService<super::GetLowStockRequest>
                    for GetLowStockSvc<T> {
                        type Response = super::GetLowStockResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetLowStockRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ProductsService>::get_low_stock(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = GetLowStockSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/products.ProductsService/Activate" => {
                    #[allow(non_camel_case_types)]
                    struct ActivateSvc<T: ProductsService>(pub Arc<T>);
                    impl<
                        T: ProductsService,
                    > tonic::server::UnaryService<super::ActivateRequest>
                    for ActivateSvc<T> {
                        type Response = super::ActivateResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ActivateRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ProductsService>::activate(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = ActivateSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/products.ProductsService/Deactivate" => {
                    #[allow(non_camel_case_types)]
                    struct DeactivateSvc<T: ProductsService>(pub Arc<T>);
                    impl<
                        T: ProductsService,
                    > tonic::server::UnaryService<super::DeactivateRequest>
                    for DeactivateSvc<T> {
                        type Response = super::DeactivateResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeactivateRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ProductsService>::deactivate(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = DeactivateSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/products.ProductsService/Discontinue" => {
                    #[allow(non_camel_case_types)]
                    struct DiscontinueSvc<T: ProductsService>(pub Arc<T>);
                    impl<
                        T: ProductsService,
                    > tonic::server::UnaryService<super::DiscontinueRequest>
                    for DiscontinueSvc<T> {
                        type Response = super::DiscontinueResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DiscontinueRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ProductsService>::discontinue(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = DiscontinueSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        let mut response = http::Response::new(
                            tonic::body::Body::default(),
                        );
                        let headers = response.headers_mut();
                        headers
                            .insert(
                                tonic::Status::GRPC_STATUS,
                                (tonic::Code::Unimplemented as i32).into(),
                            );
                        headers
                            .insert(
                                http::header::CONTENT_TYPE,
                                tonic::metadata::GRPC_CONTENT_TYPE,
                            );
                        Ok(response)
                    })
                }
            }
        }
    }
    impl<T> Clone for ProductsServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    /// Generated gRPC service name
    pub const SERVICE_NAME: &str = "products.ProductsService";
    impl<T> tonic::server::NamedService for ProductsServiceServer<T> {
        const NAME: &'static str = SERVICE_NAME;
    }
}
