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

export interface ServiceMetadata {
  serviceId: string;
  containerName: string;
  managedBy: 'nemo' | 'external';
  host: string;
  connectionUrl: string;
  deployedAt: string;
  templateId: string;
}

export interface InstanceDetails {
  metadata: ServiceMetadata | null;
  connectionUrl: string | null;
  isHealthy: boolean;
  containerStatus?: 'running' | 'stopped' | 'not_found';
}

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

export interface ContainerActionRequest {
  service_id: string;
  nats_url: string;
  deploy_path?: string;
}

export type LogCallback = (msg: string) => void;

function generateContainerName(serviceId: string): string {
  return `nemo-${serviceId}`;
}

function injectContainerName(composeStr: string, containerName: string): string {
  const lines = composeStr.split('\n');
  const result: string[] = [];
  let inServices = false;
  let injected = false;
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i] ?? '';
    result.push(line);
    if (line.trim().startsWith('services:')) {
      inServices = true;
      continue;
    }
    if (inServices && !injected) {
      const trimmed = line.trim();
      if (trimmed && !trimmed.startsWith('#') && !trimmed.startsWith('-') && trimmed.includes(':')) {
        const match = line.match(/^(\s*)/);
        const indent = match?.[1] ?? '';
        result.push(`${indent}  container_name: ${containerName}`);
        injected = true;
      }
    }
  }
  return result.join('\n');
}

export async function getInstanceDetails(serviceId: string, natsUrl: string): Promise<InstanceDetails> {
  const configs = await getAllConfigFromNats(natsUrl, [`${serviceId}.>`, `nemo_metadata.${serviceId}`]);
  const connectionUrl = configs[`${serviceId}.url`] || null;
  const metadataRaw = configs[`nemo_metadata.${serviceId}`];
  let metadata: ServiceMetadata | null = null;
  if (metadataRaw) {
    try {
      metadata = JSON.parse(metadataRaw);
    } catch {
      const host = configs[`nemo_metadata.${serviceId}_last_host`];
      if (host && connectionUrl) {
        metadata = {
          serviceId,
          containerName: generateContainerName(serviceId),
          managedBy: host === 'external' ? 'external' : 'nemo',
          host: host,
          connectionUrl,
          deployedAt: '',
          templateId: serviceId
        };
      }
    }
  }
  let containerStatus: 'running' | 'stopped' | 'not_found' | undefined;
  if (metadata && metadata.managedBy === 'nemo' && metadata.host) {
    try {
      const status = await getContainerStatus(metadata.containerName, metadata.host);
      containerStatus = status;
    } catch {
      containerStatus = 'not_found';
    }
  }
  const isHealthy = !!connectionUrl;
  return { metadata, connectionUrl, isHealthy, containerStatus };
}

async function getContainerStatus(containerName: string, host: string): Promise<'running' | 'stopped' | 'not_found'> {
  let command: string;
  if (host === 'localhost' || host === '127.0.0.1') {
    command = `docker inspect --format='{{.State.Status}}' ${containerName} 2>/dev/null || echo "not_found"`;
  } else {
    command = `ssh ${host} "docker inspect --format='{{.State.Status}}' ${containerName} 2>/dev/null || echo 'not_found'"`;
  }
  try {
    const { stdout } = await execAsync(command);
    const status = stdout.trim();
    if (status === 'not_found') return 'not_found';
    if (status === 'running') return 'running';
    return 'stopped';
  } catch {
    return 'not_found';
  }
}

