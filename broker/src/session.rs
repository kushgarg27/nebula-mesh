use std::collections::HashMap;
use std::sync::Arc;

use protocol::{Edges, Layout, LayoutDevice};
use tokio::sync::RwLock;
use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct Device {
    pub device_id: String,
    pub os: String,
    pub session_id: Option<Uuid>,
}

#[derive(Default)]
pub struct BrokerState {
    pub routes: HashMap<String, UnboundedSender<Message>>,
    pub devices: HashMap<String, Device>,
    pub pairing_codes: HashMap<String, Uuid>,
    pub active_controller: Option<String>,
    pub active_target: Option<String>,
}

pub type SharedState = Arc<RwLock<BrokerState>>;

pub fn new_state() -> SharedState {
    let mut state = BrokerState::default();
    // Static MVP pairing code; can be rotated via config later.
    state.pairing_codes.insert("123456".to_string(), Uuid::new_v4());
    Arc::new(RwLock::new(state))
}

pub fn default_layout() -> Layout {
    Layout {
        active_controller: "gaming-pc".to_string(),
        devices: vec![
            LayoutDevice {
                id: "gaming-pc".to_string(),
                edges: Edges {
                    left: Some("mac-air".to_string()),
                    right: Some("mac-pro".to_string()),
                    top: None,
                    bottom: Some("ideapad".to_string()),
                },
            },
            LayoutDevice {
                id: "ideapad".to_string(),
                edges: Edges {
                    left: None,
                    right: None,
                    top: Some("gaming-pc".to_string()),
                    bottom: None,
                },
            },
            LayoutDevice {
                id: "mac-air".to_string(),
                edges: Edges {
                    left: None,
                    right: Some("gaming-pc".to_string()),
                    top: None,
                    bottom: None,
                },
            },
            LayoutDevice {
                id: "mac-pro".to_string(),
                edges: Edges {
                    left: Some("gaming-pc".to_string()),
                    right: None,
                    top: None,
                    bottom: None,
                },
            },
        ],
    }
}
