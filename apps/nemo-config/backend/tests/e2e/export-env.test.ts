import { describe, it, expect, beforeAll, afterAll, beforeEach, afterEach } from 'bun:test';
import { cleanupTestResources, cleanupConsulForService, generateTestServiceId, getNextTestPort, getContainerName } from './helpers/consul';
import { stopContainer, removeContainer } from './helpers/docker';
import { get, post } from './helpers/api';
import { CONFIG } from './config';

const TARGET_HOST = CONFIG.DEFAULT_TARGET_HOST;

describe('Export Env API - E2E Tests', () => {
  let testServiceId: string;
  let containerName: string;
  let testPort: number;

  beforeAll(async () => {
    await cleanupTestResources();
  });

  afterAll(async () => {
    await cleanupTestResources();
  });

  beforeEach(async () => {
    testServiceId = generateTestServiceId('test-service');
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

  it('should export configurations as .env format', async () => {
    const catalogResponse = await get('/api/catalog');
    const redisTemplate = catalogResponse.data.find((t: any) => t.id === 'redis');
    
    const deployResponse = await post('/api/deploy', {
      target_host: TARGET_HOST,
      service_id: testServiceId,
      template: redisTemplate,
      env_values: { PORT: String(testPort) },
      consul_url: CONFIG.CONSUL_URL,
      mode: 'deploy',
      deploy_path: CONFIG.DEPLOY_BASE_PATH
    });
    
    expect(deployResponse.status).toBe(200);
    expect(deployResponse.data.success).toBe(true);
    
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    const { data: exportData, status } = await get(
      `/api/export-env?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}`
    );
    
    expect(status).toBe(200);
    expect(exportData.success).toBe(true);
    expect(typeof exportData.content).toBe('string');
    
    // Should contain our test service's configuration in .env format
    expect(exportData.content).toContain('NEMO_');
    expect(exportData.content).toContain('URL=');
    
    // Should not contain internal metadata keys
    expect(exportData.content).not.toContain('METADATA');
  });

  it('should handle empty Consul gracefully', async () => {
    await cleanupTestResources();
    
    const { data: exportData, status } = await get(
      `/api/export-env?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}`
    );
    
    expect(status).toBe(200);
    expect(exportData.success).toBe(true);
    expect(typeof exportData.content).toBe('string');
    expect(exportData.content).toBeTruthy();
  });
});