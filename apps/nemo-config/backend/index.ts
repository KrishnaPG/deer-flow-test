import { Elysia, t } from "elysia";
import { cors } from "@elysiajs/cors";
import { websocket } from "@elysiajs/websocket";
import { readdir, readFile } from "fs/promises";
import * as yaml from "js-yaml";
import { resolve } from "path";
import { homedir } from "os";
import { connect } from "nats";
import { 
  deployService, 
  registerExistingInstance,
  getAllConfigFromNats,
  testConnection,
  type DeployRequest 
} from "./src/deployer";

const TEMPLATE_DIR = resolve(import.meta.dir, "../templates");

interface Template {
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

const app = new Elysia()
  .use(cors())
  .use(websocket())
  .ws('/ws/logs', {
    message(ws, message) {
      // Clients just listen to a global firehose, no actions needed
    },
    open(ws) {
      console.log('Frontend connected to logs websocket');
      ws.subscribe('logs');
    }
  })
  .get("/api/catalog", async () => {
    try {
      const files = await readdir(TEMPLATE_DIR);
      const templates: Template[] = [];

      for (const file of files) {
        if (file.endsWith(".yaml") || file.endsWith(".yml")) {
          const content = await readFile(resolve(TEMPLATE_DIR, file), "utf-8");
          const parsed = yaml.load(content) as Template;
          if (parsed && parsed.id) {
            templates.push(parsed);
          }
        }
      }
      return templates;
    } catch (error: any) {
      return new Response(JSON.stringify({ error: error.message }), { status: 500 });
    }
  })
  .get("/api/ssh-hosts", async () => {
    try {
      const sshConfigPath = resolve(homedir(), ".ssh", "config");
      const content = await readFile(sshConfigPath, "utf-8");

      const hosts = content
        .split('\n')
        .map(line => line.trim())
        .filter(line => line.toLowerCase().startsWith('host ') && !line.includes('*'))
        .map(line => line.split(' ')[1]);

      return ["localhost", ...hosts];
    } catch (error) {
      console.warn("Could not read ~/.ssh/config. Defaulting to localhost only.");
      return ["localhost"];
    }
  })
  // Deploy new Docker instance
  .post("/api/deploy", async ({ body, server }) => {
    try {
      const req = body as DeployRequest;
      const onLog = (msg: string) => {
        server?.publish('logs', JSON.stringify({ serviceId: req.service_id, message: msg }));
      };
      
      // We subscribe all WS clients to the 'logs' topic
      const result = await deployService(req, onLog);
      return result;
    } catch (error: any) {
      return new Response(JSON.stringify({ error: error.message }), { status: 500 });
    }
  }, {
    body: t.Object({
      target_host: t.String(),
      service_id: t.String(),
      template: t.Any(),
      env_values: t.Record(t.String(), t.String()),
      nats_url: t.String(),
      mode: t.Literal('deploy'),
      deploy_path: t.Optional(t.String())
    })
  })
  // NEW: Register existing instance
  .post("/api/register-existing", async ({ body, server }) => {
    try {
      const onLog = (msg: string) => {
        server?.publish('logs', JSON.stringify({ serviceId: body.service_id, message: msg }));
      };
      
      const result = await registerExistingInstance(body as any, onLog);
      return result;
    } catch (error: any) {
      return new Response(JSON.stringify({ error: error.message }), { status: 500 });
    }
  }, {
    body: t.Object({
      service_id: t.String(),
      connection_url: t.String(),
      nats_url: t.String(),
      template: t.Any(),
      env_values: t.Optional(t.Record(t.String(), t.String()))
    })
  })
  // NEW: Test connection to existing instance
  .post("/api/test-connection", async ({ body }) => {
    try {
      const result = await testConnection(body as any);
      return result;
    } catch (error: any) {
      return new Response(JSON.stringify({ error: error.message }), { status: 500 });
    }
  }, {
    body: t.Object({
      service_id: t.String(),
      connection_url: t.String(),
      health_check: t.Object({
        type: t.String(),
        port: t.Number(),
        path: t.Optional(t.String())
      }),
      metadata: t.Optional(t.Record(t.String(), t.String()))
    })
  })
  // Export all config from NATS KV as .env format
  .get("/api/export-env", async ({ query }) => {
    try {
      const natsUrl = query.nats_url || 'nats://localhost:4222';
      const configs = await getAllConfigFromNats(natsUrl);
      
      // Convert to .env format
      const envLines = Object.entries(configs)
        .filter(([key]) => !key.startsWith('nemo_metadata.')) // Skip internal metadata
        .map(([key, value]) => {
          // Convert key format: service.url -> SERVICE_URL
          const envKey = key.toUpperCase().replace(/\./g, '_');
          return `${envKey}=${value}`;
        });
      
      return envLines.join('\n') || '# No configurations found in NATS KV store';
    } catch (error: any) {
      console.error('[Export] Error:', error);
      return new Response(
        JSON.stringify({ error: error.message }), 
        { status: 500, headers: { 'Content-Type': 'application/json' } }
      );
    }
  })
  // NEW: Get current config from NATS for a specific service
  .get("/api/config/:serviceId", async ({ params, query }) => {
    try {
      const natsUrl = query.nats_url || 'nats://localhost:4222';
      const configs = await getAllConfigFromNats(natsUrl);
      
      // Filter for this service
      const serviceConfigs: Record<string, string> = {};
      const serviceId = (params as any).serviceId;
      for (const [key, value] of Object.entries(configs)) {
        if (key.startsWith(serviceId + '.')) {
          serviceConfigs[key] = value;
        }
      }
      
      return serviceConfigs;
    } catch (error: any) {
      return new Response(JSON.stringify({ error: error.message }), { status: 500 });
    }
  })
  // NEW: Health check for NATS connection
  .get("/api/health/nats", async ({ query }) => {
    try {
      const natsUrl = query.nats_url || 'nats://localhost:4222';
      
      // Try to connect with a short timeout
      const nc = await connect({ 
        servers: natsUrl,
        timeout: 5000 // 5 second timeout
      });
      
      // Try to get JetStream manager to verify full functionality
      await nc.jetstreamManager();
      
      await nc.drain();
      
      return { 
        status: 'healthy', 
        connected: true,
        url: natsUrl,
        timestamp: new Date().toISOString()
      };
    } catch (error: any) {
      return { 
        status: 'unhealthy', 
        connected: false,
        url: query.nats_url,
        error: error.message,
        timestamp: new Date().toISOString()
      };
    }
  })
  .listen(3001);

console.log(`🦑 Nemo-Config Backend is running at ${app.server?.hostname}:${app.server?.port}`);
