import { describe, it, expect, beforeAll, afterAll, beforeEach, afterEach } from 'bun:test';
import { cleanupTestResources, cleanupConsulForService, getContainerName, getNextTestPort } from '../helpers/consul';
import { containerExists, containerIsRunning, waitForContainerStatus, stopContainer, removeContainer } from '../helpers/docker';
import { get, post } from '../helpers/api';
import { generateTestServiceId } from '../helpers/consul';
import { CONFIG } from '../config';

const TARGET_HOST = CONFIG.DEFAULT_TARGET_HOST;

describe('Service Details API - E2E Tests', () => {
  let testServiceId: string;
  let containerName: string;
  let testPort: number;

  beforeAll(async () => {
    await cleanupTestResources();
  }, 60000);

  afterAll(async () => {
    await cleanupTestResources();
  }, 60000);

  beforeEach(async () => {
    testServiceId = generateTestServiceId('redis');
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

  it('should return details for a deployed Redis service', async () => {
    const { data: catalogData } = await get('/api/catalog');
    const redisTemplate = catalogData.find((t: any) => t.id === 'redis');
    expect(redisTemplate).toBeDefined();

    const { data: deployData, status: deployStatus } = await post('/api/deploy', {
      target_host: TARGET_HOST,
      service_id: testServiceId,
      template: redisTemplate,
      env_values: { PORT: String(testPort) },
      consul_url: CONFIG.CONSUL_URL,
      mode: 'deploy',
      deploy_path: CONFIG.DEPLOY_BASE_PATH
    });

    expect(deployStatus).toBe(200);
    expect(deployData.success).toBe(true);

    const isReady = await waitForContainerStatus(containerName, 'running', TARGET_HOST, 60000);
    expect(isReady).toBe(true);

    const exists = await containerExists(containerName, TARGET_HOST);
    expect(exists).toBe(true);

    const isRunning = await containerIsRunning(containerName, TARGET_HOST);
    expect(isRunning).toBe(true);

    const { data: details, status } = await get(
      `/api/services/${testServiceId}/details?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}`
    );
    expect(status).toBe(200);
    expect(details).toHaveProperty('metadata');
    expect(details).toHaveProperty('connectionUrl');
    expect(details).toHaveProperty('isHealthy');
    expect(details).toHaveProperty('containerStatus');
    
    expect(details.metadata).not.toBeNull();
    expect(details.metadata?.serviceId).toBe(testServiceId);
    expect(details.metadata?.containerName).toBe(containerName);
    expect(details.metadata?.managedBy).toBe('nemo');
    expect(details.metadata?.host).toBe(TARGET_HOST);
    expect(details.connectionUrl).toContain(`${TARGET_HOST}:${testPort}`);
    expect(details.isHealthy).toBe(true);
    expect(details.containerStatus).toBe('running');
  }, 120000);

  it('should return details for an external (registered existing) service', async () => {
    const { data: catalogData } = await get('/api/catalog');
    const redisTemplate = catalogData.find((t: any) => t.id === 'redis');
    expect(redisTemplate).toBeDefined();

    const { data: registerData } = await post('/api/register-existing', {
      service_id: testServiceId,
      connection_url: `redis://${TARGET_HOST}:6379`,
      consul_url: CONFIG.CONSUL_URL,
      template: redisTemplate,
      env_values: {}
    });

    expect(registerData.success).toBe(true);

    await new Promise(resolve => setTimeout(resolve, 2000));

    const { data: details, status } = await get(
      `/api/services/${testServiceId}/details?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}`
    );
    expect(status).toBe(200);
    expect(details.metadata?.managedBy).toBe('external');
    expect(details.metadata?.host).toBe('external');
    expect(details.connectionUrl).toBe(`redis://${TARGET_HOST}:6379`);
  });

  it('should return undefined container status for non-existent service', async () => {
    const fakeServiceId = `non-existent-${Date.now()}`;
    
    const { data: details, status } = await get(
      `/api/services/${fakeServiceId}/details?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}`
    );
    expect(status).toBe(200);
    expect(details.metadata).toBeNull();
    expect(details.connectionUrl).toBeNull();
    expect(details.isHealthy).toBe(false);
  });
});

// Helper function to wait for container status (shared)
async function waitForContainerStatus(
  containerName: string, 
  expectedStatus: 'running' | 'stopped' | 'exited', 
  host: string = 'localhost',
  timeoutMs: number = 30000
): Promise<boolean> {
  const startTime = Date.now();
  
  while (Date.now() - startTime < timeoutMs) {
    let isRunning = false;
    try {
      isRunning = await containerIsRunning(containerName, host);
    } catch (error) {
      // Container might not exist yet
    }
    
    const isExited = !isRunning && await containerExists(containerName, host);
    
    if (
      (expectedStatus === 'running' && isRunning) ||
      (expectedStatus === 'stopped' && !isRunning && await containerExists(containerName, host)) ||
      (expectedStatus === 'exited' && isExited)
    ) {
      return true;
    }
    
    // Wait a bit before checking again
    await new Promise(resolve => setTimeout(resolve, 500));
  }
  
  return false;
}