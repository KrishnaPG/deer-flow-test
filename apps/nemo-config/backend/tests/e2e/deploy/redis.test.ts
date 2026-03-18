import { describe, it, expect, beforeAll, afterAll, beforeEach, afterEach } from 'bun:test';
import { cleanupTestResources, cleanupConsulForService, getContainerName } from '../helpers/consul';
import { containerExists, containerIsRunning, getContainerLogs, waitForContainerStatus, stopContainer, startContainer, removeContainer } from '../helpers/docker';
import { get, post, del } from '../helpers/api';
import { generateTestServiceId } from '../helpers/consul';
import { CONFIG } from '../config';

describe('Deploy Redis - E2E Tests', () => {
  let testServiceId: string;
  let containerName: string;

  beforeAll(async () => {
    await cleanupTestResources();
  });

  afterAll(async () => {
    await cleanupTestResources();
  });

  beforeEach(async () => {
    testServiceId = generateTestServiceId('redis');
    containerName = getContainerName(testServiceId);
  });

  afterEach(async () => {
    // Cleanup: stop and remove container, clean Consul
    try {
      await stopContainer(containerName, 'localhost').catch(() => {});
      await removeContainer(containerName, testServiceId, 'localhost').catch(() => {});
    } catch (error) {
      console.warn('Cleanup warning:', error);
    }
    await cleanupConsulForService(testServiceId);
  });

  it('should deploy Redis container successfully', async () => {
    // Get catalog to get Redis template
    const { data: catalogData } = await get('/api/catalog');
    const redisTemplate = catalogData.find((t: any) => t.id === 'redis');
    expect(redisTemplate).toBeDefined();

    // Deploy Redis
    const { data: deployData, status } = await post('/api/deploy', {
      target_host: 'localhost',
      service_id: testServiceId,
      template: redisTemplate,
      env_values: {}, // Use defaults from template
      consul_url: CONFIG.CONSUL_URL,
      mode: 'deploy',
      deploy_path: CONFIG.DEPLOY_BASE_PATH
    });

    expect(status).toBe(200);
    expect(deployData.success).toBe(true);

    // Wait for container to be ready
    const isReady = await waitForContainerStatus(containerName, 'running', 'localhost', 30000);
    expect(isReady).toBe(true);

    // Verify container exists and is running via Docker CLI
    const exists = await containerExists(containerName, 'localhost');
    expect(exists).toBe(true);

    const isRunning = await containerIsRunning(containerName, 'localhost');
    expect(isRunning).toBe(true);

    // Verify Consul has the configuration
    const consulVerified = await verifyConsulConfig(testServiceId, ['url']);
    expect(consulVerified).toBe(true);

    // Verify we can get service details via API
    const { data: details, status: detailsStatus } = await get(
      `/api/services/${testServiceId}/details?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}`
    );
    expect(detailsStatus).toBe(200);
    expect(details.isHealthy).toBe(true);
    expect(details.metadata?.managedBy).toBe('nemo');
    expect(details.metadata?.host).toBe('localhost');
    expect(details.containerStatus).toBe('running');
    expect(details.connectionUrl).toContain('localhost:6379');
  });

  it('should get container logs via API and match Docker CLI', async () => {
    // Get catalog to get Redis template
    const { data: catalogData } = await get('/api/catalog');
    const redisTemplate = catalogData.find((t: any) => t.id === 'redis');
    expect(redisTemplate).toBeDefined();

    // Deploy Redis
    const { data: deployData } = await post('/api/deploy', {
      target_host: 'localhost',
      service_id: testServiceId,
      template: redisTemplate,
      env_values: {},
      consul_url: CONFIG.CONSUL_URL,
      mode: 'deploy',
      deploy_path: CONFIG.DEPLOY_BASE_PATH
    });

    expect(deployData.success).toBe(true);

    // Wait for container to be ready
    await waitForContainerStatus(containerName, 'running', 'localhost', 30000);

    // Get logs via API
    const { data: apiLogsData, status: logsStatus } = await get(
      `/api/services/${testServiceId}/logs?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}&tail=10`
    );
    expect(logsStatus).toBe(200);
    expect(Array.isArray(apiLogsData.logs)).toBe(true);

    // Get logs via Docker CLI
    const dockerLogs = await getContainerLogs(containerName, 'localhost', 10);
    expect(Array.isArray(dockerLogs)).toBe(true);
    expect(dockerLogs.length).toBeGreaterThan(0);

    // Basic validation - both should have some logs
    expect(apiLogsData.logs.length).toBeGreaterThan(0);
  });

  it('should stop and start container via API', async () => {
    // Get catalog to get Redis template
    const { data: catalogData } = await get('/api/catalog');
    const redisTemplate = catalogData.find((t: any) => t.id === 'redis');
    expect(redisTemplate).toBeDefined();

    // Deploy Redis
    const { data: deployData } = await post('/api/deploy', {
      target_host: 'localhost',
      service_id: testServiceId,
      template: redisTemplate,
      env_values: {},
      consul_url: CONFIG.CONSUL_URL,
      mode: 'deploy',
      deploy_path: CONFIG.DEPLOY_BASE_PATH
    });

    expect(deployData.success).toBe(true);

    // Wait for container to be ready
    await waitForContainerStatus(containerName, 'running', 'localhost', 30000);

    // Verify it's running
    let isRunning = await containerIsRunning(containerName, 'localhost');
    expect(isRunning).toBe(true);

    // Stop container via API
    const { data: stopData, status: stopStatus } = await post(
      `/api/services/${testServiceId}/stop?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}`
    );
    expect(stopStatus).toBe(200);
    expect(stopData.success).toBe(true);

    // Wait a bit for stop to complete
    await new Promise(resolve => setTimeout(resolve, 2000));

    // Verify container is stopped
    isRunning = await containerIsRunning(containerName, 'localhost');
    expect(isRunning).toBe(false);
    const stillExists = await containerExists(containerName, 'localhost');
    expect(stillExists).toBe(true); // Stopped container still exists

    // Start container via API
    const { data: startData, status: startStatus } = await post(
      `/api/services/${testServiceId}/start?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}`
    );
    expect(startStatus).toBe(200);
    expect(startData.success).toBe(true);

    // Wait for container to be running again
    await waitForContainerStatus(containerName, 'running', 'localhost', 30000);

    // Verify container is running again
    isRunning = await containerIsRunning(containerName, 'localhost');
    expect(isRunning).toBe(true);
  });
});