import { describe, it, expect } from 'bun:test';
import { cleanupConsulForService, getContainerName } from '../helpers/consul';
import { containerExists, containerIsRunning, waitForContainerStatus, stopContainer, removeContainer } from '../helpers/docker';
import { get, post } from '../helpers/api';
import { generateTestServiceId } from '../helpers/consul';
import { CONFIG } from '../config';

const TARGET_HOST = CONFIG.DEFAULT_TARGET_HOST;
const TEST_TIMEOUT = 120000;

describe('Deploy Redis - Single Test', () => {
  it('should deploy Redis container successfully', async () => {
    const testServiceId = generateTestServiceId('redis');
    const containerName = getContainerName(testServiceId);

    const { data: catalogData } = await get('/api/catalog');
    const redisTemplate = catalogData.find((t: any) => t.id === 'redis');
    expect(redisTemplate).toBeDefined();

    console.log('Deploying service:', testServiceId);
    
    const { data: deployData, status } = await post('/api/deploy', {
      target_host: TARGET_HOST,
      service_id: testServiceId,
      template: redisTemplate,
      env_values: {},
      consul_url: CONFIG.CONSUL_URL,
      mode: 'deploy',
      deploy_path: CONFIG.DEPLOY_BASE_PATH
    });

    console.log('Deploy response:', status, deployData);
    
    expect(status).toBe(200);
    expect(deployData.success).toBe(true);

    const isReady = await waitForContainerStatus(containerName, 'running', TARGET_HOST, 60000);
    expect(isReady).toBe(true);

    const exists = await containerExists(containerName, TARGET_HOST);
    expect(exists).toBe(true);

    const running = await containerIsRunning(containerName, TARGET_HOST);
    expect(running).toBe(true);

    // Clean up
    await stopContainer(containerName, TARGET_HOST).catch(() => {});
    await removeContainer(containerName, testServiceId, TARGET_HOST).catch(() => {});
    await cleanupConsulForService(testServiceId);
  }, TEST_TIMEOUT);
});