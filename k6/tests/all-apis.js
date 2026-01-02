// Combined k6 load tests for all APIs
// Runs health checks and basic CRUD operations across all services

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

// Configuration - all service URLs
const ZERG_API_URL = getBaseUrl('ZERG_API_URL', 'http://zerg-api:3000');
const ZERG_MONGO_API_URL = getBaseUrl('ZERG_MONGO_API_URL', 'http://zerg-mongo-api:3001');
const PRODUCTS_API_URL = getBaseUrl('PRODUCTS_API_URL', 'http://products-api:3003');

export const options = {
  scenarios: {
    // Quick smoke test across all services
    smoke_all: {
      executor: 'constant-vus',
      vus: 1,
      duration: '1m',
      exec: 'smokeTestAll',
      tags: { test_type: 'smoke' },
    },
    // Concurrent load across all services
    load_all: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '1m', target: 10 },
        { duration: '2m', target: 30 },
        { duration: '1m', target: 0 },
      ],
      exec: 'loadTestAll',
      startTime: '1m30s',
      tags: { test_type: 'load' },
    },
  },
  thresholds: {
    ...standardThresholds,
    'http_req_duration{service:zerg-api}': ['p(95)<500'],
    'http_req_duration{service:zerg-mongo-api}': ['p(95)<500'],
    'http_req_duration{service:products-api}': ['p(95)<500'],
  },
};

export function smokeTestAll() {
  // Zerg API health
  group('Zerg API', () => {
    const healthRes = http.get(`${ZERG_API_URL}/health`, {
      tags: { service: 'zerg-api' },
    });
    checkResponse(healthRes, 200, 'zerg-api health');

    const readyRes = http.get(`${ZERG_API_URL}/ready`, {
      tags: { service: 'zerg-api' },
    });
    checkResponse(readyRes, 200, 'zerg-api ready');

    const tasksRes = http.get(`${ZERG_API_URL}/api/tasks`, {
      headers: jsonHeaders,
      tags: { service: 'zerg-api' },
    });
    checkResponse(tasksRes, 200, 'zerg-api tasks');
  });

  // Zerg MongoDB API health
  group('Zerg MongoDB API', () => {
    const healthRes = http.get(`${ZERG_MONGO_API_URL}/health`, {
      tags: { service: 'zerg-mongo-api' },
    });
    checkResponse(healthRes, 200, 'zerg-mongo-api health');

    const readyRes = http.get(`${ZERG_MONGO_API_URL}/ready`, {
      tags: { service: 'zerg-mongo-api' },
    });
    checkResponse(readyRes, 200, 'zerg-mongo-api ready');

    const itemsRes = http.get(`${ZERG_MONGO_API_URL}/api/items`, {
      headers: jsonHeaders,
      tags: { service: 'zerg-mongo-api' },
    });
    checkResponse(itemsRes, 200, 'zerg-mongo-api items');
  });

  // Products API health
  group('Products API', () => {
    const healthRes = http.get(`${PRODUCTS_API_URL}/health`, {
      tags: { service: 'products-api' },
    });
    checkResponse(healthRes, 200, 'products-api health');

    const readyRes = http.get(`${PRODUCTS_API_URL}/ready`, {
      tags: { service: 'products-api' },
    });
    checkResponse(readyRes, 200, 'products-api ready');

    const productsRes = http.get(`${PRODUCTS_API_URL}/api/products`, {
      headers: jsonHeaders,
      tags: { service: 'products-api' },
    });
    checkResponse(productsRes, 200, 'products-api products');
  });

  sleep(2);
}

export function loadTestAll() {
  // Randomly choose which service to hit (simulates real traffic distribution)
  const choice = Math.random();

  if (choice < 0.4) {
    // 40% - Zerg API
    group('Zerg API Load', () => {
      const listRes = http.get(`${ZERG_API_URL}/api/tasks`, {
        headers: jsonHeaders,
        tags: { service: 'zerg-api' },
      });
      checkResponse(listRes, 200, 'list tasks');

      // Create task
      const createPayload = JSON.stringify({
        title: `K6 Task ${randomString(6)}`,
        description: 'Load test task',
        status: 'Todo',
        priority: 'Medium',
      });
      const createRes = http.post(`${ZERG_API_URL}/api/tasks`, createPayload, {
        headers: jsonHeaders,
        tags: { service: 'zerg-api' },
      });
      if (checkResponse(createRes, 201, 'create task')) {
        const task = createRes.json();
        http.del(`${ZERG_API_URL}/api/tasks/${task.id}`, null, {
          tags: { service: 'zerg-api' },
        });
      }
    });
  } else if (choice < 0.7) {
    // 30% - Zerg MongoDB API
    group('Zerg MongoDB API Load', () => {
      const listRes = http.get(`${ZERG_MONGO_API_URL}/api/items`, {
        headers: jsonHeaders,
        tags: { service: 'zerg-mongo-api' },
      });
      checkResponse(listRes, 200, 'list items');

      // Create item
      const createPayload = JSON.stringify({
        name: `K6 Item ${randomString(6)}`,
        description: 'Load test item',
        quantity: Math.floor(Math.random() * 100),
        price: parseFloat((Math.random() * 100).toFixed(2)),
      });
      const createRes = http.post(`${ZERG_MONGO_API_URL}/api/items`, createPayload, {
        headers: jsonHeaders,
        tags: { service: 'zerg-mongo-api' },
      });
      if (checkResponse(createRes, 201, 'create item')) {
        const item = createRes.json();
        http.del(`${ZERG_MONGO_API_URL}/api/items/${item.id || item._id}`, null, {
          tags: { service: 'zerg-mongo-api' },
        });
      }
    });
  } else {
    // 30% - Products API
    group('Products API Load', () => {
      const listRes = http.get(`${PRODUCTS_API_URL}/api/products`, {
        headers: jsonHeaders,
        tags: { service: 'products-api' },
      });
      checkResponse(listRes, 200, 'list products');

      const searchRes = http.get(`${PRODUCTS_API_URL}/api/products/search?q=test`, {
        headers: jsonHeaders,
        tags: { service: 'products-api' },
      });
      checkResponse(searchRes, 200, 'search products');

      const countRes = http.get(`${PRODUCTS_API_URL}/api/products/count`, {
        headers: jsonHeaders,
        tags: { service: 'products-api' },
      });
      checkResponse(countRes, 200, 'count products');
    });
  }

  sleep(randomSleep(0.5, 2));
}

export default function () {
  loadTestAll();
}