import { exec } from "child_process";
import { promisify } from "util";
import { resolve } from "path";
import { writeFile, unlink } from "fs/promises";
import * as yaml from "js-yaml";
import { connect, StringCodec } from "nats";

const execAsync = promisify(exec);
const sc = StringCodec();

const KV_BUCKET = "deer_flow_config";

export interface DeployRequest {
  target_host: string;
  service_id: string;
  template: any;
  env_values: Record<string, string>;
  nats_url: string;
  mode: 'deploy' | 'existing'; // NEW: deploy = docker, existing = register URL
}

export interface RegisterExistingRequest {
  service_id: string;
  connection_url: string;
  nats_url: string;
  metadata?: Record<string, string>; // For extra fields like access_key/secret_key
}

export async function deployService(req: DeployRequest) {
  if (req.mode === 'existing') {
    throw new Error("Use registerExistingInstance for existing mode");
  }

  console.log(`[Deploy] Initiating deploy for ${req.service_id} to ${req.target_host}`);
  
  let composeStr = req.template.docker_compose;
  for (const [key, value] of Object.entries(req.env_values)) {
    composeStr = composeStr.replace(new RegExp(`\\$\\{${key}\\}`, "g"), value);
  }

  const tmpFile = resolve(import.meta.dir, `../../.tmp_${req.service_id}.yml`);
  
  try {
    await writeFile(tmpFile, composeStr);
    
    const remoteDir = `/opt/nemo/${req.service_id}`;
    let command = "";
    
    if (req.target_host === "localhost") {
      command = `mkdir -p ${remoteDir} && cp ${tmpFile} ${remoteDir}/docker-compose.yml && cd ${remoteDir} && docker compose up -d`;
    } else {
      command = `cat ${tmpFile} | ssh ${req.target_host} "mkdir -p ${remoteDir} && cat > ${remoteDir}/docker-compose.yml && cd ${remoteDir} && docker compose up -d"`;
    }

    console.log(`[Deploy] Executing: ${command.replace(/cat .* \\|/, "cat <template> |")}`);
    
    const { stdout, stderr } = await execAsync(command);
    console.log(`[Deploy] Success output: ${stdout}`);
    if (stderr) console.warn(`[Deploy] Warning output: ${stderr}`);

    // Build connection URL from deployed service
    const connectionUrl = buildConnectionUrl(req.service_id, req.target_host, req.env_values);
    await updateNatsKV(req.nats_url, req.service_id, connectionUrl, req.target_host, req.env_values);

    return { success: true, message: `Deployed ${req.service_id} to ${req.target_host}` };
  } catch (error: any) {
    console.error(`[Deploy] Error: ${error.message}`);
    throw error;
  } finally {
    await unlink(tmpFile).catch(() => {});
  }
}

export async function registerExistingInstance(req: RegisterExistingRequest) {
  console.log(`[Register] Registering existing ${req.service_id} at ${req.connection_url}`);
  
  await updateNatsKV(
    req.nats_url, 
    req.service_id, 
    req.connection_url, 
    'external', 
    req.metadata || {}
  );

  return { success: true, message: `Registered existing ${req.service_id}` };
}

export async function getAllConfigFromNats(natsUrl: string): Promise<Record<string, string>> {
  console.log(`[NATS] Fetching all config from ${natsUrl}`);
  const nc = await connect({ servers: natsUrl });
  const js = nc.jetstream();
  
  try {
    const kv = await js.views.kv(KV_BUCKET);
    const configs: Record<string, string> = {};
    
    // Watch all keys and collect them
    const iter = await kv.watch();
    for await (const entry of iter) {
      if (entry.operation === "PUT") {
        configs[entry.key] = sc.decode(entry.value);
      }
    }
    
    await nc.drain();
    return configs;
  } catch (error: any) {
    await nc.drain();
    if (error.message?.includes("bucket not found")) {
      return {}; // Bucket doesn't exist yet
    }
    throw error;
  }
}

