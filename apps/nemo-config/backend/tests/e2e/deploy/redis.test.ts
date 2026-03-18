import { describe, it, expect, beforeAll, afterAll } from 'bun:test';
import { cleanupTestResources, cleanupConsulForService, getContainerName, getNextTestPort } from '../helpers/consul';
import { containerExists, containerIsRunning, waitForContainerStatus, stopContainer, removeContainer } from '../helpers/docker';
import { get, post } from '../helpers/api';
import { generateTestServiceId } from '../helpers/consul';
import { CONFIG } from '../config';

const TARGET_HOST = CONFIG.DEFAULT_TARGET_HOST;
const TEST_TIMEOUT = 120000;

describe('Deploy Redis - E2E Tests', () => {
  let testServiceId: string;
  let containerName: string;
  let testPort: number;

  beforeAll(async () => {
    await cleanupTestResources();
  }, 60000);

  afterAll(async () => {
    await cleanupTestResources();
  }, 60000);

  it('should deploy Redis container successfully with PORT override', async () => {
    testServiceId = generateTestServiceId('redis');
    containerName = getContainerName(testServiceId);
    testPort = getNextTestPort();

    const { data: catalogData } = await get('/api/catalog');
    const redisTemplate = catalogData.find((t: any) => t.id === 'redis');
    expect(redisTemplate).toBeDefined();

    const { data: deployData, status } = await post('/api/deploy', {
      target_host: TARGET_HOST,
      service_id: testServiceId,
      template: redisTemplate,
      env_values: { PORT: String(testPort) },
      consul_url: CONFIG.CONSUL_URL,
      mode: 'deploy',
      deploy_path: CONFIG.DEPLOY_BASE_PATH
    });

    expect(status).toBe(200);
    expect(deployData.success).toBe(true);

    const isReady = await waitForContainerStatus(containerName, 'running', TARGET_HOST, 60000);
    expect(isReady).toBe(true);

    const exists = await containerExists(containerName, TARGET_HOST);
    expect(exists).toBe(true);

    const running = await containerIsRunning(containerName, TARGET_HOST);
    expect(running).toBe(true);

    const { data: details } = await get(
      `/api/services/${testServiceId}/details?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}`
    );
    expect(details.connectionUrl).toContain(`${TARGET_HOST}:${testPort}`);

    await stopContainer(containerName, TARGET_HOST).catch(() => {});
    await removeContainer(containerName, testServiceId, TARGET_HOST).catch(() => {});
    await cleanupConsulForService(testServiceId);
  }, TEST_TIMEOUT);

  it('should deploy Redis and get container logs via API', async () => {
    testServiceId = generateTestServiceId('redis-logs');
    containerName = getContainerName(testServiceId);
    testPort = getNextTestPort();

    const { data: catalogData } = await get('/api/catalog');
    const redisTemplate = catalogData.find((t: any) => t.id === 'redis');
    expect(redisTemplate).toBeDefined();

    const { data: deployData, status } = await post('/api/deploy', {
      target_host: TARGET_HOST,
      service_id: testServiceId,
      template: redisTemplate,
      env_values: { PORT: String(testPort) },
      consul_url: CONFIG.CONSUL_URL,
      mode: 'deploy',
      deploy_path: CONFIG.DEPLOY_BASE_PATH
    });

    expect(status).toBe(200);
    expect(deployData.success).toBe(true);

    const isReady = await waitForContainerStatus(containerName, 'running', TARGET_HOST, 60000);
    expect(isReady).toBe(true);

    const { data: logsData, status: logsStatus } = await get(
      `/api/services/${testServiceId}/logs?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}&tail=10`
    );
    expect(logsStatus).toBe(200);
    expect(Array.isArray(logsData.logs)).toBe(true);

    await stopContainer(containerName, TARGET_HOST).catch(() => {});
    await removeContainer(containerName, testServiceId, TARGET_HOST).catch(() => {});
    await cleanupConsulForService(testServiceId);
  }, TEST_TIMEOUT);

  it('should stop and start container via API', async () => {
    testServiceId = generateTestServiceId('redis-ops');
    containerName = getContainerName(testServiceId);
    testPort = getNextTestPort();

    const { data: catalogData } = await get('/api/catalog');
    const redisTemplate = catalogData.find((t: any) => t.id === 'redis');
    expect(redisTemplate).toBeDefined();

    const { data: deployData, status } = await post('/api/deploy', {
      target_host: TARGET_HOST,
      service_id: testServiceId,
      template: redisTemplate,
      env_values: { PORT: String(testPort) },
      consul_url: CONFIG.CONSUL_URL,
      mode: 'deploy',
      deploy_path: CONFIG.DEPLOY_BASE_PATH
    });

    expect(status).toBe(200);
    expect(deployData.success).toBe(true);

    const isReady = await waitForContainerStatus(containerName, 'running', TARGET_HOST, 60000);
    expect(isReady).toBe(true);

    const { data: stopData, status: stopStatus } = await post(
      `/api/services/${testServiceId}/stop?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}`
    );
    expect(stopStatus).toBe(200);
    expect(stopData.success).toBe(true);

    await new Promise(resolve => setTimeout(resolve, 2000));

    const { data: startData, status: startStatus } = await post(
      `/api/services/${testServiceId}/start?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}`
    );
    expect(startStatus).toBe(200);
    expect(startData.success).toBe(true);

    await waitForContainerStatus(containerName, 'running', TARGET_HOST, 30000);

    await stopContainer(containerName, TARGET_HOST).catch(() => {});
    await removeContainer(containerName, testServiceId, TARGET_HOST).catch(() => {});
    await cleanupConsulForService(testServiceId);
  }, TEST_TIMEOUT);
});
