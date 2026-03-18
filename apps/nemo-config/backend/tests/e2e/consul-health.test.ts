import { describe, it, expect, beforeAll, afterAll } from 'bun:test';
import { cleanupTestResources } from './helpers/consul';
import { get } from './helpers/api';
import { CONFIG } from './config';

describe('Consul Health API - E2E Tests', () => {
  beforeAll(async () => {
    await cleanupTestResources();
  });

  afterAll(async () => {
    await cleanupTestResources();
  });

  it('should return healthy status for Consul connection', async () => {
    const { data, status } = await get(`/api/health/consul?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}`);
    expect(status).toBe(200);
    
    expect(data).toHaveProperty('status');
    expect(data).toHaveProperty('connected');
    expect(data).toHaveProperty('url');
    expect(data).toHaveProperty('timestamp');
    
    // With a real Consul server, this should be healthy
    expect(data.status).toBe('healthy');
    expect(data.connected).toBe(true);
    expect(data.url).toBe(CONFIG.CONSUL_URL);
    
    // Timestamp should be recent (within last 5 seconds)
    const timestamp = new Date(data.timestamp);
    const now = new Date();
    expect(now.getTime() - timestamp.getTime()).toBeLessThan(5000);
  });

  it('should return unhealthy status for invalid Consul URL', async () => {
    const { data, status } = await get('/api/health/consul?consul_url=invalid-host:8500');
    expect(status).toBe(200); // API returns 200 even for unhealthy Consul
    
    expect(data.status).toBe('unhealthy');
    expect(data.connected).toBe(false);
    expect(data.error).toBeDefined();
    expect(data.url).toBe('invalid-host:8500');
  });
});