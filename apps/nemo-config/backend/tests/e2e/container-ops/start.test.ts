import { describe, it, expect, beforeAll, afterAll, beforeEach, afterEach } from 'bun:test';
import { cleanupTestResources, cleanupConsulForService, getContainerName } from '../../helpers/consul';
import { containerExists, containerIsRunning, stopContainer, startContainer } from '../../helpers/docker';
import { get, post } from '../../helpers/api';
import { generateTestServiceId } from '../../helpers/consul';
import { CONFIG } from '../../config';

describe('Container Start API - E2E Tests', () => {
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

  it('should start a stopped container via API', async () => {
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

    // Verify container exists and is running via Docker CLI
    let exists = await containerExists(containerName, 'localhost');
    expect(exists).toBe(true);

    let isRunning = await containerIsRunning(containerName, 'localhost');
    expect(isRunning).toBe(true);

    // Stop container first
    await stopContainer(containerName, 'localhost');
    
    // Wait a bit for stop to complete
    await new Promise(resolve => setTimeout(resolve, 2000));

    // Verify container is stopped via Docker CLI
    isRunning = await containerIsRunning(containerName, 'localhost');
    expect(isRunning).toBe(false);
    
    // Container should still exist but be stopped
    exists = await containerExists(containerName, 'localhost');
    expect(exists).toBe(true);

    // Start container via API
    const { data: startData, status } = await post(
      `/api/services/${testServiceId}/start?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}`
    );
    expect(status).toBe(200);
    expect(startData.success).toBe(true);

    // Wait for container to be running again
    await waitForContainerStatus(containerName, 'running', 'localhost', 30000);

    // Verify container is running again via Docker CLI
    isRunning = await containerIsRunning(containerName, 'localhost');
    expect(isRunning).toBe(true);

    // Verify via API that container status is updated
    const { data: details, status: detailsStatus } = await get(
      `/api/services/${testServiceId}/details?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}`
    );
    expect(detailsStatus).toBe(200);
    expect(details.containerStatus).toBe('running');
  });

  it('should return error when trying to start non-nemo managed service', async () => {
    // Get catalog to get Redis template
    const { data: catalogData } = await get('/api/catalog');
    const redisTemplate = catalogData.find((t: any) => t.id === 'redis');
    expect(redisTemplate).toBeDefined();

    // Register existing Redis service (not managed by nemo)
    const { data: registerData } = await post('/api/register-existing', {
      service_id: testServiceId,
      connection_url: 'redis://10.7.0.4:6379',
      consul_url: CONFIG.CONSUL_URL,
      template: redisTemplate,
      env_values: {}
    });

    expect(registerData.success).toBe(true);

    // Wait a moment for Consul to be updated
    await new Promise(resolve => setTimeout(resolve, 2000));

    // Try to start the external service - should fail
    const { data: startData, status } = await post(
      `/api/services/${testServiceId}/start?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}`
    );
    expect(status).toBe(500); // Should return error
    expect(startData.success).toBeUndefined(); // Error response doesn't have success field
    expect(startData.error).toContain('Cannot start: service is not managed by nemo');
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