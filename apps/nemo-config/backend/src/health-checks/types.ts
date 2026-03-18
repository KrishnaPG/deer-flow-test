export interface HealthCheckResult {
  success: boolean;
  message: string;
  details?: Record<string, any>;
}

export interface HealthChecker {
  (connectionUrl: string, metadata?: Record<string, string>): Promise<HealthCheckResult>;
}

export interface ParsedConnection {
  host: string;
  port: number;
  username?: string;
  password?: string;
  database?: string;
  pathname?: string;
}

export function parseConnectionUrl(url: string): ParsedConnection {
  try {
    const parsed = new URL(url);
    return {
      host: parsed.hostname,
      port: parsed.port ? parseInt(parsed.port, 10) : getDefaultPort(parsed.protocol),
      username: parsed.username || undefined,
      password: parsed.password || undefined,
      database: parsed.pathname.slice(1) || undefined,
      pathname: parsed.pathname
    };
  } catch {
    const match = url.match(/^([^:]+):(\d+)$/);
    if (match && match[1] && match[2]) {
      return { host: match[1], port: parseInt(match[2], 10) };
    }
    throw new Error(`Could not parse connection URL: ${url}`);
  }
}

function getDefaultPort(protocol: string): number {
  const ports: Record<string, number> = {
    'redis:': 6379,
    'rediss:': 6379,
    'postgres:': 5432,
    'postgresql:': 5432,
    'nats:': 4222,
    'http:': 80,
    'https:': 443
  };
  return ports[protocol] || 0;
}