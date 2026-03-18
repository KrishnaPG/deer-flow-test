import { describe, it, expect, beforeAll, afterAll } from 'bun:test';
import { $ } from 'bun';
import { connect, StringCodec } from 'nats';

const API_URL = 'http://localhost:3001/api';
const NATS_URL = 'nats://10.7.0.4:4222';
const DEPLOY_PATH = '/tmp/nemo-test';
const sc = StringCodec();

async function exec(cmd: string): Promise<{ stdout: string; stderr: string; exitCode: number }> {
  try {
    const result = await $`${{raw: cmd}}`.quiet();
    return { stdout: result.stdout.toString(), stderr: result.stderr.toString(), exitCode: result.exitCode };
  } catch (e: any) {
    return { stdout: e.stdout?.toString() || '', stderr: e.stderr?.toString() || '', exitCode: 1 };
  }
}

async function getConfigs(): Promise<Record<string, string>> {
  const res = await fetch(`${API_URL}/configs?nats_url=${encodeURIComponent(NATS_URL)}`);
  return await res.json();
}

async function containerExists(name: string, host: string = 'localhost'): Promise<boolean> {
  const cmd = host === 'localhost' 
    ? `docker ps -a --filter name=${name} --format '{{.Names}}'`
    : `ssh ${host} "docker ps -a --filter name=${name} --format '{{.Names}}'"`;
  const result = await exec(cmd);
  return result.stdout.trim() === name;
}

async function containerIsRunning(name: string, host: string = 'localhost'): Promise<boolean> {
  const cmd = host === 'localhost'
    ? `docker ps --filter name=${name} --filter status=running --format '{{.Names}}'`
    : `ssh ${host} "docker ps --filter name=${name} --filter status=running --format '{{.Names}}'"`;
  const result = await exec(cmd);
  return result.stdout.trim() === name;
}

async function getContainerLogs(name: string, host: string = 'localhost', tail: number = 10): Promise<string[]> {
  const cmd = host === 'localhost'
    ? `docker logs --tail ${tail} ${name} 2>&1`
    : `ssh ${host} "docker logs --tail ${tail} ${name} 2>&1"`;
  const result = await exec(cmd);
  return result.stdout.split('\n').filter(l => l.trim());
}

async function cleanupContainer(name: string, host: string = 'localhost'): Promise<void> {
  const cmd = host === 'localhost'
    ? `docker rm -f ${name} 2>/dev/null || true`
    : `ssh ${host} "docker rm -f ${name} 2>/dev/null || true"`;
  await exec(cmd);
}

