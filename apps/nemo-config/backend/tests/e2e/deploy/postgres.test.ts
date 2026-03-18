import { describe, it, expect, beforeAll, afterAll, beforeEach, afterEach } from 'bun:test';
import { cleanupTestResources, cleanupConsulForService, getContainerName } from '../helpers/consul';
import { containerExists, containerIsRunning, getContainerLogs, waitForContainerStatus, stopContainer, startContainer, removeContainer } from '../helpers/docker';
import { get, post, del } from '../helpers/api';
import { generateTestServiceId } from '../helpers/consul';
import { CONFIG } from '../config';

describe('Deploy Postgres - E2E Tests', () => {
  let testServiceId: string;
  let containerName: string;

  beforeAll(async () => {
    await cleanupTestResources();
  });

  afterAll(async () => {
    await cleanupTestResources();
  });

  beforeEach(async () => {
    testServiceId = generateTestServiceId('postgres');
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

  it('should deploy Postgres container successfully', async () => {
    // Get catalog to get Postgres template
    const { data: catalogData } = await get('/api/catalog');
    const postgresTemplate = catalogData.find((t: any) => t.id === 'postgres');
    expect(postgresTemplate).toBeDefined();

    // Deploy Postgres with required environment values
    const { data: deployData, status } = await post('/api/deploy', {
      target_host: 'localhost',
      service_id: testServiceId,
      template: postgresTemplate,
      env_values: {
        POSTGRES_USER: 'testuser',
        POSTGRES_PASSWORD: 'testpass',
        POSTGRES_DB: 'testdb'
      },
      consul_url: CONFIG.CONSUL_URL,
      mode: 'deploy',
      deploy_path: CONFIG.DEPLOY_BASE_PATH
    });

    expect(status).toBe(200);
    expect(deployData.success).toBe(true);

    // Wait for container to be ready (Postgres might take a bit longer)
    const isReady = await waitForContainerStatus(containerName, 'running', 'localhost', 45000);
    expect(isReady).toBe(true);

    // Verify container exists and is running via Docker CLI
    const exists = await containerExists(containerName, 'localhost');
    expect(exists).toBe(true);

    const isRunning = await containerIsRunning(containerName, 'localhost');
    expect(isRunning).toBe(true);

    // Verify Consul has the configuration
    const consulVerified = await verifyConsulConfig(testServiceId, ['url', 'user', 'password', 'database']);
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
    expect(details.connectionUrl).toContain('localhost:5432');
    
    // Verify exports are correctly stored
    expect(details.metadata?.connectionUrl).toContain('testuser');
    expect(details.metadata?.connectionUrl).toContain('testpass');
    expect(details.metadata?.connectionUrl).toContain('testdb');
  });

  it('should export correct Postgres connection details', async () => {
    // Get catalog to get Postgres template
    const { data: catalogData } = await get('/api/catalog');
    const postgresTemplate = catalogData.find((t: any) => t.id === 'postgres');
    expect(postgresTemplate).toBeDefined();

    // Deploy Postgres
    const { data: deployData } = await post('/api/deploy', {
      target_host: 'localhost',
      service_id: testServiceId,
      template: postgresTemplate,
      env_values: {
        POSTGRES_USER: 'exportuser',
        POSTGRES_PASSWORD: 'exportpass',
        POSTGRES_DB: 'exportdb'
      },
      consul_url: CONFIG.CONSUL_URL,
      mode: 'deploy',
      deploy_path: CONFIG.DEPLOY_BASE_PATH
    });

    expect(deployData.success).toBe(true);

    // Wait for container to be ready
    await waitForContainerStatus(containerName, 'running', 'localhost', 45000);

    // Verify Consul has the configuration with correct exports
    const configs = await getAllConfigFromConsul(CONFIG.CONSUL_URL, [testServiceId + '.>', 'nemo/' + testServiceId + '/.>']);
    
    // Check that exports are stored correctly
    expect(configs).toHaveProperty(`${testServiceId}.url`);
    expect(configs[`${testServiceId}.url`]).toContain('exportuser');
    expect(configs[`${testServiceId}.url`]).toContain('exportpass');
    expect(configs[`${testServiceId}.url`]).toContain('exportdb');
    
    expect(configs).toHaveProperty(`${testServiceId}.user`);
    expect(configs[`${testServiceId}.user`]).toBe('exportuser');
    
    expect(configs).toHaveProperty(`${testServiceId}.password`);
    expect(configs[`${testServiceId}.password`]).toBe('exportpass');
    
    expect(configs).toHaveProperty(`${testServiceId}.database`);
    expect(configs[`${testServiceId}.database`]).toBe('exportdb');
  });
});

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