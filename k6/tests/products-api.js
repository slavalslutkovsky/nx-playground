// k6 load tests for Products API
// Endpoints: /health, /ready, /api/products

import http from 'k6/http';
import { sleep, group } from 'k6';
import {
  checkResponse,
  checkJsonResponse,
  jsonHeaders,
  getBaseUrl,
  randomString,
  standardThresholds,
  randomSleep,
} from '../lib/helpers.js';

// Configuration
const BASE_URL = getBaseUrl('PRODUCTS_API_URL', 'http://products-api:3003');

export const options = {
  scenarios: {
    smoke: {
      executor: 'constant-vus',
      vus: 1,
      duration: '30s',
      exec: 'smokeTest',
      tags: { test_type: 'smoke' },
    },
    load: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '30s', target: 10 },
        { duration: '1m', target: 20 },
        { duration: '30s', target: 0 },
      ],
      exec: 'loadTest',
      startTime: '35s',
      tags: { test_type: 'load' },
    },
    stress: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '30s', target: 50 },
        { duration: '1m', target: 100 },
        { duration: '30s', target: 0 },
      ],
      exec: 'stressTest',
      startTime: '3m',
      tags: { test_type: 'stress' },
    },
  },
  thresholds: {
    ...standardThresholds,
    'http_req_duration{test_type:smoke}': ['p(95)<200'],
    'http_req_duration{test_type:load}': ['p(95)<500'],
    'http_req_duration{test_type:stress}': ['p(95)<1000'],
  },
};

// Generate a random SKU
function randomSKU() {
  return `SKU-${randomString(8).toUpperCase()}`;
}

// Generate a random barcode
function randomBarcode() {
  let barcode = '';
  for (let i = 0; i < 12; i++) {
    barcode += Math.floor(Math.random() * 10);
  }
  return barcode;
}

export function smokeTest() {
  group('Health Checks', () => {
    const healthRes = http.get(`${BASE_URL}/health`);
    checkResponse(healthRes, 200, 'health');

    const readyRes = http.get(`${BASE_URL}/ready`);
    checkResponse(readyRes, 200, 'ready');
  });

  sleep(1);
}

export function loadTest() {
  group('Products CRUD', () => {
    // List products
    const listRes = http.get(`${BASE_URL}/api/products`, { headers: jsonHeaders });
    checkResponse(listRes, 200, 'list products');
    checkJsonResponse(listRes, 'list products');

    // Create a product
    const sku = randomSKU();
    const createPayload = JSON.stringify({
      name: `Load Test Product ${randomString(6)}`,
      description: 'Created by k6 load test',
      sku: sku,
      barcode: randomBarcode(),
      category: 'test-category',
      price: parseFloat((Math.random() * 1000 + 10).toFixed(2)),
      cost: parseFloat((Math.random() * 500 + 5).toFixed(2)),
      quantity: Math.floor(Math.random() * 1000),
      min_quantity: 10,
      max_quantity: 10000,
      unit: 'pcs',
    });

    const createRes = http.post(`${BASE_URL}/api/products`, createPayload, {
      headers: jsonHeaders,
    });

    if (checkResponse(createRes, 201, 'create product')) {
      const product = createRes.json();
      const productId = product.id || product._id;

      // Get the created product
      const getRes = http.get(`${BASE_URL}/api/products/${productId}`, {
        headers: jsonHeaders,
      });
      checkResponse(getRes, 200, 'get product');

      // Get product by SKU
      const skuRes = http.get(`${BASE_URL}/api/products/sku/${sku}`, {
        headers: jsonHeaders,
      });
      checkResponse(skuRes, 200, 'get product by SKU');

      // Update the product
      const updatePayload = JSON.stringify({
        price: parseFloat((Math.random() * 1000 + 100).toFixed(2)),
      });
      const updateRes = http.put(`${BASE_URL}/api/products/${productId}`, updatePayload, {
        headers: jsonHeaders,
      });
      checkResponse(updateRes, 200, 'update product');

      // Adjust stock
      const stockPayload = JSON.stringify({
        adjustment: 50,
        reason: 'k6 load test stock adjustment',
      });
      const stockRes = http.post(
        `${BASE_URL}/api/products/${productId}/stock`,
        stockPayload,
        { headers: jsonHeaders }
      );
      checkResponse(stockRes, 200, 'adjust stock');

      // Delete the product
      const deleteRes = http.del(`${BASE_URL}/api/products/${productId}`, null, {
        headers: jsonHeaders,
      });
      checkResponse(deleteRes, 204, 'delete product');
    }
  });

  group('Products Search & Filter', () => {
    // Search products
    const searchRes = http.get(`${BASE_URL}/api/products/search?q=test`, {
      headers: jsonHeaders,
    });
    checkResponse(searchRes, 200, 'search products');

    // Get products by category
    const categoryRes = http.get(`${BASE_URL}/api/products/category/test-category`, {
      headers: jsonHeaders,
    });
    checkResponse(categoryRes, 200, 'products by category');

    // Get low stock products
    const lowStockRes = http.get(`${BASE_URL}/api/products/low-stock?threshold=5`, {
      headers: jsonHeaders,
    });
    checkResponse(lowStockRes, 200, 'low stock products');

    // Count products
    const countRes = http.get(`${BASE_URL}/api/products/count`, {
      headers: jsonHeaders,
    });
    checkResponse(countRes, 200, 'count products');
  });

  sleep(randomSleep(1, 3));
}

export function stressTest() {
  group('High Concurrency Reads', () => {
    const healthRes = http.get(`${BASE_URL}/health`);
    checkResponse(healthRes, 200, 'stress health');

    const productsRes = http.get(`${BASE_URL}/api/products`, { headers: jsonHeaders });
    checkResponse(productsRes, 200, 'stress list products');

    const searchRes = http.get(`${BASE_URL}/api/products/search?q=product`, {
      headers: jsonHeaders,
    });
    checkResponse(searchRes, 200, 'stress search');

    const countRes = http.get(`${BASE_URL}/api/products/count`, {
      headers: jsonHeaders,
    });
    checkResponse(countRes, 200, 'stress count');
  });

  sleep(randomSleep(0.5, 1));
}

export default function () {
  loadTest();
}