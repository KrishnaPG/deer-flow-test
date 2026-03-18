import { readFile } from 'fs/promises';
import * as yaml from 'js-yaml';

const content = await readFile('../templates/redis.yaml', 'utf-8');
const parsed = yaml.load(content);
console.log("docker_compose repr:", JSON.stringify(parsed.docker_compose).substring(0, 200));
console.log("First chars:", parsed.docker_compose.substring(0, 50).replace(/\n/g, "\\n"));
