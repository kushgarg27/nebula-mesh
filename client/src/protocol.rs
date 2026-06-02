use protocol::{Envelope, Payload};

pub const PROTOCOL_VERSION: u16 = 1;

pub fn make_envelope(seq: u64, origin: String, payload: Payload) -> Envelope {
    Envelope {
        protocol_version: PROTOCOL_VERSION,
        seq,
        timestamp_us: timestamp_us(),
        origin,
        session_id: None,
        payload,
    }
}

fn timestamp_us() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as u64)
        .unwrap_or(0)
}
