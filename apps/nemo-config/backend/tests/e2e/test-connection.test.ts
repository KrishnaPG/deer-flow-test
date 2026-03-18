import { describe, it, expect, beforeAll, afterAll, beforeEach, afterEach } from 'bun:test';
import { cleanupTestResources, cleanupConsulForService, generateTestServiceId, getNextTestPort, getContainerName } from './helpers/consul';
import { stopContainer, removeContainer } from './helpers/docker';
import { get, post } from './helpers/api';
import { CONFIG } from './config';

const TARGET_HOST = CONFIG.DEFAULT_TARGET_HOST;

describe('Test Connection API - E2E Tests', () => {
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

  it('should test TCP connection to Redis successfully', async () => {
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

    await new Promise(resolve => setTimeout(resolve, 3000));

    const { data, status } = await post('/api/test-connection', {
      service_id: testServiceId,
      connection_url: `redis://${TARGET_HOST}:${testPort}`,
      health_check: { type: 'tcp', port: 6379 },
      metadata: {}
    });

    expect(status).toBe(200);
    expect(data.success).toBe(true);
  }, 120000);

  it('should fail on invalid TCP port', async () => {
    const { data, status } = await post('/api/test-connection', {
      service_id: 'redis-test',
      connection_url: `redis://${TARGET_HOST}:9999`,
      health_check: { type: 'tcp', port: 9999 },
      metadata: {}
    });

    expect(status).toBe(200);
    expect(data.success).toBe(false);
    expect(data.message).toContain('TCP connection failed');
    expect(data.details).toHaveProperty('host', TARGET_HOST);
    expect(data.details).toHaveProperty('port', 9999);
  });
});
