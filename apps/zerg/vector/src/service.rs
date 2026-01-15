//! Vector gRPC service implementation
//!
//! This module contains the VectorServiceImpl struct and its gRPC trait implementation.
//! Handlers leverage From/TryFrom trait conversions defined in domain_vector::conversions.

use std::sync::Arc;

use domain_vector::{
    CreateCollection, RecommendQuery, SearchQuery, TenantContext, Vector, VectorRepository,
    VectorResult, VectorService,
    conversions::{
        self, embedding_model_from_proto, embedding_provider_from_proto,
        search_results_to_recommend_response, search_results_to_response, vector_config_from_proto,
    },
};
use rpc::vector::{
    CreateCollectionRequest, CreateCollectionResponse, DeleteCollectionRequest,
    DeleteCollectionResponse, DeleteRequest, DeleteResponse, EmbedBatchRequest, EmbedBatchResponse,
    EmbedRequest, EmbedResponse, EmbeddingResult as ProtoEmbeddingResult, GetCollectionRequest,
    GetCollectionResponse, GetRequest, GetResponse, ListCollectionsRequest,
    ListCollectionsResponse, RecommendRequest, RecommendResponse, SearchRequest, SearchResponse,
    SearchWithEmbeddingRequest, UpsertBatchRequest, UpsertBatchResponse, UpsertRequest,
    UpsertResponse, UpsertWithEmbeddingRequest,
    vector_service_server::VectorService as VectorServiceTrait,
};
use tonic::{Request, Response, Status};
use tracing::info;

/// gRPC service implementation for vector operations
///
/// Wraps the domain VectorService and handles proto ↔ domain conversions.
/// Generic over the repository type for testability.
pub struct VectorServiceImpl<R>
where
    R: VectorRepository + 'static,
{
    service: Arc<VectorService<R>>,
}

impl<R> VectorServiceImpl<R>
where
    R: VectorRepository + 'static,
{
    /// Create a new vector service implementation
    pub fn new(service: VectorService<R>) -> Self {
        Self {
            service: Arc::new(service),
        }
    }
}

