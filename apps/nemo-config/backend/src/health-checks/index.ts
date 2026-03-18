import type { HealthChecker } from './types';

type HealthCheckerFactory = () => Promise<HealthChecker>;

const registry: Map<string, HealthCheckerFactory> = new Map();

export function registerHealthChecker(serviceId: string, factory: HealthCheckerFactory) {
  registry.set(serviceId, factory);
}

export async function getHealthChecker(serviceId: string): Promise<HealthChecker | null> {
  const factory = registry.get(serviceId);
  if (!factory) return null;
  return factory();
}

export function hasHealthChecker(serviceId: string): boolean {
  return registry.has(serviceId);
}

export { parseConnectionUrl } from './types';
export type { HealthCheckResult, HealthChecker, ParsedConnection } from './types';

export async function loadHealthChecker(serviceId: string): Promise<HealthChecker | null> {
  switch (serviceId) {
    case 'redis':
      return (await import('./redis')).checkRedis;
    case 'postgres':
      return (await import('./postgres')).checkPostgres;
    case 'nats':
      return (await import('./nats')).checkNats;
    case 'clickhouse':
      return (await import('./clickhouse')).checkClickHouse;
    case 'minio':
      return (await import('./minio')).checkMinIO;
    case 'temporal':
      return (await import('./temporal')).checkTemporal;
    case 'livekit':
      return (await import('./livekit')).checkLiveKit;
    case 'signoz':
      return (await import('./signoz')).checkSignOz;
    default:
      return null;
  }
}