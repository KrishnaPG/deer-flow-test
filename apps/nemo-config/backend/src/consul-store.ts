import Consul from "consul";
import { CONSUL_PREFIX, ConsulKeys, ServiceConfigKeys } from "../../schema";

export interface ServiceMetadata {
  serviceId: string;
  containerName: string;
  managedBy: 'nemo' | 'external';
  host: string;
  connectionUrl: string;
  deployedAt: string;
  templateId: string;
}

export interface InstanceDetails {
  metadata: ServiceMetadata | null;
  connectionUrl: string | null;
  isHealthy: boolean;
  containerStatus?: 'running' | 'stopped' | 'not_found';
}

export type LogCallback = (msg: string) => void;

function getConsulClient(consulUrl: string): Consul {
  // consulUrl can be "10.7.0.4:8500" or "http://10.7.0.4:8500"
  let url = consulUrl;
  
  // Remove protocol if present for host extraction
  if (url.startsWith('http://')) {
    url = url.substring(7);
  } else if (url.startsWith('https://')) {
    url = url.substring(8);
  }
  
  const [host, portStr] = url.includes(':') 
    ? url.split(':') 
    : [url, '8500'];
  
  return new Consul({
    host: host || '10.7.0.4',
    port: parseInt(portStr || '8500', 10),
  }) as any;
}

function makeKey(serviceId: string, subKey: string): string {
  return `${CONSUL_PREFIX}/${serviceId}/${subKey}`;
}

function makeMetadataKey(serviceId: string): string {
  return `${CONSUL_PREFIX}/metadata/${serviceId}`;
}

export function getConfigKey(serviceId: string): string {
  return ServiceConfigKeys.serviceUrl(serviceId);
}

export function getMetadataKey(serviceId: string): string {
  return ServiceConfigKeys.serviceMetadata(serviceId);
}

export async function updateConsulKV(
  consulUrl: string,
  serviceId: string,
  connectionUrl: string,
  metadata: ServiceMetadata,
  exports: Record<string, string>,
  onLog?: LogCallback
) {
  const log = (msg: string) => {
    console.log(msg);
    if (onLog) onLog(msg);
  };
  
  log(`[Consul] Connecting to ${consulUrl}`);
  const consul = getConsulClient(consulUrl);

  try {
    log(`[Consul] Writing exports for ${serviceId}...`);
    for (const [key, value] of Object.entries(exports)) {
      try {
        // Convert dot notation to path: postgres.url -> nemo/postgres/url
        const consulKey = key.replace(/\./g, '/');
        const fullKey = consulKey.startsWith(CONSUL_PREFIX) 
          ? consulKey 
          : `${CONSUL_PREFIX}/${consulKey}`;
        
        await consul.kv.set(fullKey, value);
        log(`[Consul] Wrote export ${fullKey} = ${value}`);
      } catch (err: any) {
        log(`[Consul] Error writing export ${key}: ${err.message}`);
      }
    }

    // Write service URL
    const urlKey = makeKey(serviceId, 'url');
    log(`[Consul] Writing service URL: ${urlKey} = ${connectionUrl}`);
    await consul.kv.set(urlKey, connectionUrl);
    
    // Write metadata
    const metadataKey = makeMetadataKey(serviceId);
    log(`[Consul] Writing metadata: ${metadataKey}`);
    await consul.kv.set(metadataKey, JSON.stringify(metadata));
    
    log(`[Consul] Successfully stored config for ${serviceId}`);
  } catch (error: any) {
    log(`[Consul] Fatal error in updateConsulKV: ${error.message}`);
    throw error;
  }
}

function decodeConsulValue(value: string | Buffer): string {
  // Handle Buffer from consul client
  const strValue = typeof value === 'string' ? value : value.toString('utf-8');
  
  // Check if value looks like plain text (contains special chars like : / @ =)
  // If so, it's likely already decoded by the consul client
  if (/[:/@=]/.test(strValue) || strValue.startsWith('{') || strValue.startsWith('[')) {
    return strValue;
  }
  
  // Otherwise, try to base64 decode
  try {
    const decoded = Buffer.from(strValue, 'base64').toString('utf-8');
    // Verify it's valid UTF-8 (if it's gibberish, it wasn't base64)
    if (decoded && !decoded.includes('\ufffd')) {
      return decoded;
    }
  } catch {
    // Fall through to return original
  }
  
  return strValue;
}

