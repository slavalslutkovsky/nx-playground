/**
 * API client for Products API (Tauri version)
 */
import { invoke } from '@tauri-apps/api/core';

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

// Use Tauri commands for API calls
export async function fetchProducts(filter: ProductFilter = {}): Promise<Product[]> {
  try {
    return await invoke('fetch_products', { filter });
  } catch (error) {
    console.error('Failed to fetch products:', error);
    return [];
  }
}

export async function fetchProduct(id: string): Promise<Product | null> {
  try {
    return await invoke('fetch_product', { id });
  } catch (error) {
    console.error('Failed to fetch product:', error);
    return null;
  }
}

export async function searchProducts(query: string, limit = 50): Promise<Product[]> {
  try {
    return await invoke('search_products', { query, limit });
  } catch (error) {
    console.error('Failed to search products:', error);
    return [];
  }
}

export function formatPrice(cents: number): string {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
  }).format(cents / 100);
}
