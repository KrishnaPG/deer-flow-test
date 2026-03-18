import type { HealthChecker, HealthCheckResult, ParsedConnection } from './types';
import { parseConnectionUrl } from './types';

export const checkPostgres: HealthChecker = async (
  connectionUrl: string,
): Promise<HealthCheckResult> => {
  const pg = await import('pg');
  const { Client } = pg;
  
  let client: InstanceType<typeof pg.Client> | null = null;
  try {
    client = new Client({ connectionString: connectionUrl, connectionTimeoutMillis: 5000 });
    
    await client.connect();
    const result = await client.query('SELECT 1');
    
    if (result.rows && result.rows.length > 0) {
      return {
        success: true,
        message: 'PostgreSQL connection successful',
        details: { query: 'SELECT 1', rowCount: result.rowCount }
      };
    }
    
    return {
      success: false,
      message: 'PostgreSQL query returned no results',
      details: { query: 'SELECT 1' }
    };
  } catch (err: any) {
    return {
      success: false,
      message: `PostgreSQL connection failed: ${err.message}`,
      details: { error: err.message, code: err.code }
    };
  } finally {
    if (client) {
      try {
        await client.end();
      } catch {/*ignore*/
      }
    }
  }
};