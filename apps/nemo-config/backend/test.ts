import { getAllConfigFromNats } from "./src/deployer";
import { connect, StringCodec } from "nats";

async function run() {
  const natsUrl = "nats://10.7.0.4:4222"; // the URL from the user prompt
  const configs = await getAllConfigFromNats(natsUrl);
  console.log("Configs:", configs);
}
run();