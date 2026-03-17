import { exec } from "child_process";
import { promisify } from "util";
import { resolve } from "path";
import { writeFile, unlink } from "fs/promises";
import * as yaml from "js-yaml";
import { connect, StringCodec } from "nats";

const execAsync = promisify(exec);
const sc = StringCodec();

export interface DeployRequest {
  target_host: string; // 'localhost' or an ssh host entry
  service_id: string;
  template: any;
  env_values: Record<string, string>;
  nats_url: string; // the bootstrap node
}

export async function deployService(req: DeployRequest) {
  console.log(`[Deploy] Initiating deploy for ${req.service_id} to ${req.target_host}`);
  
  // 1. Substitute Environment Variables into the Docker Compose template
  let composeStr = req.template.docker_compose;
  for (const [key, value] of Object.entries(req.env_values)) {
    composeStr = composeStr.replace(new RegExp(`\\$\\{${key}\\}`, "g"), value);
  }

  const tmpFile = resolve(import.meta.dir, `../../.tmp_${req.service_id}.yml`);
  
  try {
    // 2. Write substituted compose file locally
    await writeFile(tmpFile, composeStr);
    
    // 3. Execute via SSH or Localhost
    const remoteDir = `/opt/nemo/${req.service_id}`;
    let command = "";
    
    if (req.target_host === "localhost") {
      command = `mkdir -p ${remoteDir} && cp ${tmpFile} ${remoteDir}/docker-compose.yml && cd ${remoteDir} && docker compose up -d`;
    } else {
      // Stream the file over SSH and execute
      command = `cat ${tmpFile} | ssh ${req.target_host} "mkdir -p ${remoteDir} && cat > ${remoteDir}/docker-compose.yml && cd ${remoteDir} && docker compose up -d"`;
    }

    console.log(`[Deploy] Executing: ${command.replace(/cat .* \|/, "cat <template> |")}`); // Hide exact path in logs
    
    const { stdout, stderr } = await execAsync(command);
    console.log(`[Deploy] Success output: ${stdout}`);
    if (stderr) console.warn(`[Deploy] Warning output: ${stderr}`);

    // 4. Update NATS KV
    await updateNatsKV(req);

    return { success: true, message: `Deployed ${req.service_id} to ${req.target_host}` };
  } catch (error: any) {
    console.error(`[Deploy] Error: ${error.message}`);
    throw error;
  } finally {
    // Cleanup tmp file
    await unlink(tmpFile).catch(() => {});
  }
}

async function updateNatsKV(req: DeployRequest) {
  try {
    console.log(`[NATS] Connecting to ${req.nats_url}`);
    const nc = await connect({ servers: req.nats_url });
    const jsm = await nc.jetstreamManager();
    const js = nc.jetstream();

    // Ensure Bucket Exists
    const bucketName = "deer_flow_config";
    try {
      await jsm.kv.create({ name: bucketName });
      console.log(`[NATS] KV Bucket ${bucketName} ensured.`);
    } catch (e: any) {
      if (!e.message.includes("already exists")) {
        throw e;
      }
    }

    const kv = await js.views.kv(bucketName);
    
    // Create the connection string based on the service type
    let connectionUrl = "";
    
    // Resolve IP if localhost, else use the ssh host (assuming ssh host resolves locally or we use tailscale)
    // A more robust implementation would resolve the target_host to its actual Wireguard IP.
    // For now, if it's localhost we use 127.0.0.1, otherwise we trust the target_host name.
    const ip = req.target_host === "localhost" ? "127.0.0.1" : req.target_host;
    
    if (req.service_id === "postgres") {
      connectionUrl = `postgres://${req.env_values.POSTGRES_USER}:${req.env_values.POSTGRES_PASSWORD}@${ip}:5432/state_server`;
    } else if (req.service_id === "minio") {
      connectionUrl = `http://${ip}:9000`;
      // Also need to store creds separately or in URL depending on SDK
      await kv.put("minio.access_key", sc.encode(req.env_values.MINIO_ROOT_USER));
      await kv.put("minio.secret_key", sc.encode(req.env_values.MINIO_ROOT_PASSWORD));
    } else if (req.service_id === "redis") {
      connectionUrl = `redis://${ip}:6379`;
    }

    if (connectionUrl) {
      const key = `${req.service_id}.url`;
      await kv.put(key, sc.encode(connectionUrl));
      console.log(`[NATS] Wrote ${key} to KV store.`);
    }

    // NATS KV Memory (Option D) - Remember the last used host
    await kv.put(`nemo_metadata.${req.service_id}_last_host`, sc.encode(req.target_host));
    
    await nc.drain();
  } catch (error: any) {
    console.error(`[NATS] Error updating KV: ${error.message}`);
    throw error;
  }
}
