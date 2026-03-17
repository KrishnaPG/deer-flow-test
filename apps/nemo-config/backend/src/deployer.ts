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
