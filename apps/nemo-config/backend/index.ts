import { Elysia, t } from "elysia";
import { cors } from "@elysiajs/cors";
import { readdir, readFile } from "fs/promises";
import * as yaml from "js-yaml";
import { resolve } from "path";
import { homedir } from "os";
import { deployService, DeployRequest } from "./src/deployer";

const TEMPLATE_DIR = resolve(import.meta.dir, "../../templates");

interface Template {
  name: string;
  id: string;
  icon: string;
  default_port: number;
  env_vars: { key: string; description: string; default?: string; secret?: boolean }[];
  health_check: { type: string; port: number; path?: string };
  docker_compose: string;
}

const app = new Elysia()
  .use(cors())
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
  .post("/api/deploy", async ({ body }) => {
    try {
      const req = body as DeployRequest;
      const result = await deployService(req);
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
      nats_url: t.String()
    })
  })
  // The "Export Eject" Route (Option C)
  .get("/api/export-env", async () => {
    // In a real implementation, this would connect to NATS, fetch all URLs, and build the .env string
    // Since we're in the setup phase, we'll return a stub for now.
    return "NATS_URL=nats://localhost:4222\nPOSTGRES_URL=postgres://admin:password@localhost:5432/state_server\nMINIO_ENDPOINT=http://localhost:9000";
  })
  .listen(3001);

console.log(`🦑 Nemo-Config Backend is running at ${app.server?.hostname}:${app.server?.port}`);
