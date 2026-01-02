//! gRPC service implementation for Products

use async_trait::async_trait;
use domain_products::{
    CreateProduct, MongoProductRepository, Product, ProductCategory, ProductFilter, ProductService,
    ProductStatus, StockAdjustment, StockReservation, UpdateProduct,
};
use rpc::products::products_service_server::ProductsService;
use rpc::products::*;
use tonic::{Request, Response, Status};
use uuid::Uuid;

/// gRPC implementation of ProductsService
pub struct ProductsGrpcService {
    service: ProductService<MongoProductRepository>,
}

impl ProductsGrpcService {
    pub fn new(service: ProductService<MongoProductRepository>) -> Self {
        Self { service }
    }
}

// Helper functions for conversions
fn uuid_from_bytes(bytes: &[u8]) -> Result<Uuid, Status> {
    if bytes.len() != 16 {
        return Err(Status::invalid_argument("Invalid UUID length"));
    }
    Ok(Uuid::from_slice(bytes).map_err(|e| Status::invalid_argument(e.to_string()))?)
}

fn uuid_to_bytes(uuid: Uuid) -> Vec<u8> {
    uuid.as_bytes().to_vec()
}

fn status_to_proto(status: ProductStatus) -> i32 {
    match status {
        ProductStatus::Active => rpc::products::ProductStatus::Active as i32,
        ProductStatus::Inactive => rpc::products::ProductStatus::Inactive as i32,
        ProductStatus::OutOfStock => rpc::products::ProductStatus::OutOfStock as i32,
        ProductStatus::Discontinued => rpc::products::ProductStatus::Discontinued as i32,
        ProductStatus::Draft => rpc::products::ProductStatus::Draft as i32,
    }
}

fn status_from_proto(status: i32) -> ProductStatus {
    match status {
        1 => ProductStatus::Active,
        2 => ProductStatus::Inactive,
        3 => ProductStatus::OutOfStock,
        4 => ProductStatus::Discontinued,
        5 => ProductStatus::Draft,
        _ => ProductStatus::Active,
    }
}

fn category_to_proto(category: ProductCategory) -> i32 {
    match category {
        ProductCategory::General => rpc::products::ProductCategory::General as i32,
        ProductCategory::Electronics => rpc::products::ProductCategory::Electronics as i32,
        ProductCategory::Clothing => rpc::products::ProductCategory::Clothing as i32,
        ProductCategory::Food => rpc::products::ProductCategory::Food as i32,
        ProductCategory::Books => rpc::products::ProductCategory::Books as i32,
        ProductCategory::HomeGarden => rpc::products::ProductCategory::HomeGarden as i32,
        ProductCategory::Sports => rpc::products::ProductCategory::Sports as i32,
        ProductCategory::Toys => rpc::products::ProductCategory::Toys as i32,
        ProductCategory::Health => rpc::products::ProductCategory::Health as i32,
        ProductCategory::Automotive => rpc::products::ProductCategory::Automotive as i32,
        ProductCategory::Other => rpc::products::ProductCategory::Other as i32,
    }
}

fn category_from_proto(category: i32) -> ProductCategory {
    match category {
        1 => ProductCategory::General,
        2 => ProductCategory::Electronics,
        3 => ProductCategory::Clothing,
        4 => ProductCategory::Food,
        5 => ProductCategory::Books,
        6 => ProductCategory::HomeGarden,
        7 => ProductCategory::Sports,
        8 => ProductCategory::Toys,
        9 => ProductCategory::Health,
        10 => ProductCategory::Automotive,
        11 => ProductCategory::Other,
        _ => ProductCategory::General,
    }
}

fn product_to_proto(product: Product) -> rpc::products::Product {
    rpc::products::Product {
        id: uuid_to_bytes(product.id),
        name: product.name,
        description: product.description,
        price: product.price,
        display_price: product.display_price.unwrap_or(0.0),
        stock: product.stock,
        reserved_stock: product.reserved_stock,
        category: category_to_proto(product.category),
        status: status_to_proto(product.status),
        images: product
            .images
            .into_iter()
            .map(|img| ProductImage {
                url: img.url,
                alt: img.alt,
                is_primary: img.is_primary,
                sort_order: img.sort_order,
            })
            .collect(),
        sku: product.sku,
        barcode: product.barcode,
        brand: product.brand,
        weight: product.weight,
        tags: product.tags,
        metadata: Some(product.metadata.to_string()),
        created_at: product.created_at.timestamp(),
        updated_at: product.updated_at.timestamp(),
    }
}

