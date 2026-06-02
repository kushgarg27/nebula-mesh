use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    pub protocol_version: u16,
    pub seq: u64,
    pub timestamp_us: u64,
    pub origin: String,
    pub session_id: Option<Uuid>,
    pub payload: Payload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Payload {
    Hello(Hello),
    Pair(Pair),
    Layout(Layout),
    EnterRemote(EnterRemote),
    ExitRemote(ExitRemote),
    MouseMove(MouseMove),
    MouseBtn(MouseBtn),
    MouseScroll(MouseScroll),
    Key(KeyEvent),
    ClipboardText(ClipboardText),
    Ping(Ping),
    Pong(Pong),
    Ack(Ack),
    Error(ErrorMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hello {
    pub device_id: String,
    pub host_name: String,
    pub os: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pair {
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layout {
    pub active_controller: String,
    pub devices: Vec<LayoutDevice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutDevice {
    pub id: String,
    pub edges: Edges,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Edges {
    pub left: Option<String>,
    pub right: Option<String>,
    pub top: Option<String>,
    pub bottom: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterRemote {
    pub target_device_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitRemote {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseMove {
    pub dx: i32,
    pub dy: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseBtn {
    pub button: u8,
    pub is_down: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseScroll {
    pub delta_x: i32,
    pub delta_y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEvent {
    pub key_code: u32,
    pub is_down: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardText {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ping {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pong {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ack {
    pub ack_seq: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMessage {
    pub message: String,
}