function buildConnectionUrl(
  serviceId: string, 
  targetHost: string, 
  envValues: Record<string, string>
): string {
  const ip = targetHost === "localhost" ? "127.0.0.1" : targetHost;
  
  switch (serviceId) {
    case 'postgres':
      return `postgres://${envValues.POSTGRES_USER}:${envValues.POSTGRES_PASSWORD}@${ip}:5432/state_server`;
    case 'minio':
      return `http://${ip}:9000`;
    case 'redis':
      return `redis://${ip}:6379`;
    case 'nats':
      return `nats://${ip}:4222`;
    case 'clickhouse':
      return `http://${ip}:8123`;
    case 'temporal':
      return `${ip}:7233`;
    case 'livekit':
      return `wss://${ip}:7880`;
    case 'signoz':
      return `http://${ip}:3301`;
    default:
      return `${ip}:${envValues.PORT || 'unknown'}`;
  }
}

async function updateNatsKV(
  natsUrl: string,
  serviceId: string,
  connectionUrl: string,
  targetHost: string,
  envValues: Record<string, string>
) {
  console.log(`[NATS] Connecting to ${natsUrl}`);
  const nc = await connect({ servers: natsUrl });
  const jsm = await nc.jetstreamManager();
  const js = nc.jetstream();

  // Ensure Bucket Exists
  try {
    await jsm.kv.create({ name: KV_BUCKET });
    console.log(`[NATS] KV Bucket ${KV_BUCKET} ensured.`);
  } catch (e: any) {
    if (!e.message.includes("already exists")) {
      throw e;
    }
  }

  const kv = await js.views.kv(KV_BUCKET);
  
  // Store the connection URL
  const urlKey = `${serviceId}.url`;
  await kv.put(urlKey, sc.encode(connectionUrl));
  console.log(`[NATS] Wrote ${urlKey} = ${connectionUrl}`);

  // Store extra credentials if needed
  if (serviceId === 'minio') {
    if (envValues.MINIO_ROOT_USER) {
      await kv.put("minio.access_key", sc.encode(envValues.MINIO_ROOT_USER));
    }
    if (envValues.MINIO_ROOT_PASSWORD) {
      await kv.put("minio.secret_key", sc.encode(envValues.MINIO_ROOT_PASSWORD));
    }
  }
  
  if (serviceId === 'postgres') {
    await kv.put("postgres.user", sc.encode(envValues.POSTGRES_USER || 'admin'));
    await kv.put("postgres.password", sc.encode(envValues.POSTGRES_PASSWORD || 'password'));
    await kv.put("postgres.database", sc.encode(envValues.POSTGRES_DB || 'state_server'));
  }

  // Remember the last used host
  await kv.put(`nemo_metadata.${serviceId}_last_host`, sc.encode(targetHost));
  
  await nc.drain();
}

export interface TestConnectionRequest {
  service_id: string;
  connection_url: string;
  metadata?: Record<string, string>;
}

export async function testConnection(req: TestConnectionRequest): Promise<{ success: boolean; message: string; details?: any }> {
  console.log(`[Test] Testing connection to ${req.service_id} at ${req.connection_url}`);
  
  const url = req.connection_url;
  
  try {
    // Parse URL to determine connection type
    if (url.startsWith('postgres://') || url.startsWith('postgresql://')) {
      return await testPostgresConnection(url);
    } else if (url.startsWith('redis://') || url.startsWith('rediss://')) {
      return await testRedisConnection(url);
    } else if (url.startsWith('http://') || url.startsWith('https://')) {
      return await testHttpConnection(url);
    } else if (url.startsWith('nats://')) {
      return await testNatsConnection(url);
    } else {
      // Try TCP connection for unknown protocols
      return await testTcpConnection(url);
    }
  } catch (error: any) {
    console.error(`[Test] Connection test failed: ${error.message}`);
    return { 
      success: false, 
      message: `Connection test failed: ${error.message}`,
      details: { error: error.message }
    };
  }
}

async function testPostgresConnection(url: string): Promise<{ success: boolean; message: string; details?: any }> {
  try {
    // Use pg_isready if available, otherwise use a simple TCP check
    const { host, port } = parseHostPortFromUrl(url);
    
    // Try TCP connection first
    const net = await import('net');
    await new Promise<void>((resolve, reject) => {
      const socket = net.createConnection({ host, port: port || 5432 }, () => {
        socket.end();
        resolve();
      });
      socket.setTimeout(5000);
      socket.on('error', reject);
      socket.on('timeout', () => {
        socket.destroy();
        reject(new Error('Connection timeout'));
      });
    });
    
    return { 
      success: true, 
      message: 'Successfully connected to PostgreSQL',
      details: { host, port: port || 5432 }
    };
  } catch (error: any) {
    return { 
      success: false, 
      message: `PostgreSQL connection failed: ${error.message}`,
      details: { error: error.message }
    };
  }
}

