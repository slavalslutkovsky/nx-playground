//! Product-related Tauri commands

use serde::{Deserialize, Serialize};

const API_URL: &str = "http://localhost:3003";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub description: String,
    pub price: i64,
    pub display_price: Option<f64>,
    pub stock: i32,
    pub reserved_stock: i32,
    pub category: String,
    pub status: String,
    pub images: Vec<ProductImage>,
    pub sku: Option<String>,
    pub barcode: Option<String>,
    pub brand: Option<String>,
    pub weight: Option<i32>,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductImage {
    pub url: String,
    pub alt: Option<String>,
    pub is_primary: bool,
    pub sort_order: i32,
}

#[derive(Debug, Default, Deserialize)]
pub struct ProductFilter {
    pub status: Option<String>,
    pub category: Option<String>,
    pub brand: Option<String>,
    pub min_price: Option<i64>,
    pub max_price: Option<i64>,
    pub in_stock: Option<bool>,
    pub tag: Option<String>,
    pub search: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i64>,
}

#[tauri::command]
pub async fn fetch_products(filter: ProductFilter) -> Result<Vec<Product>, String> {
    let client = reqwest::Client::new();

    let mut url = format!("{}/api/products", API_URL);
    let mut params = Vec::new();

    if let Some(status) = filter.status {
        params.push(format!("status={}", status));
    }
    if let Some(category) = filter.category {
        params.push(format!("category={}", category));
    }
    if let Some(brand) = filter.brand {
        params.push(format!("brand={}", brand));
    }
    if let Some(min_price) = filter.min_price {
        params.push(format!("min_price={}", min_price));
    }
    if let Some(max_price) = filter.max_price {
        params.push(format!("max_price={}", max_price));
    }
    if let Some(in_stock) = filter.in_stock {
        params.push(format!("in_stock={}", in_stock));
    }
    if let Some(tag) = filter.tag {
        params.push(format!("tag={}", tag));
    }
    if let Some(search) = filter.search {
        params.push(format!("search={}", search));
    }
    if let Some(limit) = filter.limit {
        params.push(format!("limit={}", limit));
    }
    if let Some(offset) = filter.offset {
        params.push(format!("offset={}", offset));
    }

    if !params.is_empty() {
        url = format!("{}?{}", url, params.join("&"));
    }

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()));
    }

    response
        .json::<Vec<Product>>()
        .await
        .map_err(|e| format!("Parse error: {}", e))
}

#[tauri::command]
pub async fn fetch_product(id: String) -> Result<Product, String> {
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/api/products/{}", API_URL, id))
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()));
    }

    response
        .json::<Product>()
        .await
        .map_err(|e| format!("Parse error: {}", e))
}

#[tauri::command]
pub async fn search_products(query: String, limit: i32) -> Result<Vec<Product>, String> {
    let client = reqwest::Client::new();

    let url = format!(
        "{}/api/products/search?q={}&limit={}",
        API_URL, query, limit
    );

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()));
    }

    response
        .json::<Vec<Product>>()
        .await
        .map_err(|e| format!("Parse error: {}", e))
}