#[tonic::async_trait]
impl<R> VectorServiceTrait for VectorServiceImpl<R>
where
    R: VectorRepository + 'static,
{
    // ===== Collection Management =====

    async fn create_collection(
        &self,
        request: Request<CreateCollectionRequest>,
    ) -> Result<Response<CreateCollectionResponse>, Status> {
        let req = request.into_inner();
        let tenant: TenantContext = req.tenant.try_into().map_err(Status::from)?;
        let config = vector_config_from_proto(req.config);

        let input = CreateCollection {
            name: req.collection_name,
            config,
        };

        let info = self
            .service
            .create_collection(&tenant, input)
            .await
            .map_err(Status::from)?;

        info!("Created collection: {}", info.name);

        Ok(Response::new(CreateCollectionResponse {
            full_collection_name: info.name,
            created: true,
        }))
    }

    async fn delete_collection(
        &self,
        request: Request<DeleteCollectionRequest>,
    ) -> Result<Response<DeleteCollectionResponse>, Status> {
        let req = request.into_inner();
        let tenant: TenantContext = req.tenant.try_into().map_err(Status::from)?;

        let deleted = self
            .service
            .delete_collection(&tenant, &req.collection_name)
            .await
            .map_err(Status::from)?;

        info!("Deleted collection: {}", req.collection_name);

        Ok(Response::new(DeleteCollectionResponse { deleted }))
    }

    async fn get_collection(
        &self,
        request: Request<GetCollectionRequest>,
    ) -> Result<Response<GetCollectionResponse>, Status> {
        let req = request.into_inner();
        let tenant: TenantContext = req.tenant.try_into().map_err(Status::from)?;

        let info = self
            .service
            .get_collection(&tenant, &req.collection_name)
            .await
            .map_err(Status::from)?
            .ok_or_else(|| Status::not_found("Collection not found"))?;

        Ok(Response::new(GetCollectionResponse {
            info: Some(info.into()),
        }))
    }

    async fn list_collections(
        &self,
        request: Request<ListCollectionsRequest>,
    ) -> Result<Response<ListCollectionsResponse>, Status> {
        let req = request.into_inner();
        let tenant: TenantContext = req.tenant.try_into().map_err(Status::from)?;

        let collections = self
            .service
            .list_collections(&tenant)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(ListCollectionsResponse {
            collections: collections.into_iter().map(Into::into).collect(),
        }))
    }

    // ===== Vector Operations =====

    async fn upsert(
        &self,
        request: Request<UpsertRequest>,
    ) -> Result<Response<UpsertResponse>, Status> {
        let req = request.into_inner();
        let tenant: TenantContext = req.tenant.try_into().map_err(Status::from)?;
        let vector: Vector = req.vector.try_into().map_err(Status::from)?;

        let id = self
            .service
            .upsert(&tenant, &req.collection_name, vector, req.wait)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(UpsertResponse {
            id: id.as_bytes().to_vec(),
            status: "completed".to_string(),
        }))
    }

    async fn upsert_batch(
        &self,
        request: Request<UpsertBatchRequest>,
    ) -> Result<Response<UpsertBatchResponse>, Status> {
        let req = request.into_inner();
        let tenant: TenantContext = req.tenant.try_into().map_err(Status::from)?;

        let vectors: VectorResult<Vec<Vector>> =
            req.vectors.into_iter().map(|v| v.try_into()).collect();
        let vectors = vectors.map_err(Status::from)?;

        let count = vectors.len() as u32;
        let ids = self
            .service
            .upsert_batch(&tenant, &req.collection_name, vectors, req.wait)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(UpsertBatchResponse {
            ids: ids.into_iter().map(|id| id.as_bytes().to_vec()).collect(),
            status: "completed".to_string(),
            upserted_count: count,
        }))
    }

    async fn search(
        &self,
        request: Request<SearchRequest>,
    ) -> Result<Response<SearchResponse>, Status> {
        let req = request.into_inner();
        let tenant: TenantContext = req.tenant.try_into().map_err(Status::from)?;

        let query = SearchQuery {
            vector: req.query_vector,
            limit: req.limit,
            score_threshold: req.score_threshold,
            filter: None, // TODO: Parse filter
            with_vectors: req.with_vectors,
            with_payloads: req.with_payloads,
        };

        let results = self
            .service
            .search(&tenant, &req.collection_name, query)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(search_results_to_response(results)))
    }

    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        let req = request.into_inner();
        let tenant: TenantContext = req.tenant.try_into().map_err(Status::from)?;

        let ids: VectorResult<Vec<uuid::Uuid>> = req
            .ids
            .iter()
            .map(|b| conversions::bytes_to_uuid(b))
            .collect();
        let ids = ids.map_err(Status::from)?;

        let vectors = self
            .service
            .get(
                &tenant,
                &req.collection_name,
                ids,
                req.with_vectors,
                req.with_payloads,
            )
            .await
            .map_err(Status::from)?;

        Ok(Response::new(GetResponse {
            vectors: vectors.into_iter().map(Into::into).collect(),
        }))
    }

    async fn delete(
        &self,
        request: Request<DeleteRequest>,
    ) -> Result<Response<DeleteResponse>, Status> {
        let req = request.into_inner();
        let tenant: TenantContext = req.tenant.try_into().map_err(Status::from)?;

        let ids: VectorResult<Vec<uuid::Uuid>> = req
            .ids
            .iter()
            .map(|b| conversions::bytes_to_uuid(b))
            .collect();
        let ids = ids.map_err(Status::from)?;

        let deleted_count = self
            .service
            .delete(&tenant, &req.collection_name, ids, req.wait)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(DeleteResponse {
            deleted_count,
            status: "completed".to_string(),
        }))
    }

    // ===== Embedding Operations =====

    async fn embed(
        &self,
        request: Request<EmbedRequest>,
    ) -> Result<Response<EmbedResponse>, Status> {
        let req = request.into_inner();
        let provider_type = embedding_provider_from_proto(req.provider);
        let model = embedding_model_from_proto(req.model, req.custom_dimension);

        let result = self
            .service
            .embed(provider_type, model, &req.text)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(EmbedResponse {
            embedding: result.values,
            dimension: result.dimension,
            tokens_used: result.tokens_used,
        }))
    }

    async fn embed_batch(
        &self,
        request: Request<EmbedBatchRequest>,
    ) -> Result<Response<EmbedBatchResponse>, Status> {
        let req = request.into_inner();
        let provider_type = embedding_provider_from_proto(req.provider);
        let model = embedding_model_from_proto(req.model, req.custom_dimension);

        let results = self
            .service
            .embed_batch(provider_type, model, &req.texts)
            .await
            .map_err(Status::from)?;

        let total_tokens: u32 = results.iter().map(|r| r.tokens_used).sum();

        Ok(Response::new(EmbedBatchResponse {
            embeddings: results
                .into_iter()
                .map(|r| ProtoEmbeddingResult {
                    values: r.values,
                    dimension: r.dimension,
                })
                .collect(),
            total_tokens,
        }))
    }

    // ===== Combined Operations =====

    async fn upsert_with_embedding(
        &self,
        request: Request<UpsertWithEmbeddingRequest>,
    ) -> Result<Response<UpsertResponse>, Status> {
        let req = request.into_inner();
        let tenant: TenantContext = req.tenant.try_into().map_err(Status::from)?;
        let id = conversions::bytes_to_uuid(&req.id).map_err(Status::from)?;
        let provider_type = embedding_provider_from_proto(req.provider);
        let model = embedding_model_from_proto(req.model, None);

        let payload = req.payload.and_then(|p| {
            if p.json.is_empty() {
                None
            } else {
                serde_json::from_slice(&p.json).ok()
            }
        });

        let result_id = self
            .service
            .upsert_with_embedding(
                &tenant,
                &req.collection_name,
                id,
                &req.text,
                payload,
                provider_type,
                model,
                req.wait,
            )
            .await
            .map_err(Status::from)?;

        Ok(Response::new(UpsertResponse {
            id: result_id.as_bytes().to_vec(),
            status: "completed".to_string(),
        }))
    }

    async fn search_with_embedding(
        &self,
        request: Request<SearchWithEmbeddingRequest>,
    ) -> Result<Response<SearchResponse>, Status> {
        let req = request.into_inner();
        let tenant: TenantContext = req.tenant.try_into().map_err(Status::from)?;
        let provider_type = embedding_provider_from_proto(req.provider);
        let model = embedding_model_from_proto(req.model, None);

        let results = self
            .service
            .search_with_embedding(
                &tenant,
                &req.collection_name,
                &req.text,
                req.limit,
                req.score_threshold,
                req.with_vectors,
                req.with_payloads,
                provider_type,
                model,
            )
            .await
            .map_err(Status::from)?;

        Ok(Response::new(search_results_to_response(results)))
    }

    // ===== Recommendations =====

    async fn recommend(
        &self,
        request: Request<RecommendRequest>,
    ) -> Result<Response<RecommendResponse>, Status> {
        let req = request.into_inner();
        let tenant: TenantContext = req.tenant.try_into().map_err(Status::from)?;

        let positive_ids: VectorResult<Vec<uuid::Uuid>> = req
            .positive_ids
            .iter()
            .map(|b| conversions::bytes_to_uuid(b))
            .collect();
        let positive_ids = positive_ids.map_err(Status::from)?;

        let negative_ids: VectorResult<Vec<uuid::Uuid>> = req
            .negative_ids
            .iter()
            .map(|b| conversions::bytes_to_uuid(b))
            .collect();
        let negative_ids = negative_ids.map_err(Status::from)?;

        let query = RecommendQuery {
            positive_ids,
            negative_ids,
            limit: req.limit,
            score_threshold: req.score_threshold,
            filter: None, // TODO: Parse filter
            with_vectors: req.with_vectors,
            with_payloads: req.with_payloads,
        };

        let results = self
            .service
            .recommend(&tenant, &req.collection_name, query)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(search_results_to_recommend_response(results)))
    }
}
