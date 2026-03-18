import * as yaml from "js-yaml";
import { readFile } from "fs/promises";
const content = await readFile("../templates/postgres.yaml", "utf-8");
const parsed = yaml.load(content);
console.log(JSON.stringify(parsed, null, 2));
