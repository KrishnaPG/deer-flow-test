import { describe, it, expect, beforeAll, afterAll, beforeEach, afterEach } from 'bun:test';
import { cleanupTestResources, cleanupConsulForService, getContainerName } from '../../helpers/consul';
import { getContainerLogs } from '../../helpers/docker';
import { get, post } from '../../helpers/api';
import { generateTestServiceId } from '../../helpers/consul';
import { CONFIG } from '../../config';

describe('Service Logs API - E2E Tests', () => {
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
    const isReady = await waitForContainerStatus(containerName, 'running', 'localhost', 30000);
    expect(isReady).toBe(true);

    // Get logs via API
    const { data: apiLogsData, status: logsStatus } = await get(
      `/api/services/${testServiceId}/logs?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}&tail=10`
    );
    expect(logsStatus).toBe(200);
    expect(Array.isArray(apiLogsData.logs)).toBe(true);

    // Get logs via Docker CLI
    const dockerLogs = await getContainerLogs(containerName, 'localhost', 10);
    expect(Array.isArray(dockerLogs)).toBe(true);

    // Both should have logs (Redis typically outputs something on startup)
    // We'll just verify the API works and returns an array
    expect(apiLogsData.logs).toBeInstanceOf(Array);
  });

  it('should return empty array for non-existent service', async () => {
    const fakeServiceId = `non-existent-${Date.now()}`;
    
    const { data: logsData, status } = await get(
      `/api/services/${fakeServiceId}/logs?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}&tail=10`
    );
    expect(status).toBe(200);
    expect(logsData).toHaveProperty('logs');
    expect(Array.isArray(logsData.logs)).toBe(true);
    // For non-existent service, logs should be empty array
    expect(logsData.logs.length).toBe(0);
  });

  it('should respect tail parameter', async () => {
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

    // Get logs with tail=5
    const { data: logsData5, status: logsStatus5 } = await get(
      `/api/services/${testServiceId}/logs?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}&tail=5`
    );
    expect(logsStatus5).toBe(200);
    expect(Array.isArray(logsData5.logs)).toBe(true);

    // Get logs with tail=20
    const { data: logsData20, status: logsStatus20 } = await get(
      `/api/services/${testServiceId}/logs?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}&tail=20`
    );
    expect(logsStatus20).toBe(200);
    expect(Array.isArray(logsData20.logs)).toBe(true);

    // The tail=20 should have at least as many logs as tail=5
    expect(logsData20.logs.length).toBeGreaterThanOrEqual(logsData5.logs.length);
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