import type { HealthChecker, HealthCheckResult } from './types';

export const checkRedis: HealthChecker = async (connectionUrl: string): Promise<HealthCheckResult> => {
  const Ioredis = await import('ioredis').then(m => m.default || m);
  
  let client: any = null;
  try {
    client = new Ioredis(connectionUrl, {
      connectTimeout: 5000,
      maxRetriesPerRequest: 1,
      retryStrategy: () => null
    });
    
    const result = await new Promise<HealthCheckResult>((resolve) => {
      client.on('error', (err: Error) => {
        resolve({
          success: false,
          message: `Redis connection error: ${err.message}`,
          details: { error: err.message }
        });
      });
      
      client.ping()
        .then((response: string) => {
          if (response === 'PONG') {
            resolve({
              success: true,
              message: 'Redis PING successful',
              details: { response }
            });
          } else {
            resolve({
              success: false,
              message: `Unexpected Redis response: ${response}`,
              details: { response }
            });
          }
        })
        .catch((err: Error) => {
          resolve({
            success: false,
            message: `Redis PING failed: ${err.message}`,
            details: { error: err.message }
          });
        });
    });
    
    return result;
  } finally {
    if (client) {
      try {
        await client.quit();
      } catch {
        client.disconnect();
      }
    }
  }
};