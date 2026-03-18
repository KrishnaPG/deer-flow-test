import { exec } from "child_process";
import { promisify } from "util";
import { resolve } from "path";
import { writeFile, unlink } from "fs/promises";
import { connect, StringCodec } from "nats";
import * as net from "net";
import { loadHealthChecker } from "./health-checks";
import { interpolate } from "./interpolate";

const execAsync = promisify(exec);
const sc = StringCodec();

const KV_BUCKET = "nemo_config";

export interface Template {
  name: string;
  id: string;
  icon: string;
  default_port: number;
  env_vars: { key: string; description: string; default?: string; secret?: boolean }[];
  health_check: { type: string; port: number; path?: string };
  docker_compose: string;
  connection_url_pattern?: string;
  exports?: Record<string, string>;
}

export interface DeployRequest {
  target_host: string;
  service_id: string;
  template: Template;
  env_values: Record<string, string>;
  nats_url: string;
  mode: 'deploy' | 'existing';
  deploy_path?: string;
}

export interface RegisterExistingRequest {
  service_id: string;
  connection_url: string;
  nats_url: string;
  template: Template;
  env_values?: Record<string, string>;
}

export type LogCallback = (msg: string) => void;

export async function deployService(req: DeployRequest, onLog?: LogCallback) {
  const log = (msg: string) => {
    console.log(msg);
    if (onLog) onLog(msg);
  };

  if (req.mode === 'existing') {
    throw new Error("Use registerExistingInstance for existing mode");
  }

  log(`[Deploy] Initiating deploy for ${req.service_id} to ${req.target_host}`);
  
  const ip = req.target_host === "localhost" ? "127.0.0.1" : req.target_host;
  const vars = { ...req.env_values, HOST: ip };
  
  let composeStr = req.template.docker_compose;
  for (const [key, value] of Object.entries(vars)) {
    composeStr = composeStr.replace(new RegExp(`\\$\\{${key}\\}`, "g"), value);
  }

  const tmpFile = resolve(import.meta.dir, `../../.tmp_${req.service_id}.yml`);
  
  try {
    await writeFile(tmpFile, composeStr);
    
    const baseDir = req.deploy_path || '~/workspace/nemo';
    const remoteDir = `${baseDir}/${req.service_id}`;
    let command = "";
    
    if (req.target_host === "localhost") {
      // Expand ~ to the actual home directory for localhost since we don't use ssh
      const expandedDir = remoteDir.replace(/^~/, process.env.HOME || '');
      command = `mkdir -p ${expandedDir} && cp ${tmpFile} ${expandedDir}/docker-compose.yml && cd ${expandedDir} && docker compose up -d`;
    } else {
      command = `cat ${tmpFile} | ssh ${req.target_host} "mkdir -p ${remoteDir} && cat > ${remoteDir}/docker-compose.yml && cd ${remoteDir} && docker compose up -d"`;
    }

    log(`[Deploy] Executing: ${command.replace(/cat .* \\/, "cat <template> |")}`);
    
    // Instead of simple exec, we spawn to capture real-time output
    const { spawn } = await import('child_process');
    await new Promise<void>((resolvePromise, reject) => {
      const child = spawn(command, { shell: true });
      
      child.stdout.on('data', (data) => {
        const text = data.toString().trim();
        if (text) log(`[Docker] ${text}`);
      });
      
      child.stderr.on('data', (data) => {
        const text = data.toString().trim();
        if (text) log(`[Docker] ${text}`);
      });
      
      child.on('close', (code) => {
        if (code !== 0) {
          reject(new Error(`Command failed with exit code ${code}`));
        } else {
          resolvePromise();
        }
      });
      
      child.on('error', (err) => {
        reject(err);
      });
    });

    const connectionUrl = buildConnectionUrl(req.template, vars);
    const exports = getExports(req.template, vars);
    
    await updateNatsKV(req.nats_url, req.service_id, connectionUrl, req.target_host, exports, onLog);

    log(`[Deploy] Successfully deployed ${req.service_id}`);
    return { success: true, message: `Deployed ${req.service_id} to ${req.target_host}` };
  } catch (error: any) {
    if (onLog) onLog(`[Deploy] Error: ${error.message}`);
    console.error(`[Deploy] Error: ${error.message}`);
    throw error;
  } finally {
    await unlink(tmpFile).catch(() => {});
  }
}