describe('Nemo Config Backend API', () => {
  let nc: any;

  beforeAll(async () => {
    nc = await connect({ servers: NATS_URL });
    await exec(`rm -rf ${DEPLOY_PATH}`);
    await exec(`mkdir -p ${DEPLOY_PATH}`);
  });

  afterAll(async () => {
    await nc?.drain();
    await exec(`rm -rf ${DEPLOY_PATH}`);
  });

  describe('Catalog API', () => {
    it('should return all service templates with required fields', async () => {
      const res = await fetch(`${API_URL}/catalog`);
      expect(res.status).toBe(200);
      const templates = await res.json();
      expect(Array.isArray(templates)).toBe(true);
      
      for (const t of templates) {
        expect(t).toHaveProperty('id');
        expect(t).toHaveProperty('name');
        expect(t).toHaveProperty('docker_compose');
        expect(t).toHaveProperty('health_check');
      }
      console.log('Templates:', templates.map((t: any) => t.id).join(', '));
    });

    it('should have correct exports in postgres template', async () => {
      const res = await fetch(`${API_URL}/catalog`);
      const templates = await res.json();
      const postgres = templates.find((t: any) => t.id === 'postgres');
      
      expect(postgres).toBeDefined();
      expect(postgres.exports).toHaveProperty('postgres.url');
      expect(postgres.exports).toHaveProperty('postgres.user');
    });
  });

  describe('SSH Hosts API', () => {
    it('should return localhost in hosts list', async () => {
      const res = await fetch(`${API_URL}/ssh-hosts`);
      expect(res.status).toBe(200);
      const hosts = await res.json();
      expect(hosts).toContain('localhost');
    });
  });

  describe('Health Check API', () => {
    it('should test TCP connection successfully', async () => {
      const res = await fetch(`${API_URL}/test-connection`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          service_id: 'redis',
          connection_url: 'redis://10.7.0.4:6379',
          health_check: { type: 'tcp', port: 6379 },
          metadata: {}
        })
      });
      expect(res.status).toBe(200);
      const result = await res.json();
      console.log('TCP health check:', result);
      expect(result).toHaveProperty('success');
    }, 15000);

    it('should fail on invalid port', async () => {
      const res = await fetch(`${API_URL}/test-connection`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          service_id: 'redis',
          connection_url: 'redis://10.7.0.4:9999',
          health_check: { type: 'tcp', port: 9999 },
          metadata: {}
        })
      });
      const result = await res.json();
      expect(result.success).toBe(false);
    }, 15000);
  });

  describe('Register Existing Instance', () => {
    it('should register postgres and store ALL data in NATS', async () => {
      const catalogRes = await fetch(`${API_URL}/catalog`);
      const templates = await catalogRes.json();
      const postgresTemplate = templates.find((t: any) => t.id === 'postgres');

      const res = await fetch(`${API_URL}/register-existing`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          service_id: 'postgres',
          connection_url: 'postgres://admin:testpass@10.7.0.4:5432/testdb',
          nats_url: NATS_URL,
          template: postgresTemplate,
          env_values: { POSTGRES_USER: 'admin', POSTGRES_PASSWORD: 'testpass', POSTGRES_DB: 'testdb' }
        })
      });

      if (res.status !== 200) {
        const err = await res.json();
        console.error('Registration failed:', JSON.stringify(err, null, 2));
      }
      expect(res.status).toBe(200);
      const result = await res.json();
      expect(result.success).toBe(true);

      const configs = await getConfigs();
      console.log('Configs after registration:', JSON.stringify(configs, null, 2));

      // CRITICAL CHECKS
      expect(configs).toHaveProperty('postgres.url');
      expect(configs['postgres.url']).toBe('postgres://admin:testpass@10.7.0.4:5432/testdb');
      
      expect(configs).toHaveProperty('postgres.user');
      expect(configs['postgres.user']).toBe('admin');
      
      expect(configs).toHaveProperty('nemo_metadata.postgres');
      const metadata = JSON.parse(configs['nemo_metadata.postgres']);
      expect(metadata.managedBy).toBe('external');
      expect(metadata.connectionUrl).toBe('postgres://admin:testpass@10.7.0.4:5432/testdb');
    }, 20000);

    it('should verify instance details API returns correct data', async () => {
      const res = await fetch(`${API_URL}/services/postgres/details?nats_url=${encodeURIComponent(NATS_URL)}`);
      expect(res.status).toBe(200);
      const details = await res.json();
      
      expect(details.connectionUrl).toBe('postgres://admin:testpass@10.7.0.4:5432/testdb');
      expect(details.isHealthy).toBe(true);
      expect(details.metadata.managedBy).toBe('external');
      expect(details.containerStatus).toBeUndefined();
    });
  });

  describe('Deploy Managed Service', () => {
    it('should deploy redis and verify with docker', async () => {
      await cleanupContainer('nemo-redis');

      const catalogRes = await fetch(`${API_URL}/catalog`);
      const templates = await catalogRes.json();
      const redisTemplate = templates.find((t: any) => t.id === 'redis');

      const res = await fetch(`${API_URL}/deploy`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          target_host: 'localhost',
          service_id: 'redis',
          template: redisTemplate,
          env_values: {},
          nats_url: NATS_URL,
          mode: 'deploy',
          deploy_path: DEPLOY_PATH
        })
      });

      expect(res.status).toBe(200);
      const result = await res.json();
      expect(result.success).toBe(true);

      await new Promise(r => setTimeout(r, 3000));

      // Verify with docker CLI
      const exists = await containerExists('nemo-redis');
      expect(exists).toBe(true);
      
      const running = await containerIsRunning('nemo-redis');
      expect(running).toBe(true);

      // Verify NATS
      const configs = await getConfigs();
      expect(configs['nemo_metadata.redis']).toBeDefined();
      const metadata = JSON.parse(configs['nemo_metadata.redis']);
      expect(metadata.managedBy).toBe('nemo');
      expect(metadata.containerName).toBe('nemo-redis');
    }, 60000);

    it('should get container logs via API and docker CLI', async () => {
      await new Promise(r => setTimeout(r, 2000));
      
      const res = await fetch(`${API_URL}/services/redis/logs?nats_url=${encodeURIComponent(NATS_URL)}&tail=10`);
      expect(res.status).toBe(200);
      const result = await res.json();
      expect(result.logs.length).toBeGreaterThan(0);
      console.log('API logs:', result.logs.slice(-3));

      const dockerLogs = await getContainerLogs('nemo-redis');
      expect(dockerLogs.length).toBeGreaterThan(0);
      console.log('Docker CLI logs:', dockerLogs.slice(-3));
    }, 30000);
  });

  describe('Container Operations', () => {
    it('should stop container', async () => {
      const res = await fetch(`${API_URL}/services/redis/stop?nats_url=${encodeURIComponent(NATS_URL)}`, {
        method: 'POST'
      });
      expect(res.status).toBe(200);
      
      await new Promise(r => setTimeout(r, 2000));
      const running = await containerIsRunning('nemo-redis');
      expect(running).toBe(false);
    }, 30000);

    it('should start container', async () => {
      const res = await fetch(`${API_URL}/services/redis/start?nats_url=${encodeURIComponent(NATS_URL)}`, {
        method: 'POST'
      });
      expect(res.status).toBe(200);
      
      await new Promise(r => setTimeout(r, 2000));
      const running = await containerIsRunning('nemo-redis');
      expect(running).toBe(true);
    }, 30000);

    it('should restart container', async () => {
      const res = await fetch(`${API_URL}/services/redis/restart?nats_url=${encodeURIComponent(NATS_URL)}`, {
        method: 'POST'
      });
      expect(res.status).toBe(200);
      
      await new Promise(r => setTimeout(r, 3000));
      const running = await containerIsRunning('nemo-redis');
      expect(running).toBe(true);
    }, 30000);
  });

  describe('Delete Container', () => {
    it('should delete container and remove config', async () => {
      const res = await fetch(
        `${API_URL}/services/redis/container?nats_url=${encodeURIComponent(NATS_URL)}&deploy_path=${encodeURIComponent(DEPLOY_PATH)}`,
        { method: 'DELETE' }
      );
      expect(res.status).toBe(200);

      await new Promise(r => setTimeout(r, 2000));

      const exists = await containerExists('nemo-redis');
      expect(exists).toBe(false);

      const configs = await getConfigs();
      expect(configs).not.toHaveProperty('redis.url');
      expect(configs).not.toHaveProperty('nemo_metadata.redis');
    }, 30000);
  });

  describe('Remove External Config', () => {
    it('should remove postgres config', async () => {
      const res = await fetch(
        `${API_URL}/services/postgres/config?nats_url=${encodeURIComponent(NATS_URL)}`,
        { method: 'DELETE' }
      );
      expect(res.status).toBe(200);

      const configs = await getConfigs();
      expect(configs).not.toHaveProperty('postgres.url');
      expect(configs).not.toHaveProperty('nemo_metadata.postgres');
    });
  });
});
