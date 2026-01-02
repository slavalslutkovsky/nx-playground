/**
 * API client for Products API
 */

const API_URL = import.meta.env.PUBLIC_API_URL || 'http://localhost:3003';

export interface Product {
  id: string;
  name: string;
  description: string;
  price: number;
  display_price?: number;
  stock: number;
  reserved_stock: number;
  category: string;
  status: string;
  images: ProductImage[];
  sku?: string;
  barcode?: string;
  brand?: string;
  weight?: number;
  tags: string[];
  created_at: string;
  updated_at: string;
}

export interface ProductImage {
  url: string;
  alt?: string;
  is_primary: boolean;
  sort_order: number;
}

export interface ProductFilter {
  status?: string;
  category?: string;
  brand?: string;
  min_price?: number;
  max_price?: number;
  in_stock?: boolean;
  tag?: string;
  search?: string;
  limit?: number;
  offset?: number;
}

export async function fetchProducts(filter: ProductFilter = {}): Promise<Product[]> {
  const params = new URLSearchParams();

  if (filter.status) params.set('status', filter.status);
  if (filter.category) params.set('category', filter.category);
  if (filter.brand) params.set('brand', filter.brand);
  if (filter.min_price) params.set('min_price', filter.min_price.toString());
  if (filter.max_price) params.set('max_price', filter.max_price.toString());
  if (filter.in_stock !== undefined) params.set('in_stock', filter.in_stock.toString());
  if (filter.tag) params.set('tag', filter.tag);
  if (filter.search) params.set('search', filter.search);
  if (filter.limit) params.set('limit', filter.limit.toString());
  if (filter.offset) params.set('offset', filter.offset.toString());

  const response = await fetch(`${API_URL}/api/products?${params.toString()}`);

  if (!response.ok) {
    throw new Error(`Failed to fetch products: ${response.statusText}`);
  }

  return response.json();
}

export async function fetchProduct(id: string): Promise<Product> {
  const response = await fetch(`${API_URL}/api/products/${id}`);

  if (!response.ok) {
    throw new Error(`Failed to fetch product: ${response.statusText}`);
  }

  return response.json();
}

export async function searchProducts(query: string, limit = 50, offset = 0): Promise<Product[]> {
  const params = new URLSearchParams({
    q: query,
    limit: limit.toString(),
    offset: offset.toString(),
  });

  const response = await fetch(`${API_URL}/api/products/search?${params.toString()}`);

  if (!response.ok) {
    throw new Error(`Failed to search products: ${response.statusText}`);
  }

  return response.json();
}

export async function fetchProductsByCategory(
  category: string,
  limit = 50,
  offset = 0
): Promise<Product[]> {
  const params = new URLSearchParams({
    limit: limit.toString(),
    offset: offset.toString(),
  });

  const response = await fetch(`${API_URL}/api/products/category/${category}?${params.toString()}`);

  if (!response.ok) {
    throw new Error(`Failed to fetch products by category: ${response.statusText}`);
  }

  return response.json();
}

export function formatPrice(cents: number): string {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
  }).format(cents / 100);
}
