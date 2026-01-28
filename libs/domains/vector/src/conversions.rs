use uuid::Uuid;

use crate::error::{VectorError, VectorResult};
use crate::models::{
    CollectionInfo, DistanceMetric, EmbeddingModel, EmbeddingProviderType, HnswConfig,
    SearchResult, TenantContext, Vector, VectorConfig,
};

// Import generated proto types
use rpc::vector::{
    CollectionInfo as ProtoCollectionInfo, DistanceMetric as ProtoDistance,
    EmbeddingModel as ProtoEmbeddingModel, EmbeddingProvider as ProtoEmbeddingProvider,
    HnswConfig as ProtoHnswConfig, Payload as ProtoPayload, RecommendResponse, SearchResponse,
    SearchResult as ProtoSearchResult, TenantContext as ProtoTenantContext, Vector as ProtoVector,
    VectorConfig as ProtoVectorConfig,
};

// ===== Tenant Context =====

impl TryFrom<Option<ProtoTenantContext>> for TenantContext {
    type Error = VectorError;

    fn try_from(proto: Option<ProtoTenantContext>) -> Result<Self, Self::Error> {
        let proto =
            proto.ok_or_else(|| VectorError::Validation("Missing tenant context".to_string()))?;
        Self::try_from(proto)
    }
}

impl TryFrom<ProtoTenantContext> for TenantContext {
    type Error = VectorError;

    fn try_from(proto: ProtoTenantContext) -> Result<Self, Self::Error> {
        let project_id = bytes_to_uuid(&proto.project_id)?;
        let namespace = proto.namespace.filter(|s| !s.is_empty());
        let user_id = match proto.user_id {
            Some(ref bytes) if !bytes.is_empty() => Some(bytes_to_uuid(bytes)?),
            _ => None,
        };

        Ok(TenantContext {
            project_id,
            namespace,
            user_id,
        })
    }
}

impl From<TenantContext> for ProtoTenantContext {
    fn from(ctx: TenantContext) -> Self {
        ProtoTenantContext {
            project_id: ctx.project_id.as_bytes().to_vec(),
            namespace: ctx.namespace,
            user_id: ctx.user_id.map(|id| id.as_bytes().to_vec()),
        }
    }
}

// ===== Distance Metric =====

pub fn distance_from_proto(proto: i32) -> DistanceMetric {
    match ProtoDistance::try_from(proto) {
        Ok(ProtoDistance::Cosine) => DistanceMetric::Cosine,
        Ok(ProtoDistance::Euclidean) => DistanceMetric::Euclidean,
        Ok(ProtoDistance::DotProduct) => DistanceMetric::DotProduct,
        Ok(ProtoDistance::Manhattan) => DistanceMetric::Manhattan,
        _ => DistanceMetric::Cosine,
    }
}

pub fn distance_to_proto(metric: DistanceMetric) -> i32 {
    match metric {
        DistanceMetric::Cosine => ProtoDistance::Cosine as i32,
        DistanceMetric::Euclidean => ProtoDistance::Euclidean as i32,
        DistanceMetric::DotProduct => ProtoDistance::DotProduct as i32,
        DistanceMetric::Manhattan => ProtoDistance::Manhattan as i32,
    }
}

// ===== HNSW Config =====

pub fn hnsw_from_proto(proto: Option<ProtoHnswConfig>) -> Option<HnswConfig> {
    proto.map(|h| HnswConfig {
        m: h.m,
        ef_construct: h.ef_construct,
        full_scan_threshold: h.full_scan_threshold,
    })
}

pub fn hnsw_to_proto(hnsw: Option<HnswConfig>) -> Option<ProtoHnswConfig> {
    hnsw.map(|h| ProtoHnswConfig {
        m: h.m,
        ef_construct: h.ef_construct,
        full_scan_threshold: h.full_scan_threshold,
    })
}

// ===== Vector Config =====

pub fn vector_config_from_proto(proto: Option<ProtoVectorConfig>) -> VectorConfig {
    match proto {
        Some(c) => VectorConfig {
            dimension: c.dimension,
            distance: distance_from_proto(c.distance),
            hnsw: hnsw_from_proto(c.hnsw),
        },
        None => VectorConfig::new(1536),
    }
}

impl From<VectorConfig> for ProtoVectorConfig {
    fn from(config: VectorConfig) -> Self {
        ProtoVectorConfig {
            dimension: config.dimension,
            distance: distance_to_proto(config.distance),
            hnsw: hnsw_to_proto(config.hnsw),
        }
    }
}

