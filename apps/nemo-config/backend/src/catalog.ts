import { readdir, readFile } from "fs/promises";
import * as yaml from "js-yaml";
import { resolve } from "path";
import { type Template } from "./deployer";

const TEMPLATE_DIR = resolve(import.meta.dir, "../../templates");

/**
 * Reads and parses all service templates from the catalog directory.
 */
export async function getCatalogTemplates(): Promise<Template[]> {
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
}

/**
 * Returns a list of valid NATS KV key prefixes based on the current catalog.
 * e.g., ["redis.>", "postgres.>", "nemo_metadata.>"]
 */
export async function getCatalogPrefixes(): Promise<string[]> {
  const templates = await getCatalogTemplates();
  const prefixes = templates.map(t => `${t.id}.>`);
  prefixes.push("nemo_metadata.>"); // Always allow internal metadata
  return prefixes;
}
