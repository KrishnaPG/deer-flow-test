import { describe, it, expect, beforeAll, afterAll, beforeEach, afterEach } from 'bun:test';
import { cleanupTestResources, cleanupConsulForService, getAllConfigFromConsul } from './helpers/consul';
import { get, post } from './helpers/api';
import { generateTestServiceId } from './helpers/consul';
import { CONFIG } from './config';

const TARGET_HOST = CONFIG.DEFAULT_TARGET_HOST;

describe('Register Existing Instance - E2E Tests', () => {
  let testServiceId: string;

  beforeAll(async () => {
    await cleanupTestResources();
  });

  afterAll(async () => {
    await cleanupTestResources();
  });

  beforeEach(async () => {
    testServiceId = generateTestServiceId('test-service');
  });

  afterEach(async () => {
    await cleanupConsulForService(testServiceId);
  });

  it('should register an existing Postgres service and store data in Consul', async () => {
    const { data: catalogData } = await get('/api/catalog');
    const postgresTemplate = catalogData.find((t: any) => t.id === 'postgres');
    expect(postgresTemplate).toBeDefined();

    const { data: registerData, status } = await post('/api/register-existing', {
      service_id: testServiceId,
      connection_url: 'postgres://testuser:testpass@10.7.0.4:5432/testdb',
      consul_url: CONFIG.CONSUL_URL,
      template: postgresTemplate,
      env_values: {
        POSTGRES_USER: 'testuser',
        POSTGRES_PASSWORD: 'testpass',
        POSTGRES_DB: 'testdb'
      }
    });

    expect(status).toBe(200);
    expect(registerData.success).toBe(true);

    await new Promise(resolve => setTimeout(resolve, 2000));

    const consulVerified = await verifyConsulConfig(testServiceId, ['url', 'user', 'password', 'database'], postgresTemplate.id);
    expect(consulVerified).toBe(true);

    const { data: details, status: detailsStatus } = await get(
      `/api/services/${testServiceId}/details?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}`
    );
    expect(detailsStatus).toBe(200);
    expect(details.isHealthy).toBe(true);
    expect(details.metadata?.managedBy).toBe('external');
    expect(details.metadata?.host).toBe('external');
    expect(details.containerStatus).toBeUndefined();
    expect(details.connectionUrl).toBe('postgres://testuser:testpass@10.7.0.4:5432/testdb');
    
    const configs = await getAllConfigFromConsul(CONFIG.CONSUL_URL, [`nemo/${postgresTemplate.id}/.>`]);
    expect(configs).toHaveProperty(`nemo.${postgresTemplate.id}.url`);
    expect(configs[`nemo.${postgresTemplate.id}.url`]).toBe('postgres://testuser:testpass@10.7.0.4:5432/testdb');
    expect(configs).toHaveProperty(`nemo.${postgresTemplate.id}.user`);
    expect(configs[`nemo.${postgresTemplate.id}.user`]).toBe('testuser');
    expect(configs).toHaveProperty(`nemo.${postgresTemplate.id}.password`);
    expect(configs[`nemo.${postgresTemplate.id}.password`]).toBe('testpass');
    expect(configs).toHaveProperty(`nemo.${postgresTemplate.id}.database`);
    expect(configs[`nemo.${postgresTemplate.id}.database`]).toBe('testdb');
  });

  it('should register an existing Redis service', async () => {
    const { data: catalogData } = await get('/api/catalog');
    const redisTemplate = catalogData.find((t: any) => t.id === 'redis');
    expect(redisTemplate).toBeDefined();

    const { data: registerData, status } = await post('/api/register-existing', {
      service_id: testServiceId,
      connection_url: `redis://${TARGET_HOST}:6379`,
      consul_url: CONFIG.CONSUL_URL,
      template: redisTemplate,
      env_values: {}
    });

    expect(status).toBe(200);
    expect(registerData.success).toBe(true);

    await new Promise(resolve => setTimeout(resolve, 2000));

    const consulVerified = await verifyConsulConfig(testServiceId, ['url'], redisTemplate.id);
    expect(consulVerified).toBe(true);

    const { data: details, status: detailsStatus } = await get(
      `/api/services/${testServiceId}/details?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}`
    );
    expect(detailsStatus).toBe(200);
    expect(details.isHealthy).toBe(true);
    expect(details.metadata?.managedBy).toBe('external');
    expect(details.metadata?.host).toBe('external');
    expect(details.containerStatus).toBeUndefined();
    expect(details.connectionUrl).toBe(`redis://${TARGET_HOST}:6379`);
  });
});

async function verifyConsulConfig(serviceId: string, expectedKeys: string[], templateId: string): Promise<boolean> {
  try {
    const configs = await getAllConfigFromConsul(CONFIG.CONSUL_URL, [`nemo/${templateId}/.>`]);
    
    for (const expectedKey of expectedKeys) {
      const dotKey = `nemo.${templateId}.${expectedKey}`;
      if (!configs[dotKey]) {
        console.log(`Key not found: ${dotKey}`);
        return false;
      }
    }
    
    return true;
  } catch (error) {
    console.error('Error verifying Consul config:', error);
    return false;
  }
}
