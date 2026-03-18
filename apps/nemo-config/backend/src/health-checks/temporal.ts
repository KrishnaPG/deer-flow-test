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

export const checkTemporal: HealthChecker = async (
  connectionUrl: string,
): Promise<HealthCheckResult> => {
  const { parseConnectionUrl } = await import('./types');
  const { host, port } = parseConnectionUrl(connectionUrl);
  
  try {
    const grpcHealthUrl = `http://${host}:${port || 7233}/health`;
    const { status, body } = await httpGet(grpcHealthUrl);
    
    if (status === 200) {
      return {
        success: true,
        message: 'Temporal health check successful',
        details: { endpoint: '/health', status }
      };
    }
    
    return {
      success: false,
      message: `Temporal returned status ${status}`,
      details: { status, body: body.slice(0, 500) }
    };
  } catch (err: any) {
    try {
      const fallbackUrl = `http://${host}:${port || 7233}/`;
      await httpGet(fallbackUrl);
      return {
        success: true,
        message: 'Temporal port is reachable',
        details: { note: 'gRPC health endpoint not available, but port is open' }
      };
    } catch {
      return {
        success: false,
        message: `Temporal health check failed: ${err.message}`,
        details: { error: err.message }
      };
    }
  }
};