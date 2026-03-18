import { describe, it, expect, beforeAll, afterAll, beforeEach, afterEach } from 'bun:test';
import { cleanupTestResources, cleanupConsulForService } from '../../helpers/consul';
import { get, post } from '../../helpers/api';
import { generateTestServiceId } from '../../helpers/consul';
import { CONFIG } from '../../config';

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
    // Get catalog to get Postgres template
    const { data: catalogData } = await get('/api/catalog');
    const postgresTemplate = catalogData.find((t: any) => t.id === 'postgres');
    expect(postgresTemplate).toBeDefined();

    // Register existing Postgres service
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

    // Wait a moment for Consul to be updated
    await new Promise(resolve => setTimeout(resolve, 2000));

    // Verify Consul has the configuration
    const consulVerified = await verifyConsulConfig(testServiceId, ['url', 'user', 'password', 'database']);
    expect(consulVerified).toBe(true);

    // Verify we can get service details via API
    const { data: details, status: detailsStatus } = await get(
      `/api/services/${testServiceId}/details?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}`
    );
    expect(detailsStatus).toBe(200);
    expect(details.isHealthy).toBe(true);
    expect(details.metadata?.managedBy).toBe('external'); // Should be external since we registered existing
    expect(details.metadata?.host).toBe('external');
    expect(details.containerStatus).toBeUndefined(); // No container for external services
    expect(details.connectionUrl).toBe('postgres://testuser:testpass@10.7.0.4:5432/testdb');
    
    // Verify exports are correctly stored
    const configs = await getAllConfigFromConsul(CONFIG.CONSUL_URL, [testServiceId + '.>', 'nemo/' + testServiceId + '/.>']);
    expect(configs).toHaveProperty(`${testServiceId}.url`);
    expect(configs[`${testServiceId}.url`]).toBe('postgres://testuser:testpass@10.7.0.4:5432/testdb');
    expect(configs).toHaveProperty(`${testServiceId}.user`);
    expect(configs[`${testServiceId}.user`]).toBe('testuser');
    expect(configs).toHaveProperty(`${testServiceId}.password`);
    expect(configs[`${testServiceId}.password`]).toBe('testpass');
    expect(configs).toHaveProperty(`${testServiceId}.database`);
    expect(configs[`${testServiceId}.database`]).toBe('testdb');
  });

  it('should register an existing Redis service', async () => {
    // Get catalog to get Redis template
    const { data: catalogData } = await get('/api/catalog');
    const redisTemplate = catalogData.find((t: any) => t.id === 'redis');
    expect(redisTemplate).toBeDefined();

    // Register existing Redis service
    const { data: registerData, status } = await post('/api/register-existing', {
      service_id: testServiceId,
      connection_url: 'redis://10.7.0.4:6379',
      consul_url: CONFIG.CONSUL_URL,
      template: redisTemplate,
      env_values: {}
    });

    expect(status).toBe(200);
    expect(registerData.success).toBe(true);

    // Wait a moment for Consul to be updated
    await new Promise(resolve => setTimeout(resolve, 2000));

    // Verify Consul has the configuration
    const consulVerified = await verifyConsulConfig(testServiceId, ['url']);
    expect(consulVerified).toBe(true);

    // Verify we can get service details via API
    const { data: details, status: detailsStatus } = await get(
      `/api/services/${testServiceId}/details?consul_url=${encodeURIComponent(CONFIG.CONSUL_URL)}`
    );
    expect(detailsStatus).toBe(200);
    expect(details.isHealthy).toBe(true);
    expect(details.metadata?.managedBy).toBe('external');
    expect(details.metadata?.host).toBe('external');
    expect(details.containerStatus).toBeUndefined();
    expect(details.connectionUrl).toBe('redis://10.7.0.4:6379');
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