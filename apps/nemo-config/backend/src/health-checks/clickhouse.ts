import type { HealthChecker, HealthCheckResult } from './types';

async function httpGet(url: string, timeout: number = 5000): Promise<{ status: number; body: string }> {
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), timeout);
  
  try {
    const response = await fetch(url, {
      method: 'GET',
      signal: controller.signal
    });
    clearTimeout(timeoutId);
    
    const body = await response.text();
    return { status: response.status, body };
  } finally {
    clearTimeout(timeoutId);
  }
}

export const checkClickHouse: HealthChecker = async (
  connectionUrl: string,
): Promise<HealthCheckResult> => {
  const { parseConnectionUrl } = await import('./types');
  const { host, port } = parseConnectionUrl(connectionUrl);
  
  try {
    const healthUrl = `http://${host}:${port || 8123}/ping`;
    const { status, body } = await httpGet(healthUrl);
    
    if (status === 200) {
      return {
        success: true,
        message: 'ClickHouse health check successful',
        details: { endpoint: '/ping', status }
      };
    }
    
    return {
      success: false,
      message: `ClickHouse returned status ${status}`,
      details: { status, body: body.slice(0, 500) }
    };
  } catch (err: any) {
    return {
      success: false,
      message: `ClickHouse health check failed: ${err.message}`,
      details: { error: err.message }
    };
  }
};