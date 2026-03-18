import { describe, it, expect, beforeAll, afterAll, beforeEach, afterEach } from 'bun:test';
import { cleanupTestResources, cleanupConsulForService, getContainerName, getNextTestPort } from '../helpers/consul';
import { containerExists, containerIsRunning, getContainerLogs, stopContainer, removeContainer } from '../helpers/docker';
import { get, post } from '../helpers/api';
import { generateTestServiceId } from '../helpers/consul';
import { CONFIG } from '../config';

const TARGET_HOST = CONFIG.DEFAULT_TARGET_HOST;

describe('Service Logs API - E2E Tests', () => {
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

  it('should get container logs via API', async () => {
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

    const { data: apiLogsData, status: logsStatus } = await get(
      `/api/services/${testServiceId}/logs?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}&tail=10`
    );
    expect(logsStatus).toBe(200);
    expect(Array.isArray(apiLogsData.logs)).toBe(true);
  }, 120000);

  it('should return empty array for non-existent service', async () => {
    const fakeServiceId = `non-existent-${Date.now()}`;
    
    const { data: logsData, status } = await get(
      `/api/services/${fakeServiceId}/logs?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}&tail=10`
    );
    expect(status).toBe(200);
    expect(logsData).toHaveProperty('logs');
    expect(Array.isArray(logsData.logs)).toBe(true);
  });
});

async function waitForContainerStatus(
  containerName: string,
  expectedStatus: 'running' | 'stopped' | 'exited',
  host: string,
  timeoutMs: number = 60000
): Promise<boolean> {
  const startTime = Date.now();

  while (Date.now() - startTime < timeoutMs) {
    let isRunning = false;
    try {
      isRunning = await containerIsRunning(containerName, host);
    } catch (error) {
    }

    const isExited = !isRunning && await containerExists(containerName, host);

    if (
      (expectedStatus === 'running' && isRunning) ||
      (expectedStatus === 'stopped' && !isRunning && await containerExists(containerName, host)) ||
      (expectedStatus === 'exited' && isExited)
    ) {
      return true;
    }

    await new Promise(resolve => setTimeout(resolve, 500));
  }

  return false;
}
