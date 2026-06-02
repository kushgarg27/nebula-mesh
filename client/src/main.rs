mod clipboard;
mod edge;
mod protocol;

#[cfg(target_os = "macos")]
mod platform {
    pub mod macos;
    pub use macos::InputEngine;
}
#[cfg(target_os = "windows")]
mod platform {
    pub mod windows;
    pub use windows::InputEngine;
}

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use protocol::{Edges, Hello, Pair, Payload, Ping};
use tokio::sync::mpsc;
use tokio::time::{Duration, interval};
use tokio_tungstenite::tungstenite::Message;

use crate::clipboard::ClipboardSync;
use crate::edge::{CursorPosition, detect_edge_transition, exit_remote};
use crate::protocol::make_envelope;

#[tokio::main]
async fn main() -> Result<()> {
    let broker_url =
        std::env::var("NEBULA_BROKER").unwrap_or_else(|_| "ws://127.0.0.1:24800".into());
    let device_id = std::env::var("NEBULA_DEVICE_ID").unwrap_or_else(|_| "local-device".into());
    let pairing_code = std::env::var("NEBULA_PAIR_CODE").unwrap_or_else(|_| "123456".into());
    let reconnect_ms = std::env::var("NEBULA_RECONNECT_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(800);

    loop {
        let conn = tokio_tungstenite::connect_async(&broker_url).await;
        if conn.is_err() {
            tokio::time::sleep(Duration::from_millis(reconnect_ms)).await;
            continue;
        }
        let (socket, _) = conn?;
        let (mut ws_tx, mut ws_rx) = socket.split();
        let (send_tx, mut send_rx) = mpsc::unbounded_channel::<Payload>();
        let write_device_id = device_id.clone();

        let writer = tokio::spawn(async move {
            let mut seq = 1_u64;
            while let Some(payload) = send_rx.recv().await {
                let envelope = make_envelope(seq, write_device_id.clone(), payload);
                seq += 1;
                if let Ok(serialized) = serde_json::to_string(&envelope) {
                    if ws_tx.send(Message::Text(serialized)).await.is_err() {
                        break;
                    }
                }
            }
        });

        send_tx.send(Payload::Hello(Hello {
            device_id: device_id.clone(),
            host_name: hostname(),
            os: std::env::consts::OS.into(),
            capabilities: vec![
                "mouse".into(),
                "keyboard".into(),
                "clipboard-text".into(),
                "edge-switch".into(),
            ],
        }))?;
        send_tx.send(Payload::Pair(Pair {
            code: pairing_code.clone(),
        }))?;

        let mut engine = platform::InputEngine::new();
        let mut clip = ClipboardSync::new(250);
        let mut tick = interval(Duration::from_millis(8));
        let mut ping = interval(Duration::from_secs(1));
        let mut edges = Edges::default();
        let mut in_remote_mode = false;
        let deadzone = 2;

        let disconnected = loop {
            tokio::select! {
                _ = tick.tick() => {
                    if !engine.is_suppressed() {
                        for payload in engine.capture_tick() {
                            let _ = send_tx.send(payload);
                        }
                    }
                    if !in_remote_mode {
                        let pos = cursor_position();
                        if let Some(payload) = detect_edge_transition(&pos, &edges, deadzone) {
                            in_remote_mode = true;
                            let _ = send_tx.send(payload);
                        }
                    } else if should_exit_remote() {
                        in_remote_mode = false;
                        let _ = send_tx.send(exit_remote());
                    }
                    if let Some(payload) = clip.maybe_emit(read_clipboard_text()) {
                        let _ = send_tx.send(payload);
                    }
                }
                _ = ping.tick() => {
                    let _ = send_tx.send(Payload::Ping(Ping{}));
                }
                maybe_msg = ws_rx.next() => {
                    if let Some(msg) = maybe_msg {
                        let msg = msg?;
                        if let Message::Text(text) = msg {
                            if let Ok(envelope) = serde_json::from_str::<protocol::Envelope>(&text) {
                                match envelope.payload {
                                    Payload::Layout(layout) => {
                                        if let Some(d) = layout.devices.into_iter().find(|d| d.id == device_id) {
                                            edges = d.edges;
                                        }
                                    }
                                    Payload::MouseMove(_)
                                    | Payload::MouseBtn(_)
                                    | Payload::MouseScroll(_)
                                    | Payload::Key(_) => {
                                        let _ = engine.inject(&envelope.payload);
                                    }
                                    Payload::ClipboardText(data) => set_clipboard_text(&data.text),
                                    _ => {}
                                }
                            }
                        }
                    } else {
                        break true;
                    }
                }
            }
        };

        writer.abort();
        if disconnected {
            tokio::time::sleep(Duration::from_millis(reconnect_ms)).await;
            continue;
        }
    }
}

fn hostname() -> String {
    std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "unknown-host".to_string())
}

fn read_clipboard_text() -> String {
    // Placeholder until OS-specific clipboard adapters are added.
    String::new()
}

fn set_clipboard_text(_text: &str) {
    // Placeholder until OS-specific clipboard adapters are added.
}

fn cursor_position() -> CursorPosition {
    // Placeholder for edge detection until native cursor getters are wired.
    CursorPosition {
        x: 100,
        y: 100,
        width: 1920,
        height: 1080,
    }
}

fn should_exit_remote() -> bool {
    // Placeholder for hotkey based exit.
    false
}