async function testRedisConnection(url: string): Promise<{ success: boolean; message: string; details?: any }> {
  try {
    const { host, port } = parseHostPortFromUrl(url);
    
    const net = await import('net');
    await new Promise<void>((resolve, reject) => {
      const socket = net.createConnection({ host, port: port || 6379 }, () => {
        socket.end();
        resolve();
      });
      socket.setTimeout(5000);
      socket.on('error', reject);
      socket.on('timeout', () => {
        socket.destroy();
        reject(new Error('Connection timeout'));
      });
    });
    
    return { 
      success: true, 
      message: 'Successfully connected to Redis',
      details: { host, port: port || 6379 }
    };
  } catch (error: any) {
    return { 
      success: false, 
      message: `Redis connection failed: ${error.message}`,
      details: { error: error.message }
    };
  }
}

async function testHttpConnection(url: string): Promise<{ success: boolean; message: string; details?: any }> {
  try {
    const response = await fetch(url, { 
      method: 'GET',
      signal: AbortSignal.timeout(10000)
    });
    
    return { 
      success: true, 
      message: `HTTP connection successful (status: ${response.status})`,
      details: { 
        status: response.status,
        statusText: response.statusText,
        url: url 
      }
    };
  } catch (error: any) {
    // For services like MinIO, they might return an error but still be reachable
    if (error.message?.includes('fetch failed') || error.message?.includes('ECONNREFUSED')) {
      return { 
        success: false, 
        message: `HTTP connection failed: Unable to reach ${url}`,
        details: { error: error.message }
      };
    }
    
    // Some services are reachable but return errors (auth required, etc.)
    return { 
      success: true, 
      message: `Service is reachable (returned error: ${error.message})`,
      details: { 
        reachable: true,
        error: error.message 
      }
    };
  }
}

async function testNatsConnection(url: string): Promise<{ success: boolean; message: string; details?: any }> {
  try {
    const nc = await connect({ 
      servers: url,
      timeout: 5000
    });
    await nc.jetstreamManager();
    await nc.drain();
    
    return { 
      success: true, 
      message: 'Successfully connected to NATS',
      details: { url }
    };
  } catch (error: any) {
    return { 
      success: false, 
      message: `NATS connection failed: ${error.message}`,
      details: { error: error.message }
    };
  }
}

async function testTcpConnection(url: string): Promise<{ success: boolean; message: string; details?: any }> {
  try {
    // Parse host:port format
    const match = url.match(/^([^:]+):(\d+)$/);
    if (!match) {
      return { 
        success: false, 
        message: `Unable to parse connection URL: ${url}. Expected format: host:port`,
        details: { url }
      };
    }
    
    const [, host, portStr] = match;
    const port = parseInt(portStr, 10);
    
    const net = await import('net');
    await new Promise<void>((resolve, reject) => {
      const socket = net.createConnection({ host, port }, () => {
        socket.end();
        resolve();
      });
      socket.setTimeout(5000);
      socket.on('error', reject);
      socket.on('timeout', () => {
        socket.destroy();
        reject(new Error('Connection timeout'));
      });
    });
    
    return { 
      success: true, 
      message: `Successfully connected to ${host}:${port}`,
      details: { host, port }
    };
  } catch (error: any) {
    return { 
      success: false, 
      message: `TCP connection failed: ${error.message}`,
      details: { error: error.message }
    };
  }
}

function parseHostPortFromUrl(url: string): { host: string; port?: number } {
  try {
    const parsed = new URL(url);
    return {
      host: parsed.hostname,
      port: parsed.port ? parseInt(parsed.port, 10) : undefined
    };
  } catch {
    // Fallback for URLs that might not parse with URL constructor
    const match = url.match(/@([^:]+):(\d+)\//);
    if (match) {
      return {
        host: match[1],
        port: parseInt(match[2], 10)
      };
    }
    throw new Error(`Could not parse host and port from URL: ${url}`);
  }
}
