import { CONFIG } from './config.ts';

const endpoint = '/api/catalog';
const url = `${CONFIG.API_URL}${endpoint}`;
console.log('CONFIG.API_URL:', CONFIG.API_URL);
console.log('endpoint:', endpoint);
console.log('constructed url:', url);

// Test what the actual endpoint should be
const expected = 'http://localhost:3001/api/api/catalog';
console.log('Expected URL (if API has /api prefix):', expected);
const expected2 = 'http://localhost:3001/api/catalog';
console.log('Expected URL (if API does NOT have /api prefix):', expected2);