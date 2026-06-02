use anyhow::Result;
use protocol::Payload;
use tokio_tungstenite::tungstenite::Message;

use crate::session::SharedState;

pub async fn route_payload(
    state: &SharedState,
    source_device_id: &str,
    serialized: String,
    payload: &Payload,
) -> Result<()> {
    let target = {
        let guard = state.read().await;
        match payload {
            Payload::EnterRemote(data) => Some(data.target_device_id.clone()),
            Payload::ExitRemote(_) => guard.active_controller.clone(),
            Payload::MouseMove(_)
            | Payload::MouseBtn(_)
            | Payload::MouseScroll(_)
            | Payload::Key(_)
            | Payload::ClipboardText(_) => guard.active_target.clone(),
            _ => None,
        }
    };

    if let Some(target_device_id) = target {
        let tx = {
            let guard = state.read().await;
            guard.routes.get(&target_device_id).cloned()
        };
        if let Some(sender) = tx {
            let _ = sender.send(Message::Text(serialized));
        }
    }

    if matches!(payload, Payload::EnterRemote(_)) {
        let mut guard = state.write().await;
        guard.active_controller = Some(source_device_id.to_string());
        if let Payload::EnterRemote(data) = payload {
            guard.active_target = Some(data.target_device_id.clone());
        }
    } else if matches!(payload, Payload::ExitRemote(_)) {
        let mut guard = state.write().await;
        guard.active_target = None;
    }

    Ok(())
}