export async function deployService(req: DeployRequest, onLog?: LogCallback) {
  const log = (msg: string) => {
    console.log(msg);
    if (onLog) onLog(msg);
  };

  if (req.mode === 'existing') {
    throw new Error("Use registerExistingInstance for existing mode");
  }

  log(`[Deploy] Initiating deploy for ${req.service_id} to ${req.target_host}`);
  
  const containerName = generateContainerName(req.service_id);
  log(`[Deploy] Using container name: ${containerName}`);
  
  const ip = req.target_host === "localhost" ? "127.0.0.1" : req.target_host;
  const vars = { ...req.env_values, HOST: ip, CONTAINER_NAME: containerName };
  
  let composeStr = req.template.docker_compose;
  composeStr = injectContainerName(composeStr, containerName);
  
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
      const expandedDir = remoteDir.replace(/^~/, process.env.HOME || '');
      command = `mkdir -p ${expandedDir} && cp ${tmpFile} ${expandedDir}/docker-compose.yml && cd ${expandedDir} && docker compose up -d`;
    } else {
      command = `cat ${tmpFile} | ssh ${req.target_host} "mkdir -p ${remoteDir} && cat > ${remoteDir}/docker-compose.yml && cd ${remoteDir} && docker compose up -d"`;
    }

    log(`[Deploy] Executing: ${command.replace(/cat .* \//, "cat <template> |")}`);
    
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
    
    const metadata: ServiceMetadata = {
      serviceId: req.service_id,
      containerName,
      managedBy: 'nemo',
      host: req.target_host,
      connectionUrl,
      deployedAt: new Date().toISOString(),
      templateId: req.template.id
    };
    
    await updateNatsKV(req.nats_url, req.service_id, connectionUrl, metadata, exports, onLog);

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
  
  const vars = { ...req.env_values, HOST: 'external' };
  const exports = getExports(req.template, vars);
  
  const metadata: ServiceMetadata = {
    serviceId: req.service_id,
    containerName: '',
    managedBy: 'external',
    host: 'external',
    connectionUrl: req.connection_url,
    deployedAt: new Date().toISOString(),
    templateId: req.template.id
  };
  
  await updateNatsKV(
    req.nats_url, 
    req.service_id, 
    req.connection_url, 
    metadata,
    exports,
    onLog
  );

  log(`[Register] Successfully registered ${req.service_id}`);
  return { success: true, message: `Registered existing ${req.service_id}` };
}

export async function getContainerLogs(serviceId: string, natsUrl: string, tail: number = 100): Promise<string[]> {
  const details = await getInstanceDetails(serviceId, natsUrl);
  if (!details.metadata || details.metadata.managedBy !== 'nemo') {
    throw new Error('Cannot get logs: service is not managed by nemo');
  }
  
  const { containerName, host } = details.metadata;
  let command: string;
  
  if (host === 'localhost' || host === '127.0.0.1') {
    command = `docker logs --tail ${tail} ${containerName} 2>&1`;
  } else {
    command = `ssh ${host} "docker logs --tail ${tail} ${containerName} 2>&1"`;
  }
  
  try {
    const { stdout } = await execAsync(command, { maxBuffer: 1024 * 1024 * 10 });
    return stdout.split('\n').filter(line => line.trim());
  } catch (error: any) {
    throw new Error(`Failed to get logs: ${error.message}`);
  }
}

export async function stopContainer(serviceId: string, natsUrl: string): Promise<{ success: boolean; message: string }> {
  const details = await getInstanceDetails(serviceId, natsUrl);
  if (!details.metadata || details.metadata.managedBy !== 'nemo') {
    throw new Error('Cannot stop: service is not managed by nemo');
  }
  
  const { containerName, host } = details.metadata;
  let command: string;
  
  if (host === 'localhost' || host === '127.0.0.1') {
    command = `docker stop ${containerName}`;
  } else {
    command = `ssh ${host} "docker stop ${containerName}"`;
  }
  
  try {
    await execAsync(command);
    return { success: true, message: `Stopped container ${containerName}` };
  } catch (error: any) {
    throw new Error(`Failed to stop container: ${error.message}`);
  }
}

export async function startContainer(serviceId: string, natsUrl: string): Promise<{ success: boolean; message: string }> {
  const details = await getInstanceDetails(serviceId, natsUrl);
  if (!details.metadata || details.metadata.managedBy !== 'nemo') {
    throw new Error('Cannot start: service is not managed by nemo');
  }
  
  const { containerName, host } = details.metadata;
  let command: string;
  
  if (host === 'localhost' || host === '127.0.0.1') {
    command = `docker start ${containerName}`;
  } else {
    command = `ssh ${host} "docker start ${containerName}"`;
  }
  
  try {
    await execAsync(command);
    return { success: true, message: `Started container ${containerName}` };
  } catch (error: any) {
    throw new Error(`Failed to start container: ${error.message}`);
  }
}

