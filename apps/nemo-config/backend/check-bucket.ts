import { connect } from "nats";
const nc = await connect({ servers: "nats://10.7.0.4:4222" });
const js = nc.jetstream();
const jsm = await nc.jetstreamManager();
const bucket = "nemo_config";
try {
    const status = await jsm.streams.info(`KV_${bucket}`);
    console.log(JSON.stringify(status, null, 2));
} catch (e) {
    console.log("Bucket not found or error:", e);
}
await nc.drain();