// ===== Collection Info =====

impl From<CollectionInfo> for ProtoCollectionInfo {
    fn from(info: CollectionInfo) -> Self {
        ProtoCollectionInfo {
            collection_name: info.name,
            vectors_count: info.vectors_count,
            indexed_vectors_count: info.indexed_vectors_count,
            points_count: info.points_count,
            config: Some(info.config.into()),
            status: info.status.as_str().to_string(),
        }
    }
}

// ===== Vector =====

impl TryFrom<Option<ProtoVector>> for Vector {
    type Error = VectorError;

    fn try_from(proto: Option<ProtoVector>) -> Result<Self, Self::Error> {
        let proto = proto.ok_or_else(|| VectorError::Validation("Missing vector".to_string()))?;
        Self::try_from(proto)
    }
}

impl TryFrom<ProtoVector> for Vector {
    type Error = VectorError;

    fn try_from(proto: ProtoVector) -> Result<Self, Self::Error> {
        let id = bytes_to_uuid(&proto.id)?;
        let payload = proto.payload.and_then(|p| {
            if p.json.is_empty() {
                None
            } else {
                serde_json::from_slice(&p.json).ok()
            }
        });

        Ok(Vector {
            id,
            values: proto.values,
            payload,
            sparse: None,
        })
    }
}

impl From<Vector> for ProtoVector {
    fn from(vector: Vector) -> Self {
        ProtoVector {
            id: vector.id.as_bytes().to_vec(),
            values: vector.values,
            payload: vector.payload.map(|p| ProtoPayload {
                json: serde_json::to_vec(&p).unwrap_or_default(),
            }),
            sparse: None,
        }
    }
}

// ===== Search Result =====

impl From<SearchResult> for ProtoSearchResult {
    fn from(result: SearchResult) -> Self {
        ProtoSearchResult {
            id: result.id.as_bytes().to_vec(),
            score: result.score,
            payload: result.payload.map(|p| ProtoPayload {
                json: serde_json::to_vec(&p).unwrap_or_default(),
            }),
            vector: result.vector.map(|values| ProtoVector {
                id: result.id.as_bytes().to_vec(),
                values,
                payload: None,
                sparse: None,
            }),
        }
    }
}

pub fn search_results_to_response(results: Vec<SearchResult>) -> SearchResponse {
    SearchResponse {
        results: results.into_iter().map(Into::into).collect(),
        search_time_ms: 0,
    }
}

pub fn search_results_to_recommend_response(results: Vec<SearchResult>) -> RecommendResponse {
    RecommendResponse {
        results: results.into_iter().map(Into::into).collect(),
        search_time_ms: 0,
    }
}

// ===== Embedding Provider =====

pub fn embedding_provider_from_proto(proto: i32) -> EmbeddingProviderType {
    match ProtoEmbeddingProvider::try_from(proto) {
        Ok(ProtoEmbeddingProvider::EmbeddingOpenai) => EmbeddingProviderType::OpenAI,
        Ok(ProtoEmbeddingProvider::EmbeddingAnthropic) => EmbeddingProviderType::Anthropic,
        Ok(ProtoEmbeddingProvider::EmbeddingLocal) => EmbeddingProviderType::Local,
        Ok(ProtoEmbeddingProvider::EmbeddingVertexai) => EmbeddingProviderType::VertexAI,
        Ok(ProtoEmbeddingProvider::EmbeddingCohere) => EmbeddingProviderType::Cohere,
        Ok(ProtoEmbeddingProvider::EmbeddingVoyage) => EmbeddingProviderType::Voyage,
        _ => EmbeddingProviderType::OpenAI,
    }
}

pub fn embedding_provider_to_proto(provider: EmbeddingProviderType) -> i32 {
    match provider {
        EmbeddingProviderType::OpenAI => ProtoEmbeddingProvider::EmbeddingOpenai as i32,
        EmbeddingProviderType::Anthropic => ProtoEmbeddingProvider::EmbeddingAnthropic as i32,
        EmbeddingProviderType::Local => ProtoEmbeddingProvider::EmbeddingLocal as i32,
        EmbeddingProviderType::VertexAI => ProtoEmbeddingProvider::EmbeddingVertexai as i32,
        EmbeddingProviderType::Cohere => ProtoEmbeddingProvider::EmbeddingCohere as i32,
        EmbeddingProviderType::Voyage => ProtoEmbeddingProvider::EmbeddingVoyage as i32,
    }
}

// ===== Embedding Model =====

