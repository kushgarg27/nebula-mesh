use protocol::{ClipboardText, Payload};
use std::time::{Duration, Instant};

pub struct ClipboardSync {
    last_sent: Instant,
    debounce: Duration,
    last_payload: String,
}

impl ClipboardSync {
    pub fn new(debounce_ms: u64) -> Self {
        Self {
            last_sent: Instant::now() - Duration::from_millis(debounce_ms),
            debounce: Duration::from_millis(debounce_ms),
            last_payload: String::new(),
        }
    }

    pub fn maybe_emit(&mut self, current_text: String) -> Option<Payload> {
        if current_text.is_empty() || current_text == self.last_payload {
            return None;
        }
        if self.last_sent.elapsed() < self.debounce {
            return None;
        }
        self.last_payload = current_text.clone();
        self.last_sent = Instant::now();
        Some(Payload::ClipboardText(ClipboardText { text: current_text }))
    }
}
