import { exec } from 'child_process';
import { promisify } from 'util';
import { updateConsulKV, getAllConfigFromConsul, removeServiceConfig } from '../../../src/consul-store';
import { CONFIG } from '../config';

const execAsync = promisify(exec);

/**
 * Executes a shell command and returns the result
 */
export async function runCommand(cmd: string): Promise<{ stdout: string; stderr: string; exitCode: number }> {
  try {
    const result = await execAsync(cmd);
    return { stdout: result.stdout.toString(), stderr: result.stderr.toString(), exitCode: 0 };
  } catch (error: any) {
    return { stdout: error.stdout?.toString() || '', stderr: error.stderr?.toString() || '', exitCode: 1 };
  }
}

/**
 * Cleans up all test-related Consul keys and containers
 */
export async function cleanupTestResources(): Promise<void> {
  // Cleanup Consul keys with our test prefix
  try {
    const consulConfig = await getAllConfigFromConsul(CONFIG.CONSUL_URL);
    const testKeys = Object.keys(consulConfig).filter(key => 
      key.startsWith(CONFIG.TEST_PREFIX)
    );
    
    for (const key of testKeys) {
      // Extract service ID from key format like "e2e-test-redis.url" or "nemo/e2e-test-redis/url"
      let serviceId = key;
      if (serviceId.startsWith('nemo/')) {
        serviceId = serviceId.split('/')[1];
      } else if (serviceId.startsWith('nemo.')) {
        serviceId = serviceId.split('.')[1];
      } else {
        // Direct format like "e2e-test-redis.url"
        serviceId = serviceId.split('.')[0];
      }
      
      await removeServiceConfig(serviceId, CONFIG.CONSUL_URL).catch(() => {});
    }
  } catch (error) {
    console.warn('Warning during Consul cleanup:', error);
  }
}

/**
 * Generates a unique test service ID
 */
export function generateTestServiceId(baseId: string): string {
  return `${CONFIG.TEST_PREFIX}${baseId}-${Date.now()}-${Math.floor(Math.random() * 1000)}`;
}

/**
 * Cleans up Consul keys for a specific service
 */
export async function cleanupConsulForService(serviceId: string): Promise<void> {
  await removeServiceConfig(serviceId, CONFIG.CONSUL_URL).catch(() => {});
}

/**
 * Verifies that a service configuration exists in Consul
 */
export async function verifyConsulConfig(serviceId: string, expectedKeys: string[]): Promise<boolean> {
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

/**
 * Gets the container name for a service ID
 */
export function getContainerName(serviceId: string): string {
  return `nemo-${serviceId}`;
}