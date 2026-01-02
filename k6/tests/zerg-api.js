// k6 load tests for Zerg API
// Endpoints: /health, /ready, /api/tasks, /api/projects, /api/users

import http from 'k6/http';
import { sleep, group } from 'k6';
import {
  checkResponse,
  checkJsonResponse,
  jsonHeaders,
  getBaseUrl,
  randomString,
  randomUUID,
  standardThresholds,
  randomSleep,
} from '../lib/helpers.js';

// Configuration
const BASE_URL = getBaseUrl('ZERG_API_URL', 'http://zerg-api:3000');

export const options = {
  scenarios: {
    // Smoke test - quick health check
    smoke: {
      executor: 'constant-vus',
      vus: 1,
      duration: '30s',
      exec: 'smokeTest',
      tags: { test_type: 'smoke' },
    },
    // Load test - normal traffic
    load: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '30s', target: 10 },
        { duration: '1m', target: 20 },
        { duration: '30s', target: 0 },
      ],
      exec: 'loadTest',
      startTime: '35s', // Start after smoke test
      tags: { test_type: 'load' },
    },
    // Stress test - high traffic
    stress: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '30s', target: 50 },
        { duration: '1m', target: 100 },
        { duration: '30s', target: 0 },
      ],
      exec: 'stressTest',
      startTime: '3m', // Start after load test
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

// Smoke test - verify endpoints are responding
export function smokeTest() {
  group('Health Checks', () => {
    const healthRes = http.get(`${BASE_URL}/health`);
    checkResponse(healthRes, 200, 'health');

    const readyRes = http.get(`${BASE_URL}/ready`);
    checkResponse(readyRes, 200, 'ready');
  });

  sleep(1);
}

// Load test - simulate normal user behavior
export function loadTest() {
  group('Tasks API', () => {
    // List tasks
    const listRes = http.get(`${BASE_URL}/api/tasks`, { headers: jsonHeaders });
    checkResponse(listRes, 200, 'list tasks');
    checkJsonResponse(listRes, 'list tasks');

    // Create a task
    const createPayload = JSON.stringify({
      title: `Load Test Task ${randomString(6)}`,
      description: 'Created by k6 load test',
      status: 'todo',
      priority: 'medium',
    });

    const createRes = http.post(`${BASE_URL}/api/tasks`, createPayload, {
      headers: jsonHeaders,
    });

    if (checkResponse(createRes, 201, 'create task')) {
      const task = createRes.json();

      // Get the created task
      const getRes = http.get(`${BASE_URL}/api/tasks/${task.id}`, {
        headers: jsonHeaders,
      });
      checkResponse(getRes, 200, 'get task');

      // Update the task
      const updatePayload = JSON.stringify({
        status: 'in_progress',
      });
      const updateRes = http.put(`${BASE_URL}/api/tasks/${task.id}`, updatePayload, {
        headers: jsonHeaders,
      });
      checkResponse(updateRes, 200, 'update task');

      // Delete the task
      const deleteRes = http.del(`${BASE_URL}/api/tasks/${task.id}`, null, {
        headers: jsonHeaders,
      });
      checkResponse(deleteRes, 204, 'delete task');
    }
  });

  group('Projects API', () => {
    const listRes = http.get(`${BASE_URL}/api/projects`, { headers: jsonHeaders });
    checkResponse(listRes, 200, 'list projects');
  });

  group('Users API', () => {
    const listRes = http.get(`${BASE_URL}/api/users`, { headers: jsonHeaders });
    checkResponse(listRes, 200, 'list users');
  });

  sleep(randomSleep(1, 3));
}

// Stress test - high concurrency read operations
export function stressTest() {
  group('High Concurrency Reads', () => {
    // Focus on read operations under stress
    const healthRes = http.get(`${BASE_URL}/health`);
    checkResponse(healthRes, 200, 'stress health');

    const tasksRes = http.get(`${BASE_URL}/api/tasks`, { headers: jsonHeaders });
    checkResponse(tasksRes, 200, 'stress list tasks');

    const projectsRes = http.get(`${BASE_URL}/api/projects`, { headers: jsonHeaders });
    checkResponse(projectsRes, 200, 'stress list projects');
  });

  sleep(randomSleep(0.5, 1));
}

// Default function (if run without scenarios)
export default function () {
  loadTest();
}