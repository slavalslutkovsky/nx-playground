// @generated
/// Generated client implementations.
pub mod vector_service_client {
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
    pub struct VectorServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl VectorServiceClient<tonic::transport::Channel> {
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
    impl<T> VectorServiceClient<T>
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
        ) -> VectorServiceClient<InterceptedService<T, F>>
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
            VectorServiceClient::new(InterceptedService::new(inner, interceptor))
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
        pub async fn create_collection(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateCollectionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateCollectionResponse>,
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
                "/vector.v1.VectorService/CreateCollection",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("vector.v1.VectorService", "CreateCollection"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_collection(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteCollectionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeleteCollectionResponse>,
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
                "/vector.v1.VectorService/DeleteCollection",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("vector.v1.VectorService", "DeleteCollection"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_collection(
            &mut self,
            request: impl tonic::IntoRequest<super::GetCollectionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetCollectionResponse>,
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
                "/vector.v1.VectorService/GetCollection",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("vector.v1.VectorService", "GetCollection"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn list_collections(
            &mut self,
            request: impl tonic::IntoRequest<super::ListCollectionsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListCollectionsResponse>,
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
                "/vector.v1.VectorService/ListCollections",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("vector.v1.VectorService", "ListCollections"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn upsert(
            &mut self,
            request: impl tonic::IntoRequest<super::UpsertRequest>,
        ) -> std::result::Result<tonic::Response<super::UpsertResponse>, tonic::Status> {
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
                "/vector.v1.VectorService/Upsert",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("vector.v1.VectorService", "Upsert"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn upsert_batch(
            &mut self,
            request: impl tonic::IntoRequest<super::UpsertBatchRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpsertBatchResponse>,
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
                "/vector.v1.VectorService/UpsertBatch",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("vector.v1.VectorService", "UpsertBatch"));
            self.inner.unary(req, path, codec).await
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
                "/vector.v1.VectorService/Search",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("vector.v1.VectorService", "Search"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get(
            &mut self,
            request: impl tonic::IntoRequest<super::GetRequest>,
        ) -> std::result::Result<tonic::Response<super::GetResponse>, tonic::Status> {
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
                "/vector.v1.VectorService/Get",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("vector.v1.VectorService", "Get"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteRequest>,
        ) -> std::result::Result<tonic::Response<super::DeleteResponse>, tonic::Status> {
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
                "/vector.v1.VectorService/Delete",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("vector.v1.VectorService", "Delete"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn embed(
            &mut self,
            request: impl tonic::IntoRequest<super::EmbedRequest>,
        ) -> std::result::Result<tonic::Response<super::EmbedResponse>, tonic::Status> {
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
                "/vector.v1.VectorService/Embed",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("vector.v1.VectorService", "Embed"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn embed_batch(
            &mut self,
            request: impl tonic::IntoRequest<super::EmbedBatchRequest>,
        ) -> std::result::Result<
            tonic::Response<super::EmbedBatchResponse>,
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
                "/vector.v1.VectorService/EmbedBatch",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("vector.v1.VectorService", "EmbedBatch"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn upsert_with_embedding(
            &mut self,
            request: impl tonic::IntoRequest<super::UpsertWithEmbeddingRequest>,
        ) -> std::result::Result<tonic::Response<super::UpsertResponse>, tonic::Status> {
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
                "/vector.v1.VectorService/UpsertWithEmbedding",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("vector.v1.VectorService", "UpsertWithEmbedding"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn search_with_embedding(
            &mut self,
            request: impl tonic::IntoRequest<super::SearchWithEmbeddingRequest>,
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
                "/vector.v1.VectorService/SearchWithEmbedding",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("vector.v1.VectorService", "SearchWithEmbedding"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn recommend(
            &mut self,
            request: impl tonic::IntoRequest<super::RecommendRequest>,
        ) -> std::result::Result<
            tonic::Response<super::RecommendResponse>,
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
                "/vector.v1.VectorService/Recommend",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("vector.v1.VectorService", "Recommend"));
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod vector_service_server {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value,
    )]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with VectorServiceServer.
    #[async_trait]
    pub trait VectorService: std::marker::Send + std::marker::Sync + 'static {
        async fn create_collection(
            &self,
            request: tonic::Request<super::CreateCollectionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateCollectionResponse>,
            tonic::Status,
        >;
        async fn delete_collection(
            &self,
            request: tonic::Request<super::DeleteCollectionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeleteCollectionResponse>,
            tonic::Status,
        >;
        async fn get_collection(
            &self,
            request: tonic::Request<super::GetCollectionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetCollectionResponse>,
            tonic::Status,
        >;
        async fn list_collections(
            &self,
            request: tonic::Request<super::ListCollectionsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListCollectionsResponse>,
            tonic::Status,
        >;
        async fn upsert(
            &self,
            request: tonic::Request<super::UpsertRequest>,
        ) -> std::result::Result<tonic::Response<super::UpsertResponse>, tonic::Status>;
        async fn upsert_batch(
            &self,
            request: tonic::Request<super::UpsertBatchRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpsertBatchResponse>,
            tonic::Status,
        >;
        async fn search(
            &self,
            request: tonic::Request<super::SearchRequest>,
        ) -> std::result::Result<tonic::Response<super::SearchResponse>, tonic::Status>;
        async fn get(
            &self,
            request: tonic::Request<super::GetRequest>,
        ) -> std::result::Result<tonic::Response<super::GetResponse>, tonic::Status>;
        async fn delete(
            &self,
            request: tonic::Request<super::DeleteRequest>,
        ) -> std::result::Result<tonic::Response<super::DeleteResponse>, tonic::Status>;
        async fn embed(
            &self,
            request: tonic::Request<super::EmbedRequest>,
        ) -> std::result::Result<tonic::Response<super::EmbedResponse>, tonic::Status>;
        async fn embed_batch(
            &self,
            request: tonic::Request<super::EmbedBatchRequest>,
        ) -> std::result::Result<
            tonic::Response<super::EmbedBatchResponse>,
            tonic::Status,
        >;
        async fn upsert_with_embedding(
            &self,
            request: tonic::Request<super::UpsertWithEmbeddingRequest>,
        ) -> std::result::Result<tonic::Response<super::UpsertResponse>, tonic::Status>;
        async fn search_with_embedding(
            &self,
            request: tonic::Request<super::SearchWithEmbeddingRequest>,
        ) -> std::result::Result<tonic::Response<super::SearchResponse>, tonic::Status>;
        async fn recommend(
            &self,
            request: tonic::Request<super::RecommendRequest>,
        ) -> std::result::Result<
            tonic::Response<super::RecommendResponse>,
            tonic::Status,
        >;
    }
    #[derive(Debug)]
    pub struct VectorServiceServer<T> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    impl<T> VectorServiceServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for VectorServiceServer<T>
    where
        T: VectorService,
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
                "/vector.v1.VectorService/CreateCollection" => {
                    #[allow(non_camel_case_types)]
                    struct CreateCollectionSvc<T: VectorService>(pub Arc<T>);
                    impl<
                        T: VectorService,
                    > tonic::server::UnaryService<super::CreateCollectionRequest>
                    for CreateCollectionSvc<T> {
                        type Response = super::CreateCollectionResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateCollectionRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as VectorService>::create_collection(&inner, request)
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
                        let method = CreateCollectionSvc(inner);
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
                "/vector.v1.VectorService/DeleteCollection" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteCollectionSvc<T: VectorService>(pub Arc<T>);
                    impl<
                        T: VectorService,
                    > tonic::server::UnaryService<super::DeleteCollectionRequest>
                    for DeleteCollectionSvc<T> {
                        type Response = super::DeleteCollectionResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteCollectionRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as VectorService>::delete_collection(&inner, request)
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
                        let method = DeleteCollectionSvc(inner);
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
                "/vector.v1.VectorService/GetCollection" => {
                    #[allow(non_camel_case_types)]
                    struct GetCollectionSvc<T: VectorService>(pub Arc<T>);
                    impl<
                        T: VectorService,
                    > tonic::server::UnaryService<super::GetCollectionRequest>
                    for GetCollectionSvc<T> {
                        type Response = super::GetCollectionResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetCollectionRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as VectorService>::get_collection(&inner, request).await
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
                        let method = GetCollectionSvc(inner);
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
                "/vector.v1.VectorService/ListCollections" => {
                    #[allow(non_camel_case_types)]
                    struct ListCollectionsSvc<T: VectorService>(pub Arc<T>);
                    impl<
                        T: VectorService,
                    > tonic::server::UnaryService<super::ListCollectionsRequest>
                    for ListCollectionsSvc<T> {
                        type Response = super::ListCollectionsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListCollectionsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as VectorService>::list_collections(&inner, request)
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
                        let method = ListCollectionsSvc(inner);
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
                "/vector.v1.VectorService/Upsert" => {
                    #[allow(non_camel_case_types)]
                    struct UpsertSvc<T: VectorService>(pub Arc<T>);
                    impl<
                        T: VectorService,
                    > tonic::server::UnaryService<super::UpsertRequest>
                    for UpsertSvc<T> {
                        type Response = super::UpsertResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpsertRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as VectorService>::upsert(&inner, request).await
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
                        let method = UpsertSvc(inner);
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
                "/vector.v1.VectorService/UpsertBatch" => {
                    #[allow(non_camel_case_types)]
                    struct UpsertBatchSvc<T: VectorService>(pub Arc<T>);
                    impl<
                        T: VectorService,
                    > tonic::server::UnaryService<super::UpsertBatchRequest>
                    for UpsertBatchSvc<T> {
                        type Response = super::UpsertBatchResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpsertBatchRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as VectorService>::upsert_batch(&inner, request).await
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
                        let method = UpsertBatchSvc(inner);
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
                "/vector.v1.VectorService/Search" => {
                    #[allow(non_camel_case_types)]
                    struct SearchSvc<T: VectorService>(pub Arc<T>);
                    impl<
                        T: VectorService,
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
                                <T as VectorService>::search(&inner, request).await
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
                "/vector.v1.VectorService/Get" => {
                    #[allow(non_camel_case_types)]
                    struct GetSvc<T: VectorService>(pub Arc<T>);
                    impl<T: VectorService> tonic::server::UnaryService<super::GetRequest>
                    for GetSvc<T> {
                        type Response = super::GetResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as VectorService>::get(&inner, request).await
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
                        let method = GetSvc(inner);
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
                "/vector.v1.VectorService/Delete" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteSvc<T: VectorService>(pub Arc<T>);
                    impl<
                        T: VectorService,
                    > tonic::server::UnaryService<super::DeleteRequest>
                    for DeleteSvc<T> {
                        type Response = super::DeleteResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as VectorService>::delete(&inner, request).await
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
                        let method = DeleteSvc(inner);
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
                "/vector.v1.VectorService/Embed" => {
                    #[allow(non_camel_case_types)]
                    struct EmbedSvc<T: VectorService>(pub Arc<T>);
                    impl<
                        T: VectorService,
                    > tonic::server::UnaryService<super::EmbedRequest> for EmbedSvc<T> {
                        type Response = super::EmbedResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::EmbedRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as VectorService>::embed(&inner, request).await
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
                        let method = EmbedSvc(inner);
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
                "/vector.v1.VectorService/EmbedBatch" => {
                    #[allow(non_camel_case_types)]
                    struct EmbedBatchSvc<T: VectorService>(pub Arc<T>);
                    impl<
                        T: VectorService,
                    > tonic::server::UnaryService<super::EmbedBatchRequest>
                    for EmbedBatchSvc<T> {
                        type Response = super::EmbedBatchResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::EmbedBatchRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as VectorService>::embed_batch(&inner, request).await
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
                        let method = EmbedBatchSvc(inner);
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
                "/vector.v1.VectorService/UpsertWithEmbedding" => {
                    #[allow(non_camel_case_types)]
                    struct UpsertWithEmbeddingSvc<T: VectorService>(pub Arc<T>);
                    impl<
                        T: VectorService,
                    > tonic::server::UnaryService<super::UpsertWithEmbeddingRequest>
                    for UpsertWithEmbeddingSvc<T> {
                        type Response = super::UpsertResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpsertWithEmbeddingRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as VectorService>::upsert_with_embedding(&inner, request)
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
                        let method = UpsertWithEmbeddingSvc(inner);
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
                "/vector.v1.VectorService/SearchWithEmbedding" => {
                    #[allow(non_camel_case_types)]
                    struct SearchWithEmbeddingSvc<T: VectorService>(pub Arc<T>);
                    impl<
                        T: VectorService,
                    > tonic::server::UnaryService<super::SearchWithEmbeddingRequest>
                    for SearchWithEmbeddingSvc<T> {
                        type Response = super::SearchResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SearchWithEmbeddingRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as VectorService>::search_with_embedding(&inner, request)
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
                        let method = SearchWithEmbeddingSvc(inner);
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
                "/vector.v1.VectorService/Recommend" => {
                    #[allow(non_camel_case_types)]
                    struct RecommendSvc<T: VectorService>(pub Arc<T>);
                    impl<
                        T: VectorService,
                    > tonic::server::UnaryService<super::RecommendRequest>
                    for RecommendSvc<T> {
                        type Response = super::RecommendResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RecommendRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as VectorService>::recommend(&inner, request).await
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
                        let method = RecommendSvc(inner);
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
    impl<T> Clone for VectorServiceServer<T> {
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
    pub const SERVICE_NAME: &str = "vector.v1.VectorService";
    impl<T> tonic::server::NamedService for VectorServiceServer<T> {
        const NAME: &'static str = SERVICE_NAME;
    }
}
