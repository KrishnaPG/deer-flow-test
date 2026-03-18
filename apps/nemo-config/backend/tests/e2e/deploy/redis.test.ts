import { describe, it, expect, beforeAll, afterAll } from 'bun:test';
import { cleanupTestResources, cleanupConsulForService, getContainerName } from '../helpers/consul';
import { containerExists, containerIsRunning, waitForContainerStatus, stopContainer, removeContainer } from '../helpers/docker';
import { get, post } from '../helpers/api';
import { generateTestServiceId } from '../helpers/consul';
import { CONFIG } from '../config';

const TARGET_HOST = CONFIG.DEFAULT_TARGET_HOST;
const TEST_TIMEOUT = 120000; // 2 minutes per test

describe('Deploy Redis - E2E Tests', () => {
  let testServiceId: string;
  let containerName: string;

  beforeAll(async () => {
    await cleanupTestResources();
  }, 60000);

  afterAll(async () => {
    await cleanupTestResources();
  }, 60000);

  it('should deploy Redis container successfully', async () => {
    testServiceId = generateTestServiceId('redis');
    containerName = getContainerName(testServiceId);

    const { data: catalogData } = await get('/api/catalog');
    const redisTemplate = catalogData.find((t: any) => t.id === 'redis');
    expect(redisTemplate).toBeDefined();

    const { data: deployData, status } = await post('/api/deploy', {
      target_host: TARGET_HOST,
      service_id: testServiceId,
      template: redisTemplate,
      env_values: {},
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

    // Clean up inline
    await stopContainer(containerName, TARGET_HOST).catch(() => {});
    await removeContainer(containerName, testServiceId, TARGET_HOST).catch(() => {});
    await cleanupConsulForService(testServiceId);
  }, TEST_TIMEOUT);

  it('should get container logs via API', async () => {
    testServiceId = generateTestServiceId('redis-logs');
    containerName = getContainerName(testServiceId);

    const { data: catalogData } = await get('/api/catalog');
    const redisTemplate = catalogData.find((t: any) => t.id === 'redis');
    expect(redisTemplate).toBeDefined();

    const { data: deployData } = await post('/api/deploy', {
      target_host: TARGET_HOST,
      service_id: testServiceId,
      template: redisTemplate,
      env_values: {},
      consul_url: CONFIG.CONSUL_URL,
      mode: 'deploy',
      deploy_path: CONFIG.DEPLOY_BASE_PATH
    });

    expect(deployData.success).toBe(true);

    await waitForContainerStatus(containerName, 'running', TARGET_HOST, 60000);

    const { data: apiLogsData, status: logsStatus } = await get(
      `/api/services/${testServiceId}/logs?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}&tail=10`
    );
    expect(logsStatus).toBe(200);
    expect(Array.isArray(apiLogsData.logs)).toBe(true);

    // Clean up inline
    await stopContainer(containerName, TARGET_HOST).catch(() => {});
    await removeContainer(containerName, testServiceId, TARGET_HOST).catch(() => {});
    await cleanupConsulForService(testServiceId);
  }, TEST_TIMEOUT);

  it('should stop and start container via API', async () => {
    testServiceId = generateTestServiceId('redis-ops');
    containerName = getContainerName(testServiceId);

    const { data: catalogData } = await get('/api/catalog');
    const redisTemplate = catalogData.find((t: any) => t.id === 'redis');
    expect(redisTemplate).toBeDefined();

    const { data: deployData } = await post('/api/deploy', {
      target_host: TARGET_HOST,
      service_id: testServiceId,
      template: redisTemplate,
      env_values: {},
      consul_url: CONFIG.CONSUL_URL,
      mode: 'deploy',
      deploy_path: CONFIG.DEPLOY_BASE_PATH
    });

    expect(deployData.success).toBe(true);

    await waitForContainerStatus(containerName, 'running', TARGET_HOST, 60000);

    const { data: stopData, status: stopStatus } = await post(
      `/api/services/${testServiceId}/stop?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}`
    );
    expect(stopStatus).toBe(200);
    expect(stopData.success).toBe(true);

    await new Promise(resolve => setTimeout(resolve, 3000));

    const isRunning = await containerIsRunning(containerName, TARGET_HOST);
    expect(isRunning).toBe(false);

    const { data: startData, status: startStatus } = await post(
      `/api/services/${testServiceId}/start?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}`
    );
    expect(startStatus).toBe(200);
    expect(startData.success).toBe(true);

    await waitForContainerStatus(containerName, 'running', TARGET_HOST, 30000);

    const isRunningAgain = await containerIsRunning(containerName, TARGET_HOST);
    expect(isRunningAgain).toBe(true);

    // Clean up inline
    await stopContainer(containerName, TARGET_HOST).catch(() => {});
    await removeContainer(containerName, testServiceId, TARGET_HOST).catch(() => {});
    await cleanupConsulForService(testServiceId);
  }, TEST_TIMEOUT);
});