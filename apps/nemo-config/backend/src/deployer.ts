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

    console.log(`[Deploy] Executing: ${command.replace(/cat .* \\/, "cat <template> |")}`);
    
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

interface HealthCheck {
  type: 'tcp' | 'http';
  port: number;
  path?: string;
}

export interface TestConnectionRequest {
  service_id: string;
  connection_url: string;
  health_check: HealthCheck;
  metadata?: Record<string, string>;
}

export async function testConnection(req: TestConnectionRequest): Promise<{ success: boolean; message: string; details?: any }> {
  console.log(`[Test] Testing connection to ${req.service_id} at ${req.connection_url}`);
  
  const { host, port } = parseHostPortFromUrl(req.connection_url);
  const healthCheck = req.health_check;
  
  try {
    if (healthCheck.type === 'tcp') {
      return await testTcpConnection(host, healthCheck.port);
    } else if (healthCheck.type === 'http') {
      const path = healthCheck.path || '/';
      // Build HTTP URL from connection URL
      let httpUrl: string;
      if (req.connection_url.startsWith('http://') || req.connection_url.startsWith('https://')) {
        // Replace the port and path in the URL
        const parsed = new URL(req.connection_url);
        parsed.port = healthCheck.port.toString();
        parsed.pathname = path;
        httpUrl = parsed.toString();
      } else {
        // Default to http if no protocol specified
        httpUrl = `http://${host}:${healthCheck.port}${path}`;
      }
      return await testHttpConnection(httpUrl);
    } else {
      throw new Error(`Unknown health check type: ${healthCheck.type}`);
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

async function testTcpConnection(host: string, port: number): Promise<{ success: boolean; message: string; details?: any }> {
  try {
    // Use netcat (nc) to test TCP connection
    const command = `nc -z -w 5 ${host} ${port}`;
    await execAsync(command);
    
    return { 
      success: true, 
      message: `TCP connection successful to ${host}:${port}`,
      details: { host, port, command }
    };
  } catch (error: any) {
    return { 
      success: false, 
      message: `TCP connection failed to ${host}:${port}: ${error.message}`,
      details: { host, port, error: error.message }
    };
  }
}

async function testHttpConnection(url: string): Promise<{ success: boolean; message: string; details?: any }> {
  try {
    // Use curl to test HTTP connection
    // -f flag makes curl return error on HTTP error codes (4xx, 5xx)
    // -s silent mode
    // -o /dev/null discard output
    // --connect-timeout 5 connection timeout
    // --max-time 10 max time for request
    const command = `curl -f -s -o /dev/null --connect-timeout 5 --max-time 10 "${url}"`;
    await execAsync(command);
    
    return { 
      success: true, 
      message: `HTTP connection successful to ${url}`,
      details: { url, command }
    };
  } catch (error: any) {
    // curl returns non-zero exit code even if server responds with error status
    // but we're using -f so any HTTP error is treated as failure
    return { 
      success: false, 
      message: `HTTP connection failed to ${url}: ${error.message}`,
      details: { url, error: error.message }
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
    // Try to extract host:port from various URL formats
    const patterns = [
      /@([^:]+):(\d+)\//,  // postgres://user:pass@host:port/db
      /:\/\/([^:]+):(\d+)/, // protocol://host:port
      /^([^:]+):(\d+)$/     // host:port
    ];
    
    for (const pattern of patterns) {
      const match = url.match(pattern);
      if (match) {
        return {
          host: match[1],
          port: parseInt(match[2], 10)
        };
      }
    }
    
    throw new Error(`Could not parse host and port from URL: ${url}`);
  }
}
