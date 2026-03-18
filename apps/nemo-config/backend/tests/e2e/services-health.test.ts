import { describe, it, expect, beforeAll, afterAll, beforeEach, afterEach } from 'bun:test';
import { cleanupTestResources, cleanupConsulForService } from './helpers/consul';
import { get, post } from './helpers/api';
import { generateTestServiceId } from './helpers/consul';
import { CONFIG } from './config';

describe('Services Health API - E2E Tests', () => {
  let testServiceId: string;

  beforeAll(async () => {
    await cleanupTestResources();
  });

  afterAll(async () => {
    await cleanupTestResources();
  });

  beforeEach(async () => {
    testServiceId = generateTestServiceId('test-service');
  });

  afterEach(async () => {
    await cleanupConsulForService(testServiceId);
  });

  it('should return health status for all registered services', async () => {
    // Deploy a test service to have something to check
    const catalogResponse = await get('/api/catalog');
    const redisTemplate = catalogResponse.data.find((t: any) => t.id === 'redis');
    
    const deployResponse = await post('/api/deploy', {
      target_host: 'localhost',
      service_id: testServiceId,
      template: redisTemplate,
      env_values: {},
      consul_url: CONFIG.CONSUL_URL,
      mode: 'deploy',
      deploy_path: CONFIG.DEPLOY_BASE_PATH
    });
    
    expect(deployResponse.status).toBe(200);
    expect(deployResponse.data.success).toBe(true);
    
    // Wait for deployment to complete and Consul to be updated
    await new Promise(resolve => setTimeout(resolve, 5000));
    
    // Get health status for all services
    const { data: healthStatus, status } = await get(
      `/api/health/services?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}`
    );
    
    expect(status).toBe(200);
    expect(typeof healthStatus).toBe('object');
    expect(healthStatus !== null).toBe(true);
    
    // Should contain our test service
    const serviceKeys = Object.keys(healthStatus);
    expect(serviceKeys.length).toBeGreaterThan(0);
    
    // Find our service in the health status
    let found = false;
    for (const key of serviceKeys) {
      if (key === testServiceId || healthStatus[key].serviceId === testServiceId) {
        found = true;
        const serviceHealth = healthStatus[key];
        expect(serviceHealth).toHaveProperty('serviceId');
        expect(serviceHealth).toHaveProperty('connectionUrl');
        expect(serviceHealth).toHaveProperty('isHealthy');
        expect(serviceHealth).toHaveProperty('managedBy');
        expect(serviceHealth).toHaveProperty('host');
        break;
      }
    }
    
    expect(found).toBe(true);
  });

  it('should show deployed services as healthy', async () => {
    // Deploy a service
    const catalogResponse = await get('/api/catalog');
    const redisTemplate = catalogResponse.data.find((t: any) => t.id === 'redis');
    
    const deployResponse = await post('/api/deploy', {
      target_host: 'localhost',
      service_id: testServiceId,
      template: redisTemplate,
      env_values: {},
      consul_url: CONFIG.CONSUL_URL,
      mode: 'deploy',
      deploy_path: CONFIG.DEPLOY_BASE_PATH
    });
    
    expect(deployResponse.status).toBe(200);
    expect(deployResponse.data.success).toBe(true);
    
    // Wait for container to be ready
    await new Promise(resolve => setTimeout(resolve, 10000));
    
    // Check health status
    const { data: healthStatus, status } = await get(
      `/api/health/services?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}`
    );
    
    expect(status).toBe(200);
    
    // Find our service and verify it's healthy
    let serviceHealth = null;
    if (healthStatus[testServiceId]) {
      serviceHealth = healthStatus[testServiceId];
    } else {
      // Search through all services
      for (const key in healthStatus) {
        if (healthStatus[key].serviceId === testServiceId) {
          serviceHealth = healthStatus[key];
          break;
        }
      }
    }
    
    expect(serviceHealth).not.toBeNull();
    expect(serviceHealth.serviceId).toBe(testServiceId);
    expect(serviceHealth.isHealthy).toBe(true);
    expect(serviceHealth.managedBy).toBe('nemo');
    expect(serviceHealth.host).toBe('localhost');
    expect(serviceHealth.connectionUrl).toContain('localhost');
  });
});