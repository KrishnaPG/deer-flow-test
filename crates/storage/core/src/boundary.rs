/// Documents the LiveKit media bypass boundary outside normal storage ingress.
pub fn livekit_bypass_note() -> &'static str {
    "LiveKit media may bypass the storage service and write directly to landing zones before later hash/promote into canonical truth."
}
