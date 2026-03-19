import { readdir, readFile } from "fs/promises";
import * as yaml from "js-yaml";
import { resolve } from "path";
import { type Template } from "./deployer";
import { CONSUL_PREFIX } from "../../schema";

const TEMPLATE_DIR = resolve(import.meta.dir, "../../templates");
console.log(`[CATALOG] Loading templates from: ${TEMPLATE_DIR}`);

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
 * Returns prefixes for fetching ALL configs from Consul.
 * Fetches both service configs and metadata.
 */
export async function getCatalogPrefixes(): Promise<string[]> {
  return [
    `${CONSUL_PREFIX}/`,
    `${CONSUL_PREFIX}/metadata/`,
  ];
}