export async function restartContainer(serviceId: string, natsUrl: string): Promise<{ success: boolean; message: string }> {
  const details = await getInstanceDetails(serviceId, natsUrl);
  if (!details.metadata || details.metadata.managedBy !== 'nemo') {
    throw new Error('Cannot restart: service is not managed by nemo');
  }
  
  const { containerName, host } = details.metadata;
  let command: string;
  
  if (host === 'localhost' || host === '127.0.0.1') {
    command = `docker restart ${containerName}`;
  } else {
    command = `ssh ${host} "docker restart ${containerName}"`;
  }
  
  try {
    await execAsync(command);
    return { success: true, message: `Restarted container ${containerName}` };
  } catch (error: any) {
    throw new Error(`Failed to restart container: ${error.message}`);
  }
}

export async function deleteContainer(serviceId: string, natsUrl: string, deployPath?: string): Promise<{ success: boolean; message: string }> {
  const details = await getInstanceDetails(serviceId, natsUrl);
  if (!details.metadata || details.metadata.managedBy !== 'nemo') {
    throw new Error('Cannot delete: service is not managed by nemo');
  }
  
  const { containerName, host } = details.metadata;
  const baseDir = deployPath || '~/workspace/nemo';
  const remoteDir = `${baseDir}/${serviceId}`;
  
  let commands: string[];
  
  if (host === 'localhost' || host === '127.0.0.1') {
    const expandedDir = remoteDir.replace(/^~/, process.env.HOME || '');
    commands = [
      `docker rm -f ${containerName} 2>/dev/null || true`,
      `rm -rf ${expandedDir}`
    ];
  } else {
    commands = [
      `ssh ${host} "docker rm -f ${containerName} 2>/dev/null || true && rm -rf ${remoteDir}"`
    ];
  }
  
  try {
    for (const cmd of commands) {
      await execAsync(cmd);
    }
    
    await removeServiceConfig(serviceId, natsUrl);
    
    return { success: true, message: `Deleted container ${containerName} and removed config` };
  } catch (error: any) {
    throw new Error(`Failed to delete container: ${error.message}`);
  }
}

export async function removeServiceConfig(serviceId: string, natsUrl: string): Promise<{ success: boolean; message: string }> {
  const nc = await connect({ servers: natsUrl });
  const js = nc.jetstream();
  const kv = await js.views.kv(KV_BUCKET);
  
  const prefixes = [`${serviceId}.>`, `nemo_metadata.${serviceId}.>`, `nemo_metadata.${serviceId}_`];
  
  try {
    for (const prefix of prefixes) {
      try {
        const keysIter = await kv.keys(prefix);
        for await (const key of keysIter) {
          if (key.startsWith(serviceId + '.') || key.startsWith(`nemo_metadata.${serviceId}`)) {
            await kv.delete(key);
            console.log(`[NATS] Deleted key: ${key}`);
          }
        }
      } catch (err: any) {
        if (!err.message?.includes('no keys')) {
          throw err;
        }
      }
    }
  } catch (err: any) {
    if (!err.message?.includes('no keys')) {
      throw err;
    }
  }
  
  await nc.drain();
  return { success: true, message: `Removed configuration for ${serviceId}` };
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
    
    // If specific prefixes requested, iterate through each
    const prefixes = allowedPrefixes && allowedPrefixes.length > 0 ? allowedPrefixes : [">"];
    console.log(`[DEBUG] using prefixes:`, prefixes);
    
    for (const prefix of prefixes) {
      try {
        const keysIter = await kv.keys(prefix);
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
  metadata: ServiceMetadata,
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

  await kv.put(`${serviceId}.url`, sc.encode(connectionUrl));
  log(`[NATS] Wrote ${serviceId}.url = ${connectionUrl}`);
  
  await kv.put(`nemo_metadata.${serviceId}`, sc.encode(JSON.stringify(metadata)));
  log(`[NATS] Wrote nemo_metadata.${serviceId}`);
  
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