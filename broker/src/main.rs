mod pairing;
mod router;
mod session;

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use protocol::{Ack, Envelope, ErrorMessage, Payload, Pong};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

use crate::pairing::resolve_pairing_code;
use crate::router::route_payload;
use crate::session::{Device, SharedState, default_layout, new_state};

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "0.0.0.0:24800";
    let listener = TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind broker on {addr}"))?;
    let state = new_state();

    println!("nebula broker listening on ws://{addr}");

    loop {
        let (stream, _) = listener.accept().await?;
        let state = state.clone();
        tokio::spawn(async move {
            if let Err(err) = handle_connection(stream, state).await {
                eprintln!("connection error: {err:?}");
            }
        });
    }
}

async fn handle_connection(stream: TcpStream, state: SharedState) -> Result<()> {
    let ws_stream = tokio_tungstenite::accept_async(stream).await?;
    let (mut ws_tx, mut ws_rx) = ws_stream.split();
    let (route_tx, mut route_rx) = mpsc::unbounded_channel::<Message>();

    let mut device_id: Option<String> = None;
    let writer = tokio::spawn(async move {
        while let Some(msg) = route_rx.recv().await {
            if ws_tx.send(msg).await.is_err() {
                break;
            }
        }
    });

    while let Some(msg) = ws_rx.next().await {
        let msg = msg?;
        if let Message::Text(text) = msg {
            let envelope: Envelope = match serde_json::from_str(&text) {
                Ok(parsed) => parsed,
                Err(_) => continue,
            };

            match &envelope.payload {
                Payload::Hello(hello) => {
                    device_id = Some(hello.device_id.clone());
                    let mut guard = state.write().await;
                    guard.routes.insert(hello.device_id.clone(), route_tx.clone());
                    guard.devices.insert(
                        hello.device_id.clone(),
                        Device {
                            device_id: hello.device_id.clone(),
                            os: hello.os.clone(),
                            session_id: None,
                        },
                    );
                }
                Payload::Pair(pair) => {
                    let resolved = resolve_pairing_code(&state, &pair.code).await;
                    match (resolved, &device_id) {
                        (Some(session_id), Some(id)) => {
                            let mut guard = state.write().await;
                            if let Some(device) = guard.devices.get_mut(id) {
                                device.session_id = Some(session_id);
                            }
                            let ack = Envelope {
                                protocol_version: envelope.protocol_version,
                                seq: envelope.seq,
                                timestamp_us: envelope.timestamp_us,
                                origin: "broker".to_string(),
                                session_id: Some(session_id),
                                payload: Payload::Ack(Ack {
                                    ack_seq: envelope.seq,
                                }),
                            };
                            let _ = route_tx.send(Message::Text(serde_json::to_string(&ack)?));
                            broadcast_layout(&state).await?;
                        }
                        _ => {
                            let err = Envelope {
                                protocol_version: envelope.protocol_version,
                                seq: envelope.seq,
                                timestamp_us: envelope.timestamp_us,
                                origin: "broker".to_string(),
                                session_id: None,
                                payload: Payload::Error(ErrorMessage {
                                    message: "pairing failed".to_string(),
                                }),
                            };
                            let _ = route_tx.send(Message::Text(serde_json::to_string(&err)?));
                        }
                    }
                }
                Payload::Ping(_) => {
                    let pong = Envelope {
                        protocol_version: envelope.protocol_version,
                        seq: envelope.seq,
                        timestamp_us: envelope.timestamp_us,
                        origin: "broker".to_string(),
                        session_id: envelope.session_id,
                        payload: Payload::Pong(Pong {}),
                    };
                    let _ = route_tx.send(Message::Text(serde_json::to_string(&pong)?));
                }
                _ => {
                    if let Some(id) = &device_id {
                        route_payload(&state, id, text.clone(), &envelope.payload).await?;
                    }
                }
            }
        }
    }

    if let Some(id) = &device_id {
        let mut guard = state.write().await;
        guard.routes.remove(id);
        guard.devices.remove(id);
        if guard.active_controller.as_ref() == Some(id) {
            guard.active_controller = None;
        }
        if guard.active_target.as_ref() == Some(id) {
            guard.active_target = None;
        }
    }

    writer.abort();
    Ok(())
}

async fn broadcast_layout(state: &SharedState) -> Result<()> {
    let layout = default_layout();
    let envelope = Envelope {
        protocol_version: 1,
        seq: 0,
        timestamp_us: 0,
        origin: "broker".to_string(),
        session_id: None,
        payload: Payload::Layout(layout),
    };
    let serialized = serde_json::to_string(&envelope)?;
    let routes = {
        let guard = state.read().await;
        guard.routes.values().cloned().collect::<Vec<_>>()
    };
    for tx in routes {
        let _ = tx.send(Message::Text(serialized.clone()));
    }
    Ok(())
}
