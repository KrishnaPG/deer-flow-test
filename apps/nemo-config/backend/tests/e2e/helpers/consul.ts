import { exec } from 'child_process';
import { promisify } from 'util';
import { CONFIG } from '../config';

const execAsync = promisify(exec);

let portCounter = 6379;

export function getNextTestPort(): number {
  portCounter++;
  if (portCounter > 9999) portCounter = 6380;
  return portCounter;
}

/**
 * Cleans up all test-related Consul keys and containers
 */
export async function cleanupTestResources(): Promise<void> {
  console.log('[Cleanup] Skipping bulk cleanup - tests clean up their own resources');
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
  const consulUrl = CONFIG.CONSUL_URL;
  try {
    const consul = new (await import('consul')).default({ host: consulUrl.split(':')[0], port: consulUrl.split(':')[1] || '8500' });
    await consul.kv.del({ key: `nemo/${serviceId}`, recurse: true });
    await consul.kv.del({ key: serviceId, recurse: true });
    console.log(`[Consul] Cleaned up config for ${serviceId}`);
  } catch (error) {
    console.log(`[Consul] Error removing config: ${error.message}`);
  }
}

/**
 * Gets all config from Consul for given prefixes
 */
export async function getAllConfigFromConsul(
  consulUrl: string, 
  allowedPrefixes?: string[]
): Promise<Record<string, string>> {
  const configs: Record<string, string> = {};
  
  try {
    const host = consulUrl.split(':')[0];
    const port = consulUrl.split(':')[1] || '8500';
    const consul = new (await import('consul')).default({ host, port });
    
    const prefixes = allowedPrefixes && allowedPrefixes.length > 0 
      ? allowedPrefixes 
      : ['nemo/'];
    
    for (const prefix of prefixes) {
      let consulPrefix = prefix;
      if (prefix.includes('.') && !prefix.includes('/')) {
        consulPrefix = prefix.replace(/\./g, '/');
        if (!consulPrefix.startsWith('nemo')) {
          consulPrefix = `nemo/${consulPrefix}`;
        }
      }
      
      if (!consulPrefix.endsWith('/')) {
        consulPrefix += '/';
      }
      
      try {
        const items = await consul.kv.get({ key: consulPrefix, recurse: true });
        if (items && Array.isArray(items)) {
          for (const item of items) {
            if (item.Key && item.Value) {
              const key = item.Key.replace(consulPrefix, '').replace('nemo/', '');
              configs[key] = Buffer.from(item.Value, 'base64').toString('utf-8');
            }
          }
        }
      } catch (e) {
        // Prefix might not exist
      }
    }
  } catch (error) {
    console.error('Error fetching Consul config:', error);
  }
  
  return configs;
}

/**
 * Gets the container name for a service ID
 */
export function getContainerName(serviceId: string): string {
  return `nemo-${serviceId}`;
}
