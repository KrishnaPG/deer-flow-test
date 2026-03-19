/**
 * End-to-End Functional Test Suite for nemo-config backend
 * 
 * This test suite validates all backend API routes against a real Consul server (10.7.0.4:8500)
 * and verifies docker deployments and container management by checking actual docker command outputs.
 * 
 * Test Suite Structure:
 * - Catalog API: GET /api/catalog
 * - SSH Hosts API: GET /api/ssh-hosts
 * - Consul Health API: GET /api/health/consul
 * - Services Health API: GET /api/health/services
 * - Configs API: GET /api/configs, GET /api/config/:serviceId
 * - Export Env API: GET /api/export-env
 * - Deploy API: POST /api/deploy (all 8 templates)
 * - Register Existing API: POST /api/register-existing
 * - Test Connection API: POST /api/test-connection
 * - Container Operations:
 *   - Details: GET /api/services/:serviceId/details
 *   - Logs: GET /api/services/:serviceId/logs
 *   - Stop: POST /api/services/:serviceId/stop
 *   - Start: POST /api/services/:serviceId/start
 *   - Restart: POST /api/services/:serviceId/restart
 *   - Delete: DELETE /api/services/:serviceId/container
 *   - Delete Config: DELETE /api/services/:serviceId/config
 */

// Import all test files to ensure they are executed
// This is the entry point for the test suite

import './catalog.test.ts';
import './ssh-hosts.test.ts';
import './consul-health.test.ts';
import './configs.test.ts';
import './export-env.test.ts';
import './services-health.test.ts';
import './deploy/redis.test.ts';
import './deploy/postgres.test.ts';
import './deploy/nats.test.ts';
import './deploy/minio.test.ts';
import './deploy/clickhouse.test.ts';
import './deploy/temporal.test.ts';
import './deploy/livekit.test.ts';
import './deploy/signoz.test.ts';
import './register-existing.test.ts';
import './test-connection.test.ts';
import './container-ops/details.test.ts';
import './container-ops/logs.test.ts';
import './container-ops/stop.test.ts';
import './container-ops/start.test.ts';
import './container-ops/restart.test.ts';
import './container-ops/delete.test.ts';

async function waitForBackend(maxRetries = 30): Promise<void> {
  for (let i = 0; i < maxRetries; i++) {
    try {
      const response = await fetch('http://localhost:3001/api/catalog');
      if (response.ok) {
        console.log('Backend is ready');
        return;
      }
    } catch {}
    await new Promise(resolve => setTimeout(resolve, 1000));
  }
  throw new Error('Backend failed to start');
}

waitForBackend().catch(err => {
  console.error('Failed to start backend:', err.message);
  process.exit(1);
});

console.log('E2E Test Suite Loaded - All tests will run when bun test is executed');