export async function registerExistingInstance(req: RegisterExistingRequest, onLog?: LogCallback) {
  const log = (msg: string) => {
    console.log(msg);
    if (onLog) onLog(msg);
  };
  
  log(`[Register] Registering existing ${req.service_id} at ${req.connection_url}`);
  
  const ip = 'external';
  const vars = { ...req.env_values, HOST: ip };
  const exports = getExports(req.template, vars);
  
  exports[`${req.service_id}.url`] = req.connection_url;
  
  await updateNatsKV(
    req.nats_url, 
    req.service_id, 
    req.connection_url, 
    'external', 
    exports,
    onLog
  );

  log(`[Register] Successfully registered ${req.service_id}`);
  return { success: true, message: `Registered existing ${req.service_id}` };
}

function buildConnectionUrl(template: Template, vars: Record<string, string>): string {
  if (!template.connection_url_pattern) {
    const ip = vars.HOST || 'localhost';
    return `${ip}:${template.default_port}`;
  }
  return interpolate(template.connection_url_pattern, vars);
}

function getExports(template: Template, vars: Record<string, string>): Record<string, string> {
  const result: Record<string, string> = {};
  
  if (template.exports) {
    for (const [key, pattern] of Object.entries(template.exports)) {
      result[key] = interpolate(pattern, vars);
    }
  }
  
  return result;
}

export async function getAllConfigFromNats(natsUrl: string, allowedPrefixes?: string[]): Promise<Record<string, string>> {
  console.log(`[NATS] Fetching config from ${natsUrl}`);
  const nc = await connect({ servers: natsUrl });
  const js = nc.jetstream();
  
  try {
    const kv = await js.views.kv(KV_BUCKET);
    const configs: Record<string, string> = {};
    
    const filters = allowedPrefixes && allowedPrefixes.length > 0 ? allowedPrefixes : [">"];
    console.log(`[DEBUG] using filters:`, filters);
    try {
      const keysIter = await kv.keys(filters);
      for await (const key of keysIter) {
        console.log(`[DEBUG] Got key from NATS: ${key}`);
        const entry = await kv.get(key);
        if (entry && entry.operation !== "DEL" && entry.operation !== "PURGE") {
          try {
            configs[key] = sc.decode(entry.value);
            console.log(`[DEBUG] Decoded key ${key} = ${configs[key]}`);
          } catch (decodeErr) {
            console.warn(`[NATS] Failed to decode key ${key}:`, decodeErr);
          }
        } else {
          console.log(`[DEBUG] Skipped key ${key} due to operation ${entry?.operation}`);
        }
      }
    } catch (err: any) {
      // If no keys match filter, it might throw a 404 No Messages error
      if (!err.message?.includes("no messages")) {
        throw err;
      }
    }
    
    await nc.drain();
    return configs;
  } catch (error: any) {
    await nc.drain();
    if (error.message?.includes("bucket not found")) {
      return {};
    }
    throw error;
  }
}

