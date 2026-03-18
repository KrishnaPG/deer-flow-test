import { describe, it, expect, beforeAll, afterAll, beforeEach, afterEach } from 'bun:test';
import { cleanupTestResources, cleanupConsulForService, getContainerName } from './helpers/consul';
import { containerExists, removeContainer } from './helpers/docker';
import { get, post, del } from './helpers/api';
import { generateTestServiceId } from './helpers/consul';
import { CONFIG } from '../../config';

describe('Container Delete API - E2E Tests', () => {
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
    // Additional cleanup just in case
    try {
      await stopContainer(containerName, 'localhost').catch(() => {});
      await removeContainer(containerName, testServiceId, 'localhost').catch(() => {});
    } catch (error) {
      console.warn('Cleanup warning:', error);
    }
    await cleanupConsulForService(testServiceId);
  });

  it('should delete container and remove config for nemo-managed service', async () => {
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

    // Verify Consul has the configuration before deletion
    const consulVerifiedBefore = await verifyConsulConfig(testServiceId, ['url']);
    expect(consulVerifiedBefore).toBe(true);

    // Delete container via API
    const { data: deleteData, status } = await del(
      `/api/services/${testServiceId}/container?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}&deploy_path=${encodeURIComponent(CONFIG.DEPLOY_BASE_PATH)}`
    );
    expect(status).toBe(200);
    expect(deleteData.success).toBe(true);

    // Wait a bit for deletion to complete
    await new Promise(resolve => setTimeout(resolve, 2000));

    // Verify container is removed via Docker CLI
    exists = await containerExists(containerName, 'localhost');
    expect(exists).toBe(false);

    // Verify Consul configuration is removed
    const consulVerifiedAfter = await verifyConsulConfig(testServiceId, ['url']);
    expect(consulVerifiedAfter).toBe(false);

    // Verify we can't get service details anymore (should return null/empty)
    const { data: details, status: detailsStatus } = await get(
      `/api/services/${testServiceId}/details?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}`
    );
    expect(detailsStatus).toBe(200);
    expect(details.metadata).toBeNull();
    expect(details.connectionUrl).toBeNull();
    expect(details.isHealthy).toBe(false);
  });

  it('should remove config only for external (registered existing) service', async () => {
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

    // Verify Consul has the configuration before deletion
    const consulVerifiedBefore = await verifyConsulConfig(testServiceId, ['url']);
    expect(consulVerifiedBefore).toBe(true);

    // Delete config only via API (for external services)
    const { data: deleteData, status } = await del(
      `/api/services/${testServiceId}/config?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}`
    );
    expect(status).toBe(200);
    expect(deleteData.success).toBe(true);

    // Wait a bit for deletion to complete
    await new Promise(resolve => setTimeout(resolve, 2000));

    // Verify Consul configuration is removed
    const consulVerifiedAfter = await verifyConsulConfig(testServiceId, ['url']);
    expect(consulVerifiedAfter).toBe(false);

    // For external services, there should be no container to delete
    // (since we never deployed one through nemo)
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

// Helper function to remove container (shared)
async function removeContainer(containerName: string, serviceId: string, host: string = 'localhost'): Promise<void> {
  const expandedDir = `~/workspace/nemo/${serviceId}`.replace(/^~/, process.env.HOME || '');
  
  let cmd: string;
  if (host === 'localhost') {
    cmd = `docker rm -f ${containerName} 2>/dev/null || true && rm -rf ${expandedDir}`;
  } else {
    cmd = `ssh ${host} "docker rm -f ${containerName} 2>/dev/null || true && rm -rf ${expandedDir}\"`;
  }
  
  const { exec } = require('child_process');
  const { promisify } = require('util');
  const execAsync = promisify(exec);
  
  const result = await execAsync(cmd);
  if (result.exitCode !== 0) {
    throw new Error(`Failed to remove container: ${result.stderr}`);
  }
}

// Helper function to verify Consul config (shared between test files)
async function verifyConsulConfig(serviceId: string, expectedKeys: string[]): Promise<boolean> {
  try {
    const configs = await getAllConfigFromConsul(CONFIG.CONSUL_URL, [serviceId + '.>', 'nemo/' + serviceId + '/.>']);
    
    for (const expectedKey of expectedKeys) {
      // Check both old format (serviceId.key) and new format (nemo.serviceId.key)
      const keyExists = Object.keys(configs).some(key => 
        key === `${serviceId}.${expectedKey}` || 
        key === `nemo.${serviceId}.${expectedKey}`
      );
      
      if (!keyExists) {
        return false;
      }
    }
    
    return true;
  } catch (error) {
    console.error('Error verifying Consul config:', error);
    return false;
  }
}