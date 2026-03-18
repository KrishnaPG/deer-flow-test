import { Elysia, t } from "elysia";
import { cors } from "@elysiajs/cors";
import { readFile } from "fs/promises";
import { resolve } from "path";
import { homedir } from "os";
import { getCatalogTemplates, getCatalogPrefixes } from "./src/catalog";
import { 
  deployService, 
  registerExistingInstance,
  getAllConfigFromNats,
  testConnection,
  getInstanceDetails,
  getContainerLogs,
  stopContainer,
  startContainer,
  restartContainer,
  deleteContainer,
  removeServiceConfig,
  type DeployRequest 
} from "./src/deployer";
import { 
  checkConsulHealth,
  getAllServicesHealth,
  DEFAULT_CONSUL_URL
} from "./src/consul-store";

const app = new Elysia()
  .use(cors())
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
      return await getCatalogTemplates();
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
      consul_url: t.String(),
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
      
      console.log(`[API] Registering existing ${body.service_id}`);
      const result = await registerExistingInstance(body as any, onLog);
      return result;
    } catch (error: any) {
      console.error(`[API] Error registering existing ${body.service_id}:`, error);
      return new Response(JSON.stringify({ error: error.message, stack: error.stack }), { 
        status: 500,
        headers: { 'Content-Type': 'application/json' }
      });
    }
  }, {
    body: t.Object({
      service_id: t.String(),
      connection_url: t.String(),
      consul_url: t.String(),
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
  // Export all config from Consul KV as .env format
  .get("/api/export-env", async ({ query }) => {
    try {
      const consulUrl = (query.consul_url as string) || DEFAULT_CONSUL_URL;
      const prefixes = await getCatalogPrefixes();
      const configs = await getAllConfigFromNats(consulUrl, prefixes);
      
      // Convert to .env format
      const envLines = Object.entries(configs)
        .filter(([key]) => !key.includes('metadata')) // Skip internal metadata
        .map(([key, value]) => {
          // Convert key format: service.url -> SERVICE_URL
          const envKey = key.toUpperCase().replace(/\./g, '_');
          return `${envKey}=${value}`;
        });
      
      return envLines.join('\n') || '# No configurations found in Consul KV store';
    } catch (error: any) {
      console.error('[Export] Error:', error);
      return new Response(
        JSON.stringify({ error: error.message }), 
        { status: 500, headers: { 'Content-Type': 'application/json' } }
      );
    }
  })
  // NEW: Get all configs for frontend state
  .get("/api/configs", async ({ query }) => {
    try {
      const consulUrl = (query.consul_url as string) || DEFAULT_CONSUL_URL;
      const prefixes = await getCatalogPrefixes();
      const configs = await getAllConfigFromNats(consulUrl, prefixes);
      return configs;
    } catch (error: any) {
      return new Response(JSON.stringify({ error: error.message }), { status: 500 });
    }
  })
  // NEW: Get current config from Consul for a specific service
  .get("/api/config/:serviceId", async ({ params, query }) => {
    try {
      const consulUrl = (query.consul_url as string) || DEFAULT_CONSUL_URL;
      const serviceId = (params as any).serviceId;
      const configs = await getAllConfigFromNats(consulUrl, [`${serviceId}.>`]);
      
      // Filter for this service
      const serviceConfigs: Record<string, string> = {};
      for (const [key, value] of Object.entries(configs)) {
        // Check both old format (serviceId.key) and new format (nemo.serviceId.key)
        if (key.startsWith(serviceId + '.') || key.startsWith('nemo.' + serviceId + '.')) {
          serviceConfigs[key] = value;
        }
      }
      
      return serviceConfigs;
    } catch (error: any) {
      return new Response(JSON.stringify({ error: error.message }), { status: 500 });
    }
  })
  // NEW: Health check for Consul connection
  .get("/api/health/consul", async ({ query }) => {
    try {
      const consulUrl = (query.consul_url as string) || DEFAULT_CONSUL_URL;
      return await checkConsulHealth(consulUrl);
    } catch (error: any) {
      return {
        status: 'unhealthy',
        connected: false,
        url: query.consul_url,
        error: error.message,
        timestamp: new Date().toISOString()
      };
    }
  })
  // NEW: Health check for all registered services
  .get("/api/health/services", async ({ query }) => {
    try {
      const consulUrl = (query.consul_url as string) || DEFAULT_CONSUL_URL;
      const health = await getAllServicesHealth(consulUrl);
      return health;
    } catch (error: any) {
      return new Response(JSON.stringify({ error: error.message }), { status: 500 });
    }
  })
  // NEW: Get instance details for a service
  .get("/api/services/:serviceId/details", async ({ params, query }) => {
    try {
      const consulUrl = (query.consul_url as string) || DEFAULT_CONSUL_URL;
      const serviceId = (params as any).serviceId;
      const details = await getInstanceDetails(serviceId, consulUrl);
      return details;
    } catch (error: any) {
      return new Response(JSON.stringify({ error: error.message }), { status: 500 });
    }
  })
  // NEW: Get container logs for a service
  .get("/api/services/:serviceId/logs", async ({ params, query }) => {
    try {
      const consulUrl = (query.consul_url as string) || DEFAULT_CONSUL_URL;
      const serviceId = (params as any).serviceId;
      const tail = parseInt((query as any).tail as string) || 100;
      const logs = await getContainerLogs(serviceId, consulUrl, tail);
      return { logs };
    } catch (error: any) {
      return new Response(JSON.stringify({ error: error.message }), { status: 500 });
    }
  })
  // NEW: Stop container
  .post("/api/services/:serviceId/stop", async ({ params, query }) => {
    try {
      const consulUrl = (query.consul_url as string) || DEFAULT_CONSUL_URL;
      const serviceId = (params as any).serviceId;
      const result = await stopContainer(serviceId, consulUrl);
      return result;
    } catch (error: any) {
      return new Response(JSON.stringify({ error: error.message }), { status: 500 });
    }
  })
  // NEW: Start container
  .post("/api/services/:serviceId/start", async ({ params, query }) => {
    try {
      const consulUrl = (query.consul_url as string) || DEFAULT_CONSUL_URL;
      const serviceId = (params as any).serviceId;
      const result = await startContainer(serviceId, consulUrl);
      return result;
    } catch (error: any) {
      return new Response(JSON.stringify({ error: error.message }), { status: 500 });
    }
  })
  // NEW: Restart container
  .post("/api/services/:serviceId/restart", async ({ params, query }) => {
    try {
      const consulUrl = (query.consul_url as string) || DEFAULT_CONSUL_URL;
      const serviceId = (params as any).serviceId;
      const result = await restartContainer(serviceId, consulUrl);
      return result;
    } catch (error: any) {
      return new Response(JSON.stringify({ error: error.message }), { status: 500 });
    }
  })
  // NEW: Delete container and remove config (for managed services)
  .delete("/api/services/:serviceId/container", async ({ params, query }) => {
    try {
      const consulUrl = (query.consul_url as string) || DEFAULT_CONSUL_URL;
      const deployPath = (query as any).deploy_path as string | undefined;
      const serviceId = (params as any).serviceId;
      const result = await deleteContainer(serviceId, consulUrl, deployPath);
      return result;
    } catch (error: any) {
      return new Response(JSON.stringify({ error: error.message }), { status: 500 });
    }
  })
  // NEW: Remove config only (for external services)
  .delete("/api/services/:serviceId/config", async ({ params, query }) => {
    try {
      const consulUrl = (query.consul_url as string) || DEFAULT_CONSUL_URL;
      const serviceId = (params as any).serviceId;
      const result = await removeServiceConfig(serviceId, consulUrl);
      return result;
    } catch (error: any) {
      return new Response(JSON.stringify({ error: error.message }), { status: 500 });
    }
  })
  .listen(3001);

console.log(`🦑 Nemo-Config Backend is running at ${app.server?.hostname}:${app.server?.port}`);
