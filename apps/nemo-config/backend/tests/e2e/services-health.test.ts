import { describe, it, expect, beforeAll, afterAll, beforeEach, afterEach } from 'bun:test';
import { cleanupTestResources, cleanupConsulForService, generateTestServiceId, getNextTestPort, getContainerName } from './helpers/consul';
import { stopContainer, removeContainer } from './helpers/docker';
import { get, post } from './helpers/api';
import { CONFIG } from './config';

const TARGET_HOST = CONFIG.DEFAULT_TARGET_HOST;

describe('Services Health API - E2E Tests', () => {
  let testServiceId: string;
  let containerName: string;
  let testPort: number;

  beforeAll(async () => {
    await cleanupTestResources();
  });

  afterAll(async () => {
    await cleanupTestResources();
  });

  beforeEach(async () => {
    testServiceId = generateTestServiceId('test-service');
    containerName = getContainerName(testServiceId);
    testPort = getNextTestPort();
  });

  afterEach(async () => {
    try {
      await stopContainer(containerName, TARGET_HOST).catch(() => {});
      await removeContainer(containerName, testServiceId, TARGET_HOST).catch(() => {});
    } catch (error) {
      console.warn('Cleanup warning:', error);
    }
    await cleanupConsulForService(testServiceId);
  });

  it('should return health status for all registered services', async () => {
    const catalogResponse = await get('/api/catalog');
    const redisTemplate = catalogResponse.data.find((t: any) => t.id === 'redis');
    
    const deployResponse = await post('/api/deploy', {
      target_host: TARGET_HOST,
      service_id: testServiceId,
      template: redisTemplate,
      env_values: { PORT: String(testPort) },
      consul_url: CONFIG.CONSUL_URL,
      mode: 'deploy',
      deploy_path: CONFIG.DEPLOY_BASE_PATH
    });
    
    expect(deployResponse.status).toBe(200);
    expect(deployResponse.data.success).toBe(true);
    
    await new Promise(resolve => setTimeout(resolve, 5000));
    
    const { data: healthStatus, status } = await get(
      `/api/health/services?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}`
    );
    
    expect(status).toBe(200);
    expect(typeof healthStatus).toBe('object');
    expect(healthStatus !== null).toBe(true);
    
    const serviceKeys = Object.keys(healthStatus);
    expect(serviceKeys.length).toBeGreaterThan(0);
    
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
  }, 120000);

  it('should show deployed services as healthy', async () => {
    const catalogResponse = await get('/api/catalog');
    const redisTemplate = catalogResponse.data.find((t: any) => t.id === 'redis');
    
    const deployResponse = await post('/api/deploy', {
      target_host: TARGET_HOST,
      service_id: testServiceId,
      template: redisTemplate,
      env_values: { PORT: String(testPort) },
      consul_url: CONFIG.CONSUL_URL,
      mode: 'deploy',
      deploy_path: CONFIG.DEPLOY_BASE_PATH
    });
    
    expect(deployResponse.status).toBe(200);
    expect(deployResponse.data.success).toBe(true);
    
    await new Promise(resolve => setTimeout(resolve, 10000));
    
    const { data: healthStatus, status } = await get(
      `/api/health/services?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}`
    );
    
    expect(status).toBe(200);
    
    let serviceHealth = null;
    if (healthStatus[testServiceId]) {
      serviceHealth = healthStatus[testServiceId];
    } else {
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
    expect(serviceHealth.host).toBe(TARGET_HOST);
    expect(serviceHealth.connectionUrl).toContain(TARGET_HOST);
  }, 120000);
});
