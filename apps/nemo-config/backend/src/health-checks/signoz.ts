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

export const checkSignOz: HealthChecker = async (
  connectionUrl: string,
): Promise<HealthCheckResult> => {
  const { parseConnectionUrl } = await import('./types');
  const { host, port } = parseConnectionUrl(connectionUrl);
  
  try {
    const healthUrl = `http://${host}:${port || 3301}/api/v1/health`;
    const { status, body } = await httpGet(healthUrl);
    
    if (status === 200) {
      return {
        success: true,
        message: 'SignOz health check successful',
        details: { endpoint: '/api/v1/health', status }
      };
    }
    
    return {
      success: false,
      message: `SignOz returned status ${status}`,
      details: { status, body: body.slice(0, 500) }
    };
  } catch (err: any) {
    return {
      success: false,
      message: `SignOz health check failed: ${err.message}`,
      details: { error: err.message }
    };
  }
};