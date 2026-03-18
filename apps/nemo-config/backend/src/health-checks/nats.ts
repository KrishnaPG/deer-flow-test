import type { HealthChecker, HealthCheckResult } from './types';
import { parseConnectionUrl } from './types';

export const checkNats: HealthChecker = async (
  connectionUrl: string,
): Promise<HealthCheckResult> => {
  const { connect } = await import('nats');
  
  let nc: any = null;
  try {
    nc = await connect({
      servers: connectionUrl,
      timeout: 5000
    });
    
    const info = nc.info;
    await nc.drain();
    
    return {
      success: true,
      message: 'NATS connection successful',
      details: {
        serverId: info?.server_id,
        version: info?.version,
        cluster: info?.cluster
      }
    };
  } catch (err: any) {
    return {
      success: false,
      message: `NATS connection failed: ${err.message}`,
      details: { error: err.message }
    };
  }
};