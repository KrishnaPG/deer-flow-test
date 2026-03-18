import { connect, StringCodec } from "nats";
const sc = StringCodec();
const nc = await connect({ servers: "nats://10.7.0.4:4222" });
const js = nc.jetstream();
const kv = await js.views.kv("nemo_config");
const keys = await kv.keys();
for await (const k of keys) {
    const entry = await kv.get(k);
    console.log(`${k} = ${entry ? sc.decode(entry.value) : "null"}`);
}
await nc.drain();
