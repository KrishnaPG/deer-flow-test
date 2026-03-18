import { describe, it, expect, beforeAll, afterAll, beforeEach, afterEach } from 'bun:test';
import { cleanupTestResources, cleanupConsulForService, getContainerName, getNextTestPort } from '../helpers/consul';
import { containerExists, containerIsRunning, stopContainer, startContainer, removeContainer } from '../helpers/docker';
import { get, post } from '../helpers/api';
import { generateTestServiceId } from '../helpers/consul';
import { CONFIG } from '../config';

const TARGET_HOST = CONFIG.DEFAULT_TARGET_HOST;

describe('Container Start API - E2E Tests', () => {
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

  it('should start a stopped container via API', async () => {
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

    await stopContainer(containerName, TARGET_HOST);
    await new Promise(resolve => setTimeout(resolve, 2000));

    let isRunning = await containerIsRunning(containerName, TARGET_HOST);
    expect(isRunning).toBe(false);

    const { data: startData, status } = await post(
      `/api/services/${testServiceId}/start?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}`
    );
    expect(status).toBe(200);
    expect(startData.success).toBe(true);

    await waitForContainerStatus(containerName, 'running', TARGET_HOST, 30000);

    isRunning = await containerIsRunning(containerName, TARGET_HOST);
    expect(isRunning).toBe(true);
  }, 120000);

  it('should return error when trying to start non-nemo managed service', async () => {
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

    const { data: startData, status } = await post(
      `/api/services/${testServiceId}/start?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}`
    );
    expect(status).toBe(500);
    expect(startData.error).toContain('Cannot start: service is not managed by nemo');
  });
});

async function waitForContainerStatus(
  containerName: string, 
  expectedStatus: 'running' | 'stopped' | 'exited', 
  host: string,
  timeoutMs: number = 30000
): Promise<boolean> {
  const startTime = Date.now();
  
  while (Date.now() - startTime < timeoutMs) {
    let isRunning = false;
    try {
      isRunning = await containerIsRunning(containerName, host);
    } catch (error) {}

    if (expectedStatus === 'running' && isRunning) {
      return true;
    }
    
    await new Promise(resolve => setTimeout(resolve, 500));
  }
  
  return false;
}