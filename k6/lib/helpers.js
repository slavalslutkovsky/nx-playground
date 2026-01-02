// Shared k6 helper functions and utilities

import { check, fail } from 'k6';
import { Rate, Trend } from 'k6/metrics';

// Custom metrics
export const errorRate = new Rate('errors');
export const requestDuration = new Trend('request_duration');

/**
 * Standard check for HTTP response
 * @param {Response} response - k6 HTTP response
 * @param {number} expectedStatus - Expected HTTP status code
 * @param {string} name - Check name for reporting
 */
export function checkResponse(response, expectedStatus, name) {
  const passed = check(response, {
    [`${name} - status is ${expectedStatus}`]: (r) => r.status === expectedStatus,
    [`${name} - response time < 500ms`]: (r) => r.timings.duration < 500,
  });

  errorRate.add(!passed);
  requestDuration.add(response.timings.duration);

  return passed;
}

/**
 * Check JSON response body
 * @param {Response} response - k6 HTTP response
 * @param {string} name - Check name
 */
export function checkJsonResponse(response, name) {
  return check(response, {
    [`${name} - has valid JSON`]: (r) => {
      try {
        r.json();
        return true;
      } catch (e) {
        return false;
      }
    },
  });
}

/**
 * Generate random string
 * @param {number} length - String length
 */
export function randomString(length = 10) {
  const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
  let result = '';
  for (let i = 0; i < length; i++) {
    result += chars.charAt(Math.floor(Math.random() * chars.length));
  }
  return result;
}

/**
 * Generate random UUID v4
 */
export function randomUUID() {
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function (c) {
    const r = (Math.random() * 16) | 0;
    const v = c === 'x' ? r : (r & 0x3) | 0x8;
    return v.toString(16);
  });
}

/**
 * Common HTTP headers
 */
export const jsonHeaders = {
  'Content-Type': 'application/json',
  Accept: 'application/json',
};

/**
 * Get base URL from environment
 * @param {string} envVar - Environment variable name
 * @param {string} defaultUrl - Default URL
 */
export function getBaseUrl(envVar, defaultUrl) {
  return __ENV[envVar] || defaultUrl;
}

/**
 * Sleep with random jitter
 * @param {number} min - Minimum seconds
 * @param {number} max - Maximum seconds
 */
export function randomSleep(min, max) {
  const duration = min + Math.random() * (max - min);
  return duration;
}

/**
 * Standard load test stages
 */
export const standardStages = [
  { duration: '30s', target: 10 }, // Ramp up
  { duration: '1m', target: 10 }, // Steady state
  { duration: '30s', target: 50 }, // Spike
  { duration: '1m', target: 50 }, // Sustained load
  { duration: '30s', target: 0 }, // Ramp down
];

/**
 * Smoke test stages (quick validation)
 */
export const smokeStages = [
  { duration: '10s', target: 1 },
  { duration: '20s', target: 1 },
  { duration: '10s', target: 0 },
];

/**
 * Stress test stages
 */
export const stressStages = [
  { duration: '1m', target: 20 },
  { duration: '2m', target: 50 },
  { duration: '2m', target: 100 },
  { duration: '2m', target: 100 },
  { duration: '1m', target: 0 },
];

/**
 * Standard thresholds
 */
export const standardThresholds = {
  http_req_duration: ['p(95)<500', 'p(99)<1000'],
  http_req_failed: ['rate<0.01'],
  errors: ['rate<0.05'],
};