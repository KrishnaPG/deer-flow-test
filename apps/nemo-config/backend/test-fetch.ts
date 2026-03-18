import { getAllConfigFromNats } from "./src/deployer";
import { getCatalogPrefixes } from "./src/catalog";

async function run() {
  const natsUrl = "nats://10.7.0.4:4222";
  const prefixes = await getCatalogPrefixes();
  console.log("Allowed prefixes:", prefixes);
  const configs = await getAllConfigFromNats(natsUrl, prefixes);
  console.log("Configs:", configs);
}
run();