async function updateNatsKV(
  natsUrl: string,
  serviceId: string,
  connectionUrl: string,
  targetHost: string,
  exports: Record<string, string>,
  onLog?: LogCallback
) {
  const log = (msg: string) => {
    console.log(msg);
    if (onLog) onLog(msg);
  };
  
  log(`[NATS] Connecting to ${natsUrl}`);
  const nc = await connect({ servers: natsUrl });
  const js = nc.jetstream();

  const kv = await js.views.kv(KV_BUCKET);
  
  for (const [key, value] of Object.entries(exports)) {
    await kv.put(key, sc.encode(value));
    log(`[NATS] Wrote ${key} = ${value}`);
  }

  // Double check the main URL is written
  await kv.put(`${serviceId}.url`, sc.encode(connectionUrl));
  
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
  
  const healthChecker = await loadHealthChecker(req.service_id);
  
  if (healthChecker) {
    console.log(`[Test] Using service-specific health checker for ${req.service_id}`);
    return healthChecker(req.connection_url, req.metadata);
  }
  
  console.log(`[Test] No service-specific health checker for ${req.service_id}, using fallback`);
  return fallbackHealthCheck(req.service_id, req.connection_url, req.health_check);
}

async function fallbackHealthCheck(
  serviceId: string,
  connectionUrl: string,
  healthCheck: { type: string; port: number; path?: string }
): Promise<{ success: boolean; message: string; details?: any }> {
  const { host, port } = parseHostPortFromUrl(connectionUrl);
  const checkPort = healthCheck.port || port || 0;
  
  if (healthCheck.type === 'http') {
    const path = healthCheck.path || '/';
    let httpUrl: string;
    if (connectionUrl.startsWith('http://') || connectionUrl.startsWith('https://')) {
      const parsed = new URL(connectionUrl);
      parsed.port = checkPort.toString();
      parsed.pathname = path;
      httpUrl = parsed.toString();
    } else {
      httpUrl = `http://${host}:${checkPort}${path}`;
    }
    return testHttpConnection(httpUrl);
  }
  
  return testTcpConnection(host, checkPort);
}

async function testTcpConnection(host: string, port: number): Promise<{ success: boolean; message: string; details?: any }> {
  return new Promise((resolve) => {
    const socket = new net.Socket();
    const timeout = 5000;
    
    const timer = setTimeout(() => {
      socket.destroy();
      resolve({ 
        success: false, 
        message: `TCP connection timed out to ${host}:${port}`,
        details: { host, port, error: 'Connection timed out' }
      });
    }, timeout);
    
    socket.connect(port, host, () => {
      clearTimeout(timer);
      socket.destroy();
      resolve({ 
        success: true, 
        message: `TCP connection successful to ${host}:${port}`,
        details: { host, port }
      });
    });
    
    socket.on('error', (err) => {
      clearTimeout(timer);
      resolve({ 
        success: false, 
        message: `TCP connection failed to ${host}:${port}: ${err.message}`,
        details: { host, port, error: err.message }
      });
    });
  });
}

async function testHttpConnection(url: string): Promise<{ success: boolean; message: string; details?: any }> {
  try {
    const command = `curl -f -s -o /dev/null --connect-timeout 5 --max-time 10 "${url}"`;
    await execAsync(command);
    
    return { 
      success: true, 
      message: `HTTP connection successful to ${url}`,
      details: { url, command }
    };
  } catch (error: any) {
    return { 
      success: false, 
      message: `HTTP connection failed to ${url}: ${error.message}`,
      details: { url, error: error.message }
    };
  }
}

export interface ParsedHostPort {
  host: string;
  port?: number;
}

function parseHostPortFromUrl(url: string): ParsedHostPort {
  try {
    const parsed = new URL(url);
    return {
      host: parsed.hostname,
      port: parsed.port ? parseInt(parsed.port, 10) : undefined
    };
  } catch {
    const patterns = [
      /@([^:]+):(\d+)\//,
      /:\/\/([^:]+):(\d+)/,
      /^([^:]+):(\d+)$/
    ];
    
    for (const pattern of patterns) {
      const match = url.match(pattern);
      if (match && match[1] && match[2]) {
        return {
          host: match[1],
          port: parseInt(match[2], 10)
        };
      }
    }
    
    throw new Error(`Could not parse host and port from URL: ${url}`);
  }
}