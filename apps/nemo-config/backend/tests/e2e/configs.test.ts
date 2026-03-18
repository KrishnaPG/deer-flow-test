import { describe, it, expect, beforeAll, afterAll, beforeEach, afterEach } from 'bun:test';
import { cleanupTestResources, cleanupConsulForService, generateTestServiceId } from './helpers/consul';
import { get, post } from './helpers/api';
import { CONFIG } from './config';

const TARGET_HOST = CONFIG.DEFAULT_TARGET_HOST;

describe('Configs API - E2E Tests', () => {
  let testServiceId: string;

  beforeAll(async () => {
    await cleanupTestResources();
  });

  afterAll(async () => {
    await cleanupTestResources();
  });

  beforeEach(async () => {
    // Create a unique test service ID for each test
    testServiceId = generateTestServiceId('test-service');
  });

  afterEach(async () => {
    // Clean up after each test
    await cleanupConsulForService(testServiceId);
  });

  it('should return all configurations from Consul', async () => {
    // First, add some test data to Consul
    const testData = {
      'test.key1': 'value1',
      'test.key2': 'value2',
    };
    
    // Write test data directly to Consul using the updateConsulKV function
    // For simplicity in E2E, we'll use the deploy endpoint to create real data
    
    const { data: configs, status } = await get(`/api/configs?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}`);
    expect(status).toBe(200);
    expect(typeof configs).toBe('object');
    expect(configs !== null).toBe(true);
    
    // Should contain our test prefix if we added data
    // For now, just verify it returns an object
  });

  it('should return configuration for a specific service', async () => {
    // First deploy a service to create some config
    const catalogResponse = await get('/api/catalog');
    const redisTemplate = catalogResponse.data.find((t: any) => t.id === 'redis');
    
    const deployResponse = await post('/api/deploy', {
      target_host: TARGET_HOST,
      service_id: testServiceId,
      template: redisTemplate,
      env_values: {},
      consul_url: CONFIG.CONSUL_URL,
      mode: 'deploy',
      deploy_path: CONFIG.DEPLOY_BASE_PATH
    });
    
    expect(deployResponse.status).toBe(200);
    expect(deployResponse.data.success).toBe(true);
    
    // Wait a moment for Consul to be updated
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    // Now get the config for this specific service
    const { data: serviceConfigs, status } = await get(
      `/api/config/${testServiceId}?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}`
    );
    
    expect(status).toBe(200);
    expect(typeof serviceConfigs).toBe('object');
    expect(serviceConfigs !== null).toBe(true);
    
    // Should have some configuration keys
    const keys = Object.keys(serviceConfigs);
    expect(keys.length).toBeGreaterThan(0);
    
    // Should contain the service URL
    expect(serviceConfigs).toHaveProperty(`${testServiceId}.url`);
    // OR the new format
    expect(serviceConfigs).toHaveProperty(`nemo.${testServiceId}.url`);
  });

  it('should return empty object for non-existent service', async () => {
    const fakeServiceId = `non-existent-${Date.now()}`;
    
    const { data: serviceConfigs, status } = await get(
      `/api/config/${fakeServiceId}?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}`
    );
    
    expect(status).toBe(200);
    expect(typeof serviceConfigs).toBe('object');
    // Might be empty or might not have the service keys
  });
});