pub fn embedding_model_from_proto(proto: i32, custom_dim: Option<u32>) -> EmbeddingModel {
    match ProtoEmbeddingModel::try_from(proto) {
        // OpenAI models
        Ok(ProtoEmbeddingModel::Embedding3Small) => EmbeddingModel::TextEmbedding3Small,
        Ok(ProtoEmbeddingModel::Embedding3Large) => EmbeddingModel::TextEmbedding3Large,
        Ok(ProtoEmbeddingModel::EmbeddingAda002) => EmbeddingModel::TextEmbeddingAda002,
        // Vertex AI models
        Ok(ProtoEmbeddingModel::Gecko) => EmbeddingModel::Gecko,
        Ok(ProtoEmbeddingModel::GeckoMultilingual) => EmbeddingModel::GeckoMultilingual,
        Ok(ProtoEmbeddingModel::TextEmbedding004) => EmbeddingModel::TextEmbedding004,
        Ok(ProtoEmbeddingModel::TextEmbedding005) => EmbeddingModel::TextEmbedding005,
        Ok(ProtoEmbeddingModel::TextMultilingualEmbedding002) => {
            EmbeddingModel::TextMultilingualEmbedding002
        }
        // Cohere models
        Ok(ProtoEmbeddingModel::CohereEmbedV3) => EmbeddingModel::CohereEmbedV3,
        Ok(ProtoEmbeddingModel::CohereEmbedMultilingualV3) => {
            EmbeddingModel::CohereEmbedMultilingualV3
        }
        // Voyage models
        Ok(ProtoEmbeddingModel::Voyage3) => EmbeddingModel::Voyage3,
        Ok(ProtoEmbeddingModel::Voyage3Lite) => EmbeddingModel::Voyage3Lite,
        Ok(ProtoEmbeddingModel::VoyageCode3) => EmbeddingModel::VoyageCode3,
        // Custom
        Ok(ProtoEmbeddingModel::Custom) => EmbeddingModel::Custom(custom_dim.unwrap_or(768)),
        _ => EmbeddingModel::TextEmbedding3Small,
    }
}

pub fn embedding_model_to_proto(model: EmbeddingModel) -> i32 {
    match model {
        // OpenAI
        EmbeddingModel::TextEmbedding3Small => ProtoEmbeddingModel::Embedding3Small as i32,
        EmbeddingModel::TextEmbedding3Large => ProtoEmbeddingModel::Embedding3Large as i32,
        EmbeddingModel::TextEmbeddingAda002 => ProtoEmbeddingModel::EmbeddingAda002 as i32,
        // Vertex AI
        EmbeddingModel::Gecko => ProtoEmbeddingModel::Gecko as i32,
        EmbeddingModel::GeckoMultilingual => ProtoEmbeddingModel::GeckoMultilingual as i32,
        EmbeddingModel::TextEmbedding004 => ProtoEmbeddingModel::TextEmbedding004 as i32,
        EmbeddingModel::TextEmbedding005 => ProtoEmbeddingModel::TextEmbedding005 as i32,
        EmbeddingModel::TextMultilingualEmbedding002 => {
            ProtoEmbeddingModel::TextMultilingualEmbedding002 as i32
        }
        // Cohere
        EmbeddingModel::CohereEmbedV3 => ProtoEmbeddingModel::CohereEmbedV3 as i32,
        EmbeddingModel::CohereEmbedMultilingualV3 => {
            ProtoEmbeddingModel::CohereEmbedMultilingualV3 as i32
        }
        // Voyage
        EmbeddingModel::Voyage3 => ProtoEmbeddingModel::Voyage3 as i32,
        EmbeddingModel::Voyage3Lite => ProtoEmbeddingModel::Voyage3Lite as i32,
        EmbeddingModel::VoyageCode3 => ProtoEmbeddingModel::VoyageCode3 as i32,
        // Custom
        EmbeddingModel::Custom(_) => ProtoEmbeddingModel::Custom as i32,
    }
}

// ===== Helper Functions =====

pub fn bytes_to_uuid(bytes: &[u8]) -> VectorResult<Uuid> {
    if bytes.len() != 16 {
        return Err(VectorError::Validation(format!(
            "Invalid UUID: expected 16 bytes, got {}",
            bytes.len()
        )));
    }

    let arr: [u8; 16] = bytes
        .try_into()
        .map_err(|_| VectorError::Validation("Invalid UUID bytes".to_string()))?;

    Ok(Uuid::from_bytes(arr))
}

pub fn uuid_to_bytes(id: Uuid) -> Vec<u8> {
    id.as_bytes().to_vec()
}
