// k6 load tests for Zerg MongoDB API
// Endpoints: /health, /ready, /api/items, /api/events

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
const BASE_URL = getBaseUrl('ZERG_MONGO_API_URL', 'http://zerg-mongo-api:3001');

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
  group('Items API', () => {
    // List items
    const listRes = http.get(`${BASE_URL}/api/items`, { headers: jsonHeaders });
    checkResponse(listRes, 200, 'list items');
    checkJsonResponse(listRes, 'list items');

    // Create an item
    const createPayload = JSON.stringify({
      name: `Load Test Item ${randomString(6)}`,
      description: 'Created by k6 load test',
      quantity: Math.floor(Math.random() * 100),
      price: parseFloat((Math.random() * 1000).toFixed(2)),
    });

    const createRes = http.post(`${BASE_URL}/api/items`, createPayload, {
      headers: jsonHeaders,
    });

    if (checkResponse(createRes, 201, 'create item')) {
      const item = createRes.json();

      // Get the created item
      const getRes = http.get(`${BASE_URL}/api/items/${item.id || item._id}`, {
        headers: jsonHeaders,
      });
      checkResponse(getRes, 200, 'get item');

      // Update the item
      const updatePayload = JSON.stringify({
        quantity: Math.floor(Math.random() * 100) + 100,
      });
      const updateRes = http.put(
        `${BASE_URL}/api/items/${item.id || item._id}`,
        updatePayload,
        { headers: jsonHeaders }
      );
      checkResponse(updateRes, 200, 'update item');

      // Delete the item
      const deleteRes = http.del(
        `${BASE_URL}/api/items/${item.id || item._id}`,
        null,
        { headers: jsonHeaders }
      );
      checkResponse(deleteRes, 204, 'delete item');
    }
  });

  group('Events API', () => {
    const listRes = http.get(`${BASE_URL}/api/events`, { headers: jsonHeaders });
    checkResponse(listRes, 200, 'list events');

    // Create an event
    const createPayload = JSON.stringify({
      name: `Load Test Event ${randomString(6)}`,
      event_type: 'test',
      payload: { test: true, timestamp: Date.now() },
    });

    const createRes = http.post(`${BASE_URL}/api/events`, createPayload, {
      headers: jsonHeaders,
    });
    checkResponse(createRes, 201, 'create event');
  });

  sleep(randomSleep(1, 3));
}

export function stressTest() {
  group('High Concurrency Reads', () => {
    const healthRes = http.get(`${BASE_URL}/health`);
    checkResponse(healthRes, 200, 'stress health');

    const itemsRes = http.get(`${BASE_URL}/api/items`, { headers: jsonHeaders });
    checkResponse(itemsRes, 200, 'stress list items');

    const eventsRes = http.get(`${BASE_URL}/api/events`, { headers: jsonHeaders });
    checkResponse(eventsRes, 200, 'stress list events');
  });

  sleep(randomSleep(0.5, 1));
}

export default function () {
  loadTest();
}