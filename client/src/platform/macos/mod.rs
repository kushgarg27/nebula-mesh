use anyhow::Result;
use protocol::{KeyEvent, MouseBtn, MouseMove, MouseScroll, Payload};

#[derive(Default)]
pub struct InputEngine {
    suppress_until_us: u64,
}

impl InputEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn capture_tick(&mut self) -> Vec<Payload> {
        // TODO: replace with CGEventTap listener integration.
        Vec::new()
    }

    pub fn inject(&mut self, payload: &Payload) -> Result<()> {
        // TODO: replace with CGEventPost integration.
        match payload {
            Payload::MouseMove(MouseMove { .. })
            | Payload::MouseBtn(MouseBtn { .. })
            | Payload::MouseScroll(MouseScroll { .. })
            | Payload::Key(KeyEvent { .. }) => {
                self.suppress_until_us = now_us() + 10_000;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub fn is_suppressed(&self) -> bool {
        now_us() < self.suppress_until_us
    }
}

fn now_us() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as u64)
        .unwrap_or(0)
}
