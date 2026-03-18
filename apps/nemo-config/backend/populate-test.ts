import { connect, StringCodec } from "nats";

async function run() {
  const natsUrl = "nats://10.7.0.4:4222"; 
  const nc = await connect({ servers: natsUrl });
  const js = nc.jetstream();
  const sc = StringCodec();
  
  // Create bucket and add a value
  const kv = await js.views.kv("nemo_config", { history: 1 });
  await kv.put("redis.url", sc.encode("redis://10.7.0.4:6379"));
  await kv.put("unknown.service", sc.encode("should not fetch this"));
  await kv.put("nemo_metadata.redis_last_host", sc.encode("10.7.0.4"));
  
  console.log("Bucket populated.");
  await nc.drain();
}
run();