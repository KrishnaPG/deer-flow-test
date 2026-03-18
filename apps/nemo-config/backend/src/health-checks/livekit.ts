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

export const checkLiveKit: HealthChecker = async (
  connectionUrl: string,
): Promise<HealthCheckResult> => {
  const { parseConnectionUrl } = await import('./types');
  const { host, port } = parseConnectionUrl(connectionUrl);
  
  try {
    const httpPort = port || 7880;
    const healthUrl = connectionUrl.replace(/^wss:\/\//, 'https://').replace(/^ws:\/\//, 'http://');
    const finalUrl = healthUrl.startsWith('http') ? `${healthUrl}/` : `http://${host}:${httpPort}/`;
    
    const { status, body } = await httpGet(finalUrl);
    
    return {
      success: status >= 200 && status < 500,
      message: status >= 200 && status < 500
        ? 'LiveKit endpointreachable'
        : `LiveKit returned status ${status}`,
      details: { status, body: body.slice(0, 500) }
    };
  } catch (err: any) {
    return {
      success: false,
      message: `LiveKit health check failed: ${err.message}`,
      details: { error: err.message }
    };
  }
};