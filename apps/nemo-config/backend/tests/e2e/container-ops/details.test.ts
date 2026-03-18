import { describe, it, expect, beforeAll, afterAll, beforeEach, afterEach } from 'bun:test';
import { cleanupTestResources, cleanupConsulForService, getContainerName } from '../../helpers/consul';
import { containerExists, containerIsRunning } from '../../helpers/docker';
import { get, post } from '../../helpers/api';
import { generateTestServiceId } from '../../helpers/consul';
import { CONFIG } from '../../config';

describe('Service Details API - E2E Tests', () => {
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
      await containerExists(containerName, 'localhost') && await stopContainer(containerName, 'localhost');
      await containerExists(containerName, 'localhost') && await removeContainer(containerName, testServiceId, 'localhost');
    } catch (error) {
      console.warn('Cleanup warning:', error);
    }
    await cleanupConsulForService(testServiceId);
  });

  it('should return details for a deployed Redis service', async () => {
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
    const exists = await containerExists(containerName, 'localhost');
    expect(exists).toBe(true);

    const isRunning = await containerIsRunning(containerName, 'localhost');
    expect(isRunning).toBe(true);

    // Get service details via API
    const { data: details, status } = await get(
      `/api/services/${testServiceId}/details?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}`
    );
    expect(status).toBe(200);
    expect(details).toHaveProperty('metadata');
    expect(details).toHaveProperty('connectionUrl');
    expect(details).toHaveProperty('isHealthy');
    expect(details).toHaveProperty('containerStatus');
    
    // Validate the details
    expect(details.metadata).not.toBeNull();
    expect(details.metadata?.serviceId).toBe(testServiceId);
    expect(details.metadata?.containerName).toBe(containerName);
    expect(details.metadata?.managedBy).toBe('nemo');
    expect(details.metadata?.host).toBe('localhost');
    expect(details.connectionUrl).toContain('localhost:6379');
    expect(details.isHealthy).toBe(true);
    expect(details.containerStatus).toBe('running');
  });

  it('should return details for an external (registered existing) service', async () => {
    // Get catalog to get Redis template
    const { data: catalogData } = await get('/api/catalog');
    const redisTemplate = catalogData.find((t: any) => t.id === 'redis');
    expect(redisTemplate).toBeDefined();

    // Register existing Redis service
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

    // Get service details via API
    const { data: details, status } = await get(
      `/api/services/${testServiceId}/details?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}`
    );
    expect(status).toBe(200);
    expect(details).toHaveProperty('metadata');
    expect(details).toHaveProperty('connectionUrl');
    expect(details).toHaveProperty('isHealthy');
    expect(details).toHaveProperty('containerStatus');
    
    // Validate the details for external service
    expect(details.metadata).not.toBeNull();
    expect(details.metadata?.serviceId).toBe(testServiceId);
    expect(details.metadata?.managedBy).toBe('external');
    expect(details.metadata?.host).toBe('external');
    expect(details.metadata?.containerName).toBe(''); // External services don't have container names
    expect(details.connectionUrl).toBe('redis://10.7.0.4:6379');
    expect(details.isHealthy).toBe(true);
    expect(details.containerStatus).toBeUndefined(); // External services don't have container status
  });

  it('should return undefined container status for non-existent service', async () => {
    const fakeServiceId = `non-existent-${Date.now()}`;
    
    const { data: details, status } = await get(
      `/api/services/${fakeServiceId}/details?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}`
    );
    expect(status).toBe(200);
    expect(details).toHaveProperty('metadata');
    expect(details).toHaveProperty('connectionUrl');
    expect(details).toHaveProperty('isHealthy');
    expect(details).toHaveProperty('containerStatus');
    
    // For non-existent service, metadata should be null, connectionUrl null, isHealthy false
    expect(details.metadata).toBeNull();
    expect(details.connectionUrl).toBeNull();
    expect(details.isHealthy).toBe(false);
    expect(details.containerStatus).toBeUndefined();
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