#[async_trait]
impl ProductsService for ProductsGrpcService {
    async fn create(
        &self,
        request: Request<CreateRequest>,
    ) -> Result<Response<CreateResponse>, Status> {
        let req = request.into_inner();

        let input = CreateProduct {
            name: req.name,
            description: req.description,
            price: req.price,
            stock: req.stock,
            category: category_from_proto(req.category),
            status: status_from_proto(req.status),
            images: req
                .images
                .into_iter()
                .map(|img| domain_products::ProductImage {
                    url: img.url,
                    alt: img.alt,
                    is_primary: img.is_primary,
                    sort_order: img.sort_order,
                })
                .collect(),
            sku: req.sku,
            barcode: req.barcode,
            brand: req.brand,
            weight: req.weight,
            tags: req.tags,
            metadata: req
                .metadata
                .map(|m| serde_json::from_str(&m).unwrap_or_default())
                .unwrap_or_default(),
        };

        let product = self
            .service
            .create_product(input)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CreateResponse {
            product: Some(product_to_proto(product)),
        }))
    }

    async fn get_by_id(
        &self,
        request: Request<GetByIdRequest>,
    ) -> Result<Response<GetByIdResponse>, Status> {
        let id = uuid_from_bytes(&request.into_inner().id)?;

        let product = self
            .service
            .get_product(id)
            .await
            .map_err(|e| Status::not_found(e.to_string()))?;

        Ok(Response::new(GetByIdResponse {
            product: Some(product_to_proto(product)),
        }))
    }

    async fn get_by_sku(
        &self,
        request: Request<GetBySkuRequest>,
    ) -> Result<Response<GetBySkuResponse>, Status> {
        let product = self
            .service
            .get_by_sku(&request.into_inner().sku)
            .await
            .map_err(|e| Status::not_found(e.to_string()))?;

        Ok(Response::new(GetBySkuResponse {
            product: Some(product_to_proto(product)),
        }))
    }

    async fn get_by_barcode(
        &self,
        request: Request<GetByBarcodeRequest>,
    ) -> Result<Response<GetByBarcodeResponse>, Status> {
        let product = self
            .service
            .get_by_barcode(&request.into_inner().barcode)
            .await
            .map_err(|e| Status::not_found(e.to_string()))?;

        Ok(Response::new(GetByBarcodeResponse {
            product: Some(product_to_proto(product)),
        }))
    }

    async fn update_by_id(
        &self,
        request: Request<UpdateByIdRequest>,
    ) -> Result<Response<UpdateByIdResponse>, Status> {
        let req = request.into_inner();
        let id = uuid_from_bytes(&req.id)?;

        let input = UpdateProduct {
            name: req.name,
            description: req.description,
            price: req.price,
            stock: req.stock,
            category: req.category.map(category_from_proto),
            status: req.status.map(status_from_proto),
            images: if req.images.is_empty() {
                None
            } else {
                Some(
                    req.images
                        .into_iter()
                        .map(|img| domain_products::ProductImage {
                            url: img.url,
                            alt: img.alt,
                            is_primary: img.is_primary,
                            sort_order: img.sort_order,
                        })
                        .collect(),
                )
            },
            sku: req.sku,
            barcode: req.barcode,
            brand: req.brand,
            weight: req.weight,
            tags: if req.tags.is_empty() {
                None
            } else {
                Some(req.tags)
            },
            metadata: req
                .metadata
                .map(|m| serde_json::from_str(&m).unwrap_or_default()),
        };

        let product = self
            .service
            .update_product(id, input)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(UpdateByIdResponse {
            product: Some(product_to_proto(product)),
        }))
    }

    async fn delete_by_id(
        &self,
        request: Request<DeleteByIdRequest>,
    ) -> Result<Response<DeleteByIdResponse>, Status> {
        let id = uuid_from_bytes(&request.into_inner().id)?;

        self.service
            .delete_product(id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(DeleteByIdResponse {}))
    }

    async fn list(&self, request: Request<ListRequest>) -> Result<Response<ListResponse>, Status> {
        let req = request.into_inner();

        let filter = ProductFilter {
            status: req.status.map(status_from_proto),
            category: req.category.map(category_from_proto),
            brand: req.brand,
            min_price: req.min_price,
            max_price: req.max_price,
            in_stock: req.in_stock,
            tag: req.tag,
            search: req.search,
            limit: req.limit as i64,
            offset: req.offset as u64,
        };

        let products = self
            .service
            .list_products(filter.clone())
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let total = self.service.count_products(filter).await.unwrap_or(0);

        Ok(Response::new(ListResponse {
            data: products.into_iter().map(product_to_proto).collect(),
            total: total as i64,
        }))
    }

    type ListStreamStream = tokio_stream::wrappers::ReceiverStream<Result<ListStreamResponse, Status>>;

    async fn list_stream(
        &self,
        request: Request<ListStreamRequest>,
    ) -> Result<Response<Self::ListStreamStream>, Status> {
        let req = request.into_inner();

        let filter = ProductFilter {
            status: req.status.map(status_from_proto),
            category: req.category.map(category_from_proto),
            in_stock: req.in_stock,
            limit: req.limit as i64,
            ..Default::default()
        };

        let products = self
            .service
            .list_products(filter)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let (tx, rx) = tokio::sync::mpsc::channel(100);

        tokio::spawn(async move {
            for product in products {
                let _ = tx.send(Ok(ListStreamResponse {
                    product: Some(product_to_proto(product)),
                })).await;
            }
        });

        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(
            rx,
        )))
    }

    async fn search(
        &self,
        request: Request<SearchRequest>,
    ) -> Result<Response<SearchResponse>, Status> {
        let req = request.into_inner();

        let products = self
            .service
            .search_products(&req.query, req.limit as i64, req.offset as u64)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(SearchResponse {
            data: products.into_iter().map(product_to_proto).collect(),
            total: 0, // Search doesn't return total
        }))
    }

    async fn count(
        &self,
        request: Request<CountRequest>,
    ) -> Result<Response<CountResponse>, Status> {
        let req = request.into_inner();

        let filter = ProductFilter {
            status: req.status.map(status_from_proto),
            category: req.category.map(category_from_proto),
            in_stock: req.in_stock,
            ..Default::default()
        };

        let count = self
            .service
            .count_products(filter)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CountResponse {
            count: count as i64,
        }))
    }

    async fn get_by_category(
        &self,
        request: Request<GetByCategoryRequest>,
    ) -> Result<Response<GetByCategoryResponse>, Status> {
        let req = request.into_inner();
        let category = category_from_proto(req.category);

        let products = self
            .service
            .get_by_category(category, req.limit as i64, req.offset as u64)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(GetByCategoryResponse {
            data: products.into_iter().map(product_to_proto).collect(),
            total: 0,
        }))
    }

    async fn update_stock(
        &self,
        request: Request<UpdateStockRequest>,
    ) -> Result<Response<UpdateStockResponse>, Status> {
        let req = request.into_inner();
        let id = uuid_from_bytes(&req.id)?;

        let adjustment = StockAdjustment {
            quantity: req.quantity_change,
            reason: req.reason,
        };

        let product = self
            .service
            .adjust_stock(id, adjustment)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(UpdateStockResponse {
            product: Some(product_to_proto(product)),
        }))
    }

    async fn reserve_stock(
        &self,
        request: Request<ReserveStockRequest>,
    ) -> Result<Response<ReservationResponse>, Status> {
        let req = request.into_inner();
        let id = uuid_from_bytes(&req.id)?;

        let reservation = StockReservation {
            quantity: req.quantity,
            order_id: req.order_id,
            ttl_seconds: req.ttl_seconds.unwrap_or(900) as u32,
        };

        let result = self
            .service
            .reserve_stock(id, reservation)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ReservationResponse {
            reservation_id: result.reservation_id,
            success: result.success,
            message: result.message,
        }))
    }

    async fn release_stock(
        &self,
        request: Request<ReleaseStockRequest>,
    ) -> Result<Response<ReleaseStockResponse>, Status> {
        let req = request.into_inner();
        let id = uuid_from_bytes(&req.id)?;

        let product = self
            .service
            .release_stock(id, req.quantity)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ReleaseStockResponse {
            product: Some(product_to_proto(product)),
        }))
    }

    async fn commit_stock(
        &self,
        request: Request<CommitStockRequest>,
    ) -> Result<Response<CommitStockResponse>, Status> {
        let req = request.into_inner();
        let id = uuid_from_bytes(&req.id)?;

        let product = self
            .service
            .commit_stock(id, req.quantity)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CommitStockResponse {
            product: Some(product_to_proto(product)),
        }))
    }

    async fn get_low_stock(
        &self,
        request: Request<GetLowStockRequest>,
    ) -> Result<Response<GetLowStockResponse>, Status> {
        let req = request.into_inner();

        let products = self
            .service
            .get_low_stock(req.threshold, req.limit as i64)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(GetLowStockResponse {
            data: products.into_iter().map(product_to_proto).collect(),
        }))
    }

    async fn activate(
        &self,
        request: Request<ActivateRequest>,
    ) -> Result<Response<ActivateResponse>, Status> {
        let id = uuid_from_bytes(&request.into_inner().id)?;

        let product = self
            .service
            .activate_product(id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ActivateResponse {
            product: Some(product_to_proto(product)),
        }))
    }

    async fn deactivate(
        &self,
        request: Request<DeactivateRequest>,
    ) -> Result<Response<DeactivateResponse>, Status> {
        let id = uuid_from_bytes(&request.into_inner().id)?;

        let product = self
            .service
            .deactivate_product(id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(DeactivateResponse {
            product: Some(product_to_proto(product)),
        }))
    }

    async fn discontinue(
        &self,
        request: Request<DiscontinueRequest>,
    ) -> Result<Response<DiscontinueResponse>, Status> {
        let id = uuid_from_bytes(&request.into_inner().id)?;

        let product = self
            .service
            .discontinue_product(id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(DiscontinueResponse {
            product: Some(product_to_proto(product)),
        }))
    }
}