export async function getAllConfigFromConsul(
  consulUrl: string, 
  allowedPrefixes?: string[]
): Promise<Record<string, string>> {
  console.log(`[Consul] Fetching config from ${consulUrl}`);
  const consul = getConsulClient(consulUrl);
  
  try {
    const configs: Record<string, string> = {};
    
    // If specific prefixes requested, iterate through each
    const prefixes = allowedPrefixes && allowedPrefixes.length > 0 
      ? allowedPrefixes 
      : [`${CONSUL_PREFIX}/`];
    
    console.log(`[Consul] Using prefixes:`, prefixes);
    
    for (const prefix of prefixes) {
      try {
        // Convert NATS-style prefix to Consul-style if needed
        // postgres.> -> nemo/postgres/
        let consulPrefix = prefix;
        if (prefix.includes('.') && !prefix.includes('/')) {
          // It's in NATS format, convert it
          consulPrefix = prefix
            .replace(/\./g, '/')
            .replace(/>/g, '');
          if (!consulPrefix.startsWith(CONSUL_PREFIX)) {
            consulPrefix = `${CONSUL_PREFIX}/${consulPrefix}`;
          }
        }
        
        // Check if this is a direct key (no wildcard) or directory query
        // A trailing / means it's a directory query for listing keys
        const isDirectKey = !prefix.includes('>') && !prefix.endsWith('.') && !prefix.endsWith('/');
        
        if (isDirectKey) {
          // Direct key lookup - don't add trailing slash
          try {
            const result = await consul.kv.get(consulPrefix);
            if (result && result.Value) {
              // Decode value - consul client may auto-decode some values
              const value = decodeConsulValue(result.Value);
              const dotKey = consulPrefix.replace(/\//g, '.');
              configs[dotKey] = value;
              console.log(`[Consul] Read direct key ${dotKey}`);
            }
          } catch (err: any) {
            if (err.message && !err.message.includes('not found')) {
              console.warn(`[Consul] Warning fetching direct key ${consulPrefix}:`, err.message);
            }
          }
        } else {
          // Directory query - add trailing slash
          if (!consulPrefix.endsWith('/')) {
            consulPrefix += '/';
          }
          
          const keys = await consul.kv.keys(consulPrefix);
          console.log(`[Consul] Found ${keys.length} keys under ${consulPrefix}`);
          
          for (const key of keys) {
            const result = await consul.kv.get(key);
            if (result && result.Value) {
              // Decode value - consul client may auto-decode some values
              const value = decodeConsulValue(result.Value);
              // Convert back to dot notation for API compatibility
              const dotKey = key.replace(/\//g, '.');
              configs[dotKey] = value;
              console.log(`[Consul] Read key ${dotKey}`);
            }
          }
        }
      } catch (err: any) {
        // If no keys match, Consul returns an empty array or error
        if (err.message && !err.message.includes('not found')) {
          console.warn(`[Consul] Warning fetching prefix ${prefix}:`, err.message);
        }
      }
    }
    
    return configs;
  } catch (error: any) {
    console.error(`[Consul] Error fetching config: ${error.message}`);
    throw error;
  }
}

export async function getInstanceDetailsFromConsul(
  serviceId: string, 
  consulUrl: string
): Promise<InstanceDetails> {
  const consul = getConsulClient(consulUrl);
  
  try {
    // Get connection URL
    const urlKey = makeKey(serviceId, 'url');
    let connectionUrl: string | null = null;
    
    try {
      const urlResult = await consul.kv.get(urlKey);
      if (urlResult && urlResult.Value) {
        connectionUrl = Buffer.from(urlResult.Value, 'base64').toString();
      }
    } catch (err) {
      // Key doesn't exist
    }
    
    // Get metadata
    const metadataKey = makeMetadataKey(serviceId);
    let metadata: ServiceMetadata | null = null;
    
    try {
      const metadataResult = await consul.kv.get(metadataKey);
      if (metadataResult && metadataResult.Value) {
        metadata = JSON.parse(Buffer.from(metadataResult.Value, 'base64').toString());
      }
    } catch (err) {
      // Key doesn't exist or parse error
    }
    
    const isHealthy = !!connectionUrl;
    
    return { metadata, connectionUrl, isHealthy, containerStatus: undefined };
  } catch (error: any) {
    console.error(`[Consul] Error getting instance details: ${error.message}`);
    throw error;
  }
}

export async function removeServiceConfig(
  serviceId: string, 
  consulUrl: string
): Promise<{ success: boolean; message: string }> {
  const consul = getConsulClient(consulUrl);
  
  try {
    // Get all keys under this service
    const servicePrefix = makeKey(serviceId, '');
    const keys = await consul.kv.keys(servicePrefix);
    
    // Also get metadata key
    const metadataKey = makeMetadataKey(serviceId);
    
    // Delete all keys
    for (const key of keys) {
      await consul.kv.del(key);
      console.log(`[Consul] Deleted key: ${key}`);
    }
    
    // Delete metadata if it exists
    try {
      await consul.kv.del(metadataKey);
      console.log(`[Consul] Deleted metadata: ${metadataKey}`);
    } catch (err) {
      // May not exist
    }
    
    return { success: true, message: `Removed configuration for ${serviceId}` };
  } catch (error: any) {
    console.error(`[Consul] Error removing config: ${error.message}`);
    throw error;
  }
}

export async function getServiceConfigs(
  serviceId: string,
  consulUrl: string
): Promise<Record<string, string>> {
  const configs = await getAllConfigFromConsul(consulUrl, [`${serviceId}.>`]);
  
  // Filter for this service
  const serviceConfigs: Record<string, string> = {};
  for (const [key, value] of Object.entries(configs)) {
    // Check if key matches nemo.<serviceId>.* pattern
    const parts = key.split('.');
    if (parts.length >= 2 && parts[1] === serviceId) {
      serviceConfigs[key] = value;
    }
  }
  
  return serviceConfigs;
}

// Export Consul health check
export async function checkConsulHealth(consulUrl: string): Promise<{ 
  status: 'healthy' | 'unhealthy'; 
  connected: boolean;
  url: string;
  timestamp: string;
  error?: string;
}> {
  try {
    const consul = getConsulClient(consulUrl);
    
    // Try to get leader status - simple health check
    await consul.status.leader();
    
    return {
      status: 'healthy',
      connected: true,
      url: consulUrl,
      timestamp: new Date().toISOString()
    };
  } catch (error: any) {
    return {
      status: 'unhealthy',
      connected: false,
      url: consulUrl,
      timestamp: new Date().toISOString(),
      error: error.message
    };
  }
}

// Get all services registered in Consul with health status
export async function getAllServicesHealth(consulUrl: string): Promise<Record<string, any>> {
  const consul = getConsulClient(consulUrl);
  
  try {
    // Get all service names under our prefix
    const allConfigs = await getAllConfigFromConsul(consulUrl);
    
    // Extract unique service IDs
    const serviceIds = new Set<string>();
    for (const key of Object.keys(allConfigs)) {
      const parts = key.split('.');
      if (parts.length >= 2 && parts[0] === 'nemo' && parts[1] !== 'metadata' && parts[1]) {
        serviceIds.add(parts[1]);
      }
    }
    
    // Get health for each service
    const health: Record<string, any> = {};
    for (const serviceId of serviceIds) {
      const details = await getInstanceDetailsFromConsul(serviceId, consulUrl);
      health[serviceId] = {
        serviceId,
        connectionUrl: details.connectionUrl,
        isHealthy: details.isHealthy,
        managedBy: details.metadata?.managedBy || 'unknown',
        host: details.metadata?.host || 'unknown'
      };
    }
    
    return health;
  } catch (error: any) {
    console.error(`[Consul] Error getting services health: ${error.message}`);
    throw error;
  }
}

// Default consul URL
export const DEFAULT_CONSUL_URL = '10.7.0.4:8500';
