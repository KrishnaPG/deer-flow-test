import { describe, it, expect, beforeAll, afterAll, beforeEach, afterEach } from 'bun:test';
import { cleanupTestResources, cleanupConsulForService, getContainerName, getNextTestPort, getAllConfigFromConsul, generateTestServiceId } from '../helpers/consul';
import { containerExists, containerIsRunning, stopContainer } from '../helpers/docker';
import { get, post, del } from '../helpers/api';
import { CONFIG } from '../config';

const TARGET_HOST = CONFIG.DEFAULT_TARGET_HOST;

describe('Container Delete API - E2E Tests', () => {
  let testServiceId: string;
  let containerName: string;
  let testPort: number;

  beforeAll(async () => {
    await cleanupTestResources();
  }, 60000);

  afterAll(async () => {
    await cleanupTestResources();
  }, 60000);

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

  it('should delete container and remove config for nemo-managed service', async () => {
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

    const isReady = await waitForContainerStatus(containerName, 'running', TARGET_HOST, 60000);
    expect(isReady).toBe(true);

    let exists = await containerExists(containerName, TARGET_HOST);
    expect(exists).toBe(true);

    let isRunning = await containerIsRunning(containerName, TARGET_HOST);
    expect(isRunning).toBe(true);

    const consulVerifiedBefore = await verifyConsulConfig(testServiceId, ['url']);
    expect(consulVerifiedBefore).toBe(true);

    const { data: deleteData, status } = await del(
      `/api/services/${testServiceId}/container?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}&deploy_path=${encodeURIComponent(CONFIG.DEPLOY_BASE_PATH)}`
    );
    expect(status).toBe(200);
    expect(deleteData.success).toBe(true);

    await new Promise(resolve => setTimeout(resolve, 2000));

    exists = await containerExists(containerName, TARGET_HOST);
    expect(exists).toBe(false);

    const consulVerifiedAfter = await verifyConsulConfig(testServiceId, ['url']);
    expect(consulVerifiedAfter).toBe(false);

    const { data: details, status: detailsStatus } = await get(
      `/api/services/${testServiceId}/details?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}`
    );
    expect(detailsStatus).toBe(200);
    expect(details.metadata).toBeNull();
    expect(details.connectionUrl).toBeNull();
    expect(details.isHealthy).toBe(false);
  }, 120000);

  it('should remove config only for external (registered existing) service', async () => {
    const { data: catalogData } = await get('/api/catalog');
    const redisTemplate = catalogData.find((t: any) => t.id === 'redis');
    expect(redisTemplate).toBeDefined();

    const { data: registerData } = await post('/api/register-existing', {
      service_id: testServiceId,
      connection_url: `redis://${TARGET_HOST}:6379`,
      consul_url: CONFIG.CONSUL_URL,
      template: redisTemplate,
      env_values: {}
    });

    expect(registerData.success).toBe(true);

    await new Promise(resolve => setTimeout(resolve, 2000));

    const consulVerifiedBefore = await verifyConsulConfig(testServiceId, ['url']);
    expect(consulVerifiedBefore).toBe(true);

    const { data: deleteData, status } = await del(
      `/api/services/${testServiceId}/config?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}`
    );
    expect(status).toBe(200);
    expect(deleteData.success).toBe(true);

    await new Promise(resolve => setTimeout(resolve, 2000));

    const consulVerifiedAfter = await verifyConsulConfig(testServiceId, ['url']);
    expect(consulVerifiedAfter).toBe(false);
  });
});

async function waitForContainerStatus(
  containerName: string,
  expectedStatus: 'running' | 'stopped' | 'exited',
  host: string,
  timeoutMs: number = 60000
): Promise<boolean> {
  const startTime = Date.now();

  while (Date.now() - startTime < timeoutMs) {
    let isRunning = false;
    try {
      isRunning = await containerIsRunning(containerName, host);
    } catch (error) {
    }

    const isExited = !isRunning && await containerExists(containerName, host);

    if (
      (expectedStatus === 'running' && isRunning) ||
      (expectedStatus === 'stopped' && !isRunning && await containerExists(containerName, host)) ||
      (expectedStatus === 'exited' && isExited)
    ) {
      return true;
    }

    await new Promise(resolve => setTimeout(resolve, 500));
  }

  return false;
}

async function removeContainer(containerName: string, serviceId: string, host: string = 'localhost'): Promise<void> {
  const expandedDir = `~/workspace/nemo/${serviceId}`.replace(/^~/, process.env.HOME || '');

  let cmd: string;
  if (host === 'localhost') {
    cmd = `docker rm -f ${containerName} 2>/dev/null || true && rm -rf ${expandedDir}`;
  } else {
    cmd = `ssh ${host} "docker rm -f ${containerName} 2>/dev/null || true && rm -rf ${expandedDir}"`;
  }

  const { exec } = require('child_process');
  const { promisify } = require('util');
  const execAsync = promisify(exec);

  const result = await execAsync(cmd);
  if (result.exitCode !== 0) {
    throw new Error(`Failed to remove container: ${result.stderr}`);
  }
}

async function verifyConsulConfig(serviceId: string, expectedKeys: string[]): Promise<boolean> {
  try {
    const configs = await getAllConfigFromConsul(CONFIG.CONSUL_URL, [serviceId + '.>', 'nemo/' + serviceId + '/.>']);

    for (const expectedKey of expectedKeys) {
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
