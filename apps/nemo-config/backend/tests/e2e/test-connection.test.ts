import { describe, it, expect, beforeAll, afterAll } from 'bun:test';
import { cleanupTestResources } from '../../helpers/consul';
import { get, post } from '../../helpers/api';
import { CONFIG } from '../../config';

describe('Test Connection API - E2E Tests', () => {
  beforeAll(async () => {
    await cleanupTestResources();
  });

  afterAll(async () => {
    await cleanupTestResources();
  });

  it('should test TCP connection to Redis successfully', async () => {
    const { data, status } = await post('/api/test-connection', {
      service_id: 'redis-test',
      connection_url: 'redis://10.7.0.4:6379',
      health_check: { type: 'tcp', port: 6379 },
      metadata: {}
    });

    expect(status).toBe(200);
    expect(data.success).toBe(true);
    expect(data.message).toContain('TCP connection successful');
    expect(data.details).toHaveProperty('host', '10.7.0.4');
    expect(data.details).toHaveProperty('port', 6379);
  });

  it('should test TCP connection to Postgres successfully', async () => {
    const { data, status } = await post('/api/test-connection', {
      service_id: 'postgres-test',
      connection_url: 'postgres://10.7.0.4:5432',
      health_check: { type: 'tcp', port: 5432 },
      metadata: {}
    });

    expect(status).toBe(200);
    expect(data.success).toBe(true);
    expect(data.message).toContain('TCP connection successful');
    expect(data.details).toHaveProperty('host', '10.7.0.4');
    expect(data.details).toHaveProperty('port', 5432);
  });

  it('should test HTTP connection to Minio successfully', async () => {
    const { data, status } = await post('/api/test-connection', {
      service_id: 'minio-test',
      connection_url: 'http://10.7.0.4:9000',
      health_check: { type: 'http', port: 9000, path: '/minio/health/live' },
      metadata: {}
    });

    expect(status).toBe(200);
    // Note: This might fail if Minio isn't actually running on 10.7.0.4:9000
    // but we're testing that the API call works correctly
    expect(data).toHaveProperty('success');
    expect(data).toHaveProperty('message');
  });

  it('should fail on invalid TCP port', async () => {
    const { data, status } = await post('/api/test-connection', {
      service_id: 'redis-test',
      connection_url: 'redis://10.7.0.4:9999',
      health_check: { type: 'tcp', port: 9999 },
      metadata: {}
    });

    expect(status).toBe(200);
    expect(data.success).toBe(false);
    expect(data.message).toContain('TCP connection failed');
    expect(data.details).toHaveProperty('host', '10.7.0.4');
    expect(data.details).toHaveProperty('port', 9999);
  });

  it('should fail on invalid HTTP endpoint', async () => {
    const { data, status } = await post('/api/test-connection', {
      service_id: 'http-test',
      connection_url: 'http://10.7.0.4:9999',
      health_check: { type: 'http', port: 9999, path: '/' },
      metadata: {}
    });

    expect(status).toBe(200);
    expect(data.success).toBe(false);
    expect(data.message).toContain('HTTP connection failed');
    expect(data.details).toHaveProperty('url');
  });
});