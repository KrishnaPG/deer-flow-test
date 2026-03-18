import { connect } from "nats";

async function run() {
  const natsUrl = "nats://10.7.0.4:4222"; 
  const nc = await connect({ servers: natsUrl });
  const js = nc.jetstream();
  
  const kv = await js.views.kv("nemo_config");
  const keysIter = await kv.keys(["redis.>", "nemo_metadata.>"]);
  for await (const key of keysIter) {
    console.log("Found key:", key);
  }
  await nc.drain();
